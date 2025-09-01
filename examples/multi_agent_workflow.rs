//! 多代理工作流演示
//!
//! 展示如何创建和执行复杂的多代理工作流，包括：
//! - 专业化代理创建
//! - 工作流编排和执行
//! - 代理间协作
//! - 条件执行和错误处理

use lumosai_core::agent::{AgentBuilder, trait_def::Agent, AgentTrait};
use lumosai_core::llm::MockLlmProvider;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🔄 多代理工作流演示");
    println!("====================");

    // 演示1: 创建专业化代理
    demo_specialized_agents().await?;

    // 演示2: 简单工作流
    demo_simple_workflow().await?;

    // 演示3: 复杂工作流编排
    demo_complex_workflow().await?;

    // 演示4: 条件执行和错误处理
    demo_conditional_workflow().await?;

    // 演示完成
    demo_complete();

    Ok(())
}

/// 演示专业化代理创建
async fn demo_specialized_agents() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示1: 专业化代理创建 ===");

    // 创建研究员代理
    let researcher = create_researcher_agent().await?;
    println!("✅ 创建研究员代理: {}", researcher.get_name());
    println!("   专长: 技术研究和信息收集");

    // 创建写作代理
    let writer = create_writer_agent().await?;
    println!("✅ 创建写作代理: {}", writer.get_name());
    println!("   专长: 技术文档撰写");

    // 创建审查代理
    let reviewer = create_reviewer_agent().await?;
    println!("✅ 创建审查代理: {}", reviewer.get_name());
    println!("   专长: 内容质量审查");

    // 创建发布代理
    let publisher = create_publisher_agent().await?;
    println!("✅ 创建发布代理: {}", publisher.get_name());
    println!("   专长: 内容格式化和发布");

    // 测试各个代理的独立功能
    println!("\n=== 代理功能测试 ===");

    // 测试研究员
    let research_result = researcher.generate_simple("请研究 Rust 异步编程的最新发展").await?;
    println!("\n研究员输出:");
    println!("{}", research_result);

    // 测试写作代理
    let writing_result = writer.generate_simple("基于研究结果，撰写一篇关于 Rust 异步编程的技术文章大纲").await?;
    println!("\n写作代理输出:");
    println!("{}", writing_result);

    Ok(())
}

/// 演示简单工作流
async fn demo_simple_workflow() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示2: 简单工作流 ===");

    // 创建代理
    let researcher = create_researcher_agent().await?;
    let writer = create_writer_agent().await?;
    let reviewer = create_reviewer_agent().await?;

    println!("创建简单工作流:");
    println!("  名称: simple_content_creation");
    println!("  描述: 简单的内容创建工作流");
    println!("  步骤数: 3");

    // 手动执行工作流步骤（简化演示）
    println!("\n执行工作流...");
    let topic = "Rust 异步编程最佳实践";

    // 步骤1: 研究
    println!("\n步骤1: 技术研究");
    let research_prompt = format!("深入研究指定主题：{}，收集相关信息和最新发展", topic);
    let research_result = researcher.generate_simple(&research_prompt).await?;
    println!("  状态: ✅ 成功");
    println!("  输出: {}", research_result);

    // 步骤2: 写作
    println!("\n步骤2: 内容撰写");
    let writing_prompt = format!("基于以下研究结果撰写技术文章：\n{}", research_result);
    let writing_result = writer.generate_simple(&writing_prompt).await?;
    println!("  状态: ✅ 成功");
    println!("  输出: {}", writing_result);

    // 步骤3: 审查
    println!("\n步骤3: 内容审查");
    let review_prompt = format!("审查以下文章的技术准确性和可读性：\n{}", writing_result);
    let review_result = reviewer.generate_simple(&review_prompt).await?;
    println!("  状态: ✅ 成功");
    println!("  输出: {}", review_result);

    println!("\n工作流执行结果:");
    println!("  状态: ✅ 成功完成");
    println!("  完成步骤: 3/3");

    Ok(())
}

