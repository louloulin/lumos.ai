//! Agent间通信系统
//! 
//! 提供Agent之间的消息传递、协作和协调功能

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::llm::Message;
use crate::agent::Agent;

/// Agent通信消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessageType {
    /// 请求消息
    Request,
    /// 响应消息
    Response,
    /// 通知消息
    Notification,
    /// 协作请求
    Collaboration,
    /// 状态更新
    StatusUpdate,
    /// 错误消息
    Error,
}

/// Agent间通信消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// 消息ID
    pub id: String,
    /// 发送者Agent ID
    pub sender_id: String,
    /// 接收者Agent ID
    pub receiver_id: String,
    /// 消息类型
    pub message_type: AgentMessageType,
    /// 消息内容
    pub content: Value,
    /// 消息元数据
    pub metadata: HashMap<String, Value>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 过期时间
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 是否需要响应
    pub requires_response: bool,
    /// 关联的请求ID（用于响应消息）
    pub correlation_id: Option<String>,
}

impl AgentMessage {
    /// 创建新的Agent消息
    pub fn new(
        sender_id: String,
        receiver_id: String,
        message_type: AgentMessageType,
        content: Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender_id,
            receiver_id,
            message_type,
            content,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            expires_at: None,
            requires_response: false,
            correlation_id: None,
        }
    }

    /// 设置是否需要响应
    pub fn with_response_required(mut self, required: bool) -> Self {
        self.requires_response = required;
        self
    }

    /// 设置过期时间
    pub fn with_expiry(mut self, expires_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// 设置关联ID
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// 检查消息是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }

    /// 创建响应消息
    pub fn create_response(&self, content: Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender_id: self.receiver_id.clone(),
            receiver_id: self.sender_id.clone(),
            message_type: AgentMessageType::Response,
            content,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            expires_at: None,
            requires_response: false,
            correlation_id: Some(self.id.clone()),
        }
    }
}

/// Agent通信处理器
pub type MessageHandler = Arc<dyn Fn(AgentMessage) -> Result<Option<AgentMessage>> + Send + Sync>;

/// Agent通信管理器
pub struct AgentCommunicationManager {
    /// 注册的Agent
    agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
    /// 消息路由表
    message_routes: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<AgentMessage>>>>,
    /// 消息处理器
    message_handlers: Arc<RwLock<HashMap<AgentMessageType, Vec<MessageHandler>>>>,
    /// 待处理的响应
    pending_responses: Arc<RwLock<HashMap<String, tokio::sync::oneshot::Sender<AgentMessage>>>>,
    /// 消息历史
    message_history: Arc<RwLock<Vec<AgentMessage>>>,
    /// 配置
    config: CommunicationConfig,
}

/// 通信配置
#[derive(Debug, Clone)]
pub struct CommunicationConfig {
    /// 最大消息历史数量
    pub max_message_history: usize,
    /// 消息默认过期时间（秒）
    pub default_message_ttl: u64,
    /// 是否启用消息持久化
    pub enable_persistence: bool,
    /// 最大并发消息数
    pub max_concurrent_messages: usize,
}

impl Default for CommunicationConfig {
    fn default() -> Self {
        Self {
            max_message_history: 10000,
            default_message_ttl: 300, // 5分钟
            enable_persistence: false,
            max_concurrent_messages: 1000,
        }
    }
}

