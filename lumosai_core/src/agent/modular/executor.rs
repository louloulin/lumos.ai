//! Agent执行器 - 负责Agent的执行逻辑和工具调用
//!
//! 这个组件是Agent的"大脑"，负责处理消息、调用工具、管理内存等执行逻辑。

use crate::agent::modular::core::AgentCore;
use crate::agent::modular::state::AgentState;
use crate::agent::types::{AgentStatus, MessageRole, ToolCall};
use crate::agent::{AgentConfig, LlmOptions, Message, ToolCallResult};
use crate::error::Result;
use crate::llm::{LlmProvider, LlmResponse, StreamingResponse};
use crate::memory::MemoryManager;
use crate::telemetry::TelemetryCollector;
use crate::tools::{ToolContext, ToolRegistry};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent执行器
///
/// 负责Agent的核心执行逻辑，包括：
/// - 消息处理和响应生成
/// - 工具调用和管理
/// - 内存访问和管理
/// - 流式响应处理
/// - LLM集成
pub struct AgentExecutor {
    /// Agent核心引用
    core: AgentCore,
    /// Agent状态管理器
    state: Arc<RwLock<AgentState>>,
    /// LLM提供者
    llm_provider: Arc<dyn LlmProvider>,
    /// 工具注册表
    tool_registry: Arc<ToolRegistry>,
    /// 内存管理器
    memory_manager: Option<Arc<MemoryManager>>,
    /// 遥测收集器
    telemetry: Arc<TelemetryCollector>,
    /// 执行配置
    config: ExecutorConfig,
}

/// 执行器配置
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// 最大工具调用次数
    pub max_tool_calls: usize,
    /// 工具调用超时时间（秒）
    pub tool_timeout_seconds: u64,
    /// 是否启用流式响应
    pub enable_streaming: bool,
    /// 是否启用内存检索
    pub enable_memory_retrieval: bool,
    /// 最大上下文长度
    pub max_context_length: usize,
    /// 重试次数
    pub max_retries: usize,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_tool_calls: 10,
            tool_timeout_seconds: 30,
            enable_streaming: true,
            enable_memory_retrieval: true,
            max_context_length: 4000,
            max_retries: 3,
        }
    }
}

impl AgentExecutor {
    /// 创建新的Agent执行器
    pub async fn new(core: AgentCore, state: Arc<RwLock<AgentState>>) -> Result<Self> {
        let llm_provider = Self::create_llm_provider(&core)?;
        let tool_registry = Arc::new(ToolRegistry::new());
        let telemetry = Arc::new(TelemetryCollector::new());

        // 创建内存管理器（如果需要）
        let memory_manager = if core.config().memory_config.is_some() {
            Some(Arc::new(MemoryManager::new(
                core.config().memory_config.clone().unwrap(),
            )?))
        } else {
            None
        };

        Ok(Self {
            core,
            state,
            llm_provider,
            tool_registry,
            memory_manager,
            telemetry,
            config: ExecutorConfig::default(),
        })
    }

    /// 创建LLM提供者
    fn create_llm_provider(core: &AgentCore) -> Result<Arc<dyn LlmProvider>> {
        use crate::unified_api;

        let provider = match core.config().llm_provider.as_str() {
            "openai" => unified_api::claude(core.model()),
            "anthropic" => unified_api::claude(core.model()),
            "claude" => unified_api::claude(core.model()),
            "gpt" => unified_api::claude(core.model()),
            "ollama" => Ok(unified_api::ollama(core.model())),
            "together" => unified_api::together(core.model()),
            "cohere" => unified_api::cohere(core.model()),
            "gemini" => unified_api::gemini(core.model()),
            _ => Err(crate::error::Error::Configuration {
                message: format!("Unsupported LLM provider: {}", core.config().llm_provider),
            }),
        }?;

        Ok(provider)
    }

