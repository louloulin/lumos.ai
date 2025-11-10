//! Real API tests for Error module
//!
//! Tests error creation, conversion, and friendly error functionality

#[cfg(test)]
mod tests {
    use crate::error::{friendly::*, Error, Result};
    use serde_json::json;

    #[test]
    fn test_error_llm_creation() {
        let error = Error::Llm("API rate limit exceeded".to_string());
        assert!(error.to_string().contains("LLM error"));
        assert!(error.to_string().contains("API rate limit exceeded"));
    }

    #[test]
    fn test_error_agent_creation() {
        let error = Error::Agent("Agent initialization failed".to_string());
        assert!(error.to_string().contains("Agent error"));
        assert!(error.to_string().contains("initialization failed"));
    }

    #[test]
    fn test_error_tool_creation() {
        let error = Error::Tool("Tool not found".to_string());
        assert!(error.to_string().contains("Tool error"));
        assert!(error.to_string().contains("not found"));
    }

    #[test]
    fn test_error_memory_creation() {
        let error = Error::Memory("Memory capacity exceeded".to_string());
        assert!(error.to_string().contains("Memory error"));
        assert!(error.to_string().contains("capacity exceeded"));
    }

    #[test]
    fn test_error_workflow_creation() {
        let error = Error::Workflow("Workflow step failed".to_string());
        assert!(error.to_string().contains("Workflow error"));
        assert!(error.to_string().contains("step failed"));
    }

    #[test]
    fn test_error_configuration_creation() {
        let error = Error::Configuration("Invalid configuration".to_string());
        assert!(error.to_string().contains("Configuration error"));
        assert!(error.to_string().contains("Invalid configuration"));
    }

    #[test]
    fn test_error_from_string() {
        let error: Error = "Test error message".into();
        assert!(matches!(error, Error::Configuration(_)));
        assert!(error.to_string().contains("Test error message"));
    }

    #[test]
    fn test_error_from_str() {
        let error: Error = "Another test error".into();
        assert!(matches!(error, Error::Configuration(_)));
    }

    #[test]
    fn test_error_not_found() {
        let error = Error::NotFound("Resource not found".to_string());
        assert!(error.to_string().contains("Not found"));
        assert!(error.to_string().contains("Resource not found"));
    }

    #[test]
    fn test_error_invalid_input() {
        let error = Error::InvalidInput("Invalid parameter value".to_string());
        assert!(error.to_string().contains("Invalid input"));
        assert!(error.to_string().contains("Invalid parameter value"));
    }

    #[test]
    fn test_error_timeout() {
        let error = Error::Timeout("Operation timed out after 30s".to_string());
        assert!(error.to_string().contains("Timeout"));
        assert!(error.to_string().contains("30s"));
    }

    #[test]
    fn test_error_authentication() {
        let error = Error::Authentication("Invalid API key".to_string());
        assert!(error.to_string().contains("Authentication error"));
        assert!(error.to_string().contains("Invalid API key"));
    }

    #[test]
    fn test_error_network() {
        let error = Error::Network("Connection refused".to_string());
        assert!(error.to_string().contains("Network error"));
        assert!(error.to_string().contains("Connection refused"));
    }

    #[test]
    fn test_error_validation() {
        let error = Error::Validation {
            field: "email".to_string(),
            message: "Invalid email format".to_string(),
        };
        assert!(error.to_string().contains("Validation error"));
        assert!(error.to_string().contains("email"));
        assert!(error.to_string().contains("Invalid email format"));
    }

    #[test]
    fn test_error_api_error() {
        let error = Error::ApiError {
            message: "API request failed".to_string(),
            status_code: Some(429),
        };
        assert!(error.to_string().contains("API error"));
        assert!(error.to_string().contains("API request failed"));
    }

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(Error::Other("Test error".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_friendly_error_creation() {
        let error = FriendlyError::new(
            Error::Configuration("Missing API key".to_string()),
            "Configuration is invalid".to_string(),
        );

        assert_eq!(error.category, ErrorCategory::Configuration);
        assert_eq!(error.severity, ErrorSeverity::Medium);
        assert_eq!(error.message, "Configuration is invalid");
    }

    #[test]
    fn test_friendly_error_with_context() {
        let error = FriendlyError::new(
            Error::Tool("Invalid parameter".to_string()),
            "Tool execution failed".to_string(),
        )
        .with_context("tool_name", "calculator")
        .with_context("parameter", "invalid_value");

        assert_eq!(error.context.len(), 2);
        assert_eq!(error.context["tool_name"], "calculator");
        assert_eq!(error.context["parameter"], "invalid_value");
    }

    #[test]
    fn test_friendly_error_with_suggestion() {
        let error = FriendlyError::new(
            Error::Network("Connection timeout".to_string()),
            "Failed to connect to service".to_string(),
        )
        .with_suggestion("Check your internet connection")
        .with_suggestion("Verify the service is running");

        assert_eq!(error.suggestions.len(), 2);
        assert!(error.suggestions[0].contains("internet connection"));
        assert!(error.suggestions[1].contains("service is running"));
    }

    #[test]
    fn test_friendly_error_generate_suggestions() {
        let mut error = FriendlyError::new(
            Error::Configuration("Invalid config".to_string()),
            "Configuration error".to_string(),
        );

        error.generate_suggestions();
        assert!(!error.suggestions.is_empty());
        assert!(error
            .suggestions
            .iter()
            .any(|s| s.contains("configuration")));
    }

    #[test]
    fn test_friendly_error_format_for_display() {
        let error = FriendlyError::new(
            Error::Network("Connection timeout".to_string()),
            "Failed to connect to service".to_string(),
        )
        .with_context("endpoint", "https://api.example.com")
        .with_suggestion("Check your internet connection");

        let formatted = error.format_for_display();
        assert!(formatted.contains("❌"));
        assert!(formatted.contains("Context:"));
        assert!(formatted.contains("Suggestions:"));
        assert!(formatted.contains("https://api.example.com"));
    }

    #[test]
    fn test_error_category_configuration() {
        let error = Error::Configuration("Test".to_string());
        let friendly = FriendlyError::new(error, "Test message".to_string());
        assert_eq!(friendly.category, ErrorCategory::Configuration);
    }

    #[test]
    fn test_error_category_authentication() {
        let error = Error::Authentication("Test".to_string());
        let friendly = FriendlyError::new(error, "Test message".to_string());
        assert_eq!(friendly.category, ErrorCategory::Authentication);
    }

    #[test]
    fn test_error_category_network() {
        let error = Error::Network("Test".to_string());
        let friendly = FriendlyError::new(error, "Test message".to_string());
        assert_eq!(friendly.category, ErrorCategory::Network);
    }

    #[test]
    fn test_error_category_validation() {
        let error = Error::ValidationError("Test".to_string());
        let friendly = FriendlyError::new(error, "Test message".to_string());
        assert_eq!(friendly.category, ErrorCategory::Validation);
    }

    #[test]
    fn test_error_severity_high() {
        let error = Error::Authentication("Invalid credentials".to_string());
        let friendly = FriendlyError::new(error, "Test message".to_string());
        assert_eq!(friendly.severity, ErrorSeverity::High);
    }

    #[test]
    fn test_error_severity_medium() {
        let error = Error::Configuration("Invalid config".to_string());
        let friendly = FriendlyError::new(error, "Test message".to_string());
        assert_eq!(friendly.severity, ErrorSeverity::Medium);
    }

    #[test]
    fn test_error_severity_low() {
        let error = Error::Tool("Tool not found".to_string());
        let friendly = FriendlyError::new(error, "Test message".to_string());
        assert_eq!(friendly.severity, ErrorSeverity::Low);
    }

    #[test]
    fn test_helper_config_error() {
        let error = helpers::config_error("Missing API key", Some("/path/to/config.yaml"));
        assert_eq!(error.category, ErrorCategory::Configuration);
        assert!(error.message.contains("Configuration Error"));
        assert!(error.message.contains("Missing API key"));
        assert_eq!(error.context["config_file"], "/path/to/config.yaml");
    }

    #[test]
    fn test_helper_tool_error() {
        let params = json!({"param1": "value1"});
        let error = helpers::tool_error("calculator", "Invalid input", Some(&params));
        assert!(error.message.contains("calculator"));
        assert!(error.message.contains("Invalid input"));
        assert_eq!(error.context["tool_name"], "calculator");
        assert_eq!(error.context["parameters"], params);
    }

    #[test]
    fn test_helper_agent_error() {
        let error = helpers::agent_error("assistant", "Failed to generate response");
        assert!(error.message.contains("assistant"));
        assert!(error.message.contains("Failed to generate response"));
        assert_eq!(error.context["agent_name"], "assistant");
    }

    #[test]
    fn test_helper_network_error() {
        let error = helpers::network_error("https://api.example.com", Some(429));
        assert!(error.message.contains("Network Error"));
        assert!(error.message.contains("429"));
        assert_eq!(error.context["endpoint"], "https://api.example.com");
        assert_eq!(error.context["status_code"], 429);
    }
}
