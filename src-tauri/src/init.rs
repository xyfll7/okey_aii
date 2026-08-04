use std::sync::{Arc, RwLock};

use tauri::Manager;

use crate::ai::config::Provider;
use crate::ai::state::ChatState;
use crate::my_tray;

pub fn init(app: &mut tauri::App) {
    setup_ai_state(app);
    setup_tray_and_activation_policy(app);
}

/// 创建系统托盘，并在 macOS 上设置为 accessory 激活策略
fn setup_tray_and_activation_policy(app: &mut tauri::App) {
    my_tray::create_tray(app.handle()).expect("failed to create tray");

    #[cfg(target_os = "macos")]
    {
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }
}

/// 构建初始的 AI 状态，并注册为 Tauri 托管状态，供 command 通过 State 获取
pub fn setup_ai_state(app: &mut tauri::App) {
    let initial_key = "sk-a36321b7ed3c47c88d6e6f371550e6f9";
    let initial = ChatState {
        provider: Provider::DeepSeek,
        model: crate::ai::config::default_model(Provider::DeepSeek).to_string(),
        preset_id: "assistant".to_string(),
        api_keys: std::collections::HashMap::from([(Provider::DeepSeek, initial_key.to_string())]),
        agent: Arc::new(
            crate::ai::state::build_agent(
                Provider::DeepSeek,
                crate::ai::config::default_model(Provider::DeepSeek),
                initial_key,
                &crate::ai::config::builtin_presets()
                    .into_iter()
                    .find(|p| p.id == "assistant")
                    .expect("assistant preset must exist"),
            )
            .expect("failed to build initial agent"),
        ),
        history: Vec::new(),
    };
    app.manage(Arc::new(RwLock::new(initial)));
}
