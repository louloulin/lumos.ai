//! BasicAgent 使用示例
//!
//! 这个示例展示了如何使用模块化的 BasicAgent（AgentCore、AgentExecutor、AgentGenerator、BasicAgent）
//!
//! # 运行示例
//!
//! ```bash
//! cargo run --example refactored_agent_demo --features="examples"
//! ```

use futures::stream::StreamExt;
use lumosai_core::agent::refactored::{AgentCore, AgentExecutor, AgentGenerator, BasicAgent};
use lumosai_core::agent::types::{AgentGenerateOptions, AgentStreamOptions};
use lumosai_core::agent::AgentConfig;
use lumosai_core::llm::{Message, MockLlmProvider, Role};
use lumosai_core::memory::{BasicMemory, Memory};
use lumosai_core::tool::create_tool;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BasicAgent 使用示例 ===\n");

    // 示例 1: 使用 BasicAgent（最简单的方式）
    example1_simple_usage().await?;
    println!();

    // 示例 2: 使用模块化组件（更灵活）
    example2_modular_components().await?;
    println!();

    // 示例 3: 使用内存
    example3_with_memory().await?;
    println!();

    // 示例 4: 使用工具
    example4_with_tools().await?;
    println!();

    // 示例 5: 流式生成
    example5_streaming().await?;

    Ok(())
}

/// 示例 1: 使用 BasicAgent（最简单的方式）
async fn example1_simple_usage() -> Result<(), Box<dyn std::error::Error>> {
    println!("示例 1: 使用 BasicAgent（最简单的方式）");

    let config = AgentConfig {
        name: "simple-agent".to_string(),
        instructions: "You are a helpful assistant.".to_string(),
        ..Default::default()
    };
    let llm = Arc::new(MockLlmProvider::new(vec![
        "Hello! How can I help you?".to_string()
    ]));

    let agent = BasicAgent::new(config, llm)?;

    let messages = vec![Message {
        role: Role::User,
        content: "Hello!".to_string(),
        metadata: None,
        name: None,
    }];

    let options = AgentGenerateOptions::default();
    let result = agent.generate(&messages, &options).await?;

    println!("Agent 名称: {}", agent.name());
    println!("Agent 指令: {}", agent.instructions());
    println!("响应: {}", result.response);
    println!("步骤数: {}", result.steps.len());

    Ok(())
}

/// 示例 2: 使用模块化组件（更灵活）
async fn example2_modular_components() -> Result<(), Box<dyn std::error::Error>> {
    println!("示例 2: 使用模块化组件（更灵活）");

    let config = AgentConfig {
        name: "modular-agent".to_string(),
        instructions: "You are a helpful assistant.".to_string(),
        ..Default::default()
    };
    let llm = Arc::new(MockLlmProvider::new(vec![
        "Hello! How can I help you?".to_string()
    ]));

    // 步骤 1: 创建 AgentCore
    let core = AgentCore::new(config, llm)?;
    println!("创建了 AgentCore: {}", core.name());

    // 步骤 2: 创建 AgentExecutor
    let executor = AgentExecutor::new(core)?;
    println!("创建了 AgentExecutor");

    // 步骤 3: 创建 AgentGenerator
    let generator = AgentGenerator::new(executor);
    println!("创建了 AgentGenerator");

    // 步骤 4: 使用 Generator 生成响应
    let messages = vec![Message {
        role: Role::User,
        content: "What is your name?".to_string(),
        metadata: None,
        name: None,
    }];

    let options = AgentGenerateOptions::default();
    let result = generator.generate(&messages, &options).await?;

    println!("响应: {}", result.response);

    Ok(())
}

/// 示例 3: 使用内存
async fn example3_with_memory() -> Result<(), Box<dyn std::error::Error>> {
    println!("示例 3: 使用内存");

    let config = AgentConfig {
        name: "memory-agent".to_string(),
        instructions: "You are a helpful assistant with memory.".to_string(),
        ..Default::default()
    };
    let llm = Arc::new(MockLlmProvider::new(vec![
        "I remember you said hello earlier!".to_string(),
    ]));

    // 创建内存
    let memory: Arc<dyn Memory> = Arc::new(BasicMemory::new(None, None));

    // 使用内存创建 Agent
    let agent = BasicAgent::with_memory(config, llm, memory.clone())?;

    // 存储第一条消息
    let message1 = Message {
        role: Role::User,
        content: "My name is Alice".to_string(),
        metadata: None,
        name: None,
    };
    memory.store(&message1).await?;

    println!("存储了消息: {}", message1.content);
    println!("Agent 有内存: {}", agent.has_memory());

    // 生成响应（应该能访问历史消息）
    let messages = vec![Message {
        role: Role::User,
        content: "What is my name?".to_string(),
        metadata: None,
        name: None,
    }];

    let options = AgentGenerateOptions {
        thread_id: Some("test-thread".to_string()),
        ..Default::default()
    };

    let result = agent.generate(&messages, &options).await?;
    println!("响应: {}", result.response);

    Ok(())
}

/// 示例 4: 使用工具
async fn example4_with_tools() -> Result<(), Box<dyn std::error::Error>> {
    println!("示例 4: 使用工具");

    let config = AgentConfig {
        name: "tool-agent".to_string(),
        instructions: "You are a helpful assistant with tools.".to_string(),
        ..Default::default()
    };
    let llm = Arc::new(MockLlmProvider::new(vec!["I can use tools!".to_string()]));

    // 创建 AgentCore 和 AgentExecutor
    let core = AgentCore::new(config, llm)?;
    let mut executor = AgentExecutor::new(core)?;

    // 添加工具
    let echo_tool = create_tool(
        "echo",
        "Echo a message",
        vec![("message", "string", "Message to echo", true)],
        |params| {
            let message = params
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("No message");
            Ok(serde_json::json!({"echo": message}))
        },
    )?;

    executor.add_tool(Box::new(echo_tool))?;
    println!("添加了工具: echo");

    // 创建 Generator
    let generator = AgentGenerator::new(executor);

    // 生成响应
    let messages = vec![Message {
        role: Role::User,
        content: "Hello!".to_string(),
        metadata: None,
        name: None,
    }];

    let options = AgentGenerateOptions::default();
    let result = generator.generate(&messages, &options).await?;

    println!("响应: {}", result.response);
    println!("步骤数: {}", result.steps.len());

    Ok(())
}

/// 示例 5: 流式生成
async fn example5_streaming() -> Result<(), Box<dyn std::error::Error>> {
    println!("示例 5: 流式生成");

    let config = AgentConfig {
        name: "streaming-agent".to_string(),
        instructions: "You are a helpful assistant.".to_string(),
        ..Default::default()
    };
    let llm = Arc::new(MockLlmProvider::new(vec![
        "This is a longer response that will be streamed in chunks.".to_string(),
    ]));

    let agent = BasicAgent::new(config, llm)?;

    let messages = vec![Message {
        role: Role::User,
        content: "Tell me a story".to_string(),
        metadata: None,
        name: None,
    }];

    let options = AgentStreamOptions::default();
    let mut stream = agent.stream(&messages, &options).await?;

    println!("流式响应:");
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        print!("{}", chunk);
    }
    println!();

    Ok(())
}
