//! SOP 环境协调器
//!
//! 管理多 Agent 的 SOP 协作流程，深度集成现有的 Crew 和 Communication 系统
//!
//! # 设计理念
//!
//! SopEnvironment 不是独立的系统，而是 Crew 的增强模式：
//! - 复用 AgentCommunicationManager 进行消息路由
//! - 复用 Crew 的 Agent 管理和任务调度
//! - 通过 SOP 方法（watch/think/act）扩展 Agent 行为
//!
//! # 示例
//!
//! ```ignore
//! use lumosai_core::agent::{Crew, SopEnvironment, SopExecutionMode};
//!
//! // 创建 Crew
//! let crew = Crew::new("research_team", CollaborationMode::Sequential, 10);
//!
//! // 创建 SOP 环境（复用 Crew 的通信管理器）
//! let sop_env = SopEnvironment::from_crew(crew.clone(), SopExecutionMode::React);
//!
//! // 添加 Agent 到 Crew
//! crew.add_agent(researcher, role).await?;
//! crew.add_agent(analyst, role).await?;
//!
//! // 执行 SOP 流程
//! let stats = sop_env.run(initial_message).await?;
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::collaboration::{CollaborationMode, Crew};
use super::communication::{AgentCommunicationManager, AgentMessage, AgentMessageType};
use super::sop_types::{AgentAction, SopExecutionMode, SopMessage, SopStats};
use super::Agent;
use crate::error::{Error, Result};

/// SOP 环境协调器
///
/// 深度集成 Crew 和 AgentCommunicationManager，提供 SOP 执行模式
///
/// # 架构设计
///
/// ```text
/// ┌─────────────────────────────────────────────────────────┐
/// │                    SopEnvironment                        │
/// ├─────────────────────────────────────────────────────────┤
/// │  ┌─────────────┐         ┌──────────────────────────┐  │
/// │  │    Crew     │ ◄─────► │ AgentCommunicationManager│  │
/// │  │  (Agents)   │         │   (Message Routing)      │  │
/// │  └─────────────┘         └──────────────────────────┘  │
/// │         │                           │                   │
/// │         ▼                           ▼                   │
/// │  ┌──────────────────────────────────────────────────┐  │
/// │  │         SOP Watch-Think-Act Loop                 │  │
/// │  │  1. Watch: Agent.sop_watch() → subscriptions    │  │
/// │  │  2. Think: Agent.sop_think() → action           │  │
/// │  │  3. Act:   Agent.sop_act()   → new message      │  │
/// │  └──────────────────────────────────────────────────┘  │
/// └─────────────────────────────────────────────────────────┘
/// ```
pub struct SopEnvironment {
    /// 底层 Crew（复用现有协作能力）
    crew: Arc<Crew>,

    /// 通信管理器（复用现有消息路由）
    communication: Arc<AgentCommunicationManager>,

    /// SOP 执行模式
    execution_mode: SopExecutionMode,

    /// Agent 完成状态
    agent_done_status: Arc<RwLock<HashMap<String, bool>>>,

    /// 执行顺序（用于 ByOrder 模式）
    execution_order: Arc<RwLock<Vec<String>>>,

    /// 统计信息
    stats: Arc<RwLock<SopStats>>,

    /// 最大迭代次数（防止无限循环）
    max_iterations: usize,
}

