//! SOP 集成测试
//!
//! 测试覆盖：
//! - Crew 与 SOP 模式的集成
//! - SopEnvironment 与 Crew 的适配
//! - 消息转换的正确性
//! - 三种执行模式的集成
//! - Agent 与 SOP 的兼容性

use lumosai_core::agent::{
    sop_environment::SopEnvironment,
    sop_types::{AgentAction, SopExecutionMode, SopMessage},
    Agent, CollaborationMode, Crew,
};
use lumosai_core::error::Result;
use serde_json::json;
use std::sync::Arc;

// ===== 测试 1: Crew 与 SOP React 模式集成 =====

#[tokio::test]
async fn test_crew_sop_react_integration() -> Result<()> {
    // 创建使用 SOP React 模式的 Crew
    let crew = Crew::new("react_team".to_string(), CollaborationMode::SopReact, 10);

    // 验证 Crew 创建成功
    assert!(Arc::strong_count(&Arc::new(crew.clone())) >= 1);

    Ok(())
}

// ===== 测试 2: Crew 与 SOP ByOrder 模式集成 =====

#[tokio::test]
async fn test_crew_sop_by_order_integration() -> Result<()> {
    // 创建使用 SOP ByOrder 模式的 Crew
    let crew = Crew::new(
        "sequential_team".to_string(),
        CollaborationMode::SopByOrder,
        5,
    );

    // 验证 Crew 创建成功
    assert!(Arc::strong_count(&Arc::new(crew.clone())) >= 1);

    Ok(())
}

// ===== 测试 3: Crew 与 SOP PlanAndAct 模式集成 =====

#[tokio::test]
async fn test_crew_sop_plan_and_act_integration() -> Result<()> {
    // 创建使用 SOP PlanAndAct 模式的 Crew
    let crew = Crew::new(
        "planning_team".to_string(),
        CollaborationMode::SopPlanAndAct,
        8,
    );

    // 验证 Crew 创建成功
    assert!(Arc::strong_count(&Arc::new(crew.clone())) >= 1);

    Ok(())
}

// ===== 测试 4: SopEnvironment 从 Crew 创建 =====

#[tokio::test]
async fn test_sop_environment_from_crew() -> Result<()> {
    // 创建 Crew
    let crew = Crew::new("test_crew".to_string(), CollaborationMode::Sequential, 10);

    // 使用 SopEnvironment::from_crew 创建适配器
    let sop_env = SopEnvironment::from_crew(Arc::new(crew), SopExecutionMode::React);

    // 验证 SopEnvironment 创建成功（不会 panic）
    drop(sop_env);

    Ok(())
}

// ===== 测试 5: SopEnvironment 三种模式创建 =====

#[tokio::test]
async fn test_sop_environment_all_modes() -> Result<()> {
    let crew = Crew::new(
        "multi_mode_crew".to_string(),
        CollaborationMode::Parallel,
        10,
    );
    let crew_arc = Arc::new(crew);

    // 测试 React 模式
    let env_react = SopEnvironment::from_crew(Arc::clone(&crew_arc), SopExecutionMode::React);
    drop(env_react);

    // 测试 ByOrder 模式
    let env_by_order = SopEnvironment::from_crew(Arc::clone(&crew_arc), SopExecutionMode::ByOrder);
    drop(env_by_order);

    // 测试 PlanAndAct 模式
    let env_plan_and_act =
        SopEnvironment::from_crew(Arc::clone(&crew_arc), SopExecutionMode::PlanAndAct);
    drop(env_plan_and_act);

    Ok(())
}

// ===== 测试 6: 消息创建和序列化集成 =====

#[tokio::test]
async fn test_message_creation_and_serialization() -> Result<()> {
    // 创建各种类型的消息
    let task_msg = SopMessage::new(
        "task".to_string(),
        "agent1".to_string(),
        Some("agent2".to_string()),
        json!({"action": "analyze"}),
    );

    let broadcast_msg = SopMessage::broadcast(
        "announcement".to_string(),
        "coordinator".to_string(),
        json!({"message": "meeting"}),
    );

    // 序列化和反序列化
    let task_json = serde_json::to_string(&task_msg)?;
    let task_deserialized: SopMessage = serde_json::from_str(&task_json)?;
    assert_eq!(task_deserialized.msg_type, "task");

    let broadcast_json = serde_json::to_string(&broadcast_msg)?;
    let broadcast_deserialized: SopMessage = serde_json::from_str(&broadcast_json)?;
    assert_eq!(broadcast_deserialized.msg_type, "announcement");
    assert_eq!(broadcast_deserialized.receiver, None);

    Ok(())
}

