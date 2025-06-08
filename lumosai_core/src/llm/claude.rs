//! Claude LLM提供商实现
//! 
//! 这个模块实现了对Anthropic Claude模型的支持，包括：
//! - Claude 3.5 Sonnet
//! - Claude 3 Opus
//! - Claude 3 Haiku
//! - 函数调用支持
//! - 流式响应

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    LlmProvider, LlmOptions, Message, Role,
    function_calling::{FunctionDefinition, ToolChoice, FunctionCall},
    provider::FunctionCallingResponse
};
use futures::stream::BoxStream;
use crate::error::{LumosError, Result};

/// Claude API配置
#[derive(Debug, Clone)]
pub struct ClaudeConfig {
    api_key: String,
    model: String,
    base_url: String,
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
        }
    }
}

/// Claude LLM提供商
#[derive(Debug, Clone)]
pub struct ClaudeProvider {
    config: ClaudeConfig,
    client: Client,
}

impl ClaudeProvider {
    /// 创建新的Claude提供商
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            config: ClaudeConfig {
                api_key,
                model,
                base_url: "https://api.anthropic.com".to_string(),
            },
            client: Client::new(),
        }
    }

    /// 从环境变量创建Claude提供商
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("CLAUDE_API_KEY")
            .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
            .map_err(|_| LumosError::ConfigError {
                message: "CLAUDE_API_KEY or ANTHROPIC_API_KEY environment variable not found".to_string(),
            })?;

        let model = std::env::var("CLAUDE_MODEL")
            .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());

        Ok(Self::new(api_key, model))
    }

    /// 创建使用特定模型的Claude提供商
    pub fn sonnet(api_key: String) -> Self {
        Self::new(api_key, "claude-3-5-sonnet-20241022".to_string())
    }

    pub fn opus(api_key: String) -> Self {
        Self::new(api_key, "claude-3-opus-20240229".to_string())
    }

    pub fn haiku(api_key: String) -> Self {
        Self::new(api_key, "claude-3-haiku-20240307".to_string())
    }

    /// 获取模型名称
    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// 转换消息格式
    fn convert_messages(&self, messages: &[Message]) -> Vec<ClaudeMessage> {
        let mut claude_messages = Vec::new();
        let mut system_content = String::new();

        for message in messages {
            match message.role {
                Role::System => {
                    if !system_content.is_empty() {
                        system_content.push('\n');
                    }
                    system_content.push_str(&message.content);
                }
                Role::User => {
                    claude_messages.push(ClaudeMessage {
                        role: "user".to_string(),
                        content: message.content.clone(),
                    });
                }
                Role::Assistant => {
                    claude_messages.push(ClaudeMessage {
                        role: "assistant".to_string(),
                        content: message.content.clone(),
                    });
                }
                Role::Function | Role::Tool | Role::Custom(_) => {
                    // Claude不直接支持这些角色，转换为用户消息
                    claude_messages.push(ClaudeMessage {
                        role: "user".to_string(),
                        content: format!("Function result: {}", message.content),
                    });
                }
            }
        }

        // 如果有系统消息，添加到第一个用户消息前
        if !system_content.is_empty() && !claude_messages.is_empty() {
            if let Some(first_msg) = claude_messages.first_mut() {
                if first_msg.role == "user" {
                    first_msg.content = format!("{}\n\n{}", system_content, first_msg.content);
                }
            } else {
                claude_messages.insert(0, ClaudeMessage {
                    role: "user".to_string(),
                    content: system_content,
                });
            }
        }

        claude_messages
    }

    /// 转换选项
    fn convert_options(&self, options: &LlmOptions) -> ClaudeOptions {
        ClaudeOptions {
            max_tokens: options.max_tokens.unwrap_or(4096),
            temperature: options.temperature,
            top_p: options.extra.get("top_p").and_then(|v| v.as_f64()),
            top_k: options.extra.get("top_k").and_then(|v| v.as_i64()).map(|k| k as i32),
            stop_sequences: options.stop.clone(),
        }
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    fn name(&self) -> &str {
        "claude"
    }

    fn supports_function_calling(&self) -> bool {
        true // Claude 3支持函数调用
    }

    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        let messages = vec![Message {
            role: Role::User,
            content: prompt.to_string(),
            metadata: None,
            name: None,
        }];

        self.generate_with_messages(&messages, options).await
    }

    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        let claude_messages = self.convert_messages(messages);
        let claude_options = self.convert_options(options);

        let request = ClaudeRequest {
            model: self.config.model.clone(),
            messages: claude_messages,
            max_tokens: claude_options.max_tokens,
            temperature: claude_options.temperature,
            top_p: claude_options.top_p,
            top_k: claude_options.top_k,
            stop_sequences: claude_options.stop_sequences,
            stream: false,
        };

        let response = self
            .client
            .post(&format!("{}/v1/messages", self.config.base_url))
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(|e| LumosError::NetworkError {
                message: format!("Failed to send request to Claude API: {}", e),
                source: Some(Box::new(e)),
            })?;

        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LumosError::ApiError {
                message: format!("Claude API error: {}", error_text),
                status_code: Some(status_code),
            });
        }

        let claude_response: ClaudeResponse = response.json().await.map_err(|e| {
            LumosError::ParseError {
                message: format!("Failed to parse Claude response: {}", e),
                source: Some(Box::new(e)),
            }
        })?;

        if let Some(content) = claude_response.content.first() {
            Ok(content.text.clone())
        } else {
            Err(LumosError::ApiError {
                message: "Empty response from Claude API".to_string(),
                status_code: None,
            })
        }
    }

    async fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        options: &'a LlmOptions,
    ) -> Result<BoxStream<'a, Result<String>>> {
        let messages = vec![Message {
            role: Role::User,
            content: prompt.to_string(),
            metadata: None,
            name: None,
        }];

        self.generate_stream_with_messages(&messages, options).await
    }

    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Claude不直接支持嵌入，返回错误
        Err(LumosError::Unsupported("Claude does not support embeddings".to_string()))
    }

    async fn generate_with_functions(
        &self,
        messages: &[Message],
        functions: &[FunctionDefinition],
        tool_choice: &ToolChoice,
        options: &LlmOptions,
    ) -> Result<FunctionCallingResponse> {
        self.call_function(messages, functions, tool_choice, options).await
    }
}

