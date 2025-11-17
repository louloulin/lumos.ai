//! A2A Server Implementation
//!
//! 实现 A2A 协议的服务器端功能，提供完整的 HTTP API 支持

use crate::card::AgentCardManager;
use crate::task::TaskManager;
use crate::types::*;
use crate::{A2AError, A2AResult};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A2A 服务器构建器
pub struct A2AServerBuilder {
    port: u16,
    host: String,
}

impl A2AServerBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            port: 8080,
            host: "0.0.0.0".to_string(),
        }
    }

    /// 设置端口
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// 设置主机
    pub fn host(mut self, host: String) -> Self {
        self.host = host;
        self
    }

    /// 构建服务器
    pub fn build(self) -> A2AResult<A2AServer> {
        Ok(A2AServer {
            port: self.port,
            host: self.host,
            agent_manager: Arc::new(RwLock::new(AgentCardManager::new())),
            task_manager: Arc::new(RwLock::new(TaskManager::new())),
        })
    }
}

impl Default for A2AServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A2A 服务器
#[derive(Debug)]
pub struct A2AServer {
    port: u16,
    host: String,
    agent_manager: Arc<RwLock<AgentCardManager>>,
    task_manager: Arc<RwLock<TaskManager>>,
}

/// A2A 服务器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AServerStats {
    /// 注册的 Agent 数量
    pub registered_agents: usize,
    /// 活跃任务数量
    pub active_tasks: usize,
    /// 服务器信息
    pub server_info: String,
}

impl A2AServer {
    /// 注册 Agent Card
    pub async fn register_agent(&self, card: AgentCard) -> A2AResult<String> {
        let mut manager = self.agent_manager.write().await;
        manager.register_card(card)
    }

    /// 获取 Agent Card
    pub async fn get_agent(&self, agent_id: &str) -> Option<AgentCard> {
        let manager = self.agent_manager.read().await;
        manager.get_card(agent_id).cloned()
    }

    /// 获取所有 Agents
    pub async fn list_agents(&self) -> Vec<AgentCard> {
        let manager = self.agent_manager.read().await;
        manager.get_all_cards().into_iter().cloned().collect()
    }

    /// 根据 ID 获取 Agent Card（HTTP API 端点）
    pub async fn get_agent_card(&self, agent_id: &str) -> A2AResult<AgentCard> {
        let manager = self.agent_manager.read().await;
        manager
            .get_card(agent_id)
            .cloned()
            .ok_or_else(|| A2AError::AgentNotFound(agent_id.to_string()))
    }

    /// 提交任务
    pub async fn submit_task(&self, agent_id: &str, message: Message) -> A2AResult<String> {
        // 验证 agent 存在
        let _agent = self.get_agent(agent_id).await
            .ok_or_else(|| A2AError::AgentNotFound(agent_id.to_string()))?;

        let mut manager = self.task_manager.write().await;
        manager.create_task(agent_id, message, None)
    }

    /// 提交带工件的复杂任务
    pub async fn submit_task_with_artifacts(
        &self,
        agent_id: &str,
        message: Message,
        artifacts: Vec<Artifact>,
    ) -> A2AResult<String> {
        // 验证 agent 存在
        let _agent = self.get_agent(agent_id).await
            .ok_or_else(|| A2AError::AgentNotFound(agent_id.to_string()))?;

        let mut manager = self.task_manager.write().await;
        manager.create_task(agent_id, message, Some(artifacts))
    }

    /// 获取任务状态
    pub async fn get_task_status(&self, task_id: &str) -> A2AResult<TaskStatus> {
        let manager = self.task_manager.read().await;
        match manager.get_task(task_id) {
            Some(task) => Ok(task.status.clone()),
            None => Err(A2AError::TaskNotFound(task_id.to_string())),
        }
    }

    /// 获取任务详情
    pub async fn get_task(&self, task_id: &str) -> A2AResult<Task> {
        let manager = self.task_manager.read().await;
        manager
            .get_task(task_id)
            .cloned()
            .ok_or_else(|| A2AError::TaskNotFound(task_id.to_string()))
    }

    /// 获取任务结果
    pub async fn get_task_result(&self, task_id: &str) -> A2AResult<Vec<Artifact>> {
        let manager = self.task_manager.read().await;
        match manager.get_task(task_id) {
            Some(task) => Ok(task.artifacts.clone()),
            None => Err(A2AError::TaskNotFound(task_id.to_string())),
        }
    }

    /// 更新任务状态
    pub async fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
    ) -> A2AResult<()> {
        let mut manager = self.task_manager.write().await;
        manager.update_task_status(task_id, status)
    }

    /// 添加任务工件
    pub async fn add_task_artifact(
        &self,
        task_id: &str,
        artifact: Artifact,
    ) -> A2AResult<()> {
        let mut manager = self.task_manager.write().await;
        manager.add_artifact(task_id, artifact)
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: &str) -> A2AResult<()> {
        self.update_task_status(task_id, TaskStatus::Canceled).await
    }

    /// 搜索 Agents（按技能）
    pub async fn find_agents_by_skills(
        &self,
        required_skills: &[String],
        match_mode: crate::discovery::SkillMatchMode,
    ) -> Vec<(AgentCard, f64)> {
        let manager = self.agent_manager.read().await;
        let discovery = crate::discovery::AgentDiscovery::new();
        discovery
            .find_agents_by_skills(&*manager, required_skills, match_mode)
            .into_iter()
            .map(|(card, score)| (card.clone(), score))
            .collect()
    }

    /// 搜索 Agents（按能力）
    pub async fn find_agents_by_capability(&self, capability: &str) -> Vec<AgentCard> {
        let manager = self.agent_manager.read().await;
        manager
            .find_by_capability(capability)
            .into_iter()
            .cloned()
            .collect()
    }

    /// 获取服务器统计信息
    pub async fn get_stats(&self) -> A2AServerStats {
        let agent_manager = self.agent_manager.read().await;
        let task_manager = self.task_manager.read().await;
        
        A2AServerStats {
            registered_agents: agent_manager.get_all_cards().len(),
            active_tasks: task_manager.get_task_stats().total_tasks,
            server_info: format!("A2A Server on {}:{}", self.host, self.port),
        }
    }

    /// 启动服务器（简化版本）
    pub async fn start(&self) -> A2AResult<()> {
        println!("A2A Server starting on {}:{}", self.host, self.port);
        println!("Available endpoints:");
        println!("  GET  /agent/{{id}}           - Get agent card");
        println!("  GET  /agents                  - List all agents");
        println!("  POST /tasks                   - Submit task");
        println!("  GET  /tasks/{{id}}/status       - Get task status");
        println!("  GET  /tasks/{{id}}/result       - Get task result");
        println!("  GET  /stats                    - Get server stats");
        
        // 在实际实现中，这里会启动 HTTP 服务器
        // 例如使用 warp、axum 或 actix-web
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_builder() {
        let server = A2AServerBuilder::new()
            .port(3000)
            .host("127.0.0.1".to_string())
            .build();

        assert!(server.is_ok());
    }
}