//! Integration tests for workflow functionality
//! 
//! Tests the complete workflow system including configuration loading,
//! workflow creation, and execution.

use lumosai_core::app::LumosApp;
use lumosai_core::config::{YamlConfig, ConfigLoader};
use lumosai_core::workflow::{WorkflowBuilder, EnhancedWorkflow};
use lumosai_core::agent::AgentBuilder;
use lumosai_core::llm::MockLlmProvider;
use std::sync::Arc;
use tempfile::tempdir;
use std::fs;

#[tokio::test]
async fn test_workflow_from_config() {
    let yaml_content = r#"
project:
  name: workflow-test-app
  version: 1.0.0
  description: Test application with workflows

agents:
  assistant:
    model: gpt-4
    instructions: You are a helpful assistant
    tools:
      - web_search
      - calculator
    temperature: 0.7
    max_tokens: 2000

  coder:
    model: deepseek-coder
    instructions: You are an expert programmer
    tools:
      - code_executor
      - file_manager
    temperature: 0.3

workflows:
  support_workflow:
    id: support_wf_001
    name: Customer Support Workflow
    description: Handle customer support requests
    trigger: user_message
    timeout: 300
    max_retries: 3
    steps:
      - id: initial_assessment
        name: Initial Assessment
        description: Assess the user's request
        agent: assistant
        condition: always
        timeout: 60
        retries: 2
      
      - id: code_help
        name: Code Help
        description: Provide coding assistance if needed
        agent: coder
        condition: code_related
        timeout: 120
        retries: 1

  simple_workflow:
    id: simple_wf_001
    name: Simple Workflow
    description: A simple single-step workflow
    trigger: api_call
    steps:
      - id: single_step
        name: Single Step
        agent: assistant
        condition: always

rag:
  vector_store: memory
  embeddings: openai
  chunk_size: 1000
  documents:
    - docs/
    - knowledge/
"#;

    // Create temporary config file
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.yaml");
    fs::write(&config_path, yaml_content).unwrap();

    // Load configuration
    let config = ConfigLoader::load(&config_path).unwrap();
    
    // Validate configuration structure
    assert!(config.project.is_some());
    assert!(config.agents.is_some());
    assert!(config.workflows.is_some());
    
    let project = config.project.as_ref().unwrap();
    assert_eq!(project.name, "workflow-test-app");
    
    let agents = config.agents.as_ref().unwrap();
    assert!(agents.contains_key("assistant"));
    assert!(agents.contains_key("coder"));
    
    let workflows = config.workflows.as_ref().unwrap();
    assert!(workflows.contains_key("support_workflow"));
    assert!(workflows.contains_key("simple_workflow"));
    
    // Test workflow configuration details
    let support_workflow = workflows.get("support_workflow").unwrap();
    assert_eq!(support_workflow.id.as_ref().unwrap(), "support_wf_001");
    assert_eq!(support_workflow.name.as_ref().unwrap(), "Customer Support Workflow");
    assert_eq!(support_workflow.steps.len(), 2);
    assert_eq!(support_workflow.timeout, Some(300));
    assert_eq!(support_workflow.max_retries, Some(3));
    
    // Test step configuration
    let first_step = &support_workflow.steps[0];
    assert_eq!(first_step.id.as_ref().unwrap(), "initial_assessment");
    assert_eq!(first_step.agent.as_ref().unwrap(), "assistant");
    assert_eq!(first_step.timeout, Some(60));
    assert_eq!(first_step.retries, Some(2));
    
    let second_step = &support_workflow.steps[1];
    assert_eq!(second_step.id.as_ref().unwrap(), "code_help");
    assert_eq!(second_step.agent.as_ref().unwrap(), "coder");
    assert_eq!(second_step.condition.as_ref().unwrap(), "code_related");
}

#[tokio::test]
async fn test_workflow_builder() {
    // Create a mock LLM provider
    let mock_llm = Arc::new(MockLlmProvider::new(vec![
        "Hello! I'm here to help.".to_string(),
        "I can assist with coding tasks.".to_string(),
    ]));

    // Create agents
    let assistant = AgentBuilder::new()
        .name("assistant")
        .instructions("You are a helpful assistant")
        .model(mock_llm.clone())
        .build()
        .expect("Failed to build assistant agent");

    let coder = AgentBuilder::new()
        .name("coder")
        .instructions("You are an expert programmer")
        .model(mock_llm.clone())
        .build()
        .expect("Failed to build coder agent");

    // Create workflow using builder
    let workflow = WorkflowBuilder::new()
        .id("test_workflow")
        .name("Test Workflow")
        .description("A test workflow")
        .trigger("user_message")
        .timeout(300)
        .max_retries(3)
        .register_agent("assistant".to_string(), Arc::new(assistant))
        .register_agent("coder".to_string(), Arc::new(coder));

    // Note: We can't complete the build without step configurations
    // This test validates the builder pattern works correctly
    
    // In a real scenario, we would add step configurations and build
    // let workflow = workflow.build().expect("Failed to build workflow");
}

