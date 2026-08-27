use std::sync::Arc;
use tauri::{
    AppHandle, Emitter, Manager, menu::{CheckMenuItem, MenuBuilder, MenuItem}, tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent}
};

use crate::{ai::commands::create_session, my_windows};
use tauri_plugin_autostart::AutoLaunchManager;

/// State to store the tray icon for later retrieval
pub struct TrayState {
    pub tray: Arc<TrayIcon<tauri::Wry>>,
}

/*******  ab7e53dc-7cba-45e1-8b3a-3837c9b2580a  *******/
pub fn create_tray(app_handle: &AppHandle) -> tauri::Result<TrayIcon<tauri::Wry>> {
    // Create tray menu
    let menu = build_tray_menu(app_handle)?;

    // Create system tray
    let tray = TrayIconBuilder::new()
        .icon(app_handle.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                match button {
                    MouseButton::Left => {
                        my_windows::window_index::window_index_show(
                            &tray.app_handle(),
                            None as Option<fn()>,
                        );
                    }
                    _ => {}
                }
            }
        })
        .build(app_handle)?;

    app_handle.on_menu_event(move |app, event| match event.id().as_ref() {
        "show" => {
             my_windows::window_index::window_index_show(&app, Some(move || {}));
        }
        "test" => {
            if let Some(window) = app.get_webview_window("index") {
                if let Ok((session_id, _)) = create_session(app.clone()) {
                    let _ = window.emit("on_open_session_with_session_id", session_id);
                }
            }
        }
        "autostart" => {
            let autostart_manager = app.state::<AutoLaunchManager>();
            // Toggle autostart state
            let current_enabled = autostart_manager.is_enabled().unwrap_or(false);
            if current_enabled {
                let _ = autostart_manager.disable();
            } else {
                let _ = autostart_manager.enable();
            }
        }
        "quit" => std::process::exit(0),
        _ => {}
    });

    Ok(tray)
}

/// Build the tray menu with current translations
fn build_tray_menu(app_handle: &AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    // Create menu items with translations
    let show_item = MenuItem::with_id(app_handle, "show", rust_i18n::t!("show"), true, None::<&str>)?;
    let test_item = MenuItem::with_id(app_handle, "test", rust_i18n::t!("test"), true, None::<&str>)?;

    // Create autostart menu item
    let autostart_manager = app_handle.state::<AutoLaunchManager>();
    let is_auto_start = autostart_manager.is_enabled().unwrap_or(false);
    let autostart_item = CheckMenuItem::with_id(
        app_handle,
        "autostart",
        rust_i18n::t!("autostart"),
        true,
        is_auto_start,
        None::<&str>,
    )?;

    let quit_item = MenuItem::with_id(app_handle, "quit", rust_i18n::t!("quit"), true, None::<&str>)?;

    // Build menu
    let menu = MenuBuilder::new(app_handle)
        .item(&show_item)
        .item(&test_item)
        .item(&autostart_item)
        .separator()
        .item(&quit_item)
        .build()?;

    Ok(menu)
}

/// Rebuild the tray menu with new locale
pub fn rebuild_tray_menu(app_handle: &AppHandle) -> tauri::Result<()> {
    // Get the tray state
    let tray_state = app_handle.state::<TrayState>();
    // Build new menu with updated translations
    let new_menu = build_tray_menu(app_handle)?;
    // Set the new menu
    tray_state.tray.set_menu(Some(new_menu))?;
    Ok(())
}
