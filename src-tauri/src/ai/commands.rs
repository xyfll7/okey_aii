use std::sync::{Arc, RwLock};

use futures::StreamExt;
use rig::message::Message;
use tauri::ipc::Channel;

use crate::ai::state::{add_message_to_history, build_session_agent, HistoryItem, Session};

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
        is_loading: false,
        cancel_handle: None,
    };

    guard.sessions.insert(session_id.clone(), session.clone());
    Ok((session_id, session))
}

/// 关闭会话并释放其历史,内存自然回收。
/// 返回 `true` 表示确实删除了一个存在的会话,`false` 表示该 session_id 本来就没有。
#[tauri::command(rename_all = "snake_case")]
pub fn close_session(app: tauri::AppHandle, session_id: String) -> Result<bool, String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let mut guard = state.write().unwrap();
    // 若该会话正在生成(持有 cancel_handle),先调用 abort() 通知对应的 send_message
    // 尽快结束(Abortable 流会在下次 poll 时产出 None),避免它在 session 被移除后
    // 仍空跑到 add_message_to_history 才因"会话不存在"报错终止。
    if let Some(sess) = guard.sessions.get(&session_id) {
        if let Some(handle) = &sess.cancel_handle {
            handle.abort();
        }
    }
    let removed = guard.sessions.remove(&session_id).is_some();
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
    prompt: HistoryItem,
    session_id: String,
) -> Result<(), String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    
    let (agent, history) = {
        let guard = state.read().unwrap();
        let sess = guard
            .sessions
            .get(&session_id)
            .ok_or("会话不存在,请先调用 create_session")?;
        (sess.agent.clone(), sess.history.clone())
    };

    // 本轮要发送给模型的消息(先 clone 出来,prompt 之后会被整体移入历史)
    let prompt_msg: Message = prompt.message.clone();

    // 先创建取消句柄,再用同一次 write 锁完成三件事:
    // 把用户消息写入历史、置 is_loading = true、挂上 cancel_handle。
    // 三者原子完成,保证从"用户消息已提交"到"cancel_handle 可用"之间不存在窗口期,
    // 避免用户此刻点击"停止"收到"当前没有正在进行的生成"的误报错误。
    let (abort_handle, abort_registration) = futures::future::AbortHandle::new_pair();
    {
        let mut guard = state.write().unwrap();
        let sess = guard
            .sessions
            .get_mut(&session_id)
            .ok_or("会话不存在,请先调用 create_session")?;
        if sess.is_loading {
            return Err("会话正在输出对话内容(loading 中),暂时禁止添加新的对话".into());
        }
        sess.history.push(prompt.clone());
        sess.is_loading = true;
        sess.cancel_handle = Some(abort_handle);
    }

    // 本轮 prompt 由前端传入,其余历史作为上下文传入
    let prompt: Message = prompt_msg;

    let history: Vec<Message> = history.iter().map(|h| h.message.clone()).collect();
    let stream = agent.stream_chat(prompt, history).await;
    // 用 Abortable 包一层:一旦 abort() 被调用,流会在下次 poll 时直接结束(产出 None)
    let mut stream = futures::stream::Abortable::new(stream, abort_registration);

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
                // 出错也要解除 loading,避免该会话永久卡在禁止追加状态
                let mut guard = state.write().unwrap();
                if let Some(sess) = guard.sessions.get_mut(&session_id) {
                    sess.is_loading = false;
                    sess.cancel_handle = None;
                }
                return Err(e);
            }
        }
    }
    let was_cancelled = stream.is_aborted();
    if was_cancelled {
        let _ = on_event.send(ChatEvent::Done);
    }

    // 2) 流结束(正常/前端断开/用户取消)后,统一解除 loading 并清空取消句柄,
    //    恢复允许追加(否则 add_message_to_history 会拒绝写入)
    {
        let mut guard = state.write().unwrap();
        if let Some(sess) = guard.sessions.get_mut(&session_id) {
            // 首条消息时,用用户输入的前 20 字作为标题
            if sess.history.is_empty() {
                sess.title = "".to_string();
            }
            sess.is_loading = false;
            sess.cancel_handle = None;
        }
    }
    // 3) 把助手这一轮的回复追加进该会话的历史;
    //    即使被取消,已生成的部分文本也保留,而不是整段丢弃
    if !full_text.is_empty() || !was_cancelled {
        add_message_to_history(
            &app,
            session_id,
            HistoryItem {
                id: uuid::Uuid::new_v4().to_string(),
                created_at: std::time::SystemTime::now(),
                message: Message::assistant(full_text.as_str()), // 助手这一轮(assistant 消息)
            },
        )?;
    }
    Ok(())
}

/// 停止指定会话当前正在进行的生成。
///
/// 通过 `Session.cancel_handle` 对 `send_message` 里的 `Abortable` 流调用 `abort()`,
/// 使其在下一次 poll 时结束;已生成的部分文本仍会写入历史。若当前没有正在进行的生成则返回错误。
#[tauri::command(rename_all = "snake_case")]
pub fn stop_generation(app: tauri::AppHandle, session_id: String) -> Result<(), String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let guard = state.read().unwrap();
    let sess = guard.sessions.get(&session_id).ok_or("会话不存在")?;
    match &sess.cancel_handle {
        Some(handle) => {
            handle.abort();
            Ok(())
        }
        None => {
            // 按第1条修复后,is_loading 为 true 时 cancel_handle 必然已挂上,故此处一般
            // 表示会话确实空闲;此处区分一下 loading 与完全空闲,给出更准确的提示。
            if sess.is_loading {
                Err("生成任务尚未初始化完成,请稍后再试".into())
            } else {
                Err("当前没有正在进行的生成".into())
            }
        }
    }
}

/// 列出所有已打开的会话,按创建时间倒序(最新在前)
#[tauri::command(rename_all = "snake_case")]
pub fn list_sessions(app: tauri::AppHandle) -> Vec<Session> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let guard = state.read().unwrap();
    let mut list: Vec<Session> = guard.sessions.values().cloned().collect();
    list.sort_by_key(|a| a.created_at);
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
pub fn get_history(app: tauri::AppHandle, session_id: String) -> Result<Vec<HistoryItem>, String> {
    let state = app.state::<Arc<RwLock<ChatState>>>();
    let guard = state.read().unwrap();
    let sess = guard.sessions.get(&session_id).ok_or("会话不存在")?;
    Ok(sess.history.clone())
}

/// 按 id 删除某个会话中的一条历史记录
#[tauri::command(rename_all = "snake_case")]
pub fn remove_history_item(
    app: tauri::AppHandle,
    session_id: String,
    history_id: String,
) -> Result<(), String> {
    crate::ai::state::remove_history_item(&app, session_id, history_id)
}
