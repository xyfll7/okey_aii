use rig::{message::UserContent, OneOrMany};
use tauri::{
    menu::{MenuBuilder, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

use crate::{
    ai::commands::create_session, ai::state::add_message_to_history, window::open_window,
};

pub fn create_tray(app_handle: &AppHandle) -> tauri::Result<()> {
    #[rustfmt::skip]
    let menu = MenuBuilder::new(app_handle)
        .item(&MenuItem::with_id(app_handle, "show", "Show", true, None::<&str>)?)
        .item(&MenuItem::with_id(app_handle, "test", "Test", true, None::<&str>)?)
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
        "test" => {
            let _ = open_window(app, "translate", "/translate");
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

                    // 调用 create_session 创建一个新会话,拿到 session_id 后写入消息
                    let state: tauri::State<'_, std::sync::Arc<std::sync::RwLock<crate::ai::state::ChatState>>> =
                        app.state();
                    if let Ok((session_id, _)) = create_session(state) {
                        let _ = add_message_to_history(app, session_id.clone(), message.clone());
                        let _ = window.emit("on_message", message);
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
