//! # Multi-Agent 核心协调器
//!
//! 负责管理和协调多个 Agent 的执行。

use crate::orchestration::{
    agent_registry::AgentRegistry,
    communication::MessageBus,
    patterns::OrchestrationPattern,
    router::TaskRouter,
    task_queue::TaskQueue,
    OrchestrationError, OrchestrationResult,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 协调器配置
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// 最大并发 Agent 数
    pub max_concurrent_agents: usize,
    /// 任务超时时间 (秒)
    pub task_timeout_seconds: u64,
    /// 是否启用日志
    pub enable_logging: bool,
    /// 重试次数
    pub max_retries: usize,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_agents: 10,
            task_timeout_seconds: 300,
            enable_logging: true,
            max_retries: 3,
        }
    }
}

/// Multi-Agent 协调器
pub struct MultiAgentCoordinator {
    /// 配置
    config: CoordinatorConfig,
    /// 编排模式
    pattern: OrchestrationPattern,
    /// Agent 注册表
    registry: Arc<AgentRegistry>,
    /// 任务路由器
    router: Arc<TaskRouter>,
    /// 消息总线
    message_bus: Arc<MessageBus>,
    /// 任务队列
    task_queue: Arc<TaskQueue>,
}

impl MultiAgentCoordinator {
    /// 创建新的协调器
    pub fn new(pattern: OrchestrationPattern) -> Self {
        Self::with_config(pattern, CoordinatorConfig::default())
    }

    /// 使用配置创建协调器
    pub fn with_config(pattern: OrchestrationPattern, config: CoordinatorConfig) -> Self {
        Self {
            pattern,
            registry: Arc::new(AgentRegistry::new()),
            router: Arc::new(TaskRouter::new()),
            message_bus: Arc::new(MessageBus::new()),
            task_queue: Arc::new(TaskQueue::new()),
            config,
        }
    }

    /// 获取注册表引用
    pub fn registry(&self) -> Arc<AgentRegistry> {
        Arc::clone(&self.registry)
    }

    /// 获取消息总线引用
    pub fn message_bus(&self) -> Arc<MessageBus> {
        Arc::clone(&self.message_bus)
    }

    /// 执行任务
    pub async fn execute(&self, input: &str) -> Result<OrchestrationResult, OrchestrationError> {
        let start = std::time::Instant::now();

        // 根据模式执行
        let result = match &self.pattern {
            OrchestrationPattern::Hierarchical => self.execute_hierarchical(input).await?,
            OrchestrationPattern::Flat => self.execute_flat(input).await?,
            OrchestrationPattern::Pipeline => self.execute_pipeline(input).await?,
            OrchestrationPattern::Graph => self.execute_graph(input).await?,
        };

        let duration = start.elapsed();

        Ok(OrchestrationResult {
            success: true,
            output: result,
            agents_involved: vec![],
            duration_ms: duration.as_millis() as u64,
            intermediate_results: vec![],
            error: None,
        })
    }

    /// 层级模式执行
    async fn execute_hierarchical(&self, input: &str) -> Result<String, OrchestrationError> {
        // 简化实现: 查找 manager 角色 Agent
        let agents = self.registry.list_agents();
        let manager = agents
            .iter()
            .find(|a| a.roles.contains(&"manager".to_string()))
            .ok_or_else(|| OrchestrationError::AgentNotRegistered("No manager agent found".to_string()))?;

        // 发送消息给 manager
        self.message_bus
            .send_string(&manager.id, input.to_string())
            .await
            .map_err(|e| OrchestrationError::CommunicationError(e.to_string()))?;

        // 等待响应
        let response = self
            .message_bus
            .receive(&manager.id)
            .await
            .map_err(|e| OrchestrationError::CommunicationError(e.to_string()))?;

        Ok(response.content)
    }

    /// 平等模式执行
    async fn execute_flat(&self, input: &str) -> Result<String, OrchestrationError> {
        // 简化实现: 所有 Agent 平等协作
        let agents = self.registry.list_agents();

        if agents.is_empty() {
            return Err(OrchestrationError::AgentNotRegistered(
                "No agents registered".to_string(),
            ));
        }

        // 广播任务
        for agent in &agents {
            self.message_bus
                .send_string(&agent.id, input.to_string())
                .await
                .map_err(|e| OrchestrationError::CommunicationError(e.to_string()))?;
        }

        // 收集所有响应
        let mut results = Vec::new();
        for agent in &agents {
            if let Ok(response) = self.message_bus.receive(&agent.id).await {
                results.push(response.content);
            }
        }

        // 简单合并结果
        Ok(results.join("\n---\n"))
    }

    /// 流水线模式执行
    async fn execute_pipeline(&self, input: &str) -> Result<String, OrchestrationError> {
        // 简化实现: 按顺序执行 Agent
        let agents = self.registry.list_agents();

        let mut current_input = input.to_string();

        for agent in &agents {
            self.message_bus
                .send_string(&agent.id, current_input.clone())
                .await
                .map_err(|e| OrchestrationError::CommunicationError(e.to_string()))?;

            let response = self
                .message_bus
                .receive(&agent.id)
                .await
                .map_err(|e| OrchestrationError::CommunicationError(e.to_string()))?;

            current_input = response.content;
        }

        Ok(current_input)
    }

    /// 图模式执行
    async fn execute_graph(&self, input: &str) -> Result<String, OrchestrationError> {
        // 简化实现: 类似层级模式,但支持更复杂的依赖关系
        self.execute_hierarchical(input).await
    }

    /// 获取配置
    pub fn config(&self) -> &CoordinatorConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_creation() {
        let coordinator = MultiAgentCoordinator::new(OrchestrationPattern::Flat);
        assert_eq!(coordinator.config().max_concurrent_agents, 10);
    }

    #[test]
    fn test_config_default() {
        let config = CoordinatorConfig::default();
        assert_eq!(config.max_concurrent_agents, 10);
        assert_eq!(config.task_timeout_seconds, 300);
        assert_eq!(config.max_retries, 3);
    }
}
