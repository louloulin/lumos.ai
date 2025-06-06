//! Enhanced error handling for CLI operations
//! 
//! This module provides user-friendly error messages and debugging tools

use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Enhanced CLI errors with helpful suggestions
#[derive(Debug, Error)]
pub enum LumosCliError {
    #[error("🤖 Agent '{name}' not found\n💡 Available agents: {available:?}\n🔧 Try: lumos list agents")]
    AgentNotFound { 
        name: String, 
        available: Vec<String> 
    },
    
    #[error("🔧 Tool '{tool}' execution failed\n❌ Error: {cause}\n💡 Suggestion: {suggestion}\n📚 Docs: {docs_url}")]
    ToolExecutionFailed { 
        tool: String, 
        cause: String, 
        suggestion: String,
        docs_url: String 
    },
    
    #[error("⚙️ Configuration error in {section}\n❌ Issue: {issue}\n✅ Expected: {expected}\n🔧 Fix: {fix_command}")]
    ConfigurationError {
        section: String,
        issue: String,
        expected: String,
        fix_command: String,
    },

    #[error("📦 Project not found\n💡 Make sure you're in a Lumos.ai project directory\n🔧 Try: lumos new my-project")]
    ProjectNotFound,

    #[error("🌐 Network error: {message}\n💡 Check your internet connection\n🔧 Try: ping marketplace.lumos.ai")]
    NetworkError { message: String },

    #[error("🔐 Authentication failed\n💡 Check your API credentials\n🔧 Try: lumos auth login")]
    AuthenticationFailed,

    #[error("📁 File system error: {message}\n💡 Check file permissions\n🔧 Path: {path}")]
    FileSystemError { message: String, path: String },

    #[error("🚀 Deployment failed to {platform}\n❌ Error: {error}\n💡 Suggestion: {suggestion}")]
    DeploymentFailed {
        platform: String,
        error: String,
        suggestion: String,
    },

    #[error("🧪 Test execution failed\n❌ {details}\n💡 Run with --verbose for more details")]
    TestExecutionFailed { details: String },

    #[error("🔄 Hot reload error: {message}\n💡 Try restarting the dev server")]
    HotReloadError { message: String },
}

/// Error context for better debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub command: String,
    pub working_directory: String,
    pub environment: HashMap<String, String>,
    pub project_info: Option<ProjectInfo>,
    pub system_info: SystemInfo,
}

/// Project information for error context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub version: String,
    pub tools_count: usize,
    pub config_path: String,
}

/// System information for error context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub rust_version: String,
    pub lumos_version: String,
}

/// Enhanced error with context
#[derive(Debug)]
pub struct EnhancedError {
    pub error: LumosCliError,
    pub context: ErrorContext,
    pub suggestions: Vec<String>,
    pub related_docs: Vec<String>,
}

impl fmt::Display for EnhancedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.error)?;
        
        if !self.suggestions.is_empty() {
            writeln!(f, "\n🔧 Additional suggestions:")?;
            for suggestion in &self.suggestions {
                writeln!(f, "  • {}", suggestion)?;
            }
        }
        
        if !self.related_docs.is_empty() {
            writeln!(f, "\n📚 Related documentation:")?;
            for doc in &self.related_docs {
                writeln!(f, "  • {}", doc)?;
            }
        }
        
        writeln!(f, "\n🐛 For more help, run: lumos help debug")?;
        
        Ok(())
    }
}

/// Error handler for CLI operations
pub struct ErrorHandler {
    pub debug_mode: bool,
    pub collect_telemetry: bool,
}

impl ErrorHandler {
    /// Create a new error handler
    pub fn new(debug_mode: bool) -> Self {
        Self {
            debug_mode,
            collect_telemetry: false, // Disabled by default for privacy
        }
    }

    /// Handle an error with enhanced context
    pub fn handle_error(&self, error: LumosCliError, command: &str) -> EnhancedError {
        let context = self.collect_error_context(command);
        let suggestions = self.generate_suggestions(&error);
        let related_docs = self.get_related_docs(&error);

        EnhancedError {
            error,
            context,
            suggestions,
            related_docs,
        }
    }

    /// Collect error context
    fn collect_error_context(&self, command: &str) -> ErrorContext {
        let mut environment = HashMap::new();
        
        // Collect relevant environment variables
        if let Ok(path) = std::env::var("PATH") {
            environment.insert("PATH".to_string(), path);
        }
        if let Ok(home) = std::env::var("HOME") {
            environment.insert("HOME".to_string(), home);
        }

        let working_directory = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let project_info = self.collect_project_info();
        let system_info = self.collect_system_info();

        ErrorContext {
            timestamp: chrono::Utc::now(),
            command: command.to_string(),
            working_directory,
            environment,
            project_info,
            system_info,
        }
    }

