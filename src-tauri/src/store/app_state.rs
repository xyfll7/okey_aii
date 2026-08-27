use crate::store::app_config::AppConfig;
use serde_json::json;
use std::sync::Arc;
use std::sync::RwLock;
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

#[derive(Clone)]
pub struct AppStateManager {
    store_key: String,
}

impl AppStateManager {
    pub fn new(store_key: impl Into<String>) -> Self {
        Self { store_key: store_key.into() }
    }

    pub fn init_app_config_state(&self, app: &AppHandle) -> Result<AppConfigState, Box<dyn std::error::Error>> {
        let config = self.load(app)?;
        let state = Arc::new(RwLock::new(config));
        let new_manager = AppStateManager::new(self.store_key.clone());
        Ok(AppConfigState::new(state, new_manager, app.clone()))
    }

    #[cfg(debug_assertions)]
    fn load(&self, app: &AppHandle) -> Result<AppConfig, Box<dyn std::error::Error>> {
        // 开发模式：强制重置为默认配置，不读 store.json。
        // create_new() 会先从 tauri-plugin-store 的缓存（按路径单例）中移除旧 store，
        // 再新建空 store（不读磁盘），避免磁盘文件删了但内存缓存还在导致旧配置残留。
        app.store_builder("store.json").create_new().build()?;
        let config = AppConfig::default();
        self.save(app, &config)?;
        log::info!("🔄 [dev] reset `{}` to defaults", self.store_key);
        Ok(config)
    }

    #[cfg(not(debug_assertions))]
    fn load(&self, app: &AppHandle) -> Result<AppConfig, Box<dyn std::error::Error>> {
        self.load_from_store(app)
    }

    #[cfg(not(debug_assertions))]
    fn load_from_store(&self, app: &AppHandle) -> Result<AppConfig, Box<dyn std::error::Error>> {
        let store = app.store("store.json")?;
        if let Some(value) = store.get(&self.store_key) {
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

    pub fn update_and_save<F>(&self, f: F) -> Result<(), Box<dyn std::error::Error>>
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
