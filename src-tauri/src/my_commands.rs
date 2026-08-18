use crate::{my_windows, utils::language_detection};
use tauri::AppHandle;

#[tauri::command]
pub fn detect_language(text: &str) -> String {
    let language = language_detection::detect_language(text);
    language.to_string()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn open_window_index(app: AppHandle) {
    my_windows::window_index::window_index_show(&app, Some(move || {}));
}
