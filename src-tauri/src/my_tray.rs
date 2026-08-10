use tauri::{
    menu::{MenuBuilder, MenuItem},
    tray::TrayIconBuilder,
    AppHandle,
};

use crate::window::open_window;

pub fn create_tray(app_handle: &AppHandle) -> tauri::Result<()> {
    #[rustfmt::skip]
    let menu = MenuBuilder::new(app_handle)
        .item(&MenuItem::with_id(app_handle, "show", "Show", true, None::<&str>)?)
        .item(&MenuItem::with_id(app_handle, "test", "Test", true, None::<&str>)?)
        .item(&MenuItem::with_id(app_handle, "quit", "Quit", true, None::<&str>)?)
        .build()?;

    let _tray = TrayIconBuilder::new()
        .icon(app_handle.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .build(app_handle)?;

    app_handle.on_menu_event(|app, event| match event.id().as_ref() {
        "show" => {
            let _ = open_window(app, "index", "/");
        }
        "test" => {
            let _ = open_window(app, "translate", "/translate");
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    });

    Ok(())
}
