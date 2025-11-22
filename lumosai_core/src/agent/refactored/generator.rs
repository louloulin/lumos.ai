//! Agent 生成器组件 - BasicAgent 重构第三步
//!
//! 这个模块定义了 AgentGenerator，负责协调 AgentCore 和 AgentExecutor 来生成响应。
//! 这是 BasicAgent 重构的第三步，将生成逻辑从 BasicAgent 中分离出来。

use crate::agent::refactored::{AgentCore, AgentExecutor};
use crate::agent::types::{AgentGenerateOptions, AgentGenerateResult, AgentStep, StepType, TokenUsage};
use crate::error::Result;
use crate::llm::{Message, Role};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Agent 生成器
///
/// 负责协调 AgentCore 和 AgentExecutor 来生成响应。
/// 这是 Agent 的"协调"组件，定义了如何生成响应。
///
/// # 示例
///
/// ```rust
/// use lumosai_core::agent::refactored::{AgentCore, AgentExecutor, AgentGenerator};
/// use lumosai_core::agent::{AgentConfig, types::AgentGenerateOptions};
/// use lumosai_core::llm::{Message, Role, MockLlmProvider};
/// use std::sync::Arc;
///
/// # async fn example() -> lumosai_core::Result<()> {
/// let config = AgentConfig {
///     name: "test-agent".to_string(),
///     instructions: "You are a helpful assistant.".to_string(),
///     ..Default::default()
/// };
/// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
///
/// let core = AgentCore::new(config, llm)?;
/// let executor = AgentExecutor::new(core)?;
/// let generator = AgentGenerator::new(executor);
///
/// let messages = vec![Message {
///     role: Role::User,
///     content: "Hello!".to_string(),
///     metadata: None,
///     name: None,
/// }];
/// let options = AgentGenerateOptions::default();
/// let result = generator.generate(&messages, &options).await?;
/// # Ok(())
/// # }
/// ```
pub struct AgentGenerator {
    /// Agent 执行器
    executor: AgentExecutor,
}

impl AgentGenerator {
    /// 创建新的 Agent 生成器
    ///
    /// # 参数
    ///
    /// * `executor` - Agent 执行器
    ///
    /// # 返回
    ///
    /// 返回 `AgentGenerator` 实例。
    pub fn new(executor: AgentExecutor) -> Self {
        Self { executor }
    }

    /// 生成响应
    ///
    /// 这是 AgentGenerator 的核心方法，协调各个组件来生成响应。
    /// 支持多步骤生成，可以循环调用 LLM 和工具直到完成或达到最大步数。
    ///
    /// # 参数
    ///
    /// * `messages` - 输入消息列表
    /// * `options` - 生成选项
    ///
    /// # 返回
    ///
    /// 返回 `Result<AgentGenerateResult>`，包含生成的响应和元数据。
    ///
    /// # 步骤
    ///
    /// 1. 准备消息（从内存检索历史消息）
    /// 2. 准备工具（从执行器获取可用工具）
    /// 3. 多步骤循环：
    ///    - 调用 LLM（使用核心的 LLM 提供者）
    ///    - 处理工具调用（如果需要）
    ///    - 如果还有工具调用，继续下一轮；否则返回最终响应
    /// 4. 更新内存（将消息存储到内存）
    pub async fn generate(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
    ) -> Result<AgentGenerateResult> {
        // 1. 准备消息：从内存检索历史消息
        let mut all_messages = self.prepare_messages(messages, options).await?;

        // 2. 准备工具：从执行器获取可用工具
        let tools = self.prepare_tools(options).await?;

        // 3. 多步骤生成循环
        let max_steps = options.max_steps.unwrap_or(5);
        let mut current_step = 0;
        let mut all_steps = Vec::new();
        let mut total_usage = TokenUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        };

        // 检查是否使用函数调用模式
        let use_function_calling = !tools.is_empty() 
            && self.executor.core().llm().supports_function_calling();

