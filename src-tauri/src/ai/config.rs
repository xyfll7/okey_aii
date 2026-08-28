use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provider {
    OpenAI,
    Anthropic,
    DeepSeek,
    Qwen,
    Zai,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: &'static str,
    pub label: &'static str,
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
        Provider::Qwen => &[
            ModelInfo {
                id: "qwen-max",
                label: "Qwen Max",
            },
            ModelInfo {
                id: "qwen-plus",
                label: "Qwen Plus",
            },
            ModelInfo {
                id: "qwen-turbo",
                label: "Qwen Turbo",
            },
        ],
        Provider::Zai => &[
            ModelInfo {
                id: "glm-5.3",
                label: "GLM-5.3",
            },
            ModelInfo {
                id: "glm-5.3-flash",
                label: "GLM-5.3-Flash",
            },
            ModelInfo {
                id: "glm-5.2",
                label: "GLM-5.2",
            },
            ModelInfo {
                id: "glm-image",
                label: "GLM-Image",
            },
            ModelInfo {
                id: "glm-ocr",
                label: "GLM-OCR",
            },
            ModelInfo {
                id: "glm-asr-2512",
                label: "GLM-ASR-2512",
            },
            ModelInfo {
                id: "glm-tts",
                label: "GLM-TTS",
            },
            ModelInfo {
                id: "cogvideox-3",
                label: "CogVideoX-3",
            },
            ModelInfo {
                id: "embedding-3",
                label: "Embedding-3",
            },
        ],
    }
}

pub fn default_model(provider: Provider) -> &'static str {
    available_models(provider)[0].id
}
