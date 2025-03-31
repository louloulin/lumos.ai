//! 消息定义和处理

use std::collections::HashMap;
use std::fmt::{self, Display};
use std::time::{Duration, SystemTime};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::types::AgentId;

/// 消息ID类型
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub String);

impl MessageId {
    /// 创建一个随机MessageId
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

/// 消息类型
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum MessageType {
    /// 文本消息
    Text,
    /// 命令消息
    Command,
    /// 事件消息
    Event,
    /// 查询消息
    Query,
    /// 响应消息
    Response,
    /// 错误消息
    Error,
    /// 系统消息
    System,
    /// 心跳消息
    Heartbeat,
    /// 自定义类型
    Custom(String),
}

impl Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Text => write!(f, "Text"),
            MessageType::Command => write!(f, "Command"),
            MessageType::Event => write!(f, "Event"),
            MessageType::Query => write!(f, "Query"),
            MessageType::Response => write!(f, "Response"),
            MessageType::Error => write!(f, "Error"),
            MessageType::System => write!(f, "System"),
            MessageType::Heartbeat => write!(f, "Heartbeat"),
            MessageType::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

/// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    /// 低优先级
    Low = 0,
    /// 普通优先级
    Normal = 1,
    /// 高优先级
    High = 2,
    /// 紧急优先级
    Urgent = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// 消息状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    /// 已创建
    Created,
    /// 已发送
    Sent,
    /// 已接收
    Received,
    /// 已处理
    Processed,
    /// 已回复
    Replied,
    /// 已失败
    Failed,
    /// 已过期
    Expired,
}

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息ID
    pub id: MessageId,
    /// 发送者ID
    pub sender: AgentId,
    /// 接收者ID
    pub receivers: Vec<AgentId>,
    /// 消息类型
    pub message_type: MessageType,
    /// 消息内容
    pub content: serde_json::Value,
    /// 消息状态
    pub status: MessageStatus,
    /// 消息优先级
    pub priority: MessagePriority,
    /// 创建时间
    pub created_at: SystemTime,
    /// 过期时间
    pub expires_at: Option<SystemTime>,
    /// 相关消息ID（例如回复的是哪条消息）
    pub reference_id: Option<MessageId>,
    /// 消息元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Message {
    /// 创建新消息
    pub fn new(
        sender: impl Into<AgentId>, 
        receivers: Vec<AgentId>, 
        message_type: MessageType, 
        content: impl Into<serde_json::Value>
    ) -> Self {
        Self {
            id: MessageId::new(),
            sender: sender.into(),
            receivers,
            message_type,
            content: content.into(),
            status: MessageStatus::Created,
            priority: MessagePriority::Normal,
            created_at: SystemTime::now(),
            expires_at: None,
            reference_id: None,
            metadata: HashMap::new(),
        }
    }
    
    /// 设置过期时间
    pub fn with_expiry(mut self, duration: Duration) -> Self {
        let expires_at = self.created_at.checked_add(duration).unwrap_or(self.created_at);
        self.expires_at = Some(expires_at);
        self
    }
    
    /// 设置优先级
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// 设置引用消息ID
    pub fn with_reference(mut self, reference_id: MessageId) -> Self {
        self.reference_id = Some(reference_id);
        self
    }
    
    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// 将状态设置为已发送
    pub fn mark_as_sent(&mut self) {
        self.status = MessageStatus::Sent;
    }
    
    /// 将状态设置为已接收
    pub fn mark_as_received(&mut self) {
        self.status = MessageStatus::Received;
    }
    
    /// 将状态设置为已处理
    pub fn mark_as_processed(&mut self) {
        self.status = MessageStatus::Processed;
    }
    
    /// 创建对此消息的回复
    pub fn create_reply(&self, content: impl Into<serde_json::Value>) -> Self {
        Self {
            id: MessageId::new(),
            sender: self.receivers[0].clone(), // 假设第一个接收者是回复者
            receivers: vec![self.sender.clone()],
            message_type: MessageType::Response,
            content: content.into(),
            status: MessageStatus::Created,
            priority: self.priority,
            created_at: SystemTime::now(),
            expires_at: None,
            reference_id: Some(self.id.clone()),
            metadata: HashMap::new(),
        }
    }
    
    /// 检查消息是否已过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            SystemTime::now() > expires_at
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        let sender = AgentId::from_str("agent1");
        let receiver = AgentId::from_str("agent2");
        
        let message = Message::new(
            sender.clone(),
            vec![receiver.clone()],
            MessageType::Text,
            "Hello, Agent 2!"
        );
        
        assert_eq!(message.sender, sender);
        assert_eq!(message.receivers[0], receiver);
        assert_eq!(message.message_type, MessageType::Text);
        assert_eq!(message.content, serde_json::json!("Hello, Agent 2!"));
        assert_eq!(message.status, MessageStatus::Created);
        assert_eq!(message.priority, MessagePriority::Normal);
    }
    
    #[test]
    fn test_message_reply() {
        let sender = AgentId::from_str("agent1");
        let receiver = AgentId::from_str("agent2");
        
        let message = Message::new(
            sender.clone(),
            vec![receiver.clone()],
            MessageType::Query,
            "What's the status?"
        );
        
        let reply = message.create_reply("Everything is fine");
        
        assert_eq!(reply.sender, receiver);
        assert_eq!(reply.receivers[0], sender);
        assert_eq!(reply.message_type, MessageType::Response);
        assert_eq!(reply.content, serde_json::json!("Everything is fine"));
        assert_eq!(reply.reference_id.unwrap(), message.id);
    }
    
    #[test]
    fn test_message_expiry() {
        // 创建一个很快过期的消息
        let sender = AgentId::from_str("agent1");
        let receiver = AgentId::from_str("agent2");
        
        let mut message = Message::new(
            sender,
            vec![receiver],
            MessageType::Text,
            "This will expire"
        ).with_expiry(Duration::from_nanos(1));
        
        // 确保时间已经过去
        std::thread::sleep(Duration::from_millis(1));
        
        assert!(message.is_expired());
        
        // 创建一个不会很快过期的消息
        message.expires_at = Some(SystemTime::now() + Duration::from_secs(3600));
        assert!(!message.is_expired());
    }
} 