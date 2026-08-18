use mouse_position::mouse_position::{Mouse, Position};
use tauri::{AppHandle, Monitor, Runtime};

pub fn get_monitor_at_position<R: Runtime>(app: &AppHandle<R>, x: i32, y: i32) -> Option<Monitor> {
    if let Ok(monitors) = app.available_monitors() {
        for monitor in monitors {
            let size = monitor.size();
            let position = monitor.position();

            if x >= position.x
                && x < position.x + size.width as i32
                && y >= position.y
                && y < position.y + size.height as i32
            {
                return Some(monitor);
            }
        }
    }

    app.primary_monitor().ok().flatten()
}

pub fn calculate_center_position<R: Runtime>(
    app: &AppHandle<R>,
    width: f64,
    height: f64,
) -> (f64, f64) {
    if let Ok(Some(primary_monitor)) = app.primary_monitor() {
        let scale_factor = primary_monitor.scale_factor();

        let monitor_position = primary_monitor.position();
        let monitor_size = primary_monitor.size();

        let monitor_x = monitor_position.x as f64 / scale_factor;
        let monitor_y = monitor_position.y as f64 / scale_factor;
        let monitor_width = monitor_size.width as f64 / scale_factor;
        let monitor_height = monitor_size.height as f64 / scale_factor;

        let x = monitor_x + (monitor_width - width) / 2.0;
        let y = monitor_y + (monitor_height - height) / 2.0;

        (x, y)
    } else {
        (0.0, 0.0)
    }
}

pub fn calculate_window_position<R: Runtime>(
    app: &AppHandle<R>,
    width: f64,
    height: f64,
    cursor_offset: f64,
) -> (f64, f64) {
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
