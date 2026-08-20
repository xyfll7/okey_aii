use std::time::SystemTime;

use rig::message::UserContent;
use tauri::{AppHandle, Emitter};

use crate::ai::{commands::list_sessions, state::HistoryItem};

/// 从托盘菜单发送一条示例消息到主窗口。
///
/// 逻辑：取最近一个会话，写入一条组合消息（示例文本 + 翻译指令 + 讲解指令），
/// 然后向主窗口 `index` emit 一个 `on_message_{session_id}` 事件，
/// 前端 `chatInit.tsx` 会监听并处理。
pub fn send_message_to_ui(app: &AppHandle, selected_text: String, target: String) {
    let message: rig::message::Message = vec![
        UserContent::text(selected_text),
        UserContent::text("请将上面的内容翻译成英文"),
    ]
    .into();
    let session_id = {
        let list = list_sessions(app.clone());
        list.last().map(|s| s.session_id.clone())
    };
    if let Some(session_id) = session_id {
        let item = HistoryItem {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: SystemTime::now(),
            message,
        };
        let _ = app.emit_to(target, &format!("on_message_{session_id}"), item);
    }
}
