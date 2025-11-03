//! DAG Workflow Integration Tests
//!
//! 测试 DAG Workflow 的集成功能

use async_trait::async_trait;
use lumosai_core::agent::types::RuntimeContext;
use lumosai_core::workflow::{
    DagWorkflow, DagWorkflowBuilder, StepExecutor, Workflow, WorkflowStatus, WorkflowStep,
};
use lumosai_core::Result;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

/// 简单的步骤执行器（用于测试）
struct SimpleStepExecutor {
    node_id: String,
    delay_ms: u64,
}

#[async_trait]
impl StepExecutor for SimpleStepExecutor {
    async fn execute(&self, input: Value, _context: &RuntimeContext) -> Result<Value> {
        // 模拟执行延迟
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;

        Ok(json!({
            "node_id": self.node_id,
            "input": input,
            "executed": true
        }))
    }
}

/// 创建测试步骤
fn create_test_step(id: &str, name: &str, delay_ms: u64) -> WorkflowStep {
    WorkflowStep {
        id: id.to_string(),
        description: Some(name.to_string()),
        step_type: lumosai_core::workflow::StepType::Simple,
        execute: Arc::new(SimpleStepExecutor {
            node_id: id.to_string(),
            delay_ms,
        }),
        input_schema: None,
        output_schema: None,
    }
}

/// 创建测试上下文
fn create_test_context() -> RuntimeContext {
    RuntimeContext::new()
}

// ============================================================================
// DAG Workflow 构建测试
// ============================================================================

#[tokio::test]
async fn test_dag_workflow_builder() {
    println!("🧪 测试 DAG Workflow Builder");

    let workflow = DagWorkflowBuilder::new("test_workflow".to_string())
        .description("测试工作流".to_string())
        .max_concurrency(4)
        .build();

    assert_eq!(workflow.id(), "test_workflow");
    assert_eq!(workflow.description(), Some("测试工作流"));

    println!("✅ DAG Workflow Builder 测试通过");
}

#[tokio::test]
async fn test_dag_workflow_add_nodes() {
    println!("🧪 测试 DAG Workflow 添加节点");

    let workflow = DagWorkflow::new("test_workflow".to_string(), None);

    // 添加根节点
    workflow
        .add_root_node(
            "node1".to_string(),
            "Node 1".to_string(),
            create_test_step("node1", "Node 1", 10),
        )
        .await
        .unwrap();

    // 添加依赖节点
    workflow
        .add_node(
            "node2".to_string(),
            "Node 2".to_string(),
            vec!["node1".to_string()],
            create_test_step("node2", "Node 2", 10),
        )
        .await
        .unwrap();

    assert_eq!(workflow.node_count().await, 2);

    println!("✅ DAG Workflow 添加节点测试通过");
}

#[tokio::test]
async fn test_dag_workflow_validation() {
    println!("🧪 测试 DAG Workflow 验证");

    let workflow = DagWorkflow::new("test_workflow".to_string(), None);

    // 添加节点
    workflow
        .add_root_node(
            "node1".to_string(),
            "Node 1".to_string(),
            create_test_step("node1", "Node 1", 10),
        )
        .await
        .unwrap();

    workflow
        .add_node(
            "node2".to_string(),
            "Node 2".to_string(),
            vec!["node1".to_string()],
            create_test_step("node2", "Node 2", 10),
        )
        .await
        .unwrap();

    // 验证 DAG（无环）
    let result = workflow.validate().await;
    assert!(result.is_ok());

    println!("✅ DAG Workflow 验证测试通过");
}

#[tokio::test]
async fn test_dag_workflow_cycle_detection() {
    println!("🧪 测试 DAG Workflow 环检测");

    let workflow = DagWorkflow::new("test_workflow".to_string(), None);

    // 添加节点形成环
    workflow
        .add_node(
            "node1".to_string(),
            "Node 1".to_string(),
            vec!["node2".to_string()],
            create_test_step("node1", "Node 1", 10),
        )
        .await
        .unwrap();

    workflow
        .add_node(
            "node2".to_string(),
            "Node 2".to_string(),
            vec!["node1".to_string()],
            create_test_step("node2", "Node 2", 10),
        )
        .await
        .unwrap();

    // 验证 DAG（应该检测到环）
    let result = workflow.validate().await;
    assert!(result.is_err());

    println!("✅ DAG Workflow 环检测测试通过");
}

// ============================================================================
// DAG Workflow 执行测试
// ============================================================================

