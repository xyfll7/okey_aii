use crate::ai::commands::create_session;
use crate::ai::config::Provider;
use crate::ai::state::{AppConfig, ChatState};
use crate::my_windows::window_index::should_use_existing_index_window;
use crate::utils::send_message_to_ui::send_message_to_ui;
use crate::{my_rdev, my_tray, my_windows};
use std::sync::{Arc, RwLock};
use tauri::Manager;

pub fn init(app: &mut tauri::App) {
    setup_ai_state(app);
    setup_tray_and_activation_policy(app);
    my_rdev::init_global_input_listener(
        app.handle(),
        |app| {
            let app = app.clone();
            let selected_text = crate::utils::selecte_text::get_selected_text();
            println!("你今天吃了吗？::{}",selected_text);
            if should_use_existing_index_window(app.clone()) {
                let app_clone = app.clone();
                my_windows::window_index::window_index_show(
                    &app,
                    Some(move || {
                        send_message_to_ui(&app_clone, selected_text);
                    }),
                );
            } else {
                let app_clone = app.clone();
                my_windows::window_translate_bubble::window_translate_bubble_show(
                    &app,
                    Some(move || {
                        send_message_to_ui(&app_clone, selected_text);
                    }),
                );
            };
        },
        |app, x, y| {
            my_windows::window_translate_bubble::window_translate_bubble_hide_if_outside(app, x, y);
        },
    );
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
    // 从 .env 读取 DEEPSEEK_API_KEY
    let _ = dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/.env"));
    let deepseek_key = std::env::var("DEEPSEEK_API_KEY")
        .unwrap_or_else(|_| panic!("DEEPSEEK_API_KEY not set in .env"));

    let api_keys = std::collections::HashMap::from([(Provider::DeepSeek, deepseek_key)]);

    let initial = ChatState {
        config: AppConfig { api_keys },
        sessions: std::collections::HashMap::from([]),
    };
    app.manage(Arc::new(RwLock::new(initial)));

    // 状态注册后，立即调用 create_session 预创建首个 session
    if let Err(e) = create_session(app.handle().clone()) {
        log::error!("failed to create initial session: {e}");
    }
}
