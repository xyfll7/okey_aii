use std::time::SystemTime;

use rig::message::UserContent;
use tauri::{AppHandle, Emitter, Manager};

use crate::ai::{commands::list_sessions, state::HistoryItem};
use crate::store::app_state::AppConfigState;

/// 取 `translator` preset 的快捷翻译指令：
/// `self_explaining_model` 开启时取 id=3（自我解释），否则取 id=2（right_ctrl 翻译）。
fn translate_prompt(app: &AppHandle) -> String {
    let state = app.state::<AppConfigState>();
    let config = state.read();
    let tag_id = if config.self_explaining_model { Some(3) } else { Some(2) };
    config
        .agent_presets
        .iter()
        .find(|p| p.id == "translator")
        .and_then(|preset| preset.prompt_tags.iter().find(|tag| tag.id == tag_id))
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
