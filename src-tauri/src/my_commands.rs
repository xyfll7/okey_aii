use tauri::AppHandle;
use crate::{my_windows::window_helper::open_window, utils::language_detection};

#[tauri::command]
pub fn detect_language(text: &str) -> String {
    let language = language_detection::detect_language(text);
    language.to_string()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn open_window_index(app: AppHandle) {
    let _ = open_window(&app, "index", "/");
}