//! Enhanced MCP integration tests
//! 
//! Tests for the enhanced MCP functionality including connection pooling,
//! health monitoring, and performance metrics.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;

// Note: These tests are designed to work without actual MCP servers
// In a real environment, you would need running MCP servers

#[tokio::test]
async fn test_enhanced_mcp_manager_creation() {
    // Test that we can create an enhanced MCP manager
    let config = lumosai_mcp::ManagerConfig {
        health_check_interval: Duration::from_secs(10),
        max_consecutive_failures: 3,
        connection_timeout: Duration::from_secs(5),
        tool_cache_ttl: Duration::from_secs(300),
        auto_reconnect: true,
        max_retry_attempts: 3,
    };
    
    let manager = lumosai_mcp::EnhancedMCPManager::new(config);
    
    // Test that we can get initial metrics
    let metrics = manager.get_metrics().await;
    assert_eq!(metrics.total_requests, 0);
    assert_eq!(metrics.successful_requests, 0);
    assert_eq!(metrics.failed_requests, 0);
}

#[tokio::test]
async fn test_enhanced_mcp_manager_health_status() {
    let config = lumosai_mcp::ManagerConfig::default();
    let manager = lumosai_mcp::EnhancedMCPManager::new(config);
    
    // Test that we can get health status (should be empty initially)
    let health_status = manager.get_health_status().await;
    assert!(health_status.is_empty());
}

#[tokio::test]
async fn test_enhanced_mcp_manager_tools() {
    let config = lumosai_mcp::ManagerConfig::default();
    let manager = lumosai_mcp::EnhancedMCPManager::new(config);
    
    // Test that we can get tools (should be empty initially)
    let tools = manager.get_all_tools().await;
    assert!(tools.is_ok());
    let tools = tools.unwrap();
    assert!(tools.is_empty());
}

#[tokio::test]
async fn test_mcp_configuration_creation() {
    // Test creating MCP configuration
    let config = lumosai_mcp::MCPConfiguration {
        name: "test-server".to_string(),
        transport: lumosai_mcp::TransportConfig::Stdio {
            command: "test-command".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
            env: HashMap::new(),
        },
        timeout_ms: 5000,
        retry_attempts: 3,
        capabilities: lumosai_mcp::ClientCapabilities {
            tools: Some(lumosai_mcp::ToolsCapability { list_changed: Some(true) }),
            resources: Some(lumosai_mcp::ResourcesCapability { 
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            logging: Some(lumosai_mcp::LoggingCapability { enabled: Some(true) }),
            extensions: HashMap::new(),
        },
    };
    
    assert_eq!(config.name, "test-server");
    assert_eq!(config.timeout_ms, 5000);
    assert_eq!(config.retry_attempts, 3);
}

#[tokio::test]
async fn test_mcp_message_serialization() {
    // Test that MCP messages can be serialized/deserialized
    let message = lumosai_mcp::MCPMessage::Initialize {
        name: "test-client".to_string(),
        version: "1.0.0".to_string(),
        capabilities: lumosai_mcp::ClientCapabilities {
            tools: Some(lumosai_mcp::ToolsCapability { list_changed: Some(true) }),
            resources: None,
            logging: None,
            extensions: HashMap::new(),
        },
    };
    
    // Test serialization
    let serialized = serde_json::to_string(&message);
    assert!(serialized.is_ok());
    
    // Test deserialization
    let deserialized: Result<lumosai_mcp::MCPMessage, _> = serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok());
}

#[tokio::test]
async fn test_mcp_tool_definition() {
    // Test creating tool definitions
    let tool = lumosai_mcp::Tool {
        name: "test-tool".to_string(),
        description: "A test tool".to_string(),
        input_schema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Input parameter"
                }
            },
            "required": ["input"]
        })),
    };
    
    assert_eq!(tool.name, "test-tool");
    assert_eq!(tool.description, "A test tool");
    assert!(tool.input_schema.is_some());
}

#[tokio::test]
async fn test_mcp_server_capabilities() {
    // Test server capabilities
    let capabilities = lumosai_mcp::ServerCapabilities {
        tools: Some(lumosai_mcp::ToolsCapability { list_changed: Some(true) }),
        resources: Some(lumosai_mcp::ResourcesCapability { 
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        logging: Some(lumosai_mcp::LoggingCapability { enabled: Some(true) }),
        extensions: {
            let mut ext = HashMap::new();
            ext.insert("custom".to_string(), serde_json::json!({"enabled": true}));
            ext
        },
    };
    
    assert!(capabilities.tools.is_some());
    assert!(capabilities.resources.is_some());
    assert!(capabilities.logging.is_some());
    assert!(!capabilities.extensions.is_empty());
}

#[tokio::test]
async fn test_mcp_resource_content() {
    // Test resource content
    let content = lumosai_mcp::ResourceContent {
        uri: "file:///test.txt".to_string(),
        mime_type: Some("text/plain".to_string()),
        text: Some("Hello, world!".to_string()),
        blob: None,
    };
    
    assert_eq!(content.uri, "file:///test.txt");
    assert_eq!(content.mime_type, Some("text/plain".to_string()));
    assert_eq!(content.text, Some("Hello, world!".to_string()));
    assert!(content.blob.is_none());
}

#[tokio::test]
async fn test_performance_metrics() {
    // Test performance metrics structure
    let mut metrics = lumosai_mcp::PerformanceMetrics::default();
    
    // Simulate some metrics
    metrics.total_requests = 100;
    metrics.successful_requests = 95;
    metrics.failed_requests = 5;
    metrics.average_response_time = Duration::from_millis(150);
    metrics.peak_response_time = Duration::from_millis(500);
    
    metrics.tool_execution_count.insert("test-tool".to_string(), 50);
    metrics.error_count_by_type.insert("timeout".to_string(), 3);
    metrics.error_count_by_type.insert("connection".to_string(), 2);
    
    assert_eq!(metrics.total_requests, 100);
    assert_eq!(metrics.successful_requests, 95);
    assert_eq!(metrics.failed_requests, 5);
    assert_eq!(metrics.average_response_time, Duration::from_millis(150));
    assert_eq!(metrics.peak_response_time, Duration::from_millis(500));
    
    assert_eq!(metrics.tool_execution_count.get("test-tool"), Some(&50));
    assert_eq!(metrics.error_count_by_type.get("timeout"), Some(&3));
    assert_eq!(metrics.error_count_by_type.get("connection"), Some(&2));
}

#[tokio::test]
async fn test_health_status() {
    // Test health status structure
    let health = lumosai_mcp::HealthStatus {
        is_healthy: true,
        last_check: std::time::Instant::now(),
        consecutive_failures: 0,
        last_error: None,
        response_time: Some(Duration::from_millis(100)),
    };
    
    assert!(health.is_healthy);
    assert_eq!(health.consecutive_failures, 0);
    assert!(health.last_error.is_none());
    assert_eq!(health.response_time, Some(Duration::from_millis(100)));
}

#[tokio::test]
async fn test_manager_config() {
    // Test manager configuration
    let config = lumosai_mcp::ManagerConfig {
        health_check_interval: Duration::from_secs(30),
        max_consecutive_failures: 5,
        connection_timeout: Duration::from_secs(10),
        tool_cache_ttl: Duration::from_secs(600),
        auto_reconnect: true,
        max_retry_attempts: 5,
    };
    
    assert_eq!(config.health_check_interval, Duration::from_secs(30));
    assert_eq!(config.max_consecutive_failures, 5);
    assert_eq!(config.connection_timeout, Duration::from_secs(10));
    assert_eq!(config.tool_cache_ttl, Duration::from_secs(600));
    assert!(config.auto_reconnect);
    assert_eq!(config.max_retry_attempts, 5);
}

#[tokio::test]
async fn test_default_manager_config() {
    // Test default manager configuration
    let config = lumosai_mcp::ManagerConfig::default();
    
    assert_eq!(config.health_check_interval, Duration::from_secs(30));
    assert_eq!(config.max_consecutive_failures, 3);
    assert_eq!(config.connection_timeout, Duration::from_secs(10));
    assert_eq!(config.tool_cache_ttl, Duration::from_secs(300));
    assert!(config.auto_reconnect);
    assert_eq!(config.max_retry_attempts, 3);
}
