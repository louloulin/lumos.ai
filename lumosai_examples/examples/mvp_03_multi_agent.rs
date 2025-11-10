//! MVP 示例 3: 多 Agent 协作
//!
//! 这个示例展示如何使用多个 Agent 协作完成复杂任务。
//!
//! 运行方式:
//! ```bash
//! cargo run --example mvp_03_multi_agent
//! ```

use lumosai_core::agent::{create_basic_agent, pipe, Agent, AgentPipeline};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use lumosai_core::llm::LlmProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MVP 示例 3: 多 Agent 协作\n");
    println!("=".repeat(50));
    
    // 步骤 1: 创建 LLM 提供商
    println!("\n📝 步骤 1: 创建 LLM 提供商");
    let llm: Arc<dyn LlmProvider> = create_test_zhipu_provider_arc();
    println!("✅ LLM 提供商创建成功");
    
    // 步骤 2: 创建多个专业 Agent
    println!("\n📝 步骤 2: 创建专业 Agent 团队");
    
    // 研究员 Agent
    let researcher = create_basic_agent(
        "researcher",
        "You are a researcher. Your job is to gather information and facts about the topic. \
         Provide detailed, factual information.",
        llm.clone(),
    );
    println!("✅ 创建 Researcher Agent");
    
    // 作家 Agent
    let writer = create_basic_agent(
        "writer",
        "You are a creative writer. Your job is to take research information and create \
         engaging, well-structured content. Be creative but accurate.",
        llm.clone(),
    );
    println!("✅ 创建 Writer Agent");
    
    // 编辑 Agent
    let editor = create_basic_agent(
        "editor",
        "You are an editor. Your job is to review and polish content. \
         Fix grammar, improve clarity, and ensure quality.",
        llm.clone(),
    );
    println!("✅ 创建 Editor Agent");
    
    // 步骤 3: 创建 Agent 流水线
    println!("\n📝 步骤 3: 创建 Agent 流水线");
    let pipeline: AgentPipeline = pipe![researcher, writer, editor];
    println!("✅ 流水线创建成功: Researcher → Writer → Editor");
    
    // 步骤 4: 执行流水线
    println!("\n📝 步骤 4: 执行流水线");
    println!("━".repeat(50));
    
    let topic = "The benefits of Rust programming language";
    println!("\n📌 任务: 创建关于 '{}' 的文章", topic);
    
    println!("\n🔄 开始执行流水线...\n");
    
    match pipeline.execute(topic).await {
        Ok(final_result) => {
            println!("━".repeat(50));
            println!("\n✅ 流水线执行完成！");
            println!("\n📄 最终结果:");
            println!("{}", "─".repeat(50));
            println!("{}", final_result);
            println!("{}", "─".repeat(50));
        }
        Err(e) => {
            eprintln!("❌ 流水线执行失败: {}", e);
        }
    }
    
    // 步骤 5: 演示并行执行
    println!("\n📝 步骤 5: 演示并行 Agent 执行");
    println!("━".repeat(50));
    
    // 创建多个分析 Agent
    let analyst1 = create_basic_agent(
        "analyst1",
        "You are a technical analyst. Focus on technical aspects.",
        llm.clone(),
    );
    
    let analyst2 = create_basic_agent(
        "analyst2",
        "You are a business analyst. Focus on business value.",
        llm.clone(),
    );
    
    let analyst3 = create_basic_agent(
        "analyst3",
        "You are a user experience analyst. Focus on developer experience.",
        llm,
    );
    
    println!("✅ 创建了 3 个分析 Agent");
    println!("\n🔄 并行执行分析任务...\n");
    
    let analysis_topic = "Analyze Rust for enterprise applications";
    
    // 使用 tokio::join! 并行执行
    let (result1, result2, result3) = tokio::join!(
        analyst1.generate_simple(analysis_topic),
        analyst2.generate_simple(analysis_topic),
        analyst3.generate_simple(analysis_topic),
    );
    
    println!("━".repeat(50));
    println!("\n✅ 并行分析完成！\n");
    
    if let Ok(r1) = result1 {
        println!("🔧 技术分析:");
        println!("{}", "─".repeat(50));
        println!("{}\n", r1);
    }
    
    if let Ok(r2) = result2 {
        println!("💼 商业分析:");
        println!("{}", "─".repeat(50));
        println!("{}\n", r2);
    }
    
    if let Ok(r3) = result3 {
        println!("👥 用户体验分析:");
        println!("{}", "─".repeat(50));
        println!("{}\n", r3);
    }
    
    println!("=".repeat(50));
    println!("✅ 示例完成！");
    println!("\n💡 提示:");
    println!("   - 使用 pipe! 宏创建顺序流水线");
    println!("   - 使用 tokio::join! 实现并行执行");
    println!("   - 不同的 Agent 可以有不同的专业领域");
    println!("   - 下一步: 查看 mvp_04_agent_with_memory.rs 学习记忆功能");
    
    Ok(())
}