    /// Collect project information
    fn collect_project_info(&self) -> Option<ProjectInfo> {
        use crate::cli::CliUtils;

        if let Some(project_root) = CliUtils::find_project_root(".") {
            let config_path = project_root.join("lumos.toml");
            if let Ok(config) = CliUtils::load_config(&config_path) {
                return Some(ProjectInfo {
                    name: config.name,
                    version: config.version,
                    tools_count: config.tools.len(),
                    config_path: config_path.to_string_lossy().to_string(),
                });
            }
        }
        None
    }

    /// Collect system information
    fn collect_system_info(&self) -> SystemInfo {
        SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            rust_version: "1.70+".to_string(), // Would be detected dynamically
            lumos_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Generate helpful suggestions based on error type
    fn generate_suggestions(&self, error: &LumosCliError) -> Vec<String> {
        match error {
            LumosCliError::ProjectNotFound => vec![
                "Initialize a new project with 'lumos new my-project'".to_string(),
                "Navigate to an existing Lumos.ai project directory".to_string(),
                "Check if lumos.toml exists in the current directory".to_string(),
            ],
            LumosCliError::NetworkError { .. } => vec![
                "Check your internet connection".to_string(),
                "Verify firewall settings".to_string(),
                "Try using a VPN if behind corporate firewall".to_string(),
                "Check if marketplace.lumos.ai is accessible".to_string(),
            ],
            LumosCliError::AuthenticationFailed => vec![
                "Run 'lumos auth login' to authenticate".to_string(),
                "Check if your API key is valid".to_string(),
                "Verify your account has the necessary permissions".to_string(),
            ],
            LumosCliError::ToolExecutionFailed { .. } => vec![
                "Check tool configuration in lumos.toml".to_string(),
                "Verify tool dependencies are installed".to_string(),
                "Try updating the tool to the latest version".to_string(),
                "Run with --debug for detailed execution logs".to_string(),
            ],
            _ => vec![
                "Run with --verbose for more detailed output".to_string(),
                "Check the Lumos.ai documentation".to_string(),
                "Report this issue on GitHub if it persists".to_string(),
            ],
        }
    }

    /// Get related documentation links
    fn get_related_docs(&self, error: &LumosCliError) -> Vec<String> {
        match error {
            LumosCliError::ProjectNotFound => vec![
                "https://docs.lumos.ai/getting-started".to_string(),
                "https://docs.lumos.ai/project-structure".to_string(),
            ],
            LumosCliError::ToolExecutionFailed { .. } => vec![
                "https://docs.lumos.ai/tools".to_string(),
                "https://docs.lumos.ai/troubleshooting".to_string(),
            ],
            LumosCliError::ConfigurationError { .. } => vec![
                "https://docs.lumos.ai/configuration".to_string(),
                "https://docs.lumos.ai/lumos-toml".to_string(),
            ],
            LumosCliError::DeploymentFailed { .. } => vec![
                "https://docs.lumos.ai/deployment".to_string(),
                "https://docs.lumos.ai/production".to_string(),
            ],
            _ => vec![
                "https://docs.lumos.ai".to_string(),
                "https://github.com/lumosai/lumos/issues".to_string(),
            ],
        }
    }

    /// Print error with enhanced formatting
    pub fn print_error(&self, enhanced_error: &EnhancedError) {
        eprintln!("{}", enhanced_error);
        
        if self.debug_mode {
            eprintln!("\n🐛 Debug Information:");
            eprintln!("  Command: {}", enhanced_error.context.command);
            eprintln!("  Working Directory: {}", enhanced_error.context.working_directory);
            eprintln!("  Timestamp: {}", enhanced_error.context.timestamp);
            
            if let Some(ref project_info) = enhanced_error.context.project_info {
                eprintln!("  Project: {} v{}", project_info.name, project_info.version);
                eprintln!("  Tools: {}", project_info.tools_count);
            }
            
            eprintln!("  System: {} {}", enhanced_error.context.system_info.os, enhanced_error.context.system_info.arch);
        }
    }
}

/// Convenience functions for creating common errors
impl LumosCliError {
    pub fn project_not_found() -> Self {
        Self::ProjectNotFound
    }

    pub fn network_error(message: impl Into<String>) -> Self {
        Self::NetworkError { 
            message: message.into() 
        }
    }

    pub fn tool_execution_failed(
        tool: impl Into<String>, 
        cause: impl Into<String>,
        suggestion: impl Into<String>
    ) -> Self {
        Self::ToolExecutionFailed {
            tool: tool.into(),
            cause: cause.into(),
            suggestion: suggestion.into(),
            docs_url: "https://docs.lumos.ai/tools".to_string(),
        }
    }

    pub fn config_error(
        section: impl Into<String>,
        issue: impl Into<String>,
        expected: impl Into<String>,
        fix_command: impl Into<String>
    ) -> Self {
        Self::ConfigurationError {
            section: section.into(),
            issue: issue.into(),
            expected: expected.into(),
            fix_command: fix_command.into(),
        }
    }
}
