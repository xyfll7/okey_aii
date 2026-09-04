use std::time::SystemTime;

use rig::message::UserContent;
use tauri::{AppHandle, Emitter};

use crate::ai::{commands::list_sessions, state::HistoryItem};
use crate::my_commands::translate_prompt;

pub fn send_message_to_ui(
    app: &AppHandle,
    selected_text: String,
    selected_files: Vec<String>,
    target: String,
) {
    let translate_instruction = translate_prompt(app.clone());
    let mut content = vec![
        UserContent::text(selected_text),
        UserContent::text(translate_instruction),
    ];
    // TODO: 目前先把 Finder 选中的文件路径拼成一条文本发给 UI，
    // 后续可换成 document 附件或读取文件内容。
    if !selected_files.is_empty() {
        content.push(UserContent::text(selected_files.join(",")));
    }
    let message: rig::message::Message = content.into();
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
