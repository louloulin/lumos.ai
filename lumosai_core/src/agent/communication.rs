//! Agent间通信系统
//!
//! 提供Agent之间的消息传递、协作和协调功能

use base64;
use chrono;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use crate::agent::Agent;
use crate::error::{Error, Result};

/// Agent通信消息类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// 心跳消息
    Heartbeat,
    /// 资源共享
    ResourceShare,
    /// 任务委派
    TaskDelegation,
    /// 会话管理
    SessionManagement,
    /// 广播消息
    Broadcast,
    /// 订阅/取消订阅
    Subscription,
}

/// Agent状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// 活跃状态
    Active,
    /// 繁忙状态
    Busy,
    /// 离线状态
    Offline,
    /// 暂停状态
    Paused,
    /// 维护状态
    Maintenance,
}

/// Agent信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent ID
    pub agent_id: String,
    /// Agent名称
    pub name: String,
    /// Agent状态
    pub status: AgentStatus,
    /// Agent能力列表
    pub capabilities: Vec<String>,
    /// 负载信息
    pub load_info: AgentLoadInfo,
    /// 元数据
    pub metadata: HashMap<String, Value>,
    /// 注册时间
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// 最后活跃时间
    pub last_active: chrono::DateTime<chrono::Utc>,
}

/// Agent负载信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLoadInfo {
    /// CPU使用率 (0-100)
    pub cpu_usage: f64,
    /// 内存使用率 (0-100)
    pub memory_usage: f64,
    /// 当前任务数
    pub current_tasks: usize,
    /// 最大任务数
    pub max_tasks: usize,
    /// 平均响应时间（毫秒）
    pub avg_response_time: f64,
}

/// 通信错误类型
#[derive(Debug, thiserror::Error)]
pub enum CommunicationError {
    #[error("Agent不存在: {0}")]
    AgentNotFound(String),

    #[error("消息无效: {0}")]
    InvalidMessage(String),

    #[error("会话不存在: {0}")]
    SessionNotFound(String),

    #[error("Agent未激活: {0}")]
    AgentNotActive(String),

    #[error("路由失败: {0}")]
    RoutingFailed(String),

    #[error("订阅失败: {0}")]
    SubscriptionError(String),

    #[error("队列为满")]
    QueueFull,

    #[error("内部错误: {0}")]
    Internal(String),
}

impl From<CommunicationError> for Error {
    fn from(err: CommunicationError) -> Self {
        match err {
            CommunicationError::AgentNotFound(msg) => Error::NotFound(msg),
            CommunicationError::InvalidMessage(msg) => Error::InvalidInput(msg),
            CommunicationError::SessionNotFound(msg) => Error::NotFound(msg),
            CommunicationError::AgentNotActive(msg) => Error::InvalidInput(msg),
            CommunicationError::RoutingFailed(msg) => Error::Internal(msg),
            CommunicationError::SubscriptionError(msg) => Error::Internal(msg),
            CommunicationError::QueueFull => Error::Internal("队列已满".to_string()),
            CommunicationError::Internal(msg) => Error::Internal(msg),
        }
    }
}

/// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessagePriority {
    /// 低优先级
    Low = 1,
    /// 普通优先级
    Normal = 5,
    /// 高优先级
    High = 8,
    /// 紧急优先级
    Urgent = 10,
}

/// Agent间通信消息 - 增强版
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// 消息ID
    pub id: String,
    /// 发送者Agent ID
    pub sender_id: String,
    /// 接收者列表（支持多接收者）
    pub recipients: Vec<String>,
    /// 消息类型
    pub message_type: AgentMessageType,
    /// 消息内容
    pub content: String,
    /// 消息优先级
    pub priority: MessagePriority,
    /// 创建时间
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 会话ID
    pub session_id: Option<String>,
    /// 主题（用于订阅/发布模式）
    pub topic: Option<String>,
    /// 重试次数
    pub retry_count: u32,
    /// 最大重试次数
    pub max_retries: u32,
    /// 过期时间
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 消息元数据
    pub metadata: HashMap<String, Value>,
    /// 关联的请求ID（用于响应消息）
    pub correlation_id: Option<String>,
    /// 是否需要响应
    pub requires_response: bool,
    /// 是否需要持久化
    pub persist: bool,
}

impl AgentMessage {
    /// 创建新的Agent消息
    pub fn new(
        sender_id: String,
        recipients: Vec<String>,
        message_type: AgentMessageType,
        content: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender_id,
            recipients,
            message_type,
            content,
            priority: MessagePriority::Normal,
            timestamp: chrono::Utc::now(),
            session_id: None,
            topic: None,
            retry_count: 0,
            max_retries: 3,
            expires_at: None,
            metadata: HashMap::new(),
            correlation_id: None,
            requires_response: false,
            persist: false,
        }
    }

    /// 创建广播消息
    pub fn new_broadcast(
        sender_id: String,
        message_type: AgentMessageType,
        content: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender_id,
            recipients: vec![], // 空接收者列表表示广播
            message_type,
            content,
            priority: MessagePriority::Normal,
            timestamp: chrono::Utc::now(),
            session_id: None,
            topic: None,
            retry_count: 0,
            max_retries: 3,
            expires_at: None,
            metadata: HashMap::new(),
            correlation_id: None,
            requires_response: false,
            persist: false,
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

    /// 设置优先级
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// 设置会话ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// 设置主题
    pub fn with_topic(mut self, topic: String) -> Self {
        self.topic = Some(topic);
        self
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// 设置持久化
    pub fn with_persist(mut self, persist: bool) -> Self {
        self.persist = persist;
        self
    }

    /// 设置时间戳
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = timestamp;
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

    /// 检查是否可以重试
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// 增加重试次数
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// 创建响应消息
    pub fn create_response(&self, content: String) -> Self {
        let mut response = Self {
            id: Uuid::new_v4().to_string(),
            sender_id: self.recipients.first().cloned().unwrap_or_default(),
            recipients: vec![self.sender_id.clone()],
            message_type: AgentMessageType::Response,
            content,
            priority: self.priority,
            timestamp: chrono::Utc::now(),
            session_id: self.session_id.clone(),
            topic: self.topic.clone(),
            retry_count: 0,
            max_retries: 3,
            expires_at: None,
            metadata: self.metadata.clone(),
            correlation_id: Some(self.id.clone()),
            requires_response: false,
            persist: self.persist,
        };

        response
    }

    /// 检查是否为广播消息
    pub fn is_broadcast(&self) -> bool {
        self.recipients.is_empty()
    }

    /// 获取目标Agent列表
    pub fn get_targets(&self) -> Vec<String> {
        if self.recipients.is_empty() {
            vec![] // 广播消息由通信管理器处理
        } else {
            self.recipients.clone()
        }
    }

    /// 获取接收者ID（兼容旧接口）
    pub fn get_receiver_id(&self) -> Option<String> {
        self.recipients.first().cloned()
    }

    /// 设置期望输出（兼容任务系统）
    pub fn with_expected_output(mut self, output: String) -> Self {
        self.metadata
            .insert("expected_output".to_string(), Value::String(output));
        self
    }
}

/// Agent通信处理器
pub type MessageHandler = Arc<dyn Fn(AgentMessage) -> Result<Option<AgentMessage>> + Send + Sync>;

/// 协作会话类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionType {
    /// 一对一对话
    OneToOne,
    /// 群组讨论
    GroupDiscussion,
    /// 头脑风暴
    Brainstorming,
    /// 决策会议
    DecisionMeeting,
    /// 工作流协调
    WorkflowCoordination,
    /// 监督模式
    Supervision,
}

