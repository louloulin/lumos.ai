//! 统一的 Agent API 包装器 - BasicAgent 重构第四步
//!
//! 这个模块提供了一个统一的 API 包装器，将重构后的模块（AgentCore、AgentExecutor、AgentGenerator）
//! 组合成一个易于使用的 Agent 接口，类似于 BasicAgent 但使用新的模块化架构。

use crate::agent::refactored::{AgentCore, AgentExecutor, AgentGenerator};
use crate::agent::types::{AgentGenerateOptions, AgentGenerateResult, AgentStreamOptions};
use crate::agent::AgentConfig;
use crate::error::Result;
use crate::llm::{LlmProvider, Message};
use crate::memory::Memory;
use crate::tool::Tool;
use futures::stream::{BoxStream, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;

/// 重构后的 Agent 实现
///
/// 这是 BasicAgent 重构后的统一接口，使用模块化架构：
/// - `AgentCore`: 管理核心配置和 LLM 提供者
/// - `AgentExecutor`: 管理工具和内存
/// - `AgentGenerator`: 协调生成逻辑
///
/// # 示例
///
/// ```rust
/// use lumosai_core::agent::refactored::RefactoredAgent;
/// use lumosai_core::agent::AgentConfig;
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
/// let agent = RefactoredAgent::new(config, llm)?;
///
/// let messages = vec![Message {
///     role: Role::User,
///     content: "Hello!".to_string(),
///     metadata: None,
///     name: None,
/// }];
/// let result = agent.generate(&messages, &Default::default()).await?;
/// # Ok(())
/// # }
/// ```
pub struct RefactoredAgent {
    /// Agent 生成器
    generator: AgentGenerator,
}

impl RefactoredAgent {
    /// 创建新的重构后的 Agent
    ///
    /// # 参数
    ///
    /// * `config` - Agent 配置
    /// * `llm` - LLM 提供者
    ///
    /// # 返回
    ///
    /// 返回 `Result<RefactoredAgent>`。
    pub fn new(config: AgentConfig, llm: Arc<dyn LlmProvider>) -> Result<Self> {
        let core = AgentCore::new(config, llm)?;
        let executor = AgentExecutor::new(core)?;
        let generator = AgentGenerator::new(executor);

        Ok(Self { generator })
    }

    /// 使用内存创建 Agent
    ///
    /// # 参数
    ///
    /// * `config` - Agent 配置
    /// * `llm` - LLM 提供者
    /// * `memory` - 内存实例
    ///
    /// # 返回
    ///
    /// 返回 `Result<RefactoredAgent>`。
    pub fn with_memory(
        config: AgentConfig,
        llm: Arc<dyn LlmProvider>,
        memory: Arc<dyn Memory>,
    ) -> Result<Self> {
        let core = AgentCore::new(config, llm)?;
        let executor = AgentExecutor::new(core)?.with_memory(memory);
        let generator = AgentGenerator::new(executor);

        Ok(Self { generator })
    }

    /// 生成响应
    ///
    /// # 参数
    ///
    /// * `messages` - 输入消息列表
    /// * `options` - 生成选项
    ///
    /// # 返回
    ///
    /// 返回 `Result<AgentGenerateResult>`，包含生成的响应和元数据。
    pub async fn generate(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
    ) -> Result<AgentGenerateResult> {
        self.generator.generate(messages, options).await
    }

    /// 流式生成响应
    ///
    /// # 参数
    ///
    /// * `messages` - 输入消息列表
    /// * `options` - 流式生成选项
    ///
    /// # 返回
    ///
    /// 返回一个流，每个元素是一个字符串块。
    pub async fn stream<'a>(
        &'a self,
        messages: &'a [Message],
        options: &'a AgentStreamOptions,
    ) -> Result<BoxStream<'a, Result<String>>> {
        self.generator.stream(messages, options).await
    }

    /// 添加工具
    ///
    /// # 参数
    ///
    /// * `tool` - 要添加的工具
    ///
    /// # 返回
    ///
    /// 返回 `Result<()>`。
    pub fn add_tool(&mut self, tool: Box<dyn Tool>) -> Result<()> {
        // 注意：这需要修改 AgentGenerator 以支持可变访问
        // 当前实现中，我们需要通过 executor 来添加工具
        // 这是一个设计限制，未来可以改进
        Err(crate::error::Error::UnsupportedOperation(
            "add_tool is not yet supported in RefactoredAgent. Use AgentExecutor directly.".to_string(),
        ))
    }

    /// 获取 Agent 名称
    pub fn name(&self) -> &str {
        self.generator.executor().core().name()
    }

    /// 获取 Agent 指令
    pub fn instructions(&self) -> &str {
        self.generator.executor().core().instructions()
    }

    /// 获取 LLM 提供者
    pub fn llm(&self) -> Arc<dyn LlmProvider> {
        self.generator.executor().core().llm().clone()
    }

    /// 获取内存（如果已配置）
    pub fn memory(&self) -> Option<Arc<dyn Memory>> {
        self.generator.executor().memory()
    }

    /// 检查是否有内存
    pub fn has_memory(&self) -> bool {
        self.generator.executor().memory().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentConfig;
    use crate::llm::MockLlmProvider;

    #[tokio::test]
    async fn test_refactored_agent_creation() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let agent = RefactoredAgent::new(config, llm).unwrap();
        assert_eq!(agent.name(), "test-agent");
        assert_eq!(agent.instructions(), "You are a helpful assistant.");
        assert!(!agent.has_memory());
    }

    #[tokio::test]
    async fn test_refactored_agent_generate() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello! How can I help you?".to_string()]));

        let agent = RefactoredAgent::new(config, llm).unwrap();

        let messages = vec![Message {
            role: crate::llm::Role::User,
            content: "Hello!".to_string(),
            metadata: None,
            name: None,
        }];
        let options = AgentGenerateOptions::default();
        let result = agent.generate(&messages, &options).await.unwrap();
        assert!(!result.response.is_empty());
    }

    #[tokio::test]
    async fn test_refactored_agent_stream() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello! How can I help you? This is a longer response.".to_string()]));

        let agent = RefactoredAgent::new(config, llm).unwrap();

        let messages = vec![Message {
            role: crate::llm::Role::User,
            content: "Hello!".to_string(),
            metadata: None,
            name: None,
        }];
        let options = AgentStreamOptions::default();
        let mut stream = agent.stream(&messages, &options).await.unwrap();

        let mut chunks = Vec::new();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.unwrap();
            chunks.push(chunk);
        }

        assert!(!chunks.is_empty());
        let full_response: String = chunks.join("");
        assert!(!full_response.is_empty());
    }
}

