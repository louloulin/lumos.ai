//! A2A Server-Sent Events (SSE) Streaming Implementation
//!
//! 实现A2A协议的Server-Sent Events流式传输，完全符合Google A2A规范

use crate::jsonrpc::*;
use crate::types::*;
use crate::{A2AError, A2AResult};
use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::pin::Pin;
use tokio::sync::mpsc;

/// SSE事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SseEvent {
    /// 任务更新
    TaskUpdate {
        /// 任务ID
        task_id: String,
        /// 任务状态
        #[serde(skip_serializing_if = "Option::is_none")]
        task: Option<Task>,
        /// 状态更新
        #[serde(skip_serializing_if = "Option::is_none")]
        status_update: Option<TaskStatusUpdateEvent>,
    },
    /// 消息事件
    Message {
        /// 任务ID
        task_id: String,
        /// 消息内容
        message: Message,
    },
    /// 工件更新
    ArtifactUpdate {
        /// 任务ID
        task_id: String,
        /// 工件
        artifact: Artifact,
    },
    /// 错误事件
    Error {
        /// 任务ID
        task_id: String,
        /// 错误信息
        error: String,
        /// 错误代码
        #[serde(skip_serializing_if = "Option::is_none")]
        error_code: Option<i64>,
    },
    /// 流结束
    End {
        /// 任务ID
        task_id: String,
        /// 最终结果
        #[serde(skip_serializing_if = "Option::is_none")]
        final_result: Option<String>,
    },
}

/// 任务状态更新事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusUpdateEvent {
    /// 任务ID
    pub task_id: String,
    /// 新状态
    pub state: TaskState,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// SSE事件发送器
#[derive(Debug, Clone)]
pub struct SseEventSender {
    /// 内部发送器
    sender: mpsc::UnboundedSender<SseEvent>,
    /// 任务ID
    task_id: String,
}

impl SseEventSender {
    /// 创建新的SSE事件发送器
    pub fn new(task_id: String) -> (Self, Pin<Box<dyn Stream<Item = String> + Send + 'static>>) {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        
        let stream = Box::pin(async_stream::stream! {
            while let Some(event) = receiver.recv().await {
                if let Ok(sse_data) = event_to_sse_data(&event) {
                    yield sse_data;
                }
            }
        });
        
        let sse_sender = Self {
            sender,
            task_id,
        };
        
        (sse_sender, stream)
    }

    /// 发送任务更新
    pub fn send_task_update(&self, task: &Task) -> A2AResult<()> {
        let event = SseEvent::TaskUpdate {
            task_id: self.task_id.clone(),
            task: Some(task.clone()),
            status_update: None,
        };
        
        self.sender.send(event)
            .map_err(|_| A2AError::InternalError("SSE stream closed".to_string()))
    }

    /// 发送状态更新
    pub fn send_status_update(&self, state: TaskState, message: Option<Message>) -> A2AResult<()> {
        let status_update = TaskStatusUpdateEvent {
            task_id: self.task_id.clone(),
            state,
            timestamp: Utc::now(),
            message,
            metadata: None,
        };

        let event = SseEvent::TaskUpdate {
            task_id: self.task_id.clone(),
            task: None,
            status_update: Some(status_update),
        };
        
        self.sender.send(event)
            .map_err(|_| A2AError::InternalError("SSE stream closed".to_string()))
    }

    /// 发送消息
    pub fn send_message(&self, message: Message) -> A2AResult<()> {
        let event = SseEvent::Message {
            task_id: self.task_id.clone(),
            message,
        };
        
        self.sender.send(event)
            .map_err(|_| A2AError::InternalError("SSE stream closed".to_string()))
    }

    /// 发送工件更新
    pub fn send_artifact(&self, artifact: Artifact) -> A2AResult<()> {
        let event = SseEvent::ArtifactUpdate {
            task_id: self.task_id.clone(),
            artifact,
        };
        
        self.sender.send(event)
            .map_err(|_| A2AError::InternalError("SSE stream closed".to_string()))
    }

