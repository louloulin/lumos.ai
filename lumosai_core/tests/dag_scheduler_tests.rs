//! DAG 调度器测试
//!
//! P0-2.1 任务：验证 DAG 并行调度器功能
//!
//! 测试范围：
//! 1. DAG 构建和验证
//! 2. 环检测
//! 3. 拓扑排序
//! 4. 并行执行
//! 5. 依赖管理

use async_trait::async_trait;
use lumosai_core::agent::types::RuntimeContext;
use lumosai_core::workflow::dag_scheduler::{Dag, DagNode, DagScheduler};
use lumosai_core::workflow::{StepExecutor, WorkflowStep};
use lumosai_core::Result;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// 测试辅助函数
// ============================================================================

/// 简单的步骤执行器（返回输入 + 节点 ID）
struct SimpleStepExecutor {
    node_id: String,
    delay_ms: u64,
}

#[async_trait]
impl StepExecutor for SimpleStepExecutor {
    async fn execute(&self, input: Value, _context: &RuntimeContext) -> Result<Value> {
        // 模拟执行延迟
        if self.delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
        }

        // 返回输入 + 节点信息
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
        input_schema: None,
        output_schema: None,
        execute: Arc::new(SimpleStepExecutor {
            node_id: id.to_string(),
            delay_ms,
        }),
    }
}

/// 创建测试上下文
fn create_test_context() -> RuntimeContext {
    RuntimeContext::new()
}

// ============================================================================
// DAG 构建和验证测试
// ============================================================================

#[tokio::test]
async fn test_dag_basic_construction() {
    let mut dag = Dag::new();

    // 添加节点
    let node1 = DagNode {
        id: "node1".to_string(),
        name: "Node 1".to_string(),
        dependencies: vec![],
        step: create_test_step("node1", "Node 1", 0),
    };

    dag.add_node(node1).unwrap();

    assert_eq!(dag.node_count(), 1);
    assert!(dag.get_node("node1").is_some());
}

#[tokio::test]
async fn test_dag_with_dependencies() {
    let mut dag = Dag::new();

    // 添加节点 1（无依赖）
    dag.add_node(DagNode {
        id: "node1".to_string(),
        name: "Node 1".to_string(),
        dependencies: vec![],
        step: create_test_step("node1", "Node 1", 0),
    })
    .unwrap();

    // 添加节点 2（依赖节点 1）
    dag.add_node(DagNode {
        id: "node2".to_string(),
        name: "Node 2".to_string(),
        dependencies: vec!["node1".to_string()],
        step: create_test_step("node2", "Node 2", 0),
    })
    .unwrap();

    // 添加节点 3（依赖节点 1）
    dag.add_node(DagNode {
        id: "node3".to_string(),
        name: "Node 3".to_string(),
        dependencies: vec!["node1".to_string()],
        step: create_test_step("node3", "Node 3", 0),
    })
    .unwrap();

    assert_eq!(dag.node_count(), 3);
}

#[tokio::test]
async fn test_dag_duplicate_node_error() {
    let mut dag = Dag::new();

    let node1 = DagNode {
        id: "node1".to_string(),
        name: "Node 1".to_string(),
        dependencies: vec![],
        step: create_test_step("node1", "Node 1", 0),
    };

    dag.add_node(node1.clone()).unwrap();

    // 尝试添加重复节点
    let result = dag.add_node(node1);
    assert!(result.is_err());
}

// ============================================================================
// 环检测测试
// ============================================================================

