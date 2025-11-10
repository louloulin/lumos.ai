//! MVP 示例 3: 多 Agent 协作
//!
//! 这个示例展示如何创建多个 Agent 并让它们协同工作完成复杂任务。
//!
//! 运行方式:
//! ```bash
//! cargo run --package lumosai_examples --example mvp_03_multi_agent
//! ```

use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::base::Base;
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MVP 示例 3: 多 Agent 协作\n");
    println!("{}", "=".repeat(50));

    // 步骤 1: 创建研究 Agent
    println!("\n📝 步骤 1: 创建研究 Agent");
    let llm1 = create_test_zhipu_provider_arc();
    let researcher = AgentBuilder::new()
        .name("researcher")
        .instructions("You are a researcher. Gather information and provide facts about the topic.")
        .model(llm1)
        .build()?;
    println!(
        "✅ 研究 Agent '{}' 创建成功",
        researcher.name().unwrap_or("unknown")
    );

    // 步骤 2: 创建写作 Agent
    println!("\n📝 步骤 2: 创建写作 Agent");
    let llm2 = create_test_zhipu_provider_arc();
    let writer = AgentBuilder::new()
        .name("writer")
        .instructions("You are a writer. Create engaging content based on research findings.")
        .model(llm2)
        .build()?;
    println!(
        "✅ 写作 Agent '{}' 创建成功",
        writer.name().unwrap_or("unknown")
    );

    // 步骤 3: 创建编辑 Agent
    println!("\n📝 步骤 3: 创建编辑 Agent");
    let llm3 = create_test_zhipu_provider_arc();
    let editor = AgentBuilder::new()
        .name("editor")
        .instructions("You are an editor. Polish and improve the content for clarity and impact.")
        .model(llm3)
        .build()?;
    println!(
        "✅ 编辑 Agent '{}' 创建成功",
        editor.name().unwrap_or("unknown")
    );

    // 步骤 4: 顺序执行 Agent 工作流
    println!("\n📝 步骤 4: 执行多 Agent 协作工作流");
    println!("{}", "━".repeat(50));

    let topic = "The future of AI in healthcare";
    println!("\n📌 主题: {}", topic);

    // 第 1 阶段：研究
    println!("\n🔍 阶段 1: 研究阶段");
    let research_result = researcher
        .generate_simple(&format!("Research and provide key facts about: {}", topic))
        .await?;
    println!("研究结果:\n{}\n", research_result);

    // 第 2 阶段：写作
    println!("✍️  阶段 2: 写作阶段");
    let writing_prompt = format!(
        "Based on this research:\n{}\n\nWrite an engaging article about: {}",
        research_result, topic
    );
    let draft = writer.generate_simple(&writing_prompt).await?;
    println!("初稿:\n{}\n", draft);

    // 第 3 阶段：编辑
    println!("📝 阶段 3: 编辑阶段");
    let editing_prompt = format!("Polish and improve this article:\n{}", draft);
    let final_article = editor.generate_simple(&editing_prompt).await?;
    println!("最终版本:\n{}\n", final_article);

    println!("{}", "=".repeat(50));
    println!("✅ 多 Agent 协作完成！");
    println!("\n💡 关键概念:");
    println!("   - 创建多个专门化的 Agent（研究、写作、编辑）");
    println!("   - 将任务分解为多个阶段");
    println!("   - 每个 Agent 的输出作为下一个 Agent 的输入");
    println!("   - 通过协作完成复杂任务");
    println!("\n🔄 工作流:");
    println!("   研究 Agent → 写作 Agent → 编辑 Agent → 最终输出");
    println!("\n下一步: 查看 mvp_04_agent_with_memory.rs 学习如何添加记忆");

    Ok(())
}