/// 会话状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    /// 活跃
    Active,
    /// 暂停
    Paused,
    /// 已结束
    Ended,
    /// 已取消
    Cancelled,
}

/// Agent会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    /// 会话ID
    pub id: String,
    /// 会话类型
    pub session_type: SessionType,
    /// 会话状态
    pub state: SessionState,
    /// 参与者列表
    pub participants: Vec<String>,
    /// 会话创建者
    pub creator: String,
    /// 会话标题
    pub title: String,
    /// 会话描述
    pub description: Option<String>,
    /// 会话开始时间
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// 会话结束时间
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 会话配置
    pub config: SessionConfig,
    /// 会话统计
    pub stats: SessionStats,
}

/// 会话配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// 最大参与者数量
    pub max_participants: usize,
    /// 是否允许新参与者加入
    pub allow_join: bool,
    /// 是否需要主持人
    pub requires_moderator: bool,
    /// 消息自动过期时间（秒）
    pub message_ttl: u64,
    /// 是否记录会话历史
    pub record_history: bool,
    /// 会话超时时间（秒）
    pub session_timeout: u64,
}

/// 会话统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    /// 总消息数
    pub total_messages: usize,
    /// 总参与者数
    pub total_participants: usize,
    /// 当前活跃参与者数
    pub active_participants: usize,
    /// 最后活动时间
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// 平均响应时间（毫秒）
    pub avg_response_time: f64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_participants: 10,
            allow_join: true,
            requires_moderator: false,
            message_ttl: 300,
            record_history: true,
            session_timeout: 3600, // 1小时
        }
    }
}

impl AgentSession {
    /// 创建新会话
    pub fn new(
        creator: String,
        session_type: SessionType,
        title: String,
        description: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            session_type,
            state: SessionState::Active,
            participants: vec![creator.clone()],
            creator,
            title,
            description,
            started_at: chrono::Utc::now(),
            ended_at: None,
            config: SessionConfig::default(),
            stats: SessionStats {
                total_messages: 0,
                total_participants: 1,
                active_participants: 1,
                last_activity: chrono::Utc::now(),
                avg_response_time: 0.0,
            },
        }
    }

    /// 添加参与者
    pub fn add_participant(&mut self, agent_id: String) -> Result<()> {
        if self.participants.len() >= self.config.max_participants {
            return Err(Error::InvalidInput(
                "Session has reached maximum participants".to_string(),
            ));
        }

        if !self.config.allow_join {
            return Err(Error::InvalidInput(
                "Session does not allow new participants".to_string(),
            ));
        }

        if self.participants.contains(&agent_id) {
            return Err(Error::InvalidInput("Agent already in session".to_string()));
        }

        self.participants.push(agent_id);
        self.stats.total_participants = self.participants.len();
        self.stats.active_participants = self.participants.len();

        Ok(())
    }

    /// 移除参与者
    pub fn remove_participant(&mut self, agent_id: &str) -> bool {
        if let Some(pos) = self.participants.iter().position(|id| id == agent_id) {
            self.participants.remove(pos);

            // 如果创建者离开，会话结束
            if agent_id == self.creator {
                self.state = SessionState::Ended;
                self.ended_at = Some(chrono::Utc::now());
            }

            self.stats.active_participants = self.participants.len();
            true
        } else {
            false
        }
    }

    /// 更新活动状态
    pub fn update_activity(&mut self) {
        self.stats.last_activity = chrono::Utc::now();
    }

    /// 增加消息计数
    pub fn increment_message_count(&mut self) {
        self.stats.total_messages += 1;
    }

    /// 检查会话是否活跃
    pub fn is_active(&self) -> bool {
        matches!(self.state, SessionState::Active)
            && (chrono::Utc::now() - self.stats.last_activity).num_seconds()
                < self.config.session_timeout as i64
    }

    /// 结束会话
    pub fn end_session(&mut self) {
        self.state = SessionState::Ended;
        self.ended_at = Some(chrono::Utc::now());
    }

    /// 暂停会话
    pub fn pause_session(&mut self) {
        self.state = SessionState::Paused;
    }

    /// 恢复会话
    pub fn resume_session(&mut self) {
        self.state = SessionState::Active;
        self.update_activity();
    }
}

/// 路由决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingDecision {
    /// 直接路由到特定Agent
    Direct { recipient: String },
    /// 广播到多个接收者
    Broadcast { recipients: Vec<String> },
    /// 主题路由
    Topic { topic: String },
    /// 会话路由
    Session { session_id: String },
}

/// 消息路由策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingStrategy {
    /// 直接路由
    Direct,
    /// 负载均衡路由
    LoadBalanced,
    /// 基于技能的路由
    SkillBased,
    /// 基于优先级的路由
    PriorityBased,
    /// 地理位置路由
    LocationBased,
}

/// 路由规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// 规则ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 路由策略
    pub strategy: RoutingStrategy,
    /// 匹配条件
    pub conditions: Vec<MatchingCondition>,
    /// 目标Agent列表（为空表示所有匹配的Agent）
    pub target_agents: Vec<String>,
    /// 优先级
    pub priority: u8,
    /// 规则是否启用
    pub enabled: bool,
}

/// 匹配条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchingCondition {
    /// 条件类型
    pub condition_type: String,
    /// 条件值
    pub value: Value,
    /// 比较操作符
    pub operator: String,
}

