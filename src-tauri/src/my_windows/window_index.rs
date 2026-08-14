use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tauri::Emitter;
use tauri::{window::Color, AppHandle, Listener, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

use crate::my_windows::window_helper::*;


pub fn window_translate_show<R: Runtime, F>(app: &AppHandle<R>, callback: Option<F>)
where
    F: FnOnce() + Send + 'static,
{
    if let Some(window) = app.get_webview_window("translate_bubble") {
        let _ = window.hide();
    }

    if let Some(window) = app.get_webview_window("translate") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.set_always_on_top(true);
        if let Some(cb) = callback {
            cb();
        }
    } else {
        const WINDOW_WIDTH: f64 = 400.0;
        const WINDOW_HEIGHT: f64 = 600.0;
        const CURSOR_OFFSET: f64 = 10.0;

        let (adjusted_x, adjusted_y) = if callback.is_none() { calculate_center_position(app, WINDOW_WIDTH, WINDOW_HEIGHT) } else { calculate_window_position(app, WINDOW_WIDTH, WINDOW_HEIGHT, CURSOR_OFFSET) };

        let mut builder = WebviewWindowBuilder::new(app, "translate", WebviewUrl::App("/translate".into())).title("Translate Window").resizable(true).fullscreen(false).skip_taskbar(true).always_on_top(true).min_inner_size(350.0, 600.0).background_color(Color(0, 0, 0, 1)).inner_size(WINDOW_WIDTH, WINDOW_HEIGHT).position(adjusted_x, adjusted_y);

        #[cfg(target_os = "macos")]
        {
            builder = builder.title_bar_style(tauri::TitleBarStyle::Overlay).hidden_title(true);
        }
        #[cfg(target_os = "linux")]
        {
            builder = builder.transparent(true)
        }
        #[cfg(not(target_os = "macos"))]
        {
            builder = builder.decorations(false);
        }

        let _ = builder.build().and_then(|window| {
            window.show().ok();
            window.set_focus().ok();
            let callback_for_listener = Arc::new(Mutex::new(callback)).clone();
            window.listen("event_names::PAGE_LOADED", move |_event| {
                if let Ok(mut cb_option) = callback_for_listener.lock() {
                    if let Some(cb) = cb_option.take() {
                        drop(cb_option);
                        cb();
                    }
                }
            });

            let state_handle = window.app_handle().clone();
            let cancelled = Arc::new(Mutex::new(false));
            let win_clone = window.clone();
            let cancel_flag = cancelled.clone();
            window.on_window_event(move |event| match event {
                tauri::WindowEvent::Focused(false) => {
                    *cancel_flag.lock().unwrap() = false;
                    let _win = win_clone.clone();
                    let local_cancel = cancel_flag.clone();
                    let state_handle = state_handle.clone();
                    thread::spawn(move || {
                        let _ = state_handle.emit("event_names::TRANSLATE_HIDE", {});
                        thread::sleep(Duration::from_millis(150));
                        if *local_cancel.lock().unwrap() {
                            return;
                        }
                        _win.destroy().ok();
                    });
                }
                tauri::WindowEvent::Focused(true) => {
                    *cancelled.lock().unwrap() = true;
                }
                tauri::WindowEvent::Moved(_) => {
                    *cancelled.lock().unwrap() = true;
                }
                _ => {}
            });
            Ok(())
        });
    }
}
