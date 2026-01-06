//! MVP 示例 1: 最简单的 Agent
//!
//! 这个示例展示如何使用 AgentBuilder 创建一个最基本的 AI Agent 并进行对话。
//!
//! 运行方式:
//! ```bash
//! cargo run --example mvp_01_simple_agent
//! ```

use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::base::Base;
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MVP 示例 1: 最简单的 Agent\n");
    println!("{}", "=".repeat(50));

    // 步骤 1: 创建 LLM 提供商
    println!("\n📝 步骤 1: 创建 LLM 提供商");
    let llm = create_test_zhipu_provider_arc();
    println!("✅ LLM 提供商创建成功");

    // 步骤 2: 使用 AgentBuilder 创建 Agent
    println!("\n📝 步骤 2: 使用 AgentBuilder 创建 Agent");
    let agent = AgentBuilder::new()
        .name("assistant")
        .instructions("You are a helpful AI assistant. Be concise and friendly.")
        .model(llm)
        .build()?;

    println!("✅ Agent '{}' 创建成功", agent.name().unwrap_or("unknown"));

    // 步骤 3: 发送消息并获取响应
    println!("\n📝 步骤 3: 与 Agent 对话");
    println!("{}", "━".repeat(50));

    let questions = vec![
        "Hello! What can you help me with?",
        "What is 2 + 2?",
        "Tell me a fun fact about Rust programming language.",
    ];

    for (i, question) in questions.iter().enumerate() {
        println!("\n💬 问题 {}: {}", i + 1, question);

        match agent.generate_simple(question).await {
            Ok(response) => {
                println!("🤖 Agent: {}", response);
            }
            Err(e) => {
                eprintln!("❌ 错误: {}", e);
            }
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("✅ 示例完成！");
    println!("\n💡 提示:");
    println!("   - 使用 AgentBuilder 创建 Agent（推荐方式）");
    println!("   - Agent 使用测试 LLM 提供商（不需要 API 密钥）");
    println!("   - AgentBuilder 支持链式调用，配置更灵活");
    println!("   - 下一步: 查看 mvp_02_agent_with_tools.rs 学习如何添加工具");

    Ok(())
}
