use tauri::{AppHandle, LogicalSize, Manager};
use tauri::{window::Color, PhysicalPosition, Runtime};
use crate::my_windows::window_helper::*;


pub const WINDOW_HEIGHT_TRANSLATE_BUBBLE: f64 = [32.0, 34.0][cfg!(target_os = "macos") as usize];
pub fn window_translate_bubble_show<R: Runtime>(app: &AppHandle<R>) {
    const WINDOW_WIDTH: f64 = 270.0;
    const CURSOR_OFFSET: f64 = 17.0;

    if let Some(window) = app.get_webview_window("translate_bubble") {
        let size = LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT_TRANSLATE_BUBBLE);
        let _ = window.set_size(size);
        let _ = window.set_min_size(Some(size));
        let _ = window.set_background_color(Some(Color(0, 0, 0, 1)));
        let _ = window.set_max_size(Some(LogicalSize::new(10_000.0, WINDOW_HEIGHT_TRANSLATE_BUBBLE)));

        let (logical_x, logical_y) = calculate_window_position(app, WINDOW_WIDTH, WINDOW_HEIGHT_TRANSLATE_BUBBLE, CURSOR_OFFSET);

        #[cfg(target_os = "macos")]
        {
            let _ = window.set_shadow(false);
        }

        let mut target_scale = 1.0;

        if let Ok(monitors) = window.available_monitors() {
            for monitor in monitors {
                let pos = monitor.position();
                let size_mon = monitor.size();
                let scale = monitor.scale_factor();

                let mon_x = pos.x as f64 / scale;
                let mon_y = pos.y as f64 / scale;
                let mon_w = size_mon.width as f64 / scale;
                let mon_h = size_mon.height as f64 / scale;

                if logical_x >= mon_x && logical_x < mon_x + mon_w && logical_y >= mon_y && logical_y < mon_y + mon_h {
                    target_scale = scale;
                    break;
                }
            }
        }

        let physical_x = (logical_x * target_scale) as i32;
        let physical_y = (logical_y * target_scale) as i32;

        let _ = window.set_position(tauri::Position::Physical(PhysicalPosition { x: physical_x, y: physical_y }));

        let _ = window.show();
        let _ = window.set_always_on_top(true);
    }
}

pub fn window_translate_bubble_hide_if_outside<R: Runtime>(app: &AppHandle<R>, x: i32, y: i32) {
    if let Some(window) = app.get_webview_window("translate_bubble") {
        if window.is_visible().unwrap_or(false) {
            if let (Ok(pos), Ok(size)) = (window.outer_position(), window.outer_size()) {
                let win_x = pos.x;
                let win_y = pos.y;
                let win_w = size.width as i32;
                let win_h = size.height as i32;

                let inside =
                    x >= win_x && x <= win_x + win_w && y >= win_y && y <= win_y + win_h;

                if !inside {
                    let _ = window.hide();
                }
            }
        }
    }
}
