//! 智谱 AI Agent 演示
//!
//! 展示如何使用修复后的智谱 AI 提供商创建真实的 AI Agent

use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::llm::{LlmOptions, ZhipuProvider};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 智谱 AI Agent 演示");
    println!("====================");

    // 获取 API Key
    let api_key = env::var("ZHIPU_API_KEY")
        .unwrap_or_else(|_| "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k".to_string());

    println!("🔑 使用 API Key: {}...", &api_key[..20]);

    // 演示1: 基础智谱 AI Agent
    demo_basic_zhipu_agent(&api_key).await?;

    // 演示2: 智谱 AI 对话 Agent
    demo_conversation_agent(&api_key).await?;

    // 演示3: 智谱 AI 专业 Agent
    demo_professional_agent(&api_key).await?;

    println!("\n✅ 智谱 AI Agent 演示完成！");
    Ok(())
}

/// 演示基础智谱 AI Agent
async fn demo_basic_zhipu_agent(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示1: 基础智谱 AI Agent ===");

    // 创建智谱 AI 提供商
    let zhipu_provider = Arc::new(ZhipuProvider::new(
        api_key.to_string(),
        Some("glm-4.6".to_string()),
    ));

    // 创建 Agent
    let agent = AgentBuilder::new()
        .name("zhipu_assistant")
        .instructions("你是一个友好的AI助手，使用智谱AI技术。请用中文回答问题，保持简洁明了。")
        .model(zhipu_provider)
        .build()?;

    // 测试基础对话
    println!("🔄 测试基础对话...");
    let response = agent.generate_simple("你好！请简单介绍一下自己。").await?;
    println!("🤖 智谱 AI 回复: {}", response);

    Ok(())
}

/// 演示对话 Agent
async fn demo_conversation_agent(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示2: 智谱 AI 对话 Agent ===");

    // 创建智谱 AI 提供商
    let zhipu_provider = Arc::new(ZhipuProvider::new(
        api_key.to_string(),
        Some("glm-4.6".to_string()),
    ));

    // 创建对话 Agent
    let agent = AgentBuilder::new()
        .name("conversation_agent")
        .instructions("你是一个善于对话的AI助手。请记住对话历史，并能够基于之前的内容进行回复。")
        .model(zhipu_provider)
        .build()?;

    // 多轮对话测试
    let conversations = vec![
        "我是一名软件工程师，专门做 Rust 开发",
        "请问你对 Rust 语言有什么了解？",
        "能给我推荐一些 Rust 学习资源吗？",
    ];

    for (i, message) in conversations.iter().enumerate() {
        println!("\n第{}轮对话:", i + 1);
        println!("👤 用户: {}", message);

        let response = agent.generate_simple(message).await?;
        println!("🤖 智谱 AI: {}", response);

        // 添加短暂延迟，避免 API 限制
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }

    Ok(())
}

/// 演示专业 Agent
async fn demo_professional_agent(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示3: 智谱 AI 专业 Agent ===");

    // 创建智谱 AI 提供商
    let zhipu_provider = Arc::new(ZhipuProvider::new(
        api_key.to_string(),
        Some("glm-4.6".to_string()),
    ));

    // 创建技术专家 Agent
    let tech_agent = AgentBuilder::new()
        .name("tech_expert")
        .instructions(
            "你是一位资深的技术专家，特别擅长 Rust、AI 和系统架构。\
            请提供专业、详细的技术解答，包含实用的建议和最佳实践。",
        )
        .model(zhipu_provider.clone())
        .build()?;

    // 技术问题测试
    let tech_questions = vec![
        "请解释 Rust 中的所有权系统，并给出一个实际的代码示例",
        "在设计 AI Agent 系统时，有哪些关键的架构考虑？",
        "如何在 Rust 中实现高性能的异步 HTTP 客户端？",
    ];

    for (i, question) in tech_questions.iter().enumerate() {
        println!("\n📋 技术问题 {}: {}", i + 1, question);

        let response = tech_agent.generate_simple(question).await?;
        println!("🔬 技术专家回复:\n{}", response);

        // 添加延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
    }

    // 创建创意写作 Agent
    println!("\n--- 创意写作 Agent ---");
    let creative_agent = AgentBuilder::new()
        .name("creative_writer")
        .instructions(
            "你是一位富有创意的作家，擅长创作有趣的故事和诗歌。\
            请发挥想象力，创作生动有趣的内容。",
        )
        .model(zhipu_provider)
        .build()?;

    let creative_prompt = "请写一首关于 AI 和人类合作开发软件的现代诗，要体现科技与人文的结合";
    println!("🎨 创意提示: {}", creative_prompt);

    let creative_response = creative_agent.generate_simple(creative_prompt).await?;
    println!("✨ 创意作品:\n{}", creative_response);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumosai_core::llm::LlmProvider;

    #[tokio::test]
    async fn test_zhipu_provider_creation() {
        let api_key = "test_key".to_string();
        let provider = ZhipuProvider::new(api_key, Some("glm-4.6".to_string()));

        assert_eq!(provider.name(), "zhipu");
    }

    #[tokio::test]
    async fn test_agent_builder_with_zhipu() {
        let api_key = "test_key".to_string();
        let zhipu_provider = Arc::new(ZhipuProvider::new(api_key, Some("glm-4.6".to_string())));

        let agent = AgentBuilder::new()
            .name("test_agent")
            .instructions("Test instructions")
            .model(zhipu_provider)
            .build();

        assert!(agent.is_ok());
        let agent = agent.unwrap();
        assert_eq!(agent.get_name(), "test_agent");
        assert_eq!(agent.get_instructions(), "Test instructions");
    }

    #[test]
    fn test_api_key_handling() {
        // 测试 API Key 处理
        let test_key = "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k";
        let truncated = &test_key[..20];
        assert_eq!(truncated, "99a311fa7920a59e9399");
    }
}
