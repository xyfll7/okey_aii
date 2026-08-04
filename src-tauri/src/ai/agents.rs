use futures::{Stream, StreamExt};
use std::pin::Pin;
use rig::agent::{Agent, MultiTurnStreamItem, StreamingResult};
use rig::message::Message;
use rig::streaming::{StreamedAssistantContent, StreamingChat};
use rig::providers::{anthropic, openai, deepseek};

/// 与 provider 无关的统一事件,直接对应你的 AG-UI 事件
pub enum ChatDelta {
    Text(String),
    Done,
}

#[derive(Clone)]
pub enum Agents {
    OpenAI(Agent<openai::completion::CompletionModel>),
    Anthropic(Agent<anthropic::completion::CompletionModel>),
    DeepSeek(Agent<deepseek::CompletionModel>),
}

/// 把某个 provider 的 stream_chat 结果统一映射成 ChatDelta。
///
/// rig 0.41 的流产出 `MultiTurnStreamItem<R>`:我们只关心文本增量
/// (`StreamAssistantItem(Text)`) 和终态 (`FinalResponse`),其余(工具调用、
/// 中间事件等)用空 `Text` 表示,在 `send_message` 端按 `!text.is_empty()` 跳过。
fn map_stream<R>(
    stream: StreamingResult<R>,
) -> Pin<Box<dyn Stream<Item = Result<ChatDelta, String>> + Send>>
where
    R: Send + 'static,
{
    let mapped = stream.map(|item| {
        item.map_err(|e| e.to_string()).map(|item| match item {
            MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t)) => {
                ChatDelta::Text(t.text)
            }
            MultiTurnStreamItem::FinalResponse(_) => ChatDelta::Done,
            _ => ChatDelta::Text(String::new()),
        })
    });
    Box::pin(mapped)
}

impl Agents {
    /// 连续聊天:把 `prompt` + `history` 一起发给模型,而不是只发一句话。
    ///
    /// `stream_chat` 来自 rig 的 `StreamingChat` trait,返回 `StreamingPromptRequest`,
    /// 再 `.stream().await` 得到真正的流产出流。
    pub async fn stream_chat<'a>(
        &'a self,
        prompt: &str,
        history: Vec<Message>,
    ) -> Pin<Box<dyn Stream<Item = Result<ChatDelta, String>> + Send>> {
        match self {
            Self::OpenAI(agent) => map_stream(agent.stream_chat(prompt, history).await),
            Self::Anthropic(agent) => map_stream(agent.stream_chat(prompt, history).await),
            Self::DeepSeek(agent) => map_stream(agent.stream_chat(prompt, history).await),
        }
    }
}
