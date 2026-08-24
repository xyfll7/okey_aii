use crate::ai::config::Provider;
use crate::store::app_config::{AutoSpeakState, Language, PromptTag};
use crate::store::app_state::AppConfigState;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[tauri::command(rename_all = "snake_case")]
pub fn get_api_keys(app: AppHandle) -> HashMap<Provider, String> {
    let state = app.state::<AppConfigState>();
    let keys = state.read().api_keys.clone();
    keys
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_api_key(app: AppHandle, provider: Provider, api_key: String) -> Result<(), String> {
    let state = app.state::<AppConfigState>();
    state
        .update(|config| {
            config.api_keys.insert(provider, api_key);
        })
        .map_err(|e| e.to_string())
}

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

#[tauri::command(rename_all = "snake_case")]
pub fn get_prompt_tags(app: AppHandle) -> Vec<PromptTag> {
    let state = app.state::<AppConfigState>();
    let tags = state.read().prompt_tags.clone();
    tags
}

#[tauri::command(rename_all = "snake_case")]
pub fn add_prompt_tag(app: AppHandle, label: String, content: String) -> Result<Vec<PromptTag>, String> {
    let state = app.state::<AppConfigState>();
    let mut new_tags = state.read().prompt_tags.clone();
    let next_id = new_tags.iter().filter_map(|t| t.id).max().map_or(0, |m| m + 1);
    new_tags.push(PromptTag {
        raw: None,
        label: Some(label),
        content: Some(content),
        id: Some(next_id),
    });
    state
        .update(|config| config.prompt_tags = new_tags.clone())
        .map_err(|e| e.to_string())?;
    Ok(new_tags)
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_prompt_tag(app: AppHandle, id: u32, label: String, content: String) -> Result<Vec<PromptTag>, String> {
    let state = app.state::<AppConfigState>();
    let mut new_tags = state.read().prompt_tags.clone();
    let Some(tag) = new_tags.iter_mut().find(|t| t.id == Some(id)) else {
        return Err(format!("Prompt tag {id} not found"));
    };
    tag.label = Some(label);
    tag.content = Some(content);
    state
        .update(|config| config.prompt_tags = new_tags.clone())
        .map_err(|e| e.to_string())?;
    Ok(new_tags)
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_prompt_tag(app: AppHandle, id: u32) -> Result<Vec<PromptTag>, String> {
    let state = app.state::<AppConfigState>();
    let mut new_tags = state.read().prompt_tags.clone();
    new_tags.retain(|t| t.id != Some(id));
    state
        .update(|config| config.prompt_tags = new_tags.clone())
        .map_err(|e| e.to_string())?;
    Ok(new_tags)
}

/// 可选的翻译语言列表，返回 `(locale, 显示名)` 二元组。
/// 显示名跟随当前 UI 语言（i18n），由 `Language::to_display_name()` 提供。
#[tauri::command(rename_all = "snake_case")]
pub fn get_language_options() -> Vec<(String, String)> {
    use crate::store::app_config::Language;
    vec![
        (Language::ZhCn.to_locale(), Language::ZhCn.to_display_name()),
        (Language::En.to_locale(), Language::En.to_display_name()),
    ]
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_local_language(app: AppHandle) -> Language {
    app.state::<AppConfigState>().read().local_language
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_local_language(app: AppHandle, language: Language) -> Result<Language, String> {
    let state = app.state::<AppConfigState>();
    state
        .update(|config| config.local_language = language)
        .map_err(|e| e.to_string())?;
    Ok(language)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_target_language(app: AppHandle) -> Language {
    app.state::<AppConfigState>().read().target_language
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_target_language(app: AppHandle, language: Language) -> Result<Language, String> {
    let state = app.state::<AppConfigState>();
    state
        .update(|config| config.target_language = language)
        .map_err(|e| e.to_string())?;
    Ok(language)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_self_explaining_model(app: AppHandle) -> bool {
    app.state::<AppConfigState>().read().self_explaining_model
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_self_explaining_model(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let state = app.state::<AppConfigState>();
    state
        .update(|config| config.self_explaining_model = enabled)
        .map_err(|e| e.to_string())?;
    Ok(enabled)
}
