//! Agent Trait 拆分
//!
//! 将 Agent Trait 拆分为多个职责单一的 Trait，提高代码的可维护性和可扩展性

use crate::agent::types::{AgentGenerateOptions, AgentGenerateResult, AgentStreamOptions, RuntimeContext};
use crate::base::Base;
use crate::error::Result;
use crate::llm::{LlmProvider, Message};
use crate::memory::Memory;
use crate::memory::thread::{CreateThreadParams, MemoryThread, ThreadStats, UpdateThreadParams};
use crate::tool::Tool;
use async_trait::async_trait;
use futures::stream::BoxStream;
use std::collections::HashMap;
use std::sync::Arc;

/// 核心 Agent Trait（精简版）
///
/// 这是 Agent 的基础 Trait，只包含最核心的功能：
/// - 名称和标识
/// - LLM Provider 访问
/// - 基础生成功能
///
/// 其他功能通过组合其他 Trait 来实现。
///
/// # Examples
///
/// ```rust
/// use lumosai_core::agent::traits::CoreAgent;
/// use lumosai_core::llm::{LlmProvider, Message, Role};
/// use lumosai_core::agent::types::AgentGenerateOptions;
///
/// # async fn example(agent: &dyn CoreAgent) -> lumosai_core::Result<()> {
/// let name = agent.get_name();
/// let llm = agent.get_llm();
/// let messages = vec![Message {
///     role: Role::User,
///     content: "Hello".to_string(),
///     metadata: None,
///     name: None,
/// }];
/// let result = agent.generate(&messages, &AgentGenerateOptions::default()).await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait CoreAgent: Base + Send + Sync {
    /// 获取 Agent 名称
    fn get_name(&self) -> &str;

    /// 获取 LLM Provider
    fn get_llm(&self) -> Arc<dyn LlmProvider>;

    /// 生成响应
    ///
    /// 这是 Agent 最核心的功能：根据消息生成响应。
    ///
    /// # Arguments
    ///
    /// * `messages` - 对话消息列表
    /// * `options` - 生成选项
    ///
    /// # Returns
    ///
    /// 生成结果，包含响应文本和元数据
    async fn generate(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
    ) -> Result<AgentGenerateResult>;
}

/// Memory Agent Trait
///
/// 为 Agent 添加内存功能，支持对话历史的存储和检索。
///
/// # Examples
///
/// ```rust
/// use lumosai_core::agent::traits::{CoreAgent, MemoryAgent};
/// use lumosai_core::llm::{Message, Role};
/// use lumosai_core::agent::types::AgentGenerateOptions;
///
/// # async fn example(agent: &dyn MemoryAgent) -> lumosai_core::Result<()> {
/// // 检查是否有内存
/// if let Some(memory) = agent.get_memory() {
///     // 使用内存生成响应
///     let result = agent.generate_with_memory(
///         &[],
///         Some("thread-123".to_string()),
///         &AgentGenerateOptions::default(),
///     ).await?;
/// }
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait MemoryAgent: CoreAgent {
    /// 获取内存实例
    ///
    /// # Returns
    ///
    /// - `Some(memory)` 如果 Agent 配置了内存
    /// - `None` 如果 Agent 没有内存
    fn get_memory(&self) -> Option<Arc<dyn Memory>>;

    /// 使用内存生成响应
    ///
    /// 这个方法会自动从内存中检索相关上下文，并将其添加到消息列表中。
    ///
    /// # Arguments
    ///
    /// * `messages` - 当前消息列表
    /// * `thread_id` - 线程 ID（可选），用于隔离不同的对话
    /// * `options` - 生成选项
    ///
    /// # Returns
    ///
    /// 生成结果，包含响应文本和元数据
    async fn generate_with_memory(
        &self,
        messages: &[Message],
        thread_id: Option<String>,
        options: &AgentGenerateOptions,
    ) -> Result<AgentGenerateResult> {
        use crate::memory::MemoryConfig;
        use crate::llm::Role;

        // 默认实现：如果有内存，从内存中检索上下文
        let mut input_messages = messages.to_vec();
        if let Some(memory) = self.get_memory() {
            // 构建内存配置
            let mut memory_config = options.memory_options.clone().unwrap_or_default();
            
            // 如果有 thread_id，使用它作为 namespace
            if let Some(tid) = thread_id {
                if memory_config.namespace.is_none() {
                    memory_config.namespace = Some(tid.clone());
                }
            }
            
            // 设置检索数量（如果没有指定，使用 context_window 或默认值）
            if memory_config.last_messages.is_none() || memory_config.last_messages == Some(0) {
                memory_config.last_messages = options.context_window.or(Some(10));
            }
            
            // 提取用户的最后一条消息作为语义搜索 query（如果启用语义召回）
            let user_query = messages
                .iter()
                .rev()
                .find(|m| matches!(m.role, Role::User))
                .map(|m| m.content.clone());
            
            if user_query.is_some() && memory_config.query.is_none() {
                memory_config.query = user_query;
            }
            
            // 从内存检索历史消息
            if let Ok(historical) = memory.retrieve(&memory_config).await {
                if !historical.is_empty() {
                    // 将历史消息添加到输入前面（历史消息按时间顺序，最新的在最后）
                    input_messages = historical.into_iter().chain(input_messages).collect();
                }
            }
        }
        
        // 使用合并后的消息调用基础 generate 方法
        self.generate(&input_messages, options).await
            }
        }
        