impl RoutingRule {
    /// 创建新路由规则
    pub fn new(name: String, strategy: RoutingStrategy, target_agents: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            strategy,
            conditions: Vec::new(),
            target_agents,
            priority: 5,
            enabled: true,
        }
    }

    /// 添加匹配条件
    pub fn add_condition(mut self, condition: MatchingCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// 匹配消息
    pub fn matches(&self, message: &AgentMessage) -> bool {
        if !self.enabled {
            return false;
        }

        for condition in &self.conditions {
            if !self.evaluate_condition(condition, message) {
                return false;
            }
        }

        true
    }

    /// 评估单个条件
    fn evaluate_condition(&self, condition: &MatchingCondition, message: &AgentMessage) -> bool {
        match condition.operator.as_str() {
            "equals" => {
                let content_value = serde_json::Value::String(message.content.clone());
                self.compare_values(&condition.value, &content_value)
            }
            "contains" => message.content.contains(&condition.value.to_string()),
            "greater_than" => {
                if let (Ok(val1), Some(val2)) =
                    (message.content.parse::<f64>(), condition.value.as_f64())
                {
                    val1 > val2
                } else {
                    false
                }
            }
            "less_than" => {
                if let (Ok(val1), Some(val2)) =
                    (message.content.parse::<f64>(), condition.value.as_f64())
                {
                    val1 < val2
                } else {
                    false
                }
            }
            _ => true, // 默认匹配
        }
    }

    /// 比较两个值
    fn compare_values(&self, val1: &Value, val2: &Value) -> bool {
        match (val1, val2) {
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Number(n1), Value::Number(n2)) => n1 == n2,
            (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            _ => false,
        }
    }
}

/// 订阅管理器
#[derive(Debug)]
pub struct SubscriptionManager {
    /// 订阅信息：主题 -> 订阅者列表
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// 订阅过滤器：订阅者ID -> 过滤器函数
    filters: Arc<RwLock<HashMap<String, Vec<SubscriptionFilter>>>>,
    /// 订阅统计
    stats: Arc<RwLock<SubscriptionStats>>,
}

/// 订阅过滤器
#[derive(Debug, Clone)]
pub struct SubscriptionFilter {
    /// 过滤器ID
    pub id: String,
    /// 过滤器类型
    pub filter_type: FilterType,
    /// 过滤器值
    pub value: Value,
}

/// 过滤器类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterType {
    /// 消息类型过滤
    MessageType,
    /// 发送者过滤
    SenderId,
    /// 优先级过滤
    Priority,
    /// 会话过滤
    SessionId,
    /// 自定义属性过滤
    CustomProperty,
}

/// 订阅统计
#[derive(Debug, Clone, Default)]
pub struct SubscriptionStats {
    /// 总订阅数
    pub total_subscriptions: usize,
    /// 活跃订阅者数
    pub active_subscribers: usize,
    /// 每日消息统计
    pub daily_message_count: HashMap<String, usize>,
}

impl SubscriptionManager {
    /// 创建新的订阅管理器
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            filters: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(SubscriptionStats::default())),
        }
    }

    /// 取消订阅
    pub async fn unsubscribe(&self, subscriber_id: &str, topic: &str) -> Result<()> {
        let removed = {
            let mut subscriptions = self.subscriptions.write().await;
            if let Some(topic_subs) = subscriptions.get_mut(topic) {
                let initial_len = topic_subs.len();
                topic_subs.retain(|id| id != subscriber_id);
                initial_len != topic_subs.len()
            } else {
                false
            }
        };

        if removed {
            {
                let mut filters = self.filters.write().await;
                filters.remove(subscriber_id);
            }

            {
                let mut stats = self.stats.write().await;
                stats.active_subscribers = stats.active_subscribers.saturating_sub(1);
            }

            tracing::info!("Agent {} unsubscribed from topic: {}", subscriber_id, topic);
        }

        Ok(())
    }

    /// 获取主题订阅者
    pub async fn get_subscribers(&self, topic: &str) -> Vec<String> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.get(topic).cloned().unwrap_or_default()
    }

    /// 获取Agent的订阅列表
    pub async fn get_agent_subscriptions(&self, subscriber_id: &str) -> Vec<String> {
        let subscriptions = self.subscriptions.read().await;
        let mut topics = Vec::new();

        for (topic, subs) in subscriptions.iter() {
            if subs.contains(&subscriber_id.to_string()) {
                topics.push(topic.clone());
            }
        }

        topics
    }

    /// 检查消息是否匹配订阅
    pub async fn message_matches_subscription(
        &self,
        subscriber_id: &str,
        topic: &str,
        message: &AgentMessage,
    ) -> bool {
        // 检查是否订阅了该主题
        let is_subscribed = {
            let subscriptions = self.subscriptions.read().await;
            subscriptions
                .get(topic)
                .map(|subs| subs.contains(&subscriber_id.to_string()))
                .unwrap_or(false)
        };

        if !is_subscribed {
            return false;
        }

        // 检查过滤器
        let filters = self.filters.read().await;
        if let Some(agent_filters) = filters.get(subscriber_id) {
            for filter in agent_filters {
                if !self.evaluate_filter(filter, message) {
                    return false;
                }
            }
        }

        true
    }

    /// 评估过滤器
    fn evaluate_filter(&self, filter: &SubscriptionFilter, message: &AgentMessage) -> bool {
        match filter.filter_type {
            FilterType::MessageType => {
                if let Some(msg_type) = serde_json::to_value(&message.message_type).ok() {
                    self.compare_values(&msg_type, &filter.value)
                } else {
                    false
                }
            }
            FilterType::SenderId => {
                if let Some(sender_id) = serde_json::to_value(&message.sender_id).ok() {
                    self.compare_values(&sender_id, &filter.value)
                } else {
                    false
                }
            }
            FilterType::Priority => {
                if let Some(priority) = serde_json::to_value(&message.priority).ok() {
                    self.compare_values(&priority, &filter.value)
                } else {
                    false
                }
            }
            FilterType::SessionId => {
                if let Some(session_id) = message
                    .session_id
                    .as_ref()
                    .and_then(|s| serde_json::to_value(s).ok())
                {
                    self.compare_values(&session_id, &filter.value)
                } else {
                    false
                }
            }
            FilterType::CustomProperty => {
                if let Some(prop_value) = message.metadata.get("filter.property") {
                    self.compare_values(prop_value, &filter.value)
                } else {
                    true // 如果属性不存在，默认通过
                }
            }
        }
    }

    /// 获取订阅统计
    pub async fn get_stats(&self) -> SubscriptionStats {
        self.stats.read().await.clone()
    }

    /// 更新消息统计
    pub async fn increment_message_count(&self, topic: &str) {
        let mut stats = self.stats.write().await;
        let count = stats
            .daily_message_count
            .entry(topic.to_string())
            .or_insert(0);
        *count += 1;
    }

    /// 订阅主题（单个过滤器）
    pub async fn subscribe(
        &self,
        subscriber_id: String,
        topic: String,
        filter: Option<SubscriptionFilter>,
    ) -> Result<()> {
        let filters = filter.map(|f| vec![f]).unwrap_or_default();
        self.subscribe_with_filters(subscriber_id, topic, filters)
            .await
    }

    /// 订阅主题（带过滤器列表）
    pub async fn subscribe_with_filters(
        &self,
        subscriber_id: String,
        topic: String,
        filters: Vec<SubscriptionFilter>,
    ) -> Result<()> {
        {
            let mut subscriptions = self.subscriptions.write().await;
            let topic_subs = subscriptions.entry(topic.clone()).or_insert_with(Vec::new);

            if !topic_subs.contains(&subscriber_id) {
                topic_subs.push(subscriber_id.clone());
            }
        }

        if !filters.is_empty() {
            let mut filter_map = self.filters.write().await;
            filter_map.insert(subscriber_id.clone(), filters);
        }

        {
            let mut stats = self.stats.write().await;
            stats.total_subscriptions += 1;
            stats.active_subscribers += 1;
        }

        println!("订阅管理器: Agent {} 订阅主题 {}", subscriber_id, topic);
        Ok(())
    }

    /// 获取主题订阅者
    pub async fn get_topic_subscribers(&self, topic: &str) -> Result<Vec<String>> {
        let subscriptions = self.subscriptions.read().await;
        Ok(subscriptions.get(topic).cloned().unwrap_or_default())
    }

    /// 移除Agent的所有订阅
    pub async fn remove_agent_subscriptions(&self, agent_id: &str) -> Result<()> {
        let mut removed_count = 0;

        // 从所有主题订阅中移除该Agent
        {
            let mut subscriptions = self.subscriptions.write().await;
            for (_, subscribers) in subscriptions.iter_mut() {
                let initial_len = subscribers.len();
                subscribers.retain(|id| id != agent_id);
                if subscribers.len() < initial_len {
                    removed_count += 1;
                }
            }
        }

        // 清理该Agent的过滤器
        {
            let mut filters = self.filters.write().await;
            filters.remove(agent_id);
        }

        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.active_subscribers = stats
                .active_subscribers
                .saturating_sub(removed_count as usize);
        }

        Ok(())
    }

    /// 获取主题数量
    pub async fn get_topic_count(&self) -> usize {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.len()
    }

    /// 比较两个值（辅助方法）
    fn compare_values(&self, val1: &Value, val2: &Value) -> bool {
        match (val1, val2) {
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Number(n1), Value::Number(n2)) => n1 == n2,
            (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            _ => false,
        }
    }
}

