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
            name: "Translator".into(),
            preamble: "You are a professional translator. Only output the translation.".into(),
        },
        AgentPreset {
            id: "assistant".into(),
            name: "Assistant".into(),
            preamble: "You are a helpful assistant.".into(),
        },
    ]
}


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

pub fn default_model(provider: Provider) -> &'static str {
    available_models(provider)[0].id
}
