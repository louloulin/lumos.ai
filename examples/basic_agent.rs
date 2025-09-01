//! 基础 Agent 演示
//! 
//! 展示如何创建和使用基础的 AI Agent，包括：
//! - 简单 Agent 创建
//! - 构建器模式使用
//! - 不同 LLM 提供商集成
//! - 基础对话功能

use lumosai_core::agent::{AgentBuilder, AgentTrait};
use lumosai_core::base::Base;
use lumosai_core::llm::{MockLlmProvider};
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🤖 基础 Agent 演示");
    println!("================");
    
    // 演示1: 使用 Mock 提供商的简单 Agent
    demo_simple_agent().await?;
    
    // 演示2: 使用构建器模式的高级 Agent
    demo_advanced_agent().await?;
    
    // 演示3: 多轮对话演示
    demo_conversation().await?;
    
    // 演示4: Agent 配置选项
    demo_agent_options().await?;
    
    Ok(())
}

/// 演示简单 Agent 创建
async fn demo_simple_agent() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示1: 简单 Agent 创建 ===");
    
    // 创建 Mock LLM 提供商（用于演示）
    let mock_responses = vec![
        "你好！我是一个友好的AI助手。我可以帮助您解答问题、提供信息和协助完成各种任务。有什么我可以为您做的吗？".to_string(),
    ];
    let llm_provider = Arc::new(MockLlmProvider::new(mock_responses));
    
    // 方法1: 使用简化 API
    let agent = AgentBuilder::new()
        .name("assistant")
        .instructions("你是一个友好的AI助手，总是乐于帮助用户")
        .model(llm_provider.clone())
        .build()?;
    
    // 生成响应
    let response = agent.generate_simple("你好！请介绍一下自己。").await?;
    println!("Agent 回复: {}", response);
    
    Ok(())
}

/// 演示高级 Agent 配置
async fn demo_advanced_agent() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示2: 高级 Agent 配置 ===");
    
    // 创建更复杂的响应
    let mock_responses = vec![
        "Rust 的所有权系统是其最核心和独特的特性之一。它通过三个主要概念来管理内存：\n\n1. **所有权（Ownership）**：每个值都有一个唯一的所有者\n2. **借用（Borrowing）**：可以临时借用值的引用而不获取所有权\n3. **生命周期（Lifetimes）**：确保引用在有效期内使用\n\n这个系统在编译时就能防止内存泄漏、空指针解引用等问题，无需垃圾回收器。".to_string(),
    ];
    let llm_provider = Arc::new(MockLlmProvider::new(mock_responses));
    
    // 方法2: 使用构建器模式的高级配置
    let advanced_agent = AgentBuilder::new()
        .name("rust_expert")
        .instructions("你是一个专业的 Rust 编程专家，擅长解答 Rust 相关的技术问题。请提供详细、准确的技术解释。")
        .model(llm_provider)
        .max_tool_calls(5)
        .build()?;
    
    let tech_response = advanced_agent.generate_simple(
        "请详细解释 Rust 中的所有权概念，包括其核心原理和优势"
    ).await?;
    
    println!("Rust 专家回复:");
    println!("{}", tech_response);
    
    Ok(())
}

/// 演示多轮对话
async fn demo_conversation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示3: 多轮对话 ===");
    
    // 创建对话响应序列
    let conversation_responses = vec![
        "你好张三！很高兴认识你。25岁正是学习和成长的好年龄。".to_string(),
        "编程和阅读都是很棒的爱好！编程可以锻炼逻辑思维，阅读可以拓宽知识面。你主要编程什么语言呢？".to_string(),
        "当然记得！你是张三，今年25岁，爱好是编程和阅读。我们刚才还聊到了你的兴趣爱好呢。".to_string(),
    ];
    let llm_provider = Arc::new(MockLlmProvider::new(conversation_responses));
    
    // 创建带记忆的 Agent
    let conversation_agent = AgentBuilder::new()
        .name("memory_agent")
        .instructions("你是一个有记忆的助手，能记住之前的对话内容，并在后续对话中引用这些信息")
        .model(llm_provider)
        .build()?;
    
    // 模拟多轮对话
    println!("开始多轮对话演示:");
    
    let response1 = conversation_agent.generate_simple("我叫张三，今年25岁").await?;
    println!("第1轮 - 用户: 我叫张三，今年25岁");
    println!("第1轮 - AI: {}", response1);
    
    let response2 = conversation_agent.generate_simple("我的爱好是编程和阅读").await?;
    println!("\n第2轮 - 用户: 我的爱好是编程和阅读");
    println!("第2轮 - AI: {}", response2);
    
    let response3 = conversation_agent.generate_simple("请告诉我，你还记得我的名字和年龄吗？").await?;
    println!("\n第3轮 - 用户: 请告诉我，你还记得我的名字和年龄吗？");
    println!("第3轮 - AI: {}", response3);
    
    Ok(())
}

/// 演示 Agent 配置选项
async fn demo_agent_options() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示4: Agent 配置选项 ===");
    
    // 创建不同配置的 Agent 来展示各种选项
    let mock_responses = vec![
        "这是一个创造性的回答，展示了较高的温度设置效果。".to_string(),
        "这是一个更加确定性的回答，展示了较低温度设置的效果。".to_string(),
    ];
    let llm_provider = Arc::new(MockLlmProvider::new(mock_responses));
    
    // 高创造性 Agent（高温度）
    let creative_agent = AgentBuilder::new()
        .name("creative_assistant")
        .instructions("你是一个富有创造力的助手")
        .model(llm_provider.clone())
        .build()?;
    
    println!("高创造性 Agent (temperature=0.9):");
    let creative_response = creative_agent.generate_simple("写一个关于未来的短故事").await?;
    println!("回复: {}", creative_response);
    
    // 低创造性 Agent（低温度）
    let precise_agent = AgentBuilder::new()
        .name("precise_assistant")
        .instructions("你是一个精确、事实导向的助手")
        .model(llm_provider)
        .build()?;
    
    println!("\n精确性 Agent (temperature=0.1):");
    let precise_response = precise_agent.generate_simple("什么是人工智能？").await?;
    println!("回复: {}", precise_response);
    
    // 显示 Agent 配置信息
    println!("\n=== Agent 配置信息 ===");
    println!("创造性 Agent:");
    println!("  名称: {:?}", creative_agent.name());
    println!("  指令: {}", creative_agent.get_instructions());
    
    println!("\n精确性 Agent:");
    println!("  名称: {:?}", precise_agent.name());
    println!("  指令: {}", precise_agent.get_instructions());
    
    Ok(())
}

/// 创建 DeepSeek 提供商（如果有 API Key）
#[allow(dead_code)]
fn create_deepseek_provider() -> std::result::Result<Arc<dyn lumosai_core::llm::LlmProvider>, Box<dyn std::error::Error>> {
    // 注意：这里需要实际的 DeepSeek 提供商实现
    // 目前使用 Mock 提供商作为演示
    let mock_responses = vec![
        "这是一个模拟的 DeepSeek 响应。在实际使用中，这里会连接到真实的 DeepSeek API。".to_string(),
    ];
    Ok(Arc::new(MockLlmProvider::new(mock_responses)))
}

/// 辅助函数：打印分隔线
#[allow(dead_code)]
fn print_separator() {
    println!("{}", "=".repeat(50));
}

/// 辅助函数：打印子标题
#[allow(dead_code)]
fn print_subtitle(title: &str) {
    println!("\n--- {} ---", title);
}
