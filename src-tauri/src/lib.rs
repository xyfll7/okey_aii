mod ai;
mod my_commands;
mod my_init;
mod my_rdev;
mod my_tray;
mod my_windows;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
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
            ai::commands::list_models,
            ai::commands::create_session,
            ai::commands::close_session,
            ai::commands::list_sessions,
            ai::commands::switch_provider,
            ai::commands::switch_model,
            ai::commands::send_message,
            ai::commands::stop_generation,
            ai::commands::clear_history,
            ai::commands::get_history,
            ai::commands::remove_history_item,
            my_commands::detect_language,
            my_commands::open_window_index
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                // 仅当是窗口关闭（code 为 None）时才阻止退出，从而保留在托盘；
                // 通过托盘 Quit 主动调用 app.exit(0)（code 为 Some）时允许正常退出
                println!("退出code:{:?}",code);
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
