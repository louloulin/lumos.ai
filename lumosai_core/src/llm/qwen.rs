use async_trait::async_trait;
// use futures::stream; // Unused
use futures::stream::BoxStream;
use async_openai::{
    types::{
        ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage,
        ChatCompletionRequestAssistantMessage,
        ChatCompletionRequestMessage,
        Role as OpenAIRole,
        CreateEmbeddingRequest,
        CreateChatCompletionRequestArgs,
        ChatCompletionRequestUserMessageContent,
        EmbeddingInput,
        // CreateChatCompletionStreamResponse, // Unused
    },
    error::OpenAIError,
    Client,
    config::OpenAIConfig,
};
use futures::{/* Stream, StreamExt, */ TryStreamExt}; // Removed unused imports
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
// use std::collections::HashMap; // Unused

use crate::Result;
use crate::Error;
use super::provider::LlmProvider;
use super::types::{LlmOptions, Message, Role};

impl From<OpenAIError> for Error {
    fn from(err: OpenAIError) -> Self {
        Error::Llm(err.to_string())
    }
}

/// OpenAI compatible API response structure
#[derive(Debug, Deserialize)]
struct OpenAICompatResponse {
    choices: Vec<OpenAICompatChoice>,
    #[serde(default)]
    usage: Option<OpenAICompatUsage>,
}

/// OpenAI compatible API choice structure
#[derive(Debug, Deserialize)]
struct OpenAICompatChoice {
    message: OpenAICompatMessage,
    finish_reason: Option<String>,
}

/// OpenAI compatible API message structure
#[derive(Debug, Deserialize)]
struct OpenAICompatMessage {
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<OpenAICompatToolCall>>,
}

/// OpenAI compatible API tool call structure
#[derive(Debug, Deserialize)]
struct OpenAICompatToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAICompatFunction,
}

/// OpenAI compatible API function structure
#[derive(Debug, Deserialize)]
struct OpenAICompatFunction {
    name: String,
    arguments: String,
}

/// OpenAI compatible API usage structure
#[derive(Debug, Deserialize)]
struct OpenAICompatUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI compatible API embedding response structure
#[derive(Debug, Deserialize)]
struct OpenAICompatEmbeddingResponse {
    data: Vec<OpenAICompatEmbeddingData>,
    usage: OpenAICompatEmbeddingUsage,
}

/// OpenAI compatible API embedding data structure
#[derive(Debug, Deserialize)]
struct OpenAICompatEmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

/// OpenAI compatible API embedding usage structure
#[derive(Debug, Deserialize)]
struct OpenAICompatEmbeddingUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

/// DashScope API response structure
#[derive(Debug, Deserialize)]
struct DashScopeResponse {
    output: DashScopeOutput,
}

/// DashScope API output structure
#[derive(Debug, Deserialize)]
struct DashScopeOutput {
    text: Option<String>,
    choices: Option<Vec<DashScopeChoice>>,
    tool_calls: Option<Vec<DashScopeToolCall>>,
}

/// DashScope API choice structure
#[derive(Debug, Deserialize)]
struct DashScopeChoice {
    message: DashScopeMessage,
}

/// DashScope API message structure
#[derive(Debug, Deserialize)]
struct DashScopeMessage {
    content: String,
    tool_calls: Option<Vec<DashScopeToolCall>>,
}

/// DashScope API tool call structure
#[derive(Debug, Deserialize)]
struct DashScopeToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: DashScopeFunction,
}

/// DashScope API function structure
#[derive(Debug, Deserialize)]
struct DashScopeFunction {
    name: String,
    arguments: String,
}

/// DashScope API embedding response structure
#[derive(Debug, Deserialize)]
struct DashScopeEmbeddingResponse {
    output: DashScopeEmbeddingOutput,
}

/// DashScope API embedding output structure
#[derive(Debug, Deserialize)]
struct DashScopeEmbeddingOutput {
    embeddings: Vec<DashScopeEmbedding>,
}

/// DashScope API embedding structure
#[derive(Debug, Deserialize)]
struct DashScopeEmbedding {
    embedding: Vec<f32>,
}

/// Tool parameter schema for Qwen function calling
#[derive(Debug, Serialize)]
struct ToolParameter {
    #[serde(rename = "type")]
    param_type: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<bool>,
}

/// Tool properties for Qwen function calling
#[derive(Debug, Serialize)]
struct ToolProperties {
    #[serde(flatten)]
    properties: serde_json::Map<String, serde_json::Value>,
}

