use std::sync::{Arc, RwLock};

use super::config::{available_models, default_model, builtin_presets, ModelInfo, Provider};
use super::state::{build_agent, ChatState};
use tauri::State;

#[tauri::command]
pub fn list_models(provider: Provider) -> Vec<ModelInfo> {
    available_models(provider).to_vec()
}

#[tauri::command]
pub fn switch_provider(
    state: State<'_, Arc<RwLock<ChatState>>>,
    provider: Provider,
    api_key: Option<String>,
) -> Result<(), String> {
    let mut guard = state.write().unwrap();
    if let Some(key) = api_key {
        guard.api_keys.insert(provider, key);
    }
    let key = guard
        .api_keys
        .get(&provider)
        .cloned()
        .ok_or_else(|| format!("{provider:?} 缺少 api key"))?;
    let preset = builtin_presets()
        .into_iter()
        .find(|p| p.id == guard.preset_id)
        .ok_or("preset not found")?;

    // 换 provider 时,model 重置成该 provider 的默认模型
    let model = default_model(provider).to_string();

    guard.agent = build_agent(provider, &model, &key, &preset)?;
    guard.provider = provider;
    guard.model = model;
    Ok(())
}

#[tauri::command]
pub fn switch_model(state: State<'_, Arc<RwLock<ChatState>>>, model: String) -> Result<(), String> {
    let mut guard = state.write().unwrap();

    // 校验一下这个 model 确实属于当前 provider,避免前端传错
    let valid = available_models(guard.provider).iter().any(|m| m.id == model);
    if !valid {
        return Err(format!("{model} 不属于当前 provider"));
    }

    let key = guard
        .api_keys
        .get(&guard.provider)
        .cloned()
        .ok_or("当前 provider 缺少 api key")?;
    let preset = builtin_presets()
        .into_iter()
        .find(|p| p.id == guard.preset_id)
        .ok_or("preset not found")?;

    guard.agent = build_agent(guard.provider, &model, &key, &preset)?;
    guard.model = model;
    Ok(())
}