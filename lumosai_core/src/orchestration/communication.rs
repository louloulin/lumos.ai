//! # Agent 通信协议
//!
//! 定义 Agent 之间的消息传递机制。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{broadcast, RwLock};

/// 消息类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// 请求
    Request,
    /// 响应
    Response,
    /// 通知
    Notification,
    /// 广播
    Broadcast,
    /// 错误
    Error,
}

/// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

/// Agent 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// 消息 ID
    pub id: String,
    /// 发送者 ID
    pub sender_id: String,
    /// 接收者 ID (None 表示广播)
    pub receiver_id: Option<String>,
    /// 消息类型
    pub message_type: MessageType,
    /// 优先级
    pub priority: Priority,
    /// 内容
    pub content: String,
    /// 元数据
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// 时间戳
    pub timestamp: u64,
}

impl AgentMessage {
    /// 创建新消息
    pub fn new(
        sender_id: impl Into<String>,
        receiver_id: Option<String>,
        content: impl Into<String>,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sender_id: sender_id.into(),
            receiver_id,
            message_type: MessageType::Request,
            priority: Priority::Normal,
            content: content.into(),
            metadata: HashMap::new(),
            timestamp,
        }
    }

    /// 设置消息类型
    pub fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = message_type;
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// 消息总线
pub struct MessageBus {
    /// 通道映射 (agent_id -> sender)
    channels: RwLock<HashMap<String, broadcast::Sender<AgentMessage>>>,
    /// 全局广播通道
    broadcast_channel: broadcast::Sender<AgentMessage>,
}

impl MessageBus {
    /// 创建新的消息总线
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        Self {
            channels: RwLock::new(HashMap::new()),
            broadcast_channel: broadcast_tx,
        }
    }

    /// 注册 Agent
    pub async fn register(&self, agent_id: impl Into<String>) {
        let agent_id = agent_id.into();
        let (tx, _rx) = broadcast::channel(100);

        let mut channels = self.channels.write().await;
        channels.insert(agent_id.clone(), tx);
    }

    /// 注销 Agent
    pub async fn unregister(&self, agent_id: &str) {
        let mut channels = self.channels.write().await;
        channels.remove(agent_id);
    }

    /// 发送消息
    pub async fn send(&self, receiver_id: &str, message: AgentMessage) -> Result<(), String> {
        let channels = self.channels.read().await;

        if let Some(sender) = channels.get(receiver_id) {
            if sender.send(message).is_ok() {
                Ok(())
            } else {
                Err("Failed to send message".to_string())
            }
        } else {
            Err(format!("Agent {} not registered", receiver_id))
        }
    }

    /// 发送简单字符串消息
    pub async fn send_string(&self, receiver_id: &str, content: impl Into<String>) -> Result<(), String> {
        let msg = AgentMessage::new("system", Some(receiver_id.to_string()), content);
        self.send(receiver_id, msg).await
    }

    /// 接收消息
    pub async fn receive(&self, agent_id: &str) -> Result<AgentMessage, String> {
        let channels = self.channels.read().await;

        if let Some(sender) = channels.get(agent_id) {
            let mut receiver = sender.subscribe();

            // 使用 tokio::select! 实现超时
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                receiver.recv(),
            )
            .await;

            match result {
                Ok(Ok(msg)) => Ok(msg),
                Ok(Err(_)) => Err("Channel closed".to_string()),
                Err(_) => Err("Timeout".to_string()),
            }
        } else {
            Err(format!("Agent {} not registered", agent_id))
        }
    }

    /// 广播消息
    pub fn broadcast(&self, message: AgentMessage) {
        let _ = self.broadcast_channel.send(message);
    }

    /// 订阅广播
    pub fn subscribe_broadcast(&self) -> broadcast::Receiver<AgentMessage> {
        self.broadcast_channel.subscribe()
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = AgentMessage::new("agent1", Some("agent2".to_string()), "Hello")
            .with_priority(Priority::High)
            .with_metadata("key", "value");

        assert_eq!(msg.sender_id, "agent1");
        assert_eq!(msg.receiver_id, Some("agent2".to_string()));
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.priority, Priority::High);
        assert_eq!(msg.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_message_bus_creation() {
        let bus = MessageBus::new();
        assert_eq!(bus.channels.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let bus = MessageBus::new();
        bus.register("agent1").await;

        let channels = bus.channels.read().await;
        assert_eq!(channels.len(), 1);
        assert!(channels.contains_key("agent1"));
    }

    #[tokio::test]
    async fn test_send_and_receive() {
        let bus = MessageBus::new();
        bus.register("agent1").await;
        bus.register("agent2").await;

        let message = AgentMessage::new("agent1", Some("agent2".to_string()), "Test message");

        // 发送消息
        let send_result = bus.send("agent2", message).await;
        assert!(send_result.is_ok());

        // 接收消息
        let recv_result = bus.receive("agent2").await;
        assert!(recv_result.is_ok());
        let received_msg = recv_result.unwrap();
        assert_eq!(received_msg.content, "Test message");
    }

    #[tokio::test]
    async fn test_broadcast() {
        let bus = MessageBus::new();

        let msg = AgentMessage::new("agent1", None, "Broadcast message");
        bus.broadcast(msg);

        let mut receiver = bus.subscribe_broadcast();
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.content, "Broadcast message");
    }
}
