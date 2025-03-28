//! Logger module for structured logging

use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::types::Metadata;

/// Log level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Debug level - detailed information for debugging
    Debug,
    /// Info level - general information about program execution
    Info,
    /// Warn level - potentially harmful situations
    Warn,
    /// Error level - error events that might still allow the application to continue running
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

/// Component identifiers for logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Component {
    /// LLM component
    Llm,
    /// Agent component
    Agent,
    /// Memory component
    Memory,
    /// Storage component
    Storage,
    /// Tool component
    Tool,
    /// Vector component
    Vector,
    /// Workflow component
    Workflow,
    /// Telemetry component
    Telemetry,
    /// Network component
    Network,
}

impl Default for Component {
    fn default() -> Self {
        Component::Llm
    }
}

impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Component::Llm => write!(f, "LLM"),
            Component::Agent => write!(f, "AGENT"),
            Component::Memory => write!(f, "MEMORY"),
            Component::Storage => write!(f, "STORAGE"),
            Component::Tool => write!(f, "TOOL"),
            Component::Vector => write!(f, "VECTOR"),
            Component::Workflow => write!(f, "WORKFLOW"),
            Component::Telemetry => write!(f, "TELEMETRY"),
            Component::Network => write!(f, "NETWORK"),
        }
    }
}

/// Logger trait - defines methods for structured logging
pub trait Logger: Send + Sync {
    /// Log a debug message
    fn debug(&self, message: &str, metadata: Option<Metadata>);
    
    /// Log an info message
    fn info(&self, message: &str, metadata: Option<Metadata>);
    
    /// Log a warning message
    fn warn(&self, message: &str, metadata: Option<Metadata>);
    
    /// Log an error message
    fn error(&self, message: &str, metadata: Option<Metadata>);
    
    /// Get logs by run ID
    fn get_logs_by_run_id(&self, run_id: &str) -> Vec<LogEntry>;
}

/// Log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp of the log entry
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Log level
    pub level: LogLevel,
    /// Component that generated the log
    pub component: Component,
    /// Name of the specific instance/object
    pub name: String,
    /// Log message
    pub message: String,
    /// Additional metadata
    pub metadata: Option<Metadata>,
}

/// Simple console logger implementation
#[derive(Clone)]
pub struct ConsoleLogger {
    name: String,
    component: Component,
    level: LogLevel,
}

impl ConsoleLogger {
    /// Create a new console logger
    pub fn new(name: impl Into<String>, component: Component, level: LogLevel) -> Self {
        Self {
            name: name.into(),
            component,
            level,
        }
    }
}

impl Logger for ConsoleLogger {
    fn debug(&self, message: &str, metadata: Option<Metadata>) {
        if self.level <= LogLevel::Debug {
            println!("[DEBUG][{}][{}] {}", self.component, self.name, message);
            if let Some(meta) = metadata {
                println!("  Metadata: {:?}", meta);
            }
        }
    }
    
    fn info(&self, message: &str, metadata: Option<Metadata>) {
        if self.level <= LogLevel::Info {
            println!("[INFO][{}][{}] {}", self.component, self.name, message);
            if let Some(meta) = metadata {
                println!("  Metadata: {:?}", meta);
            }
        }
    }
    
    fn warn(&self, message: &str, metadata: Option<Metadata>) {
        if self.level <= LogLevel::Warn {
            println!("[WARN][{}][{}] {}", self.component, self.name, message);
            if let Some(meta) = metadata {
                println!("  Metadata: {:?}", meta);
            }
        }
    }
    
    fn error(&self, message: &str, metadata: Option<Metadata>) {
        if self.level <= LogLevel::Error {
            println!("[ERROR][{}][{}] {}", self.component, self.name, message);
            if let Some(meta) = metadata {
                println!("  Metadata: {:?}", meta);
            }
        }
    }
    
    fn get_logs_by_run_id(&self, _run_id: &str) -> Vec<LogEntry> {
        // ConsoleLogger doesn't store logs, so we return an empty vector
        Vec::new()
    }
}

/// Create a default console logger
pub fn create_logger(name: impl Into<String>, component: Component, level: LogLevel) -> Arc<dyn Logger> {
    Arc::new(ConsoleLogger::new(name, component, level))
}

/// No-op logger that doesn't log anything
#[derive(Clone)]
pub struct NoopLogger;

impl Logger for NoopLogger {
    fn debug(&self, _message: &str, _metadata: Option<Metadata>) {}
    fn info(&self, _message: &str, _metadata: Option<Metadata>) {}
    fn warn(&self, _message: &str, _metadata: Option<Metadata>) {}
    fn error(&self, _message: &str, _metadata: Option<Metadata>) {}
    fn get_logs_by_run_id(&self, _run_id: &str) -> Vec<LogEntry> { Vec::new() }
}

/// Create a no-op logger
pub fn create_noop_logger() -> Arc<dyn Logger> {
    Arc::new(NoopLogger)
} 