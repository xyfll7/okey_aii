use std::collections::HashMap;
use std::sync::Arc;
use rig::message::Message;
use crate::ai::agents::Agents;
use crate::ai::config::{AgentPreset, Provider};
use rig::client::AgentClientExt;
use rig::providers::{anthropic, deepseek, openai};

pub struct ChatState {
    pub provider: Provider,
    pub model: String,        // ← 新增
    pub preset_id: String,
    pub api_keys: HashMap<Provider, String>,
    pub agent: Arc<Agents>,          // ← 包一层 Arc,读锁里 clone 即可立刻放锁
    pub history: Vec<Message>,       // ← 跨轮次保留的对话历史
}

pub fn build_agent(
    provider: Provider,
    model: &str,           // ← 新增,不再用 default_model 写死
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