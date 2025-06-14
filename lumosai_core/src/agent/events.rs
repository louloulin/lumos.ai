//! Agent事件驱动架构
//! 
//! 提供Agent系统的事件发布、订阅和处理机制，支持异步事件处理和事件持久化。

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::{Result, Error};

/// Agent事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentEvent {
    /// Agent启动事件
    AgentStarted {
        agent_id: String,
        timestamp: DateTime<Utc>,
        metadata: HashMap<String, serde_json::Value>,
    },
    /// Agent停止事件
    AgentStopped {
        agent_id: String,
        timestamp: DateTime<Utc>,
        reason: String,
    },
    /// Agent状态变更事件
    StateChanged {
        agent_id: String,
        old_state: String,
        new_state: String,
        timestamp: DateTime<Utc>,
    },
    /// 消息发送事件
    MessageSent {
        from_agent: String,
        to_agent: Option<String>, // None表示广播
        message_id: String,
        content: String,
        timestamp: DateTime<Utc>,
    },
    /// 消息接收事件
    MessageReceived {
        agent_id: String,
        from_agent: String,
        message_id: String,
        content: String,
        timestamp: DateTime<Utc>,
    },
    /// 工具调用事件
    ToolCalled {
        agent_id: String,
        tool_name: String,
        parameters: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
    /// 工具调用结果事件
    ToolResult {
        agent_id: String,
        tool_name: String,
        result: std::result::Result<serde_json::Value, String>,
        duration_ms: u64,
        timestamp: DateTime<Utc>,
    },
    /// 错误事件
    Error {
        agent_id: Option<String>,
        error_type: String,
        error_message: String,
        context: HashMap<String, serde_json::Value>,
        timestamp: DateTime<Utc>,
    },
    /// 协作开始事件
    CollaborationStarted {
        session_id: String,
        participants: Vec<String>,
        task_description: String,
        timestamp: DateTime<Utc>,
    },
    /// 协作完成事件
    CollaborationCompleted {
        session_id: String,
        participants: Vec<String>,
        results: HashMap<String, serde_json::Value>,
        duration_ms: u64,
        timestamp: DateTime<Utc>,
    },
    /// 自定义事件
    Custom {
        event_name: String,
        data: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
}

impl AgentEvent {
    /// 获取事件ID
    pub fn event_id(&self) -> String {
        Uuid::new_v4().to_string()
    }
    
    /// 获取事件时间戳
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            AgentEvent::AgentStarted { timestamp, .. } => *timestamp,
            AgentEvent::AgentStopped { timestamp, .. } => *timestamp,
            AgentEvent::StateChanged { timestamp, .. } => *timestamp,
            AgentEvent::MessageSent { timestamp, .. } => *timestamp,
            AgentEvent::MessageReceived { timestamp, .. } => *timestamp,
            AgentEvent::ToolCalled { timestamp, .. } => *timestamp,
            AgentEvent::ToolResult { timestamp, .. } => *timestamp,
            AgentEvent::Error { timestamp, .. } => *timestamp,
            AgentEvent::CollaborationStarted { timestamp, .. } => *timestamp,
            AgentEvent::CollaborationCompleted { timestamp, .. } => *timestamp,
            AgentEvent::Custom { timestamp, .. } => *timestamp,
        }
    }
    
    /// 获取相关的Agent ID
    pub fn agent_id(&self) -> Option<&str> {
        match self {
            AgentEvent::AgentStarted { agent_id, .. } => Some(agent_id),
            AgentEvent::AgentStopped { agent_id, .. } => Some(agent_id),
            AgentEvent::StateChanged { agent_id, .. } => Some(agent_id),
            AgentEvent::MessageSent { from_agent, .. } => Some(from_agent),
            AgentEvent::MessageReceived { agent_id, .. } => Some(agent_id),
            AgentEvent::ToolCalled { agent_id, .. } => Some(agent_id),
            AgentEvent::ToolResult { agent_id, .. } => Some(agent_id),
            AgentEvent::Error { agent_id, .. } => agent_id.as_deref(),
            AgentEvent::CollaborationStarted { .. } => None,
            AgentEvent::CollaborationCompleted { .. } => None,
            AgentEvent::Custom { .. } => None,
        }
    }
}