/// Tool schema for Qwen function calling
#[derive(Debug, Serialize)]
struct ToolSchema {
    #[serde(rename = "type")]
    schema_type: String,
    properties: ToolProperties,
    required: Vec<String>,
}

/// Tool definition for Qwen function calling
#[derive(Debug, Serialize)]
struct Tool {
    #[serde(rename = "type")]
    tool_type: String,
    function: ToolFunction,
}

/// Tool function for Qwen function calling
#[derive(Debug, Serialize)]
struct ToolFunction {
    name: String,
    description: String,
    parameters: ToolSchema,
}

/// API type enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QwenApiType {
    /// DashScope API (Alibaba Cloud)
    DashScope,
    /// OpenAI compatible API
    OpenAICompatible,
}

/// Qwen LLM provider implementation using async-openai
pub struct QwenProvider {
    /// OpenAI client
    client: Client<OpenAIConfig>,
    /// Default model to use
    model: String,
    /// Embedding model to use
    embedding_model: String,
    /// API type
    api_type: QwenApiType,
    /// API key
    api_key: String,
}

impl QwenProvider {
    /// Create a new Qwen provider with custom base URL and API type
    pub fn new_with_api_type(
        api_key: impl Into<String>, 
        model: impl Into<String>, 
        base_url: impl Into<String>, 
        api_type: QwenApiType
    ) -> Self {
        let api_key_str = api_key.into();
        let config = OpenAIConfig::new()
            .with_api_key(api_key_str.clone())
            .with_api_base(base_url.into());

        Self {
            client: Client::with_config(config),
            model: model.into(),
            embedding_model: match api_type {
                QwenApiType::DashScope => "text-embedding-v1".to_string(),
                QwenApiType::OpenAICompatible => "text-embedding-ada-002".to_string(),
            },
            api_type,
            api_key: api_key_str,
        }
    }

    /// Create a new Qwen provider with DashScope API
    pub fn new(api_key: impl Into<String>, model: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self::new_with_api_type(api_key, model, base_url, QwenApiType::DashScope)
    }

    /// Create a new Qwen provider with default DashScope base URL
    pub fn new_with_defaults(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new_with_api_type(
            api_key,
            model,
            "https://dashscope.aliyuncs.com/api/v1",
            QwenApiType::DashScope
        )
    }

    /// Create a new Qwen provider configured for Qwen 2.5 models with DashScope API
    pub fn new_qwen25(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new_with_api_type(
            api_key,
            model,
            "https://dashscope.aliyuncs.com/api/v1",
            QwenApiType::DashScope
        )
    }

    /// Create a new Qwen provider with OpenAI-compatible API
    pub fn new_openai_compatible(
        api_key: impl Into<String>, 
        model: impl Into<String>,
        base_url: Option<impl Into<String>>,
    ) -> Self {
        Self::new_with_api_type(
            api_key,
            model,
            base_url.map(|url| url.into())
                .unwrap_or_else(|| "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string()),
            QwenApiType::OpenAICompatible
        )
    }

    /// Convert internal Message type to OpenAI ChatCompletionRequestMessage
    fn convert_messages(&self, messages: &[Message]) -> Vec<ChatCompletionRequestMessage> {
        messages.iter().map(|msg| {
            match msg.role {
                Role::System => ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessage {
                        role: OpenAIRole::System,
                        content: msg.content.clone(),
                        name: msg.name.clone(),
                    }
                ),
                Role::User => ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        role: OpenAIRole::User,
                        content: ChatCompletionRequestUserMessageContent::Text(msg.content.clone()),
                        name: msg.name.clone(),
                    }
                ),
                Role::Assistant => ChatCompletionRequestMessage::Assistant(
                    ChatCompletionRequestAssistantMessage {
                        role: OpenAIRole::Assistant,
                        content: Some(msg.content.clone()),
                        name: msg.name.clone(),
                        tool_calls: None,
                        #[allow(deprecated)]
                        function_call: None, // Deprecated but still required
                    }
                ),
                _ => ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        role: OpenAIRole::User,
                        content: ChatCompletionRequestUserMessageContent::Text(msg.content.clone()),
                        name: msg.name.clone(),
                    }
                ),
            }
        }).collect()
    }
}

