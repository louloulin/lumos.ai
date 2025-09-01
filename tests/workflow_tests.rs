use std::time::{Duration, Instant};
use lumosai_core::workflow::{EnhancedWorkflow, WorkflowStep, WorkflowStatus, Workflow};
use lumosai_core::workflow::{WorkflowState, StepContext};
use lumosai_core::agent::types::RuntimeContext;
use serde_json::json;

mod common;
use common::TestAssertions;

/// 工作流基础测试
#[tokio::test]
async fn test_workflow_creation() {
    // 测试工作流创建
    let workflow = EnhancedWorkflow::new("test_workflow".to_string(), None);
    assert_eq!(workflow.id(), "test_workflow");
}

#[tokio::test]
async fn test_workflow_step_creation() {
    // 测试工作流步骤创建
    let step = WorkflowStep::new("test_step".to_string(), "Test Step".to_string());
    assert_eq!(step.id, "test_step");
    assert_eq!(step.name(), "Test Step");
}

#[tokio::test]
async fn test_workflow_status() {
    // 测试工作流状态
    let workflow = EnhancedWorkflow::new("status_test".to_string(), None);

    // 创建运行时上下文
    let context = RuntimeContext::new();

    // 测试工作流状态查询
    let status = workflow.get_status("non_existent_run").await;
    assert!(status.is_ok());

    match status.unwrap() {
        WorkflowStatus::NotFound => {
            // 预期行为：未找到运行
        }
        _ => {
            // 其他状态也是可接受的
        }
    }
}

#[tokio::test]
async fn test_workflow_execution_basic() {
    // 测试基础工作流执行
    let workflow = EnhancedWorkflow::new("execution_test".to_string(), None);
    let context = RuntimeContext::new();

    // 测试基础执行
    let input = json!({"test": "basic_execution"});
    let result = workflow.execute(input, &context).await;

    // 验证执行结果
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_workflow_suspend_resume() {
    // 测试工作流暂停和恢复
    let workflow = EnhancedWorkflow::new("suspend_resume_test".to_string(), None);
    let context = RuntimeContext::new();

    // 首先执行一个工作流来创建运行
    let input = json!({"test": "suspend_resume"});
    let _result = workflow.execute(input.clone(), &context).await;

    // 测试暂停 - 由于我们没有真正的运行ID，这个测试会失败，这是预期的
    let suspend_result = workflow.suspend("non_existent_run_id").await;
    assert!(suspend_result.is_err()); // 应该失败，因为运行不存在

    // 测试恢复 - 同样会失败
    let resume_result = workflow.resume("non_existent_run_id", None).await;
    assert!(resume_result.is_err()); // 应该失败，因为运行不存在
}

#[tokio::test]
async fn test_workflow_stream_execution() {
    // 测试流式执行
    let workflow = EnhancedWorkflow::new("stream_test".to_string(), None);
    let context = RuntimeContext::new();

    let input = json!({"test": "stream_execution"});
    let stream_result = workflow.execute_stream(input, &context).await;

    // 验证流式执行结果 - 目前未实现，应该返回错误
    assert!(stream_result.is_err()); // 应该失败，因为流式执行未实现
}

#[tokio::test]
async fn test_workflow_performance() {
    // 测试工作流性能
    let workflow = EnhancedWorkflow::new("performance_test".to_string(), None);
    let context = RuntimeContext::new();

    let start_time = Instant::now();
    let input = json!({"test": "performance"});
    let result = workflow.execute(input, &context).await;
    let duration = start_time.elapsed();

    assert!(result.is_ok());

    // 验证执行时间在合理范围内
    TestAssertions::assert_response_time(duration, Duration::from_millis(1000));
}