/// Thread Management Agent Trait
///
/// 为 Agent 添加线程管理功能，支持创建、获取、更新、删除线程等操作。
/// 这个 trait 扩展了 MemoryAgent，要求 Agent 必须配置了 Memory 才能使用线程管理功能。
///
/// # Examples
///
/// ```rust
/// use lumosai_core::agent::traits::{CoreAgent, MemoryAgent, ThreadManagementAgent};
/// use lumosai_core::memory::thread::CreateThreadParams;
///
/// # async fn example(agent: &dyn ThreadManagementAgent) -> lumosai_core::Result<()> {
/// // 创建新线程
/// let thread = agent.create_thread(CreateThreadParams {
///     id: None,
///     title: "My Conversation".to_string(),
///     agent_id: Some(agent.get_name().to_string()),
///     resource_id: Some("user-123".to_string()),
///     metadata: None,
/// }).await?;
///
/// // 获取线程
/// let retrieved = agent.get_thread("thread-123", Some("user-123")).await?;
///
/// // 列出用户的所有线程
/// let threads = agent.list_threads("user-123").await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait ThreadManagementAgent: MemoryAgent {
    /// 创建新线程
    ///
    /// # 参数
    ///
    /// * `params` - 线程创建参数
    ///
    /// # 错误
    ///
    /// 如果 Agent 没有配置 Memory，或者 Memory 不支持线程存储，返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use lumosai_core::agent::traits::ThreadManagementAgent;
    /// use lumosai_core::memory::thread::CreateThreadParams;
    ///
    /// # async fn example(agent: &dyn ThreadManagementAgent) -> lumosai_core::Result<()> {
    /// let thread = agent.create_thread(CreateThreadParams {
    ///     id: Some("thread-123".to_string()),
    ///     title: "New Conversation".to_string(),
    ///     agent_id: Some(agent.get_name().to_string()),
    ///     resource_id: Some("user-123".to_string()),
    ///     metadata: None,
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn create_thread(&self, params: CreateThreadParams) -> Result<MemoryThread> {
        let memory = self
            .get_memory()
            .ok_or_else(|| {
                crate::error::Error::Configuration(
                    "Agent does not have memory configured. Cannot create thread.".to_string(),
                )
            })?;
        memory.create_thread(params).await
    }

    /// 获取线程信息
    ///
    /// # 参数
    ///
    /// * `thread_id` - 线程ID
    /// * `resource_id` - 可选的资源ID，用于所有权验证
    ///
    /// # 示例
    ///
    /// ```rust
    /// use lumosai_core::agent::traits::ThreadManagementAgent;
    ///
    /// # async fn example(agent: &dyn ThreadManagementAgent) -> lumosai_core::Result<()> {
    /// if let Some(thread) = agent.get_thread("thread-123", Some("user-123")).await? {
    ///     println!("Thread title: {}", thread.title);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn get_thread(
        &self,
        thread_id: &str,
        resource_id: Option<&str>,
    ) -> Result<Option<MemoryThread>> {
        let memory = self
            .get_memory()
            .ok_or_else(|| {
                crate::error::Error::Configuration(
                    "Agent does not have memory configured. Cannot get thread.".to_string(),
                )
            })?;
        memory.get_thread(thread_id, resource_id).await
    }

    /// 更新线程
    ///
    /// # 参数
    ///
    /// * `thread_id` - 线程ID
    /// * `params` - 更新参数
    /// * `resource_id` - 可选的资源ID，用于所有权验证
    ///
    /// # 示例
    ///
    /// ```rust
    /// use lumosai_core::agent::traits::ThreadManagementAgent;
    /// use lumosai_core::memory::thread::UpdateThreadParams;
    ///
    /// # async fn example(agent: &dyn ThreadManagementAgent) -> lumosai_core::Result<()> {
    /// let updated = agent.update_thread(
    ///     "thread-123",
    ///     UpdateThreadParams {
    ///         title: Some("Updated Title".to_string()),
    ///         metadata: None,
    ///     },
    ///     Some("user-123"),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn update_thread(
        &self,
        thread_id: &str,
        params: UpdateThreadParams,
        resource_id: Option<&str>,
    ) -> Result<MemoryThread> {
        let memory = self
            .get_memory()
            .ok_or_else(|| {
                crate::error::Error::Configuration(
                    "Agent does not have memory configured. Cannot update thread.".to_string(),
                )
            })?;
        memory.update_thread(thread_id, params, resource_id).await
    }

    /// 删除线程
    ///
    /// # 参数
    ///
    /// * `thread_id` - 线程ID
    /// * `resource_id` - 可选的资源ID，用于所有权验证
    ///
    /// # 示例
    ///
    /// ```rust
    /// use lumosai_core::agent::traits::ThreadManagementAgent;
    ///
    /// # async fn example(agent: &dyn ThreadManagementAgent) -> lumosai_core::Result<()> {
    /// agent.delete_thread("thread-123", Some("user-123")).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn delete_thread(&self, thread_id: &str, resource_id: Option<&str>) -> Result<()> {
        let memory = self
            .get_memory()
            .ok_or_else(|| {
                crate::error::Error::Configuration(
                    "Agent does not have memory configured. Cannot delete thread.".to_string(),
                )
            })?;
        memory.delete_thread(thread_id, resource_id).await
    }

    /// 列出资源的所有线程
    ///
    /// # 参数
    ///
    /// * `resource_id` - 资源ID
    ///
    /// # 示例
    ///
    /// ```rust
    /// use lumosai_core::agent::traits::ThreadManagementAgent;
    ///
    /// # async fn example(agent: &dyn ThreadManagementAgent) -> lumosai_core::Result<()> {
    /// let threads = agent.list_threads("user-123").await?;
    /// println!("Found {} threads", threads.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn list_threads(&self, resource_id: &str) -> Result<Vec<MemoryThread>> {
        let memory = self
            .get_memory()
            .ok_or_else(|| {
                crate::error::Error::Configuration(
                    "Agent does not have memory configured. Cannot list threads.".to_string(),
                )
            })?;
        memory.list_threads(resource_id).await
    }

    /// 获取线程统计信息
    ///
    /// # 参数
    ///
    /// * `thread_id` - 线程ID
    /// * `resource_id` - 可选的资源ID，用于所有权验证
    ///
    /// # 示例
    ///
    /// ```rust
    /// use lumosai_core::agent::traits::ThreadManagementAgent;
    ///
    /// # async fn example(agent: &dyn ThreadManagementAgent) -> lumosai_core::Result<()> {
    /// let stats = agent.get_thread_stats("thread-123", Some("user-123")).await?;
    /// println!("Thread has {} messages", stats.message_count);
    /// # Ok(())
    /// # }
    /// ```
    async fn get_thread_stats(
        &self,
        thread_id: &str,
        resource_id: Option<&str>,
    ) -> Result<ThreadStats> {
        let memory = self
            .get_memory()
            .ok_or_else(|| {
                crate::error::Error::Configuration(
                    "Agent does not have memory configured. Cannot get thread stats.".to_string(),
                )
            })?;
        memory.get_thread_stats(thread_id, resource_id).await
    }
}

