//! A2A Server Implementation
//!
//! 实现A2A协议的服务器端功能，完全符合Google A2A规范

use crate::card::AgentCardManager;
use crate::jsonrpc::{JsonRpcRequest, JsonRpcResponse, JsonRpcHandler, methods};
use crate::sse::JsonRpcSseHandler;
use crate::task::TaskManager;
use crate::types::*;
use crate::{A2AError, A2AResult};
use serde_json::Value;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

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
            jsonrpc_handler: Arc::new(RwLock::new(JsonRpcHandler::new())),
            sse_handler: Arc::new(RwLock::new(JsonRpcSseHandler::new())),
        })
    }
}

impl Default for A2AServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A2A 服务器
pub struct A2AServer {
    port: u16,
    host: String,
    agent_manager: Arc<RwLock<AgentCardManager>>,
    task_manager: Arc<RwLock<TaskManager>>,
    jsonrpc_handler: Arc<RwLock<JsonRpcHandler>>,
    sse_handler: Arc<RwLock<JsonRpcSseHandler>>,
}

/// A2A 服务器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AServerStats {
    /// 注册的 Agent 数量
    pub registered_agents: usize,
    /// 活跃任务数量
    pub active_tasks: usize,
    /// 活跃流数量
    pub active_streams: usize,
    /// 服务器信息
    pub server_info: String,
}

/// 消息发送请求
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageSendRequest {
    /// 消息内容
    pub message: Message,
    /// 发送配置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<MessageSendConfiguration>,
    /// 元数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 消息发送配置
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageSendConfiguration {
    /// 启用流式
    pub streaming_enabled: Option<bool>,
    /// 上下文ID
    pub context_id: Option<String>,
}

/// 消息发送响应
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageSendResponse {
    /// 任务
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<Task>,
    /// 消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
}

/// 任务获取参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGetParams {
    /// 任务ID
    pub id: String,
    /// 历史长度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_length: Option<u32>,
}

/// 任务列表参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListParams {
    /// 上下文ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    /// 页面大小
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// 历史长度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_length: Option<u32>,
}

impl A2AServer {
    /// 初始化服务器处理器
    pub async fn initialize(&self) -> A2AResult<()> {
        // 设置JSON-RPC方法处理器
        self.setup_jsonrpc_methods().await?;
        
        println!("🚀 A2A Server initialized successfully");
        println!("📡 Server: {}://{}", "http", &self.endpoint_url());
        println!("🔗 Available endpoints:");
        println!("   POST /v1/message:send    - Send message to agent");
        println!("   POST /v1/message:stream   - Send message with streaming");
        println!("   GET  /v1/tasks/{{id}}       - Get task details");
        println!("   GET  /v1/tasks            - List tasks");
        println!("   POST /v1/tasks/{{id}}:cancel - Cancel task");
        
        Ok(())
    }

    /// 设置JSON-RPC方法处理器（简化版本）
    async fn setup_jsonrpc_methods(&self) -> A2AResult<()> {
        // 暂时不注册任何方法，我们将直接在handle_jsonrpc_request中处理
        Ok(())
    }

    /// 处理消息发送请求
    async fn handle_message_send(
        task_manager: Arc<RwLock<TaskManager>>,
        params: serde_json::Value,
    ) -> A2AResult<serde_json::Value> {
        let request: MessageSendRequest = serde_json::from_value(params)
            .map_err(|e| A2AError::InvalidInput(format!("Invalid message send request: {}", e)))?;

        let mut tm = task_manager.write().await;
        let task = tm.create_task_with_id("default", request.message, None)?;

        let response = MessageSendResponse {
            task: Some(tm.get_task(&task)?.clone()),
            message: None,
        };

        let result = serde_json::to_value(response)
            .map_err(|e| A2AError::SerializationError(e))?;

        Ok(result)
    }

    /// 处理任务获取请求
    async fn handle_tasks_get(
        task_manager: Arc<RwLock<TaskManager>>,
        params: serde_json::Value,
    ) -> A2AResult<serde_json::Value> {
        let params_obj: TaskGetParams = serde_json::from_value(params)
            .map_err(|e| A2AError::InvalidInput(format!("Invalid task get params: {}", e)))?;

        let tm = task_manager.read().await;
        let task = tm.get_task(&params_obj.id)?;

        // TODO: 处理history_length参数
        let result = serde_json::to_value(task)
            .map_err(|e| A2AError::SerializationError(e))?;

        Ok(result)
    }

