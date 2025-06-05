//! OpenTelemetry集成模块
//! 
//! 提供与OpenTelemetry的集成功能，包括指标导出、追踪和配置管理。

use crate::telemetry::metrics::*;
use crate::telemetry::trace::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// OpenTelemetry集成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelConfig {
    /// 服务名称
    pub service_name: String,
    /// 服务版本
    pub service_version: Option<String>,
    /// OTLP导出器端点
    pub otlp_endpoint: Option<String>,
    /// 采样率 (0.0 - 1.0)
    pub sampling_rate: f64,
    /// 是否启用指标导出
    pub enable_metrics: bool,
    /// 是否启用追踪导出
    pub enable_traces: bool,
    /// 是否启用日志导出
    pub enable_logs: bool,
    /// 导出批次大小
    pub batch_size: usize,
    /// 导出超时（毫秒）
    pub export_timeout_ms: u64,
    /// 额外的资源属性
    pub resource_attributes: HashMap<String, String>,
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            service_name: "lumosai-agent".to_string(),
            service_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            otlp_endpoint: None,
            sampling_rate: 1.0,
            enable_metrics: true,
            enable_traces: true,
            enable_logs: true,
            batch_size: 512,
            export_timeout_ms: 10000,
            resource_attributes: HashMap::new(),
        }
    }
}

/// OpenTelemetry span类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelSpan {
    /// Span ID
    pub span_id: String,
    /// Trace ID
    pub trace_id: String,
    /// 父Span ID
    pub parent_span_id: Option<String>,
    /// Span名称
    pub name: String,
    /// 开始时间（纳秒）
    pub start_time_ns: u64,
    /// 结束时间（纳秒）
    pub end_time_ns: u64,
    /// Span状态
    pub status: SpanStatus,
    /// 属性
    pub attributes: HashMap<String, AttributeValue>,
    /// 事件
    pub events: Vec<SpanEvent>,
    /// Span类型
    pub kind: SpanKind,
}

/// Span状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanStatus {
    /// 未设置
    Unset,
    /// 成功
    Ok,
    /// 错误
    Error { message: String },
}

/// Span类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpanKind {
    /// 内部span
    Internal,
    /// 服务器span
    Server,
    /// 客户端span
    Client,
    /// 生产者span
    Producer,
    /// 消费者span
    Consumer,
}

/// 属性值类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    String(String),
    Bool(bool),
    Int(i64),
    Double(f64),
    StringArray(Vec<String>),
    BoolArray(Vec<bool>),
    IntArray(Vec<i64>),
    DoubleArray(Vec<f64>),
}

/// Span事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// 事件名称
    pub name: String,
    /// 时间戳（纳秒）
    pub timestamp_ns: u64,
    /// 事件属性
    pub attributes: HashMap<String, AttributeValue>,
}

/// OpenTelemetry指标类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelMetric {
    /// 指标名称
    pub name: String,
    /// 指标描述
    pub description: Option<String>,
    /// 指标单位
    pub unit: Option<String>,
    /// 指标类型
    pub metric_type: MetricType,
    /// 数据点
    pub data_points: Vec<DataPoint>,
}

/// 指标类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    /// 计数器
    Counter,
    /// 仪表盘
    Gauge,
    /// 直方图
    Histogram,
    /// 指数直方图
    ExponentialHistogram,
    /// 摘要
    Summary,
}

/// 数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// 时间戳（纳秒）
    pub timestamp_ns: u64,
    /// 属性
    pub attributes: HashMap<String, AttributeValue>,
    /// 值
    pub value: DataPointValue,
}

/// 数据点值
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataPointValue {
    /// 整数值
    Int(i64),
    /// 浮点值
    Double(f64),
    /// 直方图值
    Histogram {
        count: u64,
        sum: f64,
        buckets: Vec<HistogramBucket>,
    },
}

/// 直方图桶
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    /// 上边界
    pub upper_bound: f64,
    /// 计数
    pub count: u64,
}

/// OpenTelemetry导出器trait
#[async_trait]
pub trait OtelExporter: Send + Sync {
    /// 导出spans
    async fn export_spans(&self, spans: Vec<OtelSpan>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 导出指标
    async fn export_metrics(&self, metrics: Vec<OtelMetric>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 强制刷新
    async fn force_flush(&self, timeout: Duration) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 关闭导出器
    async fn shutdown(&self, timeout: Duration) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// HTTP OTLP导出器
#[derive(Debug)]
pub struct HttpOtlpExporter {
    /// 端点URL
    endpoint: String,
    /// HTTP客户端
    client: reqwest::Client,
    /// 请求头
    headers: HashMap<String, String>,
    /// 超时设置
    timeout: Duration,
}

impl HttpOtlpExporter {
    /// 创建新的HTTP OTLP导出器
    pub fn new(endpoint: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/x-protobuf".to_string());
        
        Self {
            endpoint,
            client: reqwest::Client::new(),
            headers,
            timeout: Duration::from_secs(10),
        }
    }
    
    /// 设置认证头
    pub fn with_auth_header(mut self, name: String, value: String) -> Self {
        self.headers.insert(name, value);
        self
    }
    
    /// 设置超时
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[async_trait]
impl OtelExporter for HttpOtlpExporter {
    async fn export_spans(&self, spans: Vec<OtelSpan>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 实现span导出逻辑
        println!("Exporting {} spans to {}", spans.len(), self.endpoint);
        Ok(())
    }
    
    async fn export_metrics(&self, metrics: Vec<OtelMetric>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 实现指标导出逻辑
        println!("Exporting {} metrics to {}", metrics.len(), self.endpoint);
        Ok(())
    }
    
    async fn force_flush(&self, _timeout: Duration) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 实现强制刷新逻辑
        Ok(())
    }
    
    async fn shutdown(&self, _timeout: Duration) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 实现关闭逻辑
        Ok(())
    }
}

/// OpenTelemetry指标收集器
pub struct OtelMetricsCollector {
    config: OtelConfig,
    inner: Box<dyn MetricsCollector>,
    exporter: Box<dyn OtelExporter>,
}

impl OtelMetricsCollector {
    pub fn new(
        inner: Box<dyn MetricsCollector>,
        exporter: Box<dyn OtelExporter>,
        config: OtelConfig,
    ) -> Self {
        Self {
            config,
            inner,
            exporter,
        }
    }
}

#[async_trait]
impl MetricsCollector for OtelMetricsCollector {
    async fn record_agent_execution(&self, metrics: AgentMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 委托给内部收集器
        self.inner.record_agent_execution(metrics).await
    }
    
    async fn record_tool_execution(&self, metrics: ToolMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.inner.record_tool_execution(metrics).await
    }
    
    async fn record_memory_operation(&self, metrics: MemoryMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.inner.record_memory_operation(metrics).await
    }
    
    async fn get_metrics_summary(&self, agent_name: Option<&str>, from_time: Option<u64>, to_time: Option<u64>) -> Result<MetricsSummary, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.get_metrics_summary(agent_name, from_time, to_time).await
    }
    
    async fn get_agent_performance(&self, agent_name: &str) -> Result<AgentPerformance, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.get_agent_performance(agent_name).await
    }
}
