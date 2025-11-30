//! Agent 生成器组件 - BasicAgent 重构第三步
//!
//! 这个模块定义了 AgentGenerator，负责协调 AgentCore 和 AgentExecutor 来生成响应。
//! 这是 BasicAgent 重构的第三步，将生成逻辑从 BasicAgent 中分离出来。

use crate::agent::api_consistency::ApiStandardizer;
use crate::agent::refactored::AgentExecutor;
use crate::agent::types::{
    AgentGenerateOptions, AgentGenerateResult, AgentStep, AgentStreamOptions, RuntimeContext,
    StepType, TokenUsage,
};
use crate::error::Result;
use crate::llm::{Message, Role};
use crate::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// 包装器：将 Box<dyn Tool> 转换为 Arc<dyn Tool>
///
/// 这个包装器允许我们在 ConcurrentToolExecutor 中使用 Box<dyn Tool>
/// 通过 clone_box() 方法实现克隆功能
struct BoxToolWrapper {
    tool: Box<dyn Tool>,
    base: crate::base::BaseComponent,
}

impl Clone for BoxToolWrapper {
    fn clone(&self) -> Self {
        Self {
            tool: self.tool.clone_box(),
            base: self.base.clone(),
        }
    }
}

impl std::fmt::Debug for BoxToolWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxToolWrapper")
            .field("tool_id", &self.tool.id())
            .finish_non_exhaustive()
    }
}

impl crate::base::Base for BoxToolWrapper {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }

    fn component(&self) -> crate::compat::Component {
        self.base.component()
    }

    fn logger(&self) -> std::sync::Arc<dyn crate::logger::Logger> {
        self.base.logger()
    }

    fn set_logger(&mut self, logger: std::sync::Arc<dyn crate::logger::Logger>) {
        self.base.set_logger(logger);
    }

    fn telemetry(&self) -> Option<std::sync::Arc<dyn crate::telemetry::TelemetrySink>> {
        self.base.telemetry()
    }

    fn set_telemetry(&mut self, telemetry: std::sync::Arc<dyn crate::telemetry::TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
}

#[async_trait]
impl Tool for BoxToolWrapper {
    fn id(&self) -> &str {
        self.tool.id()
    }

    fn description(&self) -> &str {
        self.tool.description()
    }

    fn schema(&self) -> crate::tool::ToolSchema {
        self.tool.schema()
    }

