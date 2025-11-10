//! 智谱 AI 简单测试
//!
//! 运行方式:
//! ```bash
//! export ZHIPU_API_KEY="your-api-key"
//! cargo run --example zhipu_simple_test
//! ```

use lumosai_core::llm::types::Temperature;
use lumosai_core::llm::{LlmOptions, LlmProvider, Message, Role, ZhipuProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("ZHIPU_API_KEY")
        .unwrap_or_else(|_| "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k".to_string());

    println!("🚀 智谱 AI 简单测试");
    println!("{}", "=".repeat(50));
    println!(
        "API Key: {}...{}\n",
        &api_key[..10],
        &api_key[api_key.len() - 10..]
    );

    // 创建 Zhipu provider
    let zhipu = ZhipuProvider::new(api_key, Some("glm-4-plus".to_string()));

    // 测试 1: 使用 generate 方法（简单字符串输入）
    println!("📝 Test 1: generate() method");
    println!("{}", "-".repeat(50));

    let options1 = LlmOptions {
        temperature: Some(Temperature::BALANCED),
        max_tokens: Some(100),
        ..Default::default()
    };

    match zhipu
        .generate("你好！请用一句话介绍你自己。", &options1)
        .await
    {
        Ok(response) => {
            println!("✅ Response: {}", response);
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }

    // 测试 2: 使用 generate_with_messages 方法
    println!("\n💬 Test 2: generate_with_messages() method");
    println!("{}", "-".repeat(50));

    let messages = vec![Message::new(
        Role::User,
        "请告诉我三个关于 Rust 编程语言的优点。".to_string(),
        None,
        None,
    )];

    let options2 = LlmOptions {
        temperature: Some(Temperature::BALANCED),
        max_tokens: Some(200),
        ..Default::default()
    };

    match zhipu.generate_with_messages(&messages, &options2).await {
        Ok(response) => {
            println!("✅ Response: {}", response);
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }

    // 测试 3: 高创造性温度
    println!("\n🌡️  Test 3: High temperature (creative)");
    println!("{}", "-".repeat(50));

    let options3 = LlmOptions {
        temperature: Some(Temperature::CREATIVE),
        max_tokens: Some(100),
        ..Default::default()
    };

    match zhipu.generate("写一个关于月亮的诗句。", &options3).await {
        Ok(response) => {
            println!("✅ Response: {}", response);
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }

    // 测试 4: 低温度（确定性）
    println!("\n🎯 Test 4: Low temperature (deterministic)");
    println!("{}", "-".repeat(50));

    let options4 = LlmOptions {
        temperature: Some(Temperature::LOW),
        max_tokens: Some(50),
        ..Default::default()
    };

    match zhipu.generate("1+1=?", &options4).await {
        Ok(response) => {
            println!("✅ Response: {}", response);
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }

    // 测试 5: 代码生成
    println!("\n💻 Test 5: Code generation");
    println!("{}", "-".repeat(50));

    let options5 = LlmOptions {
        temperature: Some(Temperature::BALANCED),
        max_tokens: Some(300),
        ..Default::default()
    };

    match zhipu
        .generate("请用 Rust 写一个 Hello World 程序。", &options5)
        .await
    {
        Ok(response) => {
            println!("✅ Response:\n{}", response);
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("✅ 测试完成！");

    Ok(())
}
