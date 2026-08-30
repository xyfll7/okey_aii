use crate::ai::config::ProviderId;
use crate::store::app_config::{AutoSpeakState, Language, PromptTag};
use crate::store::app_state::AppConfigState;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[tauri::command(rename_all = "snake_case")]
pub fn get_api_keys(app: AppHandle) -> HashMap<ProviderId, String> {
    let state = app.state::<AppConfigState>();
    let keys = state.read().api_keys.clone();
    keys
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_api_key(app: AppHandle, provider: ProviderId, api_key: String) -> Result<(), String> {
    let state = app.state::<AppConfigState>();
    state
        .update_and_save(|config| {
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
        .update_and_save(|config| config.is_pin_index_window = pinned)
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
        .update_and_save(|config| config.auto_speak = auto_speak)
        .map_err(|e| e.to_string())?;
    Ok(auto_speak)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_prompt_tags(app: AppHandle, preset_id: String) -> Result<Vec<PromptTag>, String> {
    let state = app.state::<AppConfigState>();
    let config = state.read();
    let preset = config
        .agent_presets
        .iter()
        .find(|p| p.id == preset_id)
        .ok_or_else(|| format!("Agent preset {preset_id} not found"))?;
    Ok(preset.prompt_tags.clone())
}

#[tauri::command(rename_all = "snake_case")]
pub fn add_prompt_tag(
    app: AppHandle,
    preset_id: String,
    label: String,
    content: String,
) -> Result<Vec<PromptTag>, String> {
    let state = app.state::<AppConfigState>();
    if !state
        .read()
        .agent_presets
        .iter()
        .any(|p| p.id == preset_id)
    {
        return Err(format!("Agent preset {preset_id} not found"));
    }
    state
        .update_and_save(|config| {
            let preset = config
                .agent_presets
                .iter_mut()
                .find(|p| p.id == preset_id)
                .expect("preset existence checked above");
            let next_id = preset
                .prompt_tags
                .iter()
                .filter_map(|t| t.id)
                .max()
                .map_or(0, |m| m + 1);
            preset.prompt_tags.push(PromptTag {
                label: Some(label),
                content: Some(content),
                id: Some(next_id),
            });
        })
        .map_err(|e| e.to_string())?;
    get_prompt_tags(app, preset_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_prompt_tag(
    app: AppHandle,
    preset_id: String,
    id: u32,
    label: String,
    content: String,
) -> Result<Vec<PromptTag>, String> {
    let state = app.state::<AppConfigState>();
    let preset_exists = state
        .read()
        .agent_presets
        .iter()
        .any(|p| p.id == preset_id);
    if !preset_exists {
        return Err(format!("Agent preset {preset_id} not found"));
    }
    let mut found = true;
    state
        .update_and_save(|config| {
            let preset = config
                .agent_presets
                .iter_mut()
                .find(|p| p.id == preset_id)
                .expect("preset existence checked above");
            match preset.prompt_tags.iter_mut().find(|t| t.id == Some(id)) {
                Some(tag) => {
                    tag.label = Some(label);
                    tag.content = Some(content);
                }
                None => found = false,
            }
        })
        .map_err(|e| e.to_string())?;
    if !found {
        return Err(format!("Prompt tag {id} not found"));
    }
    get_prompt_tags(app, preset_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_prompt_tag(
    app: AppHandle,
    preset_id: String,
    id: u32,
) -> Result<Vec<PromptTag>, String> {
    let state = app.state::<AppConfigState>();
    if !state
        .read()
        .agent_presets
        .iter()
        .any(|p| p.id == preset_id)
    {
        return Err(format!("Agent preset {preset_id} not found"));
    }
    state
        .update_and_save(|config| {
            let preset = config
                .agent_presets
                .iter_mut()
                .find(|p| p.id == preset_id)
                .expect("preset existence checked above");
            preset.prompt_tags.retain(|t| t.id != Some(id));
        })
        .map_err(|e| e.to_string())?;
    get_prompt_tags(app, preset_id)
}

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
        .update_and_save(|config| config.local_language = language)
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
        .update_and_save(|config| config.target_language = language)
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
        .update_and_save(|config| config.self_explaining_model = enabled)
        .map_err(|e| e.to_string())?;
    Ok(enabled)
}
