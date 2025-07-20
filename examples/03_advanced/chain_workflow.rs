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
use lumosai_core::agent::AgentTrait; // 正确的Agent trait导入
use std::sync::Arc;
use anyhow::Result;
use tracing::info;

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

    // 创建不同的Agent (使用Arc来共享)
    let research_agent = Arc::new(quick_agent("researcher", "专业研究员")
        .model(llm.clone())
        .tools(vec![web_search(), file_reader()])
        .build()?);

    let analysis_agent = Arc::new(quick_agent("analyst", "数据分析师")
        .model(llm.clone())
        .tools(vec![calculator(), statistics()])
        .build()?);

    let report_agent = Arc::new(quick_agent("reporter", "报告撰写专家")
        .model(llm.clone())
        .tools(vec![file_writer()])
        .build()?);

    let review_agent = Arc::new(quick_agent("reviewer", "质量审核员")
        .model(llm.clone())
        .build()?);

    let publish_agent = Arc::new(quick_agent("publisher", "发布专员")
        .model(llm.clone())
        .build()?);

    let archive_agent = Arc::new(quick_agent("archiver", "归档管理员")
        .model(llm.clone())
        .build()?);

    // 1. 基础链式工作流 - 类似Mastra的.then()语法
    println!("\n1️⃣ 基础链式工作流");
    println!("------------------");

    // 手动链式执行演示
    info!("✅ 开始基础链式工作流");

    let topic = "AI在教育领域的应用";

    // Step 1: Research
    println!("🔍 步骤1: 研究阶段");
    let research_result = research_agent.generate_simple(&format!("请研究: {}", topic)).await?;
    println!("研究结果: {}", research_result);

    // Step 2: Analysis
    println!("📊 步骤2: 分析阶段");
    let analysis_result = analysis_agent.generate_simple(&format!("请分析以下研究结果: {}", research_result)).await?;
    println!("分析结果: {}", analysis_result);

    // Step 3: Report
    println!("📝 步骤3: 报告阶段");
    let report_result = report_agent.generate_simple(&format!("请基于以下分析撰写报告: {}", analysis_result)).await?;
    println!("报告结果: {}", report_result);

    // Step 4: Review
    println!("✅ 步骤4: 审核阶段");
    let review_result = review_agent.generate_simple(&format!("请审核以下报告: {}", report_result)).await?;
    println!("审核结果: {}", review_result);

    info!("✅ 基础链式工作流执行成功");

    // 2. 条件分支工作流演示
    println!("\n2️⃣ 条件分支工作流");
    println!("------------------");

    let topic2 = "机器学习最新进展";
    let quality_score = 0.9; // 模拟质量分数

    // Step 1: Research
    println!("🔍 步骤1: 研究阶段");
    let research_result2 = research_agent.generate_simple(&format!("请研究: {}", topic2)).await?;
    println!("研究结果: {}", research_result2);

    // Step 2: Analysis
    println!("📊 步骤2: 分析阶段");
    let analysis_result2 = analysis_agent.generate_simple(&format!("请分析以下研究结果: {}", research_result2)).await?;
    println!("分析结果: {}", analysis_result2);

    // Step 3: 条件分支
    println!("🔀 步骤3: 条件分支 (质量分数: {})", quality_score);
    if quality_score > 0.8 {
        println!("✅ 质量高，直接发布");
        let publish_result = publish_agent.generate_simple(&format!("请发布以下内容: {}", analysis_result2)).await?;
        println!("发布结果: {}", publish_result);
    } else {
        println!("⚠️ 质量需要改进，进入审核");
        let review_result2 = review_agent.generate_simple(&format!("请审核以下内容: {}", analysis_result2)).await?;
        println!("审核结果: {}", review_result2);
    }

    info!("✅ 条件分支工作流执行成功");

    // 3. 并行处理工作流演示
    println!("\n3️⃣ 并行处理工作流");
    println!("------------------");

    let topic3 = "分布式AI系统";

    // Step 1: Research
    println!("🔍 步骤1: 研究阶段");
    let research_result3 = research_agent.generate_simple(&format!("请研究: {}", topic3)).await?;
    println!("研究结果: {}", research_result3);

    // Step 2: 并行分析
    println!("🔄 步骤2: 并行分析阶段");
    let tasks = vec!["性能分析", "安全分析", "成本分析"];

    // 使用tokio::join!进行并行执行
    let prompt_a = format!("请进行{}，基于: {}", tasks[0], research_result3);
    let prompt_b = format!("请进行{}，基于: {}", tasks[1], research_result3);
    let prompt_c = format!("请进行{}，基于: {}", tasks[2], research_result3);
    let (analysis_a, analysis_b, analysis_c) = tokio::join!(
        analysis_agent.generate_simple(&prompt_a),
        analysis_agent.generate_simple(&prompt_b),
        analysis_agent.generate_simple(&prompt_c)
    );

    let analysis_a = analysis_a?;
    let analysis_b = analysis_b?;
    let analysis_c = analysis_c?;

    println!("并行分析结果A: {}", analysis_a);
    println!("并行分析结果B: {}", analysis_b);
    println!("并行分析结果C: {}", analysis_c);

    // Step 3: 合并结果
    println!("🔗 步骤3: 合并结果");
    let combined_analysis = format!("{}\n{}\n{}", analysis_a, analysis_b, analysis_c);
    let merge_result = report_agent.generate_simple(&format!("请合并以下分析结果: {}", combined_analysis)).await?;
    println!("合并结果: {}", merge_result);

    info!("✅ 并行处理工作流执行成功");

    // 4. 复杂混合工作流演示 - 组合多种模式
    println!("\n4️⃣ 复杂混合工作流");
    println!("------------------");

    let topic4 = "下一代AI架构设计";
    let confidence_threshold = 0.7;

    // Step 1: 初始研究
    println!("🔍 步骤1: 初始研究");
    let initial_research = research_agent.generate_simple(&format!("请深入研究: {}", topic4)).await?;
    println!("初始研究结果: {}", initial_research);

    // Step 2: 并行深度和快速分析
    println!("🔄 步骤2: 并行深度和快速分析");
    let deep_prompt = format!("请进行深度分析，基于: {}", initial_research);
    let quick_prompt = format!("请进行快速分析，基于: {}", initial_research);
    let (deep_analysis, quick_analysis) = tokio::join!(
        analysis_agent.generate_simple(&deep_prompt),
        analysis_agent.generate_simple(&quick_prompt)
    );

    let deep_analysis = deep_analysis?;
    let quick_analysis = quick_analysis?;

    println!("深度分析结果: {}", deep_analysis);
    println!("快速分析结果: {}", quick_analysis);

    // Step 3: 综合分析
    println!("🔗 步骤3: 综合分析");
    let combined = format!("深度分析: {}\n快速分析: {}", deep_analysis, quick_analysis);
    let synthesis = analysis_agent.generate_simple(&format!("请综合以下分析: {}", combined)).await?;
    println!("综合分析结果: {}", synthesis);

    // Step 4: 条件分支 (模拟置信度检查)
    let confidence = 0.85; // 模拟置信度
    println!("🔀 步骤4: 条件分支 (置信度: {})", confidence);

    if confidence > confidence_threshold {
        println!("✅ 置信度高，直接发布");
        let publish_result = publish_agent.generate_simple(&format!("请发布以下内容: {}", synthesis)).await?;
        println!("发布结果: {}", publish_result);
    } else {
        println!("⚠️ 置信度低，需要额外审核");
        let additional_review = review_agent.generate_simple(&format!("请额外审核: {}", synthesis)).await?;
        println!("额外审核结果: {}", additional_review);
    }

    // Step 5: 最终归档
    println!("📁 步骤5: 最终归档");
    let archive_result = archive_agent.generate_simple(&format!("请归档以下内容: {}", synthesis)).await?;
    println!("归档结果: {}", archive_result);

    info!("✅ 复杂混合工作流执行成功");

    // 5. 性能对比测试
    println!("\n5️⃣ 性能对比测试");
    println!("------------------");

    let start_time = std::time::Instant::now();

    // 简单的性能测试 - 连续执行多个Agent调用
    for i in 0..3 {
        println!("🔄 性能测试 {}/3", i + 1);
        let test_result = research_agent.generate_simple(&format!("测试查询 {}", i + 1)).await?;
        println!("测试结果 {}: {}", i + 1, test_result);
    }

    let elapsed_time = start_time.elapsed();
    info!("⏱️ 性能测试耗时: {:?}", elapsed_time);
    info!("📊 平均每次调用时间: {:?}", elapsed_time / 3);

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
    async fn test_agent_chain_execution() {
        let llm = Arc::new(MockLlmProvider::new(vec![
            "研究结果".to_string(),
            "分析结果".to_string(),
        ]));

        let agent1 = Arc::new(quick_agent("agent1", "Test agent 1")
            .model(llm.clone())
            .build()
            .expect("Agent创建失败"));

        let agent2 = Arc::new(quick_agent("agent2", "Test agent 2")
            .model(llm.clone())
            .build()
            .expect("Agent创建失败"));

        // 测试链式执行
        let result1 = agent1.generate_simple("测试输入").await.expect("Agent1执行失败");
        let result2 = agent2.generate_simple(&format!("基于: {}", result1)).await.expect("Agent2执行失败");

        assert!(!result1.is_empty());
        assert!(!result2.is_empty());
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let llm = Arc::new(MockLlmProvider::new(vec![
            "并行任务1结果".to_string(),
            "并行任务2结果".to_string(),
            "并行任务3结果".to_string(),
        ]));

        let agent = Arc::new(quick_agent("agent", "Test agent")
            .model(llm)
            .build()
            .expect("Agent创建失败"));

        // 测试并行执行
        let (result1, result2, result3) = tokio::join!(
            agent.generate_simple("任务1"),
            agent.generate_simple("任务2"),
            agent.generate_simple("任务3")
        );

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
    }
}
