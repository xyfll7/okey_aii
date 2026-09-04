use crate::ai::commands::{close_session, create_session, list_sessions};
use crate::ai::model_catalog::ModelCatalogState;
use crate::ai::state::ChatState;
use crate::my_commands::translate_prompt;
use crate::my_windows::window_index::should_use_existing_index_window;
use crate::store::app_state::AppStateManager;
use crate::utils::send_message_to_ui::send_message_to_ui;
use crate::{my_rdev, my_tray, my_windows};
use std::sync::{Arc, RwLock};
use tauri::{Emitter, Manager};

pub fn init(app: &mut tauri::App) {
    init_state(app);
    setup_ai_state(app);
    setup_tray_and_activation_policy(app);
    my_rdev::init_global_input_listener(
        app.handle(),
        |app| {
            let app = app.clone();
            let crate::utils::selecte_text::SelectedContent {
                selected_text,
                selected_files,
            } = crate::utils::selecte_text::get_selected_content();

            if should_use_existing_index_window(app.clone()) {
                let app_clone = app.clone();
                my_windows::window_index::window_index_show(
                    &app,
                    Some(move || {
                        send_message_to_ui(
                            &app_clone,
                            selected_text,
                            selected_files,
                            "index".to_string(),
                        );
                    }),
                );
            } else {
                // Close all existing sessions, then create a fresh one before
                // showing the translate bubble, so the UI always opens a clean session.
                let sessions = list_sessions(app.clone());
                for session in sessions {
                    if let Err(e) = close_session(app.clone(), session.session_id) {
                        log::error!("failed to close session: {e}");
                    }
                }
                match tauri::async_runtime::block_on(create_session(app.clone())) {
                    Ok((session_id, _)) => {
                        // Pass the selected text and the translation instruction
                        // along with the session id so the translate bubble can
                        // render the user message directly.
                        let translate_instruction = translate_prompt(app.clone());
                        let _ = app.emit_to(
                            "translate_bubble",
                            "on_open_session_with_session_id",
                            serde_json::json!({
                                "session_id": session_id,
                                "user_contents": [selected_text, translate_instruction],
                            }),
                        );
                    }
                    Err(e) => log::error!("failed to create new session: {e}"),
                }

                my_windows::window_translate_bubble::window_translate_bubble_show(&app);
            };
        },
        |app, x, y| {
            my_windows::window_translate_bubble::window_translate_bubble_hide_if_outside(app, x, y);
        },
    );
}

fn init_state(app: &mut tauri::App) {
    let state_manager = AppStateManager::new("app_config");
    let app_config_state = state_manager
        .init_app_config_state(app.handle())
        .expect("failed to init app config state");

    let config = app_config_state.read();
    rust_i18n::set_locale(&config.language.to_locale());
    drop(config);

    app.manage(app_config_state);
    app.manage(ModelCatalogState::default());
}

fn setup_tray_and_activation_policy(app: &mut tauri::App) {

    let tray = my_tray::create_tray(&app.handle()).expect("failed to create tray");
    app.handle().manage(my_tray::TrayState {
        tray: std::sync::Arc::new(tray),
    });
    #[cfg(target_os = "macos")]
    {
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }
}

pub fn setup_ai_state(app: &mut tauri::App) {
    let initial = ChatState {
        sessions: std::collections::HashMap::from([]),
        db: crate::ai::db::open(
            app.path()
                .app_data_dir()
                .map(|p| p.join("okey_aii.db"))
                .unwrap_or_else(|_| std::path::PathBuf::from("okey_aii.db")),
        )
        .unwrap_or_else(|e| panic!("failed to open database: {e}")),
    };
    app.manage(Arc::new(RwLock::new(initial)));

    // Block on the initial session so the UI always finds one via
    // `list_sessions` once the window loads. The session's model comes from
    // the provider's live listing; it fails gracefully (and logs) when no
    // model can be fetched, e.g. before an API key is configured.
    if let Err(e) = tauri::async_runtime::block_on(create_session(app.handle().clone())) {
        log::error!("failed to create initial session: {e}");
    }
}