/// 消息队列管理器
#[derive(Debug)]
pub struct MessageQueueManager {
    /// 待处理消息队列
    pending_messages: Arc<RwLock<VecDeque<AgentMessage>>>,
    /// 按优先级分组的消息队列
    priority_queues: Arc<RwLock<HashMap<MessagePriority, VecDeque<AgentMessage>>>>,
    /// 广播消息队列
    broadcast_queue: Arc<RwLock<VecDeque<AgentMessage>>>,
    /// 过期消息清理任务
    cleanup_task: Arc<tokio::task::JoinHandle<()>>,
    /// 配置
    config: QueueConfig,
}

/// 队列配置
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// 最大队列大小
    pub max_queue_size: usize,
    /// 消息清理间隔（秒）
    pub cleanup_interval: u64,
    /// 默认消息TTL（秒）
    pub default_message_ttl: u64,
    /// 是否启用优先级队列
    pub enable_priority_queue: bool,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10000,
            cleanup_interval: 60,     // 1分钟
            default_message_ttl: 300, // 5分钟
            enable_priority_queue: true,
        }
    }
}

impl MessageQueueManager {
    /// 创建新的消息队列管理器
    pub fn new() -> Self {
        Self::with_config(QueueConfig::default())
    }

    /// 使用配置创建消息队列管理器
    pub fn with_config(config: QueueConfig) -> Self {
        let cleanup_interval = config.cleanup_interval;
        let manager = Self {
            pending_messages: Arc::new(RwLock::new(VecDeque::new())),
            priority_queues: Arc::new(RwLock::new(HashMap::new())),
            broadcast_queue: Arc::new(RwLock::new(VecDeque::new())),
            cleanup_task: Arc::new(tokio::spawn(async move {
                // 清理任务在drop时会被取消
                tokio::time::sleep(tokio::time::Duration::from_secs(cleanup_interval)).await;
            })),
            config,
        };

        // 初始化优先级队列
        let mut priority_queues = manager.priority_queues.blocking_write();
        for priority in [
            MessagePriority::Low,
            MessagePriority::Normal,
            MessagePriority::High,
            MessagePriority::Urgent,
        ] {
            priority_queues.insert(priority, VecDeque::new());
        }
        drop(priority_queues);

        manager
    }

