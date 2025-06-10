//! 多代理工作流演示
//!
//! 展示如何创建和执行复杂的多代理工作流，包括：
//! - 专业化代理创建
//! - 工作流编排和执行
//! - 代理间协作
//! - 条件执行和错误处理

use lumosai_core::prelude::*;
use lumosai_core::agent::{AgentBuilder, BasicAgent};
use lumosai_core::workflow::{WorkflowBuilder, WorkflowStep, StepCondition, WorkflowResult, WorkflowStatus};
use lumosai_core::llm::{MockLlmProvider, Message, Role};
use std::sync::Arc;
use std::collections::HashMap;
use serde_json::{json, Value};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}

/// 演示专业化代理创建
async fn demo_specialized_agents() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示1: 专业化代理创建 ===");

    // 创建研究员代理
    let researcher = create_researcher_agent().await?;
    println!("✅ 创建研究员代理: {}", researcher.name());
    println!("   专长: 技术研究和信息收集");

    // 创建写作代理
    let writer = create_writer_agent().await?;
    println!("✅ 创建写作代理: {}", writer.name());
    println!("   专长: 技术文档撰写");

    // 创建审查代理
    let reviewer = create_reviewer_agent().await?;
    println!("✅ 创建审查代理: {}", reviewer.name());
    println!("   专长: 内容质量审查");

    // 创建发布代理
    let publisher = create_publisher_agent().await?;
    println!("✅ 创建发布代理: {}", publisher.name());
    println!("   专长: 内容格式化和发布");

    // 测试各个代理的独立功能
    println!("\n=== 代理功能测试 ===");

    // 测试研究员
    let research_result = researcher.generate("请研究 Rust 异步编程的最新发展").await?;
    println!("\n研究员输出:");
    println!("{}", research_result.content);

    // 测试写作代理
    let writing_result = writer.generate("基于研究结果，撰写一篇关于 Rust 异步编程的技术文章大纲").await?;
    println!("\n写作代理输出:");
    println!("{}", writing_result.content);

    Ok(())
}

/// 演示简单工作流
async fn demo_simple_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示2: 简单工作流 ===");

    // 创建代理
    let researcher = create_researcher_agent().await?;
    let writer = create_writer_agent().await?;
    let reviewer = create_reviewer_agent().await?;

    // 创建简单的线性工作流
    let workflow = WorkflowBuilder::new()
        .name("simple_content_creation")
        .description("简单的内容创建工作流")

        // 步骤1: 研究
        .add_step(WorkflowStep {
            id: "research".to_string(),
            name: "技术研究".to_string(),
            agent: researcher,
            instructions: "深入研究指定主题，收集相关信息和最新发展".to_string(),
            condition: StepCondition::Always,
            timeout_seconds: Some(30),
            retry_count: 2,
        })

        // 步骤2: 写作
        .add_step(WorkflowStep {
            id: "writing".to_string(),
            name: "内容撰写".to_string(),
            agent: writer,
            instructions: "基于研究结果撰写技术文章".to_string(),
            condition: StepCondition::PreviousSuccess("research".to_string()),
            timeout_seconds: Some(60),
            retry_count: 1,
        })

        // 步骤3: 审查
        .add_step(WorkflowStep {
            id: "review".to_string(),
            name: "内容审查".to_string(),
            agent: reviewer,
            instructions: "审查文章的技术准确性和可读性".to_string(),
            condition: StepCondition::PreviousSuccess("writing".to_string()),
            timeout_seconds: Some(30),
            retry_count: 3,
        })

        .build()?;

    println!("创建简单工作流:");
    println!("  名称: {}", workflow.name());
    println!("  描述: {}", workflow.description());
    println!("  步骤数: {}", workflow.steps().len());

    // 执行工作流
    println!("\n执行工作流...");
    let input_data = json!({
        "topic": "Rust 异步编程最佳实践",
        "target_audience": "中级开发者",
        "word_count": 2000
    });

    let result = workflow.execute(input_data).await?;

    println!("\n工作流执行结果:");
    println!("  状态: {:?}", result.status);
    println!("  执行时间: {:?}", result.execution_time);
    println!("  完成步骤: {}/{}", result.completed_steps, result.total_steps);

    // 显示每个步骤的结果
    for (step_id, step_result) in &result.step_results {
        println!("\n步骤 '{}':", step_id);
        println!("  状态: {:?}", step_result.status);
        println!("  执行时间: {:?}", step_result.execution_time);
        if let Some(output) = &step_result.output {
            println!("  输出: {}", output);
        }
        if let Some(error) = &step_result.error {
            println!("  错误: {}", error);
        }
    }

    Ok(())
}