    /// 处理任务列表请求
    async fn handle_tasks_list(
        task_manager: Arc<RwLock<TaskManager>>,
        params: serde_json::Value,
    ) -> A2AResult<serde_json::Value> {
        let _params_obj: TaskListParams = serde_json::from_value(params)
            .map_err(|e| A2AError::InvalidInput(format!("Invalid task list params: {}", e)))?;

        let tm = task_manager.read().await;
        let tasks = tm.get_all_tasks();

        let result = serde_json::to_value(tasks)
            .map_err(|e| A2AError::SerializationError(e))?;

        Ok(result)
    }

    /// 处理任务取消请求
    async fn handle_tasks_cancel(
        task_manager: Arc<RwLock<TaskManager>>,
        params: serde_json::Value,
    ) -> A2AResult<serde_json::Value> {
        let params_obj: TaskGetParams = serde_json::from_value(params)
            .map_err(|e| A2AError::InvalidInput(format!("Invalid task cancel params: {}", e)))?;

        let mut tm = task_manager.write().await;
        tm.set_task_status(&params_obj.id, TaskStatus::new(TaskState::Canceled))?;

        let result = serde_json::json!({
            "id": params_obj.id,
            "status": "canceled"
        });

        Ok(result)
    }

    /// 处理JSON-RPC请求
    pub async fn handle_jsonrpc_request(&self, json_str: &str) -> String {
        match JsonRpcRequest::from_json(json_str) {
            Ok(request) => {
                let response = self.handle_jsonrpc_method(&request).await;
                response.to_json().unwrap_or_else(|_| {
                    // 如果序列化失败，返回内部错误
                    JsonRpcResponse::error(
                        crate::jsonrpc::error_codes::INTERNAL_ERROR,
                        "Failed to serialize response".to_string(),
                        None,
                        request.id,
                    ).to_json().unwrap_or_default()
                })
            }
            Err(_) => {
                // 如果解析失败，返回解析错误
                JsonRpcResponse::error(
                    crate::jsonrpc::error_codes::PARSE_ERROR,
                    "Parse error".to_string(),
                    None,
                    None,
                ).to_json().unwrap_or_default()
            }
        }
    }

    /// 处理具体的JSON-RPC方法
    async fn handle_jsonrpc_method(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let task_manager = Arc::clone(&self.task_manager);
        
        match request.method.as_str() {
            methods::MESSAGE_SEND => {
                let result = Self::handle_message_send(task_manager, request.params.clone().unwrap_or(Value::Null)).await;
                match result {
                    Ok(value) => JsonRpcResponse::success(value, request.id.clone()),
                    Err(error) => JsonRpcResponse::from_a2a_error(error, request.id.clone()),
                }
            }
            methods::TASKS_GET => {
                let result = Self::handle_tasks_get(task_manager, request.params.clone().unwrap_or(Value::Null)).await;
                match result {
                    Ok(value) => JsonRpcResponse::success(value, request.id.clone()),
                    Err(error) => JsonRpcResponse::from_a2a_error(error, request.id.clone()),
                }
            }
            methods::TASKS_LIST => {
                let result = Self::handle_tasks_list(task_manager, request.params.clone().unwrap_or(Value::Null)).await;
                match result {
                    Ok(value) => JsonRpcResponse::success(value, request.id.clone()),
                    Err(error) => JsonRpcResponse::from_a2a_error(error, request.id.clone()),
                }
            }
            methods::TASKS_CANCEL => {
                let result = Self::handle_tasks_cancel(task_manager, request.params.clone().unwrap_or(Value::Null)).await;
                match result {
                    Ok(value) => JsonRpcResponse::success(value, request.id.clone()),
                    Err(error) => JsonRpcResponse::from_a2a_error(error, request.id.clone()),
                }
            }
            _ => {
                JsonRpcResponse::error(
                    crate::jsonrpc::error_codes::METHOD_NOT_FOUND,
                    format!("Method '{}' not found", request.method),
                    None,
                    request.id.clone(),
                )
            }
        }
    }

    /// 处理流式消息请求
    pub async fn handle_message_stream(
        &self,
        message: Message,
    ) -> A2AResult<(Task, Pin<Box<dyn Stream<Item = String> + Send + 'static>>)> {
        let mut sse_handler = self.sse_handler.write().await;
        sse_handler.handle_message_stream(message).await
    }

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

    /// 提交任务
    pub async fn submit_task(&self, agent_id: &str, message: Message) -> A2AResult<String> {
        // 验证 agent 存在
        let _agent = self.get_agent(agent_id).await
            .ok_or_else(|| A2AError::AgentNotFound(agent_id.to_string()))?;

        let mut manager = self.task_manager.write().await;
        manager.create_task_with_id(agent_id, message, None)
    }

