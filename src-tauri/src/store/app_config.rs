use crate::ai::config::Provider;
use dotenvy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    pub name: String,
    pub hot_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AutoSpeakState {
    Off,
    #[default]
    Single,
    All,
}

/// 支持的语言选项
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
    pub raw: Option<String>,
    pub label: Option<String>,
    pub content: Option<String>,
    pub id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    #[serde(default = "default_api_keys")]
    pub api_keys: HashMap<Provider, String>,
    pub shortcuts: Vec<Shortcut>,
    pub is_pin_index_window: bool,
    pub auto_speak: AutoSpeakState,
    pub language: Language,
    pub local_language: Language,
    pub target_language: Language,
    pub self_explaining_model: bool,
    pub prompt_tags: Vec<PromptTag>,
}

/// Build the default prompt tags localized for the given locale.
/// Labels and contents are loaded from the i18n locale files so a fresh
/// install (no `store.json`) starts in the user's system language.
pub fn default_prompt_tags_for_locale(locale: &str) -> Vec<PromptTag> {
    #[rustfmt::skip]
    let tags = vec![
        PromptTag { raw: None, label: Some(rust_i18n::t!("prompt_tag_system_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_system_content", locale = locale).to_string()), id: Some(0) },
        PromptTag { raw: None, label: Some(rust_i18n::t!("prompt_tag_summary_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_summary_content", locale = locale).to_string()), id: Some(1) },
        PromptTag { raw: None, label: Some(rust_i18n::t!("prompt_tag_right_ctrl_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_right_ctrl_content", locale = locale).to_string()), id: Some(2) },
        PromptTag { raw: None, label: Some(rust_i18n::t!("prompt_tag_self_explanation_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_self_explanation_content", locale = locale).to_string()), id: Some(3) },
        PromptTag { raw: None, label: Some(rust_i18n::t!("prompt_tag_word_details_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_word_details_content", locale = locale).to_string()), id: Some(4) },
        PromptTag { raw: None, label: Some(rust_i18n::t!("prompt_tag_meaning_context_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_meaning_context_content", locale = locale).to_string()), id: Some(5) },
        PromptTag { raw: None, label: Some(rust_i18n::t!("prompt_tag_detailed_explanation_label", locale = locale).to_string()), content: Some(rust_i18n::t!("prompt_tag_detailed_explanation_content", locale = locale).to_string()), id: Some(6) },
    ];
    tags
}

/// 从 .env 读取默认的 API Keys（首次运行或旧版 store.json 缺少该字段时使用）。
/// 缺失或仍为占位符的 key 会被跳过，避免把无效值写入配置。
fn default_api_keys() -> HashMap<Provider, String> {
    dotenvy::dotenv().ok();
    let mut keys = HashMap::new();
    let entries = [
        ("OPENAI_API_KEY", Provider::OpenAI),
        ("QWEN_API_KEY", Provider::Qwen),
        ("DEEPSEEK_API_KEY", Provider::DeepSeek),
        ("ZAI_API_KEY", Provider::Zai),
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
            prompt_tags: default_prompt_tags_for_locale(&crate::utils::i18n::get_default_locale()),
        }
    }
}
