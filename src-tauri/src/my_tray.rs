use rig::{message::UserContent, OneOrMany};
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, menu::{MenuBuilder, MenuItem}, tray::TrayIconBuilder
};

use crate::{
    ai::{
        commands::{create_session, list_sessions},
        state::add_message_to_history,
    },
    my_windows::window_helper::open_window, utils::calculate_text_width,
};

pub fn create_tray(app_handle: &AppHandle) -> tauri::Result<()> {
    #[rustfmt::skip]
    let menu = MenuBuilder::new(app_handle)
        .item(&MenuItem::with_id(app_handle, "show", "Show", true, None::<&str>)?)
        .item(&MenuItem::with_id(app_handle, "translate_bubble", "TranslateBubble", true, None::<&str>)?)
        .item(&MenuItem::with_id(app_handle, "create_session", "CreateSession", true, None::<&str>)?)
        .item(&MenuItem::with_id(app_handle, "drawertest", "Drawertest", true, None::<&str>)?)
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
        "translate_bubble" => {
            let _ = open_window(app, "translate_bubble", "/translate_bubble");

            let size = calculate_text_width::calculate_text_width("你好啊我是大王啊");
            if let Some(window) = app.get_webview_window("translate_bubble") {
                let _ = window.set_size(size);
                let _ = window.set_min_size(Some(size));
                let _ = window.set_max_size(Some(LogicalSize::new(10_000.0, size.height)));
            }
        }
        "create_session" => {
            if let Some(window) = app.get_webview_window("index") {
                if let Ok((session_id, _)) = create_session(app.clone()) {
                    let _ = window.emit("on_create_session", session_id);
                }
            }
        }
        "drawertest" => {
            // 发送消息：向主窗口 emit 一个 on_message 事件，前端 chatInit.tsx 会监听并处理
            if let Some(window) = app.get_webview_window("index") {
                let user_content = OneOrMany::many([
                    UserContent::text("请将下面的内容翻译成英文"),
                    UserContent::text("这是一个来自托盘菜单的示例文本"),
                ]);
                if let Ok(user_content) = user_content {
                    let message: rig::message::Message = user_content.into();
                    let session_id = {
                        let list = list_sessions(app.clone());
                        list.last().map(|s| s.session_id.clone())
                    };
                    if let Some(session_id) = session_id {
                        let _ = add_message_to_history(app, session_id.clone(), message.clone());
                        let _ = window.emit(&format!("on_message_{session_id}"), message);
                    }
                }
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    });

    Ok(())
}