/// 演示复杂工作流编排
async fn demo_complex_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示3: 复杂工作流编排 ===");

    // 创建所有需要的代理
    let researcher = create_researcher_agent().await?;
    let writer = create_writer_agent().await?;
    let reviewer = create_reviewer_agent().await?;
    let publisher = create_publisher_agent().await?;

    // 创建复杂的工作流，包含并行执行和条件分支
    let complex_workflow = WorkflowBuilder::new()
        .name("advanced_content_pipeline")
        .description("高级内容生产流水线")

        // 步骤1: 研究阶段
        .add_step(WorkflowStep {
            id: "research".to_string(),
            name: "深度研究".to_string(),
            agent: researcher.clone(),
            instructions: "进行深度技术研究，收集最新信息".to_string(),
            condition: StepCondition::Always,
            timeout_seconds: Some(30),
            retry_count: 2,
        })

        // 步骤2: 大纲创建（依赖研究）
        .add_step(WorkflowStep {
            id: "outline".to_string(),
            name: "创建大纲".to_string(),
            agent: writer.clone(),
            instructions: "基于研究结果创建详细的文章大纲".to_string(),
            condition: StepCondition::PreviousSuccess("research".to_string()),
            timeout_seconds: Some(20),
            retry_count: 1,
        })

        // 步骤3: 示例收集（并行执行，依赖研究）
        .add_step(WorkflowStep {
            id: "examples".to_string(),
            name: "收集示例".to_string(),
            agent: researcher,
            instructions: "收集相关的代码示例和案例研究".to_string(),
            condition: StepCondition::PreviousSuccess("research".to_string()),
            timeout_seconds: Some(25),
            retry_count: 1,
        })

        // 步骤4: 内容撰写（依赖大纲和示例）
        .add_step(WorkflowStep {
            id: "writing".to_string(),
            name: "内容撰写".to_string(),
            agent: writer,
            instructions: "基于大纲和示例撰写完整文章".to_string(),
            condition: StepCondition::AllPreviousSuccess(vec!["outline".to_string(), "examples".to_string()]),
            timeout_seconds: Some(60),
            retry_count: 1,
        })

        // 步骤5: 技术审查
        .add_step(WorkflowStep {
            id: "tech_review".to_string(),
            name: "技术审查".to_string(),
            agent: reviewer.clone(),
            instructions: "审查技术内容的准确性".to_string(),
            condition: StepCondition::PreviousSuccess("writing".to_string()),
            timeout_seconds: Some(30),
            retry_count: 2,
        })

        // 步骤6: 编辑审查
        .add_step(WorkflowStep {
            id: "editorial_review".to_string(),
            name: "编辑审查".to_string(),
            agent: reviewer,
            instructions: "审查文章的语言和结构".to_string(),
            condition: StepCondition::PreviousSuccess("tech_review".to_string()),
            timeout_seconds: Some(25),
            retry_count: 1,
        })

        // 步骤7: 发布准备
        .add_step(WorkflowStep {
            id: "publish".to_string(),
            name: "发布准备".to_string(),
            agent: publisher,
            instructions: "格式化文章并准备发布".to_string(),
            condition: StepCondition::PreviousSuccess("editorial_review".to_string()),
            timeout_seconds: Some(20),
            retry_count: 1,
        })

        .build()?;

    println!("创建复杂工作流:");
    println!("  名称: {}", complex_workflow.name());
    println!("  总步骤数: {}", complex_workflow.steps().len());
    println!("  并行步骤: outline + examples");
    println!("  条件依赖: writing 依赖 outline 和 examples");

    // 执行复杂工作流
    println!("\n执行复杂工作流...");
    let complex_input = json!({
        "topic": "Rust 性能优化技巧",
        "platform": "技术博客",
        "target_length": 3000,
        "include_benchmarks": true
    });

    let complex_result = complex_workflow.execute(complex_input).await?;

    println!("\n复杂工作流执行结果:");
    println!("  最终状态: {:?}", complex_result.status);
    println!("  总执行时间: {:?}", complex_result.execution_time);
    println!("  成功步骤: {}/{}", complex_result.completed_steps, complex_result.total_steps);

    // 分析执行路径
    println!("\n执行路径分析:");
    let mut execution_order = Vec::new();
    for (step_id, step_result) in &complex_result.step_results {
        execution_order.push((step_id, step_result.start_time, step_result.execution_time));
    }
    execution_order.sort_by_key(|(_, start_time, _)| *start_time);

    for (i, (step_id, start_time, duration)) in execution_order.iter().enumerate() {
        println!("  {}. {} (开始: {:?}, 耗时: {:?})",
            i + 1, step_id, start_time, duration);
    }

    Ok(())
}

