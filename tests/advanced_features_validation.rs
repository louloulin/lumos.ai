use std::time::Duration;
use std::collections::HashMap;
use lumosai_core::error::Error;
use lumosai_core::agent::{
    performance::{PerformanceMonitor, PerformanceAnalyzer},
    api_consistency::ApiConsistencyChecker,
    feature_completion::FeatureCompletenessChecker,
    trait_def::{Agent, AgentStatus},
    config::AgentConfig,
    BasicAgent,
};
use lumosai_core::llm::{LlmProvider, LlmOptions, Message, Role};
use lumosai_core::tool::{Tool, FunctionTool, ParameterSchema, ToolSchema};
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::json;

// Mock LLM Provider for testing
struct MockLlmProvider {
    responses: Vec<String>,
    current_index: std::sync::Mutex<usize>,
}

impl MockLlmProvider {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            current_index: std::sync::Mutex::new(0),
        }
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    fn name(&self) -> &str {
        "mock-llm"
    }

    async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> Result<String, Error> {
        let mut index = self.current_index.lock().unwrap();
        let response = self.responses.get(*index).unwrap_or(&"Default response".to_string()).clone();
        *index = (*index + 1) % self.responses.len();
        Ok(response)
    }

    async fn generate_with_messages(&self, _messages: &[Message], _options: &LlmOptions) -> Result<String, Error> {
        let mut index = self.current_index.lock().unwrap();
        let response = self.responses.get(*index).unwrap_or(&"Default response".to_string()).clone();
        *index = (*index + 1) % self.responses.len();
        Ok(response)
    }

    async fn generate_stream<'a>(
        &'a self,
        _prompt: &'a str,
        _options: &'a LlmOptions,
    ) -> Result<futures::stream::BoxStream<'a, Result<String, Error>>, Error> {
        use futures::stream::{self, StreamExt};
        let response = self.generate(_prompt, _options).await?;
        let chunks: Vec<Result<String, Error>> = response
            .chars()
            .map(|c| Ok(c.to_string()))
            .collect();
        Ok(stream::iter(chunks).boxed())
    }

    async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>, Error> {
        Ok(vec![0.1, 0.2, 0.3, 0.4, 0.5])
    }
}

/// 测试性能监控功能
#[tokio::test]
async fn test_performance_monitoring() {
    let monitor = PerformanceMonitor::new();
    
    // 模拟多个请求
    for i in 0..5 {
        let timer = monitor.start_request();
        
        // 模拟处理时间
        tokio::time::sleep(Duration::from_millis(50 + i * 10)).await;
        
        if i % 4 == 0 {
            timer.finish_error(); // 模拟一个错误
        } else {
            timer.finish_success();
        }
    }
    
    // 更新系统指标
    monitor.update_memory_usage(512_000_000).unwrap(); // 512MB
    monitor.update_cpu_usage(25.5).unwrap();
    monitor.update_cache_hit_rate(75.0).unwrap();
    
    // 获取指标
    let metrics = monitor.get_metrics().unwrap();
    
    assert_eq!(metrics.total_requests, 5);
    assert_eq!(metrics.successful_requests, 4);
    assert_eq!(metrics.failed_requests, 1);
    assert!(metrics.avg_response_time > 0.0);
    assert_eq!(metrics.memory_usage, 512_000_000);
    assert_eq!(metrics.cpu_usage, 25.5);
    assert_eq!(metrics.cache_hit_rate, 75.0);
    
    // 测试性能分析
    let recommendations = PerformanceAnalyzer::analyze(&metrics);
    assert!(!recommendations.is_empty(), "Should provide performance recommendations");
    
    println!("Performance metrics: {:?}", metrics);
    println!("Recommendations: {:?}", recommendations);
}

/// 测试API一致性检查
#[tokio::test]
async fn test_api_consistency_check() {
    // 创建一个测试Agent
    let config = AgentConfig {
        name: "TestAgent".to_string(),
        instructions: "You are a helpful test assistant.".to_string(),
        memory_config: None,
        model_id: Some("mock-model".to_string()),
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(true),
        context: None,
        metadata: Some(HashMap::from([
            ("version".to_string(), "1.0.0".to_string()),
            ("type".to_string(), "test".to_string()),
        ])),
        max_tool_calls: Some(5),
        tool_timeout: Some(Duration::from_secs(30)),
    };
    
    let llm = Arc::new(MockLlmProvider::new(vec![
        "Hello! I'm ready to help you.".to_string(),
        "I understand your request.".to_string(),
        "Here's the information you requested.".to_string(),
    ]));
    
    let mut agent = BasicAgent::new(config, llm);
    
    // 添加一个测试工具
    let echo_tool = create_echo_tool();
    agent.add_tool(Box::new(echo_tool)).unwrap();
    
    // 执行一致性检查
    let result = ApiConsistencyChecker::check_agent_consistency(&agent).await;
    
    assert!(result.score > 0.5, "API consistency score should be reasonable");
    assert!(!result.recommendations.is_empty(), "Should provide recommendations");
    
    println!("API Consistency Result: {:?}", result);
    
    // 检查具体的一致性方面
    let has_critical_issues = result.issues.iter().any(|issue| issue.severity == "critical");
    assert!(!has_critical_issues, "Should not have critical consistency issues");
}

/// 测试功能完整性检查
#[tokio::test]
async fn test_feature_completeness_check() {
    // 创建一个功能丰富的测试Agent
    let config = AgentConfig {
        name: "FeatureRichAgent".to_string(),
        instructions: "You are a comprehensive test assistant with many capabilities.".to_string(),
        memory_config: None,
        model_id: Some("advanced-model".to_string()),
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(true),
        context: None,
        metadata: Some(HashMap::from([
            ("version".to_string(), "2.0.0".to_string()),
            ("capabilities".to_string(), "advanced".to_string()),
        ])),
        max_tool_calls: Some(10),
        tool_timeout: Some(Duration::from_secs(60)),
    };
    
    let llm = Arc::new(MockLlmProvider::new(vec![
        "I'll solve this step by step: First, I need to understand the problem...".to_string(),
        "Step 1: Analyze the input. Step 2: Process the data. Step 3: Generate the result.".to_string(),
        r#"{"name": "John Doe", "age": 30, "city": "New York"}"#.to_string(),
        "I can help you with various tasks using my available tools.".to_string(),
    ]));
    
    let mut agent = BasicAgent::new(config, llm);
    
    // 添加多个工具来提高功能完整性
    agent.add_tool(Box::new(create_echo_tool())).unwrap();
    agent.add_tool(Box::new(create_calculator_tool())).unwrap();
    agent.add_tool(Box::new(create_json_tool())).unwrap();
    
    // 执行功能完整性检查
    let result = FeatureCompletenessChecker::check_completeness(&agent).await;
    
    assert!(result.overall_score > 0.3, "Feature completeness score should be reasonable");
    assert!(!result.implemented_features.is_empty(), "Should have some implemented features");
    assert!(!result.recommendations.is_empty(), "Should provide improvement recommendations");
    
    println!("Feature Completeness Result: {:?}", result);
    
    // 检查核心功能是否被识别
    let has_basic_generation = result.implemented_features.iter()
        .any(|feature| feature.contains("Generation"));
    assert!(has_basic_generation, "Should recognize basic generation capability");
}

/// 测试Agent状态管理
#[tokio::test]
async fn test_agent_status_management() {
    let config = AgentConfig {
        name: "StatusTestAgent".to_string(),
        instructions: "Test agent for status management.".to_string(),
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
    };
    
    let llm = Arc::new(MockLlmProvider::new(vec![
        "Agent is ready to work.".to_string(),
    ]));
    
    let mut agent = BasicAgent::new(config, llm);
    
    // 测试默认状态
    let initial_status = agent.get_status();
    assert_eq!(initial_status, AgentStatus::Ready);
    
    // 测试状态设置
    agent.set_status(AgentStatus::Running).unwrap();
    let running_status = agent.get_status();
    assert_eq!(running_status, AgentStatus::Running);
    
    // 测试错误状态
    agent.set_status(AgentStatus::Error("Test error".to_string())).unwrap();
    let error_status = agent.get_status();
    match error_status {
        AgentStatus::Error(msg) => assert_eq!(msg, "Test error"),
        _ => panic!("Expected error status"),
    }
    
    // 测试重置
    agent.reset().await.unwrap();
    let reset_status = agent.get_status();
    assert_eq!(reset_status, AgentStatus::Ready);
}

/// 测试Agent健康检查
#[tokio::test]
async fn test_agent_health_check() {
    let config = AgentConfig {
        name: "HealthTestAgent".to_string(),
        instructions: "Test agent for health monitoring.".to_string(),
        memory_config: None,
        model_id: Some("health-model".to_string()),
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(true),
        context: None,
        metadata: Some(HashMap::from([
            ("version".to_string(), "1.0.0".to_string()),
        ])),
        max_tool_calls: None,
        tool_timeout: None,
    };
    
    let llm = Arc::new(MockLlmProvider::new(vec![
        "Health check response.".to_string(),
    ]));
    
    let mut agent = BasicAgent::new(config, llm);
    agent.add_tool(Box::new(create_echo_tool())).unwrap();
    
    // 执行健康检查
    let health = agent.health_check().await.unwrap();
    
    assert!(!health.is_empty(), "Health check should return information");
    assert!(health.contains_key("status"), "Should include status");
    assert!(health.contains_key("name"), "Should include name");
    assert!(health.contains_key("has_memory"), "Should include memory info");
    assert!(health.contains_key("tools_count"), "Should include tools count");
    
    println!("Health check result: {:?}", health);
}

/// 综合集成测试
#[tokio::test]
async fn test_comprehensive_integration() {
    // 创建性能监控器
    let monitor = PerformanceMonitor::new();
    
    // 创建Agent
    let config = AgentConfig {
        name: "ComprehensiveTestAgent".to_string(),
        instructions: "You are a comprehensive test agent with all features enabled.".to_string(),
        memory_config: None,
        model_id: Some("comprehensive-model".to_string()),
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(true),
        context: None,
        metadata: Some(HashMap::from([
            ("version".to_string(), "3.0.0".to_string()),
            ("environment".to_string(), "test".to_string()),
        ])),
        max_tool_calls: Some(15),
        tool_timeout: Some(Duration::from_secs(120)),
    };
    
    let llm = Arc::new(MockLlmProvider::new(vec![
        "I'm ready to demonstrate all my capabilities.".to_string(),
        "Let me process this step by step...".to_string(),
        "Here's the comprehensive result you requested.".to_string(),
    ]));
    
    let mut agent = BasicAgent::new(config, llm);
    
    // 添加多种工具
    agent.add_tool(Box::new(create_echo_tool())).unwrap();
    agent.add_tool(Box::new(create_calculator_tool())).unwrap();
    agent.add_tool(Box::new(create_json_tool())).unwrap();
    
    // 执行多项检查
    let timer = monitor.start_request();
    
    // 1. 健康检查
    let health = agent.health_check().await.unwrap();
    assert!(!health.is_empty());
    
    // 2. API一致性检查
    let consistency = ApiConsistencyChecker::check_agent_consistency(&agent).await;
    assert!(consistency.score > 0.5);
    
    // 3. 功能完整性检查
    let completeness = FeatureCompletenessChecker::check_completeness(&agent).await;
    assert!(completeness.overall_score > 0.3);
    
    // 4. 基本功能测试
    let test_message = vec![Message {
        role: Role::User,
        content: "Hello, please demonstrate your capabilities.".to_string(),
        name: None,
        metadata: None,
    }];
    
    let response = agent.generate(&test_message, &Default::default()).await.unwrap();
    assert!(!response.response.is_empty());
    
    timer.finish_success();
    
    // 验证性能指标
    let metrics = monitor.get_metrics().unwrap();
    assert_eq!(metrics.total_requests, 1);
    assert_eq!(metrics.successful_requests, 1);
    
    println!("✅ Comprehensive integration test passed!");
    println!("Health: {:?}", health);
    println!("Consistency Score: {:.2}", consistency.score);
    println!("Completeness Score: {:.2}", completeness.overall_score);
    println!("Performance: {:?}", metrics);
}

// Helper functions to create test tools

fn create_echo_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "message".to_string(),
            description: "The message to echo".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);
    
    FunctionTool::new(
        "echo",
        "Echoes the input message",
        schema,
        |params| {
            let message = params.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("No message");
            Ok(json!(format!("Echo: {}", message)))
        },
    )
}

fn create_calculator_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "expression".to_string(),
            description: "Mathematical expression to evaluate".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);
    
    FunctionTool::new(
        "calculator",
        "Evaluates mathematical expressions",
        schema,
        |params| {
            let expr = params.get("expression")
                .and_then(|v| v.as_str())
                .unwrap_or("0");
            // Simple calculation for testing
            let result = if expr == "2+2" { 4 } else { 42 };
            Ok(json!({"result": result, "expression": expr}))
        },
    )
}

fn create_json_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "data".to_string(),
            description: "Data to format as JSON".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);
    
    FunctionTool::new(
        "json_formatter",
        "Formats data as JSON",
        schema,
        |params| {
            let data = params.get("data")
                .and_then(|v| v.as_str())
                .unwrap_or("{}");
            Ok(json!({"formatted": data, "type": "json"}))
        },
    )
}
