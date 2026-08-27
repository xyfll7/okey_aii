use std::sync::{Arc, RwLock};

use futures::StreamExt;
use rig::message::Message;
use tauri::ipc::Channel;

use crate::ai::state::{
    add_message_to_history, build_session_agent, ensure_session_title, restore_session,
    HistoryItem, Session,
};

use super::agents::ChatEvent;
use super::config::{available_models, default_model, ModelInfo, Provider};
use super::db::{self, SessionMeta};
use super::state::ChatState;
use crate::my_windows::window_index::should_use_existing_index_window;
use crate::store::app_state::AppConfigState;
use tauri::{Emitter, Manager};

#[tauri::command(rename_all = "snake_case")]
pub fn list_models(provider: Provider) -> Vec<ModelInfo> {
    available_models(provider).to_vec()
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_session(app: tauri::AppHandle) -> Result<(String, Session), String> {
    let app_config_state = app.state::<AppConfigState>();
    let (api_keys, agent_presets) = {
        let config = app_config_state.read();
        (config.api_keys.clone(), config.agent_presets.clone())
    };

    let state = app.state::<Arc<RwLock<ChatState>>>();
    let mut guard = state.write().unwrap();

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

    let agent = build_session_agent(&api_keys, provider, &model, &preset_id, &agent_presets)?;

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

    // 延迟持久化：此处仅放入内存，不写入 DB。
    // 只有当会话真正发送了第一条消息（见 send_message）才会落库，
    // 避免"创建后未聊天"的空会话在重启后出现在历史记录中。
    guard.sessions.insert(session_id.clone(), session.clone());
    Ok((session_id, session))
}

/// 新建会话（主窗口"新建会话"按钮）：
/// 若所有已加载会话的历史均为空，则不做任何操作；
/// 否则清理（中止生成、从内存移除并归档 DB）所有非空会话，
/// 然后创建一个全新的空会话，仅返回新会话 id。
#[tauri::command(rename_all = "snake_case")]
pub fn new_session(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let to_clear: Vec<String> = {
        let guard = state.read().unwrap();
        guard
            .sessions
            .values()
            .filter(|s| !s.history.is_empty())
            .map(|s| s.session_id.clone())
            .collect()
    };
    // 当前会话无历史数据 → 什么也不做
    if to_clear.is_empty() {
        return Ok(None);
    }
    for session_id in to_clear {
        close_session(app.clone(), session_id)?;
    }
    let (session_id, _) = create_session(app)?;
    Ok(Some(session_id))
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

/// 永久删除一个会话：从内存移除（若已载入），并从 DB 级联删除会话及其消息。
#[tauri::command(rename_all = "snake_case")]
pub fn delete_session(app: tauri::AppHandle, session_id: String) -> Result<(), String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let mut guard = state.write().unwrap();
    if let Some(sess) = guard.sessions.get(&session_id) {
        if let Some(handle) = &sess.cancel_handle {
            handle.abort();
        }
    }
    guard.sessions.remove(&session_id);
    db::delete_session(&guard.db, &session_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn switch_provider(
    app: tauri::AppHandle,
    session_id: String,
    provider: Provider,
    api_key: Option<String>,
) -> Result<(), String> {
    if let Some(key) = api_key {
        let app_config_state = app.state::<AppConfigState>();
        app_config_state
            .update_and_save(|config| {
                config.api_keys.insert(provider, key);
            })
            .map_err(|e| e.to_string())?;
    }

    let app_config_state = app.state::<AppConfigState>();
    let (api_keys, agent_presets) = {
        let config = app_config_state.read();
        (config.api_keys.clone(), config.agent_presets.clone())
    };

    let state = app.state::<Arc<RwLock<ChatState>>>();
    let mut guard = state.write().unwrap();

    let model = default_model(provider).to_string();

    let preset_id = guard
        .sessions
        .get(&session_id)
        .map(|s| s.preset_id.clone())
        .unwrap_or_else(|| "assistant".to_string());

    let agent = build_session_agent(&api_keys, provider, &model, &preset_id, &agent_presets)?;

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
    let app_config_state = app.state::<AppConfigState>();
    let (api_keys, agent_presets) = {
        let config = app_config_state.read();
        (config.api_keys.clone(), config.agent_presets.clone())
    };

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
        return Err(format!(
            "{model} does not belong to the current session's provider"
        ));
    }

    let agent = build_session_agent(&api_keys, provider, &model, &preset_id, &agent_presets)?;

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
pub fn assemble_prompt(app: tauri::AppHandle, item: HistoryItem) -> HistoryItem {
	crate::utils::assemble_prompt_item::assemble_prompt_item(&app, item)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn send_message(
    app: tauri::AppHandle,
    on_event: Channel<ChatEvent>,
    prompt: HistoryItem,
    session_id: String,
) -> Result<(), String> {
    let should_emit_done = !should_use_existing_index_window(app.clone());

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
        // 延迟持久化：会话首次发送消息时才写入 DB。
        // 用当前内存中的最新配置（用户可能已切换过 provider/model）。
        if db::get_session_meta(&db, &session_id)?.is_none() {
            let now = std::time::SystemTime::now();
            db::insert_session(
                &db,
                &session_id,
                sess.provider,
                &sess.model,
                &sess.preset_id,
                sess.created_at,
                now,
                &sess.title,
            )?;
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
    if should_emit_done {
        let _ = app.emit_to("index", &format!("on_message_done{session_id}"), ());
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
    list.sort_by_key(|a| a.update_at);
    list
}

/// 列出 DB 中所有历史会话的元数据（轻量，不载入内存、不重建 agent）。
#[tauri::command(rename_all = "snake_case")]
pub fn list_history_sessions(app: tauri::AppHandle) -> Result<Vec<SessionMeta>, String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let guard = state.read().unwrap();
    let metas = db::list_session_meta(&guard.db)?;
    let mut history: Vec<SessionMeta> = metas
        .into_iter()
        .filter(|m| !guard.sessions.contains_key(&m.session_id))
        .collect();
    history.sort_by_key(|m| std::cmp::Reverse(m.update_at));
    Ok(history)
}

/// 用户手动打开一个历史会话：从 DB 恢复并载入内存。已在内存中则直接返回。
#[tauri::command(rename_all = "snake_case")]
pub fn open_session(app: tauri::AppHandle, session_id: String) -> Result<Session, String> {
    let app_config_state = app.state::<AppConfigState>();
    let (api_keys, agent_presets) = {
        let config = app_config_state.read();
        (config.api_keys.clone(), config.agent_presets.clone())
    };

    let state = app.state::<Arc<RwLock<ChatState>>>();
    let mut guard = state.write().unwrap();
    let sess = if let Some(sess) = guard.sessions.get(&session_id) {
        sess.clone()
    } else {
        let meta =
            db::get_session_meta(&guard.db, &session_id)?.ok_or("Session not found in database")?;
        restore_session(&mut guard, &api_keys, &agent_presets, &meta)?
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
