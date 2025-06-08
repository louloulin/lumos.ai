//! Enhanced tool system based on Rig's design
//! 
//! Provides advanced tool capabilities with dynamic dispatch,
//! streaming support, and enhanced metadata.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use futures::Stream;

use crate::{Result, Error};
use crate::agent::types::RuntimeContext;
// Note: We define our own types to avoid conflicts with existing tool system

/// Enhanced tool trait with additional capabilities
#[async_trait]
pub trait EnhancedTool: Send + Sync {
    /// Get tool category
    fn category(&self) -> ToolCategory {
        ToolCategory::General
    }
    
    /// Get tool capabilities
    fn capabilities(&self) -> Vec<ToolCapability> {
        vec![ToolCapability::Basic]
    }
    
    /// Get tool configuration schema
    fn config_schema(&self) -> Option<Value> {
        None
    }
    
    /// Configure the tool with settings
    async fn configure(&mut self, config: Value) -> Result<()> {
        // Default implementation ignores configuration
        let _ = config;
        Ok(())
    }
    
    /// Validate tool arguments before execution
    async fn validate_args(&self, args: &Value) -> Result<()> {
        // Default implementation accepts all arguments
        let _ = args;
        Ok(())
    }
    
    /// Get tool usage statistics
    fn get_stats(&self) -> ToolStats {
        ToolStats::default()
    }
    
    /// Reset tool state
    async fn reset(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Check if tool is healthy
    async fn health_check(&self) -> Result<ToolHealth> {
        Ok(ToolHealth {
            status: HealthStatus::Healthy,
            message: None,
            last_check: chrono::Utc::now(),
        })
    }
}

/// Tool category enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolCategory {
    /// General purpose tools
    General,
    /// Web and HTTP tools
    Web,
    /// File system tools
    FileSystem,
    /// Database tools
    Database,
    /// AI and ML tools
    AI,
    /// Communication tools
    Communication,
    /// Data processing tools
    DataProcessing,
    /// System tools
    System,
    /// Math and calculation tools
    Math,
    /// Custom tools
    Custom(String),
}

/// Tool capability enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolCapability {
    /// Basic execution
    Basic,
    /// Streaming support
    Streaming,
    /// Async execution
    Async,
    /// Batch processing
    Batch,
    /// Caching
    Caching,
    /// Rate limiting
    RateLimit,
    /// Authentication
    Auth,
    /// Encryption
    Encryption,
    /// Monitoring
    Monitoring,
    /// Custom capability
    Custom(String),
}

/// Tool usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStats {
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Average execution time (milliseconds)
    pub avg_execution_time_ms: f64,
    /// Total execution time (milliseconds)
    pub total_execution_time_ms: u64,
    /// Last execution timestamp
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for ToolStats {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            avg_execution_time_ms: 0.0,
            total_execution_time_ms: 0,
            last_execution: None,
        }
    }
}

/// Tool health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolHealth {
    /// Health status
    pub status: HealthStatus,
    /// Status message
    pub message: Option<String>,
    /// Last check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Tool is healthy
    Healthy,
    /// Tool has warnings
    Warning,
    /// Tool is unhealthy
    Unhealthy,
    /// Tool status is unknown
    Unknown,
}

/// Streaming tool trait
#[async_trait]
pub trait StreamingTool: EnhancedTool {
    /// Execute tool with streaming output
    async fn execute_stream(&self, args: Value, context: &RuntimeContext) -> Result<Box<dyn Stream<Item = Result<Value>> + Send + Unpin>>;
    
    /// Get streaming configuration
    fn streaming_config(&self) -> StreamingConfig {
        StreamingConfig::default()
    }
}

/// Streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Buffer size for streaming
    pub buffer_size: usize,
    /// Chunk size for streaming
    pub chunk_size: usize,
    /// Timeout for streaming (milliseconds)
    pub timeout_ms: u64,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024,
            chunk_size: 256,
            timeout_ms: 30000,
        }
    }
}

