use std::sync::{Arc, RwLock};

use futures::StreamExt;
use rig::message::Message;
use tauri::ipc::Channel;

use crate::ai::state::{build_session_agent, Session};

use super::agents::ChatEvent;
use super::config::{available_models, default_model, ModelInfo, Provider};
use super::state::ChatState;
use tauri::Manager;

#[tauri::command(rename_all = "snake_case")]
pub fn list_models(provider: Provider) -> Vec<ModelInfo> {
    available_models(provider).to_vec()
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_session(app: tauri::AppHandle) -> Result<(String, Session), String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
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
            (
                p,
                crate::ai::config::default_model(p).to_string(),
                "assistant".to_string(),
            )
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
#[tauri::command(rename_all = "snake_case")]
pub fn close_session(app: tauri::AppHandle, session_id: String) -> Result<bool, String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let removed = state
        .write()
        .unwrap()
        .sessions
        .remove(&session_id)
        .is_some();
    Ok(removed)
}

/// 切换某会话的 provider:只影响该 session_id 对应的标签页。
#[tauri::command(rename_all = "snake_case")]
pub fn switch_provider(
    app: tauri::AppHandle,
    session_id: String,
    provider: Provider,
    api_key: Option<String>,
) -> Result<(), String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
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
#[tauri::command(rename_all = "snake_case")]
pub fn switch_model(
    app: tauri::AppHandle,
    session_id: String,
    model: String,
) -> Result<(), String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
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

#[tauri::command(rename_all = "snake_case")]
pub async fn send_message(
    app: tauri::AppHandle,
    on_event: Channel<ChatEvent>,
) -> Result<(), String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();

    let session_id = {
        let list = list_sessions(app.clone());
        list.last()
            .map(|s| s.session_id.clone())
            .ok_or("没有可用的会话,请先调用 create_session")?
    };

    let (agent, history) = {
        let guard = state.read().unwrap();
        let sess = guard
            .sessions
            .get(&session_id)
            .ok_or("会话不存在,请先调用 create_session")?;
        (sess.agent.clone(), sess.history.clone())
    };

    // 用 OneOrMany::many 构造可以传入多条 UserContent 的 content(Many not one)

    // 取出 history 中最新的一条消息作为本轮 prompt,其余作为历史传入
    let (prompt, history) = history
        .split_last()
        .ok_or("history 为空,请先调用 add_message 添加用户消息")?;
    let prompt = prompt.clone();
    let mut stream = agent.stream_chat(prompt, history.to_vec()).await;
    let mut full_text = String::new();
    while let Some(item) = stream.next().await {
        match item {
            Ok(ChatEvent::TextDelta(text)) => {
                if text.is_empty() {
                    continue;
                }
                full_text.push_str(&text);
                // 实时把增量推给前端
                if on_event.send(ChatEvent::TextDelta(text)).is_err() {
                    // 前端已断开,停止后续推送
                    break;
                }
            }
            Ok(ChatEvent::ToolCall { name, arguments }) => {
                if on_event
                    .send(ChatEvent::ToolCall { name, arguments })
                    .is_err()
                {
                    break;
                }
            }
            Ok(ChatEvent::ToolCallDelta(s)) => {
                if on_event.send(ChatEvent::ToolCallDelta(s)).is_err() {
                    break;
                }
            }
            Ok(ChatEvent::Reasoning(text)) => {
                if on_event.send(ChatEvent::Reasoning(text)).is_err() {
                    break;
                }
            }
            Ok(ChatEvent::Done) => {
                // 发结束信号给前端
                let _ = on_event.send(ChatEvent::Done);
                break;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    // 2) 流结束后再拿写锁,把这一轮追加进【该会话】的历史
    let mut guard = state.write().unwrap();
    if let Some(sess) = guard.sessions.get_mut(&session_id) {
        // 首条消息时,用用户输入的前 20 字作为标题
        if sess.history.is_empty() {
            let title: String = "".to_string();
            sess.title = title;
        }
        sess.history.push(Message::assistant(full_text.as_str())); // 助手这一轮(assistant 消息)
    }
    Ok(())
}

/// 列出所有已打开的会话,按创建时间倒序(最新在前)
#[tauri::command(rename_all = "snake_case")]
pub fn list_sessions(app: tauri::AppHandle) -> Vec<Session> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let guard = state.read().unwrap();
    let mut list: Vec<Session> = guard.sessions.iter().map(|(_, s)| s.clone()).collect();
    list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    list
}

/// 清空某个会话的历史(保留 agent)
#[tauri::command(rename_all = "snake_case")]
pub fn clear_history(app: tauri::AppHandle, session_id: String) -> Result<(), String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
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
#[tauri::command(rename_all = "snake_case")]
pub fn get_history(app: tauri::AppHandle, session_id: String) -> Result<Vec<Message>, String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let guard = state.read().unwrap();
    let sess = guard.sessions.get(&session_id).ok_or("会话不存在")?;
    Ok(sess.history.clone())
}
