use futures::{Stream, StreamExt};
use futures::stream::BoxStream;
use rig::agent::Agent;
use rig::agent::{ MultiTurnStreamItem};  
use rig::streaming::{StreamedAssistantContent, StreamingPrompt};
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

/// 把某个 provider 的 stream_prompt 结果统一映射成 ChatDelta
fn map_stream<'a, R, E>(
    stream: impl Stream<Item = Result<MultiTurnStreamItem<R>, E>> + Send + 'a,
) -> BoxStream<'a, Result<ChatDelta, String>>
where
    R: Send + 'a,
    E: std::fmt::Display + Send + 'a,
{
    Box::pin(stream.map(|item| {
        item.map_err(|e| e.to_string()).map(|item| match item {
            MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t)) => {
                ChatDelta::Text(t.text)
            }
            MultiTurnStreamItem::FinalResponse(_) => ChatDelta::Done,
            _ => ChatDelta::Text(String::new()),
        })
    }))
}

impl Agents {
    pub async fn stream_prompt<'a>(&'a self, prompt: &str) -> BoxStream<'a, Result<ChatDelta, String>> {
        match self {
            Self::OpenAI(agent) => map_stream(agent.stream_prompt(prompt).await),
            Self::Anthropic(agent) => map_stream(agent.stream_prompt(prompt).await),
            Self::DeepSeek(agent) => map_stream(agent.stream_prompt(prompt).await),
        }
    }
}