/// Tool Agent Trait
///
/// 为 Agent 添加工具调用功能，允许 Agent 使用外部工具来扩展能力。
///
/// # Examples
///
/// ```rust
/// use lumosai_core::agent::traits::{CoreAgent, ToolAgent};
/// use lumosai_core::agent::types::RuntimeContext;
///
/// # async fn example(agent: &dyn ToolAgent) -> lumosai_core::Result<()> {
/// // 获取所有工具
/// let tools = agent.get_tools();
/// println!("Agent has {} tools", tools.len());
///
/// // 根据上下文获取工具
/// let context = RuntimeContext::default();
/// let context_tools = agent.get_tools_with_context(&context).await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait ToolAgent: CoreAgent {
    /// 获取所有可用工具
    ///
    /// # Returns
    ///
    /// 工具名称到工具实现的映射
    fn get_tools(&self) -> HashMap<String, Box<dyn Tool>>;

    /// 根据运行时上下文获取工具
    ///
    /// 这个方法允许根据上下文动态选择工具，支持更灵活的工具管理。
    ///
    /// # Arguments
    ///
    /// * `context` - 运行时上下文
    ///
    /// # Returns
    ///
    /// 在当前上下文中可用的工具映射
    ///
    /// # Default Implementation
    ///
    /// 默认实现返回所有静态工具（通过 `get_tools()`）。
        async fn get_tools_with_context(
        &self,
        _context: &RuntimeContext,
    ) -> Result<HashMap<String, Box<dyn Tool>>> {
        // 默认实现返回所有静态工具
        Ok(self.get_tools())
    }
}

