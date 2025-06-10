//! 流式响应演示
//! 
//! 展示如何使用流式响应功能，包括：
//! - 基础流式输出
//! - 实时内容流
//! - 事件驱动流处理
//! - WebSocket 流式连接

use lumosai_core::prelude::*;
use lumosai_core::agent::{AgentBuilder, BasicAgent};
use lumosai_core::agent::streaming::{StreamingAgent, AgentEvent, StreamingConfig, IntoStreaming};
use lumosai_core::llm::{MockLlmProvider, Message, Role};
use futures::StreamExt;
use std::sync::Arc;
use std::io::{self, Write};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 流式响应演示");
    println!("================");
    
    // 演示1: 基础流式响应
    demo_basic_streaming().await?;
    
    // 演示2: 高级流式配置
    demo_advanced_streaming().await?;
    
    // 演示3: 事件驱动流处理
    demo_event_driven_streaming().await?;
    
    // 演示4: 流式工具调用
    demo_streaming_with_tools().await?;
    
    Ok(())
}

/// 演示基础流式响应
async fn demo_basic_streaming() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示1: 基础流式响应 ===");
    
    // 创建模拟流式响应
    let streaming_content = "人工智能的发展历史可以追溯到20世纪50年代。1950年，艾伦·图灵提出了著名的图灵测试。1956年，达特茅斯会议标志着人工智能学科的正式诞生。随后经历了多次发展浪潮，包括专家系统时代、机器学习兴起，直到近年来深度学习的突破性进展。";
    
    let mock_provider = Arc::new(MockLlmProvider::new(vec![streaming_content.to_string()]));
    
    // 创建支持流式的 Agent
    let agent = AgentBuilder::new()
        .name("streaming_agent")
        .instructions("你是一个助手，请详细回答用户问题")
        .model(mock_provider)
        .build()?;
    
    // 转换为流式 Agent
    let streaming_agent = agent.into_streaming();
    
    println!("\n问题: 请详细介绍一下人工智能的发展历史");
    print!("AI回复: ");
    io::stdout().flush().unwrap();
    
    // 发起流式请求
    let mut stream = streaming_agent.generate_stream(
        "请详细介绍一下人工智能的发展历史，包括重要的里程碑事件"
    ).await?;
    
    // 处理流式响应
    let mut full_content = String::new();
    while let Some(event) = stream.next().await {
        match event? {
            AgentEvent::ContentDelta { delta } => {
                print!("{}", delta);
                io::stdout().flush().unwrap();
                full_content.push_str(&delta);
                
                // 模拟流式延迟
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
            AgentEvent::Completed { final_content } => {
                println!("\n\n=== 流式响应完成 ===");
                println!("完整内容长度: {} 字符", final_content.len());
                break;
            }
            _ => {}
        }
    }
    
    Ok(())
}

/// 演示高级流式配置
async fn demo_advanced_streaming() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示2: 高级流式配置 ===");
    
    // 创建流式配置
    let streaming_config = StreamingConfig {
        buffer_size: 1024,
        flush_interval_ms: 100,
        enable_partial_json: true,
        enable_tool_streaming: true,
        max_concurrent_streams: 5,
    };
    
    println!("流式配置:");
    println!("  缓冲区大小: {} 字节", streaming_config.buffer_size);
    println!("  刷新间隔: {} 毫秒", streaming_config.flush_interval_ms);
    println!("  启用部分JSON: {}", streaming_config.enable_partial_json);
    println!("  启用工具流式: {}", streaming_config.enable_tool_streaming);
    println!("  最大并发流: {}", streaming_config.max_concurrent_streams);
    
    // 创建长文本响应
    let long_response = "Rust编程语言是一门系统编程语言，由Mozilla开发。它的设计目标是提供内存安全、并发安全和高性能。Rust的核心特性包括所有权系统、借用检查器、零成本抽象等。所有权系统通过编译时检查来防止内存泄漏和数据竞争。借用检查器确保引用的有效性。零成本抽象意味着高级特性不会带来运行时开销。Rust还提供了强大的类型系统、模式匹配、trait系统等现代编程语言特性。";
    
    let mock_provider = Arc::new(MockLlmProvider::new(vec![long_response.to_string()]));
    
    let agent = AgentBuilder::new()
        .name("advanced_streaming_agent")
        .instructions("你是一个技术专家，请提供详细的技术解释")
        .model(mock_provider)
        .streaming_config(streaming_config)
        .build()?;
    
    let streaming_agent = agent.into_streaming();
    
    println!("\n问题: 请详细解释Rust编程语言的特性");
    print!("AI回复: ");
    io::stdout().flush().unwrap();
    
    let mut stream = streaming_agent.generate_stream(
        "请详细解释Rust编程语言的核心特性和设计理念"
    ).await?;
    
    let mut word_count = 0;
    let mut char_count = 0;
    
    while let Some(event) = stream.next().await {
        match event? {
            AgentEvent::ContentDelta { delta } => {
                print!("{}", delta);
                io::stdout().flush().unwrap();
                
                char_count += delta.len();
                word_count += delta.split_whitespace().count();
                
                // 每50个字符显示一次统计
                if char_count % 50 == 0 {
                    print!(" [{}字符]", char_count);
                    io::stdout().flush().unwrap();
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
            }
            AgentEvent::Completed { final_content: _ } => {
                println!("\n\n=== 高级流式响应完成 ===");
                println!("总字符数: {}", char_count);
                println!("总词数: {}", word_count);
                break;
            }
            _ => {}
        }
    }
    
    Ok(())
}

