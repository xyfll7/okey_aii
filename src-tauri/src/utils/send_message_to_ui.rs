use std::time::SystemTime;

use rig::message::UserContent;
use tauri::{AppHandle, Emitter, Manager};

use crate::ai::{commands::list_sessions, state::HistoryItem};
use crate::store::app_state::AppConfigState;

/// 取 `translator` preset 第三条 prompt tag（id=2，right_ctrl）的 content 作为翻译指令。
fn translate_prompt(app: &AppHandle) -> String {
    let state = app.state::<AppConfigState>();
    let config = state.read();
    config
        .agent_presets
        .iter()
        .find(|p| p.id == "translator")
        .and_then(|preset| preset.prompt_tags.get(2))
        .and_then(|tag| tag.content.clone())
        .unwrap_or_default()
}

pub fn send_message_to_ui(app: &AppHandle, selected_text: String, target: String) {
    let translate_instruction = translate_prompt(app);
    let message: rig::message::Message = vec![
        UserContent::text(selected_text),
        UserContent::text(translate_instruction),
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
