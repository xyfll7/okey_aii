use std::time::SystemTime;

use rig::message::UserContent;
use tauri::{AppHandle, Emitter};

use crate::ai::{commands::list_sessions, state::HistoryItem};


pub fn send_message_to_ui(app: &AppHandle, selected_text: String, target: String) {
    let message: rig::message::Message = vec![
        UserContent::text(selected_text),
        UserContent::text("Translate the above content into English or Chinese"),
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
