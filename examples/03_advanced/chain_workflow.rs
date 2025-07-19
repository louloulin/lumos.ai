//! 链式工作流示例 - 展示Mastra风格的链式工作流语法
//! 
//! 这个示例展示了如何使用链式语法创建工作流，类似于Mastra的API设计。
//! 
//! 运行方式:
//! ```bash
//! cargo run --example chain_workflow
//! ```

use lumosai_core::prelude::*;
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::workflow::WorkflowBuilder;
use std::sync::Arc;
use anyhow::Result;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🔗 LumosAI 链式工作流示例");
    println!("==============================");

    // 创建LLM提供者
    let llm = Arc::new(MockLlmProvider::new(vec![
        "我已经完成了研究工作。".to_string(),
        "我已经完成了分析任务。".to_string(),
        "我已经生成了报告。".to_string(),
        "我已经完成了审核。".to_string(),
        "我已经发布了结果。".to_string(),
        "我已经完成了归档。".to_string(),
    ]));

    // 创建不同的Agent
    let research_agent = quick_agent("researcher", "专业研究员")
        .model(llm.clone())
        .tools(vec![web_search(), file_reader()])
        .build()?;

    let analysis_agent = quick_agent("analyst", "数据分析师")
        .model(llm.clone())
        .tools(vec![calculator(), statistics()])
        .build()?;

    let report_agent = quick_agent("reporter", "报告撰写专家")
        .model(llm.clone())
        .tools(vec![file_writer()])
        .build()?;

    let review_agent = quick_agent("reviewer", "质量审核员")
        .model(llm.clone())
        .build()?;

    let publish_agent = quick_agent("publisher", "发布专员")
        .model(llm.clone())
        .build()?;

    let archive_agent = quick_agent("archiver", "归档管理员")
        .model(llm.clone())
        .build()?;

    // 1. 基础链式工作流 - 类似Mastra的.then()语法
    println!("\n1️⃣ 基础链式工作流");
    println!("------------------");

    let basic_chain = WorkflowBuilder::new()
        .id("basic_chain")
        .name("Basic Chain Workflow")
        .description("基础的链式工作流")
        .step("research", research_agent.clone())
        .then("analysis", analysis_agent.clone())
        .then("report", report_agent.clone())
        .then("review", review_agent.clone())
        .build()?;

    info!("✅ 基础链式工作流创建成功");
    info!("📊 工作流信息:");
    info!("   - ID: {}", basic_chain.get_id());
    info!("   - 步骤数量: {}", basic_chain.get_steps().len());

    // 执行基础链式工作流
    let input_data = serde_json::json!({
        "topic": "AI在教育领域的应用",
        "deadline": "2024-12-31"
    });

    match basic_chain.execute(input_data).await {
        Ok(result) => {
            info!("✅ 基础链式工作流执行成功");
            println!("📄 执行结果: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            error!("❌ 基础链式工作流执行失败: {}", e);
        }
    }

    // 2. 条件分支工作流 - 类似Mastra的.branch()语法
    println!("\n2️⃣ 条件分支工作流");
    println!("------------------");

    let branch_workflow = WorkflowBuilder::new()
        .id("branch_workflow")
        .name("Conditional Branch Workflow")
        .description("包含条件分支的工作流")
        .step("research", research_agent.clone())
        .then("analysis", analysis_agent.clone())
        .branch(
            |result| {
                // 模拟质量检查条件
                result.get("quality_score")
                    .and_then(|v| v.as_f64())
                    .map(|score| score > 0.8)
                    .unwrap_or(false)
            },
            "publish",  // 质量好 -> 发布
            "review"    // 质量差 -> 审核
        )
        .build()?;

    info!("✅ 条件分支工作流创建成功");

    // 执行条件分支工作流
    let branch_input = serde_json::json!({
        "topic": "机器学习最新进展",
        "quality_score": 0.9  // 高质量分数，应该走发布分支
    });

    match branch_workflow.execute(branch_input).await {
        Ok(result) => {
            info!("✅ 条件分支工作流执行成功");
            println!("📄 分支执行结果: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            error!("❌ 条件分支工作流执行失败: {}", e);
        }
    }

    // 3. 并行处理工作流 - 类似Mastra的.parallel()语法
    println!("\n3️⃣ 并行处理工作流");
    println!("------------------");

    let parallel_workflow = WorkflowBuilder::new()
        .id("parallel_workflow")
        .name("Parallel Processing Workflow")
        .description("包含并行处理的工作流")
        .step("research", research_agent.clone())
        .parallel(vec!["analysis_a", "analysis_b", "analysis_c"])
        .then("merge", report_agent.clone())
        .then("final_review", review_agent.clone())
        .build()?;

    info!("✅ 并行处理工作流创建成功");

    // 执行并行处理工作流
    let parallel_input = serde_json::json!({
        "topic": "分布式AI系统",
        "parallel_tasks": ["性能分析", "安全分析", "成本分析"]
    });

    match parallel_workflow.execute(parallel_input).await {
        Ok(result) => {
            info!("✅ 并行处理工作流执行成功");
            println!("📄 并行执行结果: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            error!("❌ 并行处理工作流执行失败: {}", e);
        }
    }

    // 4. 复杂混合工作流 - 组合多种模式
    println!("\n4️⃣ 复杂混合工作流");
    println!("------------------");

    let complex_workflow = WorkflowBuilder::new()
        .id("complex_workflow")
        .name("Complex Mixed Workflow")
        .description("组合多种模式的复杂工作流")
        .step("initial_research", research_agent.clone())
        .parallel(vec!["deep_analysis", "quick_analysis"])
        .then("synthesis", analysis_agent.clone())
        .branch(
            |result| {
                result.get("confidence")
                    .and_then(|v| v.as_f64())
                    .map(|conf| conf > 0.7)
                    .unwrap_or(false)
            },
            "direct_publish",
            "additional_review"
        )
        .then("final_archive", archive_agent.clone())
        .build()?;

    info!("✅ 复杂混合工作流创建成功");
    info!("📊 复杂工作流统计:");
    info!("   - 总步骤数: {}", complex_workflow.get_steps().len());
    info!("   - 包含并行处理: ✓");
    info!("   - 包含条件分支: ✓");
    info!("   - 包含链式调用: ✓");

    // 执行复杂混合工作流
    let complex_input = serde_json::json!({
        "topic": "下一代AI架构设计",
        "priority": "high",
        "confidence": 0.85
    });

    match complex_workflow.execute(complex_input).await {
        Ok(result) => {
            info!("✅ 复杂混合工作流执行成功");
            println!("📄 复杂执行结果: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            error!("❌ 复杂混合工作流执行失败: {}", e);
        }
    }

    // 5. 性能对比测试
    println!("\n5️⃣ 性能对比测试");
    println!("------------------");

    let start_time = std::time::Instant::now();

    // 创建多个链式工作流测试性能
    let mut workflows = Vec::new();
    for i in 0..10 {
        let workflow = WorkflowBuilder::new()
            .id(&format!("perf_test_{}", i))
            .name(&format!("Performance Test {}", i))
            .step("step1", research_agent.clone())
            .then("step2", analysis_agent.clone())
            .then("step3", report_agent.clone())
            .build()?;
        workflows.push(workflow);
    }

    let creation_time = start_time.elapsed();
    info!("⏱️ 创建10个链式工作流耗时: {:?}", creation_time);
    info!("📊 平均每个工作流创建时间: {:?}", creation_time / 10);

    println!("\n🎉 链式工作流示例完成!");
    println!("\n📚 下一步学习:");
    println!("   - examples/03_advanced/workflow_visualization.rs - 工作流可视化");
    println!("   - examples/04_production/workflow_monitoring.rs - 工作流监控");
    println!("   - docs/best-practices/workflow-design.md - 工作流设计最佳实践");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chain_workflow_creation() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
        
        let agent1 = quick_agent("agent1", "Test agent 1")
            .model(llm.clone())
            .build()
            .expect("Agent创建失败");
        
        let agent2 = quick_agent("agent2", "Test agent 2")
            .model(llm.clone())
            .build()
            .expect("Agent创建失败");

        let workflow = WorkflowBuilder::new()
            .id("test_chain")
            .name("Test Chain")
            .step("step1", agent1)
            .then("step2", agent2)
            .build();

        assert!(workflow.is_ok());
        let workflow = workflow.unwrap();
        assert_eq!(workflow.get_id(), "test_chain");
        assert_eq!(workflow.get_steps().len(), 2);
    }

    #[tokio::test]
    async fn test_branch_workflow() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
        
        let agent = quick_agent("agent", "Test agent")
            .model(llm)
            .build()
            .expect("Agent创建失败");

        let workflow = WorkflowBuilder::new()
            .id("test_branch")
            .name("Test Branch")
            .step("step1", agent)
            .branch(
                |_| true,  // 总是返回true
                "true_step",
                "false_step"
            )
            .build();

        assert!(workflow.is_ok());
        let workflow = workflow.unwrap();
        assert_eq!(workflow.get_id(), "test_branch");
    }

    #[tokio::test]
    async fn test_parallel_workflow() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
        
        let agent = quick_agent("agent", "Test agent")
            .model(llm)
            .build()
            .expect("Agent创建失败");

        let workflow = WorkflowBuilder::new()
            .id("test_parallel")
            .name("Test Parallel")
            .step("initial", agent)
            .parallel(vec!["task1", "task2", "task3"])
            .build();

        assert!(workflow.is_ok());
        let workflow = workflow.unwrap();
        assert_eq!(workflow.get_id(), "test_parallel");
    }
}
