//! # Crew Manager
//!
//! 管理 Agent 团队(Crew)的协作和执行。

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{
    agent_registry::AgentRegistry,
    communication::MessageBus,
    task_queue::{Task, TaskQueue},
    OrchestrationError, OrchestrationResult,
};

/// Agent 角色
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    /// 管理者 - 协调其他 Agent
    Manager,
    /// 研究者 - 信息收集和分析
    Researcher,
    /// 写作者 - 内容生成
    Writer,
    /// 审核者 - 质量检查
    Reviewer,
    /// 执行者 - 任务执行
    Executor,
    /// 自定义角色
    Custom(String),
}

impl AgentRole {
    /// 从字符串创建角色
    pub fn from_str(s: impl Into<String>) -> Self {
        match s.into().as_str() {
            "manager" => AgentRole::Manager,
            "researcher" => AgentRole::Researcher,
            "writer" => AgentRole::Writer,
            "reviewer" => AgentRole::Reviewer,
            "executor" => AgentRole::Executor,
            other => AgentRole::Custom(other.to_string()),
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &str {
        match self {
            AgentRole::Manager => "manager",
            AgentRole::Researcher => "researcher",
            AgentRole::Writer => "writer",
            AgentRole::Reviewer => "reviewer",
            AgentRole::Executor => "executor",
            AgentRole::Custom(s) => s,
        }
    }
}

/// Crew 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewConfig {
    /// Crew 名称
    pub name: String,
    /// Crew 描述
    pub description: String,
    /// 最大并行任务数
    pub max_parallel_tasks: usize,
    /// 是否启用自动重试
    pub enable_retry: bool,
    /// 重试次数
    pub max_retries: usize,
}

impl Default for CrewConfig {
    fn default() -> Self {
        Self {
            name: "Default Crew".to_string(),
            description: String::new(),
            max_parallel_tasks: 5,
            enable_retry: true,
            max_retries: 3,
        }
    }
}

/// Crew Manager
pub struct CrewManager {
    /// 配置
    config: CrewConfig,
    /// Agent 注册表
    registry: Arc<AgentRegistry>,
    /// 消息总线
    message_bus: Arc<MessageBus>,
    /// 任务队列
    task_queue: Arc<TaskQueue>,
    /// Crew 成员 (agent_id -> roles)
    members: RwLock<HashMap<String, HashSet<AgentRole>>>,
}

impl CrewManager {
    /// 创建新的 Crew Manager
    pub fn new(
        config: CrewConfig,
        registry: Arc<AgentRegistry>,
        message_bus: Arc<MessageBus>,
        task_queue: Arc<TaskQueue>,
    ) -> Self {
        Self {
            config,
            registry,
            message_bus,
            task_queue,
            members: RwLock::new(HashMap::new()),
        }
    }

    /// 添加成员
    pub async fn add_member(&self, agent_id: impl Into<String>, roles: Vec<AgentRole>) {
        let agent_id = agent_id.into();
        let role_set: HashSet<AgentRole> = roles.into_iter().collect();

        let mut members = self.members.write().await;
        members.insert(agent_id, role_set);
    }

    /// 移除成员
    pub async fn remove_member(&self, agent_id: &str) {
        let mut members = self.members.write().await;
        members.remove(agent_id);
    }

