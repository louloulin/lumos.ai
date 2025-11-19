//! 华为 ModelArts MaaS (Model as a Service) LLM Provider
//!
//! 支持华为云 ModelArts 平台的 MaaS 服务，提供 DeepSeek V3.2 等模型。
//!
//! # 使用示例
//!
//! ```rust,no_run
//! use lumosai_core::llm::{HuaweiMaasProvider, LlmProvider, LlmOptions};
//!
//! # async fn example() -> lumosai_core::Result<()> {
//! // 从环境变量创建
//! let provider = HuaweiMaasProvider::from_env()?;
//!
//! // 或手动指定 API Key
//! let provider = HuaweiMaasProvider::new(
//!     "your-api-key".to_string(),
//!     Some("deepseek-v3.2-exp".to_string())
//! );
//!
//! // 生成文本
//! let options = LlmOptions::default()
//!     .with_temperature(0.7)
//!     .with_max_tokens(1000);
//!
//! let response = provider.generate("你好，请介绍一下自己", &options).await?;
//! println!("Response: {}", response);
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::TryStreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::Value;

use super::function_calling::{FunctionCall, FunctionDefinition, ToolChoice};
use super::provider::{FunctionCallingResponse, LlmProvider};
use super::types::{LlmOptions, Message, Role};
use crate::{Error, Result};

