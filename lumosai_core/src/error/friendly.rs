//! Friendly error handling system inspired by Mastra's developer experience
//! 
//! This module provides user-friendly error messages, debugging hints, and recovery suggestions

use crate::error::Error;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// Enhanced error with context and suggestions
#[derive(Debug)]
pub struct FriendlyError {
    /// The underlying error
    pub error: Error,
    /// User-friendly message
    pub message: String,
    /// Context information
    pub context: HashMap<String, Value>,
    /// Suggested fixes
    pub suggestions: Vec<String>,
    /// Error category for grouping
    pub category: ErrorCategory,
    /// Severity level
    pub severity: ErrorSeverity,
}

/// Error categories for better organization
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    Configuration,
    Authentication,
    Network,
    Tool,
    Agent,
    Memory,
    Validation,
    Runtime,
    Unknown,
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Low,      // Warning, doesn't stop execution
    Medium,   // Error, stops current operation
    High,     // Critical error, affects system
    Critical, // System failure
}

impl FriendlyError {
    /// Create a new friendly error
    pub fn new(error: Error, message: String) -> Self {
        let category = Self::categorize_error(&error);
        let severity = Self::determine_severity(&error, &category);
        
        Self {
            error,
            message,
            context: HashMap::new(),
            suggestions: Vec::new(),
            category,
            severity,
        }
    }

    /// Add context information
    pub fn with_context<K: Into<String>, V: Into<Value>>(mut self, key: K, value: V) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Add a suggestion
    pub fn with_suggestion<S: Into<String>>(mut self, suggestion: S) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    /// Add multiple suggestions
    pub fn with_suggestions<I, S>(mut self, suggestions: I) -> Self 
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.suggestions.extend(suggestions.into_iter().map(|s| s.into()));
        self
    }

    /// Categorize error based on its type
    fn categorize_error(error: &Error) -> ErrorCategory {
        match error {
            Error::Configuration(_) => ErrorCategory::Configuration,
            Error::Authentication(_) => ErrorCategory::Authentication,
            Error::Network(_) => ErrorCategory::Network,
            Error::Tool(_) => ErrorCategory::Tool,
            Error::Agent(_) => ErrorCategory::Agent,
            Error::Memory(_) => ErrorCategory::Memory,
            Error::Validation(_) => ErrorCategory::Validation,
            Error::Io(_) => ErrorCategory::Runtime,
            Error::Serialization(_) => ErrorCategory::Validation,
            Error::InvalidOperation(_) => ErrorCategory::Runtime,
            _ => ErrorCategory::Unknown,
        }
    }

    /// Determine severity based on error type and category
    fn determine_severity(error: &Error, category: &ErrorCategory) -> ErrorSeverity {
        match (error, category) {
            (Error::Authentication(_), _) => ErrorSeverity::High,
            (Error::Configuration(_), _) => ErrorSeverity::Medium,
            (Error::Network(_), _) => ErrorSeverity::Medium,
            (Error::Tool(_), _) => ErrorSeverity::Low,
            (Error::Validation(_), _) => ErrorSeverity::Low,
            (Error::Memory(_), _) => ErrorSeverity::High,
            (Error::Agent(_), _) => ErrorSeverity::Medium,
            _ => ErrorSeverity::Medium,
        }
    }

    /// Generate helpful suggestions based on error type
    pub fn generate_suggestions(&mut self) {
        match &self.category {
            ErrorCategory::Configuration => {
                self.suggestions.extend(vec![
                    "Check your configuration file syntax".to_string(),
                    "Verify all required fields are present".to_string(),
                    "Use 'lumos config validate' to check configuration".to_string(),
                ]);
            },
            ErrorCategory::Authentication => {
                self.suggestions.extend(vec![
                    "Verify your API keys are correct".to_string(),
                    "Check if your credentials have expired".to_string(),
                    "Ensure you have the necessary permissions".to_string(),
                ]);
            },
            ErrorCategory::Network => {
                self.suggestions.extend(vec![
                    "Check your internet connection".to_string(),
                    "Verify the service endpoint is accessible".to_string(),
                    "Try again in a few moments".to_string(),
                ]);
            },
            ErrorCategory::Tool => {
                self.suggestions.extend(vec![
                    "Check tool parameters are valid".to_string(),
                    "Verify the tool is properly registered".to_string(),
                    "Review tool documentation for usage examples".to_string(),
                ]);
            },
            ErrorCategory::Agent => {
                self.suggestions.extend(vec![
                    "Check agent configuration".to_string(),
                    "Verify all required tools are available".to_string(),
                    "Review agent instructions for clarity".to_string(),
                ]);
            },
            _ => {
                self.suggestions.push("Check the documentation for more information".to_string());
            }
        }
    }

    /// Format error for display
    pub fn format_for_display(&self) -> String {
        let mut output = String::new();
        
        // Error header with emoji
        let emoji = match self.severity {
            ErrorSeverity::Low => "⚠️",
            ErrorSeverity::Medium => "❌",
            ErrorSeverity::High => "🚨",
            ErrorSeverity::Critical => "💥",
        };
        
        output.push_str(&format!("{} {}\n", emoji, self.message));
        
        // Context information
        if !self.context.is_empty() {
            output.push_str("\n📋 Context:\n");
            for (key, value) in &self.context {
                output.push_str(&format!("  • {}: {}\n", key, value));
            }
        }
        
        // Suggestions
        if !self.suggestions.is_empty() {
            output.push_str("\n💡 Suggestions:\n");
            for suggestion in &self.suggestions {
                output.push_str(&format!("  • {}\n", suggestion));
            }
        }
        
        // Technical details (for debugging)
        output.push_str(&format!("\n🔧 Technical Details:\n  • Category: {:?}\n  • Severity: {:?}\n  • Error: {}\n", 
            self.category, self.severity, self.error));
        
        output
    }

    /// Get error code for programmatic handling
    pub fn error_code(&self) -> String {
        format!("{:?}_{:?}", self.category, self.severity).to_uppercase()
    }
}

