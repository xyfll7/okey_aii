use crate::ai::config::ProviderId;
use dotenvy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    pub name: String,
    pub hot_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPreset {
    pub id: String,
    pub name: String,
    pub prompt_tags: Vec<PromptTag>,
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AutoSpeakState {
    Off,
    #[default]
    Single,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum Language {
    #[serde(rename = "auto")]
    #[default]
    Auto,
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en")]
    En,
}

impl Language {
    pub fn to_locale(&self) -> String {
        match self {
            Language::Auto => crate::utils::i18n::get_default_locale(),
            Language::ZhCn => "zh-CN".to_string(),
            Language::En => "en".to_string(),
        }
    }

    pub fn to_display_name(&self) -> String {
        match self {
            Language::Auto => rust_i18n::t!("language_auto").to_string(),
            Language::ZhCn => rust_i18n::t!("language_chinese").to_string(),
            Language::En => rust_i18n::t!("language_english").to_string(),
        }
    }

    pub fn effective_language(&self) -> Self {
        match self {
            Language::Auto => Language::from_locale(&crate::utils::i18n::get_default_locale()),
            _ => *self,
        }
    }

    pub fn from_locale(locale: &str) -> Self {
        match locale {
            "zh-CN" => Language::ZhCn,
            "en" => Language::En,
            _ => Language::Auto,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptTag {
    pub label: Option<String>,
    pub content: Option<String>,
    pub id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    #[serde(default = "default_api_keys")]
    pub api_keys: HashMap<ProviderId, String>,
    pub shortcuts: Vec<Shortcut>,
    pub is_pin_index_window: bool,
    pub auto_speak: AutoSpeakState,
    pub language: Language,
    pub local_language: Language,
    pub target_language: Language,
    pub self_explaining_model: bool,
    #[serde(default = "default_agent_presets_serde")]
    pub agent_presets: Vec<AgentPreset>,
}


fn default_api_keys() -> HashMap<ProviderId, String> {
    dotenvy::dotenv().ok();
    let mut keys = HashMap::new();
    let entries = [
        ("OPENAI_API_KEY", ProviderId::OpenAI),
        ("QWEN_API_KEY", ProviderId::Qwen),
        ("DEEPSEEK_API_KEY", ProviderId::DeepSeek),
        ("ZAI_API_KEY", ProviderId::Zai),
    ];
    for (env_name, provider) in entries {
        if let Ok(key) = std::env::var(env_name) {
            if !key.is_empty() && !key.starts_with("your_") {
                keys.insert(provider, key);
            }
        }
    }
    keys
}

fn default_agent_presets_serde() -> Vec<AgentPreset> {
    default_agent_presets(&crate::utils::i18n::get_default_locale())
}

fn default_agent_presets(locale: &str) -> Vec<AgentPreset> {
    vec![
        AgentPreset {
            id: "translator".into(),
            name: "Translator".into(),
            prompt_tags: vec![
                PromptTag { label: Some(rust_i18n::t!("prompt_tag_system_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_system_content", locale = locale).to_string()), id: Some(0) },
                PromptTag { label: Some(rust_i18n::t!("prompt_tag_summary_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_summary_content", locale = locale).to_string()), id: Some(1) },
                PromptTag { label: Some(rust_i18n::t!("prompt_tag_right_ctrl_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_right_ctrl_content", locale = locale).to_string()), id: Some(2) },
                PromptTag { label: Some(rust_i18n::t!("prompt_tag_self_explanation_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_self_explanation_content", locale = locale).to_string()), id: Some(3) },
                PromptTag { label: Some(rust_i18n::t!("prompt_tag_word_details_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_word_details_content", locale = locale).to_string()), id: Some(4) },
                PromptTag { label: Some(rust_i18n::t!("prompt_tag_meaning_context_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_meaning_context_content", locale = locale).to_string()), id: Some(5) },
                PromptTag { label: Some(rust_i18n::t!("prompt_tag_detailed_explanation_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_detailed_explanation_content", locale = locale).to_string()), id: Some(6) },
            ],
        },
        AgentPreset {
            id: "assistant".into(),
            name: "Assistant".into(),
            prompt_tags: vec![
                PromptTag { label: Some(rust_i18n::t!("prompt_tag_system_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_system_content", locale = locale).to_string()), id: Some(0) },
                PromptTag { label: Some(rust_i18n::t!("prompt_tag_summary_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_summary_content", locale = locale).to_string()), id: Some(1) },
                PromptTag { label: Some(rust_i18n::t!("prompt_tag_right_ctrl_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_right_ctrl_content", locale = locale).to_string()), id: Some(2) },
                PromptTag { label: Some(rust_i18n::t!("prompt_tag_self_explanation_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_self_explanation_content", locale = locale).to_string()), id: Some(3) },
                PromptTag { label: Some(rust_i18n::t!("prompt_tag_word_details_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_word_details_content", locale = locale).to_string()), id: Some(4) },
                PromptTag { label: Some(rust_i18n::t!("prompt_tag_meaning_context_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_meaning_context_content", locale = locale).to_string()), id: Some(5) },
                PromptTag { label: Some(rust_i18n::t!("prompt_tag_detailed_explanation_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_detailed_explanation_content", locale = locale).to_string()), id: Some(6) },
            ],
        },
    ]
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            api_keys: default_api_keys(),
            shortcuts: vec![Shortcut { name: "okey_ai".to_string(), hot_key: ["Ctrl+G", "Cmd+G"][cfg!(target_os = "macos") as usize].to_string() }],
            is_pin_index_window: false,
            auto_speak: AutoSpeakState::default(),

            language: Language::default(),
            local_language: if cfg!(debug_assertions) { Language::ZhCn } else { Language::Auto.effective_language() },
            target_language: Language::Auto.effective_language(),
            self_explaining_model: false,
            agent_presets: default_agent_presets(&crate::utils::i18n::get_default_locale()),
        }
    }
}
