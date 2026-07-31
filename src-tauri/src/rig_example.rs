use rig::prelude::*;

const DEEPSEEK_API_KEY: &str = "sk-a36321b7ed3c47c88d6e6f371550e6f9";
const DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";

/// 最简单的 DeepSeek 对话示例
pub async fn ask(prompt: &str) -> Result<String, String> {
    let client = rig::providers::openai::Client::builder()
        .api_key(DEEPSEEK_API_KEY)
        .base_url(DEEPSEEK_BASE_URL)
        .build()
        .map_err(|e| format!("构建客户端失败: {e}"))?;

    let agent = client.agent("deepseek-chat").build();

    agent.prompt(prompt).await.map_err(|e| format!("{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ask() {
        let result = tauri::async_runtime::block_on(ask("你好，我看你爱吃滴电"));
        match &result {
            Ok(response) => println!("DeepSeek: {}", response),
            Err(e) => eprintln!("错误: {}", e),
        }
        assert!(result.is_ok());
    }
}
