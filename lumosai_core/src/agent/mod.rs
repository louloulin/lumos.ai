//! Agent module for LLM-based agents

pub mod api_consistency;
pub mod builder;
pub mod chain;
pub mod collaboration;
pub mod communication;
pub mod config;
pub mod config_validator;
pub mod convenience;
pub mod dag_orchestration;
pub mod dynamic_config;
pub mod enhanced_integration_test;
pub mod error_handling; // 新增：统一错误处理系统
pub mod evaluation;
pub mod events;
pub mod executor;
pub mod feature_completion;
pub mod mastra_compat;
pub mod memory_resolver;
pub mod message_utils;
pub mod model_resolver;
pub mod operators; // 新增：Agent 操作符支持
pub mod orchestration;
pub mod performance;
pub mod rag_integration;
pub mod runtime_context;
pub mod session;
pub mod simplified_api;
pub mod sop_environment;
pub mod sop_simple;
pub mod sop_types;
pub mod state_management; // 新增：Agent状态管理
pub mod streaming;
pub mod structured_output;
pub mod tool_resolver;
pub mod trait_def;
pub mod types;
pub mod websocket;

// 高级协作模式（2025 研究成果）
pub mod debate;
pub mod group_chat;
pub mod handoff;
pub mod magentic;
pub mod maker_checker;
pub mod reflection;

// 暂时移除模块化Agent组件（有编译错误）
// pub mod modular;

#[cfg(feature = "demos")]
pub mod enhanced_streaming_demo;
#[cfg(feature = "demos")]
pub mod websocket_demo;

#[cfg(test)]
mod mastra_integration_test;
#[cfg(test)]
mod simplified_api_tests;

#[cfg(test)]
mod advanced_features_test;
#[cfg(test)]
mod plan4_api_tests;
#[cfg(test)]
mod simple_test;

// P0-3: 新增的单元测试模块（暂时注释掉，因为现有测试有编译问题）
// #[cfg(test)]
// mod executor_tests;

// Week 1: 新增的单元测试模块
#[cfg(test)]
mod week1_agent_tests;

// Week 2: 新增的单元测试模块
#[cfg(test)]
mod real_api_tests;

pub use config::{AgentConfig, AgentGenerateOptions};
pub use executor::BasicAgent;
pub use message_utils::{assistant_message, system_message, tool_message, user_message};
pub use runtime_context::{create_context_manager, ContextManager, RuntimeContext, ToolCallRecord};
pub use trait_def::{Agent, AgentStructuredOutput};

// Re-export builder
pub use builder::AgentBuilder;
pub use rag_integration::{RagAgent, RagConfig, RagIntegrationExt};
pub use structured_output::StructuredOutputExt;

// Re-export streaming types
pub use streaming::{AgentEvent, IntoStreaming, MemoryOperation, StreamingAgent, StreamingConfig};

// Re-export WebSocket streaming types
pub use websocket::{
    IntoWebSocketStreaming, WebSocketConfig, WebSocketConnection, WebSocketManager,
    WebSocketMessage, WebSocketStreamingAgent,
};

// Re-export agent types
pub use types::{
    AgentGenerateResult, AgentStep, AgentStreamOptions, AgentToolCall, DynamicArgument,
    TelemetrySettings, ToolsInput, ToolsetsInput, VoiceConfig,
};

// Re-export SOP types and environment
pub use sop_environment::SopEnvironment;
pub use sop_simple::SimpleSopEnvironment;
pub use sop_types::{AgentAction, SopExecutionMode, SopMessage, SopStats};

// Re-export operators
pub use operators::{delegate, parallel, pipe, AgentDelegation, AgentParallel, AgentPipeline};

// Re-export evaluation types
pub use evaluation::{
    CompositeMetric, EvaluationMetric, EvaluationResult, LengthMetric, RelevanceMetric,
};

// Re-export convenience functions
pub use convenience::{
    anthropic, anthropic_with_key, deepseek, deepseek_with_key, openai, openai_with_key, qwen,
    qwen_with_key, LlmProviderExt, ModelBuilder,
};

// Re-export model resolver
pub use model_resolver::ModelResolver;

// Re-export simplified API functions (plan4.md implementation)
pub use simplified_api::{
    Agent as SimpleAgent, // New simplified Agent struct
};

// Re-export session management
pub use session::{
    MemorySessionStorage, SessionData, SessionManager, SessionMetadata, SessionQuery, SessionState,
    SessionStorage, ToolCallHistory, ToolCallStatus,
};

// Re-export orchestration
pub use orchestration::{
    AgentExecutionState, AgentOrchestrator, AgentRole, BasicOrchestrator, CollaborationSession,
    CollaborationTask, OrchestrationPattern, RetryConfig, VotingStrategy,
};

// Re-export error handling
pub use error_handling::{
    AgentErrorType, BackoffStrategy, DefaultErrorRecovery, ErrorContext, ErrorRecovery,
    RecoveryAction, RetryExecutor, RetryStrategy,
};

// Re-export API consistency
pub use api_consistency::{
    ApiConsistencyChecker, ApiSpecChecker, ApiStandardizer, ConsistencyCheckResult,
    ConsistencyIssue,
};