/// 演示复杂工作流编排
async fn demo_complex_workflow() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示3: 复杂工作流编排 ===");

    // 创建所有需要的代理
    let researcher = create_researcher_agent().await?;
    let writer = create_writer_agent().await?;
    let reviewer = create_reviewer_agent().await?;
    let publisher = create_publisher_agent().await?;

    println!("创建复杂工作流:");
    println!("  名称: advanced_content_pipeline");
    println!("  总步骤数: 7");
    println!("  并行步骤: outline + examples");
    println!("  条件依赖: writing 依赖 outline 和 examples");

    // 手动执行复杂工作流（简化演示）
    println!("\n执行复杂工作流...");
    let topic = "Rust 性能优化技巧";

    // 步骤1: 深度研究
    println!("\n步骤1: 深度研究");
    let research_prompt = format!("进行深度技术研究，收集关于{}的最新信息", topic);
    let research_result = researcher.generate_simple(&research_prompt).await?;
    println!("  状态: ✅ 成功");

    // 步骤2 & 3: 并行执行 - 大纲创建和示例收集
    println!("\n步骤2 & 3: 并行执行");

    // 大纲创建
    println!("  步骤2: 创建大纲");
    let outline_prompt = format!("基于研究结果创建详细的文章大纲：\n{}", research_result);
    let outline_result = writer.generate_simple(&outline_prompt).await?;
    println!("    状态: ✅ 成功");

    // 示例收集
    println!("  步骤3: 收集示例");
    let examples_prompt = format!("收集相关的代码示例和案例研究：\n{}", research_result);
    let examples_result = researcher.generate_simple(&examples_prompt).await?;
    println!("    状态: ✅ 成功");

    // 步骤4: 内容撰写（依赖大纲和示例）
    println!("\n步骤4: 内容撰写");
    let writing_prompt = format!("基于大纲和示例撰写完整文章：\n大纲：{}\n示例：{}",
        outline_result, examples_result);
    let writing_result = writer.generate_simple(&writing_prompt).await?;
    println!("  状态: ✅ 成功");

    // 步骤5: 技术审查
    println!("\n步骤5: 技术审查");
    let tech_review_prompt = format!("审查技术内容的准确性：\n{}", writing_result);
    let tech_review_result = reviewer.generate_simple(&tech_review_prompt).await?;
    println!("  状态: ✅ 成功");

    // 步骤6: 编辑审查
    println!("\n步骤6: 编辑审查");
    let editorial_review_prompt = format!("审查文章的语言和结构：\n{}", tech_review_result);
    let editorial_review_result = reviewer.generate_simple(&editorial_review_prompt).await?;
    println!("  状态: ✅ 成功");

    // 步骤7: 发布准备
    println!("\n步骤7: 发布准备");
    let publish_prompt = format!("格式化文章并准备发布：\n{}", editorial_review_result);
    let _publish_result = publisher.generate_simple(&publish_prompt).await?;
    println!("  状态: ✅ 成功");

    println!("\n复杂工作流执行结果:");
    println!("  最终状态: ✅ 成功完成");
    println!("  成功步骤: 7/7");
    println!("  执行路径: research -> (outline + examples) -> writing -> tech_review -> editorial_review -> publish");

    Ok(())
}

/// 演示条件执行和错误处理
async fn demo_conditional_workflow() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示4: 条件执行和错误处理 ===");

    // 创建代理
    let unreliable_agent = create_unreliable_agent().await?;
    let fallback_agent = create_writer_agent().await?;
    let validator = create_reviewer_agent().await?;

    println!("创建错误处理工作流:");
    println!("  主要路径: primary_task -> success_validation -> final_processing");
    println!("  备用路径: primary_task (失败) -> fallback_task -> final_processing");

    // 场景1: 模拟主要任务成功
    println!("\n--- 场景1: 主要任务成功 ---");

    // 主要任务（成功）
    println!("步骤1: 主要任务");
    let primary_result = unreliable_agent.generate_simple("这是一个成功的任务").await?;
    println!("  状态: ✅ 成功");
    println!("  输出: {}", primary_result);

    // 成功验证
    println!("步骤2: 成功验证");
    let validation_prompt = format!("验证主要任务的结果：{}", primary_result);
    let validation_result = validator.generate_simple(&validation_prompt).await?;
    println!("  状态: ✅ 成功");
    println!("  输出: {}", validation_result);

    // 最终处理
    println!("步骤3: 最终处理");
    let final_prompt = format!("执行最终处理和清理工作：{}", validation_result);
    let final_result = fallback_agent.generate_simple(&final_prompt).await?;
    println!("  状态: ✅ 成功");
    println!("  输出: {}", final_result);

    println!("场景1执行摘要:");
    println!("  状态: ✅ 成功完成");
    println!("  执行路径: primary_task -> success_validation -> final_processing");

    // 场景2: 模拟主要任务失败
    println!("\n--- 场景2: 主要任务失败 ---");

    // 主要任务（模拟失败）
    println!("步骤1: 主要任务");
    println!("  状态: ❌ 失败");
    println!("  错误: 模拟的任务失败");

    // 备用任务
    println!("步骤2: 备用任务");
    let fallback_result = fallback_agent.generate_simple("执行备用任务").await?;
    println!("  状态: ✅ 成功");
    println!("  输出: {}", fallback_result);

    // 最终处理
    println!("步骤3: 最终处理");
    let final_prompt2 = format!("执行最终处理和清理工作：{}", fallback_result);
    let final_result2 = fallback_agent.generate_simple(&final_prompt2).await?;
    println!("  状态: ✅ 成功");
    println!("  输出: {}", final_result2);

    println!("场景2执行摘要:");
    println!("  状态: ✅ 成功完成（通过备用路径）");
    println!("  执行路径: primary_task (失败) -> fallback_task -> final_processing");

    Ok(())
}

