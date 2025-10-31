//! 智谱 AI 基础测试
//!
//! 测试 LumosAI 与智谱 AI (glm-4-plus-flash) 的集成

use lumosai_core::llm::{LlmOptions, LlmProvider, Message, Role, ZhipuProvider};
use std::env;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 LumosAI 智谱 AI 集成测试");
    println!("========================================");

    // 获取 API Key
    let api_key = env::var("ZHIPU_API_KEY")
        .unwrap_or_else(|_| "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k".to_string());

    println!("🔑 使用 API Key: {}...", &api_key[..20]);

    // 创建智谱 AI 提供商 - 使用 glm-4-plus 而不是 glm-4-plus-flash
    let provider = ZhipuProvider::new(api_key, Some("glm-4-plus".to_string()));

    println!("🔧 使用模型: {}", provider.model());
    println!("🌐 API 端点: {}", provider.base_url());

    // 测试1: 基础连接测试
    test_basic_connection(&provider).await?;

    // 测试2: 中文对话测试
    test_chinese_conversation(&provider).await?;

    // 测试3: 英文对话测试
    test_english_conversation(&provider).await?;

    println!("\n✅ 所有智谱 AI 测试完成！");
    Ok(())
}

async fn test_basic_connection(provider: &ZhipuProvider) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 测试1: 智谱 AI 基础连接...");

    // 使用更简单的选项
    let mut options = LlmOptions::default();
    options.temperature = Some(0.7);
    options.max_tokens = Some(100);

    let start_time = Instant::now();
    let result = provider.generate("你好", &options).await;
    let duration = start_time.elapsed();

    match result {
        Ok(response) => {
            println!("✅ 智谱 AI 连接成功!");
            println!("📝 响应内容: {}", response);
            println!("⏱️ 响应时间: {:?}", duration);

            if response.is_empty() {
                return Err("响应内容为空".into());
            }

            if duration.as_secs() > 30 {
                return Err("响应时间过长".into());
            }
        }
        Err(e) => {
            println!("❌ 智谱 AI 连接失败: {}", e);
            println!("🔍 错误详情: {:?}", e);
            return Err(format!("智谱 AI 基础连接测试失败: {}", e).into());
        }
    }

    Ok(())
}

async fn test_chinese_conversation(
    provider: &ZhipuProvider,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 测试2: 智谱 AI 中文对话能力...");

    let messages = vec![
        Message {
            role: Role::System,
            content: "你是一个专业的AI助手，擅长回答技术问题。".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "请解释一下什么是Rust编程语言的所有权系统？".to_string(),
            metadata: None,
            name: None,
        },
    ];

    let result = provider
        .generate_with_messages(&messages, &LlmOptions::default())
        .await;

    match result {
        Ok(response) => {
            println!("✅ 中文对话测试成功!");
            println!("📝 响应内容: {}", response);

            if response.is_empty() {
                return Err("响应内容为空".into());
            }

            if !response.contains("所有权")
                && !response.contains("Rust")
                && !response.contains("内存")
            {
                println!("⚠️ 警告: 响应可能不包含预期的技术内容");
            }
        }
        Err(e) => {
            println!("❌ 中文对话测试失败: {}", e);
            return Err(format!("智谱 AI 中文对话测试失败: {}", e).into());
        }
    }

    Ok(())
}

async fn test_english_conversation(
    provider: &ZhipuProvider,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 测试3: 智谱 AI 英文对话能力...");

    let result = provider
        .generate(
            "Explain the concept of artificial intelligence in simple terms.",
            &LlmOptions::default(),
        )
        .await;

    match result {
        Ok(response) => {
            println!("✅ 英文对话测试成功!");
            println!("📝 响应内容: {}", response);

            if response.is_empty() {
                return Err("响应内容为空".into());
            }

            let response_lower = response.to_lowercase();
            if !response_lower.contains("artificial")
                && !response_lower.contains("intelligence")
                && !response_lower.contains("ai")
            {
                println!("⚠️ 警告: 响应可能不包含预期的AI相关内容");
            }
        }
        Err(e) => {
            println!("❌ 英文对话测试失败: {}", e);
            return Err(format!("智谱 AI 英文对话测试失败: {}", e).into());
        }
    }

    Ok(())
}