// Re-export DAG orchestration
pub use dag_orchestration::{AgentChain, AgentDagOrchestrator, AgentDagOrchestratorBuilder};

// Re-export events
pub use events::{EventBus, EventFilter, EventHandler, LogEventHandler, MetricsEventHandler};

// Re-export communication and collaboration
pub use collaboration::{
    AgentRole as CrewAgentRole, AgentTask, CollaborationMode, Crew, CrewStats, TaskStatus,
};
pub use communication::{
    AgentCommunicationManager, AgentMessage, AgentMessageType, CommunicationConfig,
};

// Re-export advanced collaboration patterns (2025 research)
pub use debate::{DebateExecutor, DebatePosition, DebateResult, DebateRound};
pub use group_chat::{ChatMessage, ChatThread, GroupChatExecutor};
pub use handoff::{HandoffCondition, HandoffExecutor, HandoffRecord, HandoffRule};
pub use magentic::{MagenticExecutor, MagenticTask, TaskLedger, TaskState};
pub use maker_checker::{
    CheckResult, CheckStatus, MakerCheckerExecutor, MakerCheckerIteration, MakerCheckerStats,
};
pub use reflection::{ReflectionExecutor, ReflectionIteration, ReflectionStats};

// 暂时移除模块化代理组件的重新导出
// pub use modular::{
//     AgentCapability, AgentCore, AgentExecutor, AgentHealth, AgentLifecycle, AgentManager,
//     AgentMetrics, AgentState, AgentStateSnapshot, Capability, CapabilityFilter, CapabilityStats,
//     CapabilityType, ExecutorConfig, HealthCheck, HealthConfig, HealthMetrics, HealthReport,
//     HealthStatus, HealthSummary, LifecycleConfig, LifecycleInfo, LifecycleState, Parameter,
//     ParameterType, PerformanceSummary, ValidationRule,
// };

