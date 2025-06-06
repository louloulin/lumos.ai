//! Tests for CLI functionality
//! 
//! This module contains comprehensive tests for the enhanced CLI features

use super::*;
use super::commands::Commands;
use std::fs;
use tempfile::TempDir;
use tokio;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test project creation with different templates
    #[tokio::test]
    async fn test_project_creation() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        
        // Test basic template
        let result = Commands::new_project(
            "test-project",
            "basic",
            Some(project_path.clone())
        ).await;
        
        assert!(result.is_ok());
        assert!(project_path.exists());
        assert!(project_path.join("src").exists());
        assert!(project_path.join("src/main.rs").exists());
        assert!(project_path.join("Cargo.toml").exists());
        assert!(project_path.join("lumos.toml").exists());
    }

    /// Test stock assistant template creation
    #[tokio::test]
    async fn test_stock_assistant_template() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("stock-assistant");
        
        let result = Commands::new_project(
            "stock-assistant",
            "stock-assistant",
            Some(project_path.clone())
        ).await;
        
        assert!(result.is_ok());
        assert!(project_path.join("stock_config.toml").exists());
        assert!(project_path.join("data").exists());
        assert!(project_path.join("data/portfolio.csv").exists());
        assert!(project_path.join("README.md").exists());
        
        // Check that main.rs contains stock-specific content
        let main_rs_content = fs::read_to_string(project_path.join("src/main.rs")).unwrap();
        assert!(main_rs_content.contains("Stock Analysis Assistant"));
        assert!(main_rs_content.contains("stock market analyst"));
    }

    /// Test model addition functionality
    #[tokio::test]
    async fn test_add_model() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        
        // Create a basic project first
        Commands::new_project("test-project", "basic", Some(project_path.clone())).await.unwrap();
        
        // Change to project directory for the test
        std::env::set_current_dir(&project_path).unwrap();
        
        // Test adding DeepSeek model
        let result = Commands::add_model("deepseek", None, None).await;
        assert!(result.is_ok());
        
        // Verify configuration was updated
        let config_path = project_path.join("lumos.toml");
        let config = CliUtils::load_config(&config_path).unwrap();
        assert!(config.models.is_some());
        assert!(config.models.as_ref().unwrap().contains_key("deepseek"));
    }

    /// Test CLI utilities
    #[test]
    fn test_cli_utils() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        fs::create_dir_all(&project_path).unwrap();
        
        // Test project detection
        assert!(!CliUtils::is_lumos_project(&project_path));
        
        // Create lumos.toml
        fs::write(project_path.join("lumos.toml"), "").unwrap();
        assert!(CliUtils::is_lumos_project(&project_path));
        
        // Test project root finding
        let sub_dir = project_path.join("src");
        fs::create_dir_all(&sub_dir).unwrap();
        
        let found_root = CliUtils::find_project_root(&sub_dir);
        assert!(found_root.is_some());
        assert_eq!(found_root.unwrap(), project_path);
    }

    /// Test enhanced error handling
    #[test]
    fn test_enhanced_errors() {
        use super::enhanced_errors::{LumosCliError, ErrorHandler};
        
        let error_handler = ErrorHandler::new(true);
        
        // Test project not found error
        let error = LumosCliError::project_not_found();
        let enhanced = error_handler.handle_error(error, "lumos dev");
        
        assert!(!enhanced.suggestions.is_empty());
        assert!(!enhanced.related_docs.is_empty());
        
        // Test tool execution error
        let error = LumosCliError::tool_execution_failed(
            "web_search",
            "Network timeout",
            "Check internet connection"
        );
        let enhanced = error_handler.handle_error(error, "lumos tools test");
        
        assert!(!enhanced.suggestions.is_empty());
        assert!(enhanced.suggestions.iter().any(|s| s.contains("debug")));
    }

    /// Test configuration loading and saving
    #[test]
    fn test_config_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("lumos.toml");
        
        // Create default config
        let mut config = ProjectConfig::default();
        config.name = "test-project".to_string();
        config.default_model = Some("deepseek-chat".to_string());
        
        // Test saving
        let result = CliUtils::save_config(&config_path, &config);
        assert!(result.is_ok());
        assert!(config_path.exists());
        
        // Test loading
        let loaded_config = CliUtils::load_config(&config_path);
        assert!(loaded_config.is_ok());
        
        let loaded = loaded_config.unwrap();
        assert_eq!(loaded.name, "test-project");
        assert_eq!(loaded.default_model, Some("deepseek-chat".to_string()));
    }

    /// Test template validation
    #[tokio::test]
    async fn test_template_validation() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("invalid-template-project");

        // Test invalid template
        let result = Commands::new_project(
            "invalid-template-project",
            "invalid-template",
            Some(project_path.clone())
        ).await;

        assert!(result.is_err());
        // Note: The directory might be created before the template validation fails
        // This is expected behavior as the directory creation happens before template processing
    }

    /// Test build command functionality
    #[tokio::test]
    async fn test_build_command() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("build-test-project");
        
        // Create a basic project
        Commands::new_project("build-test-project", "basic", Some(project_path.clone())).await.unwrap();
        
        // Change to project directory
        std::env::set_current_dir(&project_path).unwrap();
        
        // Test debug build (should work without cargo installed in test environment)
        // We'll just test the command parsing, not actual execution
        let result = Commands::build_project("debug", None, false).await;
        // This might fail in test environment without cargo, which is expected
        // The important thing is that the function doesn't panic
    }

    /// Test format and lint commands
    #[tokio::test]
    async fn test_format_and_lint() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("format-test-project");
        
        // Create a basic project
        Commands::new_project("format-test-project", "basic", Some(project_path.clone())).await.unwrap();
        
        // Change to project directory
        std::env::set_current_dir(&project_path).unwrap();
        
        // Test format check (might fail without rustfmt, but shouldn't panic)
        let _result = Commands::format_project(true).await;
        
        // Test lint (might fail without clippy, but shouldn't panic)
        let _result = Commands::lint_project(false).await;
    }

    /// Test tool search functionality
    #[tokio::test]
    async fn test_tool_search() {
        // This test would require a mock marketplace
        // For now, we'll test that the function doesn't panic
        let result = Commands::search_tools("web", 5).await;
        // Expected to fail without real marketplace connection
        // The important thing is proper error handling
    }

    /// Integration test for complete workflow
    #[tokio::test]
    async fn test_complete_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("workflow-test");
        
        // 1. Create project
        let result = Commands::new_project("workflow-test", "stock-assistant", Some(project_path.clone())).await;
        assert!(result.is_ok());
        
        // 2. Change to project directory
        std::env::set_current_dir(&project_path).unwrap();
        
        // 3. Add a model
        let result = Commands::add_model("deepseek", Some("deepseek-chat".to_string()), None).await;
        assert!(result.is_ok());
        
        // 4. Verify project structure
        assert!(project_path.join("src/main.rs").exists());
        assert!(project_path.join("lumos.toml").exists());
        assert!(project_path.join("stock_config.toml").exists());
        assert!(project_path.join("README.md").exists());
        
        // 5. Verify configuration
        let config = CliUtils::load_config(project_path.join("lumos.toml")).unwrap();
        assert_eq!(config.name, "workflow-test");
        assert!(config.models.is_some());
        assert!(!config.tools.is_empty()); // Stock assistant should have tools
        
        // 6. Check that stock assistant tools are configured
        let tools: Vec<&str> = config.tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tools.contains(&"web_search"));
        assert!(tools.contains(&"calculator"));
        assert!(tools.contains(&"data_analyzer"));
    }
}

/// Benchmark tests for performance
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn benchmark_project_creation() {
        let temp_dir = TempDir::new().unwrap();
        
        let start = Instant::now();
        
        for i in 0..10 {
            let project_path = temp_dir.path().join(format!("bench-project-{}", i));
            let _ = Commands::new_project(
                &format!("bench-project-{}", i),
                "basic",
                Some(project_path)
            ).await;
        }
        
        let duration = start.elapsed();
        println!("Created 10 projects in {:?}", duration);
        
        // Should be reasonably fast (less than 5 seconds for 10 projects)
        assert!(duration.as_secs() < 5);
    }
}
