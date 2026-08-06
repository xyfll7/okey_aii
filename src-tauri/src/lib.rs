mod ai;
mod my_init;
mod my_tray;

use crate::ai::commands::{clear_history, list_models, send_message, switch_model, switch_provider};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            my_init::init(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_models,
            switch_provider,
            switch_model,
            send_message,
            clear_history
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
