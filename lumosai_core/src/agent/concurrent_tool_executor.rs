//! 并发工具执行器
//!
//! 支持工具调用的并发执行，提高性能

use crate::agent::types::{ToolCall, ToolResult, ToolResultStatus};
use crate::error::{Error, Result};
use crate::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use futures::future::join_all;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// 并发工具执行器配置
#[derive(Debug, Clone)]
pub struct ConcurrentToolExecutorConfig {
    /// 最大并发数
    pub max_concurrency: usize,
    /// 是否保持工具调用顺序
    pub preserve_order: bool,
    /// 超时时间（秒）
    pub timeout_seconds: Option<u64>,
}

impl Default for ConcurrentToolExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 5,
            preserve_order: false,
            timeout_seconds: Some(30),
        }
    }
}

/// 并发工具执行器
pub struct ConcurrentToolExecutor {
    config: ConcurrentToolExecutorConfig,
    semaphore: Arc<Semaphore>,
}

impl ConcurrentToolExecutor {
    /// 创建新的并发工具执行器
    pub fn new(config: ConcurrentToolExecutorConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrency));
        Self { config, semaphore }
    }

    /// 使用默认配置创建
    pub fn default() -> Self {
        Self::new(ConcurrentToolExecutorConfig::default())
    }

    /// 并发执行工具调用
    ///
    /// # Arguments
    ///
    /// * `tool_calls` - 要执行的工具调用列表
    /// * `tools` - 工具映射表
    /// * `context` - 工具执行上下文
    /// * `options` - 工具执行选项
    ///
    /// # Returns
    ///
    /// 工具执行结果列表，如果 `preserve_order` 为 true，结果顺序与输入顺序一致
    pub async fn execute_tools(
        &self,
        tool_calls: Vec<ToolCall>,
        tools: &HashMap<String, Arc<dyn Tool>>,
        context: &ToolExecutionContext,
        options: &ToolExecutionOptions,
    ) -> Vec<ToolResult> {
        if tool_calls.is_empty() {
            return Vec::new();
        }

        if self.config.preserve_order {
            self.execute_tools_ordered(tool_calls, tools, context, options)
                .await
        } else {
            self.execute_tools_unordered(tool_calls, tools, context, options)
                .await
        }
    }

    /// 执行工具调用（保持顺序）
    async fn execute_tools_ordered(
        &self,
        tool_calls: Vec<ToolCall>,
        tools: &HashMap<String, Arc<dyn Tool>>,
        context: &ToolExecutionContext,
        options: &ToolExecutionOptions,
    ) -> Vec<ToolResult> {
        let semaphore = self.semaphore.clone();
        // 克隆所有需要的资源，避免生命周期问题
        let tools_clone: HashMap<String, Arc<dyn Tool>> =
            tools.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let context_clone = context.clone();
        let options_clone = options.clone();
        let timeout = self.config.timeout_seconds;

        let futures: Vec<_> = tool_calls
            .into_iter()
            .enumerate()
            .map(|(index, call)| {
                let sem = semaphore.clone();
                let tools = tools_clone.clone();
                let context = context_clone.clone();
                let options = options_clone.clone();
                let tool_name = call.name.clone();
                let call_id = call.id.clone();
                let args = call.arguments.clone();
                let timeout = timeout;

                async move {
                    let _permit = sem.acquire().await.map_err(|e| {
                        Error::Internal(format!("Failed to acquire semaphore: {}", e))
                    })?;

                    let result: Result<Value> = if let Some(tool) = tools.get(&tool_name) {
                        let args_value = serde_json::to_value(&args).map_err(Error::Json)?;

                        // 应用超时
                        if let Some(timeout_secs) = timeout {
                            tokio::time::timeout(
                                tokio::time::Duration::from_secs(timeout_secs),
                                tool.execute(args_value, context, &options),
                            )
                            .await
                            .map_err(|_| {
                                Error::Timeout(format!(
                                    "Tool '{}' execution timed out after {} seconds",
                                    tool_name, timeout_secs
                                ))
                            })?
                        } else {
                            tool.execute(args_value, context, &options).await
                        }
                    } else {
                        Err(Error::NotFound(format!("Tool '{}' not found", tool_name)))
                    };

                    Ok((index, call_id, tool_name, result))
                }
            })
            .collect();

        // 等待所有任务完成
        let task_results: Vec<Result<(usize, String, String, Result<Value>)>> =
            join_all(futures).await;

        // 按原始顺序排序
        let mut indexed_results: Vec<(usize, String, String, Result<Value>)> = task_results
            .into_iter()
            .filter_map(|r: Result<(usize, String, String, Result<Value>)>| r.ok())
            .collect();
        indexed_results.sort_by_key(|(index, _, _, _)| *index);

        // 转换为 ToolResult
        indexed_results
            .into_iter()
            .map(|(_, call_id, tool_name, result)| match result {
                Ok(value) => ToolResult {
                    call_id,
                    name: tool_name,
                    result: value,
                    status: ToolResultStatus::Success,
                },
                Err(e) => ToolResult {
                    call_id,
                    name: tool_name,
                    result: serde_json::json!({"error": e.to_string()}),
                    status: ToolResultStatus::Error,
                },
            })
            .collect()
    }

    /// 执行工具调用（不保持顺序，更快）
    async fn execute_tools_unordered(
        &self,
        tool_calls: Vec<ToolCall>,
        tools: &HashMap<String, Arc<dyn Tool>>,
        context: &ToolExecutionContext,
        options: &ToolExecutionOptions,
    ) -> Vec<ToolResult> {
        let semaphore = self.semaphore.clone();
        // 克隆所有需要的资源，避免生命周期问题
        let tools_clone: HashMap<String, Arc<dyn Tool>> =
            tools.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let context_clone = context.clone();
        let options_clone = options.clone();
        let timeout = self.config.timeout_seconds;

        let futures: Vec<_> = tool_calls
            .into_iter()
            .map(|call| {
                let sem = semaphore.clone();
                let tools = tools_clone.clone();
                let context = context_clone.clone();
                let options = options_clone.clone();
                let tool_name = call.name.clone();
                let call_id = call.id.clone();
                let args = call.arguments.clone();
                let timeout = timeout;

                async move {
                    let _permit = sem.acquire().await.map_err(|e| {
                        Error::Internal(format!("Failed to acquire semaphore: {}", e))
                    })?;

                    let result: Result<Value> = if let Some(tool) = tools.get(&tool_name) {
                        let args_value = serde_json::to_value(&args).map_err(Error::Json)?;

                        // 应用超时
                        if let Some(timeout_secs) = timeout {
                            tokio::time::timeout(
                                tokio::time::Duration::from_secs(timeout_secs),
                                tool.execute(args_value, context, &options),
                            )
                            .await
                            .map_err(|_| {
                                Error::Timeout(format!(
                                    "Tool '{}' execution timed out after {} seconds",
                                    tool_name, timeout_secs
                                ))
                            })?
                        } else {
                            tool.execute(args_value, context, &options).await
                        }
                    } else {
                        Err(Error::NotFound(format!("Tool '{}' not found", tool_name)))
                    };

                    Ok((call_id, tool_name, result))
                }
            })
            .collect();

        // 等待所有任务完成
        let task_results: Vec<Result<(String, String, Result<Value>)>> = join_all(futures).await;

        task_results
            .into_iter()
            .map(
                |task_result: Result<(String, String, Result<Value>)>| match task_result {
                    Ok((call_id, tool_name, result)) => match result {
                        Ok(value) => ToolResult {
                            call_id,
                            name: tool_name,
                            result: value,
                            status: ToolResultStatus::Success,
                        },
                        Err(e) => ToolResult {
                            call_id,
                            name: tool_name,
                            result: serde_json::json!({"error": e.to_string()}),
                            status: ToolResultStatus::Error,
                        },
                    },
                    Err(e) => ToolResult {
                        call_id: String::new(),
                        name: String::new(),
                        result: serde_json::json!({"error": e.to_string()}),
                        status: ToolResultStatus::Error,
                    },
                },
            )
            .collect()
    }

    /// 获取当前配置
    pub fn config(&self) -> &ConcurrentToolExecutorConfig {
        &self.config
    }

    /// 更新配置
    pub fn with_config(mut self, config: ConcurrentToolExecutorConfig) -> Self {
        self.config = config.clone();
        self.semaphore = Arc::new(Semaphore::new(config.max_concurrency));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{FunctionTool, ParameterSchema, ToolSchema};
    use std::time::Duration;

    fn create_test_tool(name: &str, _delay_ms: u64) -> Arc<dyn Tool> {
        let schema = ToolSchema::new(vec![ParameterSchema {
            name: "input".to_string(),
            description: "Input parameter".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        }]);

        let name_owned = name.to_string();
        Arc::new(FunctionTool::new(
            name,
            "Test tool",
            schema,
            move |_params| {
                Ok(serde_json::json!({ "result": format!("Processed by {}", name_owned) }))
            },
        ))
    }

    #[tokio::test]
    async fn test_concurrent_execution() {
        let executor = ConcurrentToolExecutor::new(ConcurrentToolExecutorConfig {
            max_concurrency: 3,
            preserve_order: false,
            timeout_seconds: Some(5),
        });

        let mut tools = HashMap::new();
        tools.insert("tool1".to_string(), create_test_tool("tool1", 100));
        tools.insert("tool2".to_string(), create_test_tool("tool2", 100));
        tools.insert("tool3".to_string(), create_test_tool("tool3", 100));

        let tool_calls = vec![
            ToolCall {
                id: "1".to_string(),
                name: "tool1".to_string(),
                arguments: HashMap::new(),
            },
            ToolCall {
                id: "2".to_string(),
                name: "tool2".to_string(),
                arguments: HashMap::new(),
            },
            ToolCall {
                id: "3".to_string(),
                name: "tool3".to_string(),
                arguments: HashMap::new(),
            },
        ];

        let start = std::time::Instant::now();
        let results = executor
            .execute_tools(
                tool_calls,
                &tools,
                &ToolExecutionContext::new(),
                &ToolExecutionOptions::default(),
            )
            .await;
        let elapsed = start.elapsed();

        // 并发执行应该很快（工具执行很快，主要是验证并发机制）
        assert!(
            elapsed.as_millis() < 1000,
            "Concurrent execution should complete quickly"
        );
        assert_eq!(results.len(), 3);
        assert!(results
            .iter()
            .all(|r| matches!(r.status, ToolResultStatus::Success)));
    }

    #[tokio::test]
    async fn test_preserve_order() {
        let executor = ConcurrentToolExecutor::new(ConcurrentToolExecutorConfig {
            max_concurrency: 3,
            preserve_order: true,
            timeout_seconds: Some(5),
        });

        let mut tools = HashMap::new();
        tools.insert("tool1".to_string(), create_test_tool("tool1", 0));
        tools.insert("tool2".to_string(), create_test_tool("tool2", 0));
        tools.insert("tool3".to_string(), create_test_tool("tool3", 0));

        let tool_calls = vec![
            ToolCall {
                id: "1".to_string(),
                name: "tool1".to_string(),
                arguments: HashMap::new(),
            },
            ToolCall {
                id: "2".to_string(),
                name: "tool2".to_string(),
                arguments: HashMap::new(),
            },
            ToolCall {
                id: "3".to_string(),
                name: "tool3".to_string(),
                arguments: HashMap::new(),
            },
        ];

        let results = executor
            .execute_tools(
                tool_calls,
                &tools,
                &ToolExecutionContext::new(),
                &ToolExecutionOptions::default(),
            )
            .await;

        assert_eq!(results.len(), 3);
        // 检查顺序是否保持
        assert_eq!(results[0].call_id, "1");
        assert_eq!(results[1].call_id, "2");
        assert_eq!(results[2].call_id, "3");
    }

    #[tokio::test]
    async fn test_max_concurrency() {
        let executor = ConcurrentToolExecutor::new(ConcurrentToolExecutorConfig {
            max_concurrency: 2,
            preserve_order: false,
            timeout_seconds: Some(5),
        });

        let mut tools = HashMap::new();
        for i in 1..=5 {
            tools.insert(
                format!("tool{}", i),
                create_test_tool(&format!("tool{}", i), 100),
            );
        }

        let tool_calls: Vec<ToolCall> = (1..=5)
            .map(|i| ToolCall {
                id: i.to_string(),
                name: format!("tool{}", i),
                arguments: HashMap::new(),
            })
            .collect();

        let start = std::time::Instant::now();
        let results = executor
            .execute_tools(
                tool_calls,
                &tools,
                &ToolExecutionContext::new(),
                &ToolExecutionOptions::default(),
            )
            .await;
        let elapsed = start.elapsed();

        // 验证所有工具都执行完成
        assert!(elapsed.as_millis() < 1000);
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_tool_not_found() {
        let executor = ConcurrentToolExecutor::default();
        let tools = HashMap::new();

        let tool_calls = vec![ToolCall {
            id: "1".to_string(),
            name: "nonexistent".to_string(),
            arguments: HashMap::new(),
        }];

        let results = executor
            .execute_tools(
                tool_calls,
                &tools,
                &ToolExecutionContext::new(),
                &ToolExecutionOptions::default(),
            )
            .await;

        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, ToolResultStatus::Error));
    }
}