/// 华为 MaaS API 响应结构（兼容 OpenAI 格式）
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HuaweiMaasResponse {
    choices: Vec<HuaweiMaasChoice>,
    #[serde(default)]
    usage: Option<HuaweiMaasUsage>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    created: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct HuaweiMaasChoice {
    message: HuaweiMaasMessage,
    #[serde(default)]
    finish_reason: Option<String>,
    #[serde(default)]
    index: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HuaweiMaasMessage {
    role: String,
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<HuaweiMaasToolCall>,
}

#[derive(Debug, Deserialize)]
struct HuaweiMaasToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: HuaweiMaasFunction,
}

#[derive(Debug, Deserialize)]
struct HuaweiMaasFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HuaweiMaasUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// 华为 MaaS 流式响应结构
#[derive(Debug, Deserialize)]
struct HuaweiMaasStreamResponse {
    choices: Vec<HuaweiMaasStreamChoice>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    created: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct HuaweiMaasStreamChoice {
    delta: HuaweiMaasStreamDelta,
    #[serde(default)]
    finish_reason: Option<String>,
    #[serde(default)]
    index: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct HuaweiMaasStreamDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    role: Option<String>,
}

/// 华为 ModelArts MaaS LLM 提供商
///
/// 支持通过华为云 ModelArts 平台访问各种 LLM 模型，包括 DeepSeek V3.2 等。
pub struct HuaweiMaasProvider {
    /// MaaS API Key
    api_key: String,
    /// HTTP 客户端
    client: reqwest::Client,
    /// 模型名称（默认: deepseek-v3.2-exp）
    model: String,
    /// API 基础 URL（默认: https://api.modelarts-maas.com）
    base_url: String,
}

impl HuaweiMaasProvider {
    /// 创建新的华为 MaaS 提供商
    ///
    /// # 参数
    ///
    /// * `api_key` - MaaS API Key
    /// * `model` - 可选的模型名称，默认为 "deepseek-v3.2-exp"
    ///
    /// # 示例
    ///
    /// ```rust
    /// use lumosai_core::llm::HuaweiMaasProvider;
    ///
    /// let provider = HuaweiMaasProvider::new(
    ///     "your-api-key".to_string(),
    ///     Some("deepseek-v3.2-exp".to_string())
    /// );
    /// ```
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            model: model.unwrap_or_else(|| "deepseek-v3.2-exp".to_string()),
            base_url: "https://api.modelarts-maas.com".to_string(),
        }
    }

    /// 从环境变量创建华为 MaaS 提供商
    ///
    /// 环境变量:
    /// - `MAAS_API_KEY` 或 `HUAWEI_MAAS_API_KEY`: API Key
    /// - `MAAS_MODEL` 或 `HUAWEI_MAAS_MODEL`: 模型名称（可选）
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use lumosai_core::llm::HuaweiMaasProvider;
    ///
    /// # fn example() -> lumosai_core::Result<()> {
    /// let provider = HuaweiMaasProvider::from_env()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("MAAS_API_KEY")
            .or_else(|_| std::env::var("HUAWEI_MAAS_API_KEY"))
            .map_err(|_| {
                Error::Configuration(
                    "MAAS_API_KEY or HUAWEI_MAAS_API_KEY environment variable not set"
                        .to_string(),
                )
            })?;

        let model = std::env::var("MAAS_MODEL")
            .or_else(|_| std::env::var("HUAWEI_MAAS_MODEL"))
            .ok();

        Ok(Self::new(api_key, model))
    }

    /// 使用自定义基础 URL 创建华为 MaaS 提供商
    ///
    /// # 参数
    ///
    /// * `api_key` - MaaS API Key
    /// * `base_url` - 自定义 API 基础 URL
    /// * `model` - 可选的模型名称
    pub fn with_base_url(api_key: String, base_url: String, model: Option<String>) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            model: model.unwrap_or_else(|| "deepseek-v3.2-exp".to_string()),
            base_url,
        }
    }

    /// 创建 HTTP 请求头
    fn create_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .expect("Invalid API key format"),
        );
        headers
    }

    /// 将内部消息格式转换为 API 格式
    fn convert_messages(&self, messages: &[Message]) -> Vec<Value> {
        messages
            .iter()
            .map(|msg| {
                serde_json::json!({
                    "role": match msg.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                        Role::Tool => "tool",
                        Role::Function => "function",
                        Role::Custom(_) => "user", // 自定义角色映射为 user
                    },
                    "content": msg.content.clone(),
                })
            })
            .collect()
    }

    /// 创建 SSE 流从 HTTP 响应
    async fn create_sse_stream(
        &self,
        response: reqwest::Response,
    ) -> Result<impl futures::Stream<Item = Result<String>>> {
        use futures::StreamExt;

        let byte_stream = response.bytes_stream();

        Ok(byte_stream
            .map_err(|e| Error::Llm(format!("HTTP 流错误: {}", e)))
            .map(|chunk_result| {
                chunk_result.and_then(|chunk| {
                    // 转换字节为字符串
                    let text = String::from_utf8(chunk.to_vec())
                        .map_err(|e| Error::Llm(format!("UTF-8 解码错误: {}", e)))?;

                    // 按行分割并处理每一行
                    let mut results = Vec::new();
                    for line in text.lines() {
                        // 跳过空行和注释
                        if line.trim().is_empty() || line.starts_with(':') {
                            continue;
                        }

                        // 解析 SSE 格式: "data: {...}"
                        if let Some(data) = line.strip_prefix("data: ") {
                            // 处理流结束标记
                            if data.trim() == "[DONE]" {
                                break;
                            }

                            // 解析 JSON 响应
                            match serde_json::from_str::<HuaweiMaasStreamResponse>(data) {
                                Ok(stream_response) => {
                                    if let Some(choice) = stream_response.choices.first() {
                                        if let Some(content) = &choice.delta.content {
                                            if !content.is_empty() {
                                                results.push(content.clone());
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    // 记录错误但继续处理
                                    eprintln!("解析华为 MaaS 流式响应失败: {}, 数据: {}", e, data);
                                }
                            }
                        }
                    }

                    // 合并此块中的所有内容
                    Ok(results.join(""))
                })
            })
            .filter_map(|result| async move {
                match result {
                    Ok(content) if !content.is_empty() => Some(Ok(content)),
                    Ok(_) => None, // 跳过空内容
                    Err(e) => Some(Err(e)),
                }
            }))
    }
}

#[async_trait]
impl LlmProvider for HuaweiMaasProvider {
    fn name(&self) -> &str {
        "huawei_maas"
    }

    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        // 转换为消息格式
        let messages = vec![serde_json::json!({
            "role": "user",
            "content": prompt
        })];

        // 准备请求
        let url = format!("{}/v2/chat/completions", self.base_url);

        // 构建请求体
        let mut body = serde_json::json!({
            "model": options.model.clone().unwrap_or_else(|| self.model.clone()),
            "messages": messages,
        });

        // 添加可选参数
        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }

        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        // 发送请求
        let response = self
            .client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("华为 MaaS API 请求失败: {}", e)))?;

        // 检查响应状态
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Llm(format!(
                "华为 MaaS API 错误 ({}): {}",
                status, error_text
            )));
        }

        // 解析响应
        let maas_response: HuaweiMaasResponse = response
            .json()
            .await
            .map_err(|e| Error::Llm(format!("解析华为 MaaS 响应失败: {}", e)))?;

        // 提取生成的文本
        maas_response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| Error::Llm("华为 MaaS 响应中没有内容".to_string()))
    }

    async fn generate_with_messages(
        &self,
        messages: &[Message],
        options: &LlmOptions,
    ) -> Result<String> {
        let api_messages = self.convert_messages(messages);

        let url = format!("{}/v2/chat/completions", self.base_url);

        let mut body = serde_json::json!({
            "model": options.model.clone().unwrap_or_else(|| self.model.clone()),
            "messages": api_messages,
        });

        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }

        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        let response = self
            .client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("华为 MaaS API 请求失败: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Llm(format!(
                "华为 MaaS API 错误 ({}): {}",
                status, error_text
            )));
        }

        let maas_response: HuaweiMaasResponse = response
            .json()
            .await
            .map_err(|e| Error::Llm(format!("解析华为 MaaS 响应失败: {}", e)))?;

        maas_response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| Error::Llm("华为 MaaS 响应中没有内容".to_string()))
    }

    async fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        options: &'a LlmOptions,
    ) -> Result<BoxStream<'a, Result<String>>> {
        let messages = vec![serde_json::json!({
            "role": "user",
            "content": prompt
        })];

        let url = format!("{}/v2/chat/completions", self.base_url);

        let mut body = serde_json::json!({
            "model": options.model.clone().unwrap_or_else(|| self.model.clone()),
            "messages": messages,
            "stream": true, // 启用流式响应
        });

        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }

        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        // 发送请求
        let response = self
            .client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("华为 MaaS API 流式请求失败: {}", e)))?;

        // 检查响应状态
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Llm(format!(
                "华为 MaaS API 流式错误 ({}): {}",
                status, error_text
            )));
        }

        // 创建 SSE 流
        let stream = self.create_sse_stream(response).await?;
        Ok(Box::pin(stream))
    }

    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // 华为 MaaS 暂不支持 embedding API
        let _ = text;
        Err(Error::Llm(
            "华为 MaaS 暂不支持 embedding 功能，请使用 OpenAI 或其他提供商".to_string(),
        ))
    }

    async fn generate_with_functions(
        &self,
        messages: &[Message],
        functions: &[FunctionDefinition],
        tool_choice: &ToolChoice,
        options: &LlmOptions,
    ) -> Result<FunctionCallingResponse> {
        let api_messages = self.convert_messages(messages);
        let url = format!("{}/v2/chat/completions", self.base_url);

        // 转换函数定义为 OpenAI 格式的 tools
        let tools: Vec<Value> = functions
            .iter()
            .map(|func| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": func.name,
                        "description": func.description,
                        "parameters": func.parameters,
                    }
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": options.model.clone().unwrap_or_else(|| self.model.clone()),
            "messages": api_messages,
            "tools": tools,
        });

        // 添加 tool_choice
        body["tool_choice"] = match tool_choice {
            ToolChoice::Auto => serde_json::json!("auto"),
            ToolChoice::None => serde_json::json!("none"),
            ToolChoice::Required => serde_json::json!("required"),
            ToolChoice::Function { name } => serde_json::json!({
                "type": "function",
                "function": { "name": name }
            }),
        };

        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }

        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        let response = self
            .client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("华为 MaaS API 请求失败: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Llm(format!(
                "华为 MaaS API 错误 ({}): {}",
                status, error_text
            )));
        }

        let maas_response: HuaweiMaasResponse = response
            .json()
            .await
            .map_err(|e| Error::Llm(format!("解析华为 MaaS 响应失败: {}", e)))?;

        let choice = maas_response
            .choices
            .first()
            .ok_or_else(|| Error::Llm("华为 MaaS 响应中没有选择".to_string()))?;

        // 检查是否有函数调用
        let function_calls: Vec<FunctionCall> = if !choice.message.tool_calls.is_empty() {
            choice
                .message
                .tool_calls
                .iter()
                .map(|tool_call| FunctionCall {
                    id: Some(tool_call.id.clone()),
                    name: tool_call.function.name.clone(),
                    arguments: tool_call.function.arguments.clone(),
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(FunctionCallingResponse {
            content: choice.message.content.clone(),
            function_calls,
            finish_reason: choice.finish_reason.clone().unwrap_or_else(|| "stop".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = HuaweiMaasProvider::new(
            "test-api-key".to_string(),
            Some("deepseek-v3.2-exp".to_string()),
        );
        assert_eq!(provider.name(), "huawei_maas");
        assert_eq!(provider.model, "deepseek-v3.2-exp");
    }

    #[test]
    fn test_default_model() {
        let provider = HuaweiMaasProvider::new("test-api-key".to_string(), None);
        assert_eq!(provider.model, "deepseek-v3.2-exp");
    }

    #[test]
    fn test_custom_base_url() {
        let provider = HuaweiMaasProvider::with_base_url(
            "test-api-key".to_string(),
            "https://custom.api.com".to_string(),
            None,
        );
        assert_eq!(provider.base_url, "https://custom.api.com");
    }

    #[test]
    fn test_message_conversion() {
        let provider = HuaweiMaasProvider::new("test-api-key".to_string(), None);
        let messages = vec![
            Message {
                role: Role::System,
                content: "You are helpful".to_string(),
                metadata: None,
                name: None,
            },
            Message {
                role: Role::User,
                content: "Hello".to_string(),
                metadata: None,
                name: None,
            },
        ];

        let converted = provider.convert_messages(&messages);
        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0]["role"], "system");
        assert_eq!(converted[1]["role"], "user");
    }
}