/// 演示事件驱动流处理
async fn demo_event_driven_streaming() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示3: 事件驱动流处理 ===");
    
    let mock_provider = Arc::new(MockLlmProvider::new(vec![
        "我正在分析您的请求...".to_string(),
        "根据分析结果，我建议...".to_string(),
        "最终结论是...".to_string(),
    ]));
    
    let agent = AgentBuilder::new()
        .name("event_driven_agent")
        .instructions("你是一个分析助手，会逐步分析问题")
        .model(mock_provider)
        .build()?;
    
    let streaming_agent = agent.into_streaming();
    
    println!("\n问题: 请分析当前市场趋势");
    println!("事件流处理:");
    
    let mut stream = streaming_agent.generate_stream(
        "请分析当前技术市场的发展趋势"
    ).await?;
    
    let mut event_count = 0;
    let mut content_chunks = Vec::new();
    
    while let Some(event) = stream.next().await {
        event_count += 1;
        
        match event? {
            AgentEvent::StreamStarted => {
                println!("  🚀 事件 {}: 流开始", event_count);
            }
            AgentEvent::ContentDelta { delta } => {
                println!("  📝 事件 {}: 内容片段 - '{}'", event_count, delta.trim());
                content_chunks.push(delta);
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
            AgentEvent::ThinkingStarted => {
                println!("  🤔 事件 {}: 开始思考", event_count);
            }
            AgentEvent::ThinkingCompleted => {
                println!("  💡 事件 {}: 思考完成", event_count);
            }
            AgentEvent::Completed { final_content } => {
                println!("  ✅ 事件 {}: 流完成", event_count);
                println!("     完整内容: {}", final_content);
                break;
            }
            AgentEvent::Error { error } => {
                println!("  ❌ 事件 {}: 错误 - {}", event_count, error);
            }
            _ => {
                println!("  ℹ️  事件 {}: 其他事件", event_count);
            }
        }
    }
    
    println!("\n事件统计:");
    println!("  总事件数: {}", event_count);
    println!("  内容片段数: {}", content_chunks.len());
    
    Ok(())
}

/// 演示流式工具调用
async fn demo_streaming_with_tools() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示4: 流式工具调用 ===");
    
    // 创建模拟工具调用响应
    let tool_responses = vec![
        "我需要使用计算器工具来计算这个表达式...".to_string(),
        "正在调用计算器工具...".to_string(),
        "计算完成！结果是126。".to_string(),
    ];
    
    let mock_provider = Arc::new(MockLlmProvider::new(tool_responses));
    
    let agent = AgentBuilder::new()
        .name("tool_streaming_agent")
        .instructions("你是一个助手，可以使用工具并实时报告进度")
        .model(mock_provider)
        .build()?;
    
    let streaming_agent = agent.into_streaming();
    
    println!("\n问题: 请计算 (15 + 27) * 3");
    println!("流式工具调用:");
    
    let mut stream = streaming_agent.generate_stream(
        "请使用计算器工具计算 (15 + 27) * 3 的结果"
    ).await?;
    
    while let Some(event) = stream.next().await {
        match event? {
            AgentEvent::ContentDelta { delta } => {
                print!("💬 内容: {}", delta);
                io::stdout().flush().unwrap();
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            AgentEvent::ToolCall { tool_name, arguments } => {
                println!("\n🔧 工具调用: {} - 参数: {}", tool_name, arguments);
            }
            AgentEvent::ToolResult { tool_name, result } => {
                println!("📊 工具结果: {} - 结果: {}", tool_name, result);
            }
            AgentEvent::Completed { final_content } => {
                println!("\n\n✅ 流式工具调用完成");
                println!("最终结果: {}", final_content);
                break;
            }
            AgentEvent::Error { error } => {
                println!("\n❌ 错误: {}", error);
                break;
            }
            _ => {}
        }
    }
    
    Ok(())
}

/// 辅助函数：模拟打字机效果
#[allow(dead_code)]
async fn typewriter_effect(text: &str, delay_ms: u64) {
    for char in text.chars() {
        print!("{}", char);
        io::stdout().flush().unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
    }
}

/// 辅助函数：计算流式统计
#[allow(dead_code)]
fn calculate_streaming_stats(content: &str) -> (usize, usize, usize) {
    let char_count = content.len();
    let word_count = content.split_whitespace().count();
    let line_count = content.lines().count();
    (char_count, word_count, line_count)
}

/// 辅助函数：格式化流式事件
#[allow(dead_code)]
fn format_stream_event(event: &AgentEvent, index: usize) -> String {
    match event {
        AgentEvent::StreamStarted => format!("事件 {}: 流开始", index),
        AgentEvent::ContentDelta { delta } => format!("事件 {}: 内容 '{}'", index, delta.trim()),
        AgentEvent::ToolCall { tool_name, arguments } => format!("事件 {}: 工具调用 {} ({})", index, tool_name, arguments),
        AgentEvent::Completed { .. } => format!("事件 {}: 完成", index),
        AgentEvent::Error { error } => format!("事件 {}: 错误 {}", index, error),
        _ => format!("事件 {}: 其他", index),
    }
}
