//! Simplified API for Agent creation (plan4.md implementation)
//! 
//! This module provides the simplified API as specified in plan4.md Phase 1,
//! offering Mastra-level simplicity while maintaining Rust performance.

use super::AgentBuilder;

/// Simplified Agent struct for plan4.md API
/// 
/// This provides the clean API interface as specified in the plan:
/// ```rust
/// let agent = Agent::quick("assistant", "你是一个AI助手")
///     .model(deepseek("deepseek-chat"))
///     .tools([web_search(), calculator()])
///     .build()?;
/// ```
pub struct Agent;

impl Agent {
    /// Create a quick agent with minimal configuration (plan4.md API)
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use lumosai_core::agent::Agent;
    /// use lumosai_core::llm::MockLlmProvider;
    /// use std::sync::Arc;
    /// 
    /// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    /// 
    /// let agent = Agent::quick("assistant", "You are a helpful assistant")
    ///     .model(llm)
    ///     .build()
    ///     .expect("Failed to create agent");
    /// ```
    pub fn quick(name: &str, instructions: &str) -> AgentBuilder {
        AgentBuilder::new()
            .name(name)
            .instructions(instructions)
            .enable_smart_defaults()
    }

    /// Create an agent with the full builder pattern (plan4.md API)
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use lumosai_core::agent::Agent;
    /// use lumosai_core::llm::MockLlmProvider;
    /// use std::sync::Arc;
    /// 
    /// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    /// 
    /// let agent = Agent::builder()
    ///     .name("research_agent")
    ///     .instructions("专业研究助手")
    ///     .model(llm)
    ///     .max_tool_calls(10)
    ///     .build()
    ///     .expect("Failed to create agent");
    /// ```
    pub fn builder() -> AgentBuilder {
        AgentBuilder::new()
    }
}

/// Create a quick agent with minimal configuration (plan4.md convenience function)
/// 
/// This is the most convenient way to create an agent:
/// ```rust
/// let agent = quick("assistant", "You are helpful")
///     .model(llm)
///     .build()?;
/// ```
pub fn quick(name: &str, instructions: &str) -> AgentBuilder {
    Agent::quick(name, instructions)
}

/// Create a web agent with pre-configured web tools (plan4.md API)
///
/// # Example
///
/// ```rust
/// use lumosai_core::agent::web_agent;
/// use lumosai_core::llm::MockLlmProvider;
/// use std::sync::Arc;
///
/// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
///
/// let agent = web_agent("web_helper")
///     .model(llm)
///     .build()
///     .expect("Failed to create web agent");
/// ```
pub fn web_agent(name: &str) -> AgentBuilder {
    AgentBuilder::new()
        .name(name)
        .instructions("You are a web assistant with access to web browsing tools")
        .with_web_tools()
        .enable_smart_defaults()
}

/// Create a file agent with pre-configured file tools (plan4.md API)
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::agent::file_agent;
/// use lumosai_core::llm::MockLlmProvider;
/// use std::sync::Arc;
/// 
/// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
/// 
/// let agent = file_agent("file_helper")
///     .model(llm)
///     .build()
///     .expect("Failed to create file agent");
/// ```
pub fn file_agent(name: &str) -> AgentBuilder {
    AgentBuilder::new()
        .name(name)
        .instructions("You are a file assistant with access to file system tools")
        .with_file_tools()
        .enable_smart_defaults()
}

/// Create a data agent with pre-configured data processing tools (plan4.md API)
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::agent::data_agent;
/// use lumosai_core::llm::MockLlmProvider;
/// use std::sync::Arc;
/// 
/// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
/// 
/// let agent = data_agent("data_helper")
///     .model(llm)
///     .build()
///     .expect("Failed to create data agent");
/// ```
pub fn data_agent(name: &str) -> AgentBuilder {
    AgentBuilder::new()
        .name(name)
        .instructions("You are a data assistant with access to data processing tools")
        .with_data_tools()
        .with_math_tools()
        .enable_smart_defaults()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::MockLlmProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_agent_quick_api() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let agent = Agent::quick("assistant", "You are helpful")
            .model(llm)
            .build()
            .expect("Failed to create agent");

        assert_eq!(agent.get_name(), "assistant");
        assert_eq!(agent.get_instructions(), "You are helpful");
    }

    #[tokio::test]
    async fn test_agent_builder_api() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let agent = Agent::builder()
            .name("research_agent")
            .instructions("You are a research assistant")
            .model(llm)
            .max_tool_calls(10)
            .build()
            .expect("Failed to create agent");

        assert_eq!(agent.get_name(), "research_agent");
        assert_eq!(agent.get_instructions(), "You are a research assistant");
    }

    #[tokio::test]
    async fn test_quick_convenience_function() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let agent = quick("assistant", "You are helpful")
            .model(llm)
            .build()
            .expect("Failed to create agent");

        assert_eq!(agent.get_name(), "assistant");
        assert_eq!(agent.get_instructions(), "You are helpful");
    }

    #[tokio::test]
    async fn test_web_agent() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let agent = web_agent("web_helper")
            .model(llm)
            .build()
            .expect("Failed to create web agent");

        assert_eq!(agent.get_name(), "web_helper");
        assert!(agent.get_instructions().contains("web"));
        // Should have web tools
        assert!(agent.get_tools().len() > 0);
    }

    #[tokio::test]
    async fn test_file_agent() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let agent = file_agent("file_helper")
            .model(llm)
            .build()
            .expect("Failed to create file agent");

        assert_eq!(agent.get_name(), "file_helper");
        assert!(agent.get_instructions().contains("file"));
        // Should have file tools
        assert!(agent.get_tools().len() > 0);
    }

    #[tokio::test]
    async fn test_data_agent() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let agent = data_agent("data_helper")
            .model(llm)
            .build()
            .expect("Failed to create data agent");

        assert_eq!(agent.get_name(), "data_helper");
        assert!(agent.get_instructions().contains("data"));
        // Should have data and math tools
        assert!(agent.get_tools().len() > 0);
    }
}
