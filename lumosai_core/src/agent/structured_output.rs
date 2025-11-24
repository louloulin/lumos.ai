//! Structured Output Implementation for Agents
//!
//! This module provides structured output functionality for agents,
//! allowing them to generate responses that conform to a specific schema.
//!
//! The implementation prioritizes using LLM native structured output APIs
//! (e.g., OpenAI's `response_format` or Anthropic's `structured_outputs`)
//! when available, falling back to prompt engineering for providers that
//! don't support native structured output.

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::BasicAgent;
use super::trait_def::{Agent, AgentStructuredOutput};
use super::types::AgentGenerateOptions;
use crate::error::{Error, Result};
use crate::llm::{LlmOptions, LlmProvider, Message, Role};

/// Implementation of structured output for BasicAgent
#[async_trait]
impl AgentStructuredOutput for BasicAgent {
    async fn generate_structured<T: DeserializeOwned + Send + 'static + schemars::JsonSchema>(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
    ) -> Result<T> {
        // 1. Generate JSON Schema from Rust type using schemars
        let schema = schemars::schema_for!(T);
        let schema_value = serde_json::to_value(&schema.schema)
            .map_err(|e| Error::Agent(format!("Failed to serialize schema: {}", e)))?;

        // 2. Check if LLM supports native structured output
        if self.supports_structured_output() {
            // Use native structured output API
            let llm_options = LlmOptions {
                temperature: options.llm_options.temperature,
                max_tokens: options.llm_options.max_tokens,
                stop: options.llm_options.stop.clone(),
                model: None,
                stream: false,
                extra: serde_json::Map::new(),
            };

            let response_value = self
                .llm()
                .generate_structured(messages, &schema_value, &llm_options)
                .await
                .map_err(|e| {
                    Error::Agent(format!(
                        "Failed to generate structured output using native API: {}",
                        e
                    ))
                })?;

            // Parse the JSON response to target type
            let parsed: T = serde_json::from_value(response_value).map_err(|e| {
                Error::Agent(format!(
                    "Failed to parse structured output from native API: {}",
                    e
                ))
            })?;

            return Ok(parsed);
        }

        // 3. Fallback to prompt engineering (original implementation)
        let schema_prompt = format!(
            "\n\nIMPORTANT: Return your response as valid JSON that follows this schema:\n{}\n\nReturn ONLY the JSON, no additional text.",
            serde_json::to_string_pretty(&schema_value)?
        );

        // 4. Enhance the last message
        let mut enhanced_messages = messages.to_vec();
        if let Some(last_msg) = enhanced_messages.last_mut() {
            last_msg.content.push_str(&schema_prompt);
        }

        // 5. Call LLM generate
        let result = self.generate(&enhanced_messages, options).await?;

        // 6. Extract JSON response
        let json_str = Self::extract_json(&result.response)?;

        // 7. Parse to target type
        let parsed: T = serde_json::from_str(&json_str).map_err(|e| {
            Error::Agent(format!(
                "Failed to parse structured output: {}. Response was: {}",
                e, json_str
            ))
        })?;

        Ok(parsed)
    }
}

impl BasicAgent {
    /// 提取响应中的 JSON 内容
    ///
    /// 尝试从响应中提取纯 JSON，去除额外的文本和markdown标记
    pub fn extract_json(response: &str) -> Result<String> {
        let trimmed = response.trim();

        // 情况 1: 响应已经是纯 JSON
        if (trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']'))
        {
            return Ok(trimmed.to_string());
        }

        // 情况 2: JSON 包裹在 markdown 代码块中
        if let Some(start) = trimmed.find("```json") {
            if let Some(end) = trimmed[start..].find("```") {
                let json_start = start + 7; // "```json".len()
                let json_end = start + end;
                if json_end > json_start {
                    return Ok(trimmed[json_start..json_end].trim().to_string());
                }
            }
        }

        // 情况 3: JSON 包裹在普通代码块中
        if let Some(start) = trimmed.find("```") {
            if let Some(end) = trimmed[start + 3..].find("```") {
                let json_start = start + 3;
                let json_end = start + 3 + end;
                let potential_json = trimmed[json_start..json_end].trim();
                if potential_json.starts_with('{') || potential_json.starts_with('[') {
                    return Ok(potential_json.to_string());
                }
            }
        }

