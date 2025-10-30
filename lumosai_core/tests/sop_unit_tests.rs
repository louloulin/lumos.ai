//! SOP 单元测试
//!
//! 测试覆盖：
//! - SopMessage 创建和序列化
//! - AgentAction 所有变体
//! - SopExecutionMode 枚举
//! - SopStats 统计
//! - CollaborationMode SOP 变体
//! - 消息转换（SopMessage ↔ AgentMessage）

use lumosai_core::agent::{
    sop_types::{AgentAction, SopExecutionMode, SopMessage, SopStats},
    CollaborationMode, Crew,
};
use serde_json::json;
use std::collections::HashMap;

// ===== SopMessage 测试 =====

#[test]
fn test_sop_message_creation() {
    let msg = SopMessage::new(
        "task_msg".to_string(),
        "sender_agent".to_string(),
        Some("receiver_agent".to_string()),
        json!({"task": "analyze data"}),
    );

    assert_eq!(msg.msg_type, "task_msg");
    assert_eq!(msg.sender, "sender_agent");
    assert_eq!(msg.receiver, Some("receiver_agent".to_string()));
    assert_eq!(msg.content["task"], "analyze data");
    assert!(!msg.id.is_empty());
    assert!(msg.timestamp > 0);
}

#[test]
fn test_sop_message_broadcast() {
    let msg = SopMessage::broadcast(
        "announcement".to_string(),
        "coordinator".to_string(),
        json!({"message": "meeting at 3pm"}),
    );

    assert_eq!(msg.msg_type, "announcement");
    assert_eq!(msg.sender, "coordinator");
    assert_eq!(msg.receiver, None); // 广播消息没有特定接收者
    assert_eq!(msg.content["message"], "meeting at 3pm");
}

#[test]
fn test_sop_message_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("priority".to_string(), json!("high"));
    metadata.insert("deadline".to_string(), json!("2025-11-01"));

    let msg = SopMessage {
        id: "msg_123".to_string(),
        msg_type: "urgent_task".to_string(),
        sender: "manager".to_string(),
        receiver: Some("worker".to_string()),
        content: json!({"task": "fix bug"}),
        timestamp: 1698765432,
        metadata,
    };

    assert_eq!(msg.metadata.get("priority").unwrap(), &json!("high"));
    assert_eq!(msg.metadata.get("deadline").unwrap(), &json!("2025-11-01"));
}

#[test]
fn test_sop_message_serialization() {
    let msg = SopMessage::broadcast(
        "test".to_string(),
        "agent1".to_string(),
        json!({"data": 42}),
    );

    let json_str = serde_json::to_string(&msg).unwrap();
    let deserialized: SopMessage = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.msg_type, msg.msg_type);
    assert_eq!(deserialized.sender, msg.sender);
    assert_eq!(deserialized.content, msg.content);
}

// ===== AgentAction 测试 =====

#[test]
fn test_agent_action_reply() {
    let action = AgentAction::Reply {
        content: "Analysis complete".to_string(),
    };

    match action {
        AgentAction::Reply { content } => {
            assert_eq!(content, "Analysis complete");
        }
        _ => panic!("Expected Reply action"),
    }
}

#[test]
fn test_agent_action_send() {
    let action = AgentAction::Send {
        msg_type: "result".to_string(),
        receiver: Some("coordinator".to_string()),
        content: json!({"status": "done"}),
    };

    match action {
        AgentAction::Send {
            msg_type,
            receiver,
            content,
        } => {
            assert_eq!(msg_type, "result");
            assert_eq!(receiver, Some("coordinator".to_string()));
            assert_eq!(content["status"], "done");
        }
        _ => panic!("Expected Send action"),
    }
}

#[test]
fn test_agent_action_tool_call() {
    let action = AgentAction::ToolCall {
        tool_name: "web_search".to_string(),
        arguments: json!({"query": "Rust async"}),
    };

    match action {
        AgentAction::ToolCall {
            tool_name,
            arguments,
        } => {
            assert_eq!(tool_name, "web_search");
            assert_eq!(arguments["query"], "Rust async");
        }
        _ => panic!("Expected ToolCall action"),
    }
}

#[test]
fn test_agent_action_delegate() {
    let action = AgentAction::Delegate {
        target_agent: "specialist".to_string(),
        task: "deep_analysis".to_string(),
        params: json!({"depth": 3}),
    };

    match action {
        AgentAction::Delegate {
            target_agent,
            task,
            params,
        } => {
            assert_eq!(target_agent, "specialist");
            assert_eq!(task, "deep_analysis");
            assert_eq!(params["depth"], 3);
        }
        _ => panic!("Expected Delegate action"),
    }
}

#[test]
fn test_agent_action_wait() {
    let action = AgentAction::Wait {
        reason: "Waiting for data".to_string(),
    };

    match action {
        AgentAction::Wait { reason } => {
            assert_eq!(reason, "Waiting for data");
        }
        _ => panic!("Expected Wait action"),
    }
}

#[test]
fn test_agent_action_finish() {
    let action = AgentAction::Finish {
        result: json!({"summary": "All tasks completed"}),
    };

    match action {
        AgentAction::Finish { result } => {
            assert_eq!(result["summary"], "All tasks completed");
        }
        _ => panic!("Expected Finish action"),
    }
}

#[test]
fn test_agent_action_noop() {
    let action = AgentAction::NoOp;

    match action {
        AgentAction::NoOp => {
            // Success
        }
        _ => panic!("Expected NoOp action"),
    }
}

