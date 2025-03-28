use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{Error, Result};
use super::provider::LlmProvider;
use super::types::{LlmOptions, Message};

/// OpenAI API响应结构
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    #[serde(default)]
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI embeddings API响应结构
#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
    usage: OpenAIEmbeddingUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

/// OpenAI消息请求结构
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIRequestMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct OpenAIRequestMessage {
    role: String,
    content: String,
}

/// OpenAI embedding请求结构
#[derive(Debug, Serialize)]
struct OpenAIEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

/// OpenAI LLM provider implementation
pub struct OpenAiProvider {
    /// OpenAI API密钥
    pub(crate) api_key: String,
    /// 使用的模型名称
    pub(crate) model: String,
    /// OpenAI API基础URL
    base_url: String,
    /// HTTP客户端
    client: reqwest::Client,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider
    pub fn new(api_key: String, model: String) -> Self {
        Self { 
            api_key, 
            model, 
            base_url: "https://api.openai.com/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }
    
    /// 设置自定义的API基础URL
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
    
    /// 创建授权请求头
    fn create_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .expect("Invalid API key format"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }
    
    /// 从Lomusai消息转换为OpenAI消息格式
    fn convert_messages(&self, messages: &[Message]) -> Vec<OpenAIRequestMessage> {
        messages
            .iter()
            .map(|msg| OpenAIRequestMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect()
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);
        
        // 准备消息
        let messages = if let Some(messages) = &options.messages {
            self.convert_messages(messages)
        } else {
            vec![OpenAIRequestMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }]
        };
        
        // 创建请求
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages,
            temperature: options.temperature,
            max_tokens: options.max_tokens,
            top_p: None,
            stream: None,
        };
        
        // 发送请求
        let response = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::LlmError(format!("OpenAI API request failed: {}", e)))?;
            
        // 检查响应状态
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::LlmError(format!(
                "OpenAI API error: {}, details: {}",
                status,
                error_text
            )));
        }
        
        // 解析响应
        let response_data: OpenAIResponse = response.json().await
            .map_err(|e| Error::LlmError(format!("Failed to parse OpenAI response: {}", e)))?;
            
        // 提取生成的文本
        let generated_text = response_data.choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .unwrap_or_default();
            
        Ok(generated_text)
    }
    
    async fn generate_stream<'a>(
        &'a self, 
        prompt: &'a str, 
        options: &'a LlmOptions
    ) -> Result<BoxStream<'a, Result<String>>> {
        // 简单实现，实际API流式处理需要更复杂的处理逻辑
        let response = self.generate(prompt, options).await?;
        
        // 创建一个简单的流，返回完整响应
        let stream = stream::once(async move { Ok(response) });
        
        Ok(Box::pin(stream))
    }
    
    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/embeddings", self.base_url);
        
        // 创建请求
        let request = OpenAIEmbeddingRequest {
            model: "text-embedding-ada-002".to_string(), // 使用默认嵌入模型
            input: vec![text.to_string()],
        };
        
        // 发送请求
        let response = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::LlmError(format!("OpenAI API request failed: {}", e)))?;
            
        // 检查响应状态
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::LlmError(format!(
                "OpenAI API error: {}, details: {}",
                status,
                error_text
            )));
        }
        
        // 解析响应
        let response_data: OpenAIEmbeddingResponse = response.json().await
            .map_err(|e| Error::LlmError(format!("Failed to parse OpenAI embedding response: {}", e)))?;
            
        // 提取嵌入
        let embedding = response_data.data
            .first()
            .map(|data| data.embedding.clone())
            .ok_or_else(|| Error::LlmError("No embedding returned from OpenAI".to_string()))?;
            
        Ok(embedding)
    }
} 