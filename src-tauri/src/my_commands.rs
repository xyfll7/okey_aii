use crate::store::app_state::AppConfigState;
use crate::{my_windows, utils::language_detection};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn detect_language(text: &str) -> String {
    let language = language_detection::detect_language(text);
    language.to_string()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn open_window_index(app: AppHandle) {
    my_windows::window_index::window_index_show(&app, Some(move || {}));
}

#[tauri::command(rename_all = "snake_case")]
pub fn translate_prompt(app: AppHandle) -> String {
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