    async fn execute(
        &self,
        args: Value,
        context: ToolExecutionContext,
        options: &ToolExecutionOptions,
    ) -> Result<Value> {
        self.tool.execute(args, context, options).await
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        self.tool.clone_box()
    }
}

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
        // 输入验证
        if messages.is_empty() {
            return Err(crate::error::Error::InvalidInput(
                "Messages cannot be empty".to_string(),
            ));
        }

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
        let use_function_calling =
            !tools.is_empty() && self.executor.core().llm().supports_function_calling();

        while current_step < max_steps {
            current_step += 1;

            // 调用 LLM
            let response = if use_function_calling {
                self.call_llm_with_functions(&all_messages, &tools, options)
                    .await?
            } else {
                self.call_llm(&all_messages, &tools, options).await?
            };

            // 累计 token 使用量
            total_usage.prompt_tokens += response.usage.prompt_tokens;
            total_usage.completion_tokens += response.usage.completion_tokens;
            total_usage.total_tokens += response.usage.total_tokens;

            // 检查是否有工具调用
            let has_tool_calls = response
                .steps
                .iter()
                .any(|step| !step.tool_calls.is_empty());

            if has_tool_calls {
                // 处理工具调用
                // 找到第一个包含工具调用的步骤
                let tool_step = response
                    .steps
                    .iter()
                    .find(|step| !step.tool_calls.is_empty())
                    .ok_or_else(|| {
                        crate::error::Error::Internal(
                            "Tool calls detected but no step found".to_string(),
                        )
                    })?;
                let tool_results = self.execute_tool_calls(&tool_step.tool_calls).await?;

                // 将工具结果添加到消息中，以便下一轮 LLM 调用
                for (tool_call, tool_result) in tool_step.tool_calls.iter().zip(tool_results.iter())
                {
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

                // 继续下一轮（循环条件会自动检查 max_steps）
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

        // 标准化响应格式
        let standardized_response = ApiStandardizer::standardize_response(&final_response);

        let result = AgentGenerateResult {
            response: standardized_response,
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
        // 在同步块中完成所有操作，确保 MutexGuard 在 await 之前被释放
        let function_definitions = {
            let tools_guard = tools.lock().map_err(|_| {
                crate::error::Error::Internal("Failed to lock tools mutex".to_string())
            })?;

            let mut function_definitions = Vec::new();
            for tool in tools_guard.values() {
                let schema = tool.schema();
                // 将 ToolSchema 转换为 JSON Value
                let schema_value =
                    serde_json::to_value(&schema).unwrap_or_else(|_| serde_json::json!({}));
                function_definitions.push(crate::llm::FunctionDefinition {
                    name: tool.id().to_string(),
                    description: Some(tool.description().to_string()),
                    parameters: schema_value,
                });
            }
            // tools_guard 在这里被释放
            function_definitions
        };

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

        // 如果有 LLM router，使用 router 选择 provider，否则使用固定的 provider
        let llm = if let Some(router) = self.executor.llm_router() {
            let llm_options = &options.llm_options;
            router.select_provider(llm_options).await?
        } else {
            core.llm().clone()
        };

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

        // 使用 RetryExecutor 包装 LLM 调用（如果可用）
        let response = if let Some(retry_executor) = self.executor.retry_executor() {
            let context = RuntimeContext::default();
            let llm_clone = llm.clone();
            let all_messages_clone = all_messages.clone();
            let tools_clone = tools.to_vec();
            let tool_choice_clone = tool_choice.clone();
            let llm_options_clone = llm_options.clone();

            retry_executor
                .execute(
                    || {
                        let llm = llm_clone.clone();
                        let all_messages = all_messages_clone.clone();
                        let tools = tools_clone.clone();
                        let tool_choice = tool_choice_clone.clone();
                        let llm_options = llm_options_clone.clone();
                        async move {
                            llm.generate_with_functions(
                                &all_messages,
                                &tools,
                                &tool_choice,
                                &llm_options,
                            )
                            .await
                            .map_err(|e| {
                                crate::error::Error::Llm(format!("LLM generation failed: {}", e))
                            })
                        }
                    },
                    &context,
                )
                .await?
        } else {
            llm.generate_with_functions(&all_messages, tools, &tool_choice, llm_options)
                .await
                .map_err(|e| crate::error::Error::Llm(format!("LLM generation failed: {}", e)))?
        };

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
        let estimated_completion_tokens =
            response.content.as_ref().map(|c| c.len() / 4).unwrap_or(0);

        // 转换函数调用为工具调用
        let tool_calls: Vec<crate::agent::types::ToolCall> = response
            .function_calls
            .iter()
            .map(|fc| {
                let arguments: HashMap<String, Value> =
                    serde_json::from_str(&fc.arguments).unwrap_or_else(|_| HashMap::new());
                crate::agent::types::ToolCall {
                    id: fc.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
                    name: fc.name.clone(),
                    arguments,
                }
            })
            .collect();

        // 标准化响应格式
        let response_content = response.content.unwrap_or_default();
        let standardized_response = ApiStandardizer::standardize_response(&response_content);

        Ok(AgentGenerateResult {
            response: standardized_response,
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

        // 如果有 LLM router，使用 router 选择 provider，否则使用固定的 provider
        let llm = if let Some(router) = self.executor.llm_router() {
            let llm_options = &options.llm_options;
            router.select_provider(llm_options).await?
        } else {
            core.llm().clone()
        };

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

        // 使用 RetryExecutor 包装 LLM 调用（如果可用）
        let response_content = if let Some(retry_executor) = self.executor.retry_executor() {
            let context = RuntimeContext::default();
            let llm_clone = llm.clone();
            let all_messages_clone = all_messages.clone();
            let llm_options_clone = llm_options.clone();

            retry_executor
                .execute(
                    || {
                        let llm = llm_clone.clone();
                        let all_messages = all_messages_clone.clone();
                        let llm_options = llm_options_clone.clone();
                        async move {
                            llm.generate_with_messages(&all_messages, &llm_options)
                                .await
                                .map_err(|e| {
                                    crate::error::Error::Llm(format!(
                                        "LLM generation failed: {}",
                                        e
                                    ))
                                })
                        }
                    },
                    &context,
                )
                .await?
        } else {
            llm.generate_with_messages(&all_messages, llm_options)
                .await
                .map_err(|e| crate::error::Error::Llm(format!("LLM generation failed: {}", e)))?
        };

        // 标准化响应格式
        let standardized_response = ApiStandardizer::standardize_response(&response_content);

        // 构建结果
        let output_message = Message {
            role: Role::Assistant,
            content: standardized_response.clone(),
            metadata: None,
            name: None,
        };

        // 估算 token 使用量（简化版本，实际应该从 LLM 响应中获取）
        let estimated_prompt_tokens = all_messages
            .iter()
            .map(|m| m.content.len() / 4) // 粗略估算：4 个字符约等于 1 个 token
            .sum::<usize>();
        let estimated_completion_tokens = standardized_response.len() / 4;

        Ok(AgentGenerateResult {
            response: standardized_response,
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
    ///
    /// 这个方法支持并发和顺序两种执行模式：
    /// - 如果配置了 ConcurrentToolExecutor，使用并发执行
    /// - 否则使用顺序执行
    ///
    /// # 参数
    ///
    /// * `tool_calls` - 要执行的工具调用列表
    ///
    /// # 返回
    ///
    /// 返回工具执行结果列表
    pub async fn execute_tool_calls(
        &self,
        tool_calls: &[crate::agent::types::ToolCall],
    ) -> Result<Vec<crate::agent::types::ToolResult>> {
        // 输入验证：空工具调用列表直接返回
        if tool_calls.is_empty() {
            return Ok(Vec::new());
        }

        // 如果配置了 ConcurrentToolExecutor，使用并发执行
        if let Some(concurrent_executor) = self.executor.concurrent_tool_executor() {
            // 在同步块中获取工具并转换为 Arc<dyn Tool>，确保 MutexGuard 在 await 之前被释放
            let tools_map: std::collections::HashMap<String, Arc<dyn crate::tool::Tool>> = {
                let tools_arc = self.executor.tools();
                let tools_guard = tools_arc.lock().map_err(|_| {
                    crate::error::Error::Internal("Failed to lock tools".to_string())
                })?;

                // 将 Box<dyn Tool> 转换为 Arc<dyn Tool>
                let mut tools_map: std::collections::HashMap<String, Arc<dyn crate::tool::Tool>> =
                    std::collections::HashMap::new();
                for (name, tool) in tools_guard.iter() {
                    // 创建一个包装器，将 Box<dyn Tool> 转换为 Arc<dyn Tool>
                    // 由于 Box<dyn Tool> 不能直接转换为 Arc，我们需要克隆工具
                    // 这里我们使用一个简单的包装器
                    let tool_arc: Arc<dyn crate::tool::Tool> = Arc::new(BoxToolWrapper {
                        tool: tool.clone(),
                        base: crate::base::BaseComponent::new_with_name(
                            tool.id().to_string(),
                            crate::compat::Component::Tool,
                        ),
                    });
                    tools_map.insert(name.clone(), tool_arc);
                }
                // tools_guard 在这里被释放
                tools_map
            };

            // 准备工具调用和上下文
            let tool_calls_vec: Vec<crate::agent::types::ToolCall> = tool_calls.to_vec();
            let context = crate::tool::ToolExecutionContext::default();
            let options = crate::tool::ToolExecutionOptions::default();

            // 使用并发执行器执行工具
            let results = concurrent_executor
                .execute_tools(tool_calls_vec, &tools_map, &context, &options)
                .await;

            Ok(results)
        } else {
            // 否则使用顺序执行（原有逻辑）
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
    }

    /// 执行单个工具调用
    async fn execute_tool_call(&self, tool_call: &crate::agent::types::ToolCall) -> Result<Value> {
        // 在同步块中获取工具并克隆，确保 MutexGuard 在 await 之前被释放
        let tool_clone = {
            let tools = self.executor.tools();
            let tools_guard = tools.lock().map_err(|_| {
                crate::error::Error::Internal("Failed to lock tools mutex".to_string())
            })?;

            let tool = tools_guard.get(&tool_call.name).ok_or_else(|| {
                crate::error::Error::NotFound(format!("Tool '{}' not found", tool_call.name))
            })?;

            // 克隆工具以避免持有锁
            tool.clone()
            // tools_guard 在这里被释放
        };

        // 转换参数（tool_call.arguments 已经是 HashMap，可以直接转换为 Value）
        let args_value = serde_json::to_value(&tool_call.arguments).map_err(|e| {
            crate::error::Error::Parsing(format!("Failed to serialize tool arguments: {}", e))
        })?;

        // 创建执行上下文
        let context =
            crate::tool::ToolExecutionContext::new().with_tool_call_id(tool_call.id.clone());
        let options = crate::tool::ToolExecutionOptions::default();

        // 使用 RetryExecutor 包装工具调用（如果可用）
        if let Some(retry_executor) = self.executor.retry_executor() {
            // 确保所有捕获的值都是 Send + Sync
            let tool_clone2 = tool_clone.clone();
            let args_value_clone = args_value.clone();
            let context_clone = context.clone();
            let options_clone = options.clone();

            // 在同步块中准备所有数据，确保没有非 Send 的类型被捕获
            let (tool_final, args_final, ctx_final, opts_final) =
                { (tool_clone2, args_value_clone, context_clone, options_clone) };

            // 创建 context_rt 在闭包外部，确保它是 Send
            let context_rt = RuntimeContext::default();

            retry_executor
                .execute(
                    move || {
                        let tool = tool_final.clone();
                        let args = args_final.clone();
                        let ctx = ctx_final.clone();
                        let opts = opts_final.clone();
                        async move {
                            tool.execute(args, ctx, &opts).await.map_err(|e| {
                                crate::error::Error::Tool(format!("Tool execution failed: {}", e))
                            })
                        }
                    },
                    &context_rt,
                )
                .await
        } else {
            tool_clone
                .execute(args_value, context, &options)
                .await
                .map_err(|e| crate::error::Error::Tool(format!("Tool execution failed: {}", e)))
        }
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

    /// 流式生成响应
    ///
    /// 生成响应并以流的形式返回，支持实时输出。
    ///
    /// # 参数
    ///
    /// * `messages` - 输入消息列表
    /// * `options` - 流式生成选项
    ///
    /// # 返回
    ///
    /// 返回一个流，每个元素是一个字符串块。
    ///
    /// # 实现说明
    ///
    /// 当前实现使用简化版本：先生成完整响应，然后分块返回。
    /// 未来可以改进为使用 LLM 的原生流式接口。
    pub async fn stream<'a>(
        &'a self,
        messages: &'a [Message],
        options: &'a AgentStreamOptions,
    ) -> Result<BoxStream<'a, Result<String>>> {
        // 输入验证
        if messages.is_empty() {
            return Err(crate::error::Error::InvalidInput(
                "Messages cannot be empty".to_string(),
            ));
        }

        // 将 AgentStreamOptions 转换为 AgentGenerateOptions
        let generate_options = AgentGenerateOptions {
            system_message: None,
            instructions: options.instructions.clone(),
            context: options.context.clone(),
            memory_options: options.memory_options.clone(),
            thread_id: options.thread_id.clone(),
            resource_id: options.resource_id.clone(),
            run_id: options.run_id.clone(),
            max_steps: options.max_steps,
            tool_choice: options.tool_choice.clone(),
            context_window: None,
            llm_options: options.llm_options.clone(),
        };

        // 生成完整响应
        let result = self.generate(messages, &generate_options).await?;

        // 将响应分块返回（智能分块，尊重单词和句子边界）
        let chunks = self.create_smart_chunks(&result.response);

        // 创建流
        let stream = futures::stream::iter(chunks).map(Ok).boxed();

        Ok(stream)
    }

    /// 创建智能分块，尊重单词和句子边界
    fn create_smart_chunks(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let target_chunk_size = 50; // 每个块的目标大小（字符数）

        for word in text.split_whitespace() {
            if current_chunk.len() + word.len() + 1 > target_chunk_size && !current_chunk.is_empty()
            {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
            }

            if !current_chunk.is_empty() {
                current_chunk.push(' ');
            }
            current_chunk.push_str(word);
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        // 如果没有创建任何块，返回原始文本
        if chunks.is_empty() && !text.is_empty() {
            chunks.push(text.to_string());
        }

        chunks
    }

    /// 获取执行器
    pub fn executor(&self) -> &AgentExecutor {
        &self.executor
    }

    /// 获取执行器（可变引用）
    pub fn executor_mut(&mut self) -> &mut AgentExecutor {
        &mut self.executor
    }

    /// 检查是否有工具
    ///
    /// # 返回
    ///
    /// 如果有工具返回 `true`，否则返回 `false`。
    pub fn has_tools(&self) -> bool {
        self.executor.has_tools()
    }

    /// 获取工具数量
    ///
    /// # 返回
    ///
    /// 返回已注册的工具数量。
    pub fn tool_count(&self) -> usize {
        self.executor.tool_count()
    }
}

impl std::fmt::Debug for AgentGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentGenerator")
            .field("executor", &self.executor)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{
        refactored::{AgentCore, AgentExecutor},
        AgentConfig,
    };
    use crate::llm::{Message, MockLlmProvider, Role};
    use crate::tool::{GenericTool, ToolExecutionContext, ToolSchema};
    use serde_json::json;

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
        let llm = Arc::new(MockLlmProvider::new(vec![
            "Hello! How can I help you?".to_string()
        ]));

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
        use crate::tool::create_tool;

        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec![
            "Hello! How can I help you?".to_string()
        ]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();

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

    #[tokio::test]
    async fn test_agent_generator_stream() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec![
            "Hello! How can I help you? This is a longer response to test streaming.".to_string(),
        ]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();
        let generator = AgentGenerator::new(executor);

        let messages = vec![Message {
            role: Role::User,
            content: "Hello!".to_string(),
            metadata: None,
            name: None,
        }];
        let options = AgentStreamOptions::default();
        let mut stream = generator.stream(&messages, &options).await.unwrap();

        // 收集所有块
        let mut chunks = Vec::new();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.unwrap();
            chunks.push(chunk);
        }

        // 验证流式输出
        assert!(!chunks.is_empty());
        let full_response: String = chunks.join("");
        assert!(!full_response.is_empty());
        assert!(full_response.contains("Hello") || full_response.contains("help"));
    }

    #[tokio::test]
    async fn test_agent_generator_api_standardization() {
        let config = crate::agent::AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        // 测试空响应标准化
        let llm = Arc::new(crate::llm::MockLlmProvider::new(vec!["".to_string()]));

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

        // 验证响应已被标准化（空响应应该被替换为默认消息）
        assert!(!result.response.is_empty());
        assert!(result.response.contains("apologize") || result.response.contains("couldn't"));

        // 测试响应以标点符号结尾
        let llm2 = Arc::new(crate::llm::MockLlmProvider::new(vec!["Hello".to_string()]));
        let core2 = AgentCore::new(
            crate::agent::AgentConfig {
                name: "test-agent-2".to_string(),
                instructions: "You are a helpful assistant.".to_string(),
                ..Default::default()
            },
            llm2,
        )
        .unwrap();
        let executor2 = AgentExecutor::new(core2).unwrap();
        let generator2 = AgentGenerator::new(executor2);

        let result2 = generator2.generate(&messages, &options).await.unwrap();
        // 验证响应以标点符号结尾（标准化后应该添加句号）
        assert!(
            result2.response.ends_with('.')
                || result2.response.ends_with('!')
                || result2.response.ends_with('?')
        );
    }

    #[tokio::test]
    async fn test_agent_generator_with_llm_router() {
        use crate::llm::{LlmRouter, RoutingStrategy};

        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        // 创建多个 providers
        let provider1: Arc<dyn crate::llm::LlmProvider> = Arc::new(MockLlmProvider::new(vec![
            "Response from provider 1".to_string(),
        ]));
        let provider2: Arc<dyn crate::llm::LlmProvider> = Arc::new(MockLlmProvider::new(vec![
            "Response from provider 2".to_string(),
        ]));

        // 创建 router
        let providers = vec![provider1.clone(), provider2.clone()];
        let router = Arc::new(LlmRouter::new(providers).with_strategy(RoutingStrategy::RoundRobin));

        // 使用第一个 provider 创建 core（作为 fallback）
        let core = AgentCore::new(config, provider1.clone()).unwrap();
        let executor = AgentExecutor::new(core).unwrap().with_llm_router(router);
        let generator = AgentGenerator::new(executor);

        let messages = vec![Message {
            role: Role::User,
            content: "Hello!".to_string(),
            metadata: None,
            name: None,
        }];

        let options = AgentGenerateOptions::default();
        let result = generator.generate(&messages, &options).await.unwrap();

        // 验证响应已生成（router 应该选择了某个 provider）
        assert!(!result.response.is_empty());
        assert_eq!(result.steps.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_tool_calls_with_concurrent_executor() {
        // 创建配置和 LLM
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        // 创建 core 和 executor
        let core = AgentCore::new(config, llm).unwrap();
        let mut executor = AgentExecutor::new(core).unwrap();

        // 创建并发工具执行器
        let concurrent_config =
            crate::agent::concurrent_tool_executor::ConcurrentToolExecutorConfig {
                max_concurrency: 2,
                preserve_order: true,
                timeout_seconds: Some(10),
            };
        let concurrent_executor = Arc::new(
            crate::agent::concurrent_tool_executor::ConcurrentToolExecutor::new(concurrent_config),
        );
        executor = executor.with_concurrent_tool_executor(concurrent_executor);

        // 创建测试工具
        let tool1 = GenericTool::new(
            "test_tool_1",
            "Test tool 1",
            ToolSchema::default(),
            |_params: Value, _context: ToolExecutionContext| -> Result<Value> {
                Ok(json!({"result": "tool1"}))
            },
        );
        let tool2 = GenericTool::new(
            "test_tool_2",
            "Test tool 2",
            ToolSchema::default(),
            |_params: Value, _context: ToolExecutionContext| -> Result<Value> {
                Ok(json!({"result": "tool2"}))
            },
        );

        // 添加工具
        executor.add_tool(Box::new(tool1)).unwrap();
        executor.add_tool(Box::new(tool2)).unwrap();

        // 创建 generator
        let generator = AgentGenerator::new(executor);

        // 创建工具调用
        let tool_calls = vec![
            crate::agent::types::ToolCall {
                id: "call1".to_string(),
                name: "test_tool_1".to_string(),
                arguments: HashMap::new(),
            },
            crate::agent::types::ToolCall {
                id: "call2".to_string(),
                name: "test_tool_2".to_string(),
                arguments: HashMap::new(),
            },
        ];

        // 执行工具调用
        let results = generator.execute_tool_calls(&tool_calls).await.unwrap();

        // 验证结果
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "test_tool_1");
        assert_eq!(
            results[0].status,
            crate::agent::types::ToolResultStatus::Success
        );
        assert_eq!(results[1].name, "test_tool_2");
        assert_eq!(
            results[1].status,
            crate::agent::types::ToolResultStatus::Success
        );
    }

    #[tokio::test]
    async fn test_execute_tool_calls_sequential() {
        // 测试顺序执行模式（没有配置并发执行器）
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();

        // 创建测试工具
        let tool1 = GenericTool::new(
            "test_tool_1",
            "Test tool 1",
            ToolSchema::default(),
            |_params: Value, _context: ToolExecutionContext| -> Result<Value> {
                Ok(json!({"result": "tool1"}))
            },
        );
        let tool2 = GenericTool::new(
            "test_tool_2",
            "Test tool 2",
            ToolSchema::default(),
            |_params: Value, _context: ToolExecutionContext| -> Result<Value> {
                Ok(json!({"result": "tool2"}))
            },
        );

        // 添加工具
        executor.add_tool(Box::new(tool1)).unwrap();
        executor.add_tool(Box::new(tool2)).unwrap();

        // 创建 generator（不配置并发执行器，使用顺序执行）
        let generator = AgentGenerator::new(executor);

        // 创建工具调用
        let tool_calls = vec![
            crate::agent::types::ToolCall {
                id: "call1".to_string(),
                name: "test_tool_1".to_string(),
                arguments: HashMap::new(),
            },
            crate::agent::types::ToolCall {
                id: "call2".to_string(),
                name: "test_tool_2".to_string(),
                arguments: HashMap::new(),
            },
        ];

        // 执行工具调用（应该使用顺序执行）
        let results = generator.execute_tool_calls(&tool_calls).await.unwrap();

        // 验证结果
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "test_tool_1");
        assert_eq!(
            results[0].status,
            crate::agent::types::ToolResultStatus::Success
        );
        assert_eq!(results[1].name, "test_tool_2");
        assert_eq!(
            results[1].status,
            crate::agent::types::ToolResultStatus::Success
        );
    }

    #[tokio::test]
    async fn test_execute_tool_calls_error_handling() {
        // 测试工具执行错误处理
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();

        // 创建一个会失败的工具
        let failing_tool = GenericTool::new(
            "failing_tool",
            "A tool that always fails",
            ToolSchema::default(),
            |_params: Value, _context: ToolExecutionContext| -> Result<Value> {
                Err(crate::error::Error::Internal(
                    "Tool execution failed".to_string(),
                ))
            },
        );

        executor.add_tool(Box::new(failing_tool)).unwrap();

        let generator = AgentGenerator::new(executor);

        let tool_calls = vec![crate::agent::types::ToolCall {
            id: "call1".to_string(),
            name: "failing_tool".to_string(),
            arguments: HashMap::new(),
        }];

        // 执行工具调用（应该捕获错误并返回错误状态）
        let results = generator.execute_tool_calls(&tool_calls).await.unwrap();

        // 验证错误处理
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "failing_tool");
        assert_eq!(
            results[0].status,
            crate::agent::types::ToolResultStatus::Error
        );
        assert!(results[0].result.get("error").is_some());
    }

    #[tokio::test]
    async fn test_agent_generator_input_validation() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();
        let generator = AgentGenerator::new(executor);

        // 测试空消息验证
        let empty_messages = vec![];
        let options = AgentGenerateOptions::default();
        let result = generator.generate(&empty_messages, &options).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // 测试空工具调用（应该返回空列表，不报错）
        let empty_tool_calls = vec![];
        let result = generator.execute_tool_calls(&empty_tool_calls).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