/// Streaming Agent Trait
///
/// 为 Agent 添加流式响应功能，支持实时生成和返回文本。
///
/// 注意：这个 trait 与 `streaming::StreamingAgent` 不同，它是拆分后的精简版本。
///
/// # Examples
///
/// ```rust
/// use lumosai_core::agent::traits::{CoreAgent, StreamingAgentTrait};
/// use lumosai_core::llm::{Message, Role};
/// use lumosai_core::agent::types::AgentStreamOptions;
/// use futures::StreamExt;
///
/// # async fn example(agent: &dyn StreamingAgentTrait) -> lumosai_core::Result<()> {
/// let messages = vec![Message {
///     role: Role::User,
///     content: "Tell me a story".to_string(),
///     metadata: None,
///     name: None,
/// }];
///
/// let mut stream = agent.stream(&messages, &AgentStreamOptions::default()).await?;
/// while let Some(chunk) = stream.next().await {
///     match chunk {
///         Ok(text) => print!("{}", text),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait StreamingAgentTrait: CoreAgent {
    /// 流式生成响应
    ///
    /// 这个方法返回一个流，可以实时接收生成的文本片段。
    ///
    /// # Arguments
    ///
    /// * `messages` - 对话消息列表
    /// * `options` - 流式生成选项
    ///
    /// # Returns
    ///
    /// 一个流，每个元素是一个文本片段或错误
    async fn stream(
        &self,
        messages: &[Message],
        options: &AgentStreamOptions,
    ) -> Result<BoxStream<'_, Result<String>>>;
}