    /// 发送错误
    pub fn send_error(&self, error: String, error_code: Option<i64>) -> A2AResult<()> {
        let event = SseEvent::Error {
            task_id: self.task_id.clone(),
            error,
            error_code,
        };
        
        self.sender.send(event)
            .map_err(|_| A2AError::InternalError("SSE stream closed".to_string()))
    }

    /// 发送结束事件
    pub fn send_end(&self, final_result: Option<String>) -> A2AResult<()> {
        let event = SseEvent::End {
            task_id: self.task_id.clone(),
            final_result,
        };
        
        self.sender.send(event)
            .map_err(|_| A2AError::InternalError("SSE stream closed".to_string()))
    }

    /// 检查连接是否活跃
    pub fn is_connected(&self) -> bool {
        !self.sender.is_closed()
    }
}

/// 将SSE事件转换为SSE数据格式
fn event_to_sse_data(event: &SseEvent) -> A2AResult<String> {
    let json_str = serde_json::to_string(event)
        .map_err(|e| A2AError::SerializationError(e))?;
    
    Ok(format!("data: {}\n\n", json_str))
}

/// SSE流管理器
#[derive(Debug)]
pub struct SseStreamManager {
    /// 活跃流
    streams: HashMap<String, SseEventSender>,
}

impl SseStreamManager {
    /// 创建新的SSE流管理器
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
        }
    }

    /// 创建新的SSE流
    pub fn create_stream(&mut self, task_id: &str) -> (SseEventSender, Pin<Box<dyn Stream<Item = String> + Send + 'static>>) {
        let (sender, stream) = SseEventSender::new(task_id.to_string());
        
        // 存储发送器引用（克隆）
        let sender_clone = sender.clone();
        self.streams.insert(task_id.to_string(), sender_clone);
        
        (sender, stream)
    }

    /// 获取流发送器
    pub fn get_sender(&self, task_id: &str) -> Option<&SseEventSender> {
        self.streams.get(task_id)
    }

    /// 移除流
    pub fn remove_stream(&mut self, task_id: &str) {
        self.streams.remove(task_id);
    }

    /// 广播事件到所有流
    pub fn broadcast_event(&self, event: &SseEvent) -> A2AResult<()> {
        let mut errors = Vec::new();
        
        for (task_id, sender) in &self.streams {
            if let Err(e) = sender.sender.send(event.clone()) {
                errors.push((task_id.clone(), e));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(A2AError::InternalError(
                format!("Failed to send event to {} streams", errors.len())
            ))
        }
    }

    /// 获取活跃流数量
    pub fn active_streams_count(&self) -> usize {
        self.streams.len()
    }

    /// 清理已关闭的流
    pub fn cleanup_closed_streams(&mut self) {
        self.streams.retain(|_, sender| sender.is_connected());
    }
}

impl Default for SseStreamManager {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON-RPC SSE消息处理器
#[derive(Debug)]
pub struct JsonRpcSseHandler {
    /// SSE流管理器
    stream_manager: SseStreamManager,
}

impl JsonRpcSseHandler {
    /// 创建新的SSE处理器
    pub fn new() -> Self {
        Self {
            stream_manager: SseStreamManager::new(),
        }
    }

    /// 处理流式消息请求
    pub async fn handle_message_stream(&mut self, message: Message) -> A2AResult<(Task, Pin<Box<dyn Stream<Item = String> + Send + 'static>>)> {
        // 创建任务
        let task = Task::new("Streaming message".to_string(), message);
        let task_id = task.id.clone();
        
        // 创建SSE流
        let (sse_sender, sse_stream) = self.stream_manager.create_stream(&task_id);
        
        // 发送初始任务状态
        if let Err(e) = sse_sender.send_task_update(&task) {
            return Err(A2AError::InternalError(format!("Failed to send initial task update: {}", e)));
        }
        
        Ok((task, sse_stream))
    }

