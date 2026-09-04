use crate::ai::agents::ChatAgent;
use crate::ai::config::{Provider, ProviderId};
use crate::store::app_config::AgentPreset;
use crate::ai::db::{self, Db, SessionMeta};
use rig::client::AgentClientExt;
use rig::message::{Message, UserContent};
use rig::providers::{anthropic, deepseek, openai, zai};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use tauri::{AppHandle, Manager, State};


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HistoryItem {
    
    pub id: String,
    
    #[serde(
        serialize_with = "serialize_systemtime_millis",
        deserialize_with = "deserialize_systemtime_millis"
    )]
    pub created_at: SystemTime,
    
    pub message: Message,
}


#[derive(Clone, serde::Serialize)]
pub struct Session {
    pub session_id: String,
    pub provider: Provider,
    pub model: String,
    pub preset_id: String,
    /// Whether reasoning/thinking mode is enabled for this session's requests.
    pub thinking: bool,
    #[serde(serialize_with = "serialize_systemtime_millis")]
    pub created_at: SystemTime,
    #[serde(serialize_with = "serialize_systemtime_millis")]
    pub update_at: SystemTime,
    pub title: String,
    
    
    pub is_loading: bool,
    #[serde(skip)]
    pub agent: Arc<ChatAgent>,
    #[serde(skip)]
    pub history: Vec<HistoryItem>,
    
    #[serde(skip)]
    pub cancel_handle: Option<futures::future::AbortHandle>,
}

pub(crate) fn serialize_systemtime_millis<S: serde::Serializer>(
    t: &SystemTime,
    s: S,
) -> Result<S::Ok, S::Error> {
    let millis = t
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    s.serialize_u64(millis)
}


fn deserialize_systemtime_millis<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<SystemTime, D::Error> {
    let millis = u64::deserialize(d)?;
    Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(millis))
}


pub struct ChatState {
    pub sessions: HashMap<String, Session>,
    pub db: Db,
}


pub fn add_message_to_history(
    app_handle: &AppHandle,
    session_id: String,
    item: HistoryItem,
) -> Result<HistoryItem, String> {
    let state: State<'_, Arc<RwLock<ChatState>>> = app_handle.state();
    let mut guard = state.write().unwrap();
    let sess = guard
        .sessions
        .get_mut(&session_id)
        .ok_or("Session not found, please call create_session first")?;
    if sess.is_loading {
        return Err("Session is currently generating a response (loading), adding new messages is temporarily disabled".into());
    }
    sess.history.push(item.clone());
    sess.update_at = SystemTime::now();
    Ok(item)
}

pub fn remove_history_item(
    app_handle: &AppHandle,
    session_id: String,
    history_id: String,
) -> Result<(), String> {
    let state: State<'_, Arc<RwLock<ChatState>>> = app_handle.state();
    let mut guard = state.write().unwrap();
    let sess = guard
        .sessions
        .get_mut(&session_id)
        .ok_or("Session not found, please call create_session first")?;
    let before = sess.history.len();
    sess.history.retain(|item| item.id != history_id);
    if sess.history.len() == before {
        return Err(format!("history item not found: {history_id}"));
    }
    sess.update_at = SystemTime::now();
    Ok(())
}

/// Provider-specific request-body parameters that enable the model's
/// reasoning/thinking mode. Only applied when the user toggles thinking on;
/// turning it off sends no parameter, preserving each model's default.
fn thinking_params(id: ProviderId) -> serde_json::Value {
    use serde_json::json;
    match id {
        // DeepSeek V4 (`deepseek-v4-*`): thinking is opt-in via the top-level
        // `thinking` field.
        ProviderId::DeepSeek => json!({
            "thinking": { "type": "enabled", "budget_tokens": 4096 }
        }),
        // Claude: extended thinking needs a token budget below `max_tokens`.
        ProviderId::Anthropic => json!({
            "thinking": { "type": "enabled", "budget_tokens": 1024 }
        }),
        // Z.ai GLM: OpenAI-compatible `thinking` switch.
        ProviderId::Zai => json!({ "thinking": { "type": "enabled" } }),
        // OpenAI reasoning models (o-series / gpt-5) accept `reasoning_effort`.
        ProviderId::OpenAI => json!({ "reasoning_effort": "high" }),
        // DashScope (Qwen3 thinking models).
        ProviderId::Qwen => json!({ "enable_thinking": true }),
    }
}

