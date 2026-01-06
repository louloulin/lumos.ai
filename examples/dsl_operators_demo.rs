//! DSL 操作符演示
//!
//! 演示如何使用 Agent 操作符进行协作：
//! - `pipe_to()` - 管道操作符（类似 `|>`）
//! - `parallel_with()` - 并行操作符（类似 `|`）
//! - `delegate_to()` - 委托操作符（类似 `<=`）

use lumosai_core::agent::{create_basic_agent, delegate, Agent, AgentParallel, AgentPipeline};
use lumosai_core::error::Result;
use lumosai_core::llm::test_helpers::{create_test_zhipu_provider, create_test_zhipu_provider_arc};
use lumosai_core::llm::LlmProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 DSL 操作符演示\n");
    println!("注意：此示例使用 Mock LLM 提供者进行演示\n");

    // 创建 Mock LLM 提供者（用于演示，不需要 API 密钥）
    let llm_provider: Arc<dyn LlmProvider> = create_test_zhipu_provider_arc();

    // 创建三个 Agent（使用 Arc<dyn Agent> 类型）
    let researcher: Arc<dyn Agent> = Arc::new(create_basic_agent(
        "researcher".to_string(),
        "You are a research assistant. Gather information on the given topic.".to_string(),
        Arc::clone(&llm_provider),
    )?);

    let analyzer: Arc<dyn Agent> = Arc::new(create_basic_agent(
        "analyzer".to_string(),
        "You are an analyst. Analyze the information provided and extract key insights."
            .to_string(),
        Arc::clone(&llm_provider),
    )?);

    let writer: Arc<dyn Agent> = Arc::new(create_basic_agent(
        "writer".to_string(),
        "You are a technical writer. Create a concise summary from the analysis.".to_string(),
        Arc::clone(&llm_provider),
    )?);

    // ========================================
    // 1. 管道操作符演示（类似 researcher |> analyzer |> writer）
    // ========================================
    println!("📊 1. 管道操作符演示（Pipeline）");
    println!("   researcher |> analyzer |> writer\n");

    let pipeline = AgentPipeline::new(Arc::clone(&researcher))
        .pipe(Arc::clone(&analyzer))
        .pipe(Arc::clone(&writer));

    println!("   执行管道链...");
    let result = pipeline.execute("AI trends in 2024").await?;
    println!("   ✅ 管道结果: {}\n", result);

    // ========================================
    // 2. 并行操作符演示（类似 agent1 | agent2 | agent3）
    // ========================================
    println!("📊 2. 并行操作符演示（Parallel）");
    println!("   researcher | analyzer | writer\n");

    let parallel = AgentParallel::new(Arc::clone(&researcher))
        .parallel(Arc::clone(&analyzer))
        .parallel(Arc::clone(&writer));

    println!("   并行执行所有 Agent...");
    let results = parallel.execute("Quantum computing").await?;
    println!("   ✅ 并行结果:");
    for (i, result) in results.iter().enumerate() {
        println!("      Agent {}: {}", i + 1, result);
    }
    println!();

    // ========================================
    // 3. 委托操作符演示（类似 manager <= worker）
    // ========================================
    println!("📊 3. 委托操作符演示（Delegation）");
    println!("   analyzer <= researcher\n");

    let delegation = delegate(
        Arc::clone(&analyzer),
        Arc::clone(&researcher),
        "Research and analyze blockchain technology",
    );

    println!("   执行委托操作...");
    let result = delegation.execute("Focus on scalability").await?;
    println!("   ✅ 委托结果: {}\n", result);

    // ========================================
    // 4. 使用 AgentPipeline 的链式调用
    // ========================================
    println!("📊 4. 链式调用演示");
    println!("   AgentPipeline::new(researcher).pipe(analyzer).pipe(writer)\n");

    let pipeline2 = AgentPipeline::new(Arc::clone(&researcher))
        .pipe(Arc::clone(&analyzer))
        .pipe(Arc::clone(&writer));

    println!("   执行链式管道...");
    let result = pipeline2.execute("Machine learning advancements").await?;
    println!("   ✅ 链式结果: {}\n", result);

    println!("✅ 所有操作符演示完成！");

    Ok(())
}