        while current_step < max_steps {
            current_step += 1;

            // 调用 LLM
            let response = if use_function_calling {
                self.call_llm_with_functions(&all_messages, &tools, options).await?
            } else {
                self.call_llm(&all_messages, &tools, options).await?
            };

            // 累计 token 使用量
            total_usage.prompt_tokens += response.usage.prompt_tokens;
            total_usage.completion_tokens += response.usage.completion_tokens;
            total_usage.total_tokens += response.usage.total_tokens;

            // 检查是否有工具调用
            let has_tool_calls = response.steps.iter().any(|step| !step.tool_calls.is_empty());

            if has_tool_calls {
                // 处理工具调用
                let tool_step = response.steps.first().unwrap();
                let tool_results = self.execute_tool_calls(&tool_step.tool_calls).await?;

                // 将工具结果添加到消息中，以便下一轮 LLM 调用
                for (tool_call, tool_result) in tool_step.tool_calls.iter().zip(tool_results.iter()) {
                    let tool_result_json = serde_json::to_string(&tool_result.result)
                        .unwrap_or_else(|_| "{}".to_string());
                    let tool_message = crate::agent::types::tool_message(format!(
                        "Tool {} (id: {}) result: {}",
                        tool_call.name, tool_call.id, tool_result_json
                    ));
                    all_messages.push(tool_message);
                }

                // 添加工具调用步骤
                let mut tool_step_clone = tool_step.clone();
                tool_step_clone.tool_results = tool_results;
                all_steps.push(tool_step_clone);

                // 继续下一轮
                continue;
            } else {
                // 没有工具调用，这是最终响应
                all_steps.extend(response.steps);
                break;
            }
        }

        // 构建最终结果
        let final_response = all_steps
            .last()
            .and_then(|step| step.output.as_ref())
            .map(|msg| msg.content.clone())
            .unwrap_or_default();

        let result = AgentGenerateResult {
            response: final_response,
            steps: all_steps,
            usage: total_usage,
            metadata: HashMap::new(),
        };

        // 4. 更新内存
        self.update_memory(messages, &result).await?;