/// 组合 Agent Trait
///
/// 这个 Trait 组合了所有功能，用于需要完整功能的 Agent。
/// 它同时实现了 CoreAgent、MemoryAgent、ToolAgent 和 StreamingAgentTrait。
///
/// # Examples
///
/// ```rust
/// use lumosai_core::agent::traits::FullAgent;
///
/// # fn example(agent: &dyn FullAgent) {
/// // FullAgent 可以使用所有功能
/// let name = agent.get_name();
/// let llm = agent.get_llm();
/// let memory = agent.get_memory();
/// let tools = agent.get_tools();
/// # }
/// ```
pub trait FullAgent: CoreAgent + MemoryAgent + ToolAgent + StreamingAgentTrait {}

// 为所有实现了所有 Trait 的类型自动实现 FullAgent
impl<T> FullAgent for T where T: CoreAgent + MemoryAgent + ToolAgent + StreamingAgentTrait {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::AgentGenerateOptions;
    use crate::llm::{Message, Role};
    use crate::llm::MockLlmProvider;
    use std::sync::Arc;

    // Mock implementation for testing
    struct MockCoreAgent {
        name: String,
        llm: Arc<dyn LlmProvider>,
    }

    #[async_trait]
    impl CoreAgent for MockCoreAgent {
        fn get_name(&self) -> &str {
            &self.name
        }

        fn get_llm(&self) -> Arc<dyn LlmProvider> {
            self.llm.clone()
        }

