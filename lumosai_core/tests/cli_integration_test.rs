//! Integration tests for CLI functionality
//! 
//! These tests verify that the CLI tools work correctly and provide
//! the expected developer experience.

use std::path::PathBuf;
use tempfile::TempDir;
use lumosai_core::cli::{
    ProjectConfig, CliUtils, 
    commands::Commands,
    web_interface::{WebInterface, WebInterfaceConfig, CreateSessionRequest, SendMessageRequest},
    enhanced_errors::{LumosCliError, ErrorHandler},
};

#[tokio::test]
async fn test_project_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "test-project";
    
    // Test basic project creation
    let result = Commands::new_project(
        project_name,
        "basic",
        Some(temp_dir.path().join(project_name))
    ).await;
    
    assert!(result.is_ok(), "Project creation should succeed");
    
    // Verify project structure
    let project_path = temp_dir.path().join(project_name);
    assert!(project_path.exists(), "Project directory should exist");
    assert!(project_path.join("src").exists(), "src directory should exist");
    assert!(project_path.join("src/main.rs").exists(), "main.rs should exist");
    assert!(project_path.join("Cargo.toml").exists(), "Cargo.toml should exist");
    assert!(project_path.join("lumos.toml").exists(), "lumos.toml should exist");
    
    // Verify configuration
    let config_path = project_path.join("lumos.toml");
    let config = CliUtils::load_config(&config_path).expect("Should load config");
    assert_eq!(config.name, project_name);
    assert_eq!(config.version, "0.1.0");
}

#[tokio::test]
async fn test_web_agent_template() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "web-agent-test";
    
    // Create web agent project
    let result = Commands::new_project(
        project_name,
        "web-agent",
        Some(temp_dir.path().join(project_name))
    ).await;
    
    assert!(result.is_ok(), "Web agent project creation should succeed");
    
    // Verify web-specific configuration
    let project_path = temp_dir.path().join(project_name);
    let config_path = project_path.join("lumos.toml");
    let config = CliUtils::load_config(&config_path).expect("Should load config");
    
    // Should have web tools
    assert!(!config.tools.is_empty(), "Should have tools configured");
    assert!(config.tools.iter().any(|t| t.name == "web_search"), "Should have web_search tool");
    assert!(config.tools.iter().any(|t| t.name == "http_request"), "Should have http_request tool");
    
    // Verify main.rs contains web-specific content
    let main_rs_content = std::fs::read_to_string(project_path.join("src/main.rs"))
        .expect("Should read main.rs");
    assert!(main_rs_content.contains("web-savvy assistant"), "Should contain web-specific instructions");
    assert!(main_rs_content.contains("web_search"), "Should reference web tools");
}

#[tokio::test]
async fn test_data_agent_template() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "data-agent-test";
    
    // Create data agent project
    let result = Commands::new_project(
        project_name,
        "data-agent",
        Some(temp_dir.path().join(project_name))
    ).await;
    
    assert!(result.is_ok(), "Data agent project creation should succeed");
    
    // Verify data-specific configuration
    let project_path = temp_dir.path().join(project_name);
    let config_path = project_path.join("lumos.toml");
    let config = CliUtils::load_config(&config_path).expect("Should load config");
    
    // Should have data tools
    assert!(!config.tools.is_empty(), "Should have tools configured");
    assert!(config.tools.iter().any(|t| t.name == "csv_reader"), "Should have csv_reader tool");
    assert!(config.tools.iter().any(|t| t.name == "data_analyzer"), "Should have data_analyzer tool");
    
    // Should have data directory
    assert!(project_path.join("data").exists(), "Should have data directory");
}

#[tokio::test]
async fn test_chatbot_template() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "chatbot-test";
    
    // Create chatbot project
    let result = Commands::new_project(
        project_name,
        "chat-bot",
        Some(temp_dir.path().join(project_name))
    ).await;
    
    assert!(result.is_ok(), "Chatbot project creation should succeed");
    
    // Verify chatbot-specific configuration
    let project_path = temp_dir.path().join(project_name);
    let config_path = project_path.join("lumos.toml");
    let config = CliUtils::load_config(&config_path).expect("Should load config");
    
    // Should have chatbot tools
    assert!(!config.tools.is_empty(), "Should have tools configured");
    assert!(config.tools.iter().any(|t| t.name == "memory"), "Should have memory tool");
    assert!(config.tools.iter().any(|t| t.name == "calculator"), "Should have calculator tool");
    
    // Verify main.rs contains chatbot-specific content
    let main_rs_content = std::fs::read_to_string(project_path.join("src/main.rs"))
        .expect("Should read main.rs");
    assert!(main_rs_content.contains("Interactive Chatbot"), "Should contain chatbot title");
    assert!(main_rs_content.contains("loop"), "Should contain interaction loop");
}

#[tokio::test]
async fn test_cli_utils() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Test project detection
    assert!(!CliUtils::is_lumos_project(&temp_dir.path()), "Should not detect project initially");
    
    // Create lumos.toml
    let config = ProjectConfig::default();
    let config_path = temp_dir.path().join("lumos.toml");
    CliUtils::save_config(&config_path, &config).expect("Should save config");
    
    // Now should detect project
    assert!(CliUtils::is_lumos_project(&temp_dir.path()), "Should detect project after config creation");
    
    // Test project root finding
    let subdir = temp_dir.path().join("src");
    std::fs::create_dir_all(&subdir).expect("Should create subdir");
    
    let found_root = CliUtils::find_project_root(&subdir);
    assert!(found_root.is_some(), "Should find project root from subdirectory");
    assert_eq!(found_root.unwrap(), temp_dir.path(), "Should find correct project root");
}

#[tokio::test]
async fn test_web_interface() {
    let config = ProjectConfig::default();
    let web_config = WebInterfaceConfig::default();
    let web_interface = WebInterface::new(web_config, config);
    
    // Test session creation
    let create_request = CreateSessionRequest {
        agent_name: "test-agent".to_string(),
        initial_message: Some("Hello!".to_string()),
    };
    
    let session_response = web_interface.create_session(create_request).await;
    assert!(session_response.is_ok(), "Session creation should succeed");
    
    let session = session_response.unwrap();
    assert_eq!(session.agent_name, "test-agent");
    assert!(!session.session_id.is_empty(), "Session ID should not be empty");
    
    // Test message sending
    let message_request = SendMessageRequest {
        content: "How are you?".to_string(),
        metadata: None,
    };
    
    let message_response = web_interface.send_message(&session.session_id, message_request).await;
    assert!(message_response.is_ok(), "Message sending should succeed");
    
    let response = message_response.unwrap();
    assert!(!response.response.is_empty(), "Response should not be empty");
    assert!(response.metadata.is_some(), "Should have metadata");
    
    // Test session retrieval
    let retrieved_session = web_interface.get_session(&session.session_id).await;
    assert!(retrieved_session.is_ok(), "Session retrieval should succeed");
    
    let session_data = retrieved_session.unwrap();
    assert_eq!(session_data.id, session.session_id);
    assert_eq!(session_data.messages.len(), 2); // Initial + user message
}

#[tokio::test]
async fn test_project_status() {
    let mut config = ProjectConfig::default();
    config.name = "test-project".to_string();
    config.version = "1.0.0".to_string();
    
    let web_config = WebInterfaceConfig::default();
    let web_interface = WebInterface::new(web_config, config);
    
    let status = web_interface.get_project_status().await;
    assert!(status.is_ok(), "Project status should be retrievable");
    
    let status_data = status.unwrap();
    assert_eq!(status_data.name, "test-project");
    assert_eq!(status_data.version, "1.0.0");
    assert_eq!(status_data.health.overall, "healthy");
    assert!(!status_data.agents.is_empty(), "Should have at least one agent");
}

#[tokio::test]
async fn test_enhanced_error_handling() {
    let error_handler = ErrorHandler::new(true);
    
    // Test project not found error
    let error = LumosCliError::project_not_found();
    let enhanced = error_handler.handle_error(error, "lumos dev");
    
    assert!(!enhanced.suggestions.is_empty(), "Should have suggestions");
    assert!(!enhanced.related_docs.is_empty(), "Should have related docs");
    assert_eq!(enhanced.context.command, "lumos dev");
    
    // Test tool execution error
    let error = LumosCliError::tool_execution_failed(
        "web_search",
        "Network timeout",
        "Check your internet connection"
    );
    let enhanced = error_handler.handle_error(error, "lumos tools add web_search");
    
    assert!(enhanced.suggestions.len() >= 3, "Should have multiple suggestions");
    assert!(enhanced.related_docs.iter().any(|doc| doc.contains("tools")), "Should have tool-related docs");
}

#[tokio::test]
async fn test_configuration_validation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Test valid configuration
    let config = ProjectConfig::default();
    let config_path = temp_dir.path().join("lumos.toml");
    
    let save_result = CliUtils::save_config(&config_path, &config);
    assert!(save_result.is_ok(), "Should save valid configuration");
    
    let load_result = CliUtils::load_config(&config_path);
    assert!(load_result.is_ok(), "Should load valid configuration");
    
    let loaded_config = load_result.unwrap();
    assert_eq!(loaded_config.name, config.name);
    assert_eq!(loaded_config.version, config.version);
}

#[tokio::test]
async fn test_template_validation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Test all supported templates
    let templates = vec!["basic", "web-agent", "data-agent", "chat-bot"];
    
    for template in templates {
        let project_name = format!("test-{}", template);
        let result = Commands::new_project(
            &project_name,
            template,
            Some(temp_dir.path().join(&project_name))
        ).await;
        
        assert!(result.is_ok(), "Template '{}' should be valid", template);
        
        // Verify project was created
        let project_path = temp_dir.path().join(&project_name);
        assert!(project_path.exists(), "Project directory should exist for template '{}'", template);
        assert!(project_path.join("lumos.toml").exists(), "Config should exist for template '{}'", template);
    }
    
    // Test invalid template
    let result = Commands::new_project(
        "invalid-template-test",
        "invalid-template",
        Some(temp_dir.path().join("invalid-template-test"))
    ).await;
    
    assert!(result.is_err(), "Invalid template should fail");
}

#[tokio::test]
async fn test_error_message_quality() {
    // Test that error messages are helpful and actionable
    let error = LumosCliError::config_error(
        "tools",
        "Missing required field 'name'",
        "Each tool must have a name field",
        "lumos tools add <tool-name>"
    );
    
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("⚙️"), "Should have configuration emoji");
    assert!(error_msg.contains("tools"), "Should mention the section");
    assert!(error_msg.contains("Missing required field"), "Should describe the issue");
    assert!(error_msg.contains("lumos tools add"), "Should provide fix command");
    
    // Test network error
    let error = LumosCliError::network_error("Connection timeout");
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("🌐"), "Should have network emoji");
    assert!(error_msg.contains("Connection timeout"), "Should include specific error");
    assert!(error_msg.contains("internet connection"), "Should suggest checking connection");
}

#[tokio::test]
async fn test_cli_performance() {
    // Test that CLI operations complete within reasonable time
    let start = std::time::Instant::now();
    
    // Create multiple projects quickly
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    for i in 0..5 {
        let project_name = format!("perf-test-{}", i);
        let result = Commands::new_project(
            &project_name,
            "basic",
            Some(temp_dir.path().join(&project_name))
        ).await;
        
        assert!(result.is_ok(), "Project creation {} should succeed", i);
    }
    
    let duration = start.elapsed();
    assert!(duration.as_secs() < 5, "Should create 5 projects in under 5 seconds, took {:?}", duration);
}
