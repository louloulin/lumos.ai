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

impl A2AServer {
    /// 注册 Agent Card
    pub fn register_agent(&mut self, card: AgentCard) -> A2AResult<()> {
        let agent_id = card.generate_id();
        self.agent_cards.insert(agent_id, card);
        Ok(())
    }

    /// 获取 Agent Card
    pub fn get_agent_card(&self, agent_id: &str) -> A2AResult<&AgentCard> {
        self.agent_cards
            .get(agent_id)
            .ok_or_else(|| A2AError::AgentNotFound(agent_id.to_string()))
    }

    /// 提交任务
    pub fn submit_task(&mut self, task: Task) -> A2AResult<String> {
        let task_id = task.id.clone();
        self.tasks.insert(task_id.clone(), task);
        Ok(task_id)
    }

    /// 获取任务状态
    pub fn get_task_status(&self, task_id: &str) -> A2AResult<&TaskStatus> {
        self.tasks
            .get(task_id)
            .map(|task| &task.status)
            .ok_or_else(|| A2AError::TaskNotFound(task_id.to_string()))
    }

    /// 更新任务状态
    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> A2AResult<()> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = status;
            Ok(())
        } else {
            Err(A2AError::TaskNotFound(task_id.to_string()))
        }
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