impl AgentCommunicationManager {
    /// 创建新的通信管理器
    pub fn new(config: CommunicationConfig) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            message_routes: Arc::new(RwLock::new(HashMap::new())),
            message_handlers: Arc::new(RwLock::new(HashMap::new())),
            pending_responses: Arc::new(RwLock::new(HashMap::new())),
            message_history: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// 注册Agent
    pub async fn register_agent(&self, agent_id: String, agent: Arc<dyn Agent>) -> Result<()> {
        let (sender, mut receiver) = mpsc::unbounded_channel::<AgentMessage>();
        
        // 注册Agent
        {
            let mut agents = self.agents.write().await;
            agents.insert(agent_id.clone(), agent);
        }

        // 注册消息路由
        {
            let mut routes = self.message_routes.write().await;
            routes.insert(agent_id.clone(), sender);
        }

        // 启动消息处理任务
        let manager = Arc::new(self.clone());
        let agent_id_clone = agent_id.clone();
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                if let Err(e) = manager.handle_agent_message(agent_id_clone.clone(), message).await {
                    eprintln!("处理Agent消息时出错: {}", e);
                }
            }
        });

        Ok(())
    }

    /// 注销Agent
    pub async fn unregister_agent(&self, agent_id: &str) -> Result<()> {
        {
            let mut agents = self.agents.write().await;
            agents.remove(agent_id);
        }

        {
            let mut routes = self.message_routes.write().await;
            routes.remove(agent_id);
        }

        Ok(())
    }

    /// 发送消息
    pub async fn send_message(&self, message: AgentMessage) -> Result<()> {
        // 检查接收者是否存在
        let routes = self.message_routes.read().await;
        let sender = routes.get(&message.receiver_id)
            .ok_or_else(|| Error::InvalidInput(format!("Agent {} 不存在", message.receiver_id)))?;

        // 发送消息
        sender.send(message.clone())
            .map_err(|_| Error::InvalidInput("发送消息失败".to_string()))?;

        // 记录消息历史
        self.record_message(message).await;

        Ok(())
    }

    /// 发送请求并等待响应
    pub async fn send_request(&self, mut message: AgentMessage) -> Result<AgentMessage> {
        message.requires_response = true;
        
        // 创建响应通道
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();
        
        // 注册待处理的响应
        {
            let mut pending = self.pending_responses.write().await;
            pending.insert(message.id.clone(), response_sender);
        }

        // 发送消息
        self.send_message(message).await?;

        // 等待响应
        let response = response_receiver.await
            .map_err(|_| Error::InvalidInput("等待响应超时".to_string()))?;

        Ok(response)
    }

    /// 广播消息
    pub async fn broadcast_message(&self, sender_id: String, message_type: AgentMessageType, content: Value) -> Result<()> {
        let agents = self.agents.read().await;
        let routes = self.message_routes.read().await;

        for (agent_id, _) in agents.iter() {
            if agent_id != &sender_id {
                if let Some(sender) = routes.get(agent_id) {
                    let message = AgentMessage::new(
                        sender_id.clone(),
                        agent_id.clone(),
                        message_type.clone(),
                        content.clone(),
                    );

                    let _ = sender.send(message.clone());
                    self.record_message(message).await;
                }
            }
        }

        Ok(())
    }

    /// 处理Agent消息
    async fn handle_agent_message(&self, _agent_id: String, message: AgentMessage) -> Result<()> {
        // 检查消息是否过期
        if message.is_expired() {
            return Ok(());
        }

        // 处理响应消息
        if message.message_type == AgentMessageType::Response {
            if let Some(correlation_id) = &message.correlation_id {
                let mut pending = self.pending_responses.write().await;
                if let Some(sender) = pending.remove(correlation_id) {
                    let _ = sender.send(message);
                    return Ok(());
                }
            }
        }

        // 调用消息处理器
        let handlers = self.message_handlers.read().await;
        if let Some(handler_list) = handlers.get(&message.message_type) {
            for handler in handler_list {
                if let Ok(Some(response)) = handler(message.clone()) {
                    self.send_message(response).await?;
                }
            }
        }

        Ok(())
    }

    /// 记录消息历史
    async fn record_message(&self, message: AgentMessage) {
        let mut history = self.message_history.write().await;
        history.push(message);

        // 限制历史记录数量
        if history.len() > self.config.max_message_history {
            history.remove(0);
        }
    }

    /// 获取消息历史
    pub async fn get_message_history(&self, limit: Option<usize>) -> Vec<AgentMessage> {
        let history = self.message_history.read().await;
        let limit = limit.unwrap_or(history.len());
        history.iter().rev().take(limit).cloned().collect()
    }

    /// 添加消息处理器
    pub async fn add_message_handler(&self, message_type: AgentMessageType, handler: MessageHandler) {
        let mut handlers = self.message_handlers.write().await;
        handlers.entry(message_type).or_insert_with(Vec::new).push(handler);
    }

    /// 获取已注册的Agent列表
    pub async fn get_registered_agents(&self) -> Vec<String> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }
}

// 为了支持clone，我们需要手动实现
impl Clone for AgentCommunicationManager {
    fn clone(&self) -> Self {
        Self {
            agents: Arc::clone(&self.agents),
            message_routes: Arc::clone(&self.message_routes),
            message_handlers: Arc::clone(&self.message_handlers),
            pending_responses: Arc::clone(&self.pending_responses),
            message_history: Arc::clone(&self.message_history),
            config: self.config.clone(),
        }
    }
}

/// 创建默认的通信管理器
pub fn create_communication_manager() -> AgentCommunicationManager {
    AgentCommunicationManager::new(CommunicationConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_agent_message_creation() {
        let message = AgentMessage::new(
            "agent1".to_string(),
            "agent2".to_string(),
            AgentMessageType::Request,
            json!({"test": "data"}),
        );

        assert_eq!(message.sender_id, "agent1");
        assert_eq!(message.receiver_id, "agent2");
        assert!(!message.requires_response);
        assert!(!message.is_expired());
    }

    #[test]
    fn test_message_response_creation() {
        let original = AgentMessage::new(
            "agent1".to_string(),
            "agent2".to_string(),
            AgentMessageType::Request,
            json!({"test": "data"}),
        );

        let response = original.create_response(json!({"response": "data"}));

        assert_eq!(response.sender_id, "agent2");
        assert_eq!(response.receiver_id, "agent1");
        assert_eq!(response.correlation_id, Some(original.id));
    }

    #[tokio::test]
    async fn test_communication_manager() {
        let manager = create_communication_manager();
        
        // 测试基本功能
        assert_eq!(manager.get_registered_agents().await.len(), 0);
        
        let history = manager.get_message_history(Some(10)).await;
        assert_eq!(history.len(), 0);
    }
}
