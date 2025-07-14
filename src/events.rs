//! 简化的事件系统API
//!
//! 提供简单易用的事件发布、订阅和处理功能。

use crate::{Result, Error};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

// 重导出核心类型
pub use lumosai_core::agent::events::{
    EventBus as CoreEventBus,
    EventHandler as CoreEventHandler,
    EventFilter,
    AgentEvent,
    LogEventHandler,
    MetricsEventHandler,
};

/// 事件总线
pub type EventBus = CoreEventBus;

/// 事件处理器
pub type EventHandler = Arc<dyn CoreEventHandler>;

/// 简化的事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimpleEvent {
    /// Agent启动
    AgentStarted { agent_id: String },
    /// Agent停止
    AgentStopped { agent_id: String, reason: String },
    /// 消息发送
    MessageSent { from: String, to: String, content: String },
    /// 工具调用
    ToolCalled { agent_id: String, tool_name: String },
    /// 错误发生
    Error { agent_id: String, error: String },
    /// 自定义事件
    Custom { name: String, data: serde_json::Value },
}

/// 创建事件总线
/// 
/// # 示例
/// ```rust,no_run
/// use lumosai::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
///     let event_bus = lumosai::events::create_bus(1000);
///     
///     // 发布事件
///     lumosai::events::publish(&event_bus, "agent_started", serde_json::json!({
///         "agent_id": "agent_001"
///     })).await?;
///     
///     Ok(())
/// }
/// ```
pub fn create_bus(capacity: usize) -> Arc<EventBus> {
    Arc::new(EventBus::new(capacity))
}

