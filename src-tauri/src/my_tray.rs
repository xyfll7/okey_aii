use tauri::{
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, menu::{MenuBuilder, MenuItem}, tray::TrayIconBuilder
};

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
            if let Some(window) = app.get_webview_window("index") {
                let _ = window.show();
                let _ = window.set_focus();
            } else {
                match WebviewWindowBuilder::new(app, "index", WebviewUrl::App("/".into()))
                    .resizable(true)
                    .build()
                {
                    Ok(window) => {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    Err(e) => eprintln!("failed to create window: {e}"),
                }
            }
        }
        "test" => {
            tauri::async_runtime::spawn(async move {});
        }
        "quit" => app.exit(0),
        _ => {}
    });

    Ok(())
}
