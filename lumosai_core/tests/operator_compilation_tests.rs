// 操作符编译测试
//
// 这个测试文件验证所有操作符能够正确编译
// 主要测试操作符的类型安全性和编译时检查

use lumosai_core::agent::{
    create_basic_agent, delegate, parallel, pipe, Agent, AgentDelegation, AgentParallel,
    AgentPipeline,
};
use lumosai_core::llm::test_helpers::{create_test_zhipu_provider, create_test_zhipu_provider_arc};
use lumosai_core::llm::LlmProvider;
use std::sync::Arc;

/// 创建测试用的 Agent
fn create_test_agent(name: &str, response: &str) -> Arc<dyn Agent> {
    let llm_provider: Arc<dyn LlmProvider> = create_test_zhipu_provider_arc();

    Arc::new(
        create_basic_agent(name.to_string(), format!("You are {}", name), llm_provider)
            .expect("Failed to create test agent"),
    )
}

/// 测试 1: AgentPipeline 基础编译
#[test]
fn test_pipeline_compiles() {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");

    let _pipeline = AgentPipeline::new(agent1).pipe(agent2);
}

/// 测试 2: AgentParallel 基础编译
#[test]
fn test_parallel_compiles() {
    let agent1 = create_test_agent("agent1", "Result 1");
    let agent2 = create_test_agent("agent2", "Result 2");

    let _parallel = AgentParallel::new(agent1).parallel(agent2);
}

/// 测试 3: AgentDelegation 基础编译
#[test]
fn test_delegation_compiles() {
    let manager = create_test_agent("manager", "Delegating task");
    let worker = create_test_agent("worker", "Task completed");

    let _delegation = AgentDelegation::new(manager, worker, "Complete this task".to_string());
}

/// 测试 4: pipe() 函数编译
#[test]
fn test_pipe_function_compiles() {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");

    let _pipeline = pipe(agent1, agent2);
}

/// 测试 5: parallel() 函数编译
#[test]
fn test_parallel_function_compiles() {
    let agent1 = create_test_agent("agent1", "Result 1");
    let agent2 = create_test_agent("agent2", "Result 2");

    let _parallel = parallel(agent1, agent2);
}

/// 测试 6: delegate() 函数编译
#[test]
fn test_delegate_function_compiles() {
    let manager = create_test_agent("manager", "Delegating");
    let worker = create_test_agent("worker", "Working");

    let _delegation = delegate(manager, worker, "Task description");
}

/// 测试 7: 链式 pipeline 编译
#[test]
fn test_chained_pipeline_compiles() {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");
    let agent3 = create_test_agent("agent3", "Step 3");

    let _pipeline = AgentPipeline::new(agent1).pipe(agent2).pipe(agent3);
}

/// 测试 8: 链式 parallel 编译
#[test]
fn test_chained_parallel_compiles() {
    let agent1 = create_test_agent("agent1", "Result 1");
    let agent2 = create_test_agent("agent2", "Result 2");
    let agent3 = create_test_agent("agent3", "Result 3");

    let _parallel = AgentParallel::new(agent1).parallel(agent2).parallel(agent3);
}

/// 测试 9: 多个 pipeline 编译
#[test]
fn test_multiple_pipelines_compile() {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");
    let agent3 = create_test_agent("agent3", "Step 3");
    let agent4 = create_test_agent("agent4", "Step 4");

    let _pipeline1 = pipe(agent1, agent2);
    let _pipeline2 = pipe(agent3, agent4);
}

/// 测试 10: 多个 parallel 编译
#[test]
fn test_multiple_parallels_compile() {
    let agent1 = create_test_agent("agent1", "Result 1");
    let agent2 = create_test_agent("agent2", "Result 2");
    let agent3 = create_test_agent("agent3", "Result 3");
    let agent4 = create_test_agent("agent4", "Result 4");

    let _parallel1 = parallel(agent1, agent2);
    let _parallel2 = parallel(agent3, agent4);
}

/// 测试 11: 验证操作符可以被移动
#[test]
fn test_operators_can_be_moved() {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");

    let pipeline = pipe(agent1, agent2);
    let moved_pipeline = pipeline;
    let _ = moved_pipeline;
}

/// 测试 12: 验证操作符在不同作用域中工作
#[test]
fn test_operators_in_different_scopes() {
    {
        let agent1 = create_test_agent("agent1", "Scope 1");
        let agent2 = create_test_agent("agent2", "Scope 1");
        let _pipeline = pipe(agent1, agent2);
    }

    {
        let agent1 = create_test_agent("agent1", "Scope 2");
        let agent2 = create_test_agent("agent2", "Scope 2");
        let _pipeline = pipe(agent1, agent2);
    }
}

