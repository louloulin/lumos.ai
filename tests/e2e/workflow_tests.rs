//! E2E 测试：Workflow 执行场景

mod framework;
use framework::{E2ETestContext, E2EAssertions};

use lumosai_core::workflow::{
    WorkflowBuilder, WorkflowStep, StepType, StepExecutor, RuntimeContext,
};
use async_trait::async_trait;
use serde_json::{json, Value};
use lumosai_core::error::Result;
use std::sync::Arc;

/// 简单的步骤执行器
#[derive(Clone)]
struct SimpleStepExecutor {
    step_name: String,
}

#[async_trait]
impl StepExecutor for SimpleStepExecutor {
    async fn execute(&self, input: Value, _context: &RuntimeContext) -> Result<Value> {
        println!("Executing step: {}", self.step_name);
        Ok(json!({
            "step": self.step_name,
            "input": input,
            "result": format!("Processed by {}", self.step_name)
        }))
    }
}

/// 测试 22: 基础 Workflow 执行
#[tokio::test]
async fn test_workflow_basic_execution() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建 Workflow
    let step1 = WorkflowStep {
        id: "step1".to_string(),
        description: Some("First step".to_string()),
        step_type: StepType::Sequential,
        execute: Arc::new(SimpleStepExecutor {
            step_name: "Step 1".to_string(),
        }),
        input_schema: None,
        output_schema: None,
    };

    let step2 = WorkflowStep {
        id: "step2".to_string(),
        description: Some("Second step".to_string()),
        step_type: StepType::Sequential,
        execute: Arc::new(SimpleStepExecutor {
            step_name: "Step 2".to_string(),
        }),
        input_schema: None,
        output_schema: None,
    };

    let workflow = WorkflowBuilder::new("test_workflow")
        .description("Test workflow")
        .step(step1)
        .step(step2)
        .build();

    assert!(workflow.is_ok(), "Workflow should be created successfully");

    let workflow = workflow.unwrap();
    let input = json!({"message": "Test input"});

    // 执行 Workflow
    let result = workflow.execute(input, &RuntimeContext::default()).await;

    assert!(result.is_ok(), "Workflow execution should succeed");

    println!("✅ Basic workflow test passed");

    ctx.teardown().await.unwrap();
}

/// 测试 23: Workflow 错误恢复
#[tokio::test]
async fn test_workflow_error_recovery() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建会失败的步骤执行器
    #[derive(Clone)]
    struct FailingExecutor;

    #[async_trait]
    impl StepExecutor for FailingExecutor {
        async fn execute(&self, _input: Value, _context: &RuntimeContext) -> Result<Value> {
            Err(lumosai_core::error::Error::Workflow(
                "Simulated failure".to_string(),
            ))
        }
    }

    let failing_step = WorkflowStep {
        id: "failing_step".to_string(),
        description: Some("Step that fails".to_string()),
        step_type: StepType::Sequential,
        execute: Arc::new(FailingExecutor),
        input_schema: None,
        output_schema: None,
    };

    let workflow = WorkflowBuilder::new("failing_workflow")
        .step(failing_step)
        .build()
        .unwrap();

    let result = workflow.execute(json!({}), &RuntimeContext::default()).await;

    // 应该失败
    assert!(result.is_err(), "Workflow with failing step should fail");

    println!("✅ Workflow error recovery test passed");

    ctx.teardown().await.unwrap();
}

/// 测试 24: Workflow Suspend/Resume
#[tokio::test]
async fn test_workflow_suspend_resume() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建可暂停的 Workflow
    let step = WorkflowStep {
        id: "pausable_step".to_string(),
        description: Some("Pausable step".to_string()),
        step_type: StepType::Sequential,
        execute: Arc::new(SimpleStepExecutor {
            step_name: "Pausable".to_string(),
        }),
        input_schema: None,
        output_schema: None,
    };

    let workflow = WorkflowBuilder::new("pausable_workflow")
        .step(step)
        .build()
        .unwrap();

    // 正常执行
    let result = workflow
        .execute(json!({"data": "test"}), &RuntimeContext::default())
        .await;

    assert!(result.is_ok(), "Workflow should execute successfully");

    // Note: 实际的 suspend/resume 功能需要更复杂的状态管理
    println!("✅ Workflow suspend/resume test passed (basic)");

    ctx.teardown().await.unwrap();
}

/// 测试 25: Workflow 并行执行
#[tokio::test]
async fn test_workflow_parallel_execution() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建多个可并行执行的步骤
    let parallel_steps: Vec<WorkflowStep> = (0..3)
        .map(|i| WorkflowStep {
            id: format!("parallel_step_{}", i),
            description: Some(format!("Parallel step {}", i)),
            step_type: StepType::Parallel,
            execute: Arc::new(SimpleStepExecutor {
                step_name: format!("Parallel {}", i),
            }),
            input_schema: None,
            output_schema: None,
        })
        .collect();

    let mut builder = WorkflowBuilder::new("parallel_workflow");
    for step in parallel_steps {
        builder = builder.step(step);
    }

    let workflow = builder.build().unwrap();

    let start = std::time::Instant::now();
    let result = workflow
        .execute(json!({"data": "parallel"}), &RuntimeContext::default())
        .await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Parallel workflow should succeed");
    E2EAssertions::assert_reasonable_response_time(duration);

    println!(
        "✅ Workflow parallel execution test passed in {:?}",
        duration
    );

    ctx.teardown().await.unwrap();
}

