use tauri::{AppHandle, LogicalSize, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri::{window::Color,   PhysicalPosition, Runtime, };
use mouse_position::mouse_position::{Mouse, Position};
use tauri::Monitor;

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




pub const WINDOW_HEIGHT_TRANSLATE_BUBBLE: f64 = [32.0, 34.0][cfg!(target_os = "macos") as usize];
pub fn window_translate_bubble_show<R: Runtime, F>(app: &AppHandle<R>, callback: Option<F>)
where
    F: FnOnce() + Send + 'static,
{
    const WINDOW_WIDTH: f64 = 170.0;
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

        if let Some(cb) = callback {
            cb();
        }
    }
}



fn calculate_window_position<R: Runtime>(app: &AppHandle<R>, width: f64, height: f64, cursor_offset: f64) -> (f64, f64) {
    let mouse_position = match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => Position { x, y },
        Mouse::Error => Position { x: 0, y: 0 },
    };

    match get_monitor_at_position(app, mouse_position.x, mouse_position.y) {
        Some(monitor) => {
            let scale_factor = monitor.scale_factor();

            let to_logical = |value: i32| value as f64 / scale_factor;
            let to_logical_f = |value: u32| value as f64 / scale_factor;

            let mouse_x = to_logical(mouse_position.x);
            let mouse_y = to_logical(mouse_position.y);

            let monitor_x = to_logical(monitor.position().x);
            let monitor_y = to_logical(monitor.position().y);
            let monitor_width = to_logical_f(monitor.size().width);
            let monitor_height = to_logical_f(monitor.size().height);
            let monitor_right = monitor_x + monitor_width;
            let monitor_bottom = monitor_y + monitor_height;

            let relative_x = (mouse_x - monitor_x) / monitor_width;
            let relative_y = (mouse_y - monitor_y) / monitor_height;

            let x = if relative_x < 0.5 {
                let right_pos = mouse_x + cursor_offset;
                if right_pos + width <= monitor_right {
                    right_pos
                } else {
                    (mouse_x - width - cursor_offset).max(monitor_x)
                }
            } else {
                let left_pos = mouse_x - width - cursor_offset;
                if left_pos >= monitor_x {
                    left_pos
                } else {
                    (mouse_x + cursor_offset).min(monitor_right - width)
                }
            };

            let y = if relative_y < 0.5 {
                let bottom_pos = mouse_y + cursor_offset;
                if bottom_pos + height <= monitor_bottom {
                    bottom_pos
                } else {
                    (mouse_y - height - cursor_offset).max(monitor_y)
                }
            } else {
                let top_pos = mouse_y - height - cursor_offset;
                if top_pos >= monitor_y {
                    top_pos
                } else {
                    (mouse_y + cursor_offset).min(monitor_bottom - height)
                }
            };

            let x = x.clamp(monitor_x, monitor_right - width);
            let y = y.clamp(monitor_y, monitor_bottom - height);

            (x, y)
        }
        None => {
            if let Ok(Some(monitor)) = app.primary_monitor() {
                let scale = monitor.scale_factor();
                let logical_x = mouse_position.x as f64 / scale;
                let logical_y = mouse_position.y as f64 / scale;
                (logical_x, logical_y)
            } else {
                (mouse_position.x as f64, mouse_position.y as f64)
            }
        }
    }
}


fn get_monitor_at_position<R: Runtime>(app: &AppHandle<R>, x: i32, y: i32) -> Option<Monitor> {
    if let Ok(monitors) = app.available_monitors() {
        for monitor in monitors {
            let size = monitor.size();
            let position = monitor.position();

            if x >= position.x && x < position.x + size.width as i32 && y >= position.y && y < position.y + size.height as i32 {
                return Some(monitor);
            }
        }
    }

    app.primary_monitor().ok().flatten()
}
