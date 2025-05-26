use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use async_trait::async_trait;

/// 代理执行指标数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// 执行唯一标识符
    pub execution_id: String,
    /// 代理名称
    pub agent_name: String,
    /// 执行开始时间（毫秒时间戳）
    pub start_time: u64,
    /// 执行结束时间（毫秒时间戳）
    pub end_time: u64,
    /// 执行总时长（毫秒）
    pub execution_time_ms: u64,
    /// Token使用统计
    pub token_usage: TokenUsage,
    /// 工具调用次数
    pub tool_calls_count: usize,
    /// 内存操作次数
    pub memory_operations: usize,
    /// 错误次数
    pub error_count: usize,
    /// 是否执行成功
    pub success: bool,
    /// 额外的自定义指标
    pub custom_metrics: HashMap<String, MetricValue>,
    /// 执行上下文信息
    pub context: ExecutionContext,
}

/// Token使用统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// 输入Token数量
    pub prompt_tokens: u32,
    /// 输出Token数量
    pub completion_tokens: u32,
    /// 总Token数量
    pub total_tokens: u32,
}

/// 指标值类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

/// 执行上下文信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// 会话ID
    pub session_id: Option<String>,
    /// 用户ID
    pub user_id: Option<String>,
    /// 请求ID
    pub request_id: Option<String>,
    /// 环境信息
    pub environment: String,
    /// 版本信息
    pub version: Option<String>,
}

/// 工具执行指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetrics {
    /// 工具名称
    pub tool_name: String,
    /// 执行时长（毫秒）
    pub execution_time_ms: u64,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果有）
    pub error: Option<String>,
    /// 输入参数大小（字节）
    pub input_size_bytes: usize,
    /// 输出结果大小（字节）
    pub output_size_bytes: usize,
    /// 时间戳
    pub timestamp: u64,
}

/// 内存操作指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// 操作类型（get, set, delete, clear等）
    pub operation_type: String,
    /// 执行时长（毫秒）
    pub execution_time_ms: u64,
    /// 是否成功
    pub success: bool,
    /// 内存键名
    pub key: Option<String>,
    /// 数据大小（字节）
    pub data_size_bytes: Option<usize>,
    /// 时间戳
    pub timestamp: u64,
}

/// 指标收集器trait
#[async_trait]
pub trait MetricsCollector: Send + Sync {
    /// 记录代理执行指标
    async fn record_agent_execution(&self, metrics: AgentMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 记录工具执行指标
    async fn record_tool_execution(&self, metrics: ToolMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 记录内存操作指标
    async fn record_memory_operation(&self, metrics: MemoryMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取指标统计
    async fn get_metrics_summary(&self, agent_name: Option<&str>, from_time: Option<u64>, to_time: Option<u64>) -> Result<MetricsSummary, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取代理性能统计
    async fn get_agent_performance(&self, agent_name: &str) -> Result<AgentPerformance, Box<dyn std::error::Error + Send + Sync>>;
}

/// 指标统计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// 总执行次数
    pub total_executions: u64,
    /// 成功执行次数
    pub successful_executions: u64,
    /// 失败执行次数
    pub failed_executions: u64,
    /// 平均执行时间（毫秒）
    pub avg_execution_time_ms: f64,
    /// 最小执行时间（毫秒）
    pub min_execution_time_ms: u64,
    /// 最大执行时间（毫秒）
    pub max_execution_time_ms: u64,
    /// 总Token使用量
    pub total_tokens_used: u64,
    /// 平均Token使用量
    pub avg_tokens_per_execution: f64,
    /// 工具调用统计
    pub tool_call_stats: HashMap<String, u64>,
    /// 时间范围
    pub time_range: TimeRange,
}

/// 代理性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformance {
    /// 代理名称
    pub agent_name: String,
    /// 最近24小时执行次数
    pub executions_last_24h: u64,
    /// 最近24小时成功率
    pub success_rate_24h: f64,
    /// 最近24小时平均响应时间
    pub avg_response_time_24h: f64,
    /// 错误率趋势
    pub error_rate_trend: Vec<(u64, f64)>, // (timestamp, error_rate)
    /// 性能趋势
    pub performance_trend: Vec<(u64, f64)>, // (timestamp, avg_response_time)
    /// 最常用工具
    pub top_tools: Vec<(String, u64)>,
    /// 资源使用统计
    pub resource_usage: ResourceUsage,
}

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: u64,
    pub end: u64,
}

/// 资源使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// 平均内存使用（MB）
    pub avg_memory_mb: f64,
    /// 峰值内存使用（MB）
    pub peak_memory_mb: f64,
    /// CPU使用率
    pub cpu_usage_percent: f64,
}

impl AgentMetrics {
    /// 创建新的代理指标实例
    pub fn new(agent_name: String, context: ExecutionContext) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
            
        Self {
            execution_id: Uuid::new_v4().to_string(),
            agent_name,
            start_time: now,
            end_time: now,
            execution_time_ms: 0,
            token_usage: TokenUsage::default(),
            tool_calls_count: 0,
            memory_operations: 0,
            error_count: 0,
            success: false,
            custom_metrics: HashMap::new(),
            context,
        }
    }
    
    /// 开始计时
    pub fn start_timing(&mut self) {
        self.start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }
    
    /// 结束计时
    pub fn end_timing(&mut self) {
        self.end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.execution_time_ms = self.end_time - self.start_time;
    }
    
    /// 记录工具调用
    pub fn record_tool_call(&mut self) {
        self.tool_calls_count += 1;
    }
    
    /// 记录内存操作
    pub fn record_memory_operation(&mut self) {
        self.memory_operations += 1;
    }
    
    /// 记录错误
    pub fn record_error(&mut self) {
        self.error_count += 1;
        self.success = false;
    }
    
    /// 设置成功状态
    pub fn set_success(&mut self, success: bool) {
        self.success = success;
    }
    
    /// 设置Token使用量
    pub fn set_token_usage(&mut self, token_usage: TokenUsage) {
        self.token_usage = token_usage;
    }
    
    /// 添加自定义指标
    pub fn add_custom_metric(&mut self, key: String, value: MetricValue) {
        self.custom_metrics.insert(key, value);
    }
}

impl ToolMetrics {
    /// 创建新的工具指标
    pub fn new(tool_name: String) -> Self {
        Self {
            tool_name,
            execution_time_ms: 0,
            success: false,
            error: None,
            input_size_bytes: 0,
            output_size_bytes: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
    
    /// 设置执行时长
    pub fn set_execution_time(&mut self, duration: Duration) {
        self.execution_time_ms = duration.as_millis() as u64;
    }
    
    /// 设置成功状态
    pub fn set_success(&mut self, success: bool) {
        self.success = success;
    }
    
    /// 设置错误信息
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.success = false;
    }
}

impl MemoryMetrics {
    /// 创建新的内存操作指标
    pub fn new(operation_type: String) -> Self {
        Self {
            operation_type,
            execution_time_ms: 0,
            success: false,
            key: None,
            data_size_bytes: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
    
    /// 设置执行时长
    pub fn set_execution_time(&mut self, duration: Duration) {
        self.execution_time_ms = duration.as_millis() as u64;
    }
    
    /// 设置成功状态
    pub fn set_success(&mut self, success: bool) {
        self.success = success;
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            session_id: None,
            user_id: None,
            request_id: None,
            environment: "development".to_string(),
            version: None,
        }
    }
}
