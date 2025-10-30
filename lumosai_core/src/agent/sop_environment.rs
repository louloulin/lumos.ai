//! SOP 环境协调器
//!
//! 管理多 Agent 的 SOP 协作流程，基于现有的 Crew 和 Communication 系统

use async_trait::async_trait;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::collaboration::{CollaborationMode, Crew};
use super::communication::AgentCommunicationManager;
use super::sop_types::{AgentAction, SopExecutionMode, SopMessage, SopStats};
use super::Agent;
use crate::error::{Error, Result};

/// SOP 环境协调器
///
/// 扩展 Crew 的功能，添加 SOP 执行模式支持
pub struct SopEnvironment {
    /// 底层 Crew（复用现有协作能力）
    crew: Arc<Crew>,

    /// SOP 执行模式
    execution_mode: SopExecutionMode,

    /// 消息队列
    message_queue: Arc<RwLock<VecDeque<SopMessage>>>,

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
    /// 创建新的 SOP 环境
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
        // 不立即创建 Crew，而是延迟初始化
        let name = name.into();
        let crew = Arc::new(Crew::new(
            name,
            CollaborationMode::Sequential,
            10, // max_concurrent_tasks
        ));

        Self {
            crew,
            execution_mode,
            message_queue: Arc::new(RwLock::new(VecDeque::new())),
            agent_done_status: Arc::new(RwLock::new(HashMap::new())),
            execution_order: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(SopStats::default())),
            max_iterations: 100,
        }
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
    /// # 参数
    ///
    /// * `message` - 要发布的消息
    pub async fn publish_message(&self, message: SopMessage) {
        // 更新统计信息
        let mut stats = self.stats.write().await;
        stats.total_messages += 1;
        *stats
            .message_types
            .entry(message.msg_type.clone())
            .or_insert(0) += 1;
        drop(stats);

        // 添加到消息队列
        self.message_queue.write().await.push_back(message);
    }

    /// 获取 Agent 的相关消息
    ///
    /// 根据 Agent 的 watch 列表过滤消息
    async fn get_messages_for_agent(&self, agent: &Arc<dyn Agent>) -> Vec<SopMessage> {
        let watch_list = agent.sop_watch();
        if watch_list.is_empty() {
            return Vec::new();
        }

        let queue = self.message_queue.read().await;
        let agent_id = agent.get_name();

        queue
            .iter()
            .filter(|msg| {
                // 检查消息类型是否在 watch 列表中
                watch_list.contains(&msg.msg_type) &&
                // 检查消息是否发送给此 Agent
                msg.is_for(agent_id)
            })
            .cloned()
            .collect()
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
    async fn run_react_mode(&self) -> Result<SopStats> {
        let mut iteration = 0;

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

            // 检查是否有消息需要处理
            if self.message_queue.read().await.is_empty() {
                tracing::info!("No more messages to process");
                break;
            }

            // 执行一轮 watch-think-act
            self.execute_one_round().await?;

            iteration += 1;
        }

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
    async fn execute_one_round(&self) -> Result<()> {
        // 获取所有 Agent（从 Crew 中）
        // 注意：这里需要访问 Crew 的内部 agents，但 Crew 没有提供公开方法
        // 作为临时方案，我们先实现基本逻辑，后续可以扩展 Crew API

        // 清空已处理的消息
        let mut queue = self.message_queue.write().await;
        if queue.is_empty() {
            return Ok(());
        }

        // 取出一条消息进行处理
        if let Some(msg) = queue.pop_front() {
            drop(queue); // 释放锁

            tracing::debug!(
                "Processing message: type={}, sender={}",
                msg.msg_type,
                msg.sender
            );

            // 更新统计信息
            let mut stats = self.stats.write().await;
            stats.total_messages += 1;
            *stats.message_types.entry(msg.msg_type.clone()).or_insert(0) += 1;
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
