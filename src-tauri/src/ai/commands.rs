use std::sync::{Arc, RwLock};

use futures::StreamExt;
use rig::message::Message;
use tauri::ipc::Channel;

use crate::ai::state::{
    add_message_to_history, build_session_agent, ensure_session_title, restore_session, HistoryItem,
    Session,
};

use super::agents::ChatEvent;
use super::config::{available_models, default_model, ModelInfo, Provider};
use super::db::{self, SessionMeta};
use super::state::ChatState;
use tauri::{Emitter, Manager};

#[tauri::command(rename_all = "snake_case")]
pub fn list_models(provider: Provider) -> Vec<ModelInfo> {
    available_models(provider).to_vec()
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_session(app: tauri::AppHandle) -> Result<(String, Session), String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let mut guard = state.write().unwrap();
    let api_keys = guard.config.api_keys.clone();

    let session_id = uuid::Uuid::new_v4().to_string();

    let (provider, model, preset_id) = guard
        .sessions
        .values()
        .max_by(|a, b| a.update_at.cmp(&b.update_at))
        .map(|s| (s.provider, s.model.clone(), s.preset_id.clone()))
        .unwrap_or_else(|| {
            let p = Provider::DeepSeek;
            (
                p,
                crate::ai::config::default_model(p).to_string(),
                "assistant".to_string(),
            )
        });

    let agent = build_session_agent(&api_keys, provider, &model, &preset_id)?;

    let session = Session {
        session_id: session_id.clone(),
        provider,
        model: model.clone(),
        preset_id: preset_id.clone(),
        agent,
        history: Vec::new(),
        created_at: std::time::SystemTime::now(),
        update_at: std::time::SystemTime::now(),
        title: "New Session".into(),
        is_loading: false,
        cancel_handle: None,
    };

    db::insert_session(&guard.db, &session_id, provider, &model, &preset_id, session.created_at, session.update_at, &session.title)?;
    guard.sessions.insert(session_id.clone(), session.clone());
    Ok((session_id, session))
}


#[tauri::command(rename_all = "snake_case")]
pub fn close_session(app: tauri::AppHandle, session_id: String) -> Result<bool, String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let mut guard = state.write().unwrap();
    
    
    if let Some(sess) = guard.sessions.get(&session_id) {
        if let Some(handle) = &sess.cancel_handle {
            handle.abort();
        }
    }
    let removed = guard.sessions.remove(&session_id).is_some();
    
    db::set_session_archived(&guard.db, &session_id)?;
    Ok(removed)
}


#[tauri::command(rename_all = "snake_case")]
pub fn switch_provider(
    app: tauri::AppHandle,
    session_id: String,
    provider: Provider,
    api_key: Option<String>,
) -> Result<(), String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let mut guard = state.write().unwrap();
    if let Some(key) = api_key {
        guard.config.api_keys.insert(provider, key);
    }

    
    let model = default_model(provider).to_string();

    
    let api_keys = guard.config.api_keys.clone();
    let preset_id = guard
        .sessions
        .get(&session_id)
        .map(|s| s.preset_id.clone())
        .unwrap_or_else(|| "assistant".to_string());

    
    let agent = build_session_agent(&api_keys, provider, &model, &preset_id)?;

    let update_at = std::time::SystemTime::now();
    {
        let sess = guard
            .sessions
            .get_mut(&session_id)
            .ok_or("Session not found, please call create_session first")?;
        sess.provider = provider;
        sess.model = model.clone();
        sess.agent = agent;
        sess.update_at = update_at;
    }
    db::update_session(&guard.db, &session_id, provider, &model, update_at)?;

    Ok(())
}


