#[cfg(test)]
mod tests {
    use crate::workflow::basic::{Workflow, StepCondition, WorkflowStep, BasicWorkflow, StepResult};
    use crate::workflow::step::{BasicStep, StepBuilder, StepConfig};
    use crate::error::{Error, Result};
    use crate::agent::{Agent, AgentGenerateResult, AgentStep, StepType, TokenUsage, assistant_message};
    use crate::llm::{Message, Role};
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use std::sync::Arc;
    use async_trait::async_trait;
    use crate::base::Base;
    use crate::logger::{Logger, Component};
    use crate::telemetry::TelemetrySink;
    use uuid::Uuid;
    
    // 实现一个简单的Mock代理
    #[derive(Clone)]
    struct MockAgent {
        generate_fn: Arc<dyn Fn(&[Message]) -> Result<AgentGenerateResult> + Send + Sync>,
    }
    
    impl MockAgent {
        fn new<F>(generate_fn: F) -> Self
        where
            F: Fn(&[Message]) -> Result<AgentGenerateResult> + Send + Sync + 'static,
        {
            Self {
                generate_fn: Arc::new(generate_fn),
            }
        }
    }
    
    impl Base for MockAgent {
        fn name(&self) -> Option<&str> {
            Some("MockAgent")
        }
        
        fn component(&self) -> Component {
            Component::Agent
        }
        
        fn logger(&self) -> Arc<dyn Logger> {
            panic!("此方法不应该被调用")
        }
        
        fn set_logger(&mut self, _logger: Arc<dyn Logger>) {
            // 不做任何事情
        }
        
        fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
            None
        }
        