/// Batch processing tool trait
#[async_trait]
pub trait BatchTool: EnhancedTool {
    /// Execute tool with batch input
    async fn execute_batch(&self, batch_args: Vec<Value>, context: &RuntimeContext) -> Result<Vec<Result<Value>>>;
    
    /// Get batch configuration
    fn batch_config(&self) -> BatchConfig {
        BatchConfig::default()
    }
}

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Parallel execution count
    pub parallel_count: usize,
    /// Timeout for batch processing (milliseconds)
    pub timeout_ms: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            parallel_count: 4,
            timeout_ms: 60000,
        }
    }
}

/// Cacheable tool trait
#[async_trait]
pub trait CacheableTool: EnhancedTool {
    /// Get cache key for given arguments
    fn cache_key(&self, args: &Value) -> String;
    
    /// Get cache TTL (seconds)
    fn cache_ttl(&self) -> u64 {
        3600 // 1 hour default
    }
    
    /// Check if result should be cached
    fn should_cache(&self, args: &Value, result: &Value) -> bool {
        let _ = (args, result);
        true
    }
}

/// Rate limited tool trait
#[async_trait]
pub trait RateLimitedTool: EnhancedTool {
    /// Get rate limit configuration
    fn rate_limit(&self) -> RateLimit {
        RateLimit::default()
    }
    
    /// Check if execution is allowed
    async fn check_rate_limit(&self, context: &RuntimeContext) -> Result<bool>;
}

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window (seconds)
    pub window_seconds: u64,
    /// Burst allowance
    pub burst_allowance: u32,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
            burst_allowance: 10,
        }
    }
}

/// Tool wrapper that adds enhanced capabilities
#[derive(Debug)]
pub struct EnhancedToolWrapper<T> {
    /// Inner tool
    inner: T,
    /// Tool statistics
    stats: Arc<tokio::sync::RwLock<ToolStats>>,
    /// Tool configuration
    config: Arc<tokio::sync::RwLock<Value>>,
    /// Tool category
    category: ToolCategory,
    /// Tool capabilities
    capabilities: Vec<ToolCapability>,
}

impl<T> EnhancedToolWrapper<T> {
    /// Create a new enhanced tool wrapper
    pub fn new(inner: T, category: ToolCategory, capabilities: Vec<ToolCapability>) -> Self {
        Self {
            inner,
            stats: Arc::new(tokio::sync::RwLock::new(ToolStats::default())),
            config: Arc::new(tokio::sync::RwLock::new(Value::Null)),
            category,
            capabilities,
        }
    }
    

}

impl<T> EnhancedToolWrapper<T> {
    /// Get tool category
    pub fn category(&self) -> &ToolCategory {
        &self.category
    }

    /// Get tool capabilities
    pub fn capabilities(&self) -> &[ToolCapability] {
        &self.capabilities
    }

    /// Configure the tool
    pub async fn configure(&mut self, config: Value) -> Result<()> {
        *self.config.write().await = config;
        Ok(())
    }

    /// Get tool statistics
    pub async fn get_stats(&self) -> ToolStats {
        self.stats.read().await.clone()
    }

    /// Health check
    pub async fn health_check(&self) -> Result<ToolHealth> {
        Ok(ToolHealth {
            status: HealthStatus::Healthy,
            message: Some("Tool is operational".to_string()),
            last_check: chrono::Utc::now(),
        })
    }

    /// Update statistics after execution
    pub async fn update_stats(&self, success: bool, execution_time_ms: u64) {
        let mut stats = self.stats.write().await;
        stats.total_executions += 1;
        stats.total_execution_time_ms += execution_time_ms;
        stats.last_execution = Some(chrono::Utc::now());

        if success {
            stats.successful_executions += 1;
        } else {
            stats.failed_executions += 1;
        }

        stats.avg_execution_time_ms =
            stats.total_execution_time_ms as f64 / stats.total_executions as f64;
    }
}
