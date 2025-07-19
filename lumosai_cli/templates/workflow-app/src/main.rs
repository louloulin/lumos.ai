//! {{project_name}} - {{description}}
//! 
//! 这是一个使用LumosAI工作流系统的示例项目。

use lumosai_core::prelude::*;
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::workflow::{WorkflowBuilder, WorkflowStepConfig};
use std::sync::Arc;
use anyhow::Result;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 启动 {{project_name}} 工作流");

    // 创建LLM提供者
    let llm = Arc::new(MockLlmProvider::new(vec![
        "我已经完成了研究任务。".to_string(),
        "我已经完成了分析工作。".to_string(),
        "我已经生成了报告。".to_string(),
        "我已经完成了审核。".to_string(),
    ]));

    // 创建不同角色的Agent
    let research_agent = quick_agent("researcher", "你是一个专业的研究员，负责收集和整理信息")
        .model(llm.clone())
        .tools(vec![web_search(), file_reader()])
        .build()?;

    let analysis_agent = quick_agent("analyst", "你是一个数据分析师，负责分析研究结果")
        .model(llm.clone())
        .tools(vec![calculator(), statistics(), data_transformer()])
        .build()?;

    let report_agent = quick_agent("reporter", "你是一个报告撰写专家，负责生成专业报告")
        .model(llm.clone())
        .tools(vec![file_writer()])
        .build()?;

    let review_agent = quick_agent("reviewer", "你是一个质量审核员，负责审核最终输出")
        .model(llm.clone())
        .build()?;

    info!("✅ 所有Agent创建成功");

    // 方法1: 使用WorkflowBuilder创建工作流
    info!("📋 创建工作流 - 使用构建器模式");
    
    let workflow = WorkflowBuilder::new()
        .id("research_workflow")
        .name("Research and Analysis Workflow")
        .description("完整的研究分析工作流程")
        .add_step(WorkflowStepConfig {
            id: "research".to_string(),
            name: "Research Phase".to_string(),
            agent: Some(research_agent),
            instructions: "研究指定主题，收集相关信息和数据".to_string(),
            dependencies: vec![],
            timeout: Some(30),
            max_retries: Some(3),
            ..Default::default()
        })
        .add_step(WorkflowStepConfig {
            id: "analysis".to_string(),
            name: "Analysis Phase".to_string(),
            agent: Some(analysis_agent),
            instructions: "分析研究结果，提取关键洞察".to_string(),
            dependencies: vec!["research".to_string()],
            timeout: Some(30),
            max_retries: Some(3),
            ..Default::default()
        })
        .add_step(WorkflowStepConfig {
            id: "report".to_string(),
            name: "Report Generation".to_string(),
            agent: Some(report_agent),
            instructions: "基于分析结果生成专业报告".to_string(),
            dependencies: vec!["analysis".to_string()],
            timeout: Some(30),
            max_retries: Some(3),
            ..Default::default()
        })
        .add_step(WorkflowStepConfig {
            id: "review".to_string(),
            name: "Quality Review".to_string(),
            agent: Some(review_agent),
            instructions: "审核报告质量，确保符合标准".to_string(),
            dependencies: vec!["report".to_string()],
            timeout: Some(30),
            max_retries: Some(3),
            ..Default::default()
        })
        .build()?;

    info!("✅ 工作流创建成功");
    info!("📊 工作流信息:");
    info!("   - ID: {}", workflow.get_id());
    info!("   - 名称: {}", workflow.get_name());
    info!("   - 步骤数量: {}", workflow.get_steps().len());

    // 列出所有步骤
    info!("📋 工作流步骤:");
    for (i, step) in workflow.get_steps().iter().enumerate() {
        info!("   {}. {} ({})", i + 1, step.name, step.id);
        if !step.dependencies.is_empty() {
            info!("      依赖: {:?}", step.dependencies);
        }
    }

    // 执行工作流
    info!("🚀 开始执行工作流...");
    
    let input_data = serde_json::json!({
        "topic": "人工智能在医疗领域的应用",
        "requirements": "需要包含最新技术趋势和实际应用案例"
    });

    match workflow.execute(input_data).await {
        Ok(result) => {
            info!("✅ 工作流执行成功");
            info!("📄 执行结果: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            error!("❌ 工作流执行失败: {}", e);
        }
    }

    // 方法2: 演示使用宏创建工作流 (如果启用了宏功能)
    #[cfg(feature = "macros")]
    {
        use lumos_macro::workflow;
        
        info!("📋 创建工作流 - 使用宏");
        
        let macro_workflow = workflow! {
            name: "macro_research_workflow",
            description: "使用宏创建的研究工作流",
            steps: {
                {
                    name: "research",
                    agent: research_agent,
                    instructions: "使用宏定义的研究步骤",
                    timeout: 30000,
                    retry: { count: 3, delay: 1000 }
                },
                {
                    name: "analysis",
                    agent: analysis_agent,
                    instructions: "使用宏定义的分析步骤",
                    when: { previous_step_success: true },
                    timeout: 30000
                }
            }
        };
        
        info!("✅ 宏工作流创建成功");
    }

    // 演示并行工作流
    info!("🔄 演示并行工作流");
    
    let parallel_workflow = create_parallel_workflow(llm.clone()).await?;
    
    let parallel_input = serde_json::json!({
        "tasks": ["任务A", "任务B", "任务C"]
    });
    
    match parallel_workflow.execute(parallel_input).await {
        Ok(result) => {
            info!("✅ 并行工作流执行成功");
            info!("📄 并行执行结果: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            error!("❌ 并行工作流执行失败: {}", e);
        }
    }

    info!("🎉 {{project_name}} 工作流演示完成");
    Ok(())
}

/// 创建并行工作流示例
async fn create_parallel_workflow(llm: Arc<MockLlmProvider>) -> Result<lumosai_core::workflow::EnhancedWorkflow> {
    let agent_a = quick_agent("worker_a", "工作者A，处理任务A")
        .model(llm.clone())
        .build()?;
    
    let agent_b = quick_agent("worker_b", "工作者B，处理任务B")
        .model(llm.clone())
        .build()?;
    
    let agent_c = quick_agent("worker_c", "工作者C，处理任务C")
        .model(llm.clone())
        .build()?;
    
    let merger_agent = quick_agent("merger", "合并器，合并所有结果")
        .model(llm)
        .build()?;

    let workflow = WorkflowBuilder::new()
        .id("parallel_workflow")
        .name("Parallel Processing Workflow")
        .description("并行处理多个任务的工作流")
        // 并行步骤 - 没有相互依赖
        .add_step(WorkflowStepConfig {
            id: "task_a".to_string(),
            name: "Task A".to_string(),
            agent: Some(agent_a),
            instructions: "处理任务A".to_string(),
            dependencies: vec![],
            ..Default::default()
        })
        .add_step(WorkflowStepConfig {
            id: "task_b".to_string(),
            name: "Task B".to_string(),
            agent: Some(agent_b),
            instructions: "处理任务B".to_string(),
            dependencies: vec![],
            ..Default::default()
        })
        .add_step(WorkflowStepConfig {
            id: "task_c".to_string(),
            name: "Task C".to_string(),
            agent: Some(agent_c),
            instructions: "处理任务C".to_string(),
            dependencies: vec![],
            ..Default::default()
        })
        // 合并步骤 - 依赖所有并行步骤
        .add_step(WorkflowStepConfig {
            id: "merge".to_string(),
            name: "Merge Results".to_string(),
            agent: Some(merger_agent),
            instructions: "合并所有任务的结果".to_string(),
            dependencies: vec!["task_a".to_string(), "task_b".to_string(), "task_c".to_string()],
            ..Default::default()
        })
        .build()?;

    Ok(workflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_creation() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
        
        let agent = quick_agent("test_agent", "Test agent")
            .model(llm)
            .build()
            .expect("Agent创建失败");

        let workflow = WorkflowBuilder::new()
            .id("test_workflow")
            .name("Test Workflow")
            .add_step(WorkflowStepConfig {
                id: "test_step".to_string(),
                name: "Test Step".to_string(),
                agent: Some(agent),
                instructions: "Test instructions".to_string(),
                dependencies: vec![],
                ..Default::default()
            })
            .build();

        assert!(workflow.is_ok());
        let workflow = workflow.unwrap();
        assert_eq!(workflow.get_id(), "test_workflow");
        assert_eq!(workflow.get_steps().len(), 1);
    }

    #[tokio::test]
    async fn test_parallel_workflow() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
        
        let workflow = create_parallel_workflow(llm).await;
        assert!(workflow.is_ok());
        
        let workflow = workflow.unwrap();
        assert_eq!(workflow.get_steps().len(), 4); // 3个并行步骤 + 1个合并步骤
    }
}
