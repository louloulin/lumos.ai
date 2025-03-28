#[cfg(test)]
mod tests {
    use crate::workflow::basic::{Workflow, StepCondition, WorkflowStep, BasicWorkflow, StepResult};
    use crate::workflow::step::{BasicStep, StepBuilder, StepConfig};
    use crate::error::{Error, Result};
    use serde_json::json;
    use std::sync::Arc;
    use async_trait::async_trait;
    use crate::agent::Agent;
    use crate::llm::Message;
    use crate::base::Base;
    use crate::logger::{Logger, Component};
    
    // 实现一个简单的Mock代理
    struct MockAgent {
        name: String,
    }
    
    impl MockAgent {
        fn new(name: impl Into<String>) -> Self {
            Self { name: name.into() }
        }
    }
    
    impl Base for MockAgent {
        fn name(&self) -> Option<&str> {
            Some(&self.name)
        }
        
        fn component(&self) -> Component {
            Component::Agent
        }
        
        fn logger(&self) -> Arc<dyn Logger> {
            unimplemented!()
        }
        
        fn set_logger(&mut self, _logger: Arc<dyn Logger>) {
            // 不做任何事
        }
        
        fn telemetry(&self) -> Option<Arc<dyn crate::telemetry::TelemetrySink>> {
            None
        }
        
        fn set_telemetry(&mut self, _telemetry: Arc<dyn crate::telemetry::TelemetrySink>) {
            // 不做任何事
        }
    }
    
    #[async_trait]
    impl Agent for MockAgent {
        fn get_name(&self) -> &str {
            &self.name
        }
        
        fn get_instructions(&self) -> &str {
            "Mock instructions"
        }
        
        fn set_instructions(&mut self, _instructions: String) {
            // 不做任何事
        }
        
        fn get_llm(&self) -> Arc<dyn crate::llm::LlmProvider> {
            unimplemented!()
        }
        
        fn get_memory(&self) -> Option<Arc<dyn crate::memory::Memory>> {
            None
        }
        
        fn has_own_memory(&self) -> bool {
            false
        }
        
        fn get_tools(&self) -> std::collections::HashMap<String, Box<dyn crate::tool::Tool>> {
            std::collections::HashMap::new()
        }
        
        fn add_tool(&mut self, _tool: Box<dyn crate::tool::Tool>) -> Result<()> {
            Ok(())
        }
        
        fn remove_tool(&mut self, _tool_name: &str) -> Result<()> {
            Ok(())
        }
        
        fn get_tool(&self, _tool_name: &str) -> Option<&Box<dyn crate::tool::Tool>> {
            None
        }
        
        fn parse_tool_calls(&self, _response: &str) -> Result<Vec<crate::agent::ToolCall>> {
            Ok(vec![])
        }
        
        async fn execute_tool_call(&self, _tool_call: &crate::agent::ToolCall) -> Result<serde_json::Value> {
            Ok(json!({}))
        }
        
        fn format_messages(&self, messages: &[Message], _options: &crate::agent::AgentGenerateOptions) -> Vec<Message> {
            messages.to_vec()
        }
        
        async fn generate_title(&self, _user_message: &Message) -> Result<String> {
            Ok("Mock title".to_string())
        }
        
        async fn generate(&self, messages: &[Message], _options: &crate::agent::AgentGenerateOptions) -> Result<crate::agent::AgentGenerateResult> {
            // 简单返回最后一条消息的内容
            let content = if let Some(msg) = messages.last() {
                format!("Processed: {}", msg.content)
            } else {
                "No input".to_string()
            };
            
            let step = crate::agent::AgentStep {
                id: uuid::Uuid::new_v4().to_string(),
                step_type: crate::agent::StepType::Final,
                input: messages.to_vec(),
                output: Some(crate::agent::assistant_message(&content)),
                tool_calls: vec![],
                tool_results: vec![],
                metadata: std::collections::HashMap::new(),
            };
            
            Ok(crate::agent::AgentGenerateResult {
                response: content,
                steps: vec![step],
                usage: crate::agent::TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                },
                metadata: std::collections::HashMap::new(),
            })
        }
        
        async fn stream<'a>(&'a self, _messages: &'a [Message], _options: &'a crate::agent::AgentStreamOptions) -> Result<futures::stream::BoxStream<'a, Result<String>>> {
            unimplemented!()
        }
    }
    
    #[tokio::test]
    async fn test_basic_workflow() {
        // 创建两个Mock代理
        let agent1 = Arc::new(MockAgent::new("Agent1"));
        let agent2 = Arc::new(MockAgent::new("Agent2"));
        
        // 创建一个简单的工作流
        let mut workflow = BasicWorkflow::new("Test Workflow");
        
        // 添加步骤
        let step1 = WorkflowStep {
            name: "Step1".to_string(),
            agent: agent1,
            instructions: "Process step 1".to_string(),
            condition: StepCondition::Always,
            timeout_ms: None,
            retry_count: None,
        };
        
        let step2 = WorkflowStep {
            name: "Step2".to_string(),
            agent: agent2,
            instructions: "Process step 2".to_string(),
            condition: StepCondition::StepCompleted("Step1".to_string()),
            timeout_ms: None,
            retry_count: None,
        };
        
        workflow.add_step(step1);
        workflow.add_step(step2);
        
        // 执行工作流
        let input = json!({
            "query": "Test query"
        });
        
        let result = workflow.execute(input).await;
        assert!(result.is_ok());
        
        // 如果成功，应该有最终的结果
        let output = result.unwrap();
        assert!(output.is_object());
        assert!(output.to_string().contains("Processed:"));
    }
} 