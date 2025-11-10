//! MVP 示例 2: Agent 工具概念
//!
//! 这个示例展示 Agent 如何扩展工具能力的概念。
//! （完整的工具集成需要额外的工具系统配置）
//!
//! 运行方式:
//! ```bash
//! cargo run --package lumosai_examples --example mvp_02_agent_with_tools
//! ```

use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::base::Base;
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MVP 示例 2: 带工具的 Agent\n");
    println!("{}", "=".repeat(50));

    // 步骤 1: 理解工具系统
    println!("\n📝 步骤 1: 理解 Agent 工具系统");
    println!("\n🔧 工具的作用:");
    println!("   - 扩展 Agent 的能力边界");
    println!("   - 让 Agent 能够执行特定操作");
    println!("   - 连接 Agent 与外部系统");
    
    println!("\n📦 常见工具类型:");
    println!("   - 计算器工具（数学运算）");
    println!("   - 搜索工具（信息检索）");
    println!("   - 文件工具（文件操作）");
    println!("   - API 工具（调用外部服务）");
    println!("   - 数据库工具（数据查询）");

    // 步骤 2: 创建 LLM 提供商
    println!("\n📝 步骤 2: 创建 LLM 提供商");
    let llm = create_test_zhipu_provider_arc();
    println!("✅ LLM 提供商创建成功");

    // 步骤 3: 使用 AgentBuilder 创建 Agent
    println!("\n📝 步骤 3: 使用 AgentBuilder 创建 Agent");
    let agent = AgentBuilder::new()
        .name("tool_agent")
        .instructions("You are a helpful assistant that can perform calculations and text operations.")
        .model(llm)
        .build()?;

    println!("✅ Agent '{}' 创建成功", agent.name().unwrap_or("unknown"));

    // 步骤 4: 测试 Agent 对话
    println!("\n📝 步骤 4: 测试 Agent 对话");
    println!("{}", "━".repeat(50));

    let questions = vec![
        "What is 123 + 456?",
        "Convert 'Hello World' to uppercase",
        "What is 100 divided by 5?",
    ];

    for (i, question) in questions.iter().enumerate() {
        println!("\n💬 问题 {}: {}", i + 1, question);
        match agent.generate_simple(question).await {
            Ok(response) => println!("🤖 Agent: {}", response),
            Err(e) => eprintln!("❌ 错误: {}", e),
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("✅ 示例完成！");
    println!("\n💡 提示:");
    println!("   - 使用 tool! 宏创建工具（声明式语法）");
    println!("   - 宏会自动生成 <tool_name>_tool() 函数");
    println!("   - 使用 AgentBuilder.tool() 添加工具到 Agent");
    println!("   - 可以添加多个工具，Agent 会自动选择合适的工具");
    println!("   - 下一步: 查看 mvp_03_multi_agent.rs 学习多 Agent 协作");

    Ok(())
}