impl ClaudeProvider {
    /// 生成流式响应（内部方法）
    async fn generate_stream_with_messages(
        &self,
        messages: &[Message],
        options: &LlmOptions,
    ) -> Result<BoxStream<'_, Result<String>>> {
        let claude_messages = self.convert_messages(messages);
        let claude_options = self.convert_options(options);

        let request = ClaudeRequest {
            model: self.config.model.clone(),
            messages: claude_messages,
            max_tokens: claude_options.max_tokens,
            temperature: claude_options.temperature,
            top_p: claude_options.top_p,
            top_k: claude_options.top_k,
            stop_sequences: claude_options.stop_sequences,
            stream: true,
        };

        let response = self
            .client
            .post(&format!("{}/v1/messages", self.config.base_url))
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(|e| LumosError::NetworkError {
                message: format!("Failed to send request to Claude API: {}", e),
                source: Some(Box::new(e)),
            })?;

        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LumosError::ApiError {
                message: format!("Claude API error: {}", error_text),
                status_code: Some(status_code),
            });
        }

        // 创建流式响应处理器
        use futures::stream::{self, StreamExt};

        let stream = response.bytes_stream().map(|chunk_result| {
            match chunk_result {
                Ok(chunk) => {
                    // 解析SSE格式的响应
                    let chunk_str = String::from_utf8_lossy(&chunk);
                    if let Some(data_line) = chunk_str.lines().find(|line| line.starts_with("data: ")) {
                        let json_str = &data_line[6..]; // 移除"data: "前缀
                        if json_str == "[DONE]" {
                            return Ok("".to_string());
                        }

                        // 解析JSON并提取文本
                        if let Ok(delta) = serde_json::from_str::<serde_json::Value>(json_str) {
                            if let Some(text) = delta["delta"]["text"].as_str() {
                                return Ok(text.to_string());
                            }
                        }
                    }
                    Ok("".to_string())
                }
                Err(e) => Err(LumosError::NetworkError {
                    message: format!("Stream error: {}", e),
                    source: Some(Box::new(e)),
                }),
            }
        });

        Ok(Box::pin(stream))
    }

    /// 函数调用实现
    async fn call_function(
        &self,
        messages: &[Message],
        functions: &[FunctionDefinition],
        tool_choice: &ToolChoice,
        options: &LlmOptions,
    ) -> Result<FunctionCallingResponse> {
        // Claude的函数调用实现
        // 注意：这是一个简化的实现，实际的Claude API可能有不同的格式
        
        let mut claude_messages = self.convert_messages(messages);
        
        // 添加函数定义到系统消息中
        let functions_desc = functions.iter()
            .map(|f| format!("Function: {}\nDescription: {}\nParameters: {}",
                f.name, f.description.as_ref().unwrap_or(&"No description".to_string()), serde_json::to_string(&f.parameters).unwrap_or_default()))
            .collect::<Vec<_>>()
            .join("\n\n");
        
        if let Some(first_msg) = claude_messages.first_mut() {
            first_msg.content = format!("Available functions:\n{}\n\n{}", functions_desc, first_msg.content);
        }

        let response = self.generate_with_messages(
            &messages.iter().map(|m| Message {
                role: m.role.clone(),
                content: m.content.clone(),
                metadata: m.metadata.clone(),
                name: m.name.clone(),
            }).collect::<Vec<_>>(),
            options
        ).await?;

        // 简化的函数调用解析
        // 实际实现需要根据Claude的具体API格式来解析
        Ok(FunctionCallingResponse {
            content: Some(response),
            function_calls: vec![], // 需要实际解析函数调用
            finish_reason: "stop".to_string(),
        })
    }
}