/// 测试 13: 验证操作符可以存储在变量中
#[test]
fn test_operators_can_be_stored() {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");

    let pipeline = pipe(agent1, agent2);
    let _stored_pipeline = pipeline;
}

/// 测试 14: 验证操作符可以作为函数参数
fn process_pipeline(_pipeline: AgentPipeline) {
    // 处理 pipeline
}

#[test]
fn test_operators_as_function_parameters() {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");

    let pipeline = pipe(agent1, agent2);
    process_pipeline(pipeline);
}

/// 测试 15: 验证操作符可以作为函数返回值
fn create_pipeline() -> AgentPipeline {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");

    pipe(agent1, agent2)
}

#[test]
fn test_operators_as_function_return_values() {
    let _pipeline = create_pipeline();
}

/// 测试 16: 验证 delegation 的任务描述可以是变量
#[test]
fn test_delegation_with_variable_task() {
    let manager = create_test_agent("manager", "Delegating");
    let worker = create_test_agent("worker", "Working");
    let task = "Complete this important task";

    let _delegation = delegate(manager, worker, task);
}

/// 测试 17: 验证 delegation 的任务描述可以是表达式
#[test]
fn test_delegation_with_expression_task() {
    let manager = create_test_agent("manager", "Delegating");
    let worker = create_test_agent("worker", "Working");

    let task_prefix = "Complete";
    let task_suffix = "task";
    let task = format!("{} this {}", task_prefix, task_suffix);

    let _delegation = delegate(manager, worker, &task);
}

/// 测试 18: 验证操作符类型安全
#[test]
fn test_operator_type_safety() {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");

    let pipeline: AgentPipeline = pipe(agent1, agent2);
    let _ = pipeline;
}

/// 测试 19: 验证操作符可以组合使用
#[test]
fn test_operator_composition() {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");
    let agent3 = create_test_agent("agent3", "Result 1");
    let agent4 = create_test_agent("agent4", "Result 2");

    // 创建 pipeline
    let _pipeline = pipe(agent1, agent2);

    // 创建 parallel
    let _parallel = parallel(agent3, agent4);
}

/// 测试 20: 验证操作符的构造器模式
#[test]
fn test_operator_builder_pattern() {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");
    let agent3 = create_test_agent("agent3", "Step 3");

    // 使用构造器模式
    let _pipeline = AgentPipeline::new(agent1).pipe(agent2).pipe(agent3);
}

/// 测试 21: 验证 parallel 的构造器模式
#[test]
fn test_parallel_builder_pattern() {
    let agent1 = create_test_agent("agent1", "Result 1");
    let agent2 = create_test_agent("agent2", "Result 2");
    let agent3 = create_test_agent("agent3", "Result 3");

    // 使用构造器模式
    let _parallel = AgentParallel::new(agent1).parallel(agent2).parallel(agent3);
}

/// 测试 22: 验证操作符可以在结构体中使用
struct WorkflowContainer {
    pipeline: AgentPipeline,
}

#[test]
fn test_operators_in_structs() {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");

    let _container = WorkflowContainer {
        pipeline: pipe(agent1, agent2),
    };
}

/// 测试 23: 验证操作符可以在枚举中使用
enum WorkflowType {
    Pipeline(AgentPipeline),
    Parallel(AgentParallel),
}

#[test]
fn test_operators_in_enums() {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");

    let _workflow1 = WorkflowType::Pipeline(pipe(agent1.clone(), agent2.clone()));
    let _workflow2 = WorkflowType::Parallel(parallel(agent1, agent2));
}

/// 测试 24: 验证操作符可以在 Vec 中使用
#[test]
fn test_operators_in_vec() {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");
    let agent3 = create_test_agent("agent3", "Step 3");
    let agent4 = create_test_agent("agent4", "Step 4");

    let _pipelines = vec![pipe(agent1, agent2), pipe(agent3, agent4)];
}

/// 测试 25: 验证操作符可以在 Option 中使用
#[test]
fn test_operators_in_option() {
    let agent1 = create_test_agent("agent1", "Step 1");
    let agent2 = create_test_agent("agent2", "Step 2");

    let _maybe_pipeline: Option<AgentPipeline> = Some(pipe(agent1, agent2));
}