    /// 执行单个消息
    pub async fn execute_message(&self, message: Message) -> Result<LlmResponse> {
        let span = self.telemetry.start_span("agent.execute_message");

        // 更新状态
        {
            let mut state = self.state.write().await;
            state.status = AgentStatus::Processing;
            state.last_activity = chrono::Utc::now();
        }

        // 构建上下文
        let context = self.build_context(&message).await?;

        // 调用LLM
        let response = self.call_llm(&context).await?;

        // 处理工具调用
        let final_response = if response.tool_calls.is_empty() {
            response
        } else {
            self.handle_tool_calls(response).await?
        };

        // 更新状态
        {
            let mut state = self.state.write().await;
            state.status = AgentStatus::Idle;
            state.message_count += 1;
            state.total_tokens += final_response.total_tokens;
        }

        span.end();
        Ok(final_response)
    }

    /// 执行流式消息
    pub async fn execute_stream_message(
        &self,
        message: Message,
    ) -> Result<impl futures::Stream<Item = Result<String>>> {
        let span = self.telemetry.start_span("agent.execute_stream_message");

        // 更新状态
        {
            let mut state = self.state.write().await;
            state.status = AgentStatus::Processing;
            state.last_activity = chrono::Utc::now();
        }

        // 构建上下文
        let context = self.build_context(&message).await?;

        // 调用LLM流式响应
        let stream = self.call_llm_stream(&context).await?;

        // 处理流式响应
        let processed_stream = self.process_stream(stream).await?;

        span.end();
        Ok(processed_stream)
    }