#[async_trait]
impl LlmProvider for QwenProvider {
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
        match self.api_type {
            QwenApiType::OpenAICompatible => {
                let request = CreateChatCompletionRequestArgs::default()
                    .model(&self.model)
                    .messages(self.convert_messages(messages))
                    .build()?;

                let response = self.client.chat().create(request).await?;
                
                if let Some(choice) = response.choices.first() {
                    if let Some(content) = &choice.message.content {
                        Ok(content.clone())
                    } else {
                        Err(Error::Llm("No content in response".into()))
                    }
                } else {
                    Err(Error::Llm("No choices in response".into()))
                }
            }
            QwenApiType::DashScope => {
                // Convert messages to DashScope format
                let messages_json: Vec<serde_json::Value> = messages.iter().map(|msg| {
                    json!({
                        "role": match msg.role {
                            Role::System => "system",
                            Role::User => "user",
                            Role::Assistant => "assistant",
                            _ => "user"
                        },
                        "content": msg.content
                    })
                }).collect();

                // Build DashScope request
                let request = json!({
                    "model": self.model,
                    "input": {
                        "messages": messages_json
                    },
                    "parameters": {
                        "temperature": options.temperature.unwrap_or(0.7),
                        "max_tokens": options.max_tokens.unwrap_or(1024),
                        "top_p": options.extra.get("top_p").and_then(|v| v.as_f64()).unwrap_or(0.8),
                        "result_format": "message"
                    }
                });

                // Send request using reqwest directly
                let client = reqwest::Client::new();
                let response = client
                    .post("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation")
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .header("Content-Type", "application/json")
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| Error::Llm(e.to_string()))?;

                let response_text = response.text().await
                    .map_err(|e| Error::Llm(e.to_string()))?;

                let response_json: DashScopeResponse = serde_json::from_str(&response_text)
                    .map_err(|e| Error::Llm(format!("Failed to parse response: {}\nResponse text: {}", e, response_text)))?;

                if let Some(text) = response_json.output.text {
                    Ok(text)
                } else if let Some(choices) = response_json.output.choices {
                    if let Some(choice) = choices.first() {
                        Ok(choice.message.content.clone())
                    } else {
                        Err(Error::Llm("No choices in response".into()))
                    }
                } else {
                    Err(Error::Llm("No text or choices in response".into()))
                }
            }
        }
    }

    async fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        _options: &'a LlmOptions // Prefixed with underscore to indicate unused
    ) -> Result<BoxStream<'a, Result<String>>> {
        let messages = vec![Message {
            role: Role::User,
            content: prompt.to_string(),
            metadata: None,
            name: None,
        }];
        
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(self.convert_messages(&messages))
            .stream(true)
            .build()?;

        let stream = self.client.chat().create_stream(request).await?;
        
        Ok(Box::pin(stream
            .map_err(|e| Error::Llm(e.to_string()))
            .try_filter_map(|response| async move {
                Ok(response.choices.first()
                    .and_then(|choice| choice.delta.content.clone()))
            })))
    }

    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = CreateEmbeddingRequest {
            model: self.embedding_model.clone(),
            input: EmbeddingInput::String(text.to_string()),
            encoding_format: None,
            user: None,
            dimensions: None,
        };

        let response = self.client.embeddings().create(request).await?;
        
        if let Some(embedding) = response.data.first() {
            Ok(embedding.embedding.clone())
        } else {
            Err(Error::Llm("No embedding in response".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;

    #[test]
    fn test_qwen_provider() {
        // Skip test if no API key is provided (this is a unit test, not an integration test)
        let api_key = match std::env::var("QWEN_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                println!("Skipping test_qwen_provider as QWEN_API_KEY is not set");
                return;
            }
        };

        let provider = QwenProvider::new_with_defaults(api_key, "qwen-turbo");

        // Test basic prompt generation
        let result = block_on(provider.generate("Hello", &LlmOptions::default()));
        assert!(result.is_ok(), "Failed to generate text: {:?}", result.err());

        // Test message-based generation
        let messages = vec![
            Message {
                role: Role::User,
                content: "Hello".to_string(),
                metadata: None,
                name: None,
            },
        ];
        let result = block_on(provider.generate_with_messages(&messages, &LlmOptions::default()));
        assert!(result.is_ok(), "Failed to generate text with messages: {:?}", result.err());

        // Test embeddings
        let result = block_on(provider.get_embedding("Hello"));
        assert!(result.is_ok(), "Failed to get embedding: {:?}", result.err());
    }
} 