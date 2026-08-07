use std::sync::{Arc, RwLock};

use futures::StreamExt;
use serde_json::json;
use rig::message::Message;

use crate::ai::state::{Session, build_session_agent};

use super::agents::ChatDelta;
use super::config::{available_models, default_model, ModelInfo, Provider};
use super::state::ChatState;

use tauri::{Emitter, State};

#[tauri::command]
pub fn list_models(provider: Provider) -> Vec<ModelInfo> {
    available_models(provider).to_vec()
}

#[tauri::command]
pub fn create_session(
    state: State<'_, Arc<RwLock<ChatState>>>,
) -> Result<(String, Session), String> {
    let mut guard = state.write().unwrap();
    let api_keys = guard.config.api_keys.clone();

    let session_id = uuid::Uuid::new_v4().to_string();

    let (provider, model, preset_id) = guard
        .sessions
        .values()
        .max_by(|a, b| a.created_at.cmp(&b.created_at))
        .map(|s| (s.provider, s.model.clone(), s.preset_id.clone()))
        .unwrap_or_else(|| {
            let p = Provider::DeepSeek;
            (p, crate::ai::config::default_model(p).to_string(), "assistant".to_string())
        });

    let agent = build_session_agent(&api_keys, provider, &model, &preset_id)?;

    let session = Session {
        session_id: session_id.clone(),
        provider,
        model,
        preset_id,
        agent,
        history: Vec::new(),
        created_at: std::time::SystemTime::now(),
        title: "新会话".into(),
    };

    guard.sessions.insert(session_id.clone(), session.clone());
    Ok((session_id, session))
}

/// 关闭会话并释放其历史,内存自然回收。
/// 返回 `true` 表示确实删除了一个存在的会话,`false` 表示该 session_id 本来就没有。
#[tauri::command]
pub fn close_session(
    state: State<'_, Arc<RwLock<ChatState>>>,
    session_id: String,
) -> Result<bool, String> {
    let removed = state.write().unwrap().sessions.remove(&session_id).is_some();
    Ok(removed)
}

/// 切换某会话的 provider:只影响该 session_id 对应的标签页。
#[tauri::command]
pub fn switch_provider(
    state: State<'_, Arc<RwLock<ChatState>>>,
    session_id: String,
    provider: Provider,
    api_key: Option<String>,
) -> Result<(), String> {
    let mut guard = state.write().unwrap();
    if let Some(key) = api_key {
        guard.config.api_keys.insert(provider, key);
    }

    // 换 provider 时,model 重置成该 provider 的默认模型
    let model = default_model(provider).to_string();

    // 先把需要的字段从 guard 中取出来(clone),避免下面构建 agent 时仍借用 guard,
    // 导致后面 get_mut 取可变借用冲突。
    let api_keys = guard.config.api_keys.clone();
    let preset_id = guard
        .sessions
        .get(&session_id)
        .map(|s| s.preset_id.clone())
        .unwrap_or_else(|| "assistant".to_string());

    // 先为新配置构建 agent,成功后才写入该会话;构建失败则整体不变。
    let agent = build_session_agent(&api_keys, provider, &model, &preset_id)?;

    let sess = guard
        .sessions
        .get_mut(&session_id)
        .ok_or("会话不存在,请先调用 create_session")?;
    sess.provider = provider;
    sess.model = model.clone();
    sess.agent = agent;

    Ok(())
}

/// 切换某会话的模型:只影响该 session_id 对应的标签页。
#[tauri::command]
pub fn switch_model(
    state: State<'_, Arc<RwLock<ChatState>>>,
    session_id: String,
    model: String,
) -> Result<(), String> {
    let mut guard = state.write().unwrap();

    // 先取出该校话的配置字段(Copy/String),释放不可变借用,后面才能再取可变引用
    let (provider, preset_id) = {
        let sess = guard
            .sessions
            .get(&session_id)
            .ok_or("会话不存在,请先调用 create_session")?;
        (sess.provider, sess.preset_id.clone())
    };

    // 校验这个 model 确实属于该会话当前的 provider,避免前端传错
    let valid = available_models(provider).iter().any(|m| m.id == model);
    if !valid {
        return Err(format!("{model} 不属于当前会话的 provider"));
    }

    // 先构建新 agent,成功后才写入该会话
    let api_keys = guard.config.api_keys.clone();
    let agent = build_session_agent(&api_keys, provider, &model, &preset_id)?;

    let sess = guard.sessions.get_mut(&session_id).unwrap();
    sess.model = model.clone();
    sess.agent = agent;

    Ok(())
}

