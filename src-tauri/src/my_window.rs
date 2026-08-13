use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub fn open_window(app: &AppHandle, label: &str, url: &str) -> tauri::Result<WebviewWindow> {
    let window = match app.get_webview_window(label) {
        Some(w) => w,
        None => WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
            .resizable(true)
            .build()?,
    };
    window.show()?;
    window.set_focus()?;
    Ok(window)
}
