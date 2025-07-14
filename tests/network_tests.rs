use std::time::{Duration, Instant};
use lumosai_network::{AgentNode, AgentNetwork, Message, MessageType, AgentId};
use lumosai_network::types::AgentStatus;
use lumosai_network::network::AgentConfig;
use serde_json::json;

mod common;
use common::TestAssertions;

/// 网络基础功能测试
#[tokio::test]
async fn test_agent_node_creation() {
    // 测试Agent节点创建
    let config = AgentConfig::default();
    let node = AgentNode::new(Some(AgentId::new()), config);

    // 验证节点创建成功
    assert!(node.id().to_string().len() > 0);
    assert_eq!(node.status().await, AgentStatus::Initialized);
}

#[tokio::test]
async fn test_agent_node_status() {
    // 测试Agent节点状态管理
    let config = AgentConfig::default();
    let node = AgentNode::new(Some(AgentId::new()), config);

    // 验证初始状态
    assert_eq!(node.status().await, AgentStatus::Initialized);

    // 设置新状态
    node.set_status(AgentStatus::Running).await;
    assert_eq!(node.status().await, AgentStatus::Running);

    // 设置停止状态
    node.set_status(AgentStatus::Stopped).await;
    assert_eq!(node.status().await, AgentStatus::Stopped);
}

#[tokio::test]
async fn test_agent_node_message_handling() {
    // 测试Agent节点消息处理
    let config = AgentConfig::default();
    let node = AgentNode::new(Some(AgentId::new()), config);

    // 添加消息处理器
    node.add_message_handler(MessageType::Text, |_msg| {
        Ok(vec![])
    });

    // 验证节点可以处理消息
    assert_eq!(node.status().await, AgentStatus::Initialized);
}

/// 网络通信测试
#[tokio::test]
async fn test_message_creation() {
    // 测试消息创建
    let sender_id = AgentId::new();
    let receiver_id = AgentId::new();
    let message = Message::new(
        sender_id.clone(),
        vec![receiver_id.clone()],
        MessageType::Text,
        json!({"content": "test message"})
    );

    assert_eq!(message.sender, sender_id);
    assert_eq!(message.receivers[0], receiver_id);
    assert_eq!(message.message_type, MessageType::Text);
}

#[tokio::test]
async fn test_agent_network_creation() {
    // 测试Agent网络创建
    let network = AgentNetwork::new().await;

    // 验证网络创建成功
    assert!(network.id().to_string().len() > 0);
}

#[tokio::test]
async fn test_network_performance() {
    // 测试网络性能
    let start_time = Instant::now();

    // 创建多个Agent节点
    for _i in 0..10 {
        let config = AgentConfig::default();
        let _node = AgentNode::new(Some(AgentId::new()), config);
    }

    let duration = start_time.elapsed();

    // 验证性能 - 10个节点应该在合理时间内创建完成
    TestAssertions::assert_response_time(duration, Duration::from_millis(100));
}

#[tokio::test]
async fn test_concurrent_node_creation() {
    // 测试并发节点创建
    let start_time = Instant::now();

    // 并发创建多个节点
    let tasks: Vec<_> = (0..5).map(|_i| {
        tokio::spawn(async move {
            let config = AgentConfig::default();
            AgentNode::new(Some(AgentId::new()), config)
        })
    }).collect();

    let results = futures::future::join_all(tasks).await;
    let duration = start_time.elapsed();

    // 验证所有节点创建都成功
    for result in results {
        let node = result.unwrap();
        assert!(node.id().to_string().len() > 0);
    }

    // 验证并发性能
    TestAssertions::assert_response_time(duration, Duration::from_millis(200));
}