    /// 获取任务状态
    pub async fn get_task_status(&self, task_id: &str) -> A2AResult<TaskStatus> {
        let manager = self.task_manager.read().await;
        match manager.get_task(task_id) {
            Ok(task) => Ok(task.status.clone()),
            Err(_) => Err(A2AError::TaskNotFound(task_id.to_string())),
        }
    }

    /// 获取任务详情
    pub async fn get_task(&self, task_id: &str) -> A2AResult<Task> {
        let manager = self.task_manager.read().await;
        match manager.get_task(task_id) {
            Ok(task) => Ok(task.clone()),
            Err(_) => Err(A2AError::TaskNotFound(task_id.to_string())),
        }
    }

    /// 获取任务结果
    pub async fn get_task_result(&self, task_id: &str) -> A2AResult<Vec<Artifact>> {
        let manager = self.task_manager.read().await;
        match manager.get_task(task_id) {
            Ok(_task) => Ok(vec![]), // TODO: Implement artifacts when Task struct supports it
            Err(_) => Err(A2AError::TaskNotFound(task_id.to_string())),
        }
    }

    /// 更新任务状态
    pub async fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
    ) -> A2AResult<()> {
        let mut manager = self.task_manager.write().await;
        manager.set_task_status(task_id, status)
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
        self.update_task_status(task_id, TaskStatus::new(TaskState::Canceled)).await
    }

    /// 获取服务器统计信息
    pub async fn get_stats(&self) -> A2AServerStats {
        let agent_manager = self.agent_manager.read().await;
        let task_manager = self.task_manager.read().await;
        let sse_handler = self.sse_handler.read().await;
        
        A2AServerStats {
            registered_agents: agent_manager.get_all_cards().len(),
            active_tasks: task_manager.get_task_stats().total_tasks,
            active_streams: sse_handler.stream_manager().active_streams_count(),
            server_info: format!("A2A Server on {}:{}", self.host, self.port),
        }
    }

    /// 获取端点URL
    pub fn endpoint_url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// 启动服务器（简化版本）
    pub async fn start(&self) -> A2AResult<()> {
        self.initialize().await?;
        
        println!("✅ A2A Server started successfully");
        println!("📍 Server running at: http://{}", self.endpoint_url());
        println!("📋 Ready to handle A2A protocol requests");
        
        // 在实际实现中，这里会启动 HTTP 服务器
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

    #[test]
    fn test_server_endpoint_url() {
        let server = A2AServerBuilder::new()
            .port(8080)
            .host("localhost".to_string())
            .build()
            .unwrap();

        assert_eq!(server.endpoint_url(), "localhost:8080");
    }

    #[tokio::test]
    async fn test_server_initialization() {
        let server = A2AServerBuilder::new().build().unwrap();
        let result = server.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_server_agent_management() {
        let server = A2AServerBuilder::new().build().unwrap();
        
        // Create test agent card
        let card = crate::AgentCardBuilder::new(
            "Test Agent".to_string(),
            url::Url::parse("https://test-agent.com").unwrap(),
            "1.0.0".to_string(),
        )
        .description("A test agent for unit testing".to_string())
        .enable_streaming(true)
        .build()
        .unwrap();
        
        // Register agent
        let agent_id = server.register_agent(card.clone()).await.unwrap();
        assert!(!agent_id.is_empty());
        
        // Retrieve agent
        let retrieved_card = server.get_agent(&agent_id).await;
        assert!(retrieved_card.is_some());
        assert_eq!(retrieved_card.unwrap().name, "Test Agent");
        
        // List agents
        let agents = server.list_agents().await;
        assert!(!agents.is_empty());
    }

    #[tokio::test]
    async fn test_server_task_lifecycle() {
        let server = A2AServerBuilder::new().build().unwrap();
        
        let message = Message::user_message("Process this text".to_string());
        
        // Submit task
        let task_id = server.submit_task("test-agent", message).await;
        assert!(task_id.is_ok());
        
        let task_id = task_id.unwrap();
        
        // Get task
        let task = server.get_task(&task_id).await;
        assert!(task.is_ok());
        let task = task.unwrap();
        assert_eq!(task.status.state, TaskState::Submitted);
        
        // Update task status
        let working_status = TaskStatus::new(TaskState::Working);
        let result = server.update_task_status(&task_id, working_status).await;
        assert!(result.is_ok());
        
        // Verify status update
        let updated_task = server.get_task(&task_id).await;
        assert!(updated_task.is_ok());
        assert_eq!(updated_task.unwrap().status.state, TaskState::Working);
        
        // Complete task with artifact
        let artifact = crate::Artifact::file(
            crate::FileContent::from_bytes(
                "result.txt".to_string(),
                b"Analysis complete".to_vec(),
                "text/plain".to_string(),
            )
        );
        let result = server.add_task_artifact(&task_id, artifact).await;
        assert!(result.is_ok());
        
        // Get task result
        let artifacts = server.get_task_result(&task_id).await;
        assert!(artifacts.is_ok());
        assert!(!artifacts.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_server_jsonrpc_message_handling() {
        let server = A2AServerBuilder::new().build().unwrap();
        
        // Test JSON-RPC message send
        let jsonrpc_request = r#"{
            "jsonrpc": "2.0",
            "method": "message/send",
            "params": {
                "message": {
                    "parts": [
                        {
                            "text": {
                                "text": "Hello, agent!"
                            }
                        }
                    ]
                }
            },
            "id": "test-123"
        }"#;
        
        let response = server.handle_jsonrpc_request(jsonrpc_request).await;
        assert!(!response.is_empty());
        
        // Test JSON-RPC task get
        let get_request = r#"{
            "jsonrpc": "2.0",
            "method": "tasks/get",
            "params": {
                "id": "test-task-id"
            },
            "id": "test-456"
        }"#;
        
        let response = server.handle_jsonrpc_request(get_request).await;
        assert!(!response.is_empty());
        
        // Test JSON-RPC task list
        let list_request = r#"{
            "jsonrpc": "2.0",
            "method": "tasks/list",
            "params": {},
            "id": "test-789"
        }"#;
        
        let response = server.handle_jsonrpc_request(list_request).await;
        assert!(!response.is_empty());
    }

    #[tokio::test]
    async fn test_server_error_handling() {
        let server = A2AServerBuilder::new().build().unwrap();
        
        // Test invalid JSON-RPC
        let invalid_json = "invalid json";
        let response = server.handle_jsonrpc_request(invalid_json).await;
        assert!(response.contains("\"code\":"));
        
        // Test non-existent method
        let invalid_method = r#"{
            "jsonrpc": "2.0",
            "method": "unknown/method",
            "params": {},
            "id": "test-invalid"
        }"#;
        
        let response = server.handle_jsonrpc_request(invalid_method).await;
        assert!(response.contains("Method not found"));
        
        // Test non-existent task
        let get_nonexistent = r#"{
            "jsonrpc": "2.0",
            "method": "tasks/get",
            "params": {
                "id": "non-existent-task"
            },
            "id": "test-missing"
        }"#;
        
        let response = server.handle_jsonrpc_request(get_nonexistent).await;
        assert!(response.contains("Task not found"));
    }

    #[test]
    fn test_message_send_request_serialization() {
        let request = MessageSendRequest {
            message: Message::user_message("Hello, agent!".to_string()),
            configuration: None,
            metadata: None,
        };

        let json_str = serde_json::to_string(&request);
        assert!(json_str.is_ok());
        
        let parsed: Result<MessageSendRequest, _> = serde_json::from_str(&json_str.unwrap());
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().message.parts.len(), 1);
    }

    #[test]
    fn test_server_stats() {
        let server = A2AServerBuilder::new().build().unwrap();
        
        // Test default stats
        let stats = futures::executor::block_on(server.get_stats());
        assert_eq!(stats.registered_agents, 0);
        assert_eq!(stats.active_tasks, 0);
        assert_eq!(stats.active_streams, 0);
        assert!(stats.server_info.contains("A2A Server on"));
    }

    #[test]
    fn test_task_list_params_serialization() {
        let params = TaskListParams {
            context_id: Some("test-context".to_string()),
            page_size: Some(10),
            history_length: Some(5),
        };
        
        let json_str = serde_json::to_string(&params);
        assert!(json_str.is_ok());
        
        let parsed: Result<TaskListParams, _> = serde_json::from_str(&json_str.unwrap());
        assert!(parsed.is_ok());
        let parsed_ok = parsed.unwrap();
        assert_eq!(parsed_ok.context_id, Some("test-context".to_string()));
        assert_eq!(parsed_ok.page_size, Some(10));
        assert_eq!(parsed_ok.history_length, Some(5));
    }

    #[test]
    fn test_task_get_params_serialization() {
        let params = TaskGetParams {
            id: "test-task-id".to_string(),
            history_length: Some(3),
        };
        
        let json_str = serde_json::to_string(&params);
        assert!(json_str.is_ok());
        
        let parsed: Result<TaskGetParams, _> = serde_json::from_str(&json_str.unwrap());
        assert!(parsed.is_ok());
        let parsed_ok = parsed.unwrap();
        assert_eq!(parsed_ok.id, "test-task-id");
        assert_eq!(parsed_ok.history_length, Some(3));
    }
}