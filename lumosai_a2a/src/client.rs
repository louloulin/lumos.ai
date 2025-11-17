//! A2A Client Implementation
//!
//! 实现 A2A 协议的客户端功能，用于与其他 Agent 进行通信

use crate::types::*;
use crate::{A2AError, A2AResult};
use reqwest::Client;
use serde_json;
use std::time::Duration;

/// A2A 客户端
#[derive(Debug)]
pub struct A2AClient {
    client: Client,
    base_url: String,
}

impl A2AClient {
    /// 创建新的 A2A 客户端
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("LumosAI-A2A-Client/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client, base_url }
    }

    /// 获取 Agent Card
    pub async fn get_agent_card(&self, agent_id: &str) -> A2AResult<AgentCard> {
        let url = format!("{}/agent/{}", self.base_url, agent_id);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let card: AgentCard = response.json().await?;
            Ok(card)
        } else {
            Err(A2AError::AgentNotFound(agent_id.to_string()))
        }
    }

    /// 提交任务
    pub async fn submit_task(&self, task: &Task) -> A2AResult<String> {
        let url = format!("{}/tasks", self.base_url);
        let response = self.client.post(&url).json(task).send().await?;
        
        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            if let Some(task_id) = result.get("task_id").and_then(|v| v.as_str()) {
                Ok(task_id.to_string())
            } else {
                Err(A2AError::ProtocolError("Invalid response format".to_string()))
            }
        } else {
            Err(A2AError::NetworkError(response.error_for_status().unwrap_err()))
        }
    }

    /// 获取任务状态
    pub async fn get_task_status(&self, task_id: &str) -> A2AResult<TaskStatus> {
        let url = format!("{}/tasks/{}/status", self.base_url, task_id);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let status: TaskStatus = response.json().await?;
            Ok(status)
        } else if response.status() == 404 {
            Err(A2AError::TaskNotFound(task_id.to_string()))
        } else {
            Err(A2AError::NetworkError(response.error_for_status().unwrap_err()))
        }
    }

    /// 获取任务结果
    pub async fn get_task_result(&self, task_id: &str) -> A2AResult<Vec<Artifact>> {
        let url = format!("{}/tasks/{}/result", self.base_url, task_id);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let artifacts: Vec<Artifact> = response.json().await?;
            Ok(artifacts)
        } else if response.status() == 404 {
            Err(A2AError::TaskNotFound(task_id.to_string()))
        } else {
            Err(A2AError::NetworkError(response.error_for_status().unwrap_err()))
        }
    }
}

/// A2A 客户端构建器
pub struct A2AClientBuilder {
    base_url: String,
    timeout: Duration,
    user_agent: String,
    api_key: Option<String>,
}

impl A2AClientBuilder {
    /// 创建新的构建器
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            timeout: Duration::from_secs(30),
            user_agent: "LumosAI-A2A-Client/1.0".to_string(),
            api_key: None,
        }
    }

    /// 设置超时时间
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 设置用户代理
    pub fn user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }

    /// 设置 API 密钥
    pub fn api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// 构建客户端
    pub fn build(self) -> A2AResult<A2AClient> {
        let mut client_builder = Client::builder()
            .timeout(self.timeout)
            .user_agent(self.user_agent);

        if let Some(api_key) = &self.api_key {
            client_builder = client_builder.bearer_auth(api_key);
        }

        let client = client_builder
            .build()
            .map_err(|e| A2AError::InternalError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(A2AClient {
            client,
            base_url: self.base_url,
        })
    }
}

impl Default for A2AClient {
    fn default() -> Self {
        Self::new("http://localhost:8080".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_builder() {
        let client = A2AClientBuilder::new("https://example.com".to_string())
            .timeout(Duration::from_secs(60))
            .api_key("test-key".to_string())
            .build();

        assert!(client.is_ok());
    }
}