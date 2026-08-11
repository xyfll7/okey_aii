use tauri::{
    menu::{MenuBuilder, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
    AppHandle,
};

use crate::window::open_window;

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
                let _ = window.emit(
                    "on_message",
                    serde_json::json!({
                        "translation_prompt": "请将下面的内容翻译成英文",
                        "selected_text": "这是一个来自托盘菜单的示例文本",
                    }),
                );
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    });

    Ok(())
}
