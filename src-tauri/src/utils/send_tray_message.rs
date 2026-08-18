use rig::{message::UserContent, OneOrMany};
use tauri::{AppHandle, Emitter};

use crate::ai::{commands::list_sessions, state::add_message_to_history};

/// 从托盘菜单发送一条示例消息到主窗口。
///
/// 逻辑：取最近一个会话，写入一条组合消息（示例文本 + 翻译指令 + 讲解指令），
/// 然后向主窗口 `index` emit 一个 `on_message_{session_id}` 事件，
/// 前端 `chatInit.tsx` 会监听并处理。
pub fn send_tray_message(app: &AppHandle) {
    let selected_text = crate::utils::selecte_text::get_selected_text();
    println!("add message ======::{:#?}", selected_text);
    let user_content = OneOrMany::many([
        UserContent::text(selected_text),
        UserContent::text("请将上面的内容翻译成英文"),
        UserContent::text("像是给初学者讲解一样"),
    ]);

    if let Ok(user_content) = user_content {
        let message: rig::message::Message = user_content.into();
        let session_id = {
            let list = list_sessions(app.clone());
            list.last().map(|s| s.session_id.clone())
        };
        println!("add message ::{:#?}", message);
        if let Some(session_id) = session_id {
            if let Ok(item) = add_message_to_history(app, session_id.clone(), message.clone()) {
                let _ = app.emit_to("index", &format!("on_message_{session_id}"), item);
            }
        }
    }
}
