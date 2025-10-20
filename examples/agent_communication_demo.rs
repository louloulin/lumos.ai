//! Agent通信系统演示
//!
//! 展示Agent之间的消息传递、会话管理、主题订阅等高级通信功能

use lumosai_core::agent::communication::{
    AgentCommunicationManager, AgentMessage, AgentMessageType, MessagePriority,
    AgentInfo, AgentStatus, AgentLoadInfo, SessionType, SessionMetadata,
    CommunicationConfig, RoutingStrategy
};
use lumosai_core::llm::MockLlmProvider;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Agent通信系统演示");
    println!("==================");

    // 创建Mock LLM提供商
    let llm = Arc::new(MockLlmProvider::new(vec![
        "消息已处理".to_string(),
        "任务已接受".to_string(),
        "协作请求已确认".to_string(),
        "会话消息已处理".to_string(),
    ]));

    // 创建通信配置
    let config = CommunicationConfig {
        max_message_history: 1000,
        default_message_ttl: 600,
        enable_persistence: false,
        max_concurrent_messages: 100,
        enable_advanced_routing: true,
        enable_session_management: true,
        enable_pubsub: true,
        default_routing_strategy: RoutingStrategy::Direct,
    };

    // 创建通信管理器
    let comm_manager = Arc::new(AgentCommunicationManager::new(config));

    // 第一部分：Agent注册和基本信息
    println!("\n📋 第一部分：Agent注册和基本信息");
    println!("=====================================");

    // 创建多个Agent
    let agents_data = vec![
        (
            "orchestrator",
            "orchestrator_agent",
            AgentStatus::Active,
            vec!["协调".to_string(), "任务分发".to_string(), "监控".to_string()],
        ),
        (
            "developer", 
            "developer_agent",
            AgentStatus::Active,
            vec!["编程".to_string(), "调试".to_string(), "测试".to_string()],
        ),
        (
            "analyzer",
            "analyzer_agent", 
            AgentStatus::Active,
            vec!["数据分析".to_string(), "报告生成".to_string(), "可视化".to_string()],
        ),
    ];

    // 注册Agent
    for (agent_id, name, status, capabilities) in agents_data {
        let agent_info = AgentInfo {
            agent_id: agent_id.to_string(),
            name: name.to_string(),
            status,
            capabilities,
            load_info: AgentLoadInfo {
                cpu_usage: 20.0,
                memory_usage: 30.0,
                current_tasks: 0,
                max_tasks: 10,
                avg_response_time: 100.0,
            },
            metadata: std::collections::HashMap::new(),
            registered_at: chrono::Utc::now(),
            last_active: chrono::Utc::now(),
        };

        if let Err(e) = comm_manager.register_agent(agent_id.to_string(), agent_info).await {
            eprintln!("注册Agent {} 失败: {:?}", agent_id, e);
        } else {
            println!("✅ 成功注册Agent: {}", agent_id);
        }
    }

    // 显示通信统计
    let stats = comm_manager.get_stats().await;
    println!("\n📊 通信统计:");
    println!("  - 总Agent数: {}", stats.total_agents);
    println!("  - 活跃Agent数: {}", stats.active_agents);

    // 第二部分：基础消息传递
    println!("\n💬 第二部分：基础消息传递");
    println!("========================");

    // 创建并发送不同类型的消息
    let message_types = vec![
        (AgentMessageType::Request, "请求执行数据分析任务", MessagePriority::High),
        (AgentMessageType::Notification, "系统维护通知", MessagePriority::Normal),
        (AgentMessageType::Collaboration, "请求协作完成项目", MessagePriority::Urgent),
        (AgentMessageType::StatusUpdate, "状态更新：任务进行中", MessagePriority::Low),
    ];

    for (i, (msg_type, content, priority)) in message_types.iter().enumerate() {
        let message = AgentMessage::new(
            "orchestrator".to_string(),
            vec!["developer".to_string(), "analyzer".to_string()],
            msg_type.clone(),
            content.to_string(),
        )
        .with_priority(priority.clone())
        .with_expected_output(format!("处理{}类型的消息", i + 1));

        match comm_manager.send_message(message).await {
            Ok(_) => println!("✅ 消息发送成功: {:?}", msg_type),
            Err(e) => eprintln!("❌ 消息发送失败: {:?}", e),
        }
    }

    sleep(Duration::from_millis(100)).await;

    // 第三部分：会话管理演示
    println!("\n🤝 第三部分：会话管理演示");
    println!("======================");

    // 创建一个头脑风暴会话
    let session_metadata = SessionMetadata {
        title: "项目头脑风暴".to_string(),
        description: Some("讨论新项目的技术方案".to_string()),
        tags: vec!["brainstorm".to_string(), "project".to_string()],
        custom_attributes: std::collections::HashMap::new(),
    };

    let session_id = comm_manager.create_session(
        SessionType::Brainstorming,
        vec!["orchestrator".to_string(), "developer".to_string(), "analyzer".to_string()],
        session_metadata,
    ).await?;

    println!("✅ 创建会话成功: {}", session_id);

    // 发送会话消息
    let session_message = AgentMessage::new(
        "orchestrator".to_string(),
        vec![],
        AgentMessageType::SessionMessage,
        "让我们开始头脑风暴，讨论项目的技术架构方案".to_string(),
    )
    .with_session_id(session_id.clone())
    .with_priority(MessagePriority::Normal);

    if let Err(e) = comm_manager.send_message(session_message).await {
        eprintln!("❌ 会话消息发送失败: {:?}", e);
    } else {
        println!("✅ 会话消息发送成功");
    }

    // 获取活跃会话列表
    let active_sessions = comm_manager.get_active_sessions().await;
    println!("\n📋 活跃会话列表:");
    for session in active_sessions {
        println!("  - {} (类型: {:?}, 参与者: {})", 
                session.title, session.session_type, session.participants.len());
    }

    // 第四部分：主题订阅演示
    println!("\n📢 第四部分：主题订阅演示");
    println!("======================");

    // 让不同Agent订阅不同主题
    let subscriptions = vec![
        ("developer", "development", None),
        ("analyzer", "analysis", None),
        ("developer", "updates", None),
        ("analyzer", "reports", None),
    ];

    for (agent_id, topic, filter) in subscriptions {
        if let Err(e) = comm_manager.subscribe(
            agent_id.to_string(), 
            topic.to_string(), 
            filter
        ).await {
            eprintln!("❌ {} 订阅主题 {} 失败: {:?}", agent_id, topic, e);
        } else {
            println!("✅ {} 订阅主题 {} 成功", agent_id, topic);
        }
    }

    sleep(Duration::from_millis(50)).await;

    // 发布主题消息
    let topic_messages = vec![
        ("development", "新功能开发完成", MessagePriority::High),
        ("analysis", "数据分析报告已生成", MessagePriority::Normal),
        ("updates", "系统更新可用", MessagePriority::Urgent),
    ];

    for (topic, content, priority) in topic_messages {
        let topic_message = AgentMessage::new(
            "orchestrator".to_string(),
            vec![],
            AgentMessageType::TopicMessage,
            content.to_string(),
        )
        .with_topic(topic.to_string())
        .with_priority(priority);

        if let Err(e) = comm_manager.send_message(topic_message).await {
            eprintln!("❌ 发布主题消息失败: {:?}", e);
        } else {
            println!("✅ 发布主题消息到 {}: {}", topic, content);
        }
    }

    sleep(Duration::from_millis(100)).await;

    // 第五部分：高级功能演示
    println!("\n⚡ 第五部分：高级功能演示");
    println!("======================");

    // 任务委派演示
    let task_id = comm_manager.delegate_task(
        "orchestrator".to_string(),
        "developer".to_string(),
        "实现新的API接口".to_string(),
        Some(chrono::Utc::now() + chrono::Duration::hours(2)),
    ).await?;

    println!("✅ 任务委派成功，任务ID: {}", task_id);

    // 资源共享演示
    let resource_data = b"shared resource data".to_vec();
    if let Err(e) = comm_manager.share_resource(
        "analyzer".to_string(),
        "analysis_result".to_string(),
        resource_data,
        vec!["developer".to_string()],
    ).await {
        eprintln!("❌ 资源共享失败: {:?}", e);
    } else {
        println!("✅ 资源共享成功");
    }

    // 消息历史查询演示
    let history_filters = lumosai_core::agent::communication::MessageHistoryFilters {
        sender_id: Some("orchestrator".to_string()),
        limit: Some(5),
        ..Default::default()
    };

    let message_history = comm_manager.get_message_history(history_filters).await;
    println!("\n📜 消息历史 (最近5条来自orchestrator的消息):");
    for (i, msg) in message_history.iter().enumerate() {
        println!("  {}. [{}] -> {:?}: {}", 
                i + 1, msg.sender_id, msg.message_type, msg.content);
    }

    // 第六部分：性能和状态监控
    println!("\n📈 第六部分：性能和状态监控");
    println!("==========================");

    // 获取详细统计信息
    let final_stats = comm_manager.get_stats().await;
    println!("📊 最终统计信息:");
    println!("  - 总消息数: {}", final_stats.total_messages);
    println!("  - 总Agent数: {}", final_stats.total_agents);
    println!("  - 活跃Agent数: {}", final_stats.active_agents);
    println!("  - 总会话数: {}", final_stats.total_sessions);
    println!("  - 平均投递时间: {}ms", final_stats.average_delivery_time);

    // 队列状态
    let queue_status = comm_manager.get_queue_status().await;
    println!("\n🚀 队列状态:");
    println!("  - 总消息数: {}", queue_status.total_messages);
    println!("  - 待处理消息: {}", queue_status.pending_messages);
    println!("  - 优先级消息: {}", queue_status.priority_messages);
    println!("  - 广播消息: {}", queue_status.broadcast_messages);
    println!("  - 队列健康状态: {}", if queue_status.is_healthy { "健康" } else { "异常" });

    // 清理工作
    println!("\n🧹 清理工作...");
    
    // 注销所有Agent
    for (agent_id, _, _, _) in agents_data {
        if let Err(e) = comm_manager.unregister_agent(agent_id).await {
            eprintln!("❌ 注销Agent {} 失败: {:?}", agent_id, e);
        } else {
            println!("✅ 注销Agent: {}", agent_id);
        }
    }

    // 清理过期会话
    comm_manager.cleanup_expired_sessions().await;
    println!("✅ 清理过期会话完成");

    println!("\n🎉 Agent通信系统演示完成！");
    println!("========================");
    println!("✅ 已演示功能:");
    println!("  🤝 Agent注册和管理");
    println!("  💬 基础消息传递（支持优先级和多接收者）");
    println!("  🤝 会话管理（创建、消息传递、清理）");
    println!("  📢 主题订阅和发布");
    println!("  ⚡ 高级功能（任务委派、资源共享）");
    println!("  📜 消息历史查询");
    println!("  📈 性能监控和统计");
    println!("\n🚀 Agent通信系统已准备好支持复杂的多Agent协作场景！");

    Ok(())
}