    /// 添加消息到队列
    pub async fn enqueue(&self, message: AgentMessage) -> Result<()> {
        let queue_size = self.get_queue_size().await;
        if queue_size >= self.config.max_queue_size {
            return Err(Error::Internal("Message queue is full".to_string()));
        }

        if message.is_broadcast() {
            let mut broadcast_queue = self.broadcast_queue.write().await;
            broadcast_queue.push_back(message);
        } else if self.config.enable_priority_queue {
            use std::collections::hash_map::Entry;
            let mut priority_queues = self.priority_queues.write().await;
            match priority_queues.entry(message.priority) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().push_back(message);
                }
                Entry::Vacant(entry) => {
                    let mut deque = VecDeque::new();
                    deque.push_back(message);
                    entry.insert(deque);
                }
            }
        } else {
            let mut pending_messages = self.pending_messages.write().await;
            pending_messages.push_back(message);
        }

        Ok(())
    }

    /// 从队列中取出消息
    pub async fn dequeue(&self) -> Option<AgentMessage> {
        // 优先处理广播消息
        {
            let mut broadcast_queue = self.broadcast_queue.write().await;
            if let Some(message) = broadcast_queue.pop_front() {
                return Some(message);
            }
        }

        if self.config.enable_priority_queue {
            // 按优先级顺序处理
            let priority_order = [
                MessagePriority::Urgent,
                MessagePriority::High,
                MessagePriority::Normal,
                MessagePriority::Low,
            ];

            for priority in priority_order {
                let mut priority_queues = self.priority_queues.write().await;
                if let Some(queue) = priority_queues.get_mut(&priority) {
                    if let Some(message) = queue.pop_front() {
                        return Some(message);
                    }
                }
            }
        } else {
            let mut pending_messages = self.pending_messages.write().await;
            return pending_messages.pop_front();
        }

        None
    }

    /// 获取队列大小
    pub async fn get_queue_size(&self) -> usize {
        let pending_size = self.pending_messages.read().await.len();
        let priority_size = if self.config.enable_priority_queue {
            let priority_queues = self.priority_queues.read().await;
            priority_queues.values().map(|q| q.len()).sum()
        } else {
            0
        };
        let broadcast_size = self.broadcast_queue.read().await.len();

        pending_size + priority_size + broadcast_size
    }

    /// 清理过期消息
    pub async fn cleanup_expired_messages(&self) {
        let now = chrono::Utc::now();
        let default_ttl = chrono::Duration::seconds(self.config.default_message_ttl as i64);

        // 清理优先级队列中的过期消息
        if self.config.enable_priority_queue {
            let mut priority_queues = self.priority_queues.write().await;
            for (_, queue) in priority_queues.iter_mut() {
                queue.retain(|msg| {
                    let ttl = if let Some(expires_at) = msg.expires_at {
                        expires_at.signed_duration_since(now)
                    } else {
                        default_ttl
                    };
                    ttl > chrono::Duration::seconds(0)
                });
            }
        }

        // 清理普通队列中的过期消息
        {
            let mut pending_messages = self.pending_messages.write().await;
            pending_messages.retain(|msg| {
                let ttl = if let Some(expires_at) = msg.expires_at {
                    expires_at.signed_duration_since(now)
                } else {
                    default_ttl
                };
                ttl > chrono::Duration::seconds(0)
            });
        }

        // 清理广播队列中的过期消息
        {
            let mut broadcast_queue = self.broadcast_queue.write().await;
            broadcast_queue.retain(|msg| {
                let ttl = if let Some(expires_at) = msg.expires_at {
                    expires_at.signed_duration_since(now)
                } else {
                    default_ttl
                };
                ttl > chrono::Duration::seconds(0)
            });
        }

        tracing::debug!("Cleaned up expired messages");
    }

    /// 获取队列统计信息
    pub async fn get_stats(&self) -> QueueStats {
        let pending_size = self.pending_messages.read().await.len();
        let priority_size = if self.config.enable_priority_queue {
            let priority_queues = self.priority_queues.read().await;
            priority_queues.values().map(|q| q.len()).sum()
        } else {
            0
        };
        let broadcast_size = self.broadcast_queue.read().await.len();

        QueueStats {
            pending_messages: pending_size,
            priority_messages: priority_size,
            broadcast_messages: broadcast_size,
            total_size: pending_size + priority_size + broadcast_size,
        }
    }

    /// 获取下一个消息（增强版）
    pub async fn dequeue_next(&self) -> Result<Option<AgentMessage>> {
        Ok(self.dequeue().await)
    }

    /// 获取队列状态
    pub async fn get_status(&self) -> QueueStatus {
        let stats = self.get_stats().await;
        QueueStatus {
            total_messages: stats.total_size,
            pending_messages: stats.pending_messages,
            priority_messages: stats.priority_messages,
            broadcast_messages: stats.broadcast_messages,
            is_healthy: stats.total_size < self.config.max_queue_size,
            cleanup_interval: self.config.cleanup_interval,
        }
    }
}

/// 队列状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    /// 总消息数
    pub total_messages: usize,
    /// 待处理消息数
    pub pending_messages: usize,
    /// 优先级消息数
    pub priority_messages: usize,
    /// 广播消息数
    pub broadcast_messages: usize,
    /// 队列是否健康
    pub is_healthy: bool,
    /// 清理间隔
    pub cleanup_interval: u64,
}

/// 队列统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    /// 待处理消息数
    pub pending_messages: usize,
    /// 优先级消息数
    pub priority_messages: usize,
    /// 广播消息数
    pub broadcast_messages: usize,
    /// 总消息数
    pub total_size: usize,
}

/// Agent通信管理器 - 增强版
pub struct AgentCommunicationManager {
    /// 注册的Agent信息
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    /// 消息路由表
    message_routes: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<AgentMessage>>>>,
    /// 消息处理器
    message_handlers: Arc<RwLock<HashMap<AgentMessageType, Vec<MessageHandler>>>>,
    /// 待处理的响应
    pending_responses: Arc<RwLock<HashMap<String, tokio::sync::oneshot::Sender<AgentMessage>>>>,
    /// 消息历史
    message_history: Arc<RwLock<Vec<AgentMessage>>>,
    /// 高级通信组件
    message_router: Arc<MessageRouter>,
    session_manager: Arc<SessionManager>,
    subscription_manager: Arc<SubscriptionManager>,
    queue_manager: Arc<MessageQueueManager>,
    /// 通信统计
    stats: Arc<RwLock<CommunicationStats>>,
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
    /// 是否启用高级路由
    pub enable_advanced_routing: bool,
    /// 是否启用会话管理
    pub enable_session_management: bool,
    /// 是否启用订阅/发布模式
    pub enable_pubsub: bool,
    /// 默认路由策略
    pub default_routing_strategy: RoutingStrategy,
}

impl Default for CommunicationConfig {
    fn default() -> Self {
        Self {
            max_message_history: 10000,
            default_message_ttl: 300, // 5分钟
            enable_persistence: false,
            max_concurrent_messages: 1000,
            enable_advanced_routing: true,
            enable_session_management: true,
            enable_pubsub: true,
            default_routing_strategy: RoutingStrategy::Direct,
        }
    }
}

/// 通信统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommunicationStats {
    /// 总消息数
    pub total_messages: usize,
    /// 成功投递的消息数
    pub successful_deliveries: usize,
    /// 失败的消息数
    pub failed_deliveries: usize,
    /// 广播消息数
    pub broadcast_messages: usize,
    /// 平均响应时间（毫秒）
    pub avg_response_time: f64,
    /// 活跃Agent数
    pub active_agents: usize,
    /// 总Agent数
    pub total_agents: usize,
    /// 总会话数
    pub total_sessions: usize,
    /// 活跃会话数
    pub active_sessions: usize,
    /// 总投递时间（毫秒）
    pub total_delivery_time: u64,
    /// 队列大小
    pub queue_size: usize,
    /// 总主题数
    pub total_topics: usize,
}

impl CommunicationStats {
    /// 创建新的统计信息
    pub fn new() -> Self {
        Self::default()
    }
}

/// 会话元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// 会话标题
    pub title: String,
    /// 会话描述
    pub description: Option<String>,
    /// 会话标签
    pub tags: Vec<String>,
    /// 自定义属性
    pub custom_attributes: HashMap<String, Value>,
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self {
            title: "新会话".to_string(),
            description: None,
            tags: vec![],
            custom_attributes: HashMap::new(),
        }
    }
}