/// 演示条件执行和错误处理
async fn demo_conditional_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示4: 条件执行和错误处理 ===");

    // 创建可能失败的代理（用于演示错误处理）
    let unreliable_agent = create_unreliable_agent().await?;
    let fallback_agent = create_writer_agent().await?;
    let validator = create_reviewer_agent().await?;

    // 创建带有错误处理的工作流
    let error_handling_workflow = WorkflowBuilder::new()
        .name("error_handling_demo")
        .description("演示错误处理和条件执行的工作流")

        // 步骤1: 尝试主要处理
        .add_step(WorkflowStep {
            id: "primary_task".to_string(),
            name: "主要任务".to_string(),
            agent: unreliable_agent,
            instructions: "执行主要任务（可能失败）".to_string(),
            condition: StepCondition::Always,
            timeout_seconds: Some(10),
            retry_count: 2,
        })

        // 步骤2: 备用处理（仅在主要任务失败时执行）
        .add_step(WorkflowStep {
            id: "fallback_task".to_string(),
            name: "备用任务".to_string(),
            agent: fallback_agent.clone(),
            instructions: "执行备用任务".to_string(),
            condition: StepCondition::PreviousFailure("primary_task".to_string()),
            timeout_seconds: Some(20),
            retry_count: 1,
        })

        // 步骤3: 成功路径验证（仅在主要任务成功时执行）
        .add_step(WorkflowStep {
            id: "success_validation".to_string(),
            name: "成功验证".to_string(),
            agent: validator.clone(),
            instructions: "验证主要任务的结果".to_string(),
            condition: StepCondition::PreviousSuccess("primary_task".to_string()),
            timeout_seconds: Some(15),
            retry_count: 1,
        })

        // 步骤4: 最终处理（无论前面成功还是失败都执行）
        .add_step(WorkflowStep {
            id: "final_processing".to_string(),
            name: "最终处理".to_string(),
            agent: fallback_agent,
            instructions: "执行最终处理和清理工作".to_string(),
            condition: StepCondition::AnyPreviousCompleted(vec![
                "success_validation".to_string(),
                "fallback_task".to_string()
            ]),
            timeout_seconds: Some(10),
            retry_count: 1,
        })

        .build()?;

    println!("创建错误处理工作流:");
    println!("  主要路径: primary_task -> success_validation -> final_processing");
    println!("  备用路径: primary_task (失败) -> fallback_task -> final_processing");

    // 测试两种场景

    // 场景1: 模拟主要任务成功
    println!("\n--- 场景1: 主要任务成功 ---");
    let success_input = json!({
        "task_type": "success",
        "content": "这是一个成功的任务"
    });

    let success_result = error_handling_workflow.execute(success_input).await?;
    print_workflow_execution_summary(&success_result);

    // 场景2: 模拟主要任务失败
    println!("\n--- 场景2: 主要任务失败 ---");
    let failure_input = json!({
        "task_type": "failure",
        "content": "这是一个会失败的任务"
    });

    let failure_result = error_handling_workflow.execute(failure_input).await?;
    print_workflow_execution_summary(&failure_result);

    Ok(())
}

// ============================================================================
// 代理创建函数
// ============================================================================

/// 创建研究员代理
async fn create_researcher_agent() -> Result<Arc<dyn BasicAgent>, Box<dyn std::error::Error>> {
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
async fn create_writer_agent() -> Result<Arc<dyn BasicAgent>, Box<dyn std::error::Error>> {
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
async fn create_reviewer_agent() -> Result<Arc<dyn BasicAgent>, Box<dyn std::error::Error>> {
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
async fn create_publisher_agent() -> Result<Arc<dyn BasicAgent>, Box<dyn std::error::Error>> {
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
async fn create_unreliable_agent() -> Result<Arc<dyn BasicAgent>, Box<dyn std::error::Error>> {
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

/// 打印工作流执行摘要
fn print_workflow_execution_summary(result: &WorkflowResult) {
    println!("执行摘要:");
    println!("  状态: {:?}", result.status);
    println!("  执行时间: {:?}", result.execution_time);
    println!("  完成步骤: {}/{}", result.completed_steps, result.total_steps);

    println!("  步骤详情:");
    for (step_id, step_result) in &result.step_results {
        let status_icon = match step_result.status {
            WorkflowStatus::Success => "✅",
            WorkflowStatus::Failed => "❌",
            WorkflowStatus::Skipped => "⏭️",
            WorkflowStatus::Running => "🔄",
            WorkflowStatus::Pending => "⏳",
        };

        println!("    {} {}: {:?} (耗时: {:?})",
            status_icon, step_id, step_result.status, step_result.execution_time);

        if let Some(output) = &step_result.output {
            println!("      输出: {}", output);
        }
        if let Some(error) = &step_result.error {
            println!("      错误: {}", error);
        }
    }
}

/// 分析工作流性能
#[allow(dead_code)]
fn analyze_workflow_performance(result: &WorkflowResult) -> HashMap<String, f64> {
    let mut metrics = HashMap::new();

    let total_time = result.execution_time.as_secs_f64();
    metrics.insert("total_time_seconds".to_string(), total_time);

    let success_rate = result.completed_steps as f64 / result.total_steps as f64;
    metrics.insert("success_rate".to_string(), success_rate);

    let avg_step_time = if result.step_results.is_empty() {
        0.0
    } else {
        let total_step_time: f64 = result.step_results.values()
            .map(|r| r.execution_time.as_secs_f64())
            .sum();
        total_step_time / result.step_results.len() as f64
    };
    metrics.insert("avg_step_time_seconds".to_string(), avg_step_time);

    metrics
}