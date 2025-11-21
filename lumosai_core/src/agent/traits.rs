//! Agent Trait 拆分
//!
//! 将 Agent Trait 拆分为多个职责单一的 Trait，提高代码的可维护性和可扩展性

use crate::agent::types::{AgentGenerateOptions, AgentGenerateResult, AgentStreamOptions, RuntimeContext};
use crate::base::Base;
use crate::error::Result;
use crate::llm::{LlmProvider, Message};
use crate::memory::Memory;
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
        // 默认实现：如果有内存，从内存中检索上下文
        if let Some(_memory) = self.get_memory() {
            // 如果有 thread_id，从该线程检索历史消息
            if let Some(_thread_id) = thread_id {
                // TODO: 实现从内存检索历史消息的逻辑
                // 这里先使用基础 generate 方法
            }
        }
        
        // 回退到基础 generate 方法
        self.generate(messages, options).await
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
            fn id(&self) -> &str {
                self.core.id()
            }

            fn name(&self) -> Option<&str> {
                self.core.name()
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
            fn id(&self) -> &str {
                self.core.id()
            }

            fn name(&self) -> Option<&str> {
                self.core.name()
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