#[tauri::command(rename_all = "snake_case")]
pub fn switch_model(
    app: tauri::AppHandle,
    session_id: String,
    model: String,
) -> Result<(), String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let mut guard = state.write().unwrap();

    
    let (provider, preset_id) = {
        let sess = guard
            .sessions
            .get(&session_id)
            .ok_or("Session not found, please call create_session first")?;
        (sess.provider, sess.preset_id.clone())
    };

    
    let valid = available_models(provider).iter().any(|m| m.id == model);
    if !valid {
        return Err(format!("{model} does not belong to the current session's provider"));
    }

    
    let api_keys = guard.config.api_keys.clone();
    let agent = build_session_agent(&api_keys, provider, &model, &preset_id)?;

    let update_at = std::time::SystemTime::now();
    {
        let sess = guard.sessions.get_mut(&session_id).unwrap();
        sess.model = model.clone();
        sess.agent = agent;
        sess.update_at = update_at;
    }
    db::update_session(&guard.db, &session_id, provider, &model, update_at)?;

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn send_message(
    app: tauri::AppHandle,
    on_event: Channel<ChatEvent>,
    prompt: HistoryItem,
    session_id: String,
) -> Result<(), String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    
    let (agent, history) = {
        let guard = state.read().unwrap();
        let sess = guard
            .sessions
            .get(&session_id)
            .ok_or("Session not found, please call create_session first")?;
        (sess.agent.clone(), sess.history.clone())
    };

    
    let prompt_msg: Message = prompt.message.clone();

    
    let (abort_handle, abort_registration) = futures::future::AbortHandle::new_pair();
    {
        let mut guard = state.write().unwrap();
        let db = guard.db.clone();
        let sess = guard
            .sessions
            .get_mut(&session_id)
            .ok_or("Session not found, please call create_session first")?;
        if sess.is_loading {
            return Err("Session is currently generating a response (loading), adding new messages is temporarily disabled".into());
        }
        db::insert_message(&db, &session_id, &prompt)?;
        sess.history.push(prompt.clone());
        // 首次发送消息时，用第一条用户消息作为会话标题
        ensure_session_title(&db, &session_id, sess);
        sess.is_loading = true;
        sess.cancel_handle = Some(abort_handle);
        sess.update_at = std::time::SystemTime::now();
    }

    
    let prompt: Message = prompt_msg;

    let history: Vec<Message> = history.iter().map(|h| h.message.clone()).collect();
    let stream = agent.stream_chat(prompt, history).await;
    
    let mut stream = futures::stream::Abortable::new(stream, abort_registration);

    let mut full_text = String::new();
    while let Some(item) = stream.next().await {
        match item {
            Ok(ChatEvent::TextDelta(text)) => {
                if text.is_empty() {
                    continue;
                }
                full_text.push_str(&text);
                
                if on_event.send(ChatEvent::TextDelta(text)).is_err() {
                    
                    break;
                }
            }
            Ok(ChatEvent::ToolCall { name, arguments }) => {
                if on_event
                    .send(ChatEvent::ToolCall { name, arguments })
                    .is_err()
                {
                    break;
                }
            }
            Ok(ChatEvent::ToolCallDelta(s)) => {
                if on_event.send(ChatEvent::ToolCallDelta(s)).is_err() {
                    break;
                }
            }
            Ok(ChatEvent::Reasoning(text)) => {
                if on_event.send(ChatEvent::Reasoning(text)).is_err() {
                    break;
                }
            }
            Ok(ChatEvent::Done) => {
                
                let _ = on_event.send(ChatEvent::Done);
                break;
            }
            Err(e) => {
                
                let mut guard = state.write().unwrap();
                if let Some(sess) = guard.sessions.get_mut(&session_id) {
                    sess.is_loading = false;
                    sess.cancel_handle = None;
                    sess.update_at = std::time::SystemTime::now();
                }
                return Err(e);
            }
        }
    }
    let was_cancelled = stream.is_aborted();
    if was_cancelled {
        let _ = on_event.send(ChatEvent::Done);
    }

    
    {
        let mut guard = state.write().unwrap();
        if let Some(sess) = guard.sessions.get_mut(&session_id) {
            
            if sess.history.is_empty() {
                sess.title = "".to_string();
            }
            sess.is_loading = false;
            sess.cancel_handle = None;
            sess.update_at = std::time::SystemTime::now();
        }
    }
    
    
    if !full_text.is_empty() {
        let item = HistoryItem {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: std::time::SystemTime::now(),
            message: Message::assistant(full_text.as_str()), 
        };
        {
            let state = app.state::<Arc<RwLock<ChatState>>>();
            let guard = state.read().unwrap();
            db::insert_message(&guard.db, &session_id, &item)?;
        }
        add_message_to_history(&app, session_id.clone(), item)?;
    }
    // 通知前端本次消息已生成完毕，前端会重新拉取历史
    let _ = app.emit_to("index", &format!("on_message_done{session_id}"), ());
    Ok(())
}


#[tauri::command(rename_all = "snake_case")]
pub fn stop_generation(app: tauri::AppHandle, session_id: String) -> Result<(), String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let guard = state.read().unwrap();
    let sess = guard.sessions.get(&session_id).ok_or("Session not found")?;
    match &sess.cancel_handle {
        Some(handle) => {
            handle.abort();
            Ok(())
        }
        None => {
            
            
            if sess.is_loading {
                Err("Generation task not yet initialized, please try again later".into())
            } else {
                Err("No generation currently in progress".into())
            }
        }
    }
}


#[tauri::command(rename_all = "snake_case")]
pub fn list_sessions(app: tauri::AppHandle) -> Vec<Session> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let guard = state.read().unwrap();
    let mut list: Vec<Session> = guard.sessions.values().cloned().collect();
    list.sort_by_key(|a| a.update_at);
    list
}


/// 列出 DB 中所有历史会话的元数据（轻量，不载入内存、不重建 agent）。
#[tauri::command(rename_all = "snake_case")]
pub fn list_history_sessions(app: tauri::AppHandle) -> Result<Vec<SessionMeta>, String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let guard = state.read().unwrap();
    let metas = db::list_session_meta(&guard.db)?;
    Ok(metas
        .into_iter()
        .filter(|m| !guard.sessions.contains_key(&m.session_id))
        .collect())
}


/// 用户手动打开一个历史会话：从 DB 恢复并载入内存。已在内存中则直接返回。
#[tauri::command(rename_all = "snake_case")]
pub fn open_session(app: tauri::AppHandle, session_id: String) -> Result<Session, String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let mut guard = state.write().unwrap();
    let sess = if let Some(sess) = guard.sessions.get(&session_id) {
        sess.clone()
    } else {
        let meta = db::get_session_meta(&guard.db, &session_id)?
            .ok_or("Session not found in database")?;
        restore_session(&mut guard, &meta)?
    };
    let _ = app.emit("on_open_session_with_session_id", session_id);
    Ok(sess)
}


#[tauri::command(rename_all = "snake_case")]
pub fn get_history(app: tauri::AppHandle, session_id: String) -> Result<Vec<HistoryItem>, String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let guard = state.read().unwrap();
    if let Some(sess) = guard.sessions.get(&session_id) {
        return Ok(sess.history.clone());
    }
    let db = guard.db.clone();
    drop(guard);
    db::get_history(&db, &session_id)
}


#[tauri::command(rename_all = "snake_case")]
pub fn remove_history_item(
    app: tauri::AppHandle,
    session_id: String,
    history_id: String,
) -> Result<(), String> {
    crate::ai::state::remove_history_item(&app, session_id, history_id)
}