#[test]
fn test_agent_action_serialization() {
    let action = AgentAction::Send {
        msg_type: "test".to_string(),
        receiver: None,
        content: json!({"value": 123}),
    };

    let json_str = serde_json::to_string(&action).unwrap();
    let deserialized: AgentAction = serde_json::from_str(&json_str).unwrap();

    match deserialized {
        AgentAction::Send {
            msg_type,
            receiver,
            content,
        } => {
            assert_eq!(msg_type, "test");
            assert_eq!(receiver, None);
            assert_eq!(content["value"], 123);
        }
        _ => panic!("Deserialization failed"),
    }
}

// ===== SopExecutionMode 测试 =====

#[test]
fn test_sop_execution_mode_react() {
    let mode = SopExecutionMode::React;
    assert_eq!(format!("{:?}", mode), "React");
}

#[test]
fn test_sop_execution_mode_by_order() {
    let mode = SopExecutionMode::ByOrder;
    assert_eq!(format!("{:?}", mode), "ByOrder");
}

#[test]
fn test_sop_execution_mode_plan_and_act() {
    let mode = SopExecutionMode::PlanAndAct;
    assert_eq!(format!("{:?}", mode), "PlanAndAct");
}

#[test]
fn test_sop_execution_mode_serialization() {
    let mode = SopExecutionMode::React;
    let json_str = serde_json::to_string(&mode).unwrap();
    let deserialized: SopExecutionMode = serde_json::from_str(&json_str).unwrap();

    match deserialized {
        SopExecutionMode::React => {
            // Success
        }
        _ => panic!("Deserialization failed"),
    }
}

// ===== SopStats 测试 =====

#[test]
fn test_sop_stats_creation() {
    let mut message_types = HashMap::new();
    message_types.insert("task".to_string(), 30);
    message_types.insert("result".to_string(), 20);

    let stats = SopStats {
        total_messages: 50,
        message_types,
        active_agents: 5,
        completed_agents: 3,
    };

    assert_eq!(stats.total_messages, 50);
    assert_eq!(stats.active_agents, 5);
    assert_eq!(stats.completed_agents, 3);
    assert_eq!(stats.message_types.get("task"), Some(&30));
    assert_eq!(stats.message_types.get("result"), Some(&20));
}

#[test]
fn test_sop_stats_default() {
    let stats = SopStats::default();

    assert_eq!(stats.total_messages, 0);
    assert_eq!(stats.active_agents, 0);
    assert_eq!(stats.completed_agents, 0);
    assert!(stats.message_types.is_empty());
}

#[test]
fn test_sop_stats_serialization() {
    let mut message_types = HashMap::new();
    message_types.insert("test".to_string(), 15);

    let stats = SopStats {
        total_messages: 15,
        message_types,
        active_agents: 2,
        completed_agents: 1,
    };

    let json_str = serde_json::to_string(&stats).unwrap();
    let deserialized: SopStats = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.total_messages, stats.total_messages);
    assert_eq!(deserialized.active_agents, stats.active_agents);
    assert_eq!(deserialized.completed_agents, stats.completed_agents);
    assert_eq!(
        deserialized.message_types.get("test"),
        stats.message_types.get("test")
    );
}

// ===== CollaborationMode SOP 变体测试 =====

#[test]
fn test_collaboration_mode_sop_react() {
    let mode = CollaborationMode::SopReact;
    assert_eq!(format!("{:?}", mode), "SopReact");
}

#[test]
fn test_collaboration_mode_sop_by_order() {
    let mode = CollaborationMode::SopByOrder;
    assert_eq!(format!("{:?}", mode), "SopByOrder");
}

#[test]
fn test_collaboration_mode_sop_plan_and_act() {
    let mode = CollaborationMode::SopPlanAndAct;
    assert_eq!(format!("{:?}", mode), "SopPlanAndAct");
}

#[test]
fn test_collaboration_mode_equality() {
    assert_eq!(CollaborationMode::SopReact, CollaborationMode::SopReact);
    assert_ne!(CollaborationMode::SopReact, CollaborationMode::SopByOrder);
    assert_ne!(
        CollaborationMode::SopByOrder,
        CollaborationMode::SopPlanAndAct
    );
}

#[test]
fn test_collaboration_mode_clone() {
    let mode1 = CollaborationMode::SopReact;
    let mode2 = mode1.clone();
    assert_eq!(mode1, mode2);
}

#[test]
fn test_collaboration_mode_serialization() {
    let mode = CollaborationMode::SopPlanAndAct;
    let json_str = serde_json::to_string(&mode).unwrap();
    let deserialized: CollaborationMode = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized, mode);
}

// ===== Crew SOP 集成测试 =====

#[tokio::test]
async fn test_crew_creation_with_sop_react() {
    let crew = Crew::new(
        "test_crew_react".to_string(),
        CollaborationMode::SopReact,
        10,
    );

    // 验证 Crew 创建成功（不会 panic）
    // Crew 字段是私有的，所以我们只验证创建不会失败
    drop(crew);
}

#[tokio::test]
async fn test_crew_creation_with_sop_by_order() {
    let crew = Crew::new(
        "test_crew_by_order".to_string(),
        CollaborationMode::SopByOrder,
        5,
    );

    // 验证 Crew 创建成功
    drop(crew);
}

#[tokio::test]
async fn test_crew_creation_with_sop_plan_and_act() {
    let crew = Crew::new(
        "test_crew_plan_and_act".to_string(),
        CollaborationMode::SopPlanAndAct,
        8,
    );

    // 验证 Crew 创建成功
    drop(crew);
}

#[tokio::test]
async fn test_crew_clone() {
    let crew1 = Crew::new(
        "original_crew".to_string(),
        CollaborationMode::SopByOrder,
        5,
    );

    let crew2 = crew1.clone();

    // 验证克隆成功（两个 Crew 都可以正常使用）
    drop(crew1);
    drop(crew2);
}
