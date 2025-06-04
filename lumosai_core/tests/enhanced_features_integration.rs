//! Integration tests for enhanced features (debugging, logging, marketplace)
//! 
//! This test suite validates the implementation of debugging tools, logging system, and marketplace

use lumosai_core::debug::{DebugManager, DebugEvent, DebugSession};
use lumosai_core::logging::{Logger, LogLevel, LoggerConfig, LogFormat};
use lumosai_core::marketplace::{Marketplace, ToolCategory, ToolPackage, ValidationReport};
use lumosai_core::error::friendly::{FriendlyError, helpers};
use lumosai_core::error::Error;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::env;
use serde_json::json;

#[tokio::test]
async fn test_friendly_error_system() {
    // Test basic friendly error creation
    let error = FriendlyError::new(
        Error::Configuration("Missing API key".to_string()),
        "Configuration is invalid".to_string()
    );
    
    assert_eq!(error.message, "Configuration is invalid");
    assert!(!error.error_code().is_empty());
    
    // Test error with context and suggestions
    let error_with_context = FriendlyError::new(
        Error::Tool("Invalid parameter".to_string()),
        "Tool execution failed".to_string()
    )
    .with_context("tool_name", "calculator")
    .with_context("parameter", "invalid_value")
    .with_suggestion("Check parameter format")
    .with_suggestion("Review tool documentation");
    
    assert_eq!(error_with_context.context.len(), 2);
    assert_eq!(error_with_context.suggestions.len(), 2);
    
    // Test formatted display
    let formatted = error_with_context.format_for_display();
    assert!(formatted.contains("Context:"));
    assert!(formatted.contains("Suggestions:"));
    assert!(formatted.contains("Technical Details:"));
}

#[tokio::test]
async fn test_friendly_error_helpers() {
    // Test configuration error helper
    let config_error = helpers::config_error("Missing field 'api_key'", Some("/path/to/config.json"));
    assert!(config_error.context.contains_key("config_file"));
    assert!(!config_error.suggestions.is_empty());
    
    // Test tool error helper
    let tool_error = helpers::tool_error(
        "calculator", 
        "Division by zero", 
        Some(&json!({"a": 10, "b": 0}))
    );
    assert!(tool_error.context.contains_key("tool_name"));
    assert!(tool_error.context.contains_key("parameters"));
    
    // Test agent error helper
    let agent_error = helpers::agent_error("math_agent", "Failed to initialize");
    assert!(agent_error.context.contains_key("agent_name"));
    
    // Test network error helper
    let network_error = helpers::network_error("https://api.example.com", Some(404));
    assert!(network_error.context.contains_key("endpoint"));
    assert!(network_error.context.contains_key("status_code"));
}

#[tokio::test]
async fn test_debug_system() {
    // Test debug session creation
    let mut session = DebugSession::new();
    assert!(!session.id.is_empty());
    assert_eq!(session.events.len(), 0);
    
    // Test adding events
    session.add_event(DebugEvent::AgentCreated {
        agent_name: "test_agent".to_string(),
        timestamp: Instant::now(),
        config: json!({"name": "test", "model": "gpt-4"}),
    });
    
    session.add_event(DebugEvent::ToolExecuted {
        tool_name: "calculator".to_string(),
        timestamp: Instant::now(),
        duration: Duration::from_millis(150),
        input: json!({"operation": "add", "a": 2, "b": 3}),
        output: Ok(json!({"result": 5})),
    });
    
    session.add_event(DebugEvent::ErrorOccurred {
        timestamp: Instant::now(),
        error: "Test error".to_string(),
        context: [("context_key".to_string(), json!("context_value"))]
            .into_iter().collect(),
    });
    
    assert_eq!(session.events.len(), 3);
    
    // Test summary generation
    let summary = session.generate_summary();
    assert_eq!(summary.tool_executions, 1);
    assert_eq!(summary.error_count, 1);
    assert!(summary.tool_usage.contains_key("calculator"));
    
    // Test JSON export
    let json_export = session.export_json();
    assert_eq!(json_export["event_count"], 3);
    assert!(json_export["events"].is_array());
}

#[tokio::test]
async fn test_debug_manager() {
    let mut manager = DebugManager::new();
    
    // Test initial state
    assert!(!manager.is_enabled());
    assert!(manager.current_session().is_none());

    // Test starting session
    let session_id = manager.start_session();
    assert!(!session_id.is_empty());
    assert!(manager.is_enabled());
    assert!(manager.current_session().is_some());
    
    // Test adding events
    manager.add_event(DebugEvent::AgentCreated {
        agent_name: "debug_test_agent".to_string(),
        timestamp: Instant::now(),
        config: json!({}),
    });
    
    manager.add_event(DebugEvent::ToolExecuted {
        tool_name: "test_tool".to_string(),
        timestamp: Instant::now(),
        duration: Duration::from_millis(100),
        input: json!({}),
        output: Ok(json!({"success": true})),
    });
    
    // Test session export
    let export = manager.export_current_session();
    assert!(export.is_some());
    
    // Test ending session
    let summary = manager.end_session();
    assert!(summary.is_some());
    assert!(!manager.is_enabled());
    assert!(manager.current_session().is_none());
    
    let summary = summary.unwrap();
    assert_eq!(summary.tool_executions, 1);
    
    // Test summary formatting
    let formatted = summary.format_display();
    assert!(formatted.contains("Debug Session Summary"));
    assert!(formatted.contains("Tool Executions: 1"));
}

#[tokio::test]
async fn test_logging_system() {
    // Test logger creation
    let logger = Logger::new("test_module");
    assert_eq!(logger.get_module(), "test_module");

    // Test logger with correlation ID
    let logger_with_corr = Logger::new("corr_module")
        .with_correlation_id("req-123");
    assert_eq!(logger_with_corr.get_correlation_id(), Some("req-123"));
    
    // Test log levels
    assert!(LogLevel::Error > LogLevel::Warn);
    assert!(LogLevel::Warn > LogLevel::Info);
    assert!(LogLevel::Info > LogLevel::Debug);
    
    // Test log entry creation
    let entry = lumosai_core::logging::LogEntry::new(
        LogLevel::Info,
        "Test message".to_string(),
        "test_module".to_string()
    )
    .with_field("key1", "value1")
    .with_field("key2", 42)
    .with_tag("test")
    .with_correlation_id("corr-456");
    
    assert_eq!(entry.level, LogLevel::Info);
    assert_eq!(entry.message, "Test message");
    assert_eq!(entry.fields.len(), 2);
    assert_eq!(entry.tags.len(), 1);
    assert_eq!(entry.correlation_id, Some("corr-456".to_string()));
    
    // Test formatting
    let console_format = entry.format_console();
    assert!(console_format.contains("Test message"));
    assert!(console_format.contains("key1=value1"));
    assert!(console_format.contains("[test]"));
    assert!(console_format.contains("(corr-456)"));
    
    let json_format = entry.format_json();
    assert_eq!(json_format["level"], "INFO");
    assert_eq!(json_format["message"], "Test message");
    assert_eq!(json_format["fields"]["key1"], "value1");
    assert_eq!(json_format["fields"]["key2"], 42);
}

#[tokio::test]
async fn test_logger_configuration() {
    // Test custom logger configuration
    let config = LoggerConfig {
        min_level: LogLevel::Debug,
        format: LogFormat::Json,
        include_module: true,
        include_timestamp: true,
        include_correlation_id: true,
        color_enabled: false,
    };
    
    let logger = Logger::with_config("custom_module", config);
    assert_eq!(logger.get_config().min_level, LogLevel::Debug);
    assert!(matches!(logger.get_config().format, LogFormat::Json));
    
    // Test logging methods (these would normally output to console)
    logger.info("Info message");
    logger.warn("Warning message");
    logger.error("Error message");
    
    // Test logging with fields
    let fields = [
        ("user_id".to_string(), json!("user123")),
        ("action".to_string(), json!("login")),
    ].into_iter().collect();
    
    logger.log_with_fields(LogLevel::Info, "User action", fields);
}

#[tokio::test]
async fn test_marketplace_system() {
    // Test marketplace creation
    let cache_dir = env::temp_dir().join("lumos_marketplace_test");
    let mut marketplace = Marketplace::new(
        "https://registry.lumos.ai".to_string(),
        cache_dir
    );
    
    assert_eq!(marketplace.get_registry_url(), "https://registry.lumos.ai");
    assert_eq!(marketplace.get_installed_count(), 0);
    
    // Test tool search
    let search_results = marketplace.search("web", Some(ToolCategory::Web)).await;
    assert!(search_results.is_ok());
    
    let results = search_results.unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].name, "web-scraper-pro");
    assert_eq!(results[0].categories, vec![ToolCategory::Web]);
    
    // Test package info retrieval
    let package_info = marketplace.get_package_info("web-scraper-pro").await;
    assert!(package_info.is_ok());
    
    let package = package_info.unwrap();
    assert_eq!(package.name, "web-scraper-pro");
    assert!(!package.manifest.tools.is_empty());
    
    // Test package validation
    let validation_report = marketplace.validate_package(&package);
    assert!(validation_report.is_ok());
    
    let report = validation_report.unwrap();
    assert!(report.is_valid);
    
    // Test installation (mock)
    let install_result = marketplace.install("web-scraper-pro", Some("1.2.0")).await;
    assert!(install_result.is_ok());
    
    // Test cache update
    let update_result = marketplace.update_cache().await;
    assert!(update_result.is_ok());
}

#[tokio::test]
async fn test_tool_categories() {
    // Test category properties
    assert_eq!(ToolCategory::Web.as_str(), "web");
    assert_eq!(ToolCategory::Web.emoji(), "🌐");
    
    assert_eq!(ToolCategory::AI.as_str(), "ai");
    assert_eq!(ToolCategory::AI.emoji(), "🤖");
    
    assert_eq!(ToolCategory::Database.as_str(), "database");
    assert_eq!(ToolCategory::Database.emoji(), "🗄️");
    
    // Test serialization/deserialization
    let category = ToolCategory::Web;
    let serialized = serde_json::to_string(&category).unwrap();
    assert_eq!(serialized, "\"web\"");
    
    let deserialized: ToolCategory = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, ToolCategory::Web);
}

#[tokio::test]
async fn test_validation_report() {
    let mut report = ValidationReport::new();
    
    // Test initial state
    assert!(report.is_valid);
    assert_eq!(report.errors.len(), 0);
    assert_eq!(report.warnings.len(), 0);
    
    // Test adding warnings (should not affect validity)
    report.add_warning("This is a warning");
    assert!(report.is_valid);
    assert_eq!(report.warnings.len(), 1);
    
    // Test adding errors (should affect validity)
    report.add_error("This is an error");
    assert!(!report.is_valid);
    assert_eq!(report.errors.len(), 1);
    
    // Test report formatting
    let formatted = report.format_report();
    assert!(formatted.contains("❌ Package validation failed"));
    assert!(formatted.contains("🚨 Errors:"));
    assert!(formatted.contains("⚠️  Warnings:"));
    assert!(formatted.contains("This is an error"));
    assert!(formatted.contains("This is a warning"));
}

#[tokio::test]
async fn test_comprehensive_workflow() {
    // Test a comprehensive workflow that uses all enhanced features
    
    // 1. Start debug session
    let mut debug_manager = DebugManager::new();
    let session_id = debug_manager.start_session();
    
    // 2. Create logger with correlation ID
    let logger = Logger::new("comprehensive_test")
        .with_correlation_id(session_id.clone());
    
    // 3. Log start of workflow
    logger.info("Starting comprehensive workflow test");
    
    // 4. Add debug events
    debug_manager.add_event(DebugEvent::AgentCreated {
        agent_name: "comprehensive_agent".to_string(),
        timestamp: Instant::now(),
        config: json!({"model": "gpt-4", "tools": ["calculator", "web_scraper"]}),
    });
    
    // 5. Simulate tool execution with logging
    let tool_start = Instant::now();
    logger.log_with_fields(
        LogLevel::Info,
        "Executing tool",
        [("tool_name".to_string(), json!("calculator"))]
            .into_iter().collect()
    );
    
    // Simulate some processing time
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    let tool_duration = tool_start.elapsed();
    debug_manager.add_event(DebugEvent::ToolExecuted {
        tool_name: "calculator".to_string(),
        timestamp: tool_start,
        duration: tool_duration,
        input: json!({"operation": "multiply", "a": 6, "b": 7}),
        output: Ok(json!({"result": 42})),
    });
    
    logger.log_with_fields(
        LogLevel::Info,
        "Tool execution completed",
        [
            ("tool_name".to_string(), json!("calculator")),
            ("duration_ms".to_string(), json!(tool_duration.as_millis())),
            ("success".to_string(), json!(true)),
        ].into_iter().collect()
    );
    
    // 6. Test marketplace interaction
    let cache_dir = env::temp_dir().join("comprehensive_test_cache");
    let marketplace = Marketplace::new(
        "https://registry.lumos.ai".to_string(),
        cache_dir
    );
    
    let search_results = marketplace.search("math", Some(ToolCategory::Math)).await;
    logger.log_with_fields(
        LogLevel::Info,
        "Marketplace search completed",
        [
            ("query".to_string(), json!("math")),
            ("results_count".to_string(), json!(search_results.as_ref().map(|r| r.len()).unwrap_or(0))),
        ].into_iter().collect()
    );
    
    // 7. Simulate an error and test friendly error handling
    let friendly_error = helpers::tool_error(
        "nonexistent_tool",
        "Tool not found in registry",
        Some(&json!({"tool_name": "nonexistent_tool"}))
    );
    
    debug_manager.add_event(DebugEvent::ErrorOccurred {
        timestamp: Instant::now(),
        error: friendly_error.message.clone(),
        context: friendly_error.context.clone(),
    });
    
    logger.error(&format!("Error occurred: {}", friendly_error.message));
    
    // 8. End debug session and generate summary
    let summary = debug_manager.end_session().unwrap();
    
    logger.log_with_fields(
        LogLevel::Info,
        "Workflow completed",
        [
            ("session_id".to_string(), json!(summary.session_id)),
            ("total_duration_ms".to_string(), json!(summary.total_duration.as_millis())),
            ("tool_executions".to_string(), json!(summary.tool_executions)),
            ("error_count".to_string(), json!(summary.error_count)),
        ].into_iter().collect()
    );
    
    // Verify results
    assert_eq!(summary.tool_executions, 1);
    assert_eq!(summary.error_count, 1);
    assert!(summary.tool_usage.contains_key("calculator"));
    assert!(summary.total_duration > Duration::ZERO);
    
    // Test summary formatting
    let formatted_summary = summary.format_display();
    assert!(formatted_summary.contains("Debug Session Summary"));
    assert!(formatted_summary.contains("Tool Executions: 1"));
    assert!(formatted_summary.contains("Errors: 1"));
}

#[tokio::test]
async fn test_performance_and_scalability() {
    // Test that enhanced features don't significantly impact performance
    let start_time = Instant::now();
    
    // Create multiple debug sessions
    let mut sessions = Vec::new();
    for i in 0..100 {
        let mut session = DebugSession::new();
        session.add_event(DebugEvent::AgentCreated {
            agent_name: format!("agent_{}", i),
            timestamp: Instant::now(),
            config: json!({"id": i}),
        });
        sessions.push(session);
    }
    
    // Create multiple loggers
    let mut loggers = Vec::new();
    for i in 0..100 {
        let logger = Logger::new(&format!("module_{}", i))
            .with_correlation_id(format!("corr_{}", i));
        loggers.push(logger);
    }
    
    let creation_time = start_time.elapsed();
    
    // Should be able to create 100 sessions and loggers quickly
    assert!(creation_time.as_millis() < 1000); // Less than 1 second
    
    // Test bulk operations
    let bulk_start = Instant::now();
    
    for (i, session) in sessions.iter_mut().enumerate() {
        session.add_event(DebugEvent::ToolExecuted {
            tool_name: format!("tool_{}", i),
            timestamp: Instant::now(),
            duration: Duration::from_millis(i as u64),
            input: json!({"index": i}),
            output: Ok(json!({"result": i * 2})),
        });
    }
    
    for (i, logger) in loggers.iter().enumerate() {
        logger.log_with_fields(
            LogLevel::Info,
            &format!("Bulk operation {}", i),
            [("index".to_string(), json!(i))]
                .into_iter().collect()
        );
    }
    
    let bulk_time = bulk_start.elapsed();
    
    // Bulk operations should also be fast
    assert!(bulk_time.as_millis() < 2000); // Less than 2 seconds
    
    // Verify all sessions have events
    for session in &sessions {
        assert_eq!(session.events.len(), 2); // AgentCreated + ToolExecuted
    }
}