    /// 获取流管理器
    pub fn stream_manager(&self) -> &SseStreamManager {
        &self.stream_manager
    }

    /// 获取可变流管理器
    pub fn stream_manager_mut(&mut self) -> &mut SseStreamManager {
        &mut self.stream_manager
    }
}

impl Default for JsonRpcSseHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// SSE事件聚合器，用于将多个事件组合成JSON-RPC响应格式
pub struct SseEventAggregator {
    /// 聚合的事件
    events: Vec<SseEvent>,
}

impl SseEventAggregator {
    /// 创建新的聚合器
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    /// 添加事件
    pub fn add_event(&mut self, event: SseEvent) {
        self.events.push(event);
    }

    /// 生成最终的任务对象
    pub fn build_task(&self, initial_task: &Task) -> Task {
        let mut task = initial_task.clone();
        
        // 分析事件并更新任务状态
        for event in &self.events {
            match event {
                SseEvent::TaskUpdate { status_update: Some(status_update), .. } => {
                    task.status = TaskStatus {
                        state: status_update.state.clone(),
                        message: status_update.message.clone(),
                        timestamp: status_update.timestamp,
                        metadata: status_update.metadata.clone(),
                    };
                    task.updated_at = status_update.timestamp;
                }
                SseEvent::ArtifactUpdate { .. } => {
                    // TODO: 添加工件到任务（当Task结构支持时）
                }
                _ => {}
            }
        }
        
        task
    }

    /// 生成JSON-RPC响应
    pub fn build_response(&self, initial_task: &Task, request_id: Option<JsonRpcId>) -> JsonRpcResponse {
        let final_task = self.build_task(initial_task);
        let result = json!({
            "task": final_task
        });
        
        JsonRpcResponse::success(result, request_id)
    }
}

impl Default for SseEventAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_event_creation() {
        let task_id = "test-task-123".to_string();
        let event = SseEvent::TaskUpdate {
            task_id: task_id.clone(),
            task: None,
            status_update: Some(TaskStatusUpdateEvent {
                task_id,
                state: TaskState::Working,
                timestamp: Utc::now(),
                message: None,
                metadata: None,
            }),
        };
        
        let sse_data = event_to_sse_data(&event).unwrap();
        assert!(sse_data.starts_with("data: {"));
        assert!(sse_data.ends_with("\n\n"));
    }

    #[test]
    fn test_sse_event_sender() {
        let task_id = "test-task-456".to_string();
        let (sender, mut _stream) = SseEventSender::new(task_id);
        
        assert!(sender.is_connected());
        
        let result = sender.send_status_update(TaskState::Completed, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sse_stream_manager() {
        let mut manager = SseStreamManager::new();
        assert_eq!(manager.active_streams_count(), 0);
        
        let task_id = "test-task-789";
        let (sender, _stream) = manager.create_stream(task_id);
        assert_eq!(manager.active_streams_count(), 1);
        
        assert!(manager.get_sender(task_id).is_some());
        assert!(sender.is_connected());
        
        manager.remove_stream(task_id);
        assert_eq!(manager.active_streams_count(), 0);
    }

    #[test]
    fn test_sse_event_aggregator() {
        let mut aggregator = SseEventAggregator::new();
        
        let message = Message::user_message("Test message".to_string());
        let mut task = Task::new("Test task".to_string(), message);
        
        // 添加状态更新事件
        let status_event = SseEvent::TaskUpdate {
            task_id: task.id.clone(),
            task: None,
            status_update: Some(TaskStatusUpdateEvent {
                task_id: task.id.clone(),
                state: TaskState::Completed,
                timestamp: Utc::now(),
                message: Some(Message::agent_message("Task completed".to_string())),
                metadata: None,
            }),
        };
        
        aggregator.add_event(status_event);
        
        let final_task = aggregator.build_task(&task);
        assert_eq!(final_task.status.state, TaskState::Completed);
    }
}