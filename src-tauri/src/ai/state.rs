use crate::ai::agents::Agents;
use crate::ai::config::{builtin_presets, AgentPreset, Provider};
use rig::client::AgentClientExt;
use rig::message::Message;
use rig::providers::{anthropic, deepseek, openai};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use tauri::{AppHandle, Manager, State};

/// 历史记录中的单条消息。
///
/// `rig::message::Message` 是第三方类型,无法携带我们自己的元数据,所以在此包装一层,
/// 额外保存 `id`(前端展示与按 id 删除用)和 `created_at`(创建时间,毫秒时间戳)。
#[derive(Clone, Debug, serde::Serialize)]
pub struct HistoryItem {
    /// 本条历史的唯一 id,前端用于渲染 key 与按 id 删除。
    pub id: String,
    /// 创建时间(毫秒时间戳),便于前端排序与展示。
    #[serde(serialize_with = "serialize_systemtime_millis")]
    pub created_at: SystemTime,
    /// 实际对话消息内容(rig 原生结构)。
    pub message: Message,
}

/// 全局共享状态:只保存 api key。
///
/// 新标签页的配置从最近一个 Session 中继承;每个已打开的会话各自锁定自己的
/// provider/model/preset(见 `Session`),互不干扰。
#[derive(Clone)]
pub struct AppConfig {
    pub api_keys: HashMap<Provider, String>,
}

/// 单个标签页/会话的私有状态:各自锁定 provider/model/preset,互不干扰
#[derive(Clone, serde::Serialize)]
pub struct Session {
    pub session_id: String,
    pub provider: Provider,
    pub model: String,
    pub preset_id: String,
    #[serde(serialize_with = "serialize_systemtime_millis")]
    pub created_at: SystemTime,
    pub title: String,
    #[serde(skip)]
    pub agent: Arc<Agents>,
    #[serde(skip)]
    pub history: Vec<HistoryItem>,
}

fn serialize_systemtime_millis<S: serde::Serializer>(
    t: &SystemTime,
    s: S,
) -> Result<S::Ok, S::Error> {
    let millis = t
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    s.serialize_u64(millis)
}

/// 顶层状态:一份全局配置(含最近选择) + 多个互不干扰的独立会话
pub struct ChatState {
    pub config: AppConfig,
    pub sessions: HashMap<String, Session>,
}


pub fn add_message_to_history(
    app_handle: &AppHandle,
    session_id: String,
    message: Message,
) -> Result<HistoryItem, String> {
    let state: State<'_, Arc<RwLock<ChatState>>> = app_handle.state();
    let mut guard = state.write().unwrap();
    let sess = guard
        .sessions
        .get_mut(&session_id)
        .ok_or("会话不存在,请先调用 create_session")?;
    let item = HistoryItem {
        id: uuid::Uuid::new_v4().to_string(),
        created_at: SystemTime::now(),
        message,
    };
    sess.history.push(item.clone());
    Ok(item)
}

/// 按 id 从指定会话中删除一条历史记录。
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
        .ok_or("会话不存在,请先调用 create_session")?;
    let before = sess.history.len();
    sess.history.retain(|item| item.id != history_id);
    if sess.history.len() == before {
        return Err(format!("history item 不存在: {history_id}"));
    }
    Ok(())
}

fn build_agent(
    provider: Provider,
    model: &str, // ← 新增,不再用 default_model 写死
    api_key: &str,
    preset: &AgentPreset,
) -> Result<Agents, String> {
    match provider {
        Provider::OpenAI => {
            // 0.41.0 里 `openai::Client` 默认走 Responses API，返回的是
            // `GenericResponsesCompletionModel`；completion 模型要用 `CompletionsClient`。
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

/// 按给定的 provider/model/preset 为一个会话构建 agent。
///
/// 不依赖全局配置:每个会话各自传入自己的 provider/model/preset,实现标签页互不干扰。
/// `api_keys` 仍来自全局(用户级别的凭据),命中不到对应 provider 的 key 时返回错误。
pub fn build_session_agent(
    api_keys: &HashMap<Provider, String>,
    provider: Provider,
    model: &str,
    preset_id: &str,
) -> Result<Arc<Agents>, String> {
    let key = api_keys
        .get(&provider)
        .cloned()
        .ok_or_else(|| format!("{provider:?} 缺少 api key"))?;
    let preset = builtin_presets()
        .into_iter()
        .find(|p| p.id == preset_id)
        .ok_or("preset not found")?;
    Ok(Arc::new(build_agent(provider, model, &key, &preset)?))
}
