//! Structured Output Implementation for Agents
//!
//! This module provides structured output functionality for agents,
//! allowing them to generate responses that conform to a specific schema.

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::sync::Arc;

use super::executor::BasicAgent;
use super::trait_def::{Agent, AgentStructuredOutput};
use super::types::{AgentGenerateOptions, RuntimeContext};
use crate::error::{Error, Result};
use crate::llm::{Message, Role};

/// Implementation of structured output for BasicAgent
#[async_trait]
impl AgentStructuredOutput for BasicAgent {
    async fn generate_structured<T: DeserializeOwned + Send + 'static>(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
    ) -> Result<T> {
        // 1. 使用通用的对象schema
        // Note: 如果agent有配置schema，LLM可能会使用它
        let schema_value = json!({
            "type": "object",
            "properties": {},
            "additionalProperties": true
        });

        // 3. 构建增强的 prompt，要求返回符合 schema 的 JSON
        let schema_prompt = format!(
            "\n\nIMPORTANT: Return your response as valid JSON that follows this schema:\n{}\n\nReturn ONLY the JSON, no additional text.",
            serde_json::to_string_pretty(&schema_value)?
        );

        // 4. 增强最后一条消息
        let mut enhanced_messages = messages.to_vec();
        if let Some(last_msg) = enhanced_messages.last_mut() {
            last_msg.content.push_str(&schema_prompt);
        }

        // 5. 调用 LLM 生成
        let result = self.generate(&enhanced_messages, options).await?;

        // 6. 提取 JSON 响应
        let json_str = Self::extract_json(&result.response)?;

        // 7. 解析为目标类型
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
    pub async fn generate_structured_simple<T: DeserializeOwned + Send + 'static>(
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
    pub async fn generate_with_schema<T: DeserializeOwned + Send + 'static>(
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
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TaskBreakdown {
        title: String,
        subtasks: Vec<String>,
        priority: String,
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
}