/// 消息历史过滤器
#[derive(Debug, Clone, Default)]
pub struct MessageHistoryFilters {
    /// 发送者ID
    pub sender_id: Option<String>,
    /// 会话ID
    pub session_id: Option<String>,
    /// 消息类型
    pub message_type: Option<AgentMessageType>,
    /// 开始时间
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 结束时间
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 限制数量
    pub limit: Option<usize>,
}

/// 通信事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationEvent {
    /// Agent注册
    AgentRegistered { agent_id: String, info: AgentInfo },
    /// Agent注销
    AgentUnregistered { agent_id: String },
    /// 消息发送
    MessageSent {
        message_id: String,
        sender: String,
        recipients: Vec<String>,
    },
    /// 消息接收
    MessageReceived {
        message_id: String,
        receiver: String,
    },
    /// 会话创建
    SessionCreated { session_id: String },
    /// 会话结束
    SessionEnded { session_id: String },
    /// 订阅创建
    SubscriptionCreated { agent_id: String, topic: String },
    /// 订阅取消
    SubscriptionCancelled { agent_id: String, topic: String },
}

/// 消息路由器
#[derive(Debug)]
pub struct MessageRouter {
    /// 路由规则
    rules: Arc<RwLock<Vec<RoutingRule>>>,
    /// 默认路由策略
    default_strategy: RoutingStrategy,
    /// 路由缓存
    route_cache: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

/// 会话管理器
#[derive(Debug)]
pub struct SessionManager {
    /// 活跃会话
    sessions: Arc<RwLock<HashMap<String, AgentSession>>>,
    /// Agent到会话的映射
    agent_sessions: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl MessageRouter {
    /// 创建新的消息路由器
    pub fn new(default_strategy: RoutingStrategy) -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            default_strategy,
            route_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加路由规则
    pub async fn add_rule(&self, rule: RoutingRule) {
        let mut rules = self.rules.write().await;
        rules.push(rule);
    }

    /// 路由消息并返回路由决策
    pub async fn route_message(&self, message: &AgentMessage) -> Vec<RoutingDecision> {
        let mut decisions = Vec::new();

        // 检查是否为会话消息
        if let Some(session_id) = &message.session_id {
            decisions.push(RoutingDecision::Session {
                session_id: session_id.clone(),
            });
            return decisions;
        }

        // 检查是否为主题消息
        if let Some(topic) = &message.topic {
            decisions.push(RoutingDecision::Topic {
                topic: topic.clone(),
            });
            return decisions;
        }

        // 检查是否为广播消息
        if message.recipients.is_empty() {
            decisions.push(RoutingDecision::Broadcast {
                recipients: vec![], // 将在投递时确定具体接收者
            });
            return decisions;
        }

        // 直接路由到指定接收者
        for recipient in &message.recipients {
            decisions.push(RoutingDecision::Direct {
                recipient: recipient.clone(),
            });
        }

        decisions
    }

    /// 基于内容的路由（高级功能）
    pub async fn route_by_content(
        &self,
        message: &AgentMessage,
        available_agents: &[String],
    ) -> Vec<String> {
        // 首先尝试匹配规则
        let mut matched_targets = Vec::new();
        let mut max_priority = 0;

        {
            let rules = self.rules.read().await;
            for rule in rules.iter() {
                if rule.matches(message) && rule.enabled && rule.priority >= max_priority {
                    if rule.priority > max_priority {
                        matched_targets.clear();
                        max_priority = rule.priority;
                    }
                    matched_targets.extend(rule.target_agents.clone());
                }
            }
        }

        if !matched_targets.is_empty() {
            return matched_targets;
        }

        // 如果没有匹配的规则，使用默认策略
        match self.default_strategy {
            RoutingStrategy::Direct => {
                if !message.recipients.is_empty() {
                    message.recipients.clone()
                } else {
                    available_agents.to_vec()
                }
            }
            RoutingStrategy::LoadBalanced => {
                // 简单的轮询负载均衡
                let hash = message.content.chars().map(|c| c as u32).sum::<u32>();
                let start_idx = hash as usize % available_agents.len();
                vec![available_agents[start_idx].clone()]
            }
            RoutingStrategy::PriorityBased => {
                // 基于优先级路由 - 高优先级消息路由给负载最低的Agent
                vec![available_agents.first().cloned().unwrap_or_default()]
            }
            _ => available_agents.to_vec(),
        }
    }

    /// 清空路由缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.route_cache.write().await;
        cache.clear();
    }
}

impl SessionManager {
    /// 创建新的会话管理器
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            agent_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取Agent的所有会话
    pub async fn get_agent_sessions(&self, agent_id: &str) -> Vec<AgentSession> {
        let mut sessions = Vec::new();

        if let Some(session_ids) = self.agent_sessions.read().await.get(agent_id) {
            let session_map = self.sessions.read().await;
            for session_id in session_ids {
                if let Some(session) = session_map.get(session_id) {
                    sessions.push(session.clone());
                }
            }
        }

        sessions
    }

    /// 结束会话
    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.end_session();

            // 更新所有参与者的会话映射
            for participant_id in &session.participants.clone() {
                if let Some(agent_sessions) =
                    self.agent_sessions.write().await.get_mut(participant_id)
                {
                    agent_sessions.retain(|id| id != session_id);
                }
            }

