use futures::{Stream, StreamExt};
use std::pin::Pin;
use rig::agent::{Agent, MultiTurnStreamItem, StreamingResult};
use rig::message::Message;
use rig::streaming::{StreamedAssistantContent, StreamingChat};

#[derive(Clone)]
pub enum Agents {
    OpenAI(Agent),
    Anthropic(Agent),
    DeepSeek(Agent),
}


#[derive(Clone, serde::Serialize)]
#[serde(tag = "type", content = "data")]
#[allow(dead_code)]
pub enum ChatEvent {
    
    TextDelta(String),
    
    ToolCall { name: String, arguments: serde_json::Value },
    
    ToolCallDelta(String),
    
    Reasoning(String),
    
    Done,
}


fn map_stream(stream: StreamingResult) -> Pin<Box<dyn Stream<Item = Result<ChatEvent, String>> + Send>> {
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


fn map_content(content: StreamedAssistantContent) -> Option<ChatEvent> {
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
        StreamedAssistantContent::Reasoning { reasoning, .. } => {
            
            let text = reasoning
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
    
    
    pub async fn stream_chat(
        &self,
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