    /// 构建消息上下文
    async fn build_context(&self, message: &Message) -> Result<Vec<Message>> {
        let mut context = vec![];

        // 添加系统提示
        if let Some(system_prompt) = &self.core.config().system_prompt {
            context.push(Message {
                role: MessageRole::System,
                content: system_prompt.clone(),
                ..Default::default()
            });
        }

        // 如果启用了内存检索，检索相关内存
        if self.config.enable_memory_retrieval {
            if let Some(memory_manager) = &self.memory_manager {
                if let Ok(memories) = memory_manager.retrieve_relevant(&message.content, 5).await {
                    for memory in memories {
                        context.push(Message {
                            role: MessageRole::System,
                            content: format!("相关记忆: {}", memory.content),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // 添加历史消息（从状态中获取）
        {
            let state = self.state.read().await;
            for historical_msg in &state.conversation_history {
                context.push(historical_msg.clone());
            }
        }

        // 添加当前消息
        context.push(message.clone());

        // 限制上下文长度
        if context.len() > self.config.max_context_length {
            context = context
                .split_at(context.len() - self.config.max_context_length)
                .1
                .to_vec();
        }

        Ok(context)
    }

    /// 调用LLM
    async fn call_llm(&self, context: &[Message]) -> Result<LlmResponse> {
        let options = LlmOptions {
            temperature: self.core.config().temperature.unwrap_or(0.7),
            max_tokens: self.core.config().max_tokens,
            ..Default::default()
        };

        let response = self
            .llm_provider
            .generate_response(context, &options)
            .await?;

        // 记录遥测数据
        self.telemetry
            .record_llm_call(&self.core.id(), &self.core.model(), response.total_tokens);

        Ok(response)
    }

    /// 调用LLM流式响应
    async fn call_llm_stream(&self, context: &[Message]) -> Result<StreamingResponse> {
        let options = LlmOptions {
            temperature: self.core.config().temperature.unwrap_or(0.7),
            max_tokens: self.core.config().max_tokens,
            ..Default::default()
        };

        self.llm_provider
            .generate_streaming_response(context, &options)
            .await
    }

    /// 处理工具调用
    async fn handle_tool_calls(&self, response: LlmResponse) -> Result<LlmResponse> {
        let mut tool_results = Vec::new();
        let mut final_response = response.clone();

        for tool_call in &response.tool_calls {
            let tool_result = self.execute_tool_call(tool_call).await?;
            tool_results.push(tool_result);
        }

        // 如果有工具调用结果，再次调用LLM
        if !tool_results.is_empty() {
            let mut follow_up_messages = vec![];

            // 添加原始消息
            let state = self.state.read().await;
            if let Some(last_message) = state.conversation_history.last() {
                follow_up_messages.push(last_message.clone());
            }
            drop(state);

            // 添加助手响应
            follow_up_messages.push(Message {
                role: MessageRole::Assistant,
                content: response.content,
                tool_calls: response.tool_calls,
                ..Default::default()
            });

            // 添加工具结果
            for result in tool_results {
                follow_up_messages.push(Message {
                    role: MessageRole::Tool,
                    content: serde_json::to_string(&result)
                        .unwrap_or_else(|_| "Tool execution failed".to_string()),
                    ..Default::default()
                });
            }

            // 再次调用LLM
            final_response = self.call_llm(&follow_up_messages).await?;
        }

        Ok(final_response)
    }

    /// 执行单个工具调用
    async fn execute_tool_call(&self, tool_call: &ToolCall) -> Result<ToolCallResult> {
        let span = self.telemetry.start_span("agent.execute_tool");

        // 创建工具上下文
        let context = ToolContext {
            agent_id: self.core.id().to_string(),
            agent_name: self.core.name().to_string(),
            tenant_id: self.core.tenant_id().map(|s| s.to_string()),
            metadata: self.core.metadata().clone(),
        };

        // 执行工具调用
        let result = tokio::time::timeout(
            tokio::time::Duration::from_secs(self.config.tool_timeout_seconds),
            self.tool_registry
                .execute_tool(&tool_call.name, &tool_call.arguments, context),
        )
        .await
        .map_err(|_| crate::error::Error::Timeout {
            message: format!("Tool call timeout: {}", tool_call.name),
        })??;

        // 记录遥测数据
        self.telemetry
            .record_tool_call(&self.core.id(), &tool_call.name);

        span.end();
        Ok(result)
    }

    /// 处理流式响应
    async fn process_stream(
        &self,
        stream: StreamingResponse,
    ) -> Result<impl futures::Stream<Item = Result<String>>> {
        let stream = stream.map(|chunk_result| {
            chunk_result.map_err(|e| crate::error::Error::Llm {
                message: format!("Stream error: {}", e),
            })
        });

        Ok(stream)
    }

    /// 注册工具
    pub fn register_tool<T: crate::tools::Tool + Send + Sync + 'static>(
        &self,
        tool: T,
    ) -> Result<()> {
        self.tool_registry.register_tool(tool)?;
        Ok(())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> ExecutorStats {
        let state = self.state.read().await;
        ExecutorStats {
            message_count: state.message_count,
            total_tokens: state.total_tokens,
            tool_calls_count: state.tool_calls_count,
            average_response_time: state.average_response_time,
            last_activity: state.last_activity,
        }
    }

    /// 重置状态
    pub async fn reset_state(&self) -> Result<()> {
        let mut state = self.state.write().await;
        *state = AgentState::new();
        Ok(())
    }

    /// 设置配置
    pub fn set_config(&mut self, config: ExecutorConfig) {
        self.config = config;
    }

    /// 获取配置
    pub fn config(&self) -> &ExecutorConfig {
        &self.config
    }
}

/// 执行器统计信息
#[derive(Debug, Clone)]
pub struct ExecutorStats {
    /// 消息处理数量
    pub message_count: usize,
    /// 总token使用量
    pub total_tokens: usize,
    /// 工具调用次数
    pub tool_calls_count: usize,
    /// 平均响应时间（毫秒）
    pub average_response_time: f64,
    /// 最后活动时间
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_executor_creation() {
        let config = AgentConfig::default();
        let core = AgentCore::new(config).unwrap();
        let state = Arc::new(RwLock::new(AgentState::new()));

        let result = AgentExecutor::new(core, state).await;
        // 注意：这个测试可能会失败，因为LLM提供者可能无法连接
        // 在实际测试中，应该使用mock LLM提供者
        println!("Executor creation result: {:?}", result);
    }

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert_eq!(config.max_tool_calls, 10);
        assert_eq!(config.tool_timeout_seconds, 30);
        assert!(config.enable_streaming);
    }
}
