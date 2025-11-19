//! 华为 MaaS 流式响应示例
//!
//! 演示如何使用华为 ModelArts MaaS 服务的流式响应功能
//!
//! 运行方式:
//! ```bash
//! export HUAWEI_MAAS_API_KEY="your-api-key"
//! cargo run --example huawei_maas_streaming_demo
//! ```

use futures::StreamExt;
use lumosai_core::llm::{HuaweiMaasProvider, LlmOptions, LlmProvider, Message, Role};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("华为 MaaS 流式响应示例");
    println!("========================================\n");

    // 从环境变量创建 provider
    let provider = HuaweiMaasProvider::from_env()?;
    println!("✅ Provider 初始化成功: {}\n", provider.name());

    // 测试1: 简单流式生成
    println!("========================================");
    println!("测试1: 简单流式文本生成");
    println!("========================================");

    let prompt = "请用100字左右介绍一下 Rust 编程语言的主要特点";
    println!("提示: {}\n", prompt);
    println!("流式响应:");
    println!("---");

    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(500);

    let mut stream = provider.generate_stream(prompt, &options).await?;
    let mut full_response = String::new();

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                print!("{}", chunk);
                io::stdout().flush()?;
                full_response.push_str(&chunk);
            }
            Err(e) => {
                eprintln!("\n❌ 流式错误: {}", e);
                break;
            }
        }
    }

    println!("\n---");
    println!("✅ 流式响应完成，总长度: {} 字符\n", full_response.len());

    // 测试2: 多轮对话流式生成
    println!("========================================");
    println!("测试2: 多轮对话流式生成");
    println!("========================================");

    let messages = vec![
        Message {
            role: Role::System,
            content: "你是一个有帮助的AI助手，擅长用简洁的语言解释复杂概念。".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "请用3个要点解释什么是异步编程".to_string(),
            metadata: None,
            name: None,
        },
    ];

    println!("系统提示: 你是一个有帮助的AI助手，擅长用简洁的语言解释复杂概念。");
    println!("用户: 请用3个要点解释什么是异步编程\n");
    println!("流式响应:");
    println!("---");

    // 注意：目前 generate_stream 只支持简单 prompt，这里我们使用最后一条消息
    let last_message_content = messages.last().unwrap().content.clone();
    let mut stream = provider.generate_stream(&last_message_content, &options).await?;
    let mut full_response = String::new();

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                print!("{}", chunk);
                io::stdout().flush()?;
                full_response.push_str(&chunk);
            }
            Err(e) => {
                eprintln!("\n❌ 流式错误: {}", e);
                break;
            }
        }
    }

    println!("\n---");
    println!("✅ 流式响应完成，总长度: {} 字符\n", full_response.len());

    // 测试3: 代码生成流式响应
    println!("========================================");
    println!("测试3: 代码生成流式响应");
    println!("========================================");

    let code_prompt = "请用 Rust 编写一个简单的 HTTP 客户端示例，使用 reqwest 库发送 GET 请求";
    println!("提示: {}\n", code_prompt);
    println!("流式响应:");
    println!("---");

    let code_options = LlmOptions::default()
        .with_temperature(0.3)
        .with_max_tokens(1000);

    let mut stream = provider.generate_stream(code_prompt, &code_options).await?;
    let mut full_response = String::new();

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                print!("{}", chunk);
                io::stdout().flush()?;
                full_response.push_str(&chunk);
            }
            Err(e) => {
                eprintln!("\n❌ 流式错误: {}", e);
                break;
            }
        }
    }

    println!("\n---");
    println!("✅ 流式响应完成，总长度: {} 字符\n", full_response.len());

    // 测试4: 创意写作流式响应
    println!("========================================");
    println!("测试4: 创意写作流式响应");
    println!("========================================");

    let creative_prompt = "请写一首关于编程的五言绝句";
    println!("提示: {}\n", creative_prompt);
    println!("流式响应:");
    println!("---");

    let creative_options = LlmOptions::default()
        .with_temperature(0.9)
        .with_max_tokens(200);

    let mut stream = provider
        .generate_stream(creative_prompt, &creative_options)
        .await?;
    let mut full_response = String::new();

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                print!("{}", chunk);
                io::stdout().flush()?;
                full_response.push_str(&chunk);
                
                // 添加一点延迟以更好地展示流式效果
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
            Err(e) => {
                eprintln!("\n❌ 流式错误: {}", e);
                break;
            }
        }
    }

    println!("\n---");
    println!("✅ 流式响应完成，总长度: {} 字符\n", full_response.len());

    // 测试5: 对比流式和非流式响应
    println!("========================================");
    println!("测试5: 对比流式和非流式响应");
    println!("========================================");

    let test_prompt = "用一句话解释什么是 Rust 的所有权系统";
    println!("提示: {}\n", test_prompt);

    // 非流式
    println!("【非流式响应】");
    let start = std::time::Instant::now();
    match provider.generate(test_prompt, &options).await {
        Ok(response) => {
            let duration = start.elapsed();
            println!("{}", response);
            println!("耗时: {:?}\n", duration);
        }
        Err(e) => {
            eprintln!("❌ 错误: {}\n", e);
        }
    }

    // 流式
    println!("【流式响应】");
    let start = std::time::Instant::now();
    let mut stream = provider.generate_stream(test_prompt, &options).await?;
    let mut first_chunk_time = None;
    let mut full_response = String::new();

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                if first_chunk_time.is_none() {
                    first_chunk_time = Some(start.elapsed());
                }
                print!("{}", chunk);
                io::stdout().flush()?;
                full_response.push_str(&chunk);
            }
            Err(e) => {
                eprintln!("\n❌ 流式错误: {}", e);
                break;
            }
        }
    }

    let total_duration = start.elapsed();
    println!();
    if let Some(first_time) = first_chunk_time {
        println!("首个响应块耗时: {:?}", first_time);
    }
    println!("总耗时: {:?}\n", total_duration);

    println!("========================================");
    println!("✅ 所有流式测试完成！");
    println!("========================================");

    Ok(())
}

