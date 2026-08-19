use tauri::{
    AppHandle, Emitter, Manager, menu::{MenuBuilder, MenuItem}, tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}
};

use crate::{ai::commands::create_session, my_windows};

pub fn create_tray(app_handle: &AppHandle) -> tauri::Result<()> {
    #[rustfmt::skip]
    let menu = MenuBuilder::new(app_handle)
        .item(&MenuItem::with_id(app_handle, "show", "Show", true, None::<&str>)?)
        .item(&MenuItem::with_id(app_handle, "translate_bubble", "TranslateBubble", true, None::<&str>)?)
        .item(&MenuItem::with_id(app_handle, "create_session", "CreateSession", true, None::<&str>)?)
        .item(&MenuItem::with_id(app_handle, "sendmessage", "SendMessage", true, None::<&str>)?)
        .item(&MenuItem::with_id(app_handle, "quit", "Quit", true, None::<&str>)?)
        .build()?;

    let _tray = TrayIconBuilder::new()
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
                         my_windows::window_index::window_index_show(&tray.app_handle(), Some(move || {}));
                    }
                    _ => {}
                }
            }
        })
        .build(app_handle)?;

    app_handle.on_menu_event(|app, event| match event.id().as_ref() {
        "show" => {}
        "translate_bubble" => {}
        "create_session" => {
            if let Some(window) = app.get_webview_window("index") {
                if let Ok((session_id, _)) = create_session(app.clone()) {
                    let _ = window.emit("on_create_session", session_id);
                }
            }
        }
        "sendmessage" => {
            // 发送消息：向主窗口 emit 一个 on_message 事件，前端 chatInit.tsx 会监听并处理
            crate::utils::send_tray_message::send_tray_message(app);
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    });

    Ok(())
}
