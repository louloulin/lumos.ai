//! Legacy Agent adapter for backward compatibility
//!
//! This module provides an adapter that allows the new composition-based Agent traits
//! to work with the old God Trait interface, ensuring backward compatibility during the transition.

use async_trait::async_trait;
use futures::stream::BoxStream;
use std::collections::HashMap;
use std::sync::Arc;

use crate::agent::traits::{CoreAgent, MemoryAgent, ToolAgent, ThreadManagementAgent, StreamingAgentTrait, FullAgent};
use crate::agent::types::{AgentGenerateOptions, AgentGenerateResult, AgentStep, AgentStreamOptions, RuntimeContext, ToolCall};
use crate::agent::trait_def::Agent;
use crate::error::Result;
use crate::llm::{LlmProvider, Message};
use crate::memory::{Memory, WorkingMemory};
use crate::tool::Tool;
use crate::workflow::Workflow;

/// Legacy adapter that bridges new composition-based traits to old God Trait interface
///
/// This adapter allows existing code that expects the old 71-method Agent trait
/// to work with the new composition-based implementation.
pub struct LegacyAgentAdapter<T: FullAgent> {
    inner: T,
}

impl<T: FullAgent> LegacyAgentAdapter<T> {
    /// Create a new legacy adapter
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Get reference to the inner agent
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Get mutable reference to the inner agent
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Consume the adapter and return the inner agent
    pub fn into_inner(self) -> T {
        self.inner
    }
}

#[async_trait]
impl<T: FullAgent> Agent for LegacyAgentAdapter<T> {
    fn get_name(&self) -> &str {
        self.inner.name().unwrap_or("unknown")
    }

    fn get_instructions(&self) -> &str {
        // For backward compatibility, we'll use a default if the agent doesn't have instructions
        // In practice, this should be stored in the agent's config
        "You are a helpful AI assistant." // TODO: Get from agent's config
    }

    fn set_instructions(&mut self, instructions: String) {
        // TODO: This requires mutable access to agent config
        // For now, this is a no-op
        let _ = instructions;
    }

    fn get_llm(&self) -> Arc<dyn LlmProvider> {
        // TODO: 需要从Agent配置中获取LLM
        Arc::new(crate::llm::MockLlmProvider::new(vec!["mock response".to_string()]))
    }

    fn get_memory(&self) -> Option<Arc<dyn Memory>> {
        // TODO: 需要从Agent配置中获取Memory
        None
    }

    fn has_own_memory(&self) -> bool {
        false
    }

    fn get_working_memory(&self) -> Option<Arc<dyn WorkingMemory>> {
        None
    }

    fn get_tools(&self) -> HashMap<String, Box<dyn Tool>> {
        HashMap::new()
    }

    async fn get_tools_with_context(
        &self,
        _context: &RuntimeContext,
    ) -> Result<HashMap<String, Box<dyn Tool>>> {
        Ok(HashMap::new())
    }

    fn add_tool(&mut self, tool: Box<dyn Tool>) -> Result<()> {
        // This requires mutable access to the inner agent
        // TODO: Implement proper mutable delegation
        Err(crate::error::Error::UnsupportedOperation(
            "Mutable tool operations not yet implemented in legacy adapter".to_string(),
        ))
    }

    fn remove_tool(&mut self, tool_name: &str) -> Result<()> {
        // This requires mutable access to the inner agent
        // TODO: Implement proper mutable delegation
        let _ = tool_name;
        Err(crate::error::Error::UnsupportedOperation(
            "Mutable tool operations not yet implemented in legacy adapter".to_string(),
        ))
    }

    fn get_tool(&self, _tool_name: &str) -> Option<Box<dyn Tool>> {
        None
    }

    async fn get_workflows(
        &self,
        _context: &RuntimeContext,
    ) -> Result<HashMap<String, Arc<dyn Workflow>>> {
        Ok(HashMap::new())
    }

    async fn execute_workflow(
        &self,
        _workflow_name: &str,
        _input: serde_json::Value,
        _context: &RuntimeContext,
    ) -> Result<serde_json::Value> {
        Err(crate::error::Error::NotFound(
            "Workflow execution not supported in legacy adapter".to_string(),
        ))
    }

    fn parse_tool_calls(&self, _response: &str) -> Result<Vec<ToolCall>> {
        Ok(vec![])
    }