    /// 获取具有特定角色的成员
    pub async fn get_members_with_role(&self, role: &AgentRole) -> Vec<String> {
        let members = self.members.read().await;
        members
            .iter()
            .filter(|(_, roles)| roles.contains(role))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// 执行任务 (使用 Crew 协作)
    pub async fn execute_task(&self, task_input: &str) -> Result<OrchestrationResult, OrchestrationError> {
        let start = std::time::Instant::now();

        // 1. 创建任务
        let task = Task::new("crew_task", task_input);

        // 2. 查找合适的 Agent
        let agents = self.find_available_agents().await?;

        if agents.is_empty() {
            return Err(OrchestrationError::AgentNotRegistered(
                "No available agents".to_string(),
            ));
        }

        // 3. 简化实现: 发送给第一个可用的 Agent
        let primary_agent = &agents[0];

        self.message_bus
            .send_string(primary_agent, task_input.to_string())
            .await
            .map_err(|e| OrchestrationError::CommunicationError(e.to_string()))?;

        // 4. 等待响应
        let response = self
            .message_bus
            .receive(primary_agent)
            .await
            .map_err(|e| OrchestrationError::CommunicationError(e.to_string()))?;

        let duration = start.elapsed();

        Ok(OrchestrationResult {
            success: true,
            output: response.content,
            agents_involved: agents,
            duration_ms: duration.as_millis() as u64,
            intermediate_results: vec![],
            error: None,
        })
    }

    /// 执行协作任务 (多个 Agent 协作)
    pub async fn execute_collaborative_task(
        &self,
        task_input: &str,
        required_roles: Vec<AgentRole>,
    ) -> Result<OrchestrationResult, OrchestrationError> {
        let start = std::time::Instant::now();

        let mut all_results = Vec::new();
        let mut involved_agents = Vec::new();

        // 为每个角色找到 Agent 并执行
        for role in &required_roles {
            let agents = self.get_members_with_role(role).await;

            if agents.is_empty() {
                continue; // 跳过没有 Agent 的角色
            }

            let agent_id = &agents[0];
            involved_agents.push(agent_id.clone());

            // 发送任务
            let _ = self
                .message_bus
                .send_string(agent_id, format!("[{:?}] {}", role, task_input))
                .await;

            // 收集结果 (非阻塞)
            if let Ok(response) = self.message_bus.receive(agent_id).await {
                all_results.push(response.content);
            }
        }

        let duration = start.elapsed();

        // 合并所有结果
        let combined_output = if all_results.is_empty() {
            "No results from agents".to_string()
        } else {
            all_results.join("\n---\n")
        };

        Ok(OrchestrationResult {
            success: !all_results.is_empty(),
            output: combined_output,
            agents_involved: involved_agents,
            duration_ms: duration.as_millis() as u64,
            intermediate_results: vec![],
            error: None,
        })
    }

    /// 查找可用的 Agents
    async fn find_available_agents(&self) -> Result<Vec<String>, OrchestrationError> {
        let members = self.members.read().await;
        Ok(members.keys().cloned().collect())
    }

    /// 获取 Crew 统计信息
    pub async fn stats(&self) -> CrewStats {
        let members = self.members.read().await;
        let queue_stats = self.task_queue.stats().await;

        let mut role_counts: HashMap<String, usize> = HashMap::new();

        for roles in members.values() {
            for role in roles {
                let key = role.as_str().to_string();
                *role_counts.entry(key).or_insert(0) += 1;
            }
        }

        CrewStats {
            total_members: members.len(),
            role_counts,
            pending_tasks: queue_stats.pending,
            running_tasks: queue_stats.running,
            completed_tasks: queue_stats.completed,
        }
    }
}

/// Crew 统计信息
#[derive(Debug, Clone)]
pub struct CrewStats {
    /// 总成员数
    pub total_members: usize,
    /// 角色分布
    pub role_counts: HashMap<String, usize>,
    /// 待处理任务
    pub pending_tasks: usize,
    /// 执行中任务
    pub running_tasks: usize,
    /// 已完成任务
    pub completed_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_role_conversion() {
        assert_eq!(AgentRole::from_str("manager"), AgentRole::Manager);
        assert_eq!(AgentRole::from_str("custom_role"), AgentRole::Custom("custom_role".to_string()));
        assert_eq!(AgentRole::Manager.as_str(), "manager");
    }

    #[tokio::test]
    async fn test_crew_manager_creation() {
        let registry = Arc::new(AgentRegistry::new());
        let message_bus = Arc::new(MessageBus::new());
        let task_queue = Arc::new(TaskQueue::new());

        let config = CrewConfig::default();
        let manager = CrewManager::new(config, registry, message_bus, task_queue);

        // 添加成员
        manager.add_member("agent1", vec![AgentRole::Manager]).await;
        manager.add_member("agent2", vec![AgentRole::Worker]).await;

        let stats = manager.stats().await;
        assert_eq!(stats.total_members, 2);
    }

    #[tokio::test]
    async fn test_get_members_with_role() {
        let registry = Arc::new(AgentRegistry::new());
        let message_bus = Arc::new(MessageBus::new());
        let task_queue = Arc::new(TaskQueue::new());

        let manager = CrewManager::new(
            CrewConfig::default(),
            registry,
            message_bus,
            task_queue,
        );

        manager.add_member("agent1", vec![AgentRole::Manager, AgentRole::Researcher]).await;
        manager.add_member("agent2", vec![AgentRole::Worker]).await;
        manager.add_member("agent3", vec![AgentRole::Researcher]).await;

        let researchers = manager.get_members_with_role(&AgentRole::Researcher).await;
        assert_eq!(researchers.len(), 2);
        assert!(researchers.contains(&"agent1".to_string()));
        assert!(researchers.contains(&"agent3".to_string()));
    }
}
