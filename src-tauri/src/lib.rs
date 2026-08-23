rust_i18n::i18n!("locales");
mod ai;
mod my_commands;
mod my_init;
mod my_rdev;
mod my_tray;
mod my_types;
mod my_windows;
mod store;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .filter(utils::log_filter::log_filter)
                        .build(),
                )?;
            }
            my_init::init(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ai::commands::list_models,
            ai::commands::create_session,
            ai::commands::new_session,
            ai::commands::close_session,
            ai::commands::delete_session,
            ai::commands::list_sessions,
            ai::commands::switch_provider,
            ai::commands::switch_model,
            ai::commands::send_message,
            ai::commands::stop_generation,
            ai::commands::get_history,
            ai::commands::remove_history_item,
            ai::commands::list_history_sessions,
            ai::commands::open_session,
            my_commands::detect_language,
            my_commands::open_window_index,
            store::commands::get_pin_index_window,
            store::commands::set_pin_index_window,
            store::commands::get_auto_speak,
            store::commands::set_auto_speak,
            utils::i18n::get_current_locale,
            utils::i18n::set_current_locale,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