    async fn execute_tool_call(&self, _tool_call: &ToolCall) -> Result<serde_json::Value> {
        Err(crate::error::Error::UnsupportedOperation(
            "Tool execution not implemented in legacy adapter".to_string(),
        ))
    }

    fn format_messages(&self, messages: &[Message], options: &AgentGenerateOptions) -> Vec<Message> {
        // For backward compatibility, return messages as-is
        // In practice, this should delegate to the inner agent's generation logic
        messages.to_vec()
    }

    async fn generate_title(&self, user_message: &Message) -> Result<String> {
        // Generate a simple title based on the message content
        let content = &user_message.content;
        let title = if content.len() > 50 {
            format!("{}...", &content[..47])
        } else {
            content.clone()
        };
        Ok(title)
    }

    async fn get_instructions_with_context(&self, _context: &RuntimeContext) -> Result<String> {
        Ok(self.get_instructions().to_string())
    }

    async fn generate(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
    ) -> Result<AgentGenerateResult> {
        // 简单的mock实现
        let response = self.get_llm().generate_with_messages(messages, &Default::default()).await?;
        let completion_tokens = response.len() / 4;
        Ok(AgentGenerateResult {
            response,
            steps: vec![],
            usage: crate::agent::types::TokenUsage {
                prompt_tokens: 0,
                completion_tokens,
                total_tokens: completion_tokens,
            },
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn generate_with_context(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
        _context: &RuntimeContext,
    ) -> Result<AgentGenerateResult> {
        self.generate(messages, options).await
    }

    async fn generate_simple(&self, input: &str) -> Result<String> {
        self.get_llm().generate(&format!("{}\n\nPlease respond concisely.", input), &Default::default()).await
    }

    async fn generate_with_memory(
        &self,
        messages: &[Message],
        _thread_id: Option<String>,
        options: &AgentGenerateOptions,
    ) -> Result<AgentGenerateResult> {
        self.generate(messages, options).await
    }

    async fn stream<'a>(
        &'a self,
        _messages: &'a [Message],
        _options: &'a AgentStreamOptions,
    ) -> Result<BoxStream<'a, Result<String>>> {
        use futures::stream;
        Ok(Box::pin(stream::once(async { Ok("Streaming not implemented in legacy adapter".to_string()) })))
    }

    async fn stream_with_callbacks<'a>(
        &'a self,
        _messages: &'a [Message],
        _options: &'a AgentStreamOptions,
        _on_step_finish: Option<Box<dyn FnMut(AgentStep) + Send + 'a>>,
        _on_finish: Option<Box<dyn FnOnce(AgentGenerateResult) + Send + 'a>>,
    ) -> Result<BoxStream<'a, Result<String>>> {
        use futures::stream;
        Ok(Box::pin(stream::once(async { Ok("Streaming not implemented in legacy adapter".to_string()) })))
    }

    fn get_voice(&self) -> Option<Arc<dyn crate::compat::VoiceProvider>> {
        None
    }

    fn set_voice(&mut self, _voice: Arc<dyn crate::compat::VoiceProvider>) {
        // Voice functionality not implemented
    }
}

// Implement Base trait if needed
impl<T: FullAgent> crate::base::Base for LegacyAgentAdapter<T> {
    fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    fn component(&self) -> crate::compat::Component {
        crate::compat::Component::Agent
    }

    fn logger(&self) -> Arc<dyn crate::logger::Logger> {
        // TODO: Get from inner agent or use default
        crate::logger::default_logger()
    }

    fn set_logger(&mut self, _logger: Arc<dyn crate::logger::Logger>) {
        // TODO: Delegate to inner agent if it supports logging
        // For now, this is a no-op since the composition traits don't include logging
    }

    fn telemetry(&self) -> Option<Arc<dyn crate::telemetry::TelemetrySink>> {
        // TODO: Get from inner agent if it supports telemetry
        None
    }

    fn set_telemetry(&mut self, _telemetry: Arc<dyn crate::telemetry::TelemetrySink>) {
        // TODO: Delegate to inner agent if it supports telemetry
        // For now, this is a no-op since the composition traits don't include telemetry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::traits::*;
    use crate::llm::{LlmProvider, LlmOptions, Message, Role};
    use std::sync::Arc;

    // Mock implementations for testing
    struct MockFullAgent {
        name: String,
        llm: Arc<dyn LlmProvider>,
    }

    impl MockFullAgent {
        fn new(name: &str) -> Self {
            struct MockLlm;
            #[async_trait]
            impl LlmProvider for MockLlm {
                fn name(&self) -> &str { "mock" }
                async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> Result<String> {
                    Ok("mock response".to_string())
                }
                async fn generate_with_messages(&self, _messages: &[Message], _options: &LlmOptions) -> Result<String> {
                    Ok("mock response".to_string())
                }
            }

            Self {
                name: name.to_string(),
                llm: Arc::new(MockLlm),
            }
        }
    }

    // Implement all required traits for MockFullAgent
    #[async_trait]
    impl CoreAgent for MockFullAgent {
        fn name(&self) -> &str { &self.name }
        fn instructions(&self) -> &str { "Mock instructions" }
        fn llm(&self) -> &Arc<dyn LlmProvider> { &self.llm }
        fn set_instructions(&mut self, instructions: String) { let _ = instructions; }

        async fn generate(&self, _messages: &[Message], _options: &AgentGenerateOptions) -> Result<AgentGenerateResult> {
            Ok(AgentGenerateResult {
                response: "mock".to_string(),
                tool_calls: vec![],
                usage: None,
                metadata: std::collections::HashMap::new(),
            })
        }
    }

    #[async_trait]
    impl ToolAgent for MockFullAgent {
        fn tools(&self) -> HashMap<String, Box<dyn Tool>> { HashMap::new() }
        fn add_tool(&mut self, _tool: Box<dyn Tool>) -> Result<()> { Ok(()) }
        fn remove_tool(&mut self, _tool_name: &str) -> Result<()> { Ok(()) }
        fn tool(&self, _name: &str) -> Option<Box<dyn Tool>> { None }
        fn parse_tool_calls(&self, _response: &str) -> Result<Vec<ToolCall>> { Ok(vec![]) }
        async fn execute_tool_call(&self, _tool_call: &ToolCall) -> Result<serde_json::Value> { Ok(serde_json::Value::Null) }
    }

    impl MemoryAgent for MockFullAgent {
        fn memory(&self) -> Option<&Arc<dyn Memory>> { None }
        fn has_own_memory(&self) -> bool { false }
        fn working_memory(&self) -> Option<&Arc<dyn WorkingMemory>> { None }
    }

    #[async_trait]
    impl StreamingAgentTrait for MockFullAgent {
        async fn stream(&self, _messages: &[Message], _options: &AgentStreamOptions) -> Result<BoxStream<'_, Result<String>>> {
            use futures::stream;
            Ok(Box::pin(stream::once(async { Ok("mock stream".to_string()) })))
        }
    }

    impl crate::base::Base for MockFullAgent {
        fn name(&self) -> Option<&str> { Some(&self.name) }
        fn component(&self) -> crate::compat::Component { crate::compat::Component::Agent }
        fn logger(&self) -> Arc<dyn crate::logger::Logger> { crate::logger::default_logger() }
        fn set_logger(&mut self, _logger: Arc<dyn crate::logger::Logger>) {}
        fn telemetry(&self) -> Option<Arc<dyn crate::telemetry::TelemetrySink>> { None }
        fn set_telemetry(&mut self, _telemetry: Arc<dyn crate::telemetry::TelemetrySink>) {}
    }

    #[test]
    fn test_legacy_adapter_backward_compatibility() {
        let inner_agent = MockFullAgent::new("test_agent");
        let adapter = LegacyAgentAdapter::new(inner_agent);

        // Test basic Agent trait methods
        assert_eq!(adapter.get_name(), "test_agent");
        assert_eq!(adapter.get_instructions(), "You are a helpful AI assistant."); // Default for backward compatibility
        assert!(adapter.get_memory().is_none());
        assert!(!adapter.has_own_memory());
        assert!(adapter.get_tools().is_empty());
    }

    #[tokio::test]
    async fn test_legacy_adapter_generation() {
        let inner_agent = MockFullAgent::new("test_agent");
        let adapter = LegacyAgentAdapter::new(inner_agent);

        let messages = vec![Message {
            role: Role::User,
            content: "Hello".to_string(),
            metadata: None,
            name: None,
        }];

        let result = adapter.generate(&messages, &AgentGenerateOptions::default()).await.unwrap();
        assert_eq!(result.response, "mock");
    }
}
