//! 多语言绑定集成测试
//! 
//! 测试各种语言绑定的功能和兼容性

use lumosai_bindings::core::*;
use lumosai_bindings::error::*;
use lumosai_bindings::types::*;

#[tokio::test]
async fn test_cross_lang_agent_creation() {
    let builder = CrossLangAgentBuilder::new()
        .name("test_agent")
        .instructions("你是一个测试助手")
        .model("test_model");
    
    let agent = builder.build().unwrap();
    let config = agent.get_config();
    
    assert_eq!(config.model.name, "test_model");
    assert_eq!(config.runtime.timeout_seconds, 30);
    assert!(config.runtime.enable_logging);
}

#[tokio::test]
async fn test_cross_lang_agent_generate() {
    let builder = CrossLangAgentBuilder::new()
        .name("test_agent")
        .instructions("你是一个测试助手")
        .model("test_model");
    
    let agent = builder.build().unwrap();
    
    // 测试同步生成
    let response = agent.generate("Hello, world!");
    assert!(response.is_ok());
    
    let response = response.unwrap();
    assert!(!response.content.is_empty());
    assert!(matches!(response.response_type, ResponseType::Text | ResponseType::Error));
}

#[tokio::test]
async fn test_cross_lang_agent_generate_async() {
    let builder = CrossLangAgentBuilder::new()
        .name("test_agent")
        .instructions("你是一个测试助手")
        .model("test_model");
    
    let agent = builder.build_async().await.unwrap();
    
    // 测试异步生成
    let response = agent.generate_async("Hello, async world!").await;
    assert!(response.is_ok());
    
    let response = response.unwrap();
    assert!(!response.content.is_empty());
}

#[test]
fn test_cross_lang_tool_creation() {
    let metadata = ToolMetadata {
        name: "test_tool".to_string(),
        description: "测试工具".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "输入参数"
                }
            },
            "required": ["input"]
        }),
        tool_type: "test".to_string(),
        is_async: false,
    };
    
    // 创建模拟工具
    let tool = lumosai_core::tools::math::calculator();
    let cross_lang_tool = CrossLangTool::new(tool, metadata);
    
    let tool_metadata = cross_lang_tool.metadata();
    assert_eq!(tool_metadata.name, "test_tool");
    assert_eq!(tool_metadata.tool_type, "test");
    assert!(!tool_metadata.is_async);
}

#[test]
fn test_cross_lang_tool_execution() {
    let metadata = ToolMetadata {
        name: "calculator".to_string(),
        description: "计算器工具".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "数学表达式"
                }
            },
            "required": ["expression"]
        }),
        tool_type: "math".to_string(),
        is_async: false,
    };
    
    let tool = lumosai_core::tools::math::calculator();
    let cross_lang_tool = CrossLangTool::new(tool, metadata);
    
    let parameters = serde_json::json!({
        "expression": "2 + 2"
    });
    
    let result = cross_lang_tool.execute(parameters);
    assert!(result.is_ok());
    
    let result = result.unwrap();
    assert_eq!(result.tool_name, "calculator");
    assert!(result.success);
    assert!(result.execution_time_ms > 0);
}

#[test]
fn test_error_handling() {
    // 测试各种错误类型
    let core_error = BindingError::core("Test core error");
    assert_eq!(core_error.error_code(), "CORE_ERROR");
    assert!(!core_error.is_retryable());
    
    let network_error = BindingError::network("Test network error");
    assert_eq!(network_error.error_code(), "NETWORK_ERROR");
    assert!(network_error.is_retryable());
    
    let timeout_error = BindingError::timeout(30);
    assert_eq!(timeout_error.error_code(), "TIMEOUT_ERROR");
    assert!(timeout_error.is_retryable());
}

#[test]
fn test_error_context() {
    let error = BindingError::configuration("model", "Invalid model name");
    let context = error.context();
    
    assert_eq!(context.error_code, "CONFIGURATION_ERROR");
    assert!(context.message.contains("model"));
    assert!(context.message.contains("Invalid model name"));
    assert!(matches!(context.category, ErrorCategory::Configuration));
    assert!(matches!(context.severity, ErrorSeverity::High));
    assert!(!context.retryable);
    assert!(!context.suggestions.is_empty());
}

#[test]
fn test_type_conversion() {
    use lumosai_bindings::types::conversion::*;
    
    // 测试JSON值转换
    let json_value = serde_json::json!({
        "string": "test",
        "number": 42,
        "boolean": true,
        "array": [1, 2, 3],
        "object": {"nested": "value"}
    });
    
    let cross_lang_value = to_cross_lang_value(&json_value);
    assert_eq!(cross_lang_value, json_value);
    
    let back_value = from_cross_lang_value(&cross_lang_value);
    assert_eq!(back_value, json_value);
}

#[test]
fn test_parameter_validation() {
    use lumosai_bindings::types::conversion::*;
    
    let string_schema = ParameterSchema {
        param_type: ParameterType::String,
        description: Some("String parameter".to_string()),
        required: true,
        default: None,
        enum_values: None,
        properties: None,
        items: None,
    };
    
    // 测试有效值
    let valid_string = serde_json::Value::String("test".to_string());
    assert!(validate_parameter(&valid_string, &string_schema));
    
    // 测试无效值
    let invalid_number = serde_json::Value::Number(serde_json::Number::from(42));
    assert!(!validate_parameter(&invalid_number, &string_schema));
}

#[test]
fn test_config_defaults() {
    let performance_config = PerformanceConfig::default();
    assert_eq!(performance_config.timeout_seconds, 30);
    assert!(performance_config.enable_cache);
    assert_eq!(performance_config.cache_size, Some(1000));
    
    let security_config = SecurityConfig::default();
    assert!(security_config.enable_sandbox);
    assert!(!security_config.require_api_key);
    
    let logging_config = LoggingConfig::default();
    assert!(matches!(logging_config.level, LogLevel::Info));
    assert!(matches!(logging_config.format, LogFormat::Text));
    assert!(matches!(logging_config.output, LogOutput::Stdout));
}

#[test]
fn test_conversation_management() {
    let mut conversation = Conversation {
        id: "test_conv".to_string(),
        messages: Vec::new(),
        status: ConversationStatus::Active,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
    };
    
    // 添加用户消息
    let user_message = Message {
        id: "msg1".to_string(),
        content: "Hello".to_string(),
        message_type: MessageType::User,
        sender: "user".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
    };
    
    conversation.messages.push(user_message);
    
    // 添加助手消息
    let assistant_message = Message {
        id: "msg2".to_string(),
        content: "Hello! How can I help you?".to_string(),
        message_type: MessageType::Assistant,
        sender: "assistant".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
    };
    
    conversation.messages.push(assistant_message);
    
    assert_eq!(conversation.messages.len(), 2);
    assert!(matches!(conversation.status, ConversationStatus::Active));
}

#[test]
fn test_execution_result() {
    let stats = ExecutionStats {
        start_time: chrono::Utc::now(),
        end_time: chrono::Utc::now(),
        cpu_usage_percent: Some(25.5),
        network_requests: 2,
        cache_hits: 5,
        cache_misses: 1,
    };
    
    let result = ExecutionResult {
        execution_id: "exec1".to_string(),
        success: true,
        data: serde_json::json!({"result": "success"}),
        error: None,
        execution_time_ms: 150,
        memory_usage_bytes: Some(1024 * 1024), // 1MB
        stats,
    };
    
    assert!(result.success);
    assert!(result.error.is_none());
    assert_eq!(result.execution_time_ms, 150);
    assert_eq!(result.memory_usage_bytes, Some(1024 * 1024));
    assert_eq!(result.stats.network_requests, 2);
    assert_eq!(result.stats.cache_hits, 5);
}

#[test]
fn test_tool_definition() {
    let tool_def = ToolDefinition {
        name: "web_search".to_string(),
        description: "搜索网络内容".to_string(),
        parameters: ParameterSchema {
            param_type: ParameterType::Object,
            description: Some("搜索参数".to_string()),
            required: true,
            default: None,
            enum_values: None,
            properties: Some({
                let mut props = std::collections::HashMap::new();
                props.insert("query".to_string(), ParameterSchema {
                    param_type: ParameterType::String,
                    description: Some("搜索查询".to_string()),
                    required: true,
                    default: None,
                    enum_values: None,
                    properties: None,
                    items: None,
                });
                props
            }),
            items: None,
        },
        returns: None,
        tool_type: ToolType::ExternalApi,
        is_async: true,
        tags: vec!["web".to_string(), "search".to_string()],
        version: "1.0.0".to_string(),
    };
    
    assert_eq!(tool_def.name, "web_search");
    assert!(tool_def.is_async);
    assert!(matches!(tool_def.tool_type, ToolType::ExternalApi));
    assert_eq!(tool_def.tags.len(), 2);
    assert_eq!(tool_def.version, "1.0.0");
}

#[tokio::test]
async fn test_quick_agent_convenience() {
    let builder = quick_agent("quick_test", "你是一个快速测试助手");
    let agent = builder.build().unwrap();
    
    let config = agent.get_config();
    assert_eq!(config.model.name, "default");
    
    let response = agent.generate("测试消息");
    assert!(response.is_ok());
}

#[tokio::test]
async fn test_agent_with_tools() {
    let metadata = ToolMetadata {
        name: "test_calculator".to_string(),
        description: "测试计算器".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "数学表达式"
                }
            },
            "required": ["expression"]
        }),
        tool_type: "math".to_string(),
        is_async: false,
    };
    
    let tool = lumosai_core::tools::math::calculator();
    let cross_lang_tool = CrossLangTool::new(tool, metadata);
    
    let builder = CrossLangAgentBuilder::new()
        .name("tool_test_agent")
        .instructions("你是一个带工具的测试助手")
        .model("test_model")
        .tool(cross_lang_tool);
    
    let agent = builder.build().unwrap();
    let response = agent.generate("计算 2 + 2");
    
    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(!response.content.is_empty());
}
