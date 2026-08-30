use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use rig::client::ModelListingClient;
use rig::providers::{anthropic, deepseek, openai};
use tauri::{AppHandle, Manager};

use super::config::{ModelInfo, Provider, ProviderId};
use crate::store::app_state::AppConfigState;

/// How long a fetched model catalog stays valid before being refreshed.
const CACHE_TTL: Duration = Duration::from_secs(15 * 60);

struct CachedModels {
    fetched_at: Instant,
    models: Vec<ModelInfo>,
}

/// In-memory model catalog cache.
///
/// This is runtime data (not user configuration), so it deliberately lives
/// outside of `AppConfig` / `store.json`. The catalog is fully driven by the
/// provider's listing API; there is no offline static fallback.
pub struct ModelCatalogState {
    cache: RwLock<HashMap<ProviderId, CachedModels>>,
}

impl Default for ModelCatalogState {
    fn default() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl ModelCatalogState {
    /// Returns the model list for a provider, refreshing from the provider's
    /// listing endpoint when the cache is stale. Returns an empty list when the
    /// fetch fails (no API key, network error, unsupported provider).
    pub async fn list_models(&self, app: &AppHandle, provider: Provider) -> Vec<ModelInfo> {
        if let Some(models) = self.cached_models(provider.id) {
            return models;
        }

        let api_key = {
            let config_state = app.state::<AppConfigState>();
            let config = config_state.read();
            config.api_keys.get(&provider.id).cloned().unwrap_or_default()
        };

        if !api_key.is_empty() {
            match fetch_models(&provider, &api_key).await {
                Ok(models) if !models.is_empty() => {
                    if let Ok(mut cache) = self.cache.write() {
                        cache.insert(
                            provider.id,
                            CachedModels {
                                fetched_at: Instant::now(),
                                models: models.clone(),
                            },
                        );
                    }
                    return models;
                }
                Ok(_) => {
                    log::warn!("model listing for {} returned an empty list", provider.id.as_str());
                }
                Err(e) => {
                    log::warn!("failed to fetch model list for {}: {e}", provider.id.as_str());
                }
            }
        }

        Vec::new()
    }

    /// Synchronously reads the cached model list. Returns `None` when absent or
    /// stale.
    pub fn cached_models(&self, provider: ProviderId) -> Option<Vec<ModelInfo>> {
        let cache = self.cache.read().ok()?;
        let entry = cache.get(&provider)?;
        if entry.fetched_at.elapsed() < CACHE_TTL {
            Some(entry.models.clone())
        } else {
            None
        }
    }

}

/// Fetches the live model list from the provider's listing endpoint via rig's
/// `ModelListingClient`. All providers expose an OpenAI-compatible
/// `/models` endpoint; Anthropic and DeepSeek use their native clients.
async fn fetch_models(
    provider: &Provider,
    api_key: &str,
) -> Result<Vec<ModelInfo>, Box<dyn std::error::Error>> {
    let list = match provider.id {
        ProviderId::OpenAI => openai::CompletionsClient::new(api_key)?.list_models().await?,
        ProviderId::Anthropic => anthropic::Client::new(api_key)?.list_models().await?,
        ProviderId::DeepSeek => deepseek::Client::new(api_key)?.list_models().await?,
        ProviderId::Qwen | ProviderId::Zai => openai::CompletionsClient::builder()
            .api_key(api_key)
            .base_url(provider.base_url.expect("Qwen and Zai provide an OpenAI-compatible base URL"))
            .build()?
            .list_models()
            .await?,
    };

    Ok(list
        .data
        .into_iter()
        .map(|m| ModelInfo {
            label: m.display_name().to_string(),
            id: m.id,
        })
        .collect())
}