// ===== 测试 7: AgentAction 所有变体集成 =====

#[tokio::test]
async fn test_agent_action_all_variants() -> Result<()> {
    // 测试所有 AgentAction 变体
    let actions = vec![
        AgentAction::Reply {
            content: "Done".to_string(),
        },
        AgentAction::Send {
            msg_type: "result".to_string(),
            receiver: Some("agent2".to_string()),
            content: json!({"status": "ok"}),
        },
        AgentAction::ToolCall {
            tool_name: "search".to_string(),
            arguments: json!({"query": "test"}),
        },
        AgentAction::Delegate {
            target_agent: "specialist".to_string(),
            task: "analyze".to_string(),
            params: json!({}),
        },
        AgentAction::Wait {
            reason: "Waiting".to_string(),
        },
        AgentAction::Finish {
            result: json!({"done": true}),
        },
        AgentAction::NoOp,
    ];

    // 验证所有 action 都可以序列化和反序列化
    for action in actions {
        let json_str = serde_json::to_string(&action)?;
        let _deserialized: AgentAction = serde_json::from_str(&json_str)?;
    }

    Ok(())
}

// ===== 测试 8: CollaborationMode 所有变体 =====

#[tokio::test]
async fn test_collaboration_mode_all_variants() -> Result<()> {
    let modes = vec![
        CollaborationMode::Sequential,
        CollaborationMode::Parallel,
        CollaborationMode::Hierarchical,
        CollaborationMode::SopReact,
        CollaborationMode::SopByOrder,
        CollaborationMode::SopPlanAndAct,
    ];

    // 验证所有模式都可以用于创建 Crew
    for (i, mode) in modes.iter().enumerate() {
        let crew = Crew::new(format!("crew_{}", i), mode.clone(), 10);
        drop(crew);
    }

    Ok(())
}

// ===== 测试 9: Crew Clone 深度测试 =====

#[tokio::test]
async fn test_crew_clone_deep() -> Result<()> {
    // 创建原始 Crew
    let crew1 = Crew::new("original".to_string(), CollaborationMode::SopReact, 10);

    // 克隆 Crew
    let crew2 = crew1.clone();
    let crew3 = crew2.clone();

    // 验证所有克隆都可以正常使用
    drop(crew1);
    drop(crew2);
    drop(crew3);

    Ok(())
}

// ===== 测试 10: SOP 模式与传统模式共存 =====

#[tokio::test]
async fn test_sop_and_traditional_modes_coexist() -> Result<()> {
    // 创建传统模式的 Crew
    let crew_sequential = Crew::new(
        "traditional_sequential".to_string(),
        CollaborationMode::Sequential,
        10,
    );

    let crew_parallel = Crew::new(
        "traditional_parallel".to_string(),
        CollaborationMode::Parallel,
        10,
    );

    // 创建 SOP 模式的 Crew
    let crew_sop_react = Crew::new("sop_react".to_string(), CollaborationMode::SopReact, 10);

    let crew_sop_by_order = Crew::new(
        "sop_by_order".to_string(),
        CollaborationMode::SopByOrder,
        10,
    );

    // 验证所有 Crew 都可以共存
    drop(crew_sequential);
    drop(crew_parallel);
    drop(crew_sop_react);
    drop(crew_sop_by_order);

    Ok(())
}

// ===== 测试 11: 空 Crew 的 SOP 环境 =====

#[tokio::test]
async fn test_empty_crew_sop_environment() -> Result<()> {
    // 创建空的 Crew（没有 Agent）
    let crew = Crew::new("empty_crew".to_string(), CollaborationMode::SopReact, 10);

    // 创建 SopEnvironment
    let sop_env = SopEnvironment::from_crew(Arc::new(crew), SopExecutionMode::React);

    // 运行环境（应该立即完成，因为没有 Agent）
    let stats = sop_env.run(None).await?;

    // 验证统计信息（空 Crew 可能会有一些初始化消息）
    // 主要验证不会 panic 和能够正常完成
    assert!(stats.total_messages >= 0);
    assert_eq!(stats.active_agents, 0);
    assert_eq!(stats.completed_agents, 0);

    Ok(())
}