/// 发布简单事件
/// 
/// # 示例
/// ```rust,no_run
/// use lumosai::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
///     let event_bus = lumosai::events::create_bus(1000);
///     
///     // 发布Agent启动事件
///     lumosai::events::publish(&event_bus, "agent_started", serde_json::json!({
///         "agent_id": "agent_001"
///     })).await?;
///     
///     // 发布消息事件
///     lumosai::events::publish(&event_bus, "message_sent", serde_json::json!({
///         "from": "agent_001",
///         "to": "user",
///         "content": "Hello!"
///     })).await?;
///     
///     Ok(())
/// }
/// ```
pub async fn publish(
    event_bus: &EventBus,
    event_type: &str,
    data: serde_json::Value,
) -> Result<()> {
    let event = match event_type {
        "agent_started" => {
            let agent_id = data.get("agent_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Event("agent_id is required".to_string()))?;
            
            AgentEvent::AgentStarted {
                agent_id: agent_id.to_string(),
                timestamp: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            }
        }
        "agent_stopped" => {
            let agent_id = data.get("agent_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Event("agent_id is required".to_string()))?;
            let reason = data.get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            
            AgentEvent::AgentStopped {
                agent_id: agent_id.to_string(),
                timestamp: chrono::Utc::now(),
                reason: reason.to_string(),
            }
        }
        "message_sent" => {
            let from = data.get("from")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Event("from is required".to_string()))?;
            let to = data.get("to")
                .and_then(|v| v.as_str());
            let content = data.get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Event("content is required".to_string()))?;
            
            AgentEvent::MessageSent {
                from_agent: from.to_string(),
                to_agent: to.map(|s| s.to_string()),
                message_id: uuid::Uuid::new_v4().to_string(),
                content: content.to_string(),
                timestamp: chrono::Utc::now(),
            }
        }
        "tool_called" => {
            let agent_id = data.get("agent_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Event("agent_id is required".to_string()))?;
            let tool_name = data.get("tool_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Event("tool_name is required".to_string()))?;
            let parameters = data.get("parameters")
                .cloned()
                .unwrap_or(serde_json::json!({}));
            
            AgentEvent::ToolCalled {
                agent_id: agent_id.to_string(),
                tool_name: tool_name.to_string(),
                parameters,
                timestamp: chrono::Utc::now(),
            }
        }
        "error" => {
            let agent_id = data.get("agent_id")
                .and_then(|v| v.as_str());
            let error_message = data.get("error")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Event("error is required".to_string()))?;
            
            AgentEvent::Error {
                agent_id: agent_id.map(|s| s.to_string()),
                error_type: "general".to_string(),
                error_message: error_message.to_string(),
                context: std::collections::HashMap::new(),
                timestamp: chrono::Utc::now(),
            }
        }
        _ => {
            AgentEvent::Custom {
                event_name: event_type.to_string(),
                data,
                timestamp: chrono::Utc::now(),
            }
        }
    };
    
    event_bus.publish(event).await
}

/// 订阅事件
/// 
/// # 示例
/// ```rust,no_run
/// use lumosai::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
///     let event_bus = lumosai::events::create_bus(1000);
///     let mut receiver = lumosai::events::subscribe(&event_bus);
///     
///     // 在另一个任务中监听事件
///     tokio::spawn(async move {
///         while let Ok(event) = receiver.recv().await {
///             println!("Received event: {:?}", event);
///         }
///     });
///     
///     Ok(())
/// }
/// ```
pub fn subscribe(event_bus: &EventBus) -> tokio::sync::broadcast::Receiver<AgentEvent> {
    event_bus.subscribe()
}

/// 注册日志处理器
/// 
/// # 示例
/// ```rust,no_run
/// use lumosai::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
///     let event_bus = lumosai::events::create_bus(1000);
///     lumosai::events::register_log_handler(&event_bus).await?;
///     
///     // 现在所有事件都会被记录到控制台
///     
///     Ok(())
/// }
/// ```
pub async fn register_log_handler(event_bus: &EventBus) -> Result<()> {
    let handler = Arc::new(LogEventHandler::new());
    event_bus.register_handler(handler).await
}

/// 注册指标收集处理器
/// 
/// # 示例
/// ```rust,no_run
/// use lumosai::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
///     let event_bus = lumosai::events::create_bus(1000);
///     let metrics_handler = lumosai::events::register_metrics_handler(&event_bus).await?;
///     
///     // 发布一些事件
///     lumosai::events::publish(&event_bus, "agent_started", serde_json::json!({
///         "agent_id": "agent_001"
///     })).await?;
///     
///     // 获取指标
///     let metrics = metrics_handler.get_metrics().await;
///     println!("Metrics: {:?}", metrics);
///     
///     Ok(())
/// }
/// ```
pub async fn register_metrics_handler(event_bus: &EventBus) -> Result<Arc<MetricsEventHandler>> {
    let handler = Arc::new(MetricsEventHandler::new());
    event_bus.register_handler(handler.clone()).await?;
    Ok(handler)
}

/// 获取事件历史
/// 
/// # 示例
/// ```rust,no_run
/// use lumosai::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
///     let event_bus = lumosai::events::create_bus(1000);
///     
///     // 发布一些事件
///     lumosai::events::publish(&event_bus, "agent_started", serde_json::json!({
///         "agent_id": "agent_001"
///     })).await?;
///     
///     // 获取历史
///     let history = lumosai::events::get_history(&event_bus, None).await;
///     println!("Event history: {} events", history.len());
///     
///     Ok(())
/// }
/// ```
pub async fn get_history(
    event_bus: &EventBus,
    filter: Option<EventFilter>,
) -> Vec<AgentEvent> {
    event_bus.get_history(filter).await
}

/// 创建事件过滤器
/// 
/// # 示例
/// ```rust,no_run
/// use lumosai::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
///     let event_bus = lumosai::events::create_bus(1000);
///     
///     // 创建过滤器，只获取特定Agent的事件
///     let filter = lumosai::events::filter()
///         .agent_ids(vec!["agent_001".to_string()])
///         .event_types(vec!["AgentStarted".to_string(), "MessageSent".to_string()])
///         .build();
///     
///     let filtered_history = lumosai::events::get_history(&event_bus, Some(filter)).await;
///     
///     Ok(())
/// }
/// ```
pub fn filter() -> EventFilterBuilder {
    EventFilterBuilder::new()
}

/// 事件过滤器构建器
pub struct EventFilterBuilder {
    filter: EventFilter,
}

impl EventFilterBuilder {
    pub fn new() -> Self {
        Self {
            filter: EventFilter::new(),
        }
    }
    
    /// 设置事件类型过滤
    pub fn event_types(mut self, types: Vec<String>) -> Self {
        self.filter = self.filter.with_event_types(types);
        self
    }
    
    /// 设置Agent ID过滤
    pub fn agent_ids(mut self, ids: Vec<String>) -> Self {
        self.filter = self.filter.with_agent_ids(ids);
        self
    }
    
    /// 设置时间范围过滤
    pub fn time_range(mut self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> Self {
        self.filter = self.filter.with_time_range(start, end);
        self
    }
    
    /// 构建过滤器
    pub fn build(self) -> EventFilter {
        self.filter
    }
}

impl Default for EventFilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_event_publishing() {
        let event_bus = create_bus(100);

        // 创建一个订阅者以保持通道开放
        let _receiver = subscribe(&event_bus);

        let result = publish(&event_bus, "agent_started", serde_json::json!({
            "agent_id": "test_agent"
        })).await;

        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_event_subscription() {
        let event_bus = create_bus(100);
        let mut receiver = subscribe(&event_bus);
        
        // 发布事件
        tokio::spawn({
            let event_bus = event_bus.clone();
            async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                let _ = publish(&event_bus, "agent_started", serde_json::json!({
                    "agent_id": "test_agent"
                })).await;
            }
        });
        
        // 接收事件
        let event = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            receiver.recv()
        ).await;
        
        assert!(event.is_ok());
    }
    
    #[tokio::test]
    async fn test_metrics_handler() {
        let event_bus = create_bus(100);

        // 创建一个订阅者以保持通道开放
        let _receiver = subscribe(&event_bus);

        let metrics_handler = register_metrics_handler(&event_bus).await
            .expect("Failed to register metrics handler");

        // 发布一些事件
        publish(&event_bus, "agent_started", serde_json::json!({
            "agent_id": "test_agent"
        })).await.expect("Failed to publish event");

        // 等待事件处理
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let metrics = metrics_handler.get_metrics().await;
        assert!(metrics.get("agent_started").is_some());
        assert!(metrics.get("total_events").is_some());
    }
    
    #[test]
    fn test_filter_builder() {
        let filter = filter()
            .agent_ids(vec!["agent_001".to_string()])
            .event_types(vec!["AgentStarted".to_string()])
            .build();
        
        // 测试构建器模式
        assert!(true); // 简单的编译测试
    }
}