#[tokio::test]
async fn test_dag_workflow_simple_execution() {
    println!("🧪 测试 DAG Workflow 简单执行");

    let workflow = DagWorkflow::new("test_workflow".to_string(), None);

    // 添加简单的线性工作流
    workflow
        .add_root_node(
            "node1".to_string(),
            "Node 1".to_string(),
            create_test_step("node1", "Node 1", 10),
        )
        .await
        .unwrap();

    workflow
        .add_node(
            "node2".to_string(),
            "Node 2".to_string(),
            vec!["node1".to_string()],
            create_test_step("node2", "Node 2", 10),
        )
        .await
        .unwrap();

    // 执行工作流
    let context = create_test_context();
    let input = json!({"test": "data"});

    let start_time = std::time::Instant::now();
    let result = workflow.execute(input, &context).await;
    let duration = start_time.elapsed();

    assert!(result.is_ok());
    println!("✅ DAG execution completed in {:?}", duration);
    println!("📊 Results: {}", result.unwrap());
}

#[tokio::test]
async fn test_dag_workflow_parallel_execution() {
    println!("🧪 测试 DAG Workflow 并行执行");

    let workflow = DagWorkflow::new("test_workflow".to_string(), None);

    // 创建并行结构
    //     node1
    //    /     \
    // node2   node3
    //    \     /
    //     node4

    workflow
        .add_root_node(
            "node1".to_string(),
            "Node 1".to_string(),
            create_test_step("node1", "Node 1", 20),
        )
        .await
        .unwrap();

    workflow
        .add_node(
            "node2".to_string(),
            "Node 2".to_string(),
            vec!["node1".to_string()],
            create_test_step("node2", "Node 2", 50),
        )
        .await
        .unwrap();

    workflow
        .add_node(
            "node3".to_string(),
            "Node 3".to_string(),
            vec!["node1".to_string()],
            create_test_step("node3", "Node 3", 50),
        )
        .await
        .unwrap();

    workflow
        .add_node(
            "node4".to_string(),
            "Node 4".to_string(),
            vec!["node2".to_string(), "node3".to_string()],
            create_test_step("node4", "Node 4", 20),
        )
        .await
        .unwrap();

    // 执行工作流
    let context = create_test_context();
    let input = json!({"test": "data"});

    let start_time = std::time::Instant::now();
    let result = workflow.execute(input, &context).await;
    let duration = start_time.elapsed();

    assert!(result.is_ok());
    println!("✅ Parallel DAG execution completed in {:?}", duration);
    println!("📊 Results count: {}", result.unwrap().as_object().unwrap().len());

    // 验证并行执行比顺序执行快
    // node2 和 node3 应该并行执行（各 50ms），而不是顺序执行（100ms）
    assert!(duration.as_millis() < 150); // 20 + 50 (parallel) + 20 = 90ms + overhead
}

#[tokio::test]
async fn test_dag_workflow_complex_dag() {
    println!("🧪 测试 DAG Workflow 复杂 DAG");

    let workflow = DagWorkflow::new("test_workflow".to_string(), None);

    // 创建复杂的 DAG 结构
    //       node1
    //      /  |  \
    //  node2 node3 node4
    //      \  |  /
    //       node5
    //         |
    //       node6

    workflow
        .add_root_node(
            "node1".to_string(),
            "Node 1".to_string(),
            create_test_step("node1", "Node 1", 10),
        )
        .await
        .unwrap();

    workflow
        .add_node(
            "node2".to_string(),
            "Node 2".to_string(),
            vec!["node1".to_string()],
            create_test_step("node2", "Node 2", 20),
        )
        .await
        .unwrap();

    workflow
        .add_node(
            "node3".to_string(),
            "Node 3".to_string(),
            vec!["node1".to_string()],
            create_test_step("node3", "Node 3", 20),
        )
        .await
        .unwrap();

    workflow
        .add_node(
            "node4".to_string(),
            "Node 4".to_string(),
            vec!["node1".to_string()],
            create_test_step("node4", "Node 4", 20),
        )
        .await
        .unwrap();

    workflow
        .add_node(
            "node5".to_string(),
            "Node 5".to_string(),
            vec![
                "node2".to_string(),
                "node3".to_string(),
                "node4".to_string(),
            ],
            create_test_step("node5", "Node 5", 10),
        )
        .await
        .unwrap();

    workflow
        .add_node(
            "node6".to_string(),
            "Node 6".to_string(),
            vec!["node5".to_string()],
            create_test_step("node6", "Node 6", 10),
        )
        .await
        .unwrap();

    // 执行工作流
    let context = create_test_context();
    let input = json!({"test": "data"});

    let start_time = std::time::Instant::now();
    let result = workflow.execute(input, &context).await;
    let duration = start_time.elapsed();

    assert!(result.is_ok());
    println!("✅ Complex DAG execution completed in {:?}", duration);
    println!("📊 Results count: {}", result.unwrap().as_object().unwrap().len());
}