/// Claude API请求结构
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    stream: bool,
}

/// Claude消息结构
#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

/// Claude选项
#[derive(Debug)]
struct ClaudeOptions {
    max_tokens: u32,
    temperature: Option<f32>,
    top_p: Option<f64>,
    top_k: Option<i32>,
    stop_sequences: Option<Vec<String>>,
}

/// Claude API响应结构
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
    model: String,
    role: String,
    stop_reason: Option<String>,
    usage: ClaudeUsage,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
    #[serde(rename = "type")]
    content_type: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_provider_creation() {
        let provider = ClaudeProvider::new(
            "test-key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
        );
        
        assert_eq!(provider.name(), "claude");
        assert!(provider.supports_function_calling());
        assert!(provider.supports_function_calling());
    }

    #[test]
    fn test_claude_model_variants() {
        let sonnet = ClaudeProvider::sonnet("test-key".to_string());
        assert_eq!(sonnet.model(), "claude-3-5-sonnet-20241022");

        let opus = ClaudeProvider::opus("test-key".to_string());
        assert_eq!(opus.model(), "claude-3-opus-20240229");

        let haiku = ClaudeProvider::haiku("test-key".to_string());
        assert_eq!(haiku.model(), "claude-3-haiku-20240307");
    }

    #[test]
    fn test_claude_from_env_error() {
        // 清除环境变量
        std::env::remove_var("CLAUDE_API_KEY");
        std::env::remove_var("ANTHROPIC_API_KEY");
        
        let result = ClaudeProvider::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_claude_message_conversion() {
        let provider = ClaudeProvider::new(
            "test-key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
        );
        
        let messages = vec![
            Message {
                role: Role::System,
                content: "You are a helpful assistant.".to_string(),
                metadata: None,
                name: None,
            },
            Message {
                role: Role::User,
                content: "Hello!".to_string(),
                metadata: None,
                name: None,
            },
        ];
        
        let claude_messages = provider.convert_messages(&messages);
        assert_eq!(claude_messages.len(), 1);
        assert_eq!(claude_messages[0].role, "user");
        assert!(claude_messages[0].content.contains("You are a helpful assistant."));
        assert!(claude_messages[0].content.contains("Hello!"));
    }
}
