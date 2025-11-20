//! 华为 MaaS Agent 示例
//!
//! 展示如何使用 AgentBuilder 创建基于华为 ModelArts MaaS 的 AI Agent
//!
//! # 环境变量设置
//!
//! ```bash
//! export MAAS_API_KEY="your_maas_api_key"
//! export MAAS_MODEL="deepseek-v3.2-exp"  # 可选，默认使用 deepseek-v3.2-exp
//! ```
//!
//! # 运行示例
//!
//! ```bash
//! cargo run --example huawei_maas_agent --features="llm"
//! ```

use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::llm::{providers, LlmOptions, Message, Role};
use lumosai_core::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== 华为 MaaS Agent 示例 ===\n");

    // 方法 1: 从环境变量自动创建 MaaS provider
    println!("方法 1: 从环境变量创建 MaaS provider");
    let maas_provider = match providers::huawei_maas_from_env() {
        Ok(provider) => {
            println!("✅ 成功从环境变量创建华为 MaaS provider");
            Arc::new(provider) as Arc<dyn lumosai_core::llm::LlmProvider>
        }
        Err(e) => {
            println!("❌ 无法从环境变量创建 MaaS provider: {}", e);
            println!("   请设置 MAAS_API_KEY 环境变量");
            return Err(e);
        }
    };

    // 创建基于 MaaS 的 Agent
    let agent = AgentBuilder::new()
        .name("maas_assistant")
        .instructions("你是一个基于华为 MaaS 的智能助手，擅长回答技术问题。请用中文回答。")
        .model(maas_provider.clone())
        .temperature(0.7)
        .max_tokens(1000)
        .build()?;

    println!("\n✅ 成功创建 Agent: {}", agent.name().unwrap_or("unnamed"));

    // 示例 1: 简单对话
    println!("\n--- 示例 1: 简单对话 ---");
    let question1 = "什么是 Rust 编程语言？请简要介绍。";
    println!("用户: {}", question1);

    match agent.generate(question1, &LlmOptions::default()).await {
        Ok(response) => {
            println!("助手: {}\n", response);
        }
        Err(e) => {
            println!("❌ 生成失败: {}\n", e);
        }
    }

    // 示例 2: 多轮对话
    println!("--- 示例 2: 多轮对话 ---");
    let messages = vec![
        Message {
            role: Role::System,
            content: "你是一个专业的技术顾问".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "云计算的主要优势是什么？".to_string(),
            metadata: None,
            name: None,
        },
    ];

    match agent
        .generate_with_messages(&messages, &LlmOptions::default())
        .await
    {
        Ok(response) => {
            println!("助手: {}\n", response);
        }
        Err(e) => {
            println!("❌ 生成失败: {}\n", e);
        }
    }

    // 示例 3: 流式响应
    println!("--- 示例 3: 流式响应 ---");
    let question3 = "请讲一个关于人工智能的小故事（50字以内）";
    println!("用户: {}", question3);
    print!("助手: ");

    use futures::StreamExt;
    match agent
        .generate_stream(question3, &LlmOptions::default())
        .await
    {
        Ok(mut stream) => {
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(text) => print!("{}", text),
                    Err(e) => {
                        println!("\n❌ 流式错误: {}", e);
                        break;
                    }
                }
            }
            println!("\n");
        }
        Err(e) => {
            println!("\n❌ 流式生成失败: {}\n", e);
        }
    }

    // 方法 2: 使用 auto_provider 自动选择（包括 MaaS）
    println!("--- 方法 2: 使用 auto_provider ---");
    match providers::auto_provider() {
        Ok(auto_llm) => {
            println!("✅ 自动选择的 provider: {}", auto_llm.name());

            let auto_agent = AgentBuilder::new()
                .name("auto_assistant")
                .instructions("你是一个智能助手")
                .model(Arc::new(auto_llm))
                .build()?;

            let test_question = "你好，请介绍一下自己";
            match auto_agent
                .generate(test_question, &LlmOptions::default())
                .await
            {
                Ok(response) => {
                    println!("响应: {}\n", response);
                }
                Err(e) => {
                    println!("❌ 生成失败: {}\n", e);
                }
            }
        }
        Err(e) => {
            println!("❌ auto_provider 失败: {}", e);
        }
    }

    println!("=== 示例完成 ===");
    Ok(())
}