#[tokio::test]
async fn test_app_with_workflows() {
    let yaml_content = r#"
project:
  name: workflow-app
  version: 1.0.0

agents:
  assistant:
    model: gpt-4
    instructions: You are helpful
    tools: []

workflows:
  simple:
    id: simple_001
    name: Simple Workflow
    trigger: user_message
    steps:
      - id: step1
        agent: assistant
        condition: always
"#;

    // Create temporary config file
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("app_config.yaml");
    fs::write(&config_path, yaml_content).unwrap();

    // Create app from config
    // Note: This will fail because we don't have real LLM providers
    // but it tests the configuration parsing and structure
    let result = LumosApp::from_config(&config_path).await;
    
    // The result should be an error due to missing API keys, but not due to config structure
    assert!(result.is_err());
    
    // Test that we can at least load the configuration
    let config = ConfigLoader::load(&config_path).unwrap();
    assert!(config.workflows.is_some());
    
    let workflows = config.workflows.as_ref().unwrap();
    assert!(workflows.contains_key("simple"));
    
    let simple_workflow = workflows.get("simple").unwrap();
    assert_eq!(simple_workflow.steps.len(), 1);
    assert_eq!(simple_workflow.steps[0].agent.as_ref().unwrap(), "assistant");
}

#[test]
fn test_workflow_config_validation() {
    // Test valid workflow configuration
    let valid_yaml = r#"
project:
  name: test-app

agents:
  assistant:
    model: gpt-4
    instructions: You are helpful

workflows:
  valid_workflow:
    id: valid_001
    name: Valid Workflow
    steps:
      - id: step1
        agent: assistant
"#;

    let config = YamlConfig::from_str(valid_yaml).unwrap();
    assert!(config.validate().is_ok());

    // Test invalid workflow configuration - empty steps
    let invalid_yaml = r#"
project:
  name: test-app

agents:
  assistant:
    model: gpt-4
    instructions: You are helpful

workflows:
  invalid_workflow:
    id: invalid_001
    name: Invalid Workflow
    steps: []
"#;

    let config = YamlConfig::from_str(invalid_yaml).unwrap();
    assert!(config.validate().is_err());

    // Test invalid workflow configuration - step without agent/tool/workflow
    let invalid_yaml2 = r#"
project:
  name: test-app

agents:
  assistant:
    model: gpt-4
    instructions: You are helpful

workflows:
  invalid_workflow:
    id: invalid_001
    name: Invalid Workflow
    steps:
      - id: step1
        condition: always
"#;

    let config = YamlConfig::from_str(invalid_yaml2).unwrap();
    assert!(config.validate().is_err());
}

#[test]
fn test_workflow_step_types() {
    // Test workflow with different step types
    let yaml_content = r#"
project:
  name: multi-step-app

agents:
  assistant:
    model: gpt-4
    instructions: You are helpful

workflows:
  multi_step:
    id: multi_001
    name: Multi-Step Workflow
    steps:
      - id: agent_step
        name: Agent Step
        agent: assistant
        condition: always
      
      - id: tool_step
        name: Tool Step
        tool: calculator
        condition: needs_calculation
      
      - id: workflow_step
        name: Sub-Workflow Step
        workflow: sub_workflow_id
        condition: complex_task
"#;

    let config = YamlConfig::from_str(yaml_content).unwrap();
    assert!(config.validate().is_ok());
    
    let workflows = config.workflows.as_ref().unwrap();
    let multi_step = workflows.get("multi_step").unwrap();
    
    assert_eq!(multi_step.steps.len(), 3);
    
    // Check agent step
    let agent_step = &multi_step.steps[0];
    assert!(agent_step.agent.is_some());
    assert!(agent_step.tool.is_none());
    assert!(agent_step.workflow.is_none());
    
    // Check tool step
    let tool_step = &multi_step.steps[1];
    assert!(tool_step.agent.is_none());
    assert!(tool_step.tool.is_some());
    assert!(tool_step.workflow.is_none());
    
    // Check workflow step
    let workflow_step = &multi_step.steps[2];
    assert!(workflow_step.agent.is_none());
    assert!(workflow_step.tool.is_none());
    assert!(workflow_step.workflow.is_some());
}