/// 事件处理器trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// 处理事件
    async fn handle_event(&self, event: &AgentEvent) -> Result<()>;
    
    /// 获取处理器名称
    fn name(&self) -> &str;
    
    /// 获取感兴趣的事件类型
    fn interested_events(&self) -> Vec<String>;
}

/// 事件过滤器
pub struct EventFilter {
    /// 事件类型过滤
    pub event_types: Option<Vec<String>>,
    /// Agent ID过滤
    pub agent_ids: Option<Vec<String>>,
    /// 时间范围过滤
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// 自定义过滤条件
    pub custom_filter: Option<Box<dyn Fn(&AgentEvent) -> bool + Send + Sync>>,
}

impl EventFilter {
    /// 创建新的过滤器
    pub fn new() -> Self {
        Self {
            event_types: None,
            agent_ids: None,
            time_range: None,
            custom_filter: None,
        }
    }
    
    /// 设置事件类型过滤
    pub fn with_event_types(mut self, types: Vec<String>) -> Self {
        self.event_types = Some(types);
        self
    }
    
    /// 设置Agent ID过滤
    pub fn with_agent_ids(mut self, ids: Vec<String>) -> Self {
        self.agent_ids = Some(ids);
        self
    }
    
    /// 设置时间范围过滤
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.time_range = Some((start, end));
        self
    }
    
    /// 检查事件是否匹配过滤条件
    pub fn matches(&self, event: &AgentEvent) -> bool {
        // 检查事件类型
        if let Some(ref types) = self.event_types {
            let event_type = match event {
                AgentEvent::AgentStarted { .. } => "AgentStarted",
                AgentEvent::AgentStopped { .. } => "AgentStopped",
                AgentEvent::StateChanged { .. } => "StateChanged",
                AgentEvent::MessageSent { .. } => "MessageSent",
                AgentEvent::MessageReceived { .. } => "MessageReceived",
                AgentEvent::ToolCalled { .. } => "ToolCalled",
                AgentEvent::ToolResult { .. } => "ToolResult",
                AgentEvent::Error { .. } => "Error",
                AgentEvent::CollaborationStarted { .. } => "CollaborationStarted",
                AgentEvent::CollaborationCompleted { .. } => "CollaborationCompleted",
                AgentEvent::Custom { .. } => "Custom",
            };
            
            if !types.contains(&event_type.to_string()) {
                return false;
            }
        }
        
        // 检查Agent ID
        if let Some(ref ids) = self.agent_ids {
            if let Some(agent_id) = event.agent_id() {
                if !ids.contains(&agent_id.to_string()) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // 检查时间范围
        if let Some((start, end)) = self.time_range {
            let timestamp = event.timestamp();
            if timestamp < start || timestamp > end {
                return false;
            }
        }
        
        // 检查自定义过滤条件
        if let Some(ref filter) = self.custom_filter {
            if !filter(event) {
                return false;
            }
        }
        
        true
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// 事件总线
pub struct EventBus {
    /// 广播发送器
    sender: broadcast::Sender<AgentEvent>,
    /// 事件处理器
    handlers: Arc<RwLock<HashMap<String, Arc<dyn EventHandler>>>>,
    /// 事件历史（可选）
    event_history: Arc<RwLock<Vec<AgentEvent>>>,
    /// 是否启用历史记录
    enable_history: bool,
    /// 最大历史记录数
    max_history_size: usize,
}

impl EventBus {
    /// 创建新的事件总线
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        
        Self {
            sender,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
            enable_history: true,
            max_history_size: 10000,
        }
    }
    
    /// 设置是否启用历史记录
    pub fn with_history(mut self, enable: bool) -> Self {
        self.enable_history = enable;
        self
    }
    
    /// 设置最大历史记录数
    pub fn with_max_history_size(mut self, size: usize) -> Self {
        self.max_history_size = size;
        self
    }
    
    /// 发布事件
    pub async fn publish(&self, event: AgentEvent) -> Result<()> {
        // 添加到历史记录
        if self.enable_history {
            let mut history = self.event_history.write().await;
            history.push(event.clone());
            
            // 限制历史记录大小
            if history.len() > self.max_history_size {
                history.remove(0);
            }
        }
        
        // 广播事件
        if let Err(e) = self.sender.send(event.clone()) {
            return Err(Error::Event(format!("Failed to broadcast event: {}", e)));
        }
        
        // 调用注册的处理器
        let handlers = self.handlers.read().await;
        for handler in handlers.values() {
            if let Err(e) = handler.handle_event(&event).await {
                eprintln!("Event handler {} failed: {}", handler.name(), e);
            }
        }
        
        Ok(())
    }
    
    /// 订阅事件
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.sender.subscribe()
    }
    
    /// 注册事件处理器
    pub async fn register_handler(&self, handler: Arc<dyn EventHandler>) -> Result<()> {
        let mut handlers = self.handlers.write().await;
        handlers.insert(handler.name().to_string(), handler);
        Ok(())
    }
    
    /// 注销事件处理器
    pub async fn unregister_handler(&self, name: &str) -> Result<()> {
        let mut handlers = self.handlers.write().await;
        handlers.remove(name);
        Ok(())
    }
    
    /// 获取事件历史
    pub async fn get_history(&self, filter: Option<EventFilter>) -> Vec<AgentEvent> {
        let history = self.event_history.read().await;
        
        if let Some(filter) = filter {
            history.iter()
                .filter(|event| filter.matches(event))
                .cloned()
                .collect()
        } else {
            history.clone()
        }
    }
    
    /// 清空事件历史
    pub async fn clear_history(&self) {
        let mut history = self.event_history.write().await;
        history.clear();
    }
}

/// 日志事件处理器
pub struct LogEventHandler {
    name: String,
}

impl LogEventHandler {
    pub fn new() -> Self {
        Self {
            name: "log_handler".to_string(),
        }
    }
}

impl Default for LogEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventHandler for LogEventHandler {
    async fn handle_event(&self, event: &AgentEvent) -> Result<()> {
        println!("[{}] {:?}", event.timestamp().format("%Y-%m-%d %H:%M:%S"), event);
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn interested_events(&self) -> Vec<String> {
        vec!["*".to_string()] // 对所有事件感兴趣
    }
}

/// 指标收集事件处理器
pub struct MetricsEventHandler {
    name: String,
    metrics: Arc<RwLock<HashMap<String, u64>>>,
}

impl MetricsEventHandler {
    pub fn new() -> Self {
        Self {
            name: "metrics_handler".to_string(),
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 获取指标
    pub async fn get_metrics(&self) -> HashMap<String, u64> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
    
    /// 重置指标
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.clear();
    }
}

impl Default for MetricsEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventHandler for MetricsEventHandler {
    async fn handle_event(&self, event: &AgentEvent) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        
        let event_type = match event {
            AgentEvent::AgentStarted { .. } => "agent_started",
            AgentEvent::AgentStopped { .. } => "agent_stopped",
            AgentEvent::StateChanged { .. } => "state_changed",
            AgentEvent::MessageSent { .. } => "message_sent",
            AgentEvent::MessageReceived { .. } => "message_received",
            AgentEvent::ToolCalled { .. } => "tool_called",
            AgentEvent::ToolResult { .. } => "tool_result",
            AgentEvent::Error { .. } => "error",
            AgentEvent::CollaborationStarted { .. } => "collaboration_started",
            AgentEvent::CollaborationCompleted { .. } => "collaboration_completed",
            AgentEvent::Custom { .. } => "custom",
        };
        
        *metrics.entry(event_type.to_string()).or_insert(0) += 1;
        *metrics.entry("total_events".to_string()).or_insert(0) += 1;
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn interested_events(&self) -> Vec<String> {
        vec!["*".to_string()]
    }
}