            tracing::info!("Session {} ended", session_id);
            Ok(())
        } else {
            Err(Error::NotFound(format!("Session {} not found", session_id)))
        }
    }

    /// 清理过期会话
    pub async fn cleanup_expired_sessions(&self) {
        let sessions_to_remove = {
            let sessions = self.sessions.read().await;
            sessions
                .iter()
                .filter(|(_, session)| !session.is_active())
                .map(|(session_id, _)| session_id.clone())
                .collect::<Vec<String>>()
        };

        let total_removed = sessions_to_remove.len();
        for session_id in sessions_to_remove {
            let _ = self.end_session(&session_id).await;
        }

        println!("会话管理器: 清理了 {} 个过期会话", total_removed);
    }

    /// 创建会话（简化接口）
    pub async fn create_session(
        &self,
        session_type: SessionType,
        participants: Vec<String>,
        metadata: SessionMetadata,
    ) -> Result<String> {
        let creator = participants.first().cloned().unwrap_or_default();
        let session = AgentSession::new(
            creator.clone(),
            session_type.clone(),
            metadata.title.clone(),
            metadata.description.clone(),
        );

        let session_id = session.id.clone();

        // 验证创建者是否已存在
        let creator_has_session = {
            let agent_sessions = self.agent_sessions.read().await;
            agent_sessions
                .get(&creator)
                .map(|sessions| !sessions.is_empty())
                .unwrap_or(false)
        };

        if creator_has_session && session_type != SessionType::GroupDiscussion {
            return Err(Error::InvalidInput(
                "Agent already has active sessions".to_string(),
            ));
        }

        // 添加会话
        {
            let mut sessions = self.sessions.write().await;
            let mut new_session = session;
            new_session.participants = participants.clone();
            sessions.insert(session_id.clone(), new_session);
        }

        // 更新Agent会话映射
        {
            let mut agent_sessions = self.agent_sessions.write().await;
            for participant_id in participants {
                agent_sessions
                    .entry(participant_id.clone())
                    .or_insert_with(Vec::new)
                    .push(session_id.clone());
            }
        }

        println!(
            "会话管理器: 创建会话 {} (类型: {:?})",
            session_id, session_type
        );
        Ok(session_id)
    }

    /// 获取会话
    pub async fn get_session(&self, session_id: &str) -> Result<Option<AgentSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    /// 验证会话访问权限
    pub async fn validate_session_access(&self, session_id: &str, agent_id: &str) -> Result<()> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            if session.participants.contains(&agent_id.to_string()) {
                Ok(())
            } else {
                Err(Error::InvalidInput(format!(
                    "Agent {} 不是会话 {} 的参与者",
                    agent_id, session_id
                )))
            }
        } else {
            Err(Error::NotFound(format!("Session {} not found", session_id)))
        }
    }

    /// 移除Agent从所有会话
    pub async fn remove_agent_from_sessions(&self, agent_id: &str) -> Result<()> {
        let mut affected_sessions = Vec::new();

        // 找到该Agent参与的所有会话
        {
            let agent_sessions = self.agent_sessions.read().await;
            if let Some(session_ids) = agent_sessions.get(agent_id) {
                affected_sessions = session_ids.clone();
            }
        }

        // 从每个会话中移除该Agent
        for session_id in affected_sessions {
            {
                let mut sessions = self.sessions.write().await;
                if let Some(session) = sessions.get_mut(&session_id) {
                    session.remove_participant(agent_id);

                    // 如果创建者离开，会话结束
                    if agent_id == session.creator {
                        session.state = SessionState::Ended;
                        session.ended_at = Some(chrono::Utc::now());
                    }
                }
            }

            // 更新Agent会话映射
            {
                let mut agent_sessions = self.agent_sessions.write().await;
                if let Some(sessions) = agent_sessions.get_mut(agent_id) {
                    sessions.retain(|id| id != &session_id);
                }

                // 从其他参与者的会话列表中也移除（如果会话已结束）
                let sessions = self.sessions.read().await;
                if let Some(session) = sessions.get(&session_id) {
                    if session.state == SessionState::Ended {
                        for participant_id in &session.participants {
                            if let Some(agent_session_list) = agent_sessions.get_mut(participant_id)
                            {
                                agent_session_list.retain(|id| id != &session_id);
                            }
                        }
                    }
                }
            }
        }

        // 清理空的Agent会话映射
        {
            let mut agent_sessions = self.agent_sessions.write().await;
            agent_sessions.retain(|_, sessions| !sessions.is_empty());
        }

        Ok(())
    }

    /// 获取会话数量
    pub async fn get_session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// 获取活跃会话列表
    pub async fn get_active_sessions(&self) -> Vec<AgentSession> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.is_active())
            .cloned()
            .collect()
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
            message_router: Arc::new(MessageRouter::new(config.default_routing_strategy.clone())),
            session_manager: Arc::new(SessionManager::new()),
            subscription_manager: Arc::new(SubscriptionManager::new()),
            queue_manager: Arc::new(MessageQueueManager::new()),
            stats: Arc::new(RwLock::new(CommunicationStats::new())),
            config,
        }
    }

    /// 创建默认通信管理器
    pub fn new_default() -> Self {
        Self::new(CommunicationConfig::default())
    }

    /// 注册Agent
    pub async fn register_agent(&self, agent_id: String, info: AgentInfo) -> Result<()> {
        // 验证Agent信息
        if info.agent_id != agent_id {
            return Err(Error::InvalidInput("Agent ID mismatch".to_string()));
        }

        let (sender, mut receiver) = mpsc::unbounded_channel::<AgentMessage>();

        // 注册Agent信息
        {
            let mut agents = self.agents.write().await;
            agents.insert(agent_id.clone(), info.clone());
        }

        // 注册消息路由
        {
            let mut routes = self.message_routes.write().await;
            routes.insert(agent_id.clone(), sender);
        }

        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_agents += 1;
            if info.status == AgentStatus::Active {
                stats.active_agents += 1;
            }
        }

        // 启动消息处理任务
        let agents = Arc::clone(&self.agents);
        let message_routes = Arc::clone(&self.message_routes);
        let message_handlers = Arc::clone(&self.message_handlers);
        let pending_responses = Arc::clone(&self.pending_responses);
        let message_history = Arc::clone(&self.message_history);
        let message_router = Arc::clone(&self.message_router);
        let session_manager = Arc::clone(&self.session_manager);
        let subscription_manager = Arc::clone(&self.subscription_manager);
        let queue_manager = Arc::clone(&self.queue_manager);
        let stats = Arc::clone(&self.stats);
        let config = self.config.clone();

        let agent_id_clone = agent_id.clone();
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                // 创建临时的管理器实例来处理消息
                let temp_manager = AgentCommunicationManager {
                    agents: Arc::clone(&agents),
                    message_routes: Arc::clone(&message_routes),
                    message_handlers: Arc::clone(&message_handlers),
                    pending_responses: Arc::clone(&pending_responses),
                    message_history: Arc::clone(&message_history),
                    message_router: Arc::clone(&message_router),
                    session_manager: Arc::clone(&session_manager),
                    subscription_manager: Arc::clone(&subscription_manager),
                    queue_manager: Arc::clone(&queue_manager),
                    stats: Arc::clone(&stats),
                    config: config.clone(),
                };

                if let Err(e) = temp_manager
                    .handle_agent_message(agent_id_clone.clone(), message)
                    .await
                {
                    eprintln!("处理Agent消息时出错: {e}");
                }
            }
        });

        println!(
            "通信管理器: 注册Agent {} (状态: {:?})",
            agent_id, info.status
        );
        Ok(())
    }

    /// 发送消息 - 增强版
    pub async fn send_message(&self, mut message: AgentMessage) -> Result<()> {
        let start_time = std::time::Instant::now();

        // 验证发送者
        self.validate_sender(&message.sender_id).await?;

        // 应用消息路由
        let routing_decisions = self.message_router.route_message(&message).await;

        // 检查会话有效性
        if let Some(session_id) = &message.session_id {
            self.session_manager
                .validate_session_access(session_id, &message.sender_id)
                .await?;
        }

        // 添加时间戳和序列号
        message.timestamp = chrono::Utc::now();
        message.id = self.generate_message_id(&message);

        // 消息队列处理
        if message.priority == MessagePriority::Urgent {
            // 紧急消息直接投递
            self.deliver_message_immediate(&message, &routing_decisions)
                .await?;
        } else {
            // 其他消息加入队列
            self.queue_manager.enqueue(message.clone()).await?;
            self.process_queued_messages(&routing_decisions).await?;
        }

        // 记录消息历史
        self.record_message(&message).await;

        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_messages += 1;
            stats.total_delivery_time += start_time.elapsed().as_millis() as u64;
        }

        Ok(())
    }

    /// 验证发送者
    async fn validate_sender(&self, sender_id: &str) -> Result<()> {
        let agents = self.agents.read().await;
        let agent_info = agents
            .get(sender_id)
            .ok_or_else(|| Error::NotFound(format!("Agent {} 不存在", sender_id)))?;

        if agent_info.status != AgentStatus::Active {
            return Err(Error::InvalidInput(format!("Agent {} 未激活", sender_id)));
        }

        Ok(())
    }

    /// 立即投递消息
    async fn deliver_message_immediate(
        &self,
        message: &AgentMessage,
        routing_decisions: &[RoutingDecision],
    ) -> Result<()> {
        for decision in routing_decisions {
            match decision {
                RoutingDecision::Direct { recipient } => {
                    self.deliver_to_agent(recipient, message).await?;
                }
                RoutingDecision::Broadcast { recipients } => {
                    for recipient in recipients {
                        self.deliver_to_agent(recipient, message).await?;
                    }
                }
                RoutingDecision::Topic { topic } => {
                    self.deliver_to_topic_subscribers(topic, message).await?;
                }
                RoutingDecision::Session { session_id } => {
                    self.deliver_to_session_participants(session_id, message)
                        .await?;
                }
            }
        }
        Ok(())
    }

    /// 投递消息到指定Agent
    async fn deliver_to_agent(&self, agent_id: &str, message: &AgentMessage) -> Result<()> {
        let routes = self.message_routes.read().await;
        if let Some(sender) = routes.get(agent_id) {
            let _ = sender.send(message.clone());
            println!("投递消息到Agent {}: {}", agent_id, message.content);
            Ok(())
        } else {
            Err(Error::NotFound(format!("Agent {} 路由不存在", agent_id)))
        }
    }

    /// 投递消息到主题订阅者
    async fn deliver_to_topic_subscribers(
        &self,
        topic: &str,
        message: &AgentMessage,
    ) -> Result<()> {
        let subscribers = self
            .subscription_manager
            .get_topic_subscribers(topic)
            .await?;
        for subscriber_id in subscribers {
            self.deliver_to_agent(&subscriber_id, message).await?;
        }
        Ok(())
    }

    /// 投递消息到会话参与者
    async fn deliver_to_session_participants(
        &self,
        session_id: &str,
        message: &AgentMessage,
    ) -> Result<()> {
        let session_opt = self.session_manager.get_session(session_id).await?;
        let session =
            session_opt.ok_or_else(|| Error::NotFound(format!("会话 {} 不存在", session_id)))?;

        for participant_id in &session.participants {
            if participant_id != &message.sender_id {
                self.deliver_to_agent(participant_id, message).await?;
            }
        }
        Ok(())
    }

    /// 处理队列中的消息
    async fn process_queued_messages(&self, routing_decisions: &[RoutingDecision]) -> Result<()> {
        while let Some(message) = self.queue_manager.dequeue_next().await? {
            self.deliver_message_immediate(&message, routing_decisions)
                .await?;
        }
        Ok(())
    }

    /// 记录消息
    async fn record_message(&self, message: &AgentMessage) {
        let mut history = self.message_history.write().await;
        history.push(message.clone());

        // 保持历史记录在合理范围内
        if history.len() > self.config.max_message_history {
            history.remove(0);
        }
    }

    /// 生成消息ID
    fn generate_message_id(&self, message: &AgentMessage) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();
        message.sender_id.hash(&mut hasher);
        message.content.hash(&mut hasher);
        chrono::Utc::now().timestamp_nanos().hash(&mut hasher);

        format!("msg_{:x}", hasher.finish())
    }

    /// 触发事件（暂时禁用）
    async fn trigger_event(&self, _event: CommunicationEvent) {
        // 事件处理功能暂时禁用
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

    /// 发送消息（兼容旧接口）
    pub async fn send_message_legacy(&self, message: AgentMessage) -> Result<()> {
        // 检查接收者是否存在
        if let Some(receiver_id) = message.get_receiver_id() {
            let routes = self.message_routes.read().await;
            let sender = routes
                .get(&receiver_id)
                .ok_or_else(|| Error::InvalidInput(format!("Agent {} 不存在", receiver_id)))?;

            // 发送消息
            sender
                .send(message.clone())
                .map_err(|_| Error::InvalidInput("发送消息失败".to_string()))?;

            // 记录消息历史
            self.record_message(&message).await;
        }

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
        self.send_message_legacy(message).await?;

        // 等待响应
        let response = response_receiver
            .await
            .map_err(|_| Error::InvalidInput("等待响应超时".to_string()))?;

        Ok(response)
    }

    /// 广播消息
    pub async fn broadcast_message(
        &self,
        sender_id: String,
        message_type: AgentMessageType,
        content: Value,
    ) -> Result<()> {
        let agents = self.agents.read().await;
        let routes = self.message_routes.read().await;

        for (agent_id, _) in agents.iter() {
            if agent_id != &sender_id {
                if let Some(sender) = routes.get(agent_id) {
                    let message = AgentMessage::new(
                        sender_id.clone(),
                        vec![agent_id.clone()],
                        message_type.clone(),
                        content.to_string(),
                    );

                    let _ = sender.send(message.clone());
                    self.record_message(&message).await;
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

    /// 获取消息历史
    pub async fn get_message_history(&self, limit: Option<usize>) -> Vec<AgentMessage> {
        let history = self.message_history.read().await;
        let limit = limit.unwrap_or(history.len());
        history.iter().rev().take(limit).cloned().collect()
    }

    /// 添加消息处理器
    pub async fn add_message_handler(
        &self,
        message_type: AgentMessageType,
        handler: MessageHandler,
    ) {
        let mut handlers = self.message_handlers.write().await;
        handlers
            .entry(message_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    /// 获取已注册的Agent列表
    pub async fn get_registered_agents(&self) -> Vec<String> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
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
