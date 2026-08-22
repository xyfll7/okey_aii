use std::sync::{Arc, RwLock};

use futures::StreamExt;
use rig::message::Message;
use tauri::ipc::Channel;

use crate::ai::state::{add_message_to_history, build_session_agent, HistoryItem, Session};

use super::agents::ChatEvent;
use super::config::{available_models, default_model, ModelInfo, Provider};
use super::state::ChatState;
use tauri::Manager;

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
        .max_by(|a, b| a.created_at.cmp(&b.created_at))
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
        model,
        preset_id,
        agent,
        history: Vec::new(),
        created_at: std::time::SystemTime::now(),
        title: "New Session".into(),
        is_loading: false,
        cancel_handle: None,
    };

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

    let sess = guard
        .sessions
        .get_mut(&session_id)
        .ok_or("Session not found, please call create_session first")?;
    sess.provider = provider;
    sess.model = model.clone();
    sess.agent = agent;

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

    let sess = guard.sessions.get_mut(&session_id).unwrap();
    sess.model = model.clone();
    sess.agent = agent;

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
        let sess = guard
            .sessions
            .get_mut(&session_id)
            .ok_or("Session not found, please call create_session first")?;
        if sess.is_loading {
            return Err("Session is currently generating a response (loading), adding new messages is temporarily disabled".into());
        }
        sess.history.push(prompt.clone());
        sess.is_loading = true;
        sess.cancel_handle = Some(abort_handle);
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
        }
    }
    
    
    if !full_text.is_empty() {
        add_message_to_history(
            &app,
            session_id,
            HistoryItem {
                id: uuid::Uuid::new_v4().to_string(),
                created_at: std::time::SystemTime::now(),
                message: Message::assistant(full_text.as_str()), 
            },
        )?;
    }
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
    list.sort_by_key(|a| a.created_at);
    list
}


#[tauri::command(rename_all = "snake_case")]
pub fn clear_history(app: tauri::AppHandle, session_id: String) -> Result<(), String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let mut guard = state.write().unwrap();
    guard
        .sessions
        .get_mut(&session_id)
        .ok_or("Session not found")?
        .history
        .clear();
    Ok(())
}


#[tauri::command(rename_all = "snake_case")]
pub fn get_history(app: tauri::AppHandle, session_id: String) -> Result<Vec<HistoryItem>, String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let guard = state.read().unwrap();
    let sess = guard.sessions.get(&session_id).ok_or("Session not found")?;
    Ok(sess.history.clone())
}


#[tauri::command(rename_all = "snake_case")]
pub fn remove_history_item(
    app: tauri::AppHandle,
    session_id: String,
    history_id: String,
) -> Result<(), String> {
    crate::ai::state::remove_history_item(&app, session_id, history_id)
}
