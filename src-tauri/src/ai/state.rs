use crate::ai::agents::Agents;
use crate::ai::config::{builtin_presets, AgentPreset, Provider};
use crate::ai::db::Db;
use rig::client::AgentClientExt;
use rig::message::Message;
use rig::providers::{anthropic, deepseek, openai};
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


#[derive(Clone)]
pub struct AppConfig {
    pub api_keys: HashMap<Provider, String>,
}


#[derive(Clone, serde::Serialize)]
pub struct Session {
    pub session_id: String,
    pub provider: Provider,
    pub model: String,
    pub preset_id: String,
    #[serde(serialize_with = "serialize_systemtime_millis")]
    pub created_at: SystemTime,
    #[serde(serialize_with = "serialize_systemtime_millis")]
    pub update_at: SystemTime,
    pub title: String,
    
    
    pub is_loading: bool,
    #[serde(skip)]
    pub agent: Arc<Agents>,
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
    pub config: AppConfig,
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

fn build_agent(
    provider: Provider,
    model: &str, 
    api_key: &str,
    preset: &AgentPreset,
) -> Result<Agents, String> {
    match provider {
        Provider::OpenAI => {
            
            
            let client = openai::CompletionsClient::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(&preset.preamble).build();
            Ok(Agents::OpenAI(agent))
        }
        Provider::Anthropic => {
            let client = anthropic::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(&preset.preamble).build();
            Ok(Agents::Anthropic(agent))
        }
        Provider::DeepSeek => {
            let client = deepseek::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(&preset.preamble).build();
            Ok(Agents::DeepSeek(agent))
        }
    }
}


pub fn build_session_agent(
    api_keys: &HashMap<Provider, String>,
    provider: Provider,
    model: &str,
    preset_id: &str,
) -> Result<Arc<Agents>, String> {
    let key = api_keys
        .get(&provider)
        .cloned()
        .ok_or_else(|| format!("{provider:?} missing api key"))?;
    let preset = builtin_presets()
        .into_iter()
        .find(|p| p.id == preset_id)
        .ok_or("preset not found")?;
    Ok(Arc::new(build_agent(provider, model, &key, &preset)?))
}
