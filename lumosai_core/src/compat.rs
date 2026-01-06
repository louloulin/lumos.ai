//! 兼容性模块 - 为迁移期间提供临时的类型定义
//! 这些类型将在 v2.0 重构完成后被移除

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 临时的组件类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Component {
    #[default]
    Agent,
    Tool,
    Memory,
    Workflow,
    Storage,
    Vector,
    Llm,
}

impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Component::Agent => write!(f, "Agent"),
            Component::Tool => write!(f, "Tool"),
            Component::Memory => write!(f, "Memory"),
            Component::Workflow => write!(f, "Workflow"),
            Component::Storage => write!(f, "Storage"),
            Component::Vector => write!(f, "Vector"),
            Component::Llm => write!(f, "Llm"),
        }
    }
}

/// 临时的日志级别枚举
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

/// 临时的日志器 trait
#[async_trait]
pub trait Logger: Send + Sync {
    fn log(&self, level: LogLevel, message: &str);
    fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }
    fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }
    fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }
    fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }
}

/// 临时的遥测 trait
#[async_trait]
pub trait TelemetrySink: Send + Sync {
    async fn send_event(&self, event: Event);
    fn record_event(&self, event: serde_json::Value);
}

/// 临时的存储 trait
#[async_trait]
pub trait Storage: Send + Sync {
    async fn get(
        &self,
        key: &str,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>>;
    async fn set(
        &self,
        key: &str,
        value: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 临时的指标收集器 trait
#[async_trait]
pub trait MetricsCollector: Send + Sync {
    async fn record_metric(&self, name: &str, value: f64);
    async fn record_memory_operation(
        &self,
        metrics: MemoryMetrics,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn record_tool_execution(
        &self,
        metrics: ToolMetrics,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn record_agent_execution(
        &self,
        metrics: AgentMetrics,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 临时的追踪收集器 trait
#[async_trait]
pub trait TraceCollector: Send + Sync {
    async fn start_trace(&self, name: &str) -> String;
    async fn add_trace_step(
        &self,
        trace_id: &str,
        step: TraceStep,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn end_trace(&self, trace_id: &str);
}

/// 简单的日志器实现
pub struct SimpleLogger {
    name: String,
}

impl SimpleLogger {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl Logger for SimpleLogger {
    fn log(&self, level: LogLevel, message: &str) {
        println!("[{}] {:?}: {}", self.name, level, message);
    }
}

/// 创建简单日志器的便捷函数
pub fn create_logger(name: &str, _component: Component, _level: LogLevel) -> Arc<dyn Logger> {
    Arc::new(SimpleLogger::new(name))
}

/// 创建空日志器的便捷函数
pub fn create_noop_logger() -> Arc<dyn Logger> {
    Arc::new(SimpleLogger::new("noop"))
}

/// 语音选项（临时）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceOptions {
    pub voice: String,
    pub speed: f32,
    pub pitch: f32,
}

/// 听取选项（临时）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenOptions {
    pub language: String,
    pub timeout: u64,
}

/// 语音提供者 trait（临时）
#[async_trait]
pub trait VoiceProvider: Send + Sync {
    async fn speak(
        &self,
        text: &str,
        options: &VoiceOptions,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>;
    async fn listen(
        &self,
        audio: &[u8],
        options: &ListenOptions,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

/// 工具指标（临时）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetrics {
    pub tool_name: String,
    pub execution_time_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
    pub error: Option<String>,
    pub input_size_bytes: usize,
    pub output_size_bytes: usize,
    pub timestamp: u64,
}

/// 内存指标（临时）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub operation: String,
    pub execution_time_ms: u64,
    pub success: bool,
    pub operation_type: String,
    pub key: String,
    pub data_size_bytes: usize,
    pub timestamp: std::time::SystemTime,
    pub total_entries: usize,
    pub memory_usage: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

// 新增缺失的类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}

#[derive(Debug, Clone)]
pub struct TraceStep {
    pub name: String,
    pub step_type: TraceStepType,
    pub start_time: std::time::Instant,
    pub end_time: Option<std::time::Instant>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub success: bool,
    pub duration_ms: u64,
    pub error: Option<String>,
}

impl TraceStep {
    pub fn new(name: String, step_type: TraceStepType) -> Self {
        Self {
            name,
            step_type,
            start_time: std::time::Instant::now(),
            end_time: None,
            metadata: std::collections::HashMap::new(),
            input: None,
            output: None,
            success: false,
            duration_ms: 0,
            error: None,
        }
    }

    pub fn finish(&mut self) {
        self.end_time = Some(std::time::Instant::now());
    }
}

#[derive(Debug, Clone)]
pub enum TraceStepType {
    LlmCall,
    ToolCall,
    DataProcessing,
    MemoryAccess,
}

#[derive(Debug, Clone)]
pub struct AgentMetrics {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub avg_response_time: f64,
    pub token_usage: TelemetryTokenUsage,
    pub execution_time_ms: u64,
    pub tool_calls_count: usize,
}

impl AgentMetrics {
    pub fn new(
        total_calls: u64,
        successful_calls: u64,
        failed_calls: u64,
        avg_response_time: f64,
        token_usage: TelemetryTokenUsage,
        execution_time_ms: u64,
    ) -> Self {
        Self {
            total_calls,
            successful_calls,
            failed_calls,
            avg_response_time,
            token_usage,
            execution_time_ms,
            tool_calls_count: 0,
        }
    }

    pub fn record_error(&mut self) {
        self.failed_calls += 1;
    }

    pub fn end_timing(&mut self, _duration: std::time::Duration) {
        // 临时实现
    }

    pub fn set_token_usage(&mut self, token_usage: TelemetryTokenUsage) {
        self.token_usage = token_usage;
    }

    pub fn set_success(&mut self, _success: bool) {
        // 临时实现
    }

    pub fn add_custom_metric(&mut self, _key: String, _value: MetricValue) {
        // 临时实现
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetryTokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub session_id: String,
    pub user_id: Option<String>,
    pub request_id: Option<String>,
    pub environment: String,
    pub version: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
    pub data: serde_json::Value,
    pub name: String,
}

// 云适配器相关类型（临时）
#[async_trait]
pub trait CloudAdapter: Send + Sync {
    async fn deploy(
        &self,
        config: &DeploymentConfig,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn status(
        &self,
        deployment_id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn logs(
        &self,
        deployment_id: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub name: String,
    pub image: String,
    pub replicas: u32,
    pub resources: ResourceConfig,
    pub network: NetworkConfig,
    pub version: String,
    pub environment: std::collections::HashMap<String, String>,
    pub storage: Option<String>,
    pub autoscaling: Option<bool>,
    pub health_check: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub cpu: f64,
    pub memory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub ports: Vec<PortMapping>,
    pub public: bool,
    pub domain: Option<String>,
    pub ssl: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub container_port: u16,
    pub host_port: Option<u16>,
    pub protocol: String,
}

// 临时的实现
pub struct InMemoryMetricsCollector {
    metrics: Arc<std::sync::Mutex<HashMap<String, MetricValue>>>,
}

impl Default for InMemoryMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryMetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    pub fn get_metrics(&self) -> HashMap<String, MetricValue> {
        self.metrics.lock().map(|m| m.clone()).unwrap_or_default()
    }
}

#[async_trait]
impl MetricsCollector for InMemoryMetricsCollector {
    async fn record_metric(&self, name: &str, value: f64) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.insert(name.to_string(), MetricValue::Float(value));
        }
    }

    async fn record_memory_operation(
        &self,
        _metrics: MemoryMetrics,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    async fn record_tool_execution(
        &self,
        _metrics: ToolMetrics,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    async fn record_agent_execution(
        &self,
        _metrics: AgentMetrics,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

pub struct InMemoryTraceCollector {
    traces: Arc<std::sync::Mutex<Vec<TraceStep>>>,
}

impl Default for InMemoryTraceCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryTraceCollector {
    pub fn new() -> Self {
        Self {
            traces: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn get_traces(&self) -> Vec<TraceStep> {
        self.traces.lock().map(|t| t.clone()).unwrap_or_default()
    }
}

#[async_trait]
impl TraceCollector for InMemoryTraceCollector {
    async fn start_trace(&self, _name: &str) -> String {
        let trace_id = format!(
            "trace_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        trace_id
    }

    async fn add_trace_step(
        &self,
        _trace_id: &str,
        step: TraceStep,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Ok(mut traces) = self.traces.lock() {
            traces.push(step);
        }
        Ok(())
    }

    async fn end_trace(&self, _trace_id: &str) {
        // 临时实现，什么都不做
    }
}

// 云适配器实现（临时）
pub struct AwsAdapter;
pub struct AzureAdapter;
pub struct GcpAdapter;

impl AwsAdapter {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self)
    }
}

impl AzureAdapter {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self)
    }
}

impl GcpAdapter {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self)
    }
}

#[async_trait]
impl CloudAdapter for AwsAdapter {
    async fn deploy(
        &self,
        _config: &DeploymentConfig,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("aws-deployment-id".to_string())
    }

    async fn status(
        &self,
        _deployment_id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("running".to_string())
    }

    async fn logs(
        &self,
        _deployment_id: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec!["log line 1".to_string()])
    }
}

#[async_trait]
impl CloudAdapter for AzureAdapter {
    async fn deploy(
        &self,
        _config: &DeploymentConfig,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("azure-deployment-id".to_string())
    }

    async fn status(
        &self,
        _deployment_id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("running".to_string())
    }

    async fn logs(
        &self,
        _deployment_id: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec!["log line 1".to_string()])
    }
}

#[async_trait]
impl CloudAdapter for GcpAdapter {
    async fn deploy(
        &self,
        _config: &DeploymentConfig,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("gcp-deployment-id".to_string())
    }

    async fn status(
        &self,
        _deployment_id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("running".to_string())
    }

    async fn logs(
        &self,
        _deployment_id: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec!["log line 1".to_string()])
    }
}
