//! MVP 示例 4: Agent 上下文对话
//!
//! 这个示例展示如何使用 Agent 进行多轮对话，Agent 会记住对话历史。
//!
//! 运行方式:
//! ```bash
//! cargo run --package lumosai_examples --example mvp_04_agent_with_memory
//! ```

use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::base::Base;
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MVP 示例 4: Agent 上下文对话\n");
    println!("{}", "=".repeat(50));

    // 步骤 1: 创建 Agent
    println!("\n📝 步骤 1: 创建 Agent");
    let llm = create_test_zhipu_provider_arc();
    let agent = AgentBuilder::new()
        .name("context_assistant")
        .instructions("You are a helpful assistant. Remember the context of the conversation and provide relevant responses.")
        .model(llm)
        .build()?;
    println!("✅ Agent '{}' 创建成功", agent.name().unwrap_or("unknown"));

    // 步骤 2: 进行多轮对话
    println!("\n📝 步骤 2: 多轮对话示例");
    println!("{}", "━".repeat(50));

    // 对话 1: 告诉 Agent 用户的信息
    println!("\n💬 对话 1: 提供个人信息");
    let message1 = "My name is Alice and I'm a software engineer working on AI projects.";
    println!("👤 用户: {}", message1);

    let response1 = agent.generate_simple(message1).await?;
    println!("🤖 Agent: {}\n", response1);

    // 对话 2: 询问兴趣
    println!("💬 对话 2: 分享兴趣爱好");
    let message2 = "I love programming in Rust and building AI applications.";
    println!("👤 用户: {}", message2);

    let response2 = agent.generate_simple(message2).await?;
    println!("🤖 Agent: {}\n", response2);

    // 对话 3: 简单问题
    println!("💬 对话 3: 询问技术建议");
    let message3 = "What are some good practices for Rust programming?";
    println!("👤 用户: {}", message3);

    let response3 = agent.generate_simple(message3).await?;
    println!("🤖 Agent: {}", response3);

    println!("\n{}", "=".repeat(50));
    println!("✅ Agent 对话示例完成！");
    println!("\n💡 关键概念:");
    println!("   - Agent 通过多轮对话提供帮助");
    println!("   - 每次调用 generate_simple() 都会与 Agent 交互");
    println!("   - Agent 根据配置的指令（instructions）生成响应");
    println!("   - 可以设计对话流程实现复杂交互");
    println!("\n🔄 对话管理:");
    println!("   - 使用 Agent 的 generate_simple() 方法进行对话");
    println!("   - 可以添加记忆系统来保存对话历史");
    println!("   - 支持多轮对话和上下文保持");
    println!("\n💾 记忆系统（高级功能）:");
    println!("   - BasicMemory: 基础记忆实现");
    println!("   - WorkingMemory: 工作记忆（有容量限制）");
    println!("   - SemanticMemory: 语义记忆（基于向量检索）");
    println!("\n下一步: 查看 mvp_05_workflow.rs 学习工作流编排");

    Ok(())
}
