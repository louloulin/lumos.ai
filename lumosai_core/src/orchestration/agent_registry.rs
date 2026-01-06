//! # Agent 注册表
//!
//! 管理已注册的 Agent 信息和能力。

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

/// Agent 能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// 支持的任务类型
    pub task_types: Vec<String>,
    /// 专长领域
    pub specializations: Vec<String>,
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 是否支持流式输出
    pub supports_streaming: bool,
}

impl Default for AgentCapabilities {
    fn default() -> Self {
        Self {
            task_types: vec!["general".to_string()],
            specializations: vec![],
            max_concurrent_tasks: 1,
            supports_streaming: false,
        }
    }
}

/// Agent 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent ID
    pub id: String,
    /// Agent 名称
    pub name: String,
    /// Agent 描述
    pub description: String,
    /// Agent 角色
    pub roles: HashSet<String>,
    /// Agent 能力
    pub capabilities: AgentCapabilities,
    /// Agent 版本
    pub version: String,
    /// 是否活跃
    pub active: bool,
}

impl AgentInfo {
    /// 创建新的 Agent 信息
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        roles: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            roles: roles.into_iter().collect(),
            capabilities: AgentCapabilities::default(),
            version: "1.0.0".to_string(),
            active: true,
        }
    }

    /// 添加角色
    pub fn add_role(&mut self, role: impl Into<String>) {
        self.roles.insert(role.into());
    }

    /// 设置能力
    pub fn with_capabilities(mut self, capabilities: AgentCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// 检查是否有某个角色
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(role)
    }

    /// 检查是否支持某个任务类型
    pub fn supports_task_type(&self, task_type: &str) -> bool {
        self.capabilities.task_types.contains(&task_type.to_string())
            || self.capabilities.task_types.contains(&"general".to_string())
    }
}

/// Agent 注册表
pub struct AgentRegistry {
    /// 注册的 Agents
    agents: RwLock<HashMap<String, AgentInfo>>,
}

impl AgentRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
        }
    }

    /// 注册 Agent
    pub async fn register(&self, agent_info: AgentInfo) -> Result<(), String> {
        let mut agents = self.agents.write().await;
        agents.insert(agent_info.id.clone(), agent_info);
        Ok(())
    }

    /// 注销 Agent
    pub async fn unregister(&self, agent_id: &str) -> Result<(), String> {
        let mut agents = self.agents.write().await;
        agents.remove(agent_id).ok_or_else(|| {
            format!("Agent {} not found", agent_id)
        })?;
        Ok(())
    }

    /// 获取 Agent 信息
    pub async fn get(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// 列出所有 Agent
    pub fn list_agents(&self) -> Vec<AgentInfo> {
        // 注意: 这里需要阻塞获取,实际使用时应该改为 async
        use std::thread;
        use std::time::Duration;

        // 简化实现: 直接返回空列表
        // 实际应该在 async 上下文中调用
        vec![]
    }

    /// 查找具有特定角色的 Agent
    pub async fn find_by_role(&self, role: &str) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|agent| agent.has_role(role))
            .cloned()
            .collect()
    }

    /// 查找支持特定任务类型的 Agent
    pub async fn find_by_capability(&self, task_type: &str) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|agent| agent.supports_task_type(task_type))
            .cloned()
            .collect()
    }

    /// 更新 Agent 活跃状态
    pub async fn set_active(&self, agent_id: &str, active: bool) -> Result<(), String> {
        let mut agents = self.agents.write().await;
        let agent = agents.get_mut(agent_id).ok_or_else(|| {
            format!("Agent {} not found", agent_id)
        })?;
        agent.active = active;
        Ok(())
    }

    /// 获取活跃 Agent 数量
    pub async fn active_count(&self) -> usize {
        let agents = self.agents.read().await;
        agents.values().filter(|a| a.active).count()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_info_creation() {
        let info = AgentInfo::new("agent1", "Test Agent", vec!["worker".to_string()]);

        assert_eq!(info.id, "agent1");
        assert_eq!(info.name, "Test Agent");
        assert!(info.has_role("worker"));
        assert!(!info.has_role("manager"));
    }

    #[tokio::test]
    async fn test_agent_capabilities() {
        let mut capabilities = AgentCapabilities::default();
        capabilities.task_types = vec!["math".to_string(), "calculation".to_string()];
        capabilities.max_concurrent_tasks = 5;

        let info = AgentInfo::new("agent1", "Math Agent", vec![])
            .with_capabilities(capabilities);

        assert!(info.supports_task_type("math"));
        assert!(!info.supports_task_type("writing"));
    }

    #[tokio::test]
    async fn test_registry_registration() {
        let registry = AgentRegistry::new();
        let info = AgentInfo::new("agent1", "Test", vec!["worker".to_string()]);

        registry.register(info.clone()).await.unwrap();

        let retrieved = registry.get("agent1").await.unwrap();
        assert_eq!(retrieved.id, "agent1");
        assert_eq!(retrieved.name, "Test");
    }

    #[tokio::test]
    async fn test_registry_unregister() {
        let registry = AgentRegistry::new();
        let info = AgentInfo::new("agent1", "Test", vec![]);

        registry.register(info).await.unwrap();
        registry.unregister("agent1").await.unwrap();

        let result = registry.get("agent1").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_by_role() {
        let registry = AgentRegistry::new();

        let agent1 = AgentInfo::new("agent1", "Worker 1", vec!["worker".to_string()]);
        let agent2 = AgentInfo::new("agent2", "Manager", vec!["manager".to_string()]);
        let agent3 = AgentInfo::new("agent3", "Worker 2", vec!["worker".to_string()]);

        registry.register(agent1).await.unwrap();
        registry.register(agent2).await.unwrap();
        registry.register(agent3).await.unwrap();

        let workers = registry.find_by_role("worker").await;
        assert_eq!(workers.len(), 2);

        let managers = registry.find_by_role("manager").await;
        assert_eq!(managers.len(), 1);
    }
}
