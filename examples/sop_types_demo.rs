//! # SOP 类型系统演示
//!
//! 本示例展示 SOP 核心类型的使用，不涉及运行时创建。
//!
//! ## 核心类型
//!
//! 1. **SopMessage**：SOP 消息类型
//! 2. **AgentAction**：Agent 行动类型
//! 3. **SopExecutionMode**：执行模式
//! 4. **SopStats**：统计信息
//!
//! ## 设计理念
//!
//! SOP 机制与 Crew 深度融合：
//! - 复用 AgentCommunicationManager 进行消息路由
//! - 复用 Crew 的 Agent 管理
//! - SopMessage ↔ AgentMessage 自动转换
//!
//! ## 运行方式
//!
//! ```bash
//! cargo run --example sop_types_demo
//! ```

use lumosai_core::agent::{AgentAction, SopExecutionMode, SopMessage, SopStats};
use lumosai_core::error::Result;
use serde_json::json;

fn main() -> Result<()> {
    println!("\n{}", "=".repeat(80));
    println!("🚀 SOP 类型系统演示");
    println!("{}", "=".repeat(80));

    // ========================================
    // 第一部分：SopMessage 类型
    // ========================================
    println!("\n📦 第一步：SopMessage 类型");
    println!("{}", "-".repeat(80));

    // 创建广播消息
    let broadcast_msg = SopMessage::broadcast(
        "research_request",
        "system",
        json!({
            "topic": "AI Agent 协作机制",
            "requirements": [
                "调研现有的 AI Agent 协作框架",
                "分析 SOP 机制的优势",
                "提出改进建议"
            ]
        }),
    );

    println!("✅ 广播消息创建完成");
    println!("   - 消息类型: {}", broadcast_msg.msg_type);
    println!("   - 发送者: {}", broadcast_msg.sender);
    println!("   - 接收者: {:?}", broadcast_msg.receiver);
    println!("   - 内容: {}", broadcast_msg.content);

    // 创建点对点消息
    let p2p_msg = SopMessage::new(
        "task_assignment",
        "manager",
        Some("researcher".to_string()),
        json!({
            "task": "调研 MetaGPT SOP 机制",
            "deadline": "2024-01-15"
        }),
    );

    println!("\n✅ 点对点消息创建完成");
    println!("   - 消息类型: {}", p2p_msg.msg_type);
    println!("   - 发送者: {}", p2p_msg.sender);
    println!("   - 接收者: {:?}", p2p_msg.receiver);

    // ========================================
    // 第二部分：AgentAction 类型
    // ========================================
    println!("\n📦 第二步：AgentAction 类型");
    println!("{}", "-".repeat(80));

    let actions = vec![
        (AgentAction::NoOp, "无操作"),
        (
            AgentAction::Send {
                msg_type: "test".to_string(),
                receiver: None,
                content: json!({}),
            },
            "发送消息",
        ),
        (
            AgentAction::Reply {
                content: "需要更多信息".to_string(),
            },
            "请求信息",
        ),
        (
            AgentAction::delegate(
                "agent2",
                "task1",
                json!({}),
            ),
            "委托任务",
        ),
        (
            AgentAction::finish(json!({"result": "success"})),
            "完成任务",
        ),
        (
            AgentAction::wait("等待5秒"),
            "等待",
        ),
        (
            AgentAction::Reply {
                content: "发生错误".to_string(),
            },
            "错误",
        ),
    ];

    for (action, desc) in actions {
        println!("   - {}: {:?}", desc, action);
    }

    // ========================================
    // 第三部分：SopExecutionMode
    // ========================================
    println!("\n📦 第三步：SopExecutionMode");
    println!("{}", "-".repeat(80));

    let modes = vec![
        (
            SopExecutionMode::React,
            "事件驱动模式",
            "Agent 根据接收到的消息做出反应",
        ),
        (
            SopExecutionMode::ByOrder,
            "顺序执行模式",
            "Agent 按照预定义的顺序执行",
        ),
        (
            SopExecutionMode::PlanAndAct,
            "计划执行模式",
            "Agent 先制定计划，然后执行",
        ),
    ];

    for (mode, name, desc) in modes {
        println!("   - {:?} ({}): {}", mode, name, desc);
    }

    // ========================================
    // 第四部分：SopStats
    // ========================================
    println!("\n📦 第四步：SopStats");
    println!("{}", "-".repeat(80));

    let mut stats = SopStats::default();
    stats.total_messages = 10;
    stats.active_agents = 3;
    stats
        .message_types
        .insert("research_request".to_string(), 1);
    stats.message_types.insert("task_assignment".to_string(), 5);
    stats.message_types.insert("status_update".to_string(), 4);

    println!("\n📊 SOP 执行统计:");
    println!("   - 总消息数: {}", stats.total_messages);
    println!("   - 活跃 Agent 数: {}", stats.active_agents);
    println!("   - 消息类型分布:");
    for (msg_type, count) in &stats.message_types {
        println!("     * {}: {}", msg_type, count);
    }

    // ========================================
    // 第五部分：融合优势说明
    // ========================================
    println!("\n{}", "=".repeat(80));
    println!("💡 SOP 与 Crew 融合优势");
    println!("{}", "=".repeat(80));

    println!("\n1. **复用现有基础设施**");
    println!("   - 使用 AgentCommunicationManager 进行消息路由");
    println!("   - 使用 Crew 的 Agent 管理功能");
    println!("   - 使用 SessionManager 管理会话");

    println!("\n2. **消息桥接**");
    println!("   - SopMessage ↔ AgentMessage 自动转换");
    println!("   - 保留原始 SOP 类型信息在 metadata 中");
    println!("   - 支持 12 种 AgentMessageType 映射");

    println!("\n3. **向后兼容**");
    println!("   - 现有 Agent 无需修改即可参与 SOP 工作流");
    println!("   - 可选的 SOP 方法（sop_watch/think/act/is_done）");
    println!("   - 渐进式迁移路径");

    println!("\n4. **下一步计划**");
    println!("   - 扩展 Crew API 支持 SOP 执行模式");
    println!("   - 实现 #[agent] 宏自动生成 SOP 方法");
    println!("   - 添加更多集成测试和示例");

    println!("\n{}", "=".repeat(80));
    println!("✅ 示例完成！");
    println!("{}", "=".repeat(80));

    Ok(())
}
