use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Error, Result};
use super::provider::{LlmProvider, FunctionCallingResponse};
use super::types::{LlmOptions, Message};
use super::function_calling::{FunctionDefinition, FunctionCall, ToolChoice};

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
    #[serde(default)]
    tool_calls: Vec<OpenAIToolCall>,
}

#[derive(Debug, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Deserialize)]
struct OpenAIFunction {
    name: String,
    arguments: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<Value>,
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
    
    /// 从Lumosai消息转换为OpenAI消息格式
    fn convert_messages(&self, messages: &[Message]) -> Vec<OpenAIRequestMessage> {
        messages
            .iter()
            .map(|msg| OpenAIRequestMessage {
                role: msg.role.as_str().to_string(),
                content: msg.content.clone(),
            })
            .collect()
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        // 准备请求数据
        let url = format!("{}/chat/completions", self.base_url);
        
        let messages = vec![
            serde_json::json!({
                "role": "user",
                "content": prompt
            })
        ];
        
        // 构建请求正文
        let mut body = serde_json::json!({
            "model": self.model.clone(),
            "messages": messages,
        });
        
        // 添加选项参数
        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }
        
        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }
        
        if let Some(stop) = &options.stop {
            body["stop"] = serde_json::json!(stop);
        }
        
        // 发送请求
        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("OpenAI API request failed: {}", e)))?;
            
        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read OpenAI response: {}", e)))?;
            
        if !status.is_success() {
            return Err(Error::Llm(format!(
                "OpenAI API returned error status {}: {}",
                status, text
            )));
        }
        
        // 解析响应
        let response: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse OpenAI response: {}", e)))?;
            
        // 提取生成的文本
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::Llm("Invalid response format from OpenAI".to_string()))?;
            
        Ok(content.to_string())
    }
    
    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        // 准备请求数据
        let url = format!("{}/chat/completions", self.base_url);
        
        // 转换消息格式
        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|msg| {
                serde_json::json!({
                    "role": msg.role.as_str(),
                    "content": msg.content.clone(),
                    "name": msg.name.clone(),
                })
            })
            .collect();
        
        // 构建请求正文
        let mut body = serde_json::json!({
            "model": options.model.clone().unwrap_or_else(|| self.model.clone()),
            "messages": api_messages,
        });
        
        // 添加选项参数
        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }
        
        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }
        
        if let Some(stop) = &options.stop {
            body["stop"] = serde_json::json!(stop);
        }
        
        // 发送请求
        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("OpenAI API request failed: {}", e)))?;
            
        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read OpenAI response: {}", e)))?;
            
        if !status.is_success() {
            return Err(Error::Llm(format!(
                "OpenAI API returned error status {}: {}",
                status, text
            )));
        }
        
        // 解析响应
        let response: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse OpenAI response: {}", e)))?;
            
        // 提取生成的文本
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::Llm("Invalid response format from OpenAI".to_string()))?;
            
        Ok(content.to_string())
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
        // 准备请求数据
        let url = format!("{}/embeddings", self.base_url);
        
        let body = serde_json::json!({
            "model": "text-embedding-ada-002",
            "input": text
        });
        
        // 发送请求
        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("OpenAI API request failed: {}", e)))?;
            
        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read OpenAI embedding response: {}", e)))?;
            
        if !status.is_success() {
            return Err(Error::Llm(format!(
                "OpenAI API returned error status {}: {}",
                status, text
            )));
        }
        
        // 解析响应
        let response: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse OpenAI embedding response: {}", e)))?;
            
        // 提取嵌入向量
        let embedding = response["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| Error::Llm("No embedding returned from OpenAI".to_string()))?;
            
        let embedding: Vec<f32> = embedding
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();
            
        Ok(embedding)
    }

    fn supports_function_calling(&self) -> bool {
        true
    }

    async fn generate_with_functions(
        &self,
        messages: &[Message],
        functions: &[FunctionDefinition],
        tool_choice: &ToolChoice,
        options: &LlmOptions,
    ) -> Result<FunctionCallingResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        
        // 转换消息格式
        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|msg| {
                serde_json::json!({
                    "role": msg.role.as_str(),
                    "content": msg.content.clone(),
                    "name": msg.name.clone(),
                })
            })
            .collect();

        // 转换函数定义为 OpenAI tools 格式
        let tools: Vec<Value> = functions.iter().map(|func| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": func.name,
                    "description": func.description,
                    "parameters": func.parameters
                }
            })
        }).collect();

        // 转换工具选择
        let tool_choice_value = match tool_choice {
            ToolChoice::Auto => serde_json::json!("auto"),
            ToolChoice::None => serde_json::json!("none"),
            ToolChoice::Required => serde_json::json!("required"),
            ToolChoice::Function { name } => serde_json::json!({
                "type": "function",
                "function": { "name": name }
            }),
        };

        // 构建请求
        let mut body = serde_json::json!({
            "model": options.model.clone().unwrap_or_else(|| self.model.clone()),
            "messages": api_messages,
        });

        if !tools.is_empty() {
            body["tools"] = Value::Array(tools);
            body["tool_choice"] = tool_choice_value;
        }

        // 添加其他选项
        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }
        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        // 发送请求
        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("OpenAI API request failed: {}", e)))?;

        let status = res.status();
        let response_text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read OpenAI response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Llm(format!(
                "OpenAI API returned error status {}: {}",
                status, response_text
            )));
        }

        // 解析响应
        let response: OpenAIResponse = serde_json::from_str(&response_text)
            .map_err(|e| Error::Llm(format!("Failed to parse OpenAI response: {}", e)))?;

        if response.choices.is_empty() {
            return Err(Error::Llm("No choices in OpenAI response".to_string()));
        }

        let choice = &response.choices[0];
        let message = &choice.message;

        // 转换 function calls
        let function_calls: Vec<FunctionCall> = message.tool_calls
            .iter()
            .filter(|tc| tc.call_type == "function")
            .map(|tc| FunctionCall {
                id: Some(tc.id.clone()),
                name: tc.function.name.clone(),
                arguments: tc.function.arguments.clone(),
            })
            .collect();

        Ok(FunctionCallingResponse {
            content: message.content.clone(),
            function_calls,
            finish_reason: choice.finish_reason.clone().unwrap_or_else(|| "stop".to_string()),
        })
    }
}