use futures::{Stream, StreamExt};
use std::pin::Pin;
use rig::agent::{Agent, MultiTurnStreamItem, StreamingResult};
use rig::message::Message;
use rig::streaming::{StreamedAssistantContent, StreamingChat};
use rig::providers::{anthropic, openai, deepseek};

#[derive(Clone)]
pub enum Agents {
    OpenAI(Agent<openai::completion::CompletionModel>),
    Anthropic(Agent<anthropic::completion::CompletionModel>),
    DeepSeek(Agent<deepseek::CompletionModel>),
}

/// 与 provider 无关的精简流事件,只保留下游真正关心的语义信息。
///
/// 从 rig 的 `MultiTurnStreamItem<R>` / `StreamedAssistantContent<R>` 中提取,
/// 直接丢弃 provider 相关的泛型 `R`(最终响应对象),不保留 `Final` 位置。
#[derive(Clone, serde::Serialize)]
#[serde(tag = "type", content = "data")]
#[allow(dead_code)]
pub enum ChatEvent {
    /// 文本增量
    TextDelta(String),
    /// 完整的工具调用(模型已提交)
    ToolCall { name: String, arguments: serde_json::Value },
    /// 工具调用增量(部分名称或参数片段)
    ToolCallDelta(String),
    /// 完整的推理块
    Reasoning(String),
    /// 流结束信号
    Done,
}

/// 把某个 provider 的 stream_chat 结果映射成 `ChatEvent` 流。
///
/// rig 0.41 的流产出 `MultiTurnStreamItem<R>`,这里只提取关心的信息,
/// `Final(R)` 及其它不关心的变体直接 `filter_map` 掉,用独立的 `Done` 信号代替。
fn map_stream<R>(
    stream: StreamingResult<R>,
) -> Pin<Box<dyn Stream<Item = Result<ChatEvent, String>> + Send>>
where
    R: Send + 'static,
{
    let mapped = stream.filter_map(|item| async move {
        match item {
            Ok(MultiTurnStreamItem::StreamAssistantItem(content)) => {
                map_content(content).map(Ok)
            }
            Ok(MultiTurnStreamItem::FinalResponse(_)) => Some(Ok(ChatEvent::Done)),
            Ok(_) => None,
            Err(e) => Some(Err(e.to_string())),
        }
    });
    Box::pin(mapped)
}

/// 从 `StreamedAssistantContent<R>` 中提取 `ChatEvent`,
/// `Final(R)` 及 `Unknown` 直接丢弃。
fn map_content<R>(content: StreamedAssistantContent<R>) -> Option<ChatEvent> {
    match content {
        StreamedAssistantContent::Text(t) => Some(ChatEvent::TextDelta(t.text)),
        StreamedAssistantContent::ToolCall { tool_call, .. } => Some(ChatEvent::ToolCall {
            name: tool_call.function.name,
            arguments: tool_call.function.arguments,
        }),
        StreamedAssistantContent::ToolCallDelta { content, .. } => {
            let s = match content {
                rig::streaming::ToolCallDeltaContent::Name(n) => n,
                rig::streaming::ToolCallDeltaContent::Delta(d) => d,
            };
            Some(ChatEvent::ToolCallDelta(s))
        }
        StreamedAssistantContent::Reasoning(r) => {
            // Reasoning.content 是 Vec<ReasoningContent>,这里简单拼接文本部分
            let text = r
                .content
                .into_iter()
                .map(|c| match c {
                    rig::message::ReasoningContent::Text { text, .. } => text,
                    rig::message::ReasoningContent::Summary(s) => s,
                    _ => String::new(),
                })
                .collect::<String>();
            Some(ChatEvent::Reasoning(text))
        }
        StreamedAssistantContent::ReasoningDelta { reasoning, .. } => {
            Some(ChatEvent::Reasoning(reasoning))
        }
        _ => None,
    }
}

impl Agents {
    /// 连续聊天:把 `prompt` + `history` 一起发给模型,而不是只发一句话。
    ///
    /// `stream_chat` 来自 rig 的 `StreamingChat` trait,返回 `StreamingPromptRequest`,
    /// 再 `.await`(`IntoFuture`)得到真正的流产出流。
    pub async fn stream_chat<'a>(
        &'a self,
        prompt: Message,
        history: Vec<Message>,
    ) -> Pin<Box<dyn Stream<Item = Result<ChatEvent, String>> + Send>> {
        match self {
            Self::OpenAI(agent) => map_stream(agent.stream_chat(prompt, history).await),
            Self::Anthropic(agent) => map_stream(agent.stream_chat(prompt, history).await),
            Self::DeepSeek(agent) => map_stream(agent.stream_chat(prompt, history).await),
        }
    }
}
