//! Zhipu AI Real API Test
//!
//! 这个示例演示如何使用真实的 Zhipu AI API
//! 需要设置环境变量: ZHIPU_API_KEY
//!
//! 运行方式:
//! ```bash
//! export ZHIPU_API_KEY="your-api-key"
//! cargo run --example zhipu_real_test
//! ```

use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::agent::{Agent, AgentConfig, BasicAgent};
use lumosai_core::llm::{Message, Role, ZhipuProvider};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量获取 API key
    let api_key = std::env::var("ZHIPU_API_KEY")
        .unwrap_or_else(|_| "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k".to_string());

    println!("🚀 Zhipu AI Real API Test");
    println!("{}", "=".repeat(50));
    println!(
        "API Key: {}...{}",
        &api_key[..10],
        &api_key[api_key.len() - 10..]
    );
    println!();

    // 创建 Zhipu provider
    let zhipu = Arc::new(ZhipuProvider::new(
        api_key.clone(),
        Some("glm-4-plus".to_string()),
    ));
    println!("✅ Created Zhipu provider with model: glm-4-plus");

    // 测试 1: 简单的文本生成
    println!("\n📝 Test 1: Simple Text Generation");
    println!("{}", "-".repeat(50));

    let config = AgentConfig {
        name: "zhipu_assistant".to_string(),
        instructions: "你是一个有帮助的AI助手，请用中文回答问题。".to_string(),
        ..Default::default()
    };

    let agent = BasicAgent::new(config, zhipu.clone());

    let messages = vec![Message::new(
        Role::User,
        "你好！请用一句话介绍一下你自己。".to_string(),
        None,
        None,
    )];

    let options = AgentGenerateOptions::default();

    match agent.generate(&messages, &options).await {
        Ok(result) => {
            println!("✅ Response: {}", result.response);
            println!("📊 Metadata: {:?}", result.metadata);
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }

    // 测试 2: 多轮对话
    println!("\n💬 Test 2: Multi-turn Conversation");
    println!("{}", "-".repeat(50));

    let conversation = vec![Message::new(
        Role::User,
        "请告诉我三个关于 Rust 编程语言的优点。".to_string(),
        None,
        None,
    )];

    match agent.generate(&conversation, &options).await {
        Ok(result) => {
            println!("✅ Response: {}", result.response);
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }

    // 测试 3: 代码生成
    println!("\n💻 Test 3: Code Generation");
    println!("{}", "-".repeat(50));

    let code_request = vec![Message::new(
        Role::User,
        "请用 Rust 写一个简单的 Hello World 程序。".to_string(),
        None,
        None,
    )];

    match agent.generate(&code_request, &options).await {
        Ok(result) => {
            println!("✅ Response:\n{}", result.response);
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }

    // 测试 4: 使用 generate_simple 便捷方法
    println!("\n⚡ Test 4: Using generate_simple");
    println!("{}", "-".repeat(50));

    match agent.generate_simple("用一句话解释什么是人工智能？").await {
        Ok(response) => {
            println!("✅ Response: {}", response);
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }

    // 测试 5: 测试不同的温度参数
    println!("\n🌡️  Test 5: Different Temperature Settings");
    println!("{}", "-".repeat(50));

    use lumosai_core::llm::types::Temperature;
    use lumosai_core::llm::LlmOptions;

    let creative_options = AgentGenerateOptions {
        llm_options: LlmOptions {
            temperature: Some(Temperature::CREATIVE),
            max_tokens: Some(100),
            ..Default::default()
        },
        ..Default::default()
    };

    let creative_request = vec![Message::new(
        Role::User,
        "写一个关于月亮的诗句。".to_string(),
        None,
        None,
    )];

    match agent.generate(&creative_request, &creative_options).await {
        Ok(result) => {
            println!("✅ Creative Response (temp=0.9): {}", result.response);
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }

    // 测试 6: 测试 JSON 输出
    println!("\n📋 Test 6: JSON Output");
    println!("{}", "-".repeat(50));

    let json_request = vec![
        Message::new(
            Role::User,
            "请以 JSON 格式返回三个编程语言的名称和它们的主要用途。格式：{\"languages\": [{\"name\": \"...\", \"use\": \"...\"}]}".to_string(),
            None,
            None,
        ),
    ];

    match agent.generate(&json_request, &options).await {
        Ok(result) => {
            println!("✅ JSON Response:\n{}", result.response);
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }

    // 测试 7: 长文本生成
    println!("\n📚 Test 7: Long Text Generation");
    println!("{}", "-".repeat(50));

    let long_options = AgentGenerateOptions {
        llm_options: LlmOptions {
            max_tokens: Some(500),
            ..Default::default()
        },
        ..Default::default()
    };

    let long_request = vec![Message::new(
        Role::User,
        "请详细解释什么是机器学习，包括它的定义、主要类型和应用场景。".to_string(),
        None,
        None,
    )];

    match agent.generate(&long_request, &long_options).await {
        Ok(result) => {
            println!(
                "✅ Long Response ({} chars):\n{}",
                result.response.len(),
                result.response
            );
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }

    // 测试 8: 错误处理 - 空消息
    println!("\n⚠️  Test 8: Error Handling - Empty Message");
    println!("{}", "-".repeat(50));

    let empty_request = vec![Message::new(Role::User, "".to_string(), None, None)];

    match agent.generate(&empty_request, &options).await {
        Ok(result) => {
            println!("✅ Response to empty: {}", result.response);
        }
        Err(e) => {
            println!("⚠️  Expected error: {:?}", e);
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("✅ All tests completed!");
    println!("\n💡 Tips:");
    println!("  - 确保 ZHIPU_API_KEY 环境变量已设置");
    println!("  - 可以使用不同的模型: glm-4-plus, glm-4-plus-flash, glm-3-turbo");
    println!("  - 调整 temperature 参数控制创造性 (0.0-1.0)");
    println!("  - 使用 max_tokens 限制响应长度");

    Ok(())
}