// ============================================================================
// 代理创建函数
// ============================================================================

/// 创建研究员代理
async fn create_researcher_agent() -> std::result::Result<Arc<dyn AgentTrait>, Box<dyn std::error::Error>> {
    let responses = vec![
        "我已经完成了深入的技术研究。Rust 异步编程的最新发展包括：async/await 语法的稳定、Tokio 生态系统的成熟、以及新的异步特性如 async closures 的提案。".to_string(),
        "研究完成。我收集了相关的代码示例和最佳实践，包括错误处理模式、性能优化技巧和常见陷阱的避免方法。".to_string(),
    ];

    let llm_provider = Arc::new(MockLlmProvider::new(responses));

    Ok(Arc::new(
        AgentBuilder::new()
            .name("researcher")
            .instructions("你是一个专业的技术研究员，擅长收集和分析最新的技术信息。请提供准确、详细的研究结果。")
            .model(llm_provider)
            .build()?
    ))
}

/// 创建写作代理
async fn create_writer_agent() -> std::result::Result<Arc<dyn AgentTrait>, Box<dyn std::error::Error>> {
    let responses = vec![
        "我已经创建了详细的文章大纲：1. 引言 2. 异步编程基础 3. 高级特性 4. 最佳实践 5. 性能优化 6. 总结。每个部分都包含了具体的要点和示例。".to_string(),
        "文章撰写完成。我基于研究结果和大纲，撰写了一篇全面的技术文章，涵盖了 Rust 异步编程的核心概念、实用技巧和最佳实践。".to_string(),
        "备用任务已完成。我提供了一个简化但完整的解决方案，确保即使在主要流程失败的情况下也能产出有价值的内容。".to_string(),
        "最终处理完成。我已经整理了所有输出，确保格式一致，并添加了必要的元数据和总结信息。".to_string(),
    ];

    let llm_provider = Arc::new(MockLlmProvider::new(responses));

    Ok(Arc::new(
        AgentBuilder::new()
            .name("writer")
            .instructions("你是一个技术写作专家，能够将复杂的技术概念转化为清晰易懂的文章。请确保内容结构清晰、逻辑严密。")
            .model(llm_provider)
            .build()?
    ))
}

/// 创建审查代理
async fn create_reviewer_agent() -> std::result::Result<Arc<dyn AgentTrait>, Box<dyn std::error::Error>> {
    let responses = vec![
        "技术审查完成。文章的技术内容准确，代码示例正确，概念解释清晰。建议在性能部分添加更多的基准测试数据。".to_string(),
        "编辑审查完成。文章结构良好，语言流畅，逻辑清晰。已修正了几处语法错误和术语不一致的问题。".to_string(),
        "验证完成。主要任务的输出质量符合预期标准，技术准确性和可读性都达到了要求。".to_string(),
    ];

    let llm_provider = Arc::new(MockLlmProvider::new(responses));

    Ok(Arc::new(
        AgentBuilder::new()
            .name("reviewer")
            .instructions("你是一个严格的技术审查员，专注于确保内容的准确性、完整性和质量。请提供详细的审查意见。")
            .model(llm_provider)
            .build()?
    ))
}

/// 创建发布代理
async fn create_publisher_agent() -> std::result::Result<Arc<dyn AgentTrait>, Box<dyn std::error::Error>> {
    let responses = vec![
        "发布准备完成。文章已格式化为适合博客发布的格式，添加了适当的标题层级、代码高亮和元数据。SEO 优化也已完成。".to_string(),
    ];

    let llm_provider = Arc::new(MockLlmProvider::new(responses));

    Ok(Arc::new(
        AgentBuilder::new()
            .name("publisher")
            .instructions("你负责最终发布内容，包括格式化、SEO优化和平台适配。请确保内容符合发布标准。")
            .model(llm_provider)
            .build()?
    ))
}

/// 创建不可靠代理（用于演示错误处理）
async fn create_unreliable_agent() -> std::result::Result<Arc<dyn AgentTrait>, Box<dyn std::error::Error>> {
    let responses = vec![
        "主要任务成功完成。这是一个高质量的输出结果。".to_string(),
        // 注意：这个代理会根据输入决定是否"失败"
    ];

    let llm_provider = Arc::new(MockLlmProvider::new(responses));

    Ok(Arc::new(
        AgentBuilder::new()
            .name("unreliable_agent")
            .instructions("你是一个可能失败的代理，用于演示错误处理。根据输入的 task_type 决定成功或失败。")
            .model(llm_provider)
            .build()?
    ))
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 演示完成
fn demo_complete() {
    println!("\n🎉 多代理工作流演示完成！");
    println!("演示内容包括：");
    println!("  ✅ 专业化代理创建");
    println!("  ✅ 简单线性工作流");
    println!("  ✅ 复杂并行工作流");
    println!("  ✅ 条件执行和错误处理");
}