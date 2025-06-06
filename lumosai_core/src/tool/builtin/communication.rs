//! 通信工具集
//! 
//! 提供邮件发送、Slack消息、Webhook调用、短信发送等通信功能

use crate::tool::{ToolSchema, ParameterSchema, FunctionTool};
use crate::error::Result;
use serde_json::{Value, json};

/// 邮件发送工具
pub fn email_sender() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "smtp_config".to_string(),
            description: "SMTP配置（JSON格式）".to_string(),
            r#type: "object".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "to".to_string(),
            description: "收件人邮箱地址".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "subject".to_string(),
            description: "邮件主题".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "body".to_string(),
            description: "邮件正文".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "is_html".to_string(),
            description: "是否为HTML格式".to_string(),
            r#type: "boolean".to_string(),
            required: false,
            properties: None,
            default: Some(json!(false)),
        },
    ]);

    FunctionTool::new(
        "email_sender",
        "发送邮件，支持HTML格式、附件和批量发送",
        schema,
        |params| {
            let smtp_config = params.get("smtp_config")
                .ok_or_else(|| crate::error::Error::Tool("Missing smtp_config parameter".to_string()))?;
            
            let to = params.get("to")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing to parameter".to_string()))?;
            
            let subject = params.get("subject")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing subject parameter".to_string()))?;
            
            let body = params.get("body")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing body parameter".to_string()))?;
            
            let is_html = params.get("is_html").and_then(|v| v.as_bool()).unwrap_or(false);

            // 模拟邮件发送
            Ok(json!({
                "success": true,
                "message_id": format!("msg_{}", chrono::Utc::now().timestamp()),
                "to": to,
                "subject": subject,
                "body_length": body.len(),
                "is_html": is_html,
                "sent_at": chrono::Utc::now().to_rfc3339(),
                "smtp_server": smtp_config.get("host").unwrap_or(&json!("smtp.example.com")),
                "delivery_status": "sent"
            }))
        },
    )
}

/// Slack消息工具
pub fn slack_messenger() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "webhook_url".to_string(),
            description: "Slack Webhook URL或Bot Token".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "channel".to_string(),
            description: "频道名称或用户ID".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "text".to_string(),
            description: "消息文本".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "username".to_string(),
            description: "发送者用户名".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("Lumos Bot")),
        },
    ]);

    FunctionTool::new(
        "slack_messenger",
        "发送Slack消息，支持频道、私信和富文本格式",
        schema,
        |params| {
            let webhook_url = params.get("webhook_url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing webhook_url parameter".to_string()))?;
            
            let channel = params.get("channel")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing channel parameter".to_string()))?;
            
            let text = params.get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing text parameter".to_string()))?;
            
            let username = params.get("username").and_then(|v| v.as_str()).unwrap_or("Lumos Bot");

            // 模拟Slack消息发送
            Ok(json!({
                "success": true,
                "ok": true,
                "channel": channel,
                "ts": format!("{}.{:06}", chrono::Utc::now().timestamp(), 123456),
                "text": text,
                "username": username,
                "webhook_url": webhook_url.chars().take(50).collect::<String>() + "...",
                "sent_at": chrono::Utc::now().to_rfc3339()
            }))
        },
    )
}

/// Webhook调用工具
pub fn webhook_caller() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "url".to_string(),
            description: "Webhook URL".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "method".to_string(),
            description: "HTTP方法：GET, POST, PUT, DELETE, PATCH".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("POST")),
        },
        ParameterSchema {
            name: "body".to_string(),
            description: "请求体数据".to_string(),
            r#type: "object".to_string(),
            required: false,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "timeout_seconds".to_string(),
            description: "请求超时时间（秒）".to_string(),
            r#type: "number".to_string(),
            required: false,
            properties: None,
            default: Some(json!(30)),
        },
    ]);

    FunctionTool::new(
        "webhook_caller",
        "调用HTTP Webhook，支持各种HTTP方法和认证方式",
        schema,
        |params| {
            let url = params.get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::Error::Tool("Missing url parameter".to_string()))?;
            
            let method = params.get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("POST");
            
            let body = params.get("body");
            let timeout_seconds = params.get("timeout_seconds")
                .and_then(|v| v.as_u64())
                .unwrap_or(30);

            // 模拟Webhook调用
            let response_status = if url.contains("error") { 500 } else { 200 };
            let response_body = if response_status == 200 {
                json!({"status": "success", "message": "Webhook processed successfully"})
            } else {
                json!({"status": "error", "message": "Internal server error"})
            };

            Ok(json!({
                "success": response_status == 200,
                "request": {
                    "url": url,
                    "method": method.to_uppercase(),
                    "timeout_seconds": timeout_seconds,
                    "body": body
                },
                "response": {
                    "status_code": response_status,
                    "headers": {
                        "content-type": "application/json",
                        "server": "webhook-server/1.0"
                    },
                    "body": response_body,
                    "response_time_ms": 245
                },
                "sent_at": chrono::Utc::now().to_rfc3339()
            }))
        },
    )
}

/// 获取所有通信工具
pub fn all_communication_tools() -> Vec<FunctionTool> {
    vec![
        email_sender(),
        slack_messenger(),
        webhook_caller(),
    ]
}
