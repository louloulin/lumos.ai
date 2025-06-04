//! Enhanced logging system inspired by Mastra's observability features
//! 
//! This module provides structured logging, metrics, and observability tools

use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Log levels with semantic meaning
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            LogLevel::Trace => "🔍",
            LogLevel::Debug => "🐛",
            LogLevel::Info => "ℹ️",
            LogLevel::Warn => "⚠️",
            LogLevel::Error => "❌",
            LogLevel::Fatal => "💥",
        }
    }
}

/// Structured log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: u64,
    pub level: LogLevel,
    pub message: String,
    pub module: String,
    pub fields: HashMap<String, Value>,
    pub tags: Vec<String>,
    pub correlation_id: Option<String>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, message: String, module: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            level,
            message,
            module,
            fields: HashMap::new(),
            tags: Vec::new(),
            correlation_id: None,
        }
    }

    /// Add a field
    pub fn with_field<K: Into<String>, V: Into<Value>>(mut self, key: K, value: V) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// Add multiple fields
    pub fn with_fields(mut self, fields: HashMap<String, Value>) -> Self {
        self.fields.extend(fields);
        self
    }

    /// Add a tag
    pub fn with_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags
    pub fn with_tags<I, S>(mut self, tags: I) -> Self 
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags.extend(tags.into_iter().map(|s| s.into()));
        self
    }

    /// Set correlation ID for tracing
    pub fn with_correlation_id<S: Into<String>>(mut self, id: S) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Format for console output
    pub fn format_console(&self) -> String {
        let timestamp = chrono::DateTime::from_timestamp_millis(self.timestamp as i64)
            .unwrap_or_default()
            .format("%Y-%m-%d %H:%M:%S%.3f");

        let mut output = format!(
            "{} {} [{}] {}",
            timestamp,
            self.level.emoji(),
            self.module,
            self.message
        );

        // Add fields if present
        if !self.fields.is_empty() {
            output.push_str(" | ");
            let field_strs: Vec<String> = self.fields
                .iter()
                .map(|(k, v)| {
                    // Format values without JSON quotes for strings
                    let formatted_value = match v {
                        Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    };
                    format!("{}={}", k, formatted_value)
                })
                .collect();
            output.push_str(&field_strs.join(" "));
        }

        // Add tags if present
        if !self.tags.is_empty() {
            output.push_str(&format!(" [{}]", self.tags.join(", ")));
        }

        // Add correlation ID if present
        if let Some(correlation_id) = &self.correlation_id {
            output.push_str(&format!(" ({})", correlation_id));
        }

        output
    }

    /// Format as JSON
    pub fn format_json(&self) -> Value {
        json!({
            "id": self.id,
            "timestamp": self.timestamp,
            "level": self.level.as_str(),
            "message": self.message,
            "module": self.module,
            "fields": self.fields,
            "tags": self.tags,
            "correlation_id": self.correlation_id
        })
    }
}

/// Logger configuration
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub min_level: LogLevel,
    pub format: LogFormat,
    pub include_module: bool,
    pub include_timestamp: bool,
    pub include_correlation_id: bool,
    pub color_enabled: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            min_level: LogLevel::Info,
            format: LogFormat::Console,
            include_module: true,
            include_timestamp: true,
            include_correlation_id: true,
            color_enabled: true,
        }
    }
}

/// Log output formats
#[derive(Debug, Clone)]
pub enum LogFormat {
    Console,
    Json,
    Structured,
}

/// Enhanced logger with Mastra-style features
pub struct Logger {
    config: LoggerConfig,
    correlation_id: Option<String>,
    module: String,
}

impl Logger {
    /// Create a new logger for a module
    pub fn new(module: &str) -> Self {
        Self {
            config: LoggerConfig::default(),
            correlation_id: None,
            module: module.to_string(),
        }
    }

    /// Create logger with custom config
    pub fn with_config(module: &str, config: LoggerConfig) -> Self {
        Self {
            config,
            correlation_id: None,
            module: module.to_string(),
        }
    }

    /// Set correlation ID for request tracing
    pub fn with_correlation_id<S: Into<String>>(mut self, id: S) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Get the module name
    pub fn get_module(&self) -> &str {
        &self.module
    }

    /// Get the correlation ID
    pub fn get_correlation_id(&self) -> Option<&str> {
        self.correlation_id.as_deref()
    }

    /// Get the logger config
    pub fn get_config(&self) -> &LoggerConfig {
        &self.config
    }

    /// Log at trace level
    pub fn trace(&self, message: &str) {
        self.log(LogLevel::Trace, message, HashMap::new());
    }

