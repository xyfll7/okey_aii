use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provider {
    OpenAI,
    Anthropic,
    DeepSeek,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: &'static str,
    pub label: &'static str,
}

/// 用户在前端下拉框里选的"agent" —— preamble/工具组合，与 provider 无关
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPreset {
    pub id: String,
    pub name: String,
    pub preamble: String,
}

pub fn builtin_presets() -> Vec<AgentPreset> {
    vec![
        AgentPreset {
            id: "translator".into(),
            name: "翻译助手".into(),
            preamble: "You are a professional translator. Only output the translation.".into(),
        },
        AgentPreset {
            id: "assistant".into(),
            name: "通用助手".into(),
            preamble: "You are a helpful assistant.".into(),
        },
    ]
}

/// 每个 provider 支持哪些模型,给前端下拉框用
///
/// 注:deepseek-chat / deepseek-reasoner 已被 rig 标记为 deprecated(将于 2026/07/24 废弃),
/// 这里直接用字符串字面量,等价于 DEEPSEEK_V4_FLASH 的非思考/思考模式,避免编译告警。
pub fn available_models(provider: Provider) -> &'static [ModelInfo] {
    match provider {
        Provider::OpenAI => &[ModelInfo {
            id: "gpt-5.5",
            label: "GPT-5.5",
        }],
        Provider::Anthropic => &[ModelInfo {
            id: "claude-sonnet-4-6",
            label: "Claude Sonnet 4.6",
        }],
        Provider::DeepSeek => &[
            ModelInfo {
                id: "deepseek-chat",
                label: "DeepSeek Chat",
            },
            ModelInfo {
                id: "deepseek-reasoner",
                label: "DeepSeek Reasoner",
            },
            ModelInfo {
                id: "deepseek-v4-flash",
                label: "DeepSeek V4 Flash",
            },
            ModelInfo {
                id: "deepseek-v4-pro",
                label: "DeepSeek V4 Pro",
            },
        ],
    }
}

/// 切换 provider 时,没显式选模型就用第一个当默认值
pub fn default_model(provider: Provider) -> &'static str {
    available_models(provider)[0].id
}
