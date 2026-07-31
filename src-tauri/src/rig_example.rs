use futures::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::prelude::*;
use rig::streaming::{StreamedAssistantContent, StreamingPrompt};


const DEEPSEEK_API_KEY: &str = "sk-a36321b7ed3c47c88d6e6f371550e6f9";
const DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";

fn make_client() -> Result<rig::providers::openai::Client, String> {
    rig::providers::openai::Client::builder()
        .api_key(DEEPSEEK_API_KEY)
        .base_url(DEEPSEEK_BASE_URL)
        .build().map_err(|e| format!("构建客户端失败: {e}"))
}

pub async fn stream_ask(prompt: &str) -> Result<String, String> {
    let client = make_client()?;
    let agent = client
        .agent("deepseek-chat")
        .preamble("You are a helpful assistant with math skills.")
        .build();

    // stream_prompt().await 直接返回流，不包装在 Result 中
    let mut stream = agent.stream_prompt(prompt).await;
    let mut full_text = String::new();

    while let Some(item) = stream.next().await {
        match item {
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                print!("{}", text.text);
                full_text.push_str(&text.text);
            }
            Ok(MultiTurnStreamItem::FinalResponse(_)) => {
                println!(); // 结束时换行
            }
            Err(e) => return Err(format!("流式错误: {e}")),
            _ => {}
        }
    }

    Ok(full_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_stream_ask() {
        let result = tauri::async_runtime::block_on(stream_ask("请用中文详细解释以下单词的含义：'Rust'"));
        match &result {
            Ok(response) => println!("完整响应: {}", response),
            Err(e) => eprintln!("错误: {}", e),
        }
        assert!(result.is_ok());
    }
}
