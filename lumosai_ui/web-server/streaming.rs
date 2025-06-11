/*!
# Streaming Response Module

流式响应处理模块，实现Server-Sent Events (SSE)。

## 功能特性

- **SSE流式响应**: 实时推送AI回复
- **错误处理**: 优雅的错误恢复机制
- **状态管理**: 聊天状态实时更新
- **客户端兼容**: 支持多种前端框架
*/

use axum::{
    extract::{Path, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;

use crate::ai_client::{AIClient, ChatMessage, MessageRole, StreamChunk};
use crate::database::{Database, MessageRole as DbMessageRole};

/// 流式聊天请求
#[derive(Debug, Deserialize)]
pub struct StreamChatRequest {
    pub message: String,
    pub conversation_id: Option<String>,
    pub model: Option<String>,
}

/// 流式聊天响应事件
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    /// 开始流式响应
    #[serde(rename = "start")]
    Start {
        conversation_id: String,
        message_id: String,
    },
    /// 文本增量
    #[serde(rename = "delta")]
    Delta {
        content: String,
    },
    /// 工具调用
    #[serde(rename = "tool_call")]
    ToolCall {
        id: String,
        name: String,
        arguments: String,
    },
    /// 完成
    #[serde(rename = "done")]
    Done {
        message_id: String,
        total_tokens: Option<u32>,
    },
    /// 错误
    #[serde(rename = "error")]
    Error {
        message: String,
        code: Option<String>,
    },
}

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub ai_client: AIClient,
    pub database: Database,
}

/// 流式聊天处理器
pub async fn stream_chat(
    State(state): State<AppState>,
    Json(request): Json<StreamChatRequest>,
) -> impl IntoResponse {
    let stream = create_chat_stream(state.ai_client, state.database, request).await;
    
    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive-text"),
        )
}

/// 创建聊天流
async fn create_chat_stream(
    ai_client: AIClient,
    database: Database,
    request: StreamChatRequest,
) -> impl Stream<Item = Result<Event, Infallible>> {
    let conversation_id = request.conversation_id.unwrap_or_else(|| generate_id());
    let message_id = generate_id();

    // 创建消息历史
    let messages = vec![ChatMessage {
        role: MessageRole::User,
        content: Some(request.message),
        tool_calls: None,
        tool_call_id: None,
    }];

    // 发送开始事件
    let start_event = StreamEvent::Start {
        conversation_id: conversation_id.clone(),
        message_id: message_id.clone(),
    };

    let start_stream = stream::once(async move {
        Ok(Event::default()
            .event("message")
            .data(serde_json::to_string(&start_event).unwrap_or_default()))
    });

    // 创建AI响应流
    let ai_stream = match ai_client.chat_completion_stream(messages).await {
        Ok(stream) => {
            let mapped_stream = stream.map(move |chunk_result| {
                match chunk_result {
                    Ok(chunk) => {
                        // 处理流式响应块
                        if let Some(choice) = chunk.choices.first() {
                            if let Some(ref delta) = choice.delta {
                                if let Some(ref content) = delta.content {
                                    let event = StreamEvent::Delta {
                                        content: content.clone(),
                                    };
                                    return Ok(Event::default()
                                        .event("message")
                                        .data(serde_json::to_string(&event).unwrap_or_default()));
                                }
                            }
                        }
                        
                        // 如果没有内容，发送空的delta
                        let event = StreamEvent::Delta {
                            content: String::new(),
                        };
                        Ok(Event::default()
                            .event("message")
                            .data(serde_json::to_string(&event).unwrap_or_default()))
                    }
                    Err(e) => {
                        let event = StreamEvent::Error {
                            message: e.to_string(),
                            code: Some("ai_error".to_string()),
                        };
                        Ok(Event::default()
                            .event("error")
                            .data(serde_json::to_string(&event).unwrap_or_default()))
                    }
                }
            });
            
            Box::pin(mapped_stream) as std::pin::Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>
        }
        Err(e) => {
            let error_event = StreamEvent::Error {
                message: e.to_string(),
                code: Some("connection_error".to_string()),
            };
            let error_stream = stream::once(async move {
                Ok(Event::default()
                    .event("error")
                    .data(serde_json::to_string(&error_event).unwrap_or_default()))
            });
            Box::pin(error_stream)
        }
    };

    // 创建完成事件流
    let done_event = StreamEvent::Done {
        message_id: message_id.clone(),
        total_tokens: None,
    };
    let done_stream = stream::once(async move {
        Ok(Event::default()
            .event("done")
            .data(serde_json::to_string(&done_event).unwrap_or_default()))
    });

    // 合并所有流
    start_stream.chain(ai_stream).chain(done_stream)
}

/// 简单的聊天处理器（非流式）
pub async fn simple_chat(
    State(state): State<AppState>,
    Json(request): Json<StreamChatRequest>,
) -> impl IntoResponse {
    let messages = vec![ChatMessage {
        role: MessageRole::User,
        content: Some(request.message),
        tool_calls: None,
        tool_call_id: None,
    }];

    match state.ai_client.chat_completion(messages).await {
        Ok(response) => {
            if let Some(choice) = response.choices.first() {
                if let Some(ref message) = choice.message {
                    if let Some(ref content) = message.content {
                        return Json(serde_json::json!({
                            "success": true,
                            "content": content,
                            "usage": response.usage
                        }));
                    }
                }
            }
            Json(serde_json::json!({
                "success": false,
                "error": "No response content"
            }))
        }
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

/// 获取对话历史
pub async fn get_conversation(
    Path(conversation_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let conversation_id: i64 = match conversation_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return Json(serde_json::json!({
                "success": false,
                "error": "Invalid conversation ID"
            }));
        }
    };

    // 默认用户ID为1（系统用户）
    let user_id = 1;

    match state.database.get_conversation(conversation_id, user_id).await {
        Ok(conversation) => {
            match state.database.get_messages(conversation_id).await {
                Ok(messages) => Json(serde_json::json!({
                    "success": true,
                    "conversation": {
                        "id": conversation.id,
                        "title": conversation.title,
                        "created_at": conversation.created_at,
                        "updated_at": conversation.updated_at
                    },
                    "messages": messages
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to get messages: {}", e)
                }))
            }
        }
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": format!("Conversation not found: {}", e)
        }))
    }
}

/// 删除对话
pub async fn delete_conversation(
    Path(conversation_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let conversation_id: i64 = match conversation_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return Json(serde_json::json!({
                "success": false,
                "error": "Invalid conversation ID"
            }));
        }
    };

    // 默认用户ID为1（系统用户）
    let user_id = 1;

    match state.database.delete_conversation(conversation_id, user_id).await {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "conversation_id": conversation_id
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": format!("Failed to delete conversation: {}", e)
        }))
    }
}

/// 获取对话列表
pub async fn list_conversations(
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 默认用户ID为1（系统用户）
    let user_id = 1;

    match state.database.get_conversations(user_id).await {
        Ok(conversations) => Json(serde_json::json!({
            "success": true,
            "conversations": conversations,
            "total": conversations.len()
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": format!("Failed to get conversations: {}", e)
        }))
    }
}

/// 生成唯一ID
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("id_{}", timestamp)
}

/// 健康检查端点
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// WebSocket升级处理器（未来实现）
pub async fn websocket_handler() -> impl IntoResponse {
    // TODO: 实现WebSocket支持
    Json(serde_json::json!({
        "error": "WebSocket not implemented yet"
    }))
}