        // 情况 4: 尝试找到 JSON 对象
        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                if end >= start {
                    return Ok(trimmed[start..=end].to_string());
                }
            }
        }

        // 情况 5: 尝试找到 JSON 数组
        if let Some(start) = trimmed.find('[') {
            if let Some(end) = trimmed.rfind(']') {
                if end >= start {
                    return Ok(trimmed[start..=end].to_string());
                }
            }
        }

        // 如果都失败了，尝试直接解析整个响应
        Ok(trimmed.to_string())
    }

    /// 便捷方法：生成简单的结构化输出
    ///
    /// 使用单个消息生成结构化输出
    pub async fn generate_structured_simple<
        T: DeserializeOwned + Send + 'static + schemars::JsonSchema,
    >(
        &self,
        prompt: &str,
    ) -> Result<T> {
        let messages = vec![Message {
            role: Role::User,
            content: prompt.to_string(),
            metadata: None,
            name: None,
        }];

        self.generate_structured(&messages, &AgentGenerateOptions::default())
            .await
    }

    /// 便捷方法：使用自定义 schema 生成结构化输出
    pub async fn generate_with_schema<
        T: DeserializeOwned + Send + 'static + schemars::JsonSchema,
    >(
        &self,
        prompt: &str,
        schema: Value,
    ) -> Result<T> {
        // 创建带 schema 的临时 agent 配置
        // 注意：这里我们不能直接修改 self，所以使用 prompt 增强的方式
        let schema_prompt = format!(
            "{}\n\nReturn response as JSON following this schema:\n{}",
            prompt,
            serde_json::to_string_pretty(&schema)?
        );

        self.generate_structured_simple(&schema_prompt).await
    }
}

/// 结构化输出的 Builder 扩展
///
/// 注意：此扩展为 AgentBuilder 添加了结构化输出配置方法
/// 实际的 schema 存储在 AgentConfig 中
pub trait StructuredOutputExt {
    /// 设置输出 schema（用于文档目的）
    ///
    /// Note: 当前版本通过 prompt engineering 实现结构化输出
    fn structured_output_hint(self) -> Self;
}

impl StructuredOutputExt for crate::agent::AgentBuilder {
    fn structured_output_hint(self) -> Self {
        // 返回 self 以保持链式调用
        // 实际的结构化输出通过 AgentStructuredOutput trait 实现
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
    struct TaskBreakdown {
        title: String,
        subtasks: Vec<String>,
        priority: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
    struct SimpleStruct {
        name: String,
        age: u32,
    }

    #[test]
    fn test_extract_json_pure() {
        let response = r#"{"title": "Test", "subtasks": []}"#;
        let extracted = BasicAgent::extract_json(response).unwrap();
        assert_eq!(extracted, response);
    }

    #[test]
    fn test_extract_json_markdown() {
        let response = r#"Here's the response:
```json
{"title": "Test", "subtasks": []}
```
Hope this helps!"#;
        let extracted = BasicAgent::extract_json(response).unwrap();
        assert!(extracted.contains("\"title\""));
    }

    #[test]
    fn test_extract_json_embedded() {
        let response = "Some text before {\"title\": \"Test\"} some text after";
        let extracted = BasicAgent::extract_json(response).unwrap();
        assert_eq!(extracted, r#"{"title": "Test"}"#);
    }

    #[test]
    fn test_schema_generation() {
        // Test that schemars can generate schema from Rust type
        let schema = schemars::schema_for!(TaskBreakdown);
        let schema_value = serde_json::to_value(&schema.schema).unwrap();

        // Verify schema has expected structure
        assert!(schema_value.get("type").is_some());
        assert_eq!(schema_value["type"], "object");
        assert!(schema_value.get("properties").is_some());
    }

    #[test]
    fn test_simple_struct_schema() {
        let schema = schemars::schema_for!(SimpleStruct);
        let schema_value = serde_json::to_value(&schema.schema).unwrap();

        assert_eq!(schema_value["type"], "object");
        assert!(schema_value["properties"].get("name").is_some());
        assert!(schema_value["properties"].get("age").is_some());
    }
}
