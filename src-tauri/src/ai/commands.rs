use std::sync::{Arc, RwLock};

use futures::StreamExt;
use rig::message::Message;
use tauri::ipc::Channel;

use crate::ai::state::{
    add_message_to_history, build_session_agent, ensure_session_title, restore_session,
    HistoryItem, Session,
};

use super::agents::ChatEvent;
use super::config::{ModelInfo, Provider, ProviderId};
use super::db::{self, SessionMeta};
use super::model_catalog::ModelCatalogState;
use super::state::ChatState;
use crate::my_windows::window_index::should_use_existing_index_window;
use crate::store::app_state::AppConfigState;
use tauri::{Emitter, Manager};

/// Returns the providers currently supported by the backend. Each item carries
/// its localized label (resolved via the backend's rust_i18n locale), so the
/// frontend renders provider options from this list instead of hard-coding its
/// own mapping.
#[tauri::command(rename_all = "snake_case")]
pub fn list_providers() -> Vec<Provider> {
    Provider::ALL.to_vec()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_models(app: tauri::AppHandle, provider: ProviderId) -> Vec<ModelInfo> {
    let catalog = app.state::<ModelCatalogState>();
    catalog.list_models(&app, Provider::from_id(provider)).await
}

/// Resolves the default model for a provider from its live listing API.
///
/// There is deliberately no compile-time fallback: model data comes entirely
/// from the provider. If the listing is unreachable or empty (e.g. no API key
/// configured), this returns an error so callers never construct a session
/// with a hard-coded model.
async fn default_model_from_api(
    app: &tauri::AppHandle,
    provider: Provider,
) -> Result<String, String> {
    let catalog = app.state::<ModelCatalogState>();
    let models = catalog.list_models(app, provider).await;
    models.first().map(|m| m.id.clone()).ok_or_else(|| {
        format!(
            "{} returned no models from its listing API; \
             add an API key in settings and try again",
            provider.label()
        )
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_session(app: tauri::AppHandle) -> Result<(String, Session), String> {
    let app_config_state = app.state::<AppConfigState>();
    let (api_keys, agent_presets) = {
        let config = app_config_state.read();
        (config.api_keys.clone(), config.agent_presets.clone())
    };

    // Inherit provider/model/preset from the most recently touched session.
    // No locks are held across the await below.
    let inherited = {
        let state = app.state::<Arc<RwLock<ChatState>>>();
        let guard = state.read().unwrap();
        guard
            .sessions
            .values()
            .max_by(|a, b| a.update_at.cmp(&b.update_at))
            .map(|s| (s.provider, s.model.clone(), s.preset_id.clone()))
    };

    let (provider, model, preset_id) = match inherited {
        Some(inherited) => inherited,
        None => {
            let p = Provider::deepseek();
            // No session exists yet, so the model must come from the live
            // listing; there is no hard-coded model to fall back on.
            let model = default_model_from_api(&app, p).await?;
            (p, model, "assistant".to_string())
        }
    };

    let agent = build_session_agent(&api_keys, provider, &model, &preset_id, &agent_presets)?;

    let session_id = uuid::Uuid::new_v4().to_string();
    let session = Session {
        session_id: session_id.clone(),
        provider: agent.provider,
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

    let state = app.state::<Arc<RwLock<ChatState>>>();
    let mut guard = state.write().unwrap();
    guard.sessions.insert(session_id.clone(), session.clone());
    Ok((session_id, session))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn new_session(app: tauri::AppHandle) -> Result<Option<String>, String> {
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
    if to_clear.is_empty() {
        return Ok(None);
    }
    for session_id in to_clear {
        close_session(app.clone(), session_id)?;
    }
    let (session_id, _) = create_session(app).await?;
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

/// Moves a session onto a provider + model pair in a single step.
///
/// The two halves are applied atomically because neither is meaningful alone:
/// a model only exists within a provider, so changing one without the other
/// would leave the session briefly holding a model from the previous provider.
///
/// `model` is deliberately not validated against a local model list. The
/// provider is the only authority on which models it serves, while any cached
/// listing is a snapshot that may be stale or incomplete; rejecting a model it
/// does not know about only produces false negatives. An unusable model
/// surfaces as the provider's own error on the next `send_message`, which is
/// both accurate and actionable.
#[tauri::command(rename_all = "snake_case")]
pub fn switch_combo(
    app: tauri::AppHandle,
    session_id: String,
    provider: ProviderId,
    model: String,
    api_key: Option<String>,
) -> Result<(), String> {
    let provider = Provider::from_id(provider);
    if let Some(key) = api_key {
        let app_config_state = app.state::<AppConfigState>();
        app_config_state
            .update_and_save(|config| {
                config.api_keys.insert(provider.id, key);
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
        sess.provider = agent.provider;
        sess.model = model.clone();
        sess.agent = agent;
        sess.update_at = update_at;
    }
    db::update_session(&guard.db, &session_id, provider.id, &model, update_at)?;

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
    println!("abc8888********{}",session_id);
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
    let prompt_id = prompt.id.clone();

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
        if db::get_session_meta(&db, &session_id)?.is_none() {
            let now = std::time::SystemTime::now();
            db::insert_session(
                &db,
                &session_id,
                sess.provider.id,
                &sess.model,
                &sess.preset_id,
                sess.created_at,
                now,
                &sess.title,
            )?;
        }
        db::insert_message(&db, &session_id, &prompt)?;
        // A retry sends the same user message again with the same id: replace the
        // previous attempt instead of stacking a duplicate onto the history.
        if sess.history.last().is_some_and(|h| h.id == prompt_id) {
            if let Some(last) = sess.history.last_mut() {
                *last = prompt.clone();
            }
        } else {
            sess.history.push(prompt.clone());
        }
        ensure_session_title(&db, &session_id, sess);
        sess.is_loading = true;
        sess.cancel_handle = Some(abort_handle);
        sess.update_at = std::time::SystemTime::now();
    }

    let prompt: Message = prompt_msg;

    // The prompt is passed separately, so drop its own copy from the context.
    // On a retry it is already the last history item and would otherwise be
    // sent to the model twice.
    let history: Vec<Message> = history
        .iter()
        .filter(|h| h.id != prompt_id)
        .map(|h| h.message.clone())
        .collect();
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
            Ok(ChatEvent::Error(e)) => {
                eprintln!("[send_message] stream error: {e}");
                let _ = on_event.send(ChatEvent::Error(e));
                let mut guard = state.write().unwrap();
                if let Some(sess) = guard.sessions.get_mut(&session_id) {
                    sess.is_loading = false;
                    sess.cancel_handle = None;
                    sess.update_at = std::time::SystemTime::now();
                }
                return Ok(());
            }
            Err(e) => {
                eprintln!("[send_message] stream error: {e}");
                let _ = on_event.send(ChatEvent::Error(e));
                let mut guard = state.write().unwrap();
                if let Some(sess) = guard.sessions.get_mut(&session_id) {
                    sess.is_loading = false;
                    sess.cancel_handle = None;
                    sess.update_at = std::time::SystemTime::now();
                }
                return Ok(());
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
