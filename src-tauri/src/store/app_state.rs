use crate::store::app_config::AppConfig;
use serde_json::json;
use std::sync::Arc;
use std::sync::RwLock;
#[allow(unused_imports)]
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_store::StoreExt;

#[derive(Clone)]
pub struct AppStateManager {
    store_key: String,
}

impl AppStateManager {
    pub fn new(store_key: impl Into<String>) -> Self {
        Self { store_key: store_key.into() }
    }

    #[cfg(debug_assertions)]
    pub fn clear_store_dev(&self, app: &AppHandle) {
        if let Ok(app_data_dir) = app.path().app_data_dir() {
            let store_path = app_data_dir.join("store.json");
            if store_path.exists() {
                match std::fs::remove_file(&store_path) {
                    Ok(_) => log::info!("🧹 [dev] cleared store.json at {}", store_path.display()),
                    Err(e) => log::error!("⚠️ [dev] failed to remove store.json at {}: {e}", store_path.display()),
                }
            } else {
                log::info!("🧹 [dev] store.json not found at {}, nothing to clear", store_path.display());
            }
        }
    }

    pub fn init_app_config_state(&self, app: &AppHandle) -> Result<AppConfigState, Box<dyn std::error::Error>> {
        let config = self.load(app)?;
        let state = Arc::new(RwLock::new(config));
        let new_manager = AppStateManager::new(self.store_key.clone());
        Ok(AppConfigState::new(state, new_manager, app.clone()))
    }

    #[cfg(debug_assertions)]
    fn print_store_path(&self, app: &AppHandle) {
        let app_data_dir = app.path().app_data_dir().expect("Failed to get app data directory");
        let store_path = app_data_dir.join("store.json");
        log::debug!("📁 store.json path: {}", store_path.to_string_lossy());
    }

    fn load(&self, app: &AppHandle) -> Result<AppConfig, Box<dyn std::error::Error>> {
        let store = app.store("store.json")?;
        #[cfg(debug_assertions)]
        self.print_store_path(app);
        if let Some(value) = store.get(&self.store_key) {
            // 兼容旧版本 store.json：如果配置结构变更导致反序列化失败，
            // 则回退到默认配置并覆盖存储，避免应用在 setup 阶段直接崩溃。
            let config_result: Result<AppConfig, serde_json::Error> = serde_json::from_value(value.clone());
            match config_result {
                Ok(config) => Ok(config),
                Err(e) => {
                    log::error!("Failed to parse `{}` from store.json, reset to defaults: {e}", self.store_key);
                    let config = AppConfig::default();
                    self.save(app, &config)?;
                    Ok(config)
                }
            }
        } else {
            let config = AppConfig::default();
            self.save(app, &config)?;
            Ok(config)
        }
    }

    fn save<R: Runtime>(&self, app: &AppHandle<R>, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        let store = app.store("store.json")?;
        store.set(&self.store_key, json!(config));
        store.save().map_err(|e| e.into())
    }
}

#[derive(Clone)]
pub struct AppConfigState {
    inner: Arc<RwLock<AppConfig>>,
    manager: AppStateManager,
    app_handle: AppHandle<tauri::Wry>,
}

impl AppConfigState {
    fn new(inner: Arc<RwLock<AppConfig>>, manager: AppStateManager, app_handle: AppHandle<tauri::Wry>) -> Self {
        Self { inner, manager, app_handle }
    }

    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, AppConfig> {
        self.inner.read().unwrap()
    }

    pub fn update<F>(&self, f: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut AppConfig),
    {
        {
            let mut guard = self.inner.write().map_err(|e| format!("Failed to acquire write lock: {}", e))?;
            f(&mut guard);
        }
        let _ = self.manager.save(&self.app_handle, &self.inner.read().unwrap());
        Ok(())
    }
}