fn build_agent(
    provider: Provider,
    model: &str,
    api_key: &str,
    preset: &AgentPreset,
    thinking: bool,
) -> Result<ChatAgent, String> {
    let preamble = &preset.prompt_tags.iter()
        .find(|t| t.id == Some(0))
        .and_then(|t| t.content.as_ref())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_default();
    let mut builder = match provider.id {
        ProviderId::OpenAI => {
            let client = openai::CompletionsClient::new(api_key).map_err(|e| e.to_string())?;
            client.agent(model).preamble(&preamble)
        }
        ProviderId::Anthropic => {
            let client = anthropic::Client::new(api_key).map_err(|e| e.to_string())?;
            client.agent(model).preamble(&preamble)
        }
        ProviderId::DeepSeek => {
            let client = deepseek::Client::new(api_key).map_err(|e| e.to_string())?;
            client.agent(model).preamble(&preamble)
        }
        ProviderId::Qwen => {
            let client = openai::CompletionsClient::builder()
                .api_key(api_key)
                .base_url(provider.base_url.expect("Qwen provides an OpenAI-compatible base URL"))
                .build()
                .map_err(|e| e.to_string())?;
            client.agent(model).preamble(&preamble)
        }
        ProviderId::Zai => {
            let client = zai::Client::new(api_key).map_err(|e| e.to_string())?;
            client.agent(model).preamble(&preamble)
        }
    };
    if thinking {
        builder = builder.additional_params(thinking_params(provider.id));
    }
    Ok(ChatAgent {
        provider,
        agent: builder.build(),
    })
}

pub fn build_session_agent(
    api_keys: &HashMap<ProviderId, String>,
    provider: Provider,
    model: &str,
    preset_id: &str,
    presets: &[AgentPreset],
    thinking: bool,
) -> Result<Arc<ChatAgent>, String> {
    let key = api_keys
        .get(&provider.id)
        .cloned()
        .ok_or_else(|| format!("{} missing api key", provider.id.as_str()))?;
    let preset = presets
        .iter()
        .find(|p| p.id == preset_id)
        .ok_or("preset not found")?;
    Ok(Arc::new(build_agent(provider, model, &key, preset, thinking)?))
}

pub fn restore_session(
    guard: &mut ChatState,
    api_keys: &HashMap<ProviderId, String>,
    presets: &[AgentPreset],
    meta: &SessionMeta,
) -> Result<Session, String> {
    let agent = build_session_agent(
        api_keys,
        meta.provider,
        &meta.model,
        &meta.preset_id,
        presets,
        meta.thinking,
    )?;
    let history = db::get_history(&guard.db, &meta.session_id)?;
    let sess = Session {
        session_id: meta.session_id.clone(),
        provider: agent.provider,
        model: meta.model.clone(),
        preset_id: meta.preset_id.clone(),
        thinking: meta.thinking,
        created_at: meta.created_at,
        update_at: SystemTime::now(),
        title: meta.title.clone(),
        is_loading: false,
        agent,
        history,
        cancel_handle: None,
    };
    guard.sessions.insert(meta.session_id.clone(), sess.clone());
    Ok(sess)
}

fn derive_title(msg: &Message) -> Option<String> {
    const MAX_TITLE_LEN: usize = 30;
    let text = match msg {
        Message::User { content } => content
            .iter()
            .find_map(|c| match c {
                UserContent::Text(t) => Some(t.text.as_str()),
                _ => None,
            }),
        _ => return None,
    }?;
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty() {
        return None;
    }
    let truncated: String = text.chars().take(MAX_TITLE_LEN).collect();
    Some(if text.chars().count() > MAX_TITLE_LEN {
        format!("{truncated}…")
    } else {
        truncated
    })
}

pub fn ensure_session_title(db: &Db, session_id: &str, sess: &mut Session) {
    let Some(title) = sess.history.iter().find_map(|h| derive_title(&h.message)) else {
        return;
    };
    sess.title = title.clone();
    if let Err(e) = db::update_session_title(db, session_id, &title) {
        log::warn!("failed to update session title: {e}");
    }
}
