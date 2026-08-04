use std::io::Write;
use std::sync::{Arc, RwLock};
use tauri::Manager;
use crate::ai::agents::ChatDelta;
use crate::ai::state::ChatState;
use futures::StreamExt;

/// 示例代码：展示如何从 Tauri 托管状态中取出 AI agent，并与其聊天（流式）。
///
/// 在 my_tray.rs 的 "test" 菜单项里被调用，点击系统托盘的 Test 即可运行。
pub async fn run_chat_example(app: &tauri::AppHandle) {
    // 1. 从 Tauri 托管状态中取出已初始化的 ChatState（见 init.rs 的 setup_ai_state）。
    let state = app.state::<Arc<RwLock<ChatState>>>();

    // 2. 只在同步临界区里 clone 出需要的字段，读锁立刻释放。
    //    Agents 内部是 Arc 包裹、实现了 Clone，clone 是廉价操作；
    //    这样读锁不会跨 .await 持有，future 依旧是 Send，可被 spawn。
    let (provider, model, agent) = {
        let guard = state.read().unwrap();
        (guard.provider, guard.model.clone(), guard.agent.clone())
    };
    println!("🤖 [test] 使用 {provider:?} / {model} 开始聊天示例");

    // 3. 发送一条示例消息，拿到流式响应。
    let prompt = "用一句话介绍你自己。";
    println!("👤 [test] 用户: {prompt}");

    // agent 是 clone 出来的独立副本，用完即弃，无需写回状态。
    // 这里传空历史，仅作一次性示例；连续聊天请走 send_message command。
    let mut stream = agent.stream_chat(prompt, Vec::new()).await;

    // 4. 消费流式事件：ChatDelta::Text 是增量文本，ChatDelta::Done 表示结束。
    print!("🤖 [test] 助手: ");
    while let Some(item) = stream.next().await {
        match item {
            Ok(ChatDelta::Text(text)) => {
                print!("{text}");
                // 强制刷新 stdout，让流式输出实时可见。
                let _ = std::io::stdout().flush();
            }
            Ok(ChatDelta::Done) => {
                println!();
                break;
            }
            Err(e) => {
                eprintln!("❌ [test] 流式响应出错: {e}");
                break;
            }
        }
    }

    println!("✅ [test] 聊天示例结束");
}
