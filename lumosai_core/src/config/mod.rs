//! Configuration management for LumosAI
//!
//! This module provides unified configuration loading and management,
//! supporting multiple sources (environment variables, files, secrets, etc.)
//! with validation and hot-reloading capabilities.

pub mod format;
pub mod loader;
pub mod merge;
pub mod simplified;
pub mod source;
pub mod types;
pub mod validator;
pub mod yaml_config;

#[cfg(test)]
mod real_api_tests;

// Re-export with specific imports to avoid conflicts
pub use format::ConfigFormat;
pub use loader::{ConfigLoader, ConfigSource, MergeStrategy};
pub use types::*;
pub use validator::{ConfigValidator, ValidationMessage, ValidationSeverity};

// Re-export simplified configuration system
pub use simplified::{LumosConfigBuilder, ConfigMode, ConfigOptimizer, ValidationIssue, IssueLevel};

// Re-export quick configuration functions
pub use simplified::quick_config;

// Re-export only specific items from yaml_config to avoid conflicts
pub use yaml_config::{AgentConfig, WorkflowConfig, YamlConfig};