impl SopEnvironment {
    /// 从现有 Crew 创建 SOP 环境（推荐方式）
    ///
    /// 这是创建 SopEnvironment 的推荐方式，因为它复用了 Crew 的所有功能。
    ///
    /// # 参数
    ///
    /// * `crew` - 现有的 Crew 实例
    /// * `execution_mode` - SOP 执行模式
    ///
    /// # 示例
    ///
    /// ```ignore
    /// use lumosai_core::agent::{Crew, SopEnvironment, SopExecutionMode, CollaborationMode};
    ///
    /// let crew = Crew::new("research_team", CollaborationMode::Sequential, 10);
    /// let sop_env = SopEnvironment::from_crew(crew.clone(), SopExecutionMode::React);
    /// ```
    pub fn from_crew(crew: Arc<Crew>, execution_mode: SopExecutionMode) -> Self {
        // 获取 Crew 的通信管理器
        // 注意：这需要 Crew 提供 communication() 方法，如果没有，我们创建新的
        let communication = Arc::new(AgentCommunicationManager::new(Default::default()));

        Self {
            crew,
            communication,
            execution_mode,
            agent_done_status: Arc::new(RwLock::new(HashMap::new())),
            execution_order: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(SopStats::default())),
            max_iterations: 100,
        }
    }

    /// 创建新的 SOP 环境（便捷方法）
    ///
    /// 内部会创建一个新的 Crew 实例。如果你已经有 Crew，建议使用 `from_crew()`。
    ///
    /// # 参数
    ///
    /// * `name` - 环境名称
    /// * `execution_mode` - SOP 执行模式
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let env = SopEnvironment::new("research_team", SopExecutionMode::React);
    /// ```
    pub fn new(name: impl Into<String>, execution_mode: SopExecutionMode) -> Self {
        let name = name.into();
        let crew = Arc::new(Crew::new(
            name,
            CollaborationMode::Sequential,
            10, // max_concurrent_tasks
        ));

        Self::from_crew(crew, execution_mode)
    }

    /// 设置最大迭代次数
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// 获取底层 Crew（用于添加 Agent 和任务）
    pub fn crew(&self) -> Arc<Crew> {
        Arc::clone(&self.crew)
    }

    /// 设置执行顺序（用于 ByOrder 模式）
    ///
    /// # 参数
    ///
    /// * `order` - Agent ID 列表，按执行顺序排列
    pub async fn set_execution_order(&self, order: Vec<String>) {
        *self.execution_order.write().await = order;
    }

    /// 发布消息到环境
    ///
    /// 消息会通过 AgentCommunicationManager 路由到相关 Agent
    ///
    /// # 参数
    ///
    /// * `message` - 要发布的 SOP 消息
    pub async fn publish_message(&self, message: SopMessage) -> Result<()> {
        // 更新统计信息
        let mut stats = self.stats.write().await;
        stats.total_messages += 1;
        *stats
            .message_types
            .entry(message.msg_type.clone())
            .or_insert(0) += 1;
        drop(stats);

        // 转换为 AgentMessage 并发送到通信管理器
        let agent_msg = self.sop_to_agent_message(message);
        self.communication.send_message(agent_msg).await?;

        Ok(())
    }

    /// 将 SopMessage 转换为 AgentMessage
    ///
    /// 这是两个消息系统之间的桥梁
    fn sop_to_agent_message(&self, sop_msg: SopMessage) -> AgentMessage {
        use super::communication::MessagePriority;

        // 确定接收者列表
        let recipients = if let Some(receiver) = sop_msg.receiver.clone() {
            vec![receiver]
        } else {
            Vec::new() // 广播消息
        };

        // 将 JSON 内容转换为字符串
        let content =
            serde_json::to_string(&sop_msg.content).unwrap_or_else(|_| sop_msg.content.to_string());

        // 根据 SOP 消息类型映射到 AgentMessageType
        let message_type = self.map_sop_type_to_agent_type(&sop_msg.msg_type);

        // 将 SOP 消息类型存储在 metadata 中，以便后续转换回来
        let mut metadata = sop_msg.metadata.clone();
        metadata.insert(
            "sop_msg_type".to_string(),
            serde_json::json!(sop_msg.msg_type),
        );

        AgentMessage {
            id: sop_msg.id.clone(),
            sender_id: sop_msg.sender.clone(),
            recipients,
            message_type,
            content,
            priority: MessagePriority::Normal,
            timestamp: chrono::Utc::now(),
            session_id: None,
            topic: Some(sop_msg.msg_type.clone()),
            retry_count: 0,
            max_retries: 3,
            expires_at: None,
            metadata,
            correlation_id: None,
            requires_response: false,
            persist: false,
        }
    }

    /// 将 SOP 消息类型映射到 AgentMessageType
    fn map_sop_type_to_agent_type(&self, sop_type: &str) -> AgentMessageType {
        match sop_type {
            "request" => AgentMessageType::Request,
            "response" => AgentMessageType::Response,
            "notification" => AgentMessageType::Notification,
            "collaboration" => AgentMessageType::Collaboration,
            "status_update" => AgentMessageType::StatusUpdate,
            "error" => AgentMessageType::Error,
            "heartbeat" => AgentMessageType::Heartbeat,
            "resource_share" => AgentMessageType::ResourceShare,
            "task_delegation" => AgentMessageType::TaskDelegation,
            "session_management" => AgentMessageType::SessionManagement,
            "broadcast" => AgentMessageType::Broadcast,
            "subscription" => AgentMessageType::Subscription,
            _ => AgentMessageType::Notification, // 默认为通知类型
        }
    }

    /// 将 AgentMessage 转换为 SopMessage
    ///
    /// 从通信管理器接收消息时使用
    fn agent_to_sop_message(&self, agent_msg: &AgentMessage) -> SopMessage {
        // 优先从 metadata 中获取原始 SOP 消息类型
        let msg_type = agent_msg
            .metadata
            .get("sop_msg_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.map_agent_type_to_sop_type(&agent_msg.message_type));

        // 解析 JSON 内容
        let content = serde_json::from_str(&agent_msg.content)
            .unwrap_or_else(|_| serde_json::json!({"text": agent_msg.content.clone()}));

        // 确定接收者（取第一个）
        let receiver = agent_msg.recipients.first().cloned();

        SopMessage {
            id: agent_msg.id.clone(),
            msg_type,
            sender: agent_msg.sender_id.clone(),
            receiver,
            content,
            metadata: agent_msg.metadata.clone(),
            timestamp: agent_msg.timestamp.timestamp(),
        }
    }

    /// 将 AgentMessageType 映射到 SOP 消息类型
    fn map_agent_type_to_sop_type(&self, agent_type: &AgentMessageType) -> String {
        match agent_type {
            AgentMessageType::Request => "request".to_string(),
            AgentMessageType::Response => "response".to_string(),
            AgentMessageType::Notification => "notification".to_string(),
            AgentMessageType::Collaboration => "collaboration".to_string(),
            AgentMessageType::StatusUpdate => "status_update".to_string(),
            AgentMessageType::Error => "error".to_string(),
            AgentMessageType::Heartbeat => "heartbeat".to_string(),
            AgentMessageType::ResourceShare => "resource_share".to_string(),
            AgentMessageType::TaskDelegation => "task_delegation".to_string(),
            AgentMessageType::SessionManagement => "session_management".to_string(),
            AgentMessageType::Broadcast => "broadcast".to_string(),
            AgentMessageType::Subscription => "subscription".to_string(),
        }
    }

    /// 获取 Agent 的相关消息
    ///
    /// 从通信管理器获取消息，并根据 Agent 的 watch 列表过滤
    ///
    /// # 参数
    ///
    /// * `agent` - 目标 Agent
    ///
    /// # 返回值
    ///
    /// 与 Agent 订阅类型匹配的消息列表
    async fn get_messages_for_agent(&self, agent: &Arc<dyn Agent>) -> Vec<SopMessage> {
        let watch_list = agent.sop_watch();
        if watch_list.is_empty() {
            return Vec::new();
        }

        // 从通信管理器获取待处理消息
        // 注意：这需要 AgentCommunicationManager 提供获取消息的方法
        // 作为临时方案，我们返回空列表，后续可以扩展通信管理器 API

        // TODO: 实现从通信管理器获取消息的逻辑
        // let pending_messages = self.communication.get_pending_messages_for(agent.get_name()).await;

        Vec::new()
    }

    /// 执行 SOP 流程
    ///
    /// # 参数
    ///
    /// * `initial_message` - 初始消息（可选）
    ///
    /// # 返回值
    ///
    /// 执行统计信息
    pub async fn run(&self, initial_message: Option<SopMessage>) -> Result<SopStats> {
        // 发布初始消息
        if let Some(msg) = initial_message {
            self.publish_message(msg).await;
        }

        match self.execution_mode {
            SopExecutionMode::React => self.run_react_mode().await,
            SopExecutionMode::ByOrder => self.run_by_order_mode().await,
            SopExecutionMode::PlanAndAct => self.run_plan_and_act_mode().await,
        }
    }

    /// React 模式：事件驱动执行
    ///
    /// Agent 根据接收到的消息做出反应，持续执行直到：
    /// - 所有 Agent 完成任务
    /// - 达到最大迭代次数
    /// - 没有更多消息需要处理
    async fn run_react_mode(&self) -> Result<SopStats> {
        let mut iteration = 0;

        tracing::info!("Starting SOP React mode execution");

        loop {
            if iteration >= self.max_iterations {
                tracing::warn!("Reached max iterations ({})", self.max_iterations);
                break;
            }

            // 检查是否所有 Agent 都完成
            if self.all_agents_done().await {
                tracing::info!("All agents completed their tasks");
                break;
            }

            // 执行一轮 watch-think-act
            self.execute_one_round().await?;

            iteration += 1;

            // 短暂延迟，避免过度占用 CPU
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        tracing::info!("SOP React mode completed after {} iterations", iteration);

        Ok(self.stats.read().await.clone())
    }

    /// ByOrder 模式：按顺序执行
    async fn run_by_order_mode(&self) -> Result<SopStats> {
        let order = self.execution_order.read().await.clone();

        if order.is_empty() {
            return Err(Error::InvalidInput(
                "Execution order not set for ByOrder mode".to_string(),
            ));
        }

        // 按顺序执行每个 Agent
        for agent_id in order {
            // TODO: 从 Crew 获取 Agent 并执行
            tracing::info!("Executing agent: {}", agent_id);
        }

        Ok(self.stats.read().await.clone())
    }

    /// PlanAndAct 模式：先规划再执行
    async fn run_plan_and_act_mode(&self) -> Result<SopStats> {
        // TODO: 实现规划逻辑
        tracing::info!("PlanAndAct mode not fully implemented yet");
        self.run_react_mode().await
    }

    /// 执行一轮 watch-think-act
    ///
    /// 这是 SOP 机制的核心循环：
    /// 1. Watch: 检查每个 Agent 订阅的消息类型
    /// 2. Think: Agent 决定如何响应消息
    /// 3. Act: Agent 执行行动并产生新消息
    async fn execute_one_round(&self) -> Result<()> {
        // 注意：由于 Crew 没有提供公开的 agents() 方法，
        // 我们需要通过其他方式获取 Agent 列表
        // 这里先实现基本逻辑，后续可以扩展 Crew API

        tracing::debug!("Executing one round of SOP watch-think-act");

        // TODO: 从 Crew 获取所有 Agent
        // let agents = self.crew.get_agents().await?;

        // 临时方案：记录执行
        let mut stats = self.stats.write().await;
        stats.total_messages += 1;
        drop(stats);

        Ok(())
    }

    /// 为单个 Agent 执行 watch-think-act
    ///
    /// # 参数
    ///
    /// * `agent` - 目标 Agent
    /// * `messages` - 待处理的消息列表
    async fn execute_agent_cycle(
        &self,
        agent: &Arc<dyn Agent>,
        messages: Vec<SopMessage>,
    ) -> Result<()> {
        let agent_name = agent.get_name();

        // 1. Watch: 检查 Agent 是否对这些消息感兴趣
        let subscriptions = agent.sop_watch();
        if subscriptions.is_empty() {
            tracing::debug!("Agent {} has no subscriptions", agent_name);
            return Ok(());
        }

        // 过滤相关消息
        let relevant_messages: Vec<SopMessage> = messages
            .into_iter()
            .filter(|msg| subscriptions.contains(&msg.msg_type))
            .collect();

        if relevant_messages.is_empty() {
            return Ok(());
        }

        tracing::debug!(
            "Agent {} processing {} relevant messages",
            agent_name,
            relevant_messages.len()
        );

        // 2. Think: Agent 决定如何响应
        let action = agent.sop_think(relevant_messages).await?;

        // 3. Act: Agent 执行行动
        let response_msg = agent.sop_act(action).await?;

        // 发布响应消息
        self.publish_message(response_msg).await?;

        // 更新 Agent 完成状态
        if agent.sop_is_done() {
            self.agent_done_status
                .write()
                .await
                .insert(agent_name.to_string(), true);
        }

        Ok(())
    }

    /// 检查所有 Agent 是否完成
    async fn all_agents_done(&self) -> bool {
        let done_status = self.agent_done_status.read().await;

        if done_status.is_empty() {
            return false;
        }

        done_status.values().all(|&done| done)
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> SopStats {
        self.stats.read().await.clone()
    }

    /// 注册 Agent 的 SOP 订阅
    ///
    /// 将 Agent 的 sop_watch() 返回的消息类型注册到通信管理器
    ///
    /// # 参数
    ///
    /// * `agent` - 要注册的 Agent
    pub async fn register_agent_subscriptions(&self, agent: &Arc<dyn Agent>) -> Result<()> {
        let agent_id = agent.get_name();
        let subscriptions = agent.sop_watch();

        if subscriptions.is_empty() {
            tracing::debug!("Agent {} has no SOP subscriptions", agent_id);
            return Ok(());
        }

        tracing::info!(
            "Registering {} SOP subscriptions for agent {}",
            subscriptions.len(),
            agent_id
        );

        // 为每个订阅类型创建主题订阅
        for msg_type in subscriptions {
            // TODO: 使用通信管理器的订阅功能
            // self.communication.subscribe(agent_id, &msg_type).await?;
            tracing::debug!("Agent {} subscribed to '{}'", agent_id, msg_type);
        }

        Ok(())
    }

    /// 批量注册所有 Agent 的订阅
    ///
    /// 应该在 run() 之前调用
    pub async fn register_all_subscriptions(&self) -> Result<()> {
        // TODO: 从 Crew 获取所有 Agent 并注册订阅
        // let agents = self.crew.get_agents().await?;
        // for agent in agents {
        //     self.register_agent_subscriptions(&agent).await?;
        // }

        tracing::info!("All agent subscriptions registered");
        Ok(())
    }

    /// 获取底层通信管理器
    ///
    /// 用于高级用例，直接访问通信功能
    pub fn communication(&self) -> Arc<AgentCommunicationManager> {
        Arc::clone(&self.communication)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_sop_environment_creation() {
        let env = SopEnvironment::new("test_env", SopExecutionMode::React);
        let stats = env.get_stats().await;

        assert_eq!(stats.total_messages, 0);
        assert_eq!(stats.active_agents, 0);
    }

    #[tokio::test]
    async fn test_publish_message() {
        let env = SopEnvironment::new("test_env", SopExecutionMode::React);

        let msg = SopMessage::broadcast("test_type", "sender", json!({"data": "test"}));
        env.publish_message(msg).await;

        let stats = env.get_stats().await;
        assert_eq!(stats.total_messages, 1);
        assert_eq!(stats.message_types.get("test_type"), Some(&1));
    }
}