        Ok(result)
    }

    /// 准备消息：从内存检索历史消息
    async fn prepare_messages(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
    ) -> Result<Vec<Message>> {
        let mut prepared = messages.to_vec();

        // 如果有内存，尝试检索历史消息
        if let Some(memory) = self.executor.memory() {
            // 提取用户查询（提前计算，避免生命周期问题）
            let user_query = messages
                .iter()
                .rev()
                .find(|m| matches!(m.role, Role::User))
                .map(|m| m.content.clone());
            
            let memory_config = crate::memory::MemoryConfig {
                store_id: None,
                namespace: options.thread_id.clone(),
                enabled: true,
                working_memory: None,
                semantic_recall: None,
                last_messages: options.context_window,
                query: user_query,
            };

            if let Ok(historical) = memory.retrieve(&memory_config).await {
                if !historical.is_empty() {
                    // 将历史消息添加到前面
                    let mut all_messages = historical;
                    all_messages.append(&mut prepared);
                    prepared = all_messages;
                }
            }
        }

        Ok(prepared)
    }

    /// 准备工具：从执行器获取可用工具
    async fn prepare_tools(
        &self,
        _options: &AgentGenerateOptions,
    ) -> Result<Vec<crate::llm::FunctionDefinition>> {
        let tools = self.executor.tools();
        let tools_guard = tools.lock().map_err(|_| {
            crate::error::Error::Internal("Failed to lock tools mutex".to_string())
        })?;

        let mut function_definitions = Vec::new();
        for tool in tools_guard.values() {
            let schema = tool.schema();
            // 将 ToolSchema 转换为 JSON Value
            let schema_value = serde_json::to_value(&schema)
                .unwrap_or_else(|_| serde_json::json!({}));
            function_definitions.push(crate::llm::FunctionDefinition {
                name: tool.id().to_string(),
                description: Some(tool.description().to_string()),
                parameters: schema_value,
            });
        }

        Ok(function_definitions)
    }

    /// 调用 LLM（使用函数调用模式）
    async fn call_llm_with_functions(
        &self,
        messages: &[Message],
        tools: &[crate::llm::FunctionDefinition],
        options: &AgentGenerateOptions,
    ) -> Result<AgentGenerateResult> {
        let core = self.executor.core();
        let llm = core.llm();

        // 构建系统消息
        let instructions = options
            .instructions
            .as_deref()
            .unwrap_or_else(|| core.instructions());
        let system_message = Message {
            role: Role::System,
            content: instructions.to_string(),
            metadata: None,
            name: None,
        };

        // 构建完整的消息列表
        let mut all_messages = vec![system_message];
        all_messages.extend_from_slice(messages);

        // 调用 LLM with functions
        let llm_options = &options.llm_options;
        let tool_choice = match &options.tool_choice {
            Some(crate::agent::types::ToolChoice::Auto) => crate::llm::ToolChoice::Auto,
            Some(crate::agent::types::ToolChoice::None) => crate::llm::ToolChoice::None,
            Some(crate::agent::types::ToolChoice::Required) => crate::llm::ToolChoice::Required,
            Some(crate::agent::types::ToolChoice::Tool { tool_name }) => {
                crate::llm::ToolChoice::Function {
                    name: tool_name.clone(),
                }
            }
            None => crate::llm::ToolChoice::Auto,
        };

        let response = llm
            .generate_with_functions(&all_messages, tools, &tool_choice, llm_options)
            .await
            .map_err(|e| crate::error::Error::Llm(format!("LLM generation failed: {}", e)))?;

        // 构建结果（包含函数调用信息）
        let output_message = Message {
            role: Role::Assistant,
            content: response.content.clone().unwrap_or_default(),
            metadata: None,
            name: None,
        };

        // 估算 token 使用量
        let estimated_prompt_tokens = all_messages
            .iter()
            .map(|m| m.content.len() / 4)
            .sum::<usize>();
        let estimated_completion_tokens = response.content.as_ref().map(|c| c.len() / 4).unwrap_or(0);

        // 转换函数调用为工具调用
        let tool_calls: Vec<crate::agent::types::ToolCall> = response
            .function_calls
            .iter()
            .map(|fc| {
                let arguments: HashMap<String, Value> = serde_json::from_str(&fc.arguments)
                    .unwrap_or_else(|_| HashMap::new());
                crate::agent::types::ToolCall {
                    id: fc.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
                    name: fc.name.clone(),
                    arguments,
                }
            })
            .collect();

        Ok(AgentGenerateResult {
            response: response.content.unwrap_or_default(),
            steps: vec![AgentStep {
                id: Uuid::new_v4().to_string(),
                step_type: if tool_calls.is_empty() {
                    StepType::Final
                } else {
                    StepType::Tool
                },
                input: all_messages.clone(),
                output: Some(output_message),
                tool_calls: tool_calls.clone(),
                tool_results: vec![],
                metadata: HashMap::new(),
            }],
            usage: TokenUsage {
                prompt_tokens: estimated_prompt_tokens,
                completion_tokens: estimated_completion_tokens,
                total_tokens: estimated_prompt_tokens + estimated_completion_tokens,
            },
            metadata: HashMap::new(),
        })
    }

    /// 调用 LLM（普通模式）
    async fn call_llm(
        &self,
        messages: &[Message],
        _tools: &[crate::llm::FunctionDefinition],
        options: &AgentGenerateOptions,
    ) -> Result<AgentGenerateResult> {
        let core = self.executor.core();
        let llm = core.llm();

        // 构建系统消息
        let instructions = options
            .instructions
            .as_deref()
            .unwrap_or_else(|| core.instructions());
        let system_message = Message {
            role: Role::System,
            content: instructions.to_string(),
            metadata: None,
            name: None,
        };

        // 构建完整的消息列表
        let mut all_messages = vec![system_message];
        all_messages.extend_from_slice(messages);

        // 调用 LLM - 直接使用 options 中的 llm_options
        let llm_options = &options.llm_options;

        let response_content = llm
            .generate_with_messages(&all_messages, llm_options)
            .await
            .map_err(|e| crate::error::Error::Llm(format!("LLM generation failed: {}", e)))?;

        // 构建结果
        let output_message = Message {
            role: Role::Assistant,
            content: response_content.clone(),
            metadata: None,
            name: None,
        };

        // 估算 token 使用量（简化版本，实际应该从 LLM 响应中获取）
        let estimated_prompt_tokens = all_messages
            .iter()
            .map(|m| m.content.len() / 4) // 粗略估算：4 个字符约等于 1 个 token
            .sum::<usize>();
        let estimated_completion_tokens = response_content.len() / 4;

        Ok(AgentGenerateResult {
            response: response_content,
            steps: vec![AgentStep {
                id: Uuid::new_v4().to_string(),
                step_type: StepType::Final,
                input: all_messages.clone(),
                output: Some(output_message),
                tool_calls: vec![],
                tool_results: vec![],
                metadata: HashMap::new(),
            }],
            usage: TokenUsage {
                prompt_tokens: estimated_prompt_tokens,
                completion_tokens: estimated_completion_tokens,
                total_tokens: estimated_prompt_tokens + estimated_completion_tokens,
            },
            metadata: HashMap::new(),
        })
    }

    /// 执行多个工具调用
    async fn execute_tool_calls(
        &self,
        tool_calls: &[crate::agent::types::ToolCall],
    ) -> Result<Vec<crate::agent::types::ToolResult>> {
        let mut tool_results = Vec::new();

        for tool_call in tool_calls {
            match self.execute_tool_call(tool_call).await {
                Ok(result_value) => {
                    tool_results.push(crate::agent::types::ToolResult {
                        call_id: tool_call.id.clone(),
                        name: tool_call.name.clone(),
                        result: result_value,
                        status: crate::agent::types::ToolResultStatus::Success,
                    });
                }
                Err(e) => {
                    tool_results.push(crate::agent::types::ToolResult {
                        call_id: tool_call.id.clone(),
                        name: tool_call.name.clone(),
                        result: serde_json::json!({"error": e.to_string()}),
                        status: crate::agent::types::ToolResultStatus::Error,
                    });
                }
            }
        }

        Ok(tool_results)
    }

    /// 执行单个工具调用
    async fn execute_tool_call(
        &self,
        tool_call: &crate::agent::types::ToolCall,
    ) -> Result<Value> {
        let tools = self.executor.tools();
        let tools_guard = tools.lock().map_err(|_| {
            crate::error::Error::Internal("Failed to lock tools mutex".to_string())
        })?;

        let tool = tools_guard.get(&tool_call.name).ok_or_else(|| {
            crate::error::Error::NotFound(format!("Tool '{}' not found", tool_call.name))
        })?;

        // 克隆工具以避免持有锁
        let tool_clone = tool.clone();
        drop(tools_guard);

        // 转换参数（tool_call.arguments 已经是 HashMap，可以直接转换为 Value）
        let args_value = serde_json::to_value(&tool_call.arguments)
            .map_err(|e| crate::error::Error::Parsing(format!("Failed to serialize tool arguments: {}", e)))?;

        // 创建执行上下文
        let context = crate::tool::ToolExecutionContext::new()
            .with_tool_call_id(tool_call.id.clone());
        let options = crate::tool::ToolExecutionOptions::default();

        // 执行工具
        tool_clone
            .execute(args_value, context, &options)
            .await
            .map_err(|e| crate::error::Error::Tool(format!("Tool execution failed: {}", e)))
    }

    /// 更新内存：将消息存储到内存
    async fn update_memory(
        &self,
        messages: &[Message],
        _result: &AgentGenerateResult,
    ) -> Result<()> {
        if let Some(memory) = self.executor.memory() {
            // 存储用户消息
            for message in messages {
                if matches!(message.role, Role::User) {
                    if let Err(e) = memory.store(message).await {
                        // 静默失败，不影响主流程
                        eprintln!("Failed to store message to memory: {}", e);
                    }
                }
            }
        }
        Ok(())
    }

    /// 获取执行器
    pub fn executor(&self) -> &AgentExecutor {
        &self.executor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentConfig;
    use crate::llm::MockLlmProvider;

    #[tokio::test]
    async fn test_agent_generator_creation() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();
        let generator = AgentGenerator::new(executor);
        assert!(generator.executor().memory().is_none());
    }

    #[tokio::test]
    async fn test_agent_generator_generate() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello! How can I help you?".to_string()]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();
        let generator = AgentGenerator::new(executor);

        let messages = vec![Message {
            role: Role::User,
            content: "Hello!".to_string(),
            metadata: None,
            name: None,
        }];
        let options = AgentGenerateOptions::default();
        let result = generator.generate(&messages, &options).await.unwrap();
        assert!(!result.response.is_empty());
        assert_eq!(result.steps.len(), 1);
    }

    #[tokio::test]
    async fn test_agent_generator_with_tool() {
        use crate::tool::{create_tool, Tool};

        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello! How can I help you?".to_string()]));

        let core = AgentCore::new(config, llm).unwrap();
        let mut executor = AgentExecutor::new(core).unwrap();

        // 添加一个测试工具
        let echo_tool = create_tool(
            "echo",
            "Echo a message",
            vec![("message", "string", "Message to echo", true)],
            |params| {
                let message = params
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No message");
                Ok(serde_json::json!({"echo": message}))
            },
        )
        .unwrap();
        executor.add_tool(Box::new(echo_tool)).unwrap();

        let generator = AgentGenerator::new(executor);

        let messages = vec![Message {
            role: Role::User,
            content: "Hello!".to_string(),
            metadata: None,
            name: None,
        }];
        let options = AgentGenerateOptions::default();
        let result = generator.generate(&messages, &options).await.unwrap();
        assert!(!result.response.is_empty());
        assert_eq!(result.steps.len(), 1);
    }
}

