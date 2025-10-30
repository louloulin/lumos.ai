//! 简化的 SOP 环境实现
//!
//! 不依赖 Crew，专注于 SOP 消息路由和执行流程

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::sop_types::{AgentAction, SopExecutionMode, SopMessage, SopStats};
use super::Agent;
use crate::error::{Error, Result};

/// 简化的 SOP 环境
///
/// 轻量级的消息协调器，管理 Agent 间的 SOP 消息流转
///
/// # 示例
///
/// ```ignore
/// let env = SimpleSopEnvironment::new("team", SopExecutionMode::React);
/// env.add_agent(agent1).await;
/// env.add_agent(agent2).await;
///
/// let initial_msg = SopMessage::broadcast("start", "system", json!({}));
/// let stats = env.run(Some(initial_msg)).await?;
/// ```
pub struct SimpleSopEnvironment {
    /// 环境名称
    name: String,

    /// SOP 执行模式
    execution_mode: SopExecutionMode,

    /// 消息队列
    message_queue: Arc<RwLock<VecDeque<SopMessage>>>,

    /// Agent 列表
    agents: Arc<RwLock<Vec<Arc<dyn Agent>>>>,

    /// Agent 完成状态
    agent_done_status: Arc<RwLock<HashMap<String, bool>>>,

    /// 执行顺序（用于 ByOrder 模式）
    execution_order: Arc<RwLock<Vec<String>>>,

    /// 统计信息
    stats: Arc<RwLock<SopStats>>,

    /// 最大迭代次数
    max_iterations: usize,
}

impl SimpleSopEnvironment {
    /// 创建新的 SOP 环境
    pub fn new(name: impl Into<String>, execution_mode: SopExecutionMode) -> Self {
        Self {
            name: name.into(),
            execution_mode,
            message_queue: Arc::new(RwLock::new(VecDeque::new())),
            agents: Arc::new(RwLock::new(Vec::new())),
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

    /// 添加 Agent
    pub async fn add_agent(&self, agent: Arc<dyn Agent>) {
        let agent_name = agent.get_name().to_string();
        self.agents.write().await.push(agent);
        self.agent_done_status
            .write()
            .await
            .insert(agent_name, false);
    }

    /// 设置执行顺序（用于 ByOrder 模式）
    pub async fn set_execution_order(&self, order: Vec<String>) {
        *self.execution_order.write().await = order;
    }

    /// 发布消息到环境
    pub async fn publish_message(&self, message: SopMessage) {
        tracing::debug!(
            "Publishing message: type={}, sender={}",
            message.msg_type,
            message.sender
        );

        self.message_queue.write().await.push_back(message);
    }

    /// 获取 Agent 关注的消息
    async fn get_messages_for_agent(&self, agent: &Arc<dyn Agent>) -> Vec<SopMessage> {
        let watch_list = agent.sop_watch();
        if watch_list.is_empty() {
            return Vec::new();
        }

        let queue = self.message_queue.read().await;
        queue
            .iter()
            .filter(|msg| {
                // 检查消息类型是否在 watch 列表中
                watch_list.contains(&msg.msg_type)
                    // 或者消息是发给这个 Agent 的
                    || msg.receiver.as_ref().map(|r| r == &agent.get_name()).unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// 执行 SOP 流程
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

            // 短暂延迟，避免忙等待
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
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
        for agent_name in order {
            tracing::info!("Executing agent: {}", agent_name);

            // 找到对应的 Agent
            let agents = self.agents.read().await;
            if let Some(agent) = agents.iter().find(|a| a.get_name() == agent_name) {
                // 获取消息
                let messages = self.get_messages_for_agent(agent).await;

                // Think
                let action = agent.sop_think(messages).await?;

                // Act
                let response_msg = agent.sop_act(action).await?;

                // 发布响应消息
                self.publish_message(response_msg).await;

                // 更新统计
                let mut stats = self.stats.write().await;
                stats.total_messages += 1;
            }
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
        // 取出一条消息进行处理
        let msg = {
            let mut queue = self.message_queue.write().await;
            queue.pop_front()
        };

        let msg = match msg {
            Some(m) => m,
            None => return Ok(()), // 没有消息，直接返回
        };

        tracing::debug!(
            "Processing message: type={}, sender={}",
            msg.msg_type,
            msg.sender
        );

        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_messages += 1;
            *stats.message_types.entry(msg.msg_type.clone()).or_insert(0) += 1;
        }

        // 遍历所有 Agent，让它们 watch-think-act
        let agents = self.agents.read().await.clone();
        let mut new_messages = Vec::new();

        for agent in agents.iter() {
            // Watch: 检查 Agent 是否关注这个消息
            let watch_list = agent.sop_watch();
            let is_interested = watch_list.contains(&msg.msg_type)
                || msg
                    .receiver
                    .as_ref()
                    .map(|r| r == &agent.get_name())
                    .unwrap_or(false);

            if !is_interested {
                continue; // 不关注，跳过
            }

            tracing::debug!(
                "Agent {} is processing message type: {}",
                agent.get_name(),
                msg.msg_type
            );

            // Think: 决定如何响应
            let action = match agent.sop_think(vec![msg.clone()]).await {
                Ok(action) => action,
                Err(e) => {
                    tracing::error!("Agent {} think failed: {}", agent.get_name(), e);
                    continue;
                }
            };

            // 检查是否是 NoOp
            if matches!(action, AgentAction::NoOp) {
                tracing::debug!("Agent {} decided to do nothing", agent.get_name());
                continue;
            }

            // Act: 执行行动
            let response_msg = match agent.sop_act(action).await {
                Ok(msg) => msg,
                Err(e) => {
                    tracing::error!("Agent {} act failed: {}", agent.get_name(), e);
                    continue;
                }
            };

            tracing::debug!(
                "Agent {} produced message type: {}",
                agent.get_name(),
                response_msg.msg_type
            );

            new_messages.push(response_msg);

            // 检查 Agent 是否完成
            if agent.sop_is_done() {
                let agent_name = agent.get_name().to_string();
                self.agent_done_status
                    .write()
                    .await
                    .insert(agent_name.clone(), true);
                tracing::info!("Agent {} marked as done", agent_name);
            }
        }

        // 将新消息加入队列
        if !new_messages.is_empty() {
            let mut queue = self.message_queue.write().await;
            for msg in new_messages {
                queue.push_back(msg);
            }
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

    /// 获取环境名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_simple_sop_environment_creation() {
        let env = SimpleSopEnvironment::new("test_env", SopExecutionMode::React);
        let stats = env.get_stats().await;

        assert_eq!(stats.total_messages, 0);
        assert_eq!(stats.active_agents, 0);
        assert_eq!(env.name(), "test_env");
    }

    #[tokio::test]
    async fn test_message_publishing() {
        let env = SimpleSopEnvironment::new("test_env", SopExecutionMode::React);

        let msg = SopMessage::broadcast("test_type", "sender", json!({"key": "value"}));
        env.publish_message(msg).await;

        let queue = env.message_queue.read().await;
        assert_eq!(queue.len(), 1);
    }
}