    /// Log at debug level
    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message, HashMap::new());
    }

    /// Log at info level
    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message, HashMap::new());
    }

    /// Log at warn level
    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message, HashMap::new());
    }

    /// Log at error level
    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message, HashMap::new());
    }

    /// Log at fatal level
    pub fn fatal(&self, message: &str) {
        self.log(LogLevel::Fatal, message, HashMap::new());
    }

    /// Log with fields
    pub fn log_with_fields(&self, level: LogLevel, message: &str, fields: HashMap<String, Value>) {
        self.log(level, message, fields);
    }

    /// Internal log method
    fn log(&self, level: LogLevel, message: &str, fields: HashMap<String, Value>) {
        if level < self.config.min_level {
            return;
        }

        let mut entry = LogEntry::new(level, message.to_string(), self.module.clone())
            .with_fields(fields);

        if let Some(correlation_id) = &self.correlation_id {
            entry = entry.with_correlation_id(correlation_id.clone());
        }

        self.output_entry(&entry);
    }

    /// Output log entry based on configuration
    fn output_entry(&self, entry: &LogEntry) {
        match self.config.format {
            LogFormat::Console => {
                println!("{}", entry.format_console());
            },
            LogFormat::Json => {
                println!("{}", entry.format_json());
            },
            LogFormat::Structured => {
                // Custom structured format
                println!("{}", entry.format_console());
            }
        }
    }
}

/// Convenience macros for logging
#[macro_export]
macro_rules! log_trace {
    ($logger:expr, $($arg:tt)*) => {
        $logger.trace(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($logger:expr, $($arg:tt)*) => {
        $logger.debug(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($logger:expr, $($arg:tt)*) => {
        $logger.info(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($logger:expr, $($arg:tt)*) => {
        $logger.warn(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($logger:expr, $($arg:tt)*) => {
        $logger.error(&format!($($arg)*))
    };
}

/// Agent-specific logging utilities
pub mod agent_logging {
    use super::*;
    use crate::agent::BasicAgent;
    use crate::agent::trait_def::Agent;

    /// Create a logger for an agent
    pub fn create_agent_logger(agent: &BasicAgent) -> Logger {
        Logger::new(&format!("agent::{}", agent.get_name()))
    }

    /// Log agent creation
    pub fn log_agent_created(logger: &Logger, agent_name: &str, config: &Value) {
        logger.log_with_fields(
            LogLevel::Info,
            &format!("Agent '{}' created", agent_name),
            [
                ("agent_name".to_string(), json!(agent_name)),
                ("config".to_string(), config.clone()),
            ].into_iter().collect()
        );
    }

    /// Log tool execution
    pub fn log_tool_execution(
        logger: &Logger, 
        tool_name: &str, 
        duration_ms: u64, 
        success: bool
    ) {
        let level = if success { LogLevel::Info } else { LogLevel::Error };
        let message = if success {
            format!("Tool '{}' executed successfully", tool_name)
        } else {
            format!("Tool '{}' execution failed", tool_name)
        };

        logger.log_with_fields(
            level,
            &message,
            [
                ("tool_name".to_string(), json!(tool_name)),
                ("duration_ms".to_string(), json!(duration_ms)),
                ("success".to_string(), json!(success)),
            ].into_iter().collect()
        );
    }

    /// Log message processing
    pub fn log_message_processing(
        logger: &Logger,
        input_count: usize,
        output_length: usize,
        duration_ms: u64,
        token_count: Option<u32>
    ) {
        let mut fields = [
            ("input_count".to_string(), json!(input_count)),
            ("output_length".to_string(), json!(output_length)),
            ("duration_ms".to_string(), json!(duration_ms)),
        ].into_iter().collect::<HashMap<_, _>>();

        if let Some(tokens) = token_count {
            fields.insert("token_count".to_string(), json!(tokens));
        }

        logger.log_with_fields(
            LogLevel::Info,
            "Message processed",
            fields
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(
            LogLevel::Info,
            "Test message".to_string(),
            "test_module".to_string()
        );

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.module, "test_module");
        assert!(!entry.id.is_empty());
    }

    #[test]
    fn test_log_entry_with_fields() {
        let entry = LogEntry::new(
            LogLevel::Debug,
            "Debug message".to_string(),
            "debug_module".to_string()
        )
        .with_field("key1", "value1")
        .with_field("key2", 42)
        .with_tag("test")
        .with_correlation_id("corr-123");

        assert_eq!(entry.fields.len(), 2);
        assert_eq!(entry.tags.len(), 1);
        assert_eq!(entry.correlation_id, Some("corr-123".to_string()));
    }

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new("test_module");
        assert_eq!(logger.module, "test_module");
        assert_eq!(logger.config.min_level, LogLevel::Info);
    }

    #[test]
    fn test_log_levels() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
    }

    #[test]
    fn test_json_formatting() {
        let entry = LogEntry::new(
            LogLevel::Error,
            "Error occurred".to_string(),
            "error_module".to_string()
        )
        .with_field("error_code", 500);

        let json = entry.format_json();
        assert_eq!(json["level"], "ERROR");
        assert_eq!(json["message"], "Error occurred");
        assert_eq!(json["fields"]["error_code"], 500);
    }
}
