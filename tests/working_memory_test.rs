use lomusai_core::{
    agent::{Agent, AgentConfig, BasicAgent, create_basic_agent, AgentGenerateOptions},
    llm::{Message, Role},
    memory::{WorkingMemory, WorkingMemoryConfig, WorkingMemoryContent},
    tool::Tool,
    Result,
};
use serde_json::{Value, json};
use std::sync::Arc;
use async_trait::async_trait;

// 用于测试的Mock LLM Provider
struct MockLlmProvider;

#[async_trait]
impl lomusai_core::llm::LlmProvider for MockLlmProvider {
    async fn generate(&self, _prompt: &str, _options: &lomusai_core::llm::LlmOptions) -> Result<String> {
        Ok("WORKING_MEMORY_UPDATE:{\"test_key\":\"updated_value\"}WORKING_MEMORY_END".to_string())
    }

    async fn generate_with_messages(&self, _messages: &[Message], _options: &lomusai_core::llm::LlmOptions) -> Result<String> {
        Ok("WORKING_MEMORY_UPDATE:{\"test_key\":\"updated_value\"}WORKING_MEMORY_END".to_string())
    }

    async fn generate_stream<'a>(
        &'a self,
        _prompt: &'a str,
        _options: &'a lomusai_core::llm::LlmOptions,
    ) -> Result<futures::stream::BoxStream<'a, Result<String>>> {
        unimplemented!("Stream not needed for this test")
    }

    async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        unimplemented!("Embedding not needed for this test")
    }
}

#[tokio::test]
async fn test_agent_with_working_memory() -> Result<()> {
    // 创建工作内存配置
    let working_memory_config = WorkingMemoryConfig {
        enabled: true,
        template: Some(r#"{"initial_key": "initial_value"}"#.to_string()),
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1024),
    };

    // 创建Agent配置
    let agent_config = AgentConfig {
        name: "TestAgent".to_string(),
        instructions: "You are a test agent. Use working memory to store state.".to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: Some(working_memory_config),
    };

    // 创建Agent
    let llm_provider = Arc::new(MockLlmProvider);
    let agent = create_basic_agent(agent_config, llm_provider);

    // 测试生成
    let user_message = Message {
        role: Role::User,
        content: "Update the working memory".to_string(),
        metadata: None,
        name: None,
    };

    let options = AgentGenerateOptions::default();
    let result = agent.generate(&[user_message], &options).await?;

    // 验证工作内存是否被更新
    let working_memory = agent.get_working_memory().unwrap();
    let memory_content = working_memory.get().await?;
    
    if let Some(test_value) = memory_content.content.get("test_key") {
        assert_eq!(test_value, &json!("updated_value"));
        println!("Working memory updated successfully");
    } else {
        panic!("Working memory not updated correctly");
    }

    Ok(())
} 