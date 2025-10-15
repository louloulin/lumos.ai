//! # LumosAI Network 基础功能演示
//!
//! 本示例演示了 LumosAI Network 模块的基础功能，包括：
//! - Agent 网络创建和管理
//! - 消息路由和传递
//! - 服务发现
//! - 网络拓扑管理

use lumosai_core::prelude::*;
use lumosai_network::{
    AgentCapability, AgentId, AgentStatus, AgentType, Message, MessageType,
    router::DefaultMessageRouter, topology::{GraphTopology, NetworkTopology, TopologyType},
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚀 LumosAI Network 基础功能演示");
    println!("{}", "=".repeat(50));

    // 1. 创建消息路由器
    println!("\n📡 创建消息路由器...");
    let router = Arc::new(DefaultMessageRouter::new());
    println!("✅ 消息路由器创建成功");

    // 2. 演示服务发现功能
    println!("\n🔍 演示服务发现功能...");
    demo_service_discovery().await?;

    // 3. 演示消息路由
    println!("\n📨 演示消息路由功能...");
    demo_message_routing(router.clone()).await?;

    // 4. 演示网络拓扑
    println!("\n🌐 演示网络拓扑管理...");
    demo_network_topology().await?;

    // 5. 演示 Agent 协作
    println!("\n🤝 演示 Agent 协作...");
    demo_agent_collaboration().await?;

    println!("\n✅ 所有网络功能演示完成！");
    Ok(())
}

/// 演示服务发现功能
async fn demo_service_discovery() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("  📋 创建服务发现演示...");

    // 创建几个 Agent 类型和能力的示例
    let agents = vec![
        ("chat-agent", AgentType::Regular, vec![
            AgentCapability::new("text_generation", "生成文本内容"),
        ]),
        ("code-agent", AgentType::Worker, vec![
            AgentCapability::new("code_generation", "生成代码"),
        ]),
        ("data-agent", AgentType::Coordinator, vec![
            AgentCapability::new("data_analysis", "数据分析"),
        ]),
    ];

    println!("  📊 模拟注册 {} 个 Agent:", agents.len());
    for (name, agent_type, capabilities) in &agents {
        let _agent_id = AgentId::from_str(*name);
        println!("  ✅ 注册 Agent: {} (类型: {})", name, agent_type);
        for capability in capabilities {
            println!("    - 能力: {} - {}", capability.name, capability.description);
        }
    }

    // 模拟查询服务
    println!("  🔍 模拟查询所有注册的 Agent...");
    println!("  📊 发现 {} 个 Agent:", agents.len());
    for (name, agent_type, _) in &agents {
        println!("    - {} ({})", name, agent_type);
    }

    // 模拟按能力查询
    println!("  🔍 模拟查询具有代码生成能力的 Agent...");
    let code_agents: Vec<_> = agents.iter()
        .filter(|(_, _, capabilities)| {
            capabilities.iter().any(|cap| cap.name == "code_generation")
        })
        .collect();
    println!("  📊 找到 {} 个代码 Agent:", code_agents.len());
    for (name, _, _) in code_agents {
        println!("    - {}", name);
    }

    Ok(())
}

/// 演示消息路由功能
async fn demo_message_routing(_router: Arc<DefaultMessageRouter>) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("  📋 使用消息路由器...");

    // 创建测试消息
    let messages = vec![
        Message::new(
            AgentId::from_str("sender-1"),
            vec![AgentId::from_str("receiver-1")],
            MessageType::Query,
            "Hello from sender-1",
        ),
        Message::new(
            AgentId::from_str("sender-2"),
            vec![AgentId::from_str("receiver-2")],
            MessageType::Response,
            "Response from sender-2",
        ),
    ];

    // 模拟路由消息
    for message in messages {
        println!("  📤 路由消息: {} -> {:?}",
                message.sender.as_str(),
                message.receivers.iter().map(|r| r.as_str()).collect::<Vec<_>>());
        // 注意：这里只是演示，实际使用需要先注册 Agent
        // router.route(message).await?;
    }

    println!("  ✅ 消息路由演示完成");
    Ok(())
}

/// 演示网络拓扑管理
async fn demo_network_topology() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("  📋 创建网络拓扑管理器...");
    let topology = Arc::new(GraphTopology::new(TopologyType::Mesh));

    // 添加节点
    let nodes = vec![
        AgentId::from_str("node-1"),
        AgentId::from_str("node-2"),
        AgentId::from_str("node-3"),
    ];

    for node in &nodes {
        topology.add_node(node.clone(), None).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        println!("  ✅ 添加节点: {}", node.as_str());
    }

    // 添加连接
    topology.add_edge(&nodes[0], &nodes[1], None).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    topology.add_edge(&nodes[1], &nodes[2], None).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    topology.add_edge(&nodes[2], &nodes[0], None).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    println!("  ✅ 添加网络连接");

    // 查询邻居
    let neighbors = topology.get_neighbors(&nodes[0]).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    println!("  📊 节点 {} 的邻居: {:?}",
            nodes[0].as_str(),
            neighbors.iter().map(|n| n.as_str()).collect::<Vec<_>>());

    Ok(())
}

/// 演示 Agent 协作
async fn demo_agent_collaboration() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("  📋 创建协作场景...");

    // 模拟一个简单的协作场景：
    // 1. 用户 Agent 发送请求
    // 2. 协调 Agent 分配任务
    // 3. 工作 Agent 执行任务
    // 4. 结果汇总

    let agents = vec![
        ("user-agent", AgentStatus::Running),
        ("coordinator-agent", AgentStatus::Running),
        ("worker-agent-1", AgentStatus::Running),
        ("worker-agent-2", AgentStatus::Running),
    ];

    println!("  👥 参与协作的 Agent:");
    for (name, status) in &agents {
        println!("    - {} ({:?})", name, status);
    }

    // 模拟协作流程
    println!("  🔄 开始协作流程...");

    println!("    1️⃣ 用户 Agent 发送请求");
    sleep(Duration::from_millis(100)).await;

    println!("    2️⃣ 协调 Agent 分析请求并分配任务");
    sleep(Duration::from_millis(100)).await;

    println!("    3️⃣ 工作 Agent 1 执行子任务 A");
    sleep(Duration::from_millis(200)).await;

    println!("    4️⃣ 工作 Agent 2 执行子任务 B");
    sleep(Duration::from_millis(200)).await;

    println!("    5️⃣ 协调 Agent 汇总结果");
    sleep(Duration::from_millis(100)).await;

    println!("    6️⃣ 返回最终结果给用户 Agent");

    println!("  ✅ 协作流程完成");
    Ok(())
}