#[tokio::test]
async fn test_dag_cycle_detection_simple() {
    let mut dag = Dag::new();

    // 创建环：node1 -> node2 -> node1
    dag.add_node(DagNode {
        id: "node1".to_string(),
        name: "Node 1".to_string(),
        dependencies: vec!["node2".to_string()],
        step: create_test_step("node1", "Node 1", 0),
    })
    .unwrap();

    dag.add_node(DagNode {
        id: "node2".to_string(),
        name: "Node 2".to_string(),
        dependencies: vec!["node1".to_string()],
        step: create_test_step("node2", "Node 2", 0),
    })
    .unwrap();

    // 检测环
    let result = dag.detect_cycle();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_dag_no_cycle() {
    let mut dag = Dag::new();

    // 创建无环 DAG：node1 -> node2 -> node3
    dag.add_node(DagNode {
        id: "node1".to_string(),
        name: "Node 1".to_string(),
        dependencies: vec![],
        step: create_test_step("node1", "Node 1", 0),
    })
    .unwrap();

    dag.add_node(DagNode {
        id: "node2".to_string(),
        name: "Node 2".to_string(),
        dependencies: vec!["node1".to_string()],
        step: create_test_step("node2", "Node 2", 0),
    })
    .unwrap();

    dag.add_node(DagNode {
        id: "node3".to_string(),
        name: "Node 3".to_string(),
        dependencies: vec!["node2".to_string()],
        step: create_test_step("node3", "Node 3", 0),
    })
    .unwrap();

    // 检测环
    let result = dag.detect_cycle();
    assert!(result.is_ok());
}

// ============================================================================
// 拓扑排序测试
// ============================================================================

#[tokio::test]
async fn test_dag_topological_sort_linear() {
    let mut dag = Dag::new();

    // 创建线性 DAG：node1 -> node2 -> node3
    dag.add_node(DagNode {
        id: "node1".to_string(),
        name: "Node 1".to_string(),
        dependencies: vec![],
        step: create_test_step("node1", "Node 1", 0),
    })
    .unwrap();

    dag.add_node(DagNode {
        id: "node2".to_string(),
        name: "Node 2".to_string(),
        dependencies: vec!["node1".to_string()],
        step: create_test_step("node2", "Node 2", 0),
    })
    .unwrap();

    dag.add_node(DagNode {
        id: "node3".to_string(),
        name: "Node 3".to_string(),
        dependencies: vec!["node2".to_string()],
        step: create_test_step("node3", "Node 3", 0),
    })
    .unwrap();

    let levels = dag.topological_sort().unwrap();

    println!("✅ Topological sort (linear): {:?}", levels);

    // 应该有 3 个层级
    assert_eq!(levels.len(), 3);
    assert_eq!(levels[0], vec!["node1"]);
    assert_eq!(levels[1], vec!["node2"]);
    assert_eq!(levels[2], vec!["node3"]);
}

#[tokio::test]
async fn test_dag_topological_sort_parallel() {
    let mut dag = Dag::new();

    // 创建并行 DAG：
    //     node1
    //    /  |  \
    // node2 node3 node4
    //    \  |  /
    //     node5

    dag.add_node(DagNode {
        id: "node1".to_string(),
        name: "Node 1".to_string(),
        dependencies: vec![],
        step: create_test_step("node1", "Node 1", 0),
    })
    .unwrap();

    dag.add_node(DagNode {
        id: "node2".to_string(),
        name: "Node 2".to_string(),
        dependencies: vec!["node1".to_string()],
        step: create_test_step("node2", "Node 2", 0),
    })
    .unwrap();

    dag.add_node(DagNode {
        id: "node3".to_string(),
        name: "Node 3".to_string(),
        dependencies: vec!["node1".to_string()],
        step: create_test_step("node3", "Node 3", 0),
    })
    .unwrap();

    dag.add_node(DagNode {
        id: "node4".to_string(),
        name: "Node 4".to_string(),
        dependencies: vec!["node1".to_string()],
        step: create_test_step("node4", "Node 4", 0),
    })
    .unwrap();

    dag.add_node(DagNode {
        id: "node5".to_string(),
        name: "Node 5".to_string(),
        dependencies: vec![
            "node2".to_string(),
            "node3".to_string(),
            "node4".to_string(),
        ],
        step: create_test_step("node5", "Node 5", 0),
    })
    .unwrap();

    let levels = dag.topological_sort().unwrap();

    println!("✅ Topological sort (parallel): {:?}", levels);

    // 应该有 3 个层级
    assert_eq!(levels.len(), 3);
    assert_eq!(levels[0], vec!["node1"]);
    assert_eq!(levels[1].len(), 3); // node2, node3, node4 可以并行
    assert_eq!(levels[2], vec!["node5"]);
}

// ============================================================================
// DAG 执行测试
// ============================================================================

#[tokio::test]
async fn test_dag_scheduler_simple_execution() {
    let mut dag = Dag::new();

    // 创建简单的 DAG：node1 -> node2
    dag.add_node(DagNode {
        id: "node1".to_string(),
        name: "Node 1".to_string(),
        dependencies: vec![],
        step: create_test_step("node1", "Node 1", 10),
    })
    .unwrap();

    dag.add_node(DagNode {
        id: "node2".to_string(),
        name: "Node 2".to_string(),
        dependencies: vec!["node1".to_string()],
        step: create_test_step("node2", "Node 2", 10),
    })
    .unwrap();

    let scheduler = DagScheduler::new(4);
    let context = create_test_context();
    let input = json!({"test": "data"});

    let start = std::time::Instant::now();
    let results = scheduler.execute(&dag, input, &context).await.unwrap();
    let elapsed = start.elapsed();

    println!("✅ DAG execution completed in {:?}", elapsed);
    println!("📊 Results: {:?}", results);

    // 验证结果
    assert_eq!(results.len(), 2);
    assert!(results.contains_key("node1"));
    assert!(results.contains_key("node2"));

    // 验证 node2 的输入包含 node1 的输出
    let node2_result = &results["node2"];
    assert!(node2_result["input"]["node1"].is_object());
}

#[tokio::test]
async fn test_dag_scheduler_parallel_execution() {
    let mut dag = Dag::new();

    // 创建并行 DAG：
    //     node1
    //    /     \
    // node2   node3
    //    \     /
    //     node4

    dag.add_node(DagNode {
        id: "node1".to_string(),
        name: "Node 1".to_string(),
        dependencies: vec![],
        step: create_test_step("node1", "Node 1", 10),
    })
    .unwrap();

    dag.add_node(DagNode {
        id: "node2".to_string(),
        name: "Node 2".to_string(),
        dependencies: vec!["node1".to_string()],
        step: create_test_step("node2", "Node 2", 50),
    })
    .unwrap();

    dag.add_node(DagNode {
        id: "node3".to_string(),
        name: "Node 3".to_string(),
        dependencies: vec!["node1".to_string()],
        step: create_test_step("node3", "Node 3", 50),
    })
    .unwrap();

    dag.add_node(DagNode {
        id: "node4".to_string(),
        name: "Node 4".to_string(),
        dependencies: vec!["node2".to_string(), "node3".to_string()],
        step: create_test_step("node4", "Node 4", 10),
    })
    .unwrap();

    let scheduler = DagScheduler::new(4);
    let context = create_test_context();
    let input = json!({"test": "parallel"});

    let start = std::time::Instant::now();
    let results = scheduler.execute(&dag, input, &context).await.unwrap();
    let elapsed = start.elapsed();

    println!("✅ Parallel DAG execution completed in {:?}", elapsed);
    println!("📊 Results count: {}", results.len());

    // 验证结果
    assert_eq!(results.len(), 4);

    // 并行执行应该比顺序执行快
    // node2 和 node3 应该并行执行（各 50ms），而不是顺序执行（100ms）
    assert!(
        elapsed.as_millis() < 150,
        "Parallel execution should be faster than sequential"
    );
}

#[tokio::test]
async fn test_dag_scheduler_complex_dag() {
    let mut dag = Dag::new();

    // 创建复杂的 DAG：
    //       node1
    //      /  |  \
    //   node2 node3 node4
    //      \  |  /
    //       node5
    //         |
    //       node6

    dag.add_node(DagNode {
        id: "node1".to_string(),
        name: "Node 1".to_string(),
        dependencies: vec![],
        step: create_test_step("node1", "Node 1", 10),
    })
    .unwrap();

    for i in 2..=4 {
        dag.add_node(DagNode {
            id: format!("node{}", i),
            name: format!("Node {}", i),
            dependencies: vec!["node1".to_string()],
            step: create_test_step(&format!("node{}", i), &format!("Node {}", i), 20),
        })
        .unwrap();
    }

    dag.add_node(DagNode {
        id: "node5".to_string(),
        name: "Node 5".to_string(),
        dependencies: vec![
            "node2".to_string(),
            "node3".to_string(),
            "node4".to_string(),
        ],
        step: create_test_step("node5", "Node 5", 10),
    })
    .unwrap();

    dag.add_node(DagNode {
        id: "node6".to_string(),
        name: "Node 6".to_string(),
        dependencies: vec!["node5".to_string()],
        step: create_test_step("node6", "Node 6", 10),
    })
    .unwrap();

    let scheduler = DagScheduler::new(4);
    let context = create_test_context();
    let input = json!({"test": "complex"});

    let start = std::time::Instant::now();
    let results = scheduler.execute(&dag, input, &context).await.unwrap();
    let elapsed = start.elapsed();

    println!("✅ Complex DAG execution completed in {:?}", elapsed);
    println!("📊 Results count: {}", results.len());

    // 验证结果
    assert_eq!(results.len(), 6);

    // 验证所有节点都被执行
    for i in 1..=6 {
        assert!(results.contains_key(&format!("node{}", i)));
    }
}

#[tokio::test]
async fn test_dag_scheduler_concurrency_limit() {
    let mut dag = Dag::new();

    // 创建 10 个并行节点
    dag.add_node(DagNode {
        id: "root".to_string(),
        name: "Root".to_string(),
        dependencies: vec![],
        step: create_test_step("root", "Root", 10),
    })
    .unwrap();

    for i in 1..=10 {
        dag.add_node(DagNode {
            id: format!("node{}", i),
            name: format!("Node {}", i),
            dependencies: vec!["root".to_string()],
            step: create_test_step(&format!("node{}", i), &format!("Node {}", i), 50),
        })
        .unwrap();
    }

    // 使用较小的并发度
    let scheduler = DagScheduler::new(2);
    let context = create_test_context();
    let input = json!({"test": "concurrency"});

    let start = std::time::Instant::now();
    let results = scheduler.execute(&dag, input, &context).await.unwrap();
    let elapsed = start.elapsed();

    println!(
        "✅ Concurrency-limited DAG execution completed in {:?}",
        elapsed
    );
    println!("📊 Results count: {}", results.len());

    // 验证结果
    assert_eq!(results.len(), 11);

    // 由于并发度限制为 2，10 个节点应该分 5 批执行
    // 每批 50ms，总共约 250ms（加上 root 的 10ms）
    assert!(
        elapsed.as_millis() >= 250,
        "Should respect concurrency limit"
    );
}