#[tauri::command]
pub async fn send_message(
    app: tauri::AppHandle,
    state: State<'_, Arc<RwLock<ChatState>>>,
    session_id: String,
    prompt: String,
) -> Result<(), String> {
    // 事件按 session_id 路由,避免跨标签页串台
    let event_name = format!("agui-event:{session_id}");
    app.emit(&event_name, json!({ "type": "RUN_STARTED" })).ok();

    // 1) 只在拿锁的一瞬间 clone 该会话的 agent/history,不带着锁 await
    let (agent, history) = {
        let guard = state.read().unwrap();
        let sess = guard
            .sessions
            .get(&session_id)
            .ok_or("会话不存在,请先调用 create_session")?;
        (sess.agent.clone(), sess.history.clone())
    };

    let mut stream = agent.stream_chat(&prompt, history).await;
    let mut full_text = String::new();

    while let Some(item) = stream.next().await {
        match item {
            Ok(ChatDelta::Text(text)) if !text.is_empty() => {
                full_text.push_str(&text);
                app.emit(
                    &event_name,
                    json!({ "type": "TEXT_MESSAGE_CONTENT", "delta": text }),
                )
                .ok();
            }
            Ok(ChatDelta::Done) => {
                app.emit(&event_name, json!({ "type": "TEXT_MESSAGE_END" })).ok();
            }
            Ok(_) => {}
            Err(e) => {
                app.emit(&event_name, json!({ "type": "RUN_ERROR", "message": e })).ok();
                return Err(e);
            }
        }
    }

    // 2) 流结束后再拿写锁,把这一轮追加进【该会话】的历史
    let mut guard = state.write().unwrap();
    if let Some(sess) = guard.sessions.get_mut(&session_id) {
        // 首条消息时,用用户输入的前 20 字作为标题
        if sess.history.is_empty() {
            let title: String = prompt.chars().take(20).collect();
            sess.title = if prompt.chars().count() > 20 {
                format!("{title}…")
            } else {
                title
            };
        }
        sess.history.push(Message::from(prompt.as_str())); // 用户这一轮(user 消息)
        sess.history.push(Message::assistant(full_text.as_str())); // 助手这一轮(assistant 消息)
    }
    Ok(())
}

/// 列出所有已打开的会话,按创建时间倒序(最新在前)
#[tauri::command]
pub fn list_sessions(
    state: State<'_, Arc<RwLock<ChatState>>>,
) -> Vec<Session> {
    let guard = state.read().unwrap();
    let mut list: Vec<Session> = guard
        .sessions
        .iter()
        .map(|(_, s)| s.clone())
        .collect();
    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    list
}

/// 清空某个会话的历史(保留 agent)
#[tauri::command]
pub fn clear_history(
    state: State<'_, Arc<RwLock<ChatState>>>,
    session_id: String,
) -> Result<(), String> {
    let mut guard = state.write().unwrap();
    guard
        .sessions
        .get_mut(&session_id)
        .ok_or("会话不存在")?
        .history
        .clear();
    Ok(())
}

/// 获取某个会话的聊天记录
#[tauri::command]
pub fn get_history(
    state: State<'_, Arc<RwLock<ChatState>>>,
    session_id: String,
) -> Result<Vec<Message>, String> {
    let guard = state.read().unwrap();
    let sess = guard
        .sessions
        .get(&session_id)
        .ok_or("会话不存在")?;
    Ok(sess.history.clone())
}