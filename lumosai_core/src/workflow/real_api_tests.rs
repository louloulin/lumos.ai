//! Workflow tests using real Zhipu AI API
//!
//! These tests verify workflow functionality with actual LLM calls

#[cfg(test)]
mod tests {
    use crate::agent::{AgentConfig, BasicAgent};
    use crate::llm::test_helpers::create_test_zhipu_provider_arc;
    use crate::workflow::basic::{BasicWorkflow, StepCondition, Workflow, WorkflowStep};
    use serde_json::{json, Value};
    use std::sync::Arc;
    use std::time::Duration;

    /// Helper function to retry API calls with exponential backoff
    async fn retry_with_backoff<F, Fut, T>(
        mut f: F,
        max_retries: u32,
        initial_delay_ms: u64,
    ) -> crate::Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = crate::Result<T>>,
    {
        let mut delay = initial_delay_ms;
        for attempt in 0..max_retries {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    if error_msg.contains("429")
                        || error_msg.contains("Too Many Requests")
                        || error_msg.contains("1302")
                    {
                        if attempt < max_retries - 1 {
                            eprintln!(
                                "⚠️  Rate limit hit (attempt {}/{}), retrying after {}ms...",
                                attempt + 1,
                                max_retries,
                                delay
                            );
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                            delay *= 2; // Exponential backoff
                            continue;
                        }
                    }
                    return Err(e);
                }
            }
        }
        unreachable!()
    }

    #[tokio::test]
    async fn test_workflow_single_step_execution() {
        let llm = create_test_zhipu_provider_arc();
        let agent = Arc::new(BasicAgent::new(
            AgentConfig {
                name: "test_agent".to_string(),
                instructions: "You are a helpful assistant. Respond concisely.".to_string(),
                ..Default::default()
            },
            llm,
        ));

        let mut workflow = BasicWorkflow::new("single_step_workflow");
        workflow.add_step(WorkflowStep {
            name: "step1".to_string(),
            agent: agent.clone(),
            instructions: "Process the input and return a summary".to_string(),
            condition: StepCondition::Always,
            timeout_ms: None,
            retry_count: None,
        });

        let input = json!({"task": "Summarize: AI is transforming technology"});

        tokio::time::sleep(Duration::from_millis(1000)).await;

        let result =
            retry_with_backoff(|| async { workflow.execute(input.clone()).await }, 5, 2000).await;

        assert!(
            result.is_ok(),
            "Workflow execution failed: {:?}",
            result.err()
        );
        let output = result.unwrap();
        assert!(!output.is_null(), "Workflow output should not be null");
    }

    #[tokio::test]
    async fn test_workflow_multi_step_sequential() {
        let llm1 = create_test_zhipu_provider_arc();
        let llm2 = create_test_zhipu_provider_arc();

        let agent1 = Arc::new(BasicAgent::new(
            AgentConfig {
                name: "analyzer".to_string(),
                instructions: "Analyze the input and extract key points".to_string(),
                ..Default::default()
            },
            llm1,
        ));

        let agent2 = Arc::new(BasicAgent::new(
            AgentConfig {
                name: "summarizer".to_string(),
                instructions: "Summarize the analysis concisely".to_string(),
                ..Default::default()
            },
            llm2,
        ));

        let mut workflow = BasicWorkflow::new("multi_step_workflow");

        workflow.add_step(WorkflowStep {
            name: "analyze".to_string(),
            agent: agent1,
            instructions: "Analyze the input text".to_string(),
            condition: StepCondition::Always,
            timeout_ms: None,
            retry_count: None,
        });

        workflow.add_step(WorkflowStep {
            name: "summarize".to_string(),
            agent: agent2,
            instructions: "Summarize the analysis".to_string(),
            condition: StepCondition::StepCompleted("analyze".to_string()),
            timeout_ms: None,
            retry_count: None,
        });

        let input = json!({"text": "Machine learning is a subset of AI"});

        tokio::time::sleep(Duration::from_millis(1500)).await;

        let result =
            retry_with_backoff(|| async { workflow.execute(input.clone()).await }, 5, 2000).await;

        assert!(
            result.is_ok(),
            "Multi-step workflow failed: {:?}",
            result.err()
        );
        let output = result.unwrap();
        assert!(!output.is_null(), "Workflow output should not be null");
    }

    #[tokio::test]
    async fn test_workflow_with_description() {
        let llm = create_test_zhipu_provider_arc();
        let agent = Arc::new(BasicAgent::new(
            AgentConfig {
                name: "processor".to_string(),
                instructions: "Process the input".to_string(),
                ..Default::default()
            },
            llm,
        ));

        let workflow =
            BasicWorkflow::new("test_workflow").with_description("A test workflow for validation");

        assert_eq!(workflow.name(), "test_workflow");
        assert_eq!(
            workflow.description(),
            Some("A test workflow for validation")
        );
    }

    #[tokio::test]
    async fn test_workflow_steps_list() {
        let llm = create_test_zhipu_provider_arc();
        let agent = Arc::new(BasicAgent::new(
            AgentConfig {
                name: "test_agent".to_string(),
                instructions: "Test instructions".to_string(),
                ..Default::default()
            },
            llm,
        ));

        let mut workflow = BasicWorkflow::new("steps_test");

        workflow.add_step(WorkflowStep {
            name: "step1".to_string(),
            agent: agent.clone(),
            instructions: "First step".to_string(),
            condition: StepCondition::Always,
            timeout_ms: None,
            retry_count: None,
        });

        workflow.add_step(WorkflowStep {
            name: "step2".to_string(),
            agent: agent.clone(),
            instructions: "Second step".to_string(),
            condition: StepCondition::Always,
            timeout_ms: None,
            retry_count: None,
        });

        let steps = workflow.steps();
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0], "step1");
        assert_eq!(steps[1], "step2");
    }

    #[tokio::test]
    async fn test_workflow_condition_always() {
        let llm = create_test_zhipu_provider_arc();
        let agent = Arc::new(BasicAgent::new(
            AgentConfig {
                name: "test_agent".to_string(),
                instructions: "Always execute".to_string(),
                ..Default::default()
            },
            llm,
        ));

        let mut workflow = BasicWorkflow::new("condition_test");
        workflow.add_step(WorkflowStep {
            name: "always_step".to_string(),
            agent,
            instructions: "This step always executes".to_string(),
            condition: StepCondition::Always,
            timeout_ms: None,
            retry_count: None,
        });

        let input = json!({"test": "data"});

        tokio::time::sleep(Duration::from_millis(1000)).await;

        let result =
            retry_with_backoff(|| async { workflow.execute(input.clone()).await }, 5, 2000).await;

        assert!(result.is_ok(), "Workflow with Always condition failed");
    }

    #[tokio::test]
    async fn test_workflow_empty_input() {
        let llm = create_test_zhipu_provider_arc();
        let agent = Arc::new(BasicAgent::new(
            AgentConfig {
                name: "test_agent".to_string(),
                instructions: "Handle empty input".to_string(),
                ..Default::default()
            },
            llm,
        ));

        let mut workflow = BasicWorkflow::new("empty_input_test");
        workflow.add_step(WorkflowStep {
            name: "step1".to_string(),
            agent,
            instructions: "Process empty input".to_string(),
            condition: StepCondition::Always,
            timeout_ms: None,
            retry_count: None,
        });

        let input = json!({});

        tokio::time::sleep(Duration::from_millis(1000)).await;

        let result =
            retry_with_backoff(|| async { workflow.execute(input.clone()).await }, 5, 2000).await;

        assert!(result.is_ok(), "Workflow with empty input failed");
    }
}
