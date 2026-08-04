use std::sync::{Arc, RwLock};

use futures::StreamExt;
use serde_json::json;
use rig::message::Message;
use super::agents::ChatDelta;
use super::config::{available_models, default_model, builtin_presets, ModelInfo, Provider};
use super::state::{build_agent, ChatState};
use tauri::{Emitter, State};

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

    guard.agent = Arc::new(build_agent(provider, &model, &key, &preset)?);
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

    guard.agent = Arc::new(build_agent(guard.provider, &model, &key, &preset)?);
    guard.model = model;
    Ok(())
}

#[tauri::command]
pub async fn send_message(
    app: tauri::AppHandle,
    state: State<'_, Arc<RwLock<ChatState>>>,
    prompt: String,
) -> Result<(), String> {
    app.emit(
        "agui-event",
        json!({ "type": "RUN_STARTED" }),
    )
    .ok();

    // 1) 只在拿锁的一瞬间 clone,不带着锁 await,期间可以随便切 provider/model
    let (agent, history) = {
        let guard = state.read().unwrap();
        (guard.agent.clone(), guard.history.clone())
    };

    let mut stream = agent.stream_chat(&prompt, history).await;
    let mut full_text = String::new();

    while let Some(item) = stream.next().await {
        match item {
            Ok(ChatDelta::Text(text)) if !text.is_empty() => {
                full_text.push_str(&text);
                app.emit(
                    "agui-event",
                    json!({ "type": "TEXT_MESSAGE_CONTENT", "delta": text }),
                )
                .ok();
            }
            Ok(ChatDelta::Done) => {
                app.emit("agui-event", json!({ "type": "TEXT_MESSAGE_END" })).ok();
            }
            Ok(_) => {}
            Err(e) => {
                app.emit("agui-event", json!({ "type": "RUN_ERROR", "message": e })).ok();
                return Err(e);
            }
        }
    }

    // 2) 流结束后再拿写锁,把这一轮追加进历史,供下一轮使用
    let mut guard = state.write().unwrap();
    guard.history.push(Message::from(prompt.as_str())); // 用户这一轮(user 消息)
    guard.history.push(Message::assistant(full_text.as_str())); // 助手这一轮(assistant 消息)
    Ok(())
}

#[tauri::command]
pub fn clear_history(state: State<'_, Arc<RwLock<ChatState>>>) -> Result<(), String> {
    state.write().unwrap().history.clear();
    Ok(())
}