use crate::store::app_config::AutoSpeakState;
use crate::store::app_state::AppConfigState;
use tauri::{AppHandle, Manager};

#[tauri::command(rename_all = "snake_case")]
pub fn get_pin_index_window(app: AppHandle) -> bool {
    let state = app.state::<AppConfigState>();
    let pinned = state.read().is_pin_index_window;
    pinned
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_pin_index_window(app: AppHandle, pinned: bool) -> Result<bool, String> {
    let state = app.state::<AppConfigState>();
    state
        .update(|config| config.is_pin_index_window = pinned)
        .map_err(|e| e.to_string())?;

    if let Some(window) = app.get_webview_window("index") {
        let _ = window.set_always_on_top(pinned);
    }
    Ok(pinned)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_auto_speak(app: AppHandle) -> AutoSpeakState {
    let state = app.state::<AppConfigState>();
    let auto_speak = state.read().auto_speak;
    auto_speak
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_auto_speak(app: AppHandle, auto_speak: AutoSpeakState) -> Result<AutoSpeakState, String> {
    let state = app.state::<AppConfigState>();
    state
        .update(|config| config.auto_speak = auto_speak)
        .map_err(|e| e.to_string())?;
    Ok(auto_speak)
}
