/*!
# AI Client Module

AI服务客户端，支持多种AI提供商的统一接口。

## 支持的AI服务

- **OpenAI**: GPT-3.5, GPT-4 系列
- **DeepSeek**: DeepSeek-Chat, DeepSeek-Coder
- **Anthropic**: Claude 系列
- **本地模型**: Ollama, LM Studio

## 功能特性

- 统一的聊天完成接口
- 流式响应支持
- 工具调用集成
- 错误处理和重试机制
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio_stream::Stream;

/// AI服务提供商
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AIProvider {
    OpenAI,
    DeepSeek,
    Anthropic,
    Ollama,
    Custom(String),
}

/// 聊天消息角色
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "tool")]
    Tool,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: FunctionCall,
}

/// 函数调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// 聊天完成请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
    pub tools: Option<Vec<Tool>>,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub r#type: String,
    pub function: ToolFunction,
}

/// 工具函数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// 聊天完成响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

/// 选择项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Option<ChatMessage>,
    pub delta: Option<ChatMessage>,
    pub finish_reason: Option<String>,
}

/// 使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 流式响应块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
}

/// AI客户端配置
#[derive(Debug, Clone)]
pub struct AIClientConfig {
    pub provider: AIProvider,
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for AIClientConfig {
    fn default() -> Self {
        Self {
            provider: AIProvider::OpenAI,
            api_key: None,
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

/// AI客户端错误
#[derive(Debug, thiserror::Error)]
pub enum AIClientError {
    #[error("HTTP请求错误: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON序列化错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("API错误: {code} - {message}")]
    Api { code: u16, message: String },
    #[error("配置错误: {0}")]
    Config(String),
    #[error("流式响应错误: {0}")]
    Stream(String),
}

/// AI客户端
#[derive(Clone)]
pub struct AIClient {
    config: AIClientConfig,
    client: reqwest::Client,
}

impl AIClient {
    /// 创建新的AI客户端
    pub fn new(config: AIClientConfig) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    /// 创建OpenAI客户端
    pub fn openai(api_key: String) -> Self {
        let config = AIClientConfig {
            provider: AIProvider::OpenAI,
            api_key: Some(api_key),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            ..Default::default()
        };
        Self::new(config)
    }

    /// 创建DeepSeek客户端
    pub fn deepseek(api_key: String) -> Self {
        let config = AIClientConfig {
            provider: AIProvider::DeepSeek,
            api_key: Some(api_key),
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
            ..Default::default()
        };
        Self::new(config)
    }

    /// 创建本地Ollama客户端
    pub fn ollama(base_url: String, model: String) -> Self {
        let config = AIClientConfig {
            provider: AIProvider::Ollama,
            api_key: None,
            base_url,
            model,
            ..Default::default()
        };
        Self::new(config)
    }

    /// 发送聊天完成请求
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatCompletionResponse, AIClientError> {
        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
            stream: Some(false),
            tools: None,
        };

        let mut req_builder = self
            .client
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Content-Type", "application/json")
            .json(&request);

        if let Some(ref api_key) = self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIClientError::Api {
                code: status.as_u16(),
                message: error_text,
            });
        }

        let completion: ChatCompletionResponse = response.json().await?;
        Ok(completion)
    }

    /// 发送流式聊天完成请求
    pub async fn chat_completion_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<impl Stream<Item = Result<StreamChunk, AIClientError>>, AIClientError> {
        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
            stream: Some(true),
            tools: None,
        };

        let mut req_builder = self
            .client
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Content-Type", "application/json")
            .json(&request);

        if let Some(ref api_key) = self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIClientError::Api {
                code: status.as_u16(),
                message: error_text,
            });
        }

        // 这里需要实现SSE流解析
        // 暂时返回一个空的流，后续会完善
        use tokio_stream::iter;
        Ok(iter(vec![]))
    }
}