        async fn generate(
            &self,
            messages: &[Message],
            _options: &AgentGenerateOptions,
        ) -> Result<AgentGenerateResult> {
            // 简单的 mock 实现
            let response = self.llm.generate_with_messages(messages, &Default::default()).await?;
            Ok(AgentGenerateResult {
                response,
                steps: vec![],
                usage: crate::agent::types::TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                },
                metadata: Default::default(),
            })
        }
    }

    impl Base for MockCoreAgent {
        fn name(&self) -> Option<&str> {
            Some(&self.name)
        }

        fn component(&self) -> crate::compat::Component {
            crate::compat::Component::Agent
        }

        fn logger(&self) -> Arc<dyn crate::logger::Logger> {
            crate::logger::default_logger()
        }

        fn set_logger(&mut self, _logger: Arc<dyn crate::logger::Logger>) {
            // Mock implementation
        }

        fn telemetry(&self) -> Option<Arc<dyn crate::telemetry::TelemetrySink>> {
            None
        }

        fn set_telemetry(&mut self, _telemetry: Arc<dyn crate::telemetry::TelemetrySink>) {
            // Mock implementation
        }
    }

    #[tokio::test]
    async fn test_core_agent() {
        let llm = Arc::new(MockLlmProvider::new(vec!["test response".to_string()]));
        let agent = MockCoreAgent {
            name: "test_agent".to_string(),
            llm,
        };

        assert_eq!(agent.get_name(), "test_agent");
        
        let messages = vec![Message {
            role: Role::User,
            content: "Hello".to_string(),
            metadata: None,
            name: None,
        }];

        let result = agent.generate(&messages, &AgentGenerateOptions::default()).await.unwrap();
        assert_eq!(result.response, "test response");
    }

    #[tokio::test]
    async fn test_memory_agent_default_implementation() {
        struct MockMemoryAgent {
            core: MockCoreAgent,
        }

        #[async_trait]
        impl CoreAgent for MockMemoryAgent {
            fn get_name(&self) -> &str {
                self.core.get_name()
            }

            fn get_llm(&self) -> Arc<dyn LlmProvider> {
                self.core.get_llm()
            }

            async fn generate(
                &self,
                messages: &[Message],
                options: &AgentGenerateOptions,
            ) -> Result<AgentGenerateResult> {
                self.core.generate(messages, options).await
            }
        }

        impl Base for MockMemoryAgent {
            fn name(&self) -> Option<&str> {
                self.core.name()
            }

            fn component(&self) -> crate::compat::Component {
                self.core.component()
            }

            fn logger(&self) -> Arc<dyn crate::logger::Logger> {
                self.core.logger()
            }

            fn set_logger(&mut self, _logger: Arc<dyn crate::logger::Logger>) {
                // Cannot mutate through shared reference, but this is just for testing
            }

            fn telemetry(&self) -> Option<Arc<dyn crate::telemetry::TelemetrySink>> {
                self.core.telemetry()
            }

            fn set_telemetry(&mut self, _telemetry: Arc<dyn crate::telemetry::TelemetrySink>) {
                // Cannot mutate through shared reference, but this is just for testing
            }
        }

        #[async_trait]
        impl MemoryAgent for MockMemoryAgent {
            fn get_memory(&self) -> Option<Arc<dyn Memory>> {
                None // 没有内存
            }
        }

        let llm = Arc::new(MockLlmProvider::new(vec!["test response".to_string()]));
        let agent = MockMemoryAgent {
            core: MockCoreAgent {
                name: "test_memory_agent".to_string(),
                llm,
            },
        };

        let messages = vec![Message {
            role: Role::User,
            content: "Hello".to_string(),
            metadata: None,
            name: None,
        }];

        // 测试默认实现
        let result = agent
            .generate_with_memory(&messages, None, &AgentGenerateOptions::default())
            .await
            .unwrap();
        assert_eq!(result.response, "test response");
    }

    #[tokio::test]
    async fn test_tool_agent_default_implementation() {
        struct MockToolAgent {
            core: MockCoreAgent,
        }

        #[async_trait]
        impl CoreAgent for MockToolAgent {
            fn get_name(&self) -> &str {
                self.core.get_name()
            }

            fn get_llm(&self) -> Arc<dyn LlmProvider> {
                self.core.get_llm()
            }

            async fn generate(
                &self,
                messages: &[Message],
                options: &AgentGenerateOptions,
            ) -> Result<AgentGenerateResult> {
                self.core.generate(messages, options).await
            }
        }

        impl Base for MockToolAgent {
            fn name(&self) -> Option<&str> {
                self.core.name()
            }

            fn component(&self) -> crate::compat::Component {
                self.core.component()
            }

            fn logger(&self) -> Arc<dyn crate::logger::Logger> {
                self.core.logger()
            }

            fn set_logger(&mut self, _logger: Arc<dyn crate::logger::Logger>) {
                // Cannot mutate through shared reference, but this is just for testing
            }

            fn telemetry(&self) -> Option<Arc<dyn crate::telemetry::TelemetrySink>> {
                self.core.telemetry()
            }

            fn set_telemetry(&mut self, _telemetry: Arc<dyn crate::telemetry::TelemetrySink>) {
                // Cannot mutate through shared reference, but this is just for testing
            }
        }

        #[async_trait]
        impl ToolAgent for MockToolAgent {
            fn get_tools(&self) -> HashMap<String, Box<dyn Tool>> {
                HashMap::new()
            }
        }

        let llm = Arc::new(MockLlmProvider::new(vec!["test response".to_string()]));
        let agent = MockToolAgent {
            core: MockCoreAgent {
                name: "test_tool_agent".to_string(),
                llm,
            },
        };

        // 测试默认实现
        let tools = agent.get_tools();
        assert_eq!(tools.len(), 0);

        let context = RuntimeContext::default();
        let context_tools = agent.get_tools_with_context(&context).await.unwrap();
        assert_eq!(context_tools.len(), 0);
    }
}