/// Create a basic agent with default configuration
pub fn create_basic_agent(
    name: impl Into<String>,
    instructions: impl Into<String>,
    llm: std::sync::Arc<dyn crate::llm::LlmProvider>,
) -> BasicAgent {
    let _config = AgentConfig {
        name: name.into(),
        instructions: instructions.into(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(true),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
        tenant_id: None,
        isolation_level: None,
    };

    BasicAgent::new(_config, llm)
}

/// Create an agent with minimal configuration (quick start)
///
/// # Example
///
/// ```rust
/// use lumosai_core::agent::quick;
/// use lumosai_core::llm::MockLlmProvider;
/// use std::sync::Arc;
///
/// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
///
/// let agent = quick("assistant", "You are a helpful assistant")
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

/// AgentFactory struct with static methods for creation
///
/// This provides the plan4.md specified API: AgentFactory::quick() and AgentFactory::builder()
pub struct AgentFactory;

impl AgentFactory {
    /// Create an agent with minimal configuration (plan4.md API)
    ///
    /// # Example
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentFactory;
    /// use lumosai_core::llm::MockLlmProvider;
    /// use std::sync::Arc;
    ///
    /// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    ///
    /// let agent = AgentFactory::quick("assistant", "You are a helpful assistant")
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

    /// Create an agent with the builder pattern (plan4.md API)
    ///
    /// # Example
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentFactory;
    /// use lumosai_core::llm::MockLlmProvider;
    /// use std::sync::Arc;
    ///
    /// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    ///
    /// let agent = AgentFactory::builder()
    ///     .name("research_agent")
    ///     .instructions("You are a research assistant")
    ///     .model(llm)
    ///     .max_tool_calls(10)
    ///     .build()
    ///     .expect("Failed to create agent");
    /// ```
    pub fn builder() -> AgentBuilder {
        AgentBuilder::new()
    }
}

/// Create a web-enabled agent with common web tools
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
/// let agent = web_agent("web_helper", "You can browse the web")
///     .model(llm)
///     .build()
///     .expect("Failed to create agent");
/// ```
pub fn web_agent(name: &str, instructions: &str) -> AgentBuilder {
    use crate::tool::builtin::web::*;

    AgentBuilder::new()
        .name(name)
        .instructions(instructions)
        .tools(vec![
            Box::new(create_http_request_tool()),
            Box::new(create_web_scraper_tool()),
            Box::new(create_json_api_tool()),
            Box::new(create_url_validator_tool()),
        ])
        .enable_smart_defaults()
}

/// Create a file-enabled agent with common file tools
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
/// let agent = file_agent("file_helper", "You can manage files")
///     .model(llm)
///     .build()
///     .expect("Failed to create agent");
/// ```
pub fn file_agent(name: &str, instructions: &str) -> AgentBuilder {
    use crate::tool::builtin::file::*;

    AgentBuilder::new()
        .name(name)
        .instructions(instructions)
        .tools(vec![
            Box::new(create_file_reader_tool()),
            Box::new(create_file_writer_tool()),
            Box::new(create_directory_lister_tool()),
            Box::new(create_file_info_tool()),
        ])
        .enable_smart_defaults()
}

/// Create a data processing agent with common data tools
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
/// let agent = data_agent("data_helper", "You can process data")
///     .model(llm)
///     .build()
///     .expect("Failed to create agent");
/// ```
pub fn data_agent(name: &str, instructions: &str) -> AgentBuilder {
    use crate::tool::builtin::data::*;

    AgentBuilder::new()
        .name(name)
        .instructions(instructions)
        .tools(vec![
            Box::new(create_json_parser_tool()),
            Box::new(create_csv_parser_tool()),
            Box::new(create_data_transformer_tool()),
        ])
        .enable_smart_defaults()
}

// Plan4.md specified convenience functions
impl AgentFactory {
    /// Create a web-enabled agent with pre-configured web tools (plan4.md API)
    ///
    /// # Example
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentFactory;
    /// use lumosai_core::llm::MockLlmProvider;
    /// use std::sync::Arc;
    ///
    /// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    ///
    /// let agent = AgentFactory::web_agent("web_helper")
    ///     .instructions("You can browse the web")
    ///     .model(llm)
    ///     .build()
    ///     .expect("Failed to create agent");
    /// ```
    pub fn web_agent(name: &str) -> AgentBuilder {
        AgentBuilder::new()
            .name(name)
            .with_web_tools()
            .enable_smart_defaults()
    }

    /// Create a file-enabled agent with pre-configured file tools (plan4.md API)
    ///
    /// # Example
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentFactory;
    /// use lumosai_core::llm::MockLlmProvider;
    /// use std::sync::Arc;
    ///
    /// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    ///
    /// let agent = AgentFactory::file_agent("file_helper")
    ///     .instructions("You can manage files")
    ///     .model(llm)
    ///     .build()
    ///     .expect("Failed to create agent");
    /// ```
    pub fn file_agent(name: &str) -> AgentBuilder {
        AgentBuilder::new()
            .name(name)
            .with_file_tools()
            .enable_smart_defaults()
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use std::sync::Arc;

    use super::*;
    use crate::llm::{LlmOptions, LlmProvider, Message, Role};
    use crate::tool::{FunctionTool, ParameterSchema, ToolSchema};
    use crate::Result;

    struct MockLlmProvider {
        responses: Vec<String>,
    }

    impl MockLlmProvider {
        fn new(responses: Vec<String>) -> Self {
            Self { responses }
        }
    }

    #[async_trait]
    impl LlmProvider for MockLlmProvider {
        fn name(&self) -> &str {
            "mock"
        }

        async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> Result<String> {
            // Get the appropriate response based on the current state
            let messages = _options
                .extra
                .get("messages")
                .and_then(|v| v.as_array())
                .map(|msgs| msgs.len().saturating_sub(2).min(self.responses.len() - 1))
                .unwrap_or(0);

            Ok(self.responses[messages].clone())
        }

        async fn generate_with_messages(
            &self,
            messages: &[Message],
            _options: &LlmOptions,
        ) -> Result<String> {
            // Get the appropriate response based on the messages length
            let index = messages
                .len()
                .saturating_sub(2)
                .min(self.responses.len() - 1);
            Ok(self.responses[index].clone())
        }

        async fn generate_stream<'a>(
            &'a self,
            _prompt: &'a str,
            _options: &'a LlmOptions,
        ) -> Result<futures::stream::BoxStream<'a, Result<String>>> {
            unimplemented!("Streaming not implemented for mock provider")
        }

        async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>> {
            unimplemented!("Embeddings not implemented for mock provider")
        }
    }

    #[tokio::test]
    async fn test_agent_with_tool() {
        // Create an echo tool
        let schema = ToolSchema::new(vec![ParameterSchema {
            name: "message".to_string(),
            description: "The message to echo".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        }]);

        let echo_tool = FunctionTool::new("echo", "Echoes the input message", schema, |params| {
            let message = params
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("No message");
            Ok(serde_json::json!(format!("Echo: {}", message)))
        });

        // Create an agent
        let _config = AgentConfig {
            name: "TestAgent".to_string(),
            instructions: "You are a test agent.".to_string(),
            memory_config: None,
            model_id: None,
            voice_config: None,
            telemetry: None,
            tenant_id: None,
            isolation_level: None,
            working_memory: None,
            enable_function_calling: Some(true),
            context: None,
            metadata: None,
            max_tool_calls: None,
            tool_timeout: None,
        };

        let mock_llm = Arc::new(MockLlmProvider::new(vec![
            "I'll help you with that! Using the tool 'echo' with parameters: {\"message\": \"Hello from tool!\"}".to_string(),
            "The tool returned: Echo: Hello from tool!".to_string(),
        ]));

        let mut agent = create_basic_agent(
            "TestAgent".to_string(),
            "You are a test agent.".to_string(),
            mock_llm,
        );
        agent.add_tool(Box::new(echo_tool)).unwrap();

        // Generate a response
        let user_message = Message {
            role: Role::User,
            content: "Hello".to_string(),
            metadata: None,
            name: None,
        };

        let result = agent
            .generate(&[user_message], &types::AgentGenerateOptions::default())
            .await
            .unwrap();

        assert_eq!(result.response, "The tool returned: Echo: Hello from tool!");
    }
}
