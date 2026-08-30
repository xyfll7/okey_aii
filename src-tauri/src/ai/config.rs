use rust_i18n::t;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

/// Stable identifier of a provider.
///
/// Kept as a small enum so `match` sites stay exhaustive and values round-trip
/// as plain strings ("OpenAI", "DeepSeek", ...) through the persisted config,
/// the sessions table and command arguments. All display metadata lives on
/// [`Provider`], the public-facing type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderId {
    OpenAI,
    Anthropic,
    DeepSeek,
    Qwen,
    Zai,
}

impl ProviderId {
    /// Every provider id currently supported by the backend, in display order.
    pub const ALL: [ProviderId; 5] = [
        ProviderId::OpenAI,
        ProviderId::Anthropic,
        ProviderId::DeepSeek,
        ProviderId::Qwen,
        ProviderId::Zai,
    ];

    /// The stable string form used in the DB, config and command arguments.
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderId::OpenAI => "OpenAI",
            ProviderId::Anthropic => "Anthropic",
            ProviderId::DeepSeek => "DeepSeek",
            ProviderId::Qwen => "Qwen",
            ProviderId::Zai => "Zai",
        }
    }

    pub fn from_str(s: &str) -> Option<ProviderId> {
        ProviderId::ALL.iter().copied().find(|p| p.as_str() == s)
    }
}

/// A provider supported by the backend, described by its own metadata.
///
/// Deliberately a struct, not an enum: each provider carries its stable id, a
/// display name resolved from the backend's current locale (`rust_i18n`), the
/// URL where its API key can be created, and an optional OpenAI-compatible
/// base URL. The label is resolved on the backend whenever the provider is
/// serialized, so the frontend never keeps its own provider → label mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Provider {
    pub id: ProviderId,
    /// Page where the user can create an API key for this provider.
    pub api_key_url: &'static str,
    /// OpenAI-compatible base URL override; `None` for native clients.
    pub base_url: Option<&'static str>,
}

impl Provider {
    pub const fn openai() -> Self {
        Self {
            id: ProviderId::OpenAI,
            api_key_url: "https://platform.openai.com/api-keys",
            base_url: None,
        }
    }

    pub const fn anthropic() -> Self {
        Self {
            id: ProviderId::Anthropic,
            api_key_url: "https://console.anthropic.com/settings/keys",
            base_url: None,
        }
    }

    pub const fn deepseek() -> Self {
        Self {
            id: ProviderId::DeepSeek,
            api_key_url: "https://platform.deepseek.com/api_keys",
            base_url: None,
        }
    }

    pub const fn qwen() -> Self {
        Self {
            id: ProviderId::Qwen,
            api_key_url: "https://bailian.console.aliyun.com/cn-beijing/#/home",
            base_url: Some(QWEN_COMPATIBLE_BASE_URL),
        }
    }

    pub const fn zai() -> Self {
        Self {
            id: ProviderId::Zai,
            api_key_url: "https://open.bigmodel.cn/login",
            base_url: Some(ZAI_GENERAL_BASE_URL),
        }
    }

    /// Every provider currently supported, in display order.
    pub const ALL: [Provider; 5] = [
        Self::openai(),
        Self::anthropic(),
        Self::deepseek(),
        Self::qwen(),
        Self::zai(),
    ];

    /// Returns the canonical provider instance for an id.
    pub fn from_id(id: ProviderId) -> Provider {
        Self::ALL
            .iter()
            .find(|p| p.id == id)
            .expect("every ProviderId has a matching Provider")
            .clone()
    }

    /// Localized display name, resolved from the backend's current locale.
    pub fn label(&self) -> String {
        match self.id {
            ProviderId::OpenAI => t!("provider.OpenAI").to_string(),
            ProviderId::Anthropic => t!("provider.Anthropic").to_string(),
            ProviderId::DeepSeek => t!("provider.DeepSeek").to_string(),
            ProviderId::Qwen => t!("provider.Qwen").to_string(),
            ProviderId::Zai => t!("provider.Zai").to_string(),
        }
    }
}

impl Serialize for Provider {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Provider", 4)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("label", &self.label())?;
        state.serialize_field("api_key_url", self.api_key_url)?;
        state.serialize_field("base_url", &self.base_url)?;
        state.end()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub label: String,
}

/// DashScope (Qwen) OpenAI-compatible endpoint used for both chat and model listing.
const QWEN_COMPATIBLE_BASE_URL: &str = "https://dashscope.aliyuncs.com/compatible-mode/v1";

/// Z.ai (Zai) OpenAI-compatible endpoint used for model listing.
const ZAI_GENERAL_BASE_URL: &str = "https://api.z.ai/api/paas/v4";
