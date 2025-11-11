//! E2E Test Context
//!
//! Provides a shared context for E2E tests with common setup and teardown.

use lumosai_core::agent::{AgentBuilder, BasicAgent};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use lumosai_core::llm::LlmProvider;
use lumosai_core::prelude::*;
use lumosai_core::tool::{Tool, ToolBuilder};
use lumosai_core::vector::MemoryVectorStorage;
use lumosai_core::workflow::DagWorkflow;
use std::sync::Arc;

/// E2E test context with common test resources
pub struct E2ETestContext {
    /// Test LLM provider
    pub llm: Arc<dyn LlmProvider>,
    /// Test vector storage
    pub vector_storage: Arc<MemoryVectorStorage>,
    /// Test agents
    pub agents: Vec<BasicAgent>,
    /// Test tools
    pub tools: Vec<Arc<dyn Tool>>,
    /// Test workflows
    pub workflows: Vec<Arc<DagWorkflow>>,
}

impl E2ETestContext {
    /// Create a new test context
    pub async fn new() -> Result<Self> {
        Ok(Self {
            llm: create_test_zhipu_provider_arc(),
            vector_storage: Arc::new(MemoryVectorStorage::new(384, None)),
            agents: Vec::new(),
            tools: Vec::new(),
            workflows: Vec::new(),
        })
    }

    /// Create a test agent with default configuration
    pub async fn create_agent(&mut self, name: &str, instructions: &str) -> Result<BasicAgent> {
        let agent = AgentBuilder::new()
            .name(name)
            .instructions(instructions)
            .model(self.llm.clone())
            .build()?;

        self.agents.push(agent.clone());
        Ok(agent)
    }

    /// Create a test agent with tools
    pub async fn create_agent_with_tools(
        &mut self,
        name: &str,
        instructions: &str,
        tools: Vec<Arc<dyn Tool>>,
    ) -> Result<BasicAgent> {
        let mut builder = AgentBuilder::new()
            .name(name)
            .instructions(instructions)
            .model(self.llm.clone());

        for tool in tools {
            builder = builder.tool(tool);
        }

        let agent = builder.build()?;
        self.agents.push(agent.clone());
        Ok(agent)
    }

    /// Create a simple test tool
    pub fn create_simple_tool(&mut self, name: &str, description: &str) -> Result<Arc<dyn Tool>> {
        let tool = ToolBuilder::new()
            .name(name)
            .description(description)
            .parameter("input", "string", "Input parameter", true)
            .handler(|params| {
                let input = params
                    .get("input")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                Ok(serde_json::json!({
                    "result": format!("Processed: {}", input)
                }))
            })
            .build()?;

        let tool_arc = Arc::new(tool);
        self.tools.push(tool_arc.clone());
        Ok(tool_arc)
    }

    /// Create a calculator tool
    pub fn create_calculator_tool(&mut self) -> Result<Arc<dyn Tool>> {
        let tool = ToolBuilder::new()
            .name("calculator")
            .description("Performs basic math operations")
            .parameter("operation", "string", "Operation type (add, subtract, multiply, divide)", true)
            .parameter("a", "number", "First number", true)
            .parameter("b", "number", "Second number", true)
            .handler(|params| {
                let operation = params
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::InvalidInput("Missing operation".to_string()))?;

                let a = params
                    .get("a")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| Error::InvalidInput("Missing parameter 'a'".to_string()))?;

                let b = params
                    .get("b")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| Error::InvalidInput("Missing parameter 'b'".to_string()))?;

                let result = match operation {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    "divide" => {
                        if b == 0.0 {
                            return Err(Error::InvalidInput("Division by zero".to_string()));
                        }
                        a / b
                    }
                    _ => return Err(Error::InvalidInput(format!("Unknown operation: {}", operation))),
                };

                Ok(serde_json::json!({
                    "result": result,
                    "operation": operation
                }))
            })
            .build()?;

        let tool_arc = Arc::new(tool);
        self.tools.push(tool_arc.clone());
        Ok(tool_arc)
    }

    /// Create a test workflow
    pub async fn create_workflow(&mut self, name: &str) -> Result<Arc<DagWorkflow>> {
        let workflow = Arc::new(DagWorkflow::new(name.to_string(), None));
        self.workflows.push(workflow.clone());
        Ok(workflow)
    }

    /// Cleanup test resources
    pub async fn cleanup(&mut self) {
        self.agents.clear();
        self.tools.clear();
        self.workflows.clear();
    }
}

impl Drop for E2ETestContext {
    fn drop(&mut self) {
        // Cleanup is handled by Drop implementations of individual resources
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_context() {
        let context = E2ETestContext::new().await;
        assert!(context.is_ok());
    }

    #[tokio::test]
    async fn test_create_agent() {
        let mut context = E2ETestContext::new().await.unwrap();
        let agent = context.create_agent("test", "Test agent").await;
        assert!(agent.is_ok());
        assert_eq!(context.agents.len(), 1);
    }

    #[tokio::test]
    async fn test_create_tool() {
        let mut context = E2ETestContext::new().await.unwrap();
        let tool = context.create_simple_tool("test_tool", "Test tool");
        assert!(tool.is_ok());
        assert_eq!(context.tools.len(), 1);
    }

    #[tokio::test]
    async fn test_create_calculator_tool() {
        let mut context = E2ETestContext::new().await.unwrap();
        let tool = context.create_calculator_tool();
        assert!(tool.is_ok());
        assert_eq!(context.tools.len(), 1);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let mut context = E2ETestContext::new().await.unwrap();
        context.create_agent("test", "Test agent").await.unwrap();
        context.create_simple_tool("test_tool", "Test tool").unwrap();

        assert_eq!(context.agents.len(), 1);
        assert_eq!(context.tools.len(), 1);

        context.cleanup().await;

        assert_eq!(context.agents.len(), 0);
        assert_eq!(context.tools.len(), 0);
    }
}