        fn set_telemetry(&mut self, _telemetry: Arc<dyn TelemetrySink>) {
            // 不做任何事情
        }
    }
    
    #[async_trait::async_trait]
    impl Agent for MockAgent {
        async fn generate(&self, messages: &[Message], _options: &crate::agent::AgentGenerateOptions) -> Result<AgentGenerateResult> {
            (self.generate_fn)(messages)
        }
        
        fn get_name(&self) -> &str {
            "MockAgent"
        }
        
        fn get_instructions(&self) -> &str {
            "Mock instructions"
        }
        
        fn set_instructions(&mut self, _instructions: String) {
            // 不做任何事情
        }
        
        fn get_llm(&self) -> Arc<dyn crate::llm::LlmProvider> {
            panic!("此方法不应该被调用")
        }
        
        fn get_memory(&self) -> Option<Arc<dyn crate::memory::Memory>> {
            None
        }
        
        fn has_own_memory(&self) -> bool {
            false
        }
        
        fn get_tools(&self) -> HashMap<String, Box<dyn crate::tool::Tool>> {
            HashMap::new()
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
            Ok(Vec::new())
        }
        
        async fn execute_tool_call(&self, _tool_call: &crate::agent::ToolCall) -> Result<Value> {
            Ok(json!({}))
        }
        
        fn format_messages(&self, messages: &[Message], _options: &crate::agent::AgentGenerateOptions) -> Vec<Message> {
            messages.to_vec()
        }
        
        async fn generate_title(&self, _user_message: &Message) -> Result<String> {
            Ok("Mock title".to_string())
        }
        
        async fn stream<'a>(&'a self, _messages: &'a [Message], _options: &'a crate::agent::AgentStreamOptions) -> Result<futures::stream::BoxStream<'a, Result<String>>> {
            unimplemented!("Stream not implemented for MockAgent")
        }
    }

    #[tokio::test]
    async fn test_basic_workflow() {
        // 创建两个Mock代理
        let agent1 = Arc::new(MockAgent::new(|messages| {
            let input = if let Some(msg) = messages.last() {
                &msg.content
            } else {
                "empty"
            };
            let output = format!("已处理: {}", input);
            
            Ok(AgentGenerateResult {
                response: output.clone(),
                steps: vec![AgentStep {
                    id: Uuid::new_v4().to_string(),
                    step_type: StepType::Final,
                    input: messages.to_vec(),
                    output: Some(assistant_message(&output)),
                    tool_calls: vec![],
                    tool_results: vec![],
                    metadata: HashMap::new(),
                }],
                usage: TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                },
                metadata: HashMap::new(),
            })
        }));
        let agent2 = Arc::new(MockAgent::new(|messages| {
            let input = if let Some(msg) = messages.last() {
                &msg.content
            } else {
                "empty"
            };
            let output = format!("已处理: {}", input);
            
            Ok(AgentGenerateResult {
                response: output.clone(),
                steps: vec![AgentStep {
                    id: Uuid::new_v4().to_string(),
                    step_type: StepType::Final,
                    input: messages.to_vec(),
                    output: Some(assistant_message(&output)),
                    tool_calls: vec![],
                    tool_results: vec![],
                    metadata: HashMap::new(),
                }],
                usage: TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                },
                metadata: HashMap::new(),
            })
        }));
        
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
        
        // 如果成功，应该有最终的结果 - 最后一个步骤的输出
        let output = result.unwrap();
        assert!(output.is_object() || output.is_string(), "输出应该是对象或字符串");
        
        let output_str = output.to_string();
        assert!(output_str.contains("已处理:") || output_str.contains("Processed:"), 
               "输出应包含处理后的内容");
    }

    #[tokio::test]
    async fn test_workflow_conditions() {
        // 创建测试代理
        let agent1 = Arc::new(MockAgent::new(|messages| {
            let input = if let Some(msg) = messages.last() {
                &msg.content
            } else {
                "empty"
            };
            let output = format!("已处理: {}", input);
            
            Ok(AgentGenerateResult {
                response: output.clone(),
                steps: vec![AgentStep {
                    id: Uuid::new_v4().to_string(),
                    step_type: StepType::Final,
                    input: messages.to_vec(),
                    output: Some(assistant_message(&output)),
                    tool_calls: vec![],
                    tool_results: vec![],
                    metadata: HashMap::new(),
                }],
                usage: TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                },
                metadata: HashMap::new(),
            })
        }));
        let agent2 = Arc::new(MockAgent::new(|messages| {
            let input = if let Some(msg) = messages.last() {
                &msg.content
            } else {
                "empty"
            };
            let output = format!("已处理: {}", input);
            
            Ok(AgentGenerateResult {
                response: output.clone(),
                steps: vec![AgentStep {
                    id: Uuid::new_v4().to_string(),
                    step_type: StepType::Final,
                    input: messages.to_vec(),
                    output: Some(assistant_message(&output)),
                    tool_calls: vec![],
                    tool_results: vec![],
                    metadata: HashMap::new(),
                }],
                usage: TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                },
                metadata: HashMap::new(),
            })
        }));
        let agent3 = Arc::new(MockAgent::new(|messages| {
            let input = if let Some(msg) = messages.last() {
                &msg.content
            } else {
                "empty"
            };
            let output = format!("已处理: {}", input);
            
            Ok(AgentGenerateResult {
                response: output.clone(),
                steps: vec![AgentStep {
                    id: Uuid::new_v4().to_string(),
                    step_type: StepType::Final,
                    input: messages.to_vec(),
                    output: Some(assistant_message(&output)),
                    tool_calls: vec![],
                    tool_results: vec![],
                    metadata: HashMap::new(),
                }],
                usage: TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                },
                metadata: HashMap::new(),
            })
        }));
        
        // 创建工作流
        let mut workflow = BasicWorkflow::new("Conditional Workflow");
        
        // 第一个步骤始终执行
        let step1 = WorkflowStep {
            name: "Step1".to_string(),
            agent: agent1.clone(),
            instructions: "Execute step 1".to_string(),
            condition: StepCondition::Always,
            timeout_ms: None,
            retry_count: None,
        };
        
        // 第二个步骤在第一个步骤成功后执行
        let step2 = WorkflowStep {
            name: "Step2".to_string(),
            agent: agent2.clone(),
            instructions: "Execute step 2".to_string(),
            condition: StepCondition::StepCompleted("Step1".to_string()),
            timeout_ms: None,
            retry_count: None,
        };
        
        // 第三个步骤在第一个步骤成功且第二个步骤成功后执行
        let step3 = WorkflowStep {
            name: "Step3".to_string(),
            agent: agent3.clone(),
            instructions: "Execute step 3".to_string(),
            condition: StepCondition::And(vec![
                StepCondition::StepCompleted("Step1".to_string()),
                StepCondition::StepCompleted("Step2".to_string()),
            ]),
            timeout_ms: None,
            retry_count: None,
        };
        
        // 添加步骤到工作流
        workflow.add_step(step1);
        workflow.add_step(step2);
        workflow.add_step(step3);
        
        // 执行工作流
        let input = json!({
            "query": "Test conditional workflow"
        });
        
        let result = workflow.execute(input).await;
        assert!(result.is_ok());
        
        // 工作流应该完成并包含最终的结果 - 最后一个步骤的输出
        let output = result.unwrap();
        assert!(output.is_object() || output.is_string(), "输出应该是对象或字符串");
        
        let output_str = output.to_string();
        assert!(output_str.contains("已处理:") || output_str.contains("Processed:"), 
               "输出应包含处理后的内容");
    }

    #[tokio::test]
    async fn test_workflow_error_handling() {
        // 创建会失败的模拟代理
        let failing_agent = MockAgent::new(|_| {
            Err(Error::Agent("模拟代理故意失败".to_string()))
        });

        let success_agent = MockAgent::new(|messages| {
            let input = if let Some(msg) = messages.last() {
                &msg.content
            } else {
                "empty"
            };
            let output = format!("已处理: {}", input);
            
            Ok(AgentGenerateResult {
                response: output.clone(),
                steps: vec![AgentStep {
                    id: Uuid::new_v4().to_string(),
                    step_type: StepType::Final,
                    input: messages.to_vec(),
                    output: Some(assistant_message(&output)),
                    tool_calls: vec![],
                    tool_results: vec![],
                    metadata: HashMap::new(),
                }],
                usage: TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                },
                metadata: HashMap::new(),
            })
        });

        // 创建工作流
        let mut workflow = BasicWorkflow::new("错误处理工作流".to_string());
        workflow = workflow.with_description("测试工作流错误处理能力".to_string());
        
        // 添加会失败的步骤
        let failing_step = WorkflowStep {
            name: "失败步骤".to_string(),
            agent: Arc::new(failing_agent),
            instructions: "这个步骤会失败".to_string(),
            condition: StepCondition::Always,
            timeout_ms: None,
            retry_count: None,
        };
        
        // 添加只有在上一步成功时才会执行的步骤
        let conditional_step = WorkflowStep {
            name: "条件步骤".to_string(),
            agent: Arc::new(success_agent.clone()),
            instructions: "这个步骤应该不会执行，因为依赖的步骤失败了".to_string(),
            condition: StepCondition::StepCompleted("失败步骤".to_string()),
            timeout_ms: None,
            retry_count: None,
        };
        
        // 添加无论前面步骤成功与否都会执行的步骤
        let final_step = WorkflowStep {
            name: "最终步骤".to_string(),
            agent: Arc::new(success_agent),
            instructions: "这个步骤应该总是执行".to_string(),
            condition: StepCondition::Always,
            timeout_ms: None,
            retry_count: None,
        };
        
        workflow.add_step(failing_step);
        workflow.add_step(conditional_step);
        workflow.add_step(final_step);

        // 执行工作流
        let result = workflow.execute(serde_json::json!({})).await;
        
        // 验证结果
        assert!(result.is_ok(), "工作流执行应该成功完成，即使有步骤失败");
        
        let output = result.unwrap();
        
        // 验证输出是对象格式
        assert!(output.is_object(), "输出应该是一个对象");
        let output_map = output.as_object().unwrap();
        
        // 验证第一个步骤失败
        if let Some(failed_step) = output_map.get("失败步骤") {
            if let Some(failed_obj) = failed_step.as_object() {
                assert_eq!(failed_obj.get("success").unwrap().as_bool().unwrap(), false);
                assert!(failed_obj.get("error").unwrap().as_str().unwrap().contains("模拟代理故意失败"));
            } else {
                panic!("失败步骤的结果不是对象格式");
            }
        } else {
            // 如果输出中没有失败步骤的特定键，则检查输出本身是否包含错误信息
            assert!(output.to_string().contains("模拟代理故意失败"), 
                   "输出应该包含失败步骤的错误信息");
        }
        
        // 验证最终步骤执行了 - 适应多种可能的输出格式
        if let Some(final_step) = output_map.get("最终步骤") {
            if let Some(final_obj) = final_step.as_object() {
                assert!(final_obj.get("success").unwrap().as_bool().unwrap(), "最终步骤应该成功执行");
                if let Some(output_val) = final_obj.get("output") {
                    if let Some(output_str) = output_val.as_str() {
                        assert!(output_str.contains("这个步骤应该总是执行"), "最终步骤的输出不正确");
                    } else {
                        // 输出可能是另一种格式
                        assert!(!output_val.is_null(), "最终步骤应该有输出");
                    }
                }
            }
        } else {
            // 输出本身可能就是最终步骤的结果
            assert!(output.to_string().contains("这个步骤应该总是执行"), 
                  "输出应该包含最终步骤的处理结果");
        }
        
        // 验证条件步骤没有执行（因为依赖的步骤失败了）
        assert!(!output.to_string().contains("这个步骤应该不会执行"), 
               "条件步骤不应该被执行");
    }
} 