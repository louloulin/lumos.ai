//! A2A JSON-RPC 2.0 Transport Layer
//!
//! 实现A2A协议的JSON-RPC 2.0传输层，完全符合Google A2A规范

use crate::types::*;
use crate::{A2AError, A2AResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;

/// JSON-RPC 2.0 请求对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC版本，必须为"2.0"
    pub jsonrpc: String,
    /// 方法名
    pub method: String,
    /// 参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// 请求ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonRpcId>,
}

/// JSON-RPC 2.0 ID类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId {
    String(String),
    Number(i64),
    Null,
}

/// JSON-RPC 2.0 响应对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC版本，必须为"2.0"
    pub jsonrpc: String,
    /// 结果或错误
    #[serde(flatten)]
    pub payload: JsonRpcPayload,
    /// 请求ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonRpcId>,
}

/// JSON-RPC 2.0 响应载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcPayload {
    Result(Value),
    Error(JsonRpcError),
}

/// JSON-RPC 2.0 错误对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// 错误代码
    pub code: i64,
    /// 错误消息
    pub message: String,
    /// 错误数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// A2A JSON-RPC 方法常量
pub mod methods {
    pub const MESSAGE_SEND: &str = "message/send";
    pub const MESSAGE_STREAM: &str = "message/stream";
    pub const TASKS_GET: &str = "tasks/get";
    pub const TASKS_LIST: &str = "tasks/list";
    pub const TASKS_CANCEL: &str = "tasks/cancel";
    pub const TASKS_SET_ARTIFACT: &str = "tasks/setArtifact";
}

impl JsonRpcRequest {
    /// 创建新的JSON-RPC请求
    pub fn new(method: String, params: Option<Value>, id: Option<JsonRpcId>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id,
        }
    }

    /// 创建消息发送请求
    pub fn message_send(message: Message, id: Option<JsonRpcId>) -> Self {
        let params = Some(json!({
            "message": message
        }));
        Self::new(methods::MESSAGE_SEND.to_string(), params, id)
    }

    /// 创建流式消息请求
    pub fn message_stream(message: Message, id: Option<JsonRpcId>) -> Self {
        let params = Some(json!({
            "message": message
        }));
        Self::new(methods::MESSAGE_STREAM.to_string(), params, id)
    }

    /// 创建获取任务请求
    pub fn tasks_get(task_id: &str, history_length: Option<u32>, id: Option<JsonRpcId>) -> Self {
        let mut params = json!({
            "id": task_id
        });
        
        if let Some(length) = history_length {
            params["historyLength"] = json!(length);
        }
        
        Self::new(methods::TASKS_GET.to_string(), Some(params), id)
    }

    /// 创建任务列表请求
    pub fn tasks_list(context_id: Option<String>, page_size: Option<u32>, 
                      history_length: Option<u32>, id: Option<JsonRpcId>) -> Self {
        let mut params = json!({});
        
        if let Some(ctx_id) = context_id {
            params["contextId"] = json!(ctx_id);
        }
        
        if let Some(size) = page_size {
            params["pageSize"] = json!(size);
        }
        
        if let Some(length) = history_length {
            params["historyLength"] = json!(length);
        }
        
        Self::new(methods::TASKS_LIST.to_string(), Some(params), id)
    }

    /// 创建取消任务请求
    pub fn tasks_cancel(task_id: &str, id: Option<JsonRpcId>) -> Self {
        let params = Some(json!({
            "id": task_id
        }));
        Self::new(methods::TASKS_CANCEL.to_string(), params, id)
    }

    /// 序列化为JSON字符串
    pub fn to_json(&self) -> A2AResult<String> {
        serde_json::to_string(self)
            .map_err(|e| A2AError::SerializationError(e))
    }

    /// 从JSON字符串反序列化
    pub fn from_json(json_str: &str) -> A2AResult<Self> {
        serde_json::from_str(json_str)
            .map_err(|e| A2AError::SerializationError(e))
    }
}

impl JsonRpcResponse {
    /// 创建成功响应
    pub fn success(result: Value, id: Option<JsonRpcId>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            payload: JsonRpcPayload::Result(result),
            id,
        }
    }

    /// 创建错误响应
    pub fn error(code: i64, message: String, data: Option<Value>, id: Option<JsonRpcId>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            payload: JsonRpcPayload::Error(JsonRpcError {
                code,
                message,
                data,
            }),
            id,
        }
    }

    /// 从A2AError创建错误响应
    pub fn from_a2a_error(error: A2AError, id: Option<JsonRpcId>) -> Self {
        let (code, message) = match error {
            A2AError::InvalidInput(msg) => (-32602, msg),
            A2AError::AgentNotFound(msg) => (-32601, format!("Agent not found: {}", msg)),
            A2AError::TaskNotFound(msg) => (-32601, format!("Task not found: {}", msg)),
            A2AError::ProtocolError(msg) => (-32603, format!("Protocol error: {}", msg)),
            A2AError::AuthenticationFailed(msg) => (-32001, format!("Authentication failed: {}", msg)),
            A2AError::TaskFailed(msg) => (-32002, format!("Task failed: {}", msg)),
            A2AError::CapabilityNotSupported(msg) => (-32003, format!("Capability not supported: {}", msg)),
            A2AError::TimeoutError(msg) => (-32004, format!("Timeout error: {}", msg)),
            A2AError::InternalError(msg) => (-32603, format!("Internal error: {}", msg)),
            A2AError::NetworkError(_) => (-32005, "Network error".to_string()),
            A2AError::SerializationError(_) => (-32700, "Parse error".to_string()),
        };

        Self::error(code, message, None, id)
    }

    /// 序列化为JSON字符串
    pub fn to_json(&self) -> A2AResult<String> {
        serde_json::to_string(self)
            .map_err(|e| A2AError::SerializationError(e))
    }

    /// 从JSON字符串反序列化
    pub fn from_json(json_str: &str) -> A2AResult<Self> {
        serde_json::from_str(json_str)
            .map_err(|e| A2AError::SerializationError(e))
    }

    /// 获取结果
    pub fn get_result(&self) -> Option<&Value> {
        match &self.payload {
            JsonRpcPayload::Result(result) => Some(result),
            _ => None,
        }
    }

    /// 获取错误
    pub fn get_error(&self) -> Option<&JsonRpcError> {
        match &self.payload {
            JsonRpcPayload::Error(error) => Some(error),
            _ => None,
        }
    }

    /// 检查是否为成功响应
    pub fn is_success(&self) -> bool {
        matches!(&self.payload, JsonRpcPayload::Result(_))
    }
}

/// JSON-RPC 错误代码常量
pub mod error_codes {
    /// 无效请求
    pub const INVALID_REQUEST: i64 = -32600;
    /// 无效参数
    pub const INVALID_PARAMS: i64 = -32602;
    /// 方法未找到
    pub const METHOD_NOT_FOUND: i64 = -32601;
    /// 内部错误
    pub const INTERNAL_ERROR: i64 = -32603;
    /// 解析错误
    pub const PARSE_ERROR: i64 = -32700;
    /// 认证失败
    pub const AUTHENTICATION_FAILED: i64 = -32001;
    /// 任务失败
    pub const TASK_FAILED: i64 = -32002;
    /// 功能不支持
    pub const CAPABILITY_NOT_SUPPORTED: i64 = -32003;
    /// 超时错误
    pub const TIMEOUT_ERROR: i64 = -32004;
    /// 网络错误
    pub const NETWORK_ERROR: i64 = -32005;
}

/// JSON-RPC 传输处理器
pub struct JsonRpcHandler {
    methods: HashMap<String, Box<dyn Fn(Value) -> A2AResult<Value> + Send + Sync>>,
}

impl JsonRpcHandler {
    /// 创建新的JSON-RPC处理器
    pub fn new() -> Self {
        Self {
            methods: HashMap::new(),
        }
    }

    /// 注册方法处理器
    pub fn register_method<F>(&mut self, method_name: &str, handler: F)
    where
        F: Fn(Value) -> A2AResult<Value> + Send + Sync + 'static,
    {
        self.methods.insert(method_name.to_string(), Box::new(handler));
    }

    /// 从JSON字符串处理请求
    pub fn handle_json_request(&self, json_str: &str) -> String {
        match JsonRpcRequest::from_json(json_str) {
            Ok(request) => {
                let response = self.handle_request(&request);
                response.to_json().unwrap_or_else(|_| {
                    // 如果序列化失败，返回内部错误
                    JsonRpcResponse::error(
                        error_codes::INTERNAL_ERROR,
                        "Failed to serialize response".to_string(),
                        None,
                        None,
                    ).to_json().unwrap_or_default()
                })
            }
            Err(_) => {
                // 如果解析失败，返回解析错误
                JsonRpcResponse::error(
                    error_codes::PARSE_ERROR,
                    "Parse error".to_string(),
                    None,
                    None,
                ).to_json().unwrap_or_default()
            }
        }
    }

    /// 处理JSON-RPC请求
    fn handle_request(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        // 检查方法是否存在
        if let Some(method) = self.methods.get(&request.method) {
            let params = request.params.clone().unwrap_or(Value::Null);
            match method(params) {
                Ok(result) => JsonRpcResponse::success(result, request.id.clone()),
                Err(error) => JsonRpcResponse::from_a2a_error(error, request.id.clone()),
            }
        } else {
            JsonRpcResponse::error(
                error_codes::METHOD_NOT_FOUND,
                format!("Method '{}' not found", request.method),
                None,
                request.id.clone(),
            )
        }
    }
}

impl Default for JsonRpcHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request_creation() {
        let message = Message::user_message("Hello".to_string());
        let request = JsonRpcRequest::message_send(message, Some(JsonRpcId::String("test-123".to_string())));
        
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "message/send");
        assert!(request.params.is_some());
    }

    #[test]
    fn test_jsonrpc_response_success() {
        let result = json!({"status": "ok"});
        let response = JsonRpcResponse::success(result, Some(JsonRpcId::String("test-123".to_string())));
        
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.is_success());
        assert!(response.get_result().is_some());
        assert!(response.get_error().is_none());
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let response = JsonRpcResponse::error(
            error_codes::INVALID_PARAMS,
            "Invalid parameters".to_string(),
            None,
            Some(JsonRpcId::String("test-123".to_string())),
        );
        
        assert_eq!(response.jsonrpc, "2.0");
        assert!(!response.is_success());
        assert!(response.get_result().is_none());
        assert!(response.get_error().is_some());
    }

    #[test]
    fn test_jsonrpc_handler() {
        let mut handler = JsonRpcHandler::new();
        
        // 注册测试方法
        handler.register_method("test.echo", |params| {
            Ok(params)
        });
        
        let request = JsonRpcRequest::new(
            "test.echo".to_string(),
            Some(json!({"message": "hello"})),
            Some(JsonRpcId::String("test-123".to_string())),
        );
        
        let response = handler.handle_request(request);
        assert!(response.is_success());
    }
}