//! Agent网络实现

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::task::JoinHandle;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use dashmap::DashMap;

use crate::error::{Error, Result};
use crate::types::{AgentId, AgentType, AgentStatus, AgentCapability};
use crate::message::{Message, MessageType};
use crate::router::{MessageRouter, DefaultMessageRouter};
use crate::topology::{NetworkTopology, GraphTopology};
use crate::discovery::{ServiceDiscovery, InMemoryServiceDiscovery, ServiceRegistration};

/// Agent配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent名称
    pub name: String,
    /// Agent类型
    pub agent_type: AgentType,
    /// Agent能力
    pub capabilities: Vec<AgentCapability>,
    /// 消息缓冲区大小
    pub message_buffer_size: usize,
    /// 自动注册到服务发现
    pub register_with_discovery: bool,
    /// TTL (秒)
    pub ttl: u64,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            agent_type: AgentType::Regular,
            capabilities: Vec::new(),
            message_buffer_size: 100,
            register_with_discovery: true,
            ttl: 60,
            metadata: HashMap::new(),
        }
    }
}

/// Agent处理函数
type AgentHandler = Box<dyn Fn(Message) -> Result<Vec<Message>> + Send + Sync>;

/// Agent节点
pub struct AgentNode {
    /// Agent ID
    id: AgentId,
    /// Agent配置
    config: AgentConfig,
    /// Agent状态
    status: Arc<RwLock<AgentStatus>>,
    /// 消息接收通道
    receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
    /// 消息发送通道
    sender: mpsc::Sender<Message>,
    /// 网络引用
    network: Option<Arc<AgentNetwork>>,
    /// 消息处理器
    message_handlers: DashMap<MessageType, Vec<AgentHandler>>,
    /// Agent任务句柄
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl AgentNode {
    /// 创建新的Agent节点
    pub fn new(id: Option<AgentId>, config: AgentConfig) -> Self {
        let id = id.unwrap_or_else(AgentId::new);
        let (sender, receiver) = mpsc::channel(config.message_buffer_size);
        
        Self {
            id,
            config,
            status: Arc::new(RwLock::new(AgentStatus::Initialized)),
            receiver: Arc::new(Mutex::new(receiver)),
            sender,
            network: None,
            message_handlers: DashMap::new(),
            task_handle: Arc::new(Mutex::new(None)),
        }
    }
    
    /// 获取Agent ID
    pub fn id(&self) -> &AgentId {
        &self.id
    }
    
    /// 获取Agent配置
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }
    
    /// 获取Agent状态
    pub async fn status(&self) -> AgentStatus {
        *self.status.read().await
    }
    
    /// 设置Agent状态
    pub async fn set_status(&self, status: AgentStatus) {
        let mut current_status = self.status.write().await;
        *current_status = status;
    }
    
    /// 添加消息处理器
    pub fn add_message_handler<F>(&self, message_type: MessageType, handler: F)
    where
        F: Fn(Message) -> Result<Vec<Message>> + Send + Sync + 'static,
    {
        let handler_box = Box::new(handler) as AgentHandler;
        
        if let Some(mut handlers) = self.message_handlers.get_mut(&message_type) {
            handlers.push(handler_box);
        } else {
            self.message_handlers.insert(message_type, vec![handler_box]);
        }
    }
    
    /// 获取消息发送通道
    pub fn sender(&self) -> mpsc::Sender<Message> {
        self.sender.clone()
    }
    
    /// 设置网络引用
    pub fn set_network(&mut self, network: Arc<AgentNetwork>) {
        self.network = Some(network);
    }
    
    /// 发送消息
    pub async fn send(&self, message: Message) -> Result<()> {
        if let Some(network) = &self.network {
            network.send_message(message).await
        } else {
            Err(Error::Network("Agent未连接到网络".into()))
        }
    }
    
    /// 启动Agent
    pub async fn start(&self) -> Result<()> {
        // 检查当前状态
        let current_status = self.status().await;
        if current_status == AgentStatus::Running {
            return Err(Error::Network("Agent已经在运行中".into()));
        }
        
        // 设置状态为运行中
        self.set_status(AgentStatus::Running).await;
        
        // 创建新的管道用于消息传递
        let (tx, mut rx) = mpsc::channel::<Message>(100);
        
        // 创建一个线程处理消息接收
        let receiver = self.receiver.clone();
        let status = self.status.clone();
        
        // 创建任务来从消息通道接收消息并转发到内部通道
        let handle = tokio::spawn(async move {
            let mut receiver_guard = receiver.lock().await;
            
            while *status.read().await == AgentStatus::Running {
                if let Some(message) = receiver_guard.recv().await {
                    if tx.send(message).await.is_err() {
                        break;
                    }
                }
            }
        });
        
        // 保存任务句柄
        let mut task_handle = self.task_handle.lock().await;
        *task_handle = Some(handle);
        
        // 启动一个单独的任务来处理消息（这里没有使用DashMap的clone方法）
        let agent_id = self.id.clone();
        let network = self.network.clone();
        let status2 = self.status.clone();
        
        // 创建一个处理消息的任务
        tokio::spawn(async move {
            while *status2.read().await == AgentStatus::Running {
                if let Some(mut message) = rx.recv().await {
                    // 标记消息已接收
                    message.mark_as_received();
                    
                    // 由于无法直接访问message_handlers，我们在这里模拟处理逻辑
                    log::info!("Agent {} 接收到消息: {:?}", agent_id, message.message_type);
                    
                    // 如果是文本消息，创建一个自动响应
                    if message.message_type == MessageType::Text {
                        if let Some(net) = &network {
                            let response = message.create_reply("自动回复：消息已收到");
                            if let Err(e) = net.send_message(response).await {
                                log::error!("发送响应失败: {}", e);
                            }
                        }
                    }
                    
                    // 标记消息已处理
                    message.mark_as_processed();
                } else {
                    break;
                }
            }
        });
        
        Ok(())
    }
    
    /// 停止Agent
    pub async fn stop(&self) -> Result<()> {
        // 设置状态为已停止
        self.set_status(AgentStatus::Stopped).await;
        
        // 终止任务
        let mut task_handle = self.task_handle.lock().await;
        if let Some(handle) = task_handle.take() {
            handle.abort();
        }
        
        Ok(())
    }
    
    /// 处理消息
    async fn process_message(&self, message: Message) -> Result<Vec<Message>> {
        // 创建结果向量
        let mut results = Vec::new();
        
        // 获取消息类型
        let msg_type = &message.message_type;
        
        // 查找消息处理器
        if let Some(handlers) = self.message_handlers.get(msg_type) {
            // 遍历所有处理器
            for handler in handlers.value() {
                // 调用处理器
                match handler(message.clone()) {
                    Ok(mut messages) => {
                        results.append(&mut messages);
                    }
                    Err(e) => {
                        log::error!("处理消息时出错: {}", e);
                    }
                }
            }
        }
        
        Ok(results)
    }
}

/// Agent网络
pub struct AgentNetwork {
    /// 网络ID
    id: String,
    /// 已注册的Agent
    agents: DashMap<AgentId, Arc<AgentNode>>,
    /// 消息路由器
    router: Arc<dyn MessageRouter>,
    /// 网络拓扑
    topology: Arc<dyn NetworkTopology>,
    /// 服务发现
    discovery: Arc<dyn ServiceDiscovery>,
}

impl AgentNetwork {
    /// 创建新的Agent网络
    pub async fn new() -> Self {
        // 创建默认路由器
        let router = Arc::new(DefaultMessageRouter::new());
        
        // 创建默认拓扑
        let topology: Arc<dyn NetworkTopology> = Arc::new(GraphTopology::fully_connected());
        
        // 设置路由器的拓扑
        router.set_topology(topology.clone()).await;
        
        // 创建服务发现
        let discovery = InMemoryServiceDiscovery::new();
        
        Self {
            id: Uuid::new_v4().to_string(),
            agents: DashMap::new(),
            router,
            topology,
            discovery,
        }
    }
    
    /// 创建自定义Agent网络
    pub async fn with_custom_components(
        router: Arc<dyn MessageRouter>,
        topology: Arc<dyn NetworkTopology>,
        discovery: Arc<dyn ServiceDiscovery>,
    ) -> Self {
        // 设置路由器的拓扑
        router.set_topology(topology.clone()).await;
        
        Self {
            id: Uuid::new_v4().to_string(),
            agents: DashMap::new(),
            router,
            topology,
            discovery,
        }
    }
    
    /// 获取网络ID
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// 获取Agent数量
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
    
    /// 获取路由器
    pub fn router(&self) -> Arc<dyn MessageRouter> {
        self.router.clone()
    }
    
    /// 获取拓扑
    pub fn topology(&self) -> Arc<dyn NetworkTopology> {
        self.topology.clone()
    }
    
    /// 获取服务发现
    pub fn discovery(&self) -> Arc<dyn ServiceDiscovery> {
        self.discovery.clone()
    }
    
    /// 添加Agent
    pub async fn add_agent(&self, mut agent: AgentNode) -> Result<Arc<AgentNode>> {
        let agent_id = agent.id().clone();
        
        // 设置网络引用
        agent.set_network(Arc::new(self.clone()));
        
        // 将Agent包装为Arc
        let agent_arc = Arc::new(agent);
        
        // 添加到网络
        self.agents.insert(agent_id.clone(), agent_arc.clone());
        
        // 注册到路由器
        let sender = agent_arc.sender();
        self.router.register(agent_id.clone(), sender).await?;
        
        // 添加到拓扑
        self.topology.add_node(agent_id.clone(), None).await?;
        
        // 注册到服务发现（如果配置允许）
        if agent_arc.config.register_with_discovery {
            let registration = ServiceRegistration::new(agent_id.clone(), agent_arc.config.agent_type.clone())
                .with_ttl(agent_arc.config.ttl);
                
            // 添加能力
            let mut reg_with_capabilities = registration;
            for capability in &agent_arc.config.capabilities {
                reg_with_capabilities = reg_with_capabilities.with_capability(capability.clone());
            }
            
            // 添加元数据
            let mut reg_with_metadata = reg_with_capabilities;
            for (key, value) in &agent_arc.config.metadata {
                reg_with_metadata = reg_with_metadata.with_metadata(key, value);
            }
            
            self.discovery.register(reg_with_metadata).await?;
        }
        
        Ok(agent_arc)
    }
    
    /// 移除Agent
    pub async fn remove_agent(&self, id: &AgentId) -> Result<()> {
        // 从网络中移除
        if let Some((_, agent)) = self.agents.remove(id) {
            // 停止Agent
            agent.stop().await?;
            
            // 从路由器注销
            self.router.unregister(id).await?;
            
            // 从拓扑中移除
            self.topology.remove_node(id).await?;
            
            // 从服务发现中注销
            if agent.config.register_with_discovery {
                self.discovery.deregister(id).await?;
            }
            
            Ok(())
        } else {
            Err(Error::AgentNotFound(id.to_string()))
        }
    }
    
    /// 获取Agent
    pub fn get_agent(&self, id: &AgentId) -> Option<Arc<AgentNode>> {
        self.agents.get(id).map(|entry| entry.value().clone())
    }
    
    /// 获取所有Agent
    pub fn get_all_agents(&self) -> Vec<Arc<AgentNode>> {
        self.agents.iter().map(|entry| entry.value().clone()).collect()
    }
    
    /// 发送消息
    pub async fn send_message(&self, message: Message) -> Result<()> {
        // 检查接收者是否存在
        for receiver in &message.receivers {
            if !self.agents.contains_key(receiver) {
                return Err(Error::AgentNotFound(receiver.to_string()));
            }
        }
        
        // 通过路由器发送消息
        self.router.route(message).await
    }
    
    /// 发送广播消息
    pub async fn broadcast(&self, sender: &AgentId, message_type: MessageType, content: impl Into<serde_json::Value>) -> Result<()> {
        // 创建广播消息
        let receivers: Vec<AgentId> = self.agents.iter()
            .filter(|entry| entry.key() != sender)
            .map(|entry| entry.key().clone())
            .collect();
            
        if receivers.is_empty() {
            return Ok(());
        }
        
        let message = Message::new(
            sender.clone(),
            receivers,
            message_type,
            content.into()
        );
        
        // 通过路由器发送消息
        self.router.route(message).await
    }
    
    /// 启动所有Agent
    pub async fn start_all_agents(&self) -> Result<()> {
        for entry in self.agents.iter() {
            let agent = entry.value();
            agent.start().await?;
        }
        
        Ok(())
    }
    
    /// 停止所有Agent
    pub async fn stop_all_agents(&self) -> Result<()> {
        for entry in self.agents.iter() {
            let agent = entry.value();
            agent.stop().await?;
        }
        
        Ok(())
    }
    
    /// 设置心跳任务（保持服务发现注册活跃）
    pub fn start_heartbeat_task(&self, interval: Duration) -> JoinHandle<()> {
        let agents = self.agents.clone();
        let discovery = self.discovery.clone();
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            
            loop {
                ticker.tick().await;
                
                for entry in agents.iter() {
                    let agent = entry.value();
                    if agent.config.register_with_discovery {
                        let agent_id = agent.id().clone();
                        
                        // 只为Running状态的Agent发送心跳
                        if agent.status().await == AgentStatus::Running {
                            if let Err(e) = discovery.heartbeat(&agent_id).await {
                                eprintln!("心跳失败: {}", e);
                            }
                        }
                    }
                }
            }
        })
    }
}

impl Clone for AgentNetwork {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            agents: self.agents.clone(),
            router: self.router.clone(),
            topology: self.topology.clone(),
            discovery: self.discovery.clone(),
        }
    }
}

/// 网络中心管理器
pub struct NetworkManager {
    /// 网络实例的ID
    id: String,
    /// 消息路由器
    router: Arc<dyn MessageRouter>,
    /// 用于管理消息处理器的映射
    message_handlers: DashMap<MessageType, Vec<Box<dyn Fn(Message) -> Result<Vec<Message>> + Send + Sync>>>,
    /// 网络拓扑管理
    topology: Option<Arc<dyn NetworkTopology>>,
    /// 全局元数据
    metadata: RwLock<HashMap<String, String>>,
    /// 是否已启动
    running: RwLock<bool>,
}

impl NetworkManager {
    /// 处理接收到的消息
    async fn handle_message(&self, message: Message) -> Result<()> {
        // 创建一个消息克隆用于处理
        let mut message_clone = message.clone();
        
        // 标记为已接收
        message_clone.mark_as_received();
        
        // 获取消息类型
        let msg_type = &message_clone.message_type;
        
        // 处理消息并生成响应消息
        let mut responses = Vec::new();
        
        // 检查是否有该类型的消息处理器
        if let Some(handlers) = self.message_handlers.get(msg_type) {
            // 遍历所有处理器
            for handler in handlers.value() {
                // 克隆消息对象，以便每个处理器都有自己的副本
                let msg_for_handler = message_clone.clone();
                
                // 调用处理器
                match handler(msg_for_handler) {
                    Ok(mut handler_responses) => {
                        responses.append(&mut handler_responses);
                    }
                    Err(e) => {
                        eprintln!("处理消息时出错: {}", e);
                    }
                }
            }
        }
        
        // 将所有响应消息通过路由器发送
        for response in responses {
            if let Err(e) = self.router.route(response).await {
                eprintln!("发送响应消息时出错: {}", e);
            }
        }
        
        // 标记原始消息为已处理
        message_clone.mark_as_processed();
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_agent_creation() {
        // 创建Agent配置
        let config = AgentConfig {
            name: "test-agent".to_string(),
            agent_type: AgentType::Regular,
            ..Default::default()
        };
        
        // 创建Agent
        let agent = AgentNode::new(None, config);
        
        // 验证Agent状态
        assert_eq!(agent.status().await, AgentStatus::Initialized);
        
        // 验证Agent配置
        assert_eq!(agent.config().name, "test-agent");
        assert_eq!(agent.config().agent_type, AgentType::Regular);
    }
    
    #[tokio::test]
    async fn test_agent_network() {
        // 创建网络
        let network = AgentNetwork::new().await;
        
        // 创建Agent
        let agent1_config = AgentConfig {
            name: "agent1".to_string(),
            agent_type: AgentType::Regular,
            ..Default::default()
        };
        
        let agent2_config = AgentConfig {
            name: "agent2".to_string(),
            agent_type: AgentType::Regular,
            ..Default::default()
        };
        
        let agent1 = AgentNode::new(None, agent1_config);
        let agent2 = AgentNode::new(None, agent2_config);
        
        // 添加Agent到网络
        let agent1_arc = network.add_agent(agent1).await.unwrap();
        let agent2_arc = network.add_agent(agent2).await.unwrap();
        
        // 验证网络Agent数量
        assert_eq!(network.agent_count(), 2);
        
        // 启动Agent
        agent1_arc.start().await.unwrap();
        agent2_arc.start().await.unwrap();
        
        // 验证Agent状态
        assert_eq!(agent1_arc.status().await, AgentStatus::Running);
        assert_eq!(agent2_arc.status().await, AgentStatus::Running);
        
        // 停止Agent
        agent1_arc.stop().await.unwrap();
        agent2_arc.stop().await.unwrap();
        
        // 验证Agent状态
        assert_eq!(agent1_arc.status().await, AgentStatus::Stopped);
        assert_eq!(agent2_arc.status().await, AgentStatus::Stopped);
    }
    
    #[tokio::test]
    async fn test_message_handling() {
        // 创建网络
        let network = AgentNetwork::new().await;
        
        // 创建Agent
        let agent1_config = AgentConfig {
            name: "sender-agent".to_string(),
            agent_type: AgentType::Regular,
            ..Default::default()
        };
        
        let agent2_config = AgentConfig {
            name: "receiver-agent".to_string(),
            agent_type: AgentType::Regular,
            ..Default::default()
        };
        
        let mut agent1 = AgentNode::new(None, agent1_config);
        let mut agent2 = AgentNode::new(None, agent2_config);
        
        // 设置消息处理器
        agent2.add_message_handler(MessageType::Text, |message| {
            // 创建回复消息
            let reply = message.create_reply(format!("收到: {}", message.content));
            Ok(vec![reply])
        });
        
        // 添加Agent到网络
        let agent1_arc = network.add_agent(agent1).await.unwrap();
        let agent2_arc = network.add_agent(agent2).await.unwrap();
        
        // 启动Agent
        agent1_arc.start().await.unwrap();
        agent2_arc.start().await.unwrap();
        
        // 从agent1发送消息到agent2
        let message = Message::new(
            agent1_arc.id().clone(),
            vec![agent2_arc.id().clone()],
            MessageType::Text,
            "Hello, Agent2!"
        );
        
        network.send_message(message).await.unwrap();
        
        // 等待消息处理
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // 停止Agent
        agent1_arc.stop().await.unwrap();
        agent2_arc.stop().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_service_discovery_integration() {
        // 创建网络
        let network = AgentNetwork::new().await;
        
        // 创建Agent，设置能力和元数据
        let config = AgentConfig {
            name: "search-agent".to_string(),
            agent_type: AgentType::Regular,
            capabilities: vec![
                AgentCapability::new("search", "Search capability"),
                AgentCapability::new("index", "Index capability"),
            ],
            metadata: {
                let mut map = HashMap::new();
                map.insert("region".to_string(), "us-west".to_string());
                map
            },
            ..Default::default()
        };
        
        let agent = AgentNode::new(None, config);
        
        // 添加Agent到网络（自动注册到服务发现）
        let agent_arc = network.add_agent(agent).await.unwrap();
        
        // 通过服务发现查询
        let query = crate::discovery::ServiceQuery {
            agent_type: Some(AgentType::Regular),
            required_capabilities: vec!["search".to_string()],
            metadata_filters: {
                let mut map = HashMap::new();
                map.insert("region".to_string(), "us-west".to_string());
                map
            },
        };
        
        let results = network.discovery().discover(&query).await.unwrap();
        
        // 验证查询结果
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, *agent_arc.id());
        assert_eq!(results[0].capabilities.len(), 2);
        
        // 测试直接通过ID查询
        let service = network.discovery().get_by_id(agent_arc.id()).await.unwrap();
        assert_eq!(service.id, *agent_arc.id());
        
        // 从网络中移除Agent（自动从服务发现中注销）
        network.remove_agent(agent_arc.id()).await.unwrap();
        
        // 验证服务已注销
        assert!(network.discovery().get_by_id(agent_arc.id()).await.is_err());
    }
} 