impl fmt::Display for FriendlyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_for_display())
    }
}

impl std::error::Error for FriendlyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Helper functions for creating common friendly errors
pub mod helpers {
    use super::*;

    /// Create a configuration error with helpful context
    pub fn config_error(message: &str, config_path: Option<&str>) -> FriendlyError {
        let mut error = FriendlyError::new(
            Error::Configuration(message.to_string()),
            format!("Configuration Error: {}", message)
        );
        
        if let Some(path) = config_path {
            error = error.with_context("config_file", path);
        }
        
        error.generate_suggestions();
        error
    }

    /// Create a tool error with usage hints
    pub fn tool_error(tool_name: &str, message: &str, params: Option<&Value>) -> FriendlyError {
        let mut error = FriendlyError::new(
            Error::Tool(message.to_string()),
            format!("Tool '{}' Error: {}", tool_name, message)
        )
        .with_context("tool_name", tool_name);
        
        if let Some(params) = params {
            error = error.with_context("parameters", params.clone());
        }
        
        error.generate_suggestions();
        error
    }

    /// Create an agent error with debugging info
    pub fn agent_error(agent_name: &str, message: &str) -> FriendlyError {
        let mut error = FriendlyError::new(
            Error::Agent(message.to_string()),
            format!("Agent '{}' Error: {}", agent_name, message)
        )
        .with_context("agent_name", agent_name);
        
        error.generate_suggestions();
        error
    }

    /// Create a network error with retry suggestions
    pub fn network_error(endpoint: &str, status_code: Option<u16>) -> FriendlyError {
        let message = if let Some(code) = status_code {
            format!("Network request failed with status {}", code)
        } else {
            "Network request failed".to_string()
        };
        
        let mut error = FriendlyError::new(
            Error::Network(message.clone()),
            format!("Network Error: {}", message)
        )
        .with_context("endpoint", endpoint);
        
        if let Some(code) = status_code {
            error = error.with_context("status_code", code);
        }
        
        error.generate_suggestions();
        error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_friendly_error_creation() {
        let error = FriendlyError::new(
            Error::Configuration("Missing API key".to_string()),
            "Configuration is invalid".to_string()
        );
        
        assert_eq!(error.category, ErrorCategory::Configuration);
        assert_eq!(error.severity, ErrorSeverity::Medium);
        assert_eq!(error.message, "Configuration is invalid");
    }

    #[test]
    fn test_error_with_context() {
        let error = FriendlyError::new(
            Error::Tool("Invalid parameter".to_string()),
            "Tool execution failed".to_string()
        )
        .with_context("tool_name", "calculator")
        .with_context("parameter", "invalid_value");
        
        assert_eq!(error.context.len(), 2);
        assert_eq!(error.context["tool_name"], "calculator");
    }

    #[test]
    fn test_suggestion_generation() {
        let mut error = FriendlyError::new(
            Error::Authentication("Invalid token".to_string()),
            "Authentication failed".to_string()
        );
        
        error.generate_suggestions();
        assert!(!error.suggestions.is_empty());
        assert!(error.suggestions.iter().any(|s| s.contains("API keys")));
    }

    #[test]
    fn test_helper_functions() {
        let config_error = helpers::config_error("Missing field", Some("/path/to/config"));
        assert_eq!(config_error.category, ErrorCategory::Configuration);
        assert!(config_error.context.contains_key("config_file"));
        
        let tool_error = helpers::tool_error("calculator", "Division by zero", None);
        assert_eq!(tool_error.category, ErrorCategory::Tool);
        assert!(tool_error.context.contains_key("tool_name"));
    }

    #[test]
    fn test_error_formatting() {
        let error = FriendlyError::new(
            Error::Network("Connection timeout".to_string()),
            "Failed to connect to service".to_string()
        )
        .with_context("endpoint", "https://api.example.com")
        .with_suggestion("Check your internet connection");
        
        let formatted = error.format_for_display();
        assert!(formatted.contains("❌"));
        assert!(formatted.contains("Context:"));
        assert!(formatted.contains("Suggestions:"));
        assert!(formatted.contains("Technical Details:"));
    }
}
