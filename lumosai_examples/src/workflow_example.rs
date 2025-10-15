use async_trait::async_trait;
use futures::stream::BoxStream;
use lumosai_core::agent::{create_basic_agent, AgentConfig};
use lumosai_core::error::Result as LumusResult;
use lumosai_core::llm::{LlmOptions, LlmProvider, Message};
use lumosai_core::workflow::basic::Workflow;
use lumosai_core::workflow::basic::{BasicWorkflow, StepCondition, WorkflowStep};
use serde_json::json;
use std::sync::Arc;

// 创建一个模拟LLM提供者
struct MockLlmProvider {
    response: String,
}

impl MockLlmProvider {
    fn new(response: String) -> Self {
        Self { response }
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> LumusResult<String> {
        Ok(self.response.clone())
    }

    async fn generate_with_messages(
        &self,
        _messages: &[Message],
        _options: &LlmOptions,
    ) -> LumusResult<String> {
        Ok(self.response.clone())
    }

    async fn generate_stream<'a>(
        &'a self,
        _prompt: &'a str,
        _options: &'a LlmOptions,
    ) -> LumusResult<BoxStream<'a, LumusResult<String>>> {
        unimplemented!("Stream not implemented for MockLlmProvider")
    }

    async fn get_embedding(&self, _text: &str) -> LumusResult<Vec<f32>> {
        unimplemented!("Embedding not implemented for MockLlmProvider")
    }
}

#[tokio::main]
async fn main() -> LumusResult<()> {
    println!("Lumosai Workflow Example");
    println!("=======================");

    // 创建各个步骤的模拟LLM提供者
    let extract_provider = Arc::new(MockLlmProvider::new(
        json!({
            "data": {
                "source": "customer_database",
                "records": 150,
                "fields": ["name", "email", "purchase_history"]
            }
        })
        .to_string(),
    ));

    let transform_provider = Arc::new(MockLlmProvider::new(
        json!({
            "transformed_data": {
                "valid_records": 142,
                "invalid_records": 8,
                "normalized_fields": ["customer_name", "customer_email", "purchases"]
            }
        })
        .to_string(),
    ));

    let analyze_provider = Arc::new(MockLlmProvider::new(
        json!({
            "analysis_results": {
                "customer_segments": ["high_value", "medium_value", "low_value"],
                "avg_purchase_value": 120.5,
                "purchase_frequency": "monthly"
            }
        })
        .to_string(),
    ));

    let load_provider = Arc::new(MockLlmProvider::new(
        json!({
            "load_status": "success",
            "destination": "analytics_database",
            "records_loaded": 142,
            "timestamp": "2023-11-01T14:30:00Z"
        })
        .to_string(),
    ));

    // 创建各个步骤的代理
    let extract_agent = Arc::new(create_basic_agent(
        "extract_agent".to_string(),
        "请从给定的数据中提取关键信息。".to_string(),
        extract_provider,
    ));

    let transform_agent = Arc::new(create_basic_agent(
        "transform_agent".to_string(),
        "请对提取的数据进行转换和处理。".to_string(),
        transform_provider,
    ));

    let analyze_agent = Arc::new(create_basic_agent(
        "analyze_agent".to_string(),
        "请分析处理后的数据并提供洞察。".to_string(),
        analyze_provider,
    ));

    let load_agent = Arc::new(create_basic_agent(
        "load_agent".to_string(),
        "请将分析结果加载到目标位置。".to_string(),
        load_provider,
    ));

    // 创建一个基本工作流
    let mut workflow = BasicWorkflow::new("etl_workflow".to_string());

    // 添加步骤
    workflow.add_step(WorkflowStep {
        name: "extract".to_string(),
        agent: extract_agent,
        instructions: "Extract data from the input".to_string(),
        condition: StepCondition::Always,
        timeout_ms: None,
        retry_count: None,
    });

    workflow.add_step(WorkflowStep {
        name: "transform".to_string(),
        agent: transform_agent,
        instructions: "Transform the extracted data".to_string(),
        condition: StepCondition::StepCompleted("extract".to_string()),
        timeout_ms: None,
        retry_count: None,
    });

    workflow.add_step(WorkflowStep {
        name: "analyze".to_string(),
        agent: analyze_agent,
        instructions: "Analyze the transformed data".to_string(),
        condition: StepCondition::StepCompleted("transform".to_string()),
        timeout_ms: None,
        retry_count: None,
    });

    workflow.add_step(WorkflowStep {
        name: "load".to_string(),
        agent: load_agent,
        instructions: "Load the analyzed data".to_string(),
        condition: StepCondition::StepCompleted("analyze".to_string()),
        timeout_ms: None,
        retry_count: None,
    });

    println!("\n🚀 Starting workflow: {}", workflow.name());

    // 执行工作流
    let trigger_data = json!({
        "job_id": "example-001",
        "source": "user_upload",
        "data": "This is example text for processing.",
        "timestamp": "2023-08-15T14:00:00Z"
    });

    let result = workflow.execute(trigger_data).await?;

    // 显示结果
    println!("\n✅ Workflow completed!");
    println!("Final result: {}", result);

    println!("\nWorkflow execution complete!");

    Ok(())
}
