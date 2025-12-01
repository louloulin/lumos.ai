//! 遥测和监控模块
//!
//! 提供性能监控、指标收集和分布式追踪功能

pub mod analyzer;
pub mod collector;
pub mod alert;
pub mod monitor;
pub mod otel;

// 使用更具体的导出以避免名称冲突
pub use analyzer::{
    PerformanceAnalyzer, PerformanceAnalysis, PerformanceAnomaly, PerformanceBottleneck,
    OptimizationRecommendation, PerformanceTrend, TimeRange,
    ImplementationDifficulty, Priority,
};
// PerformancePrediction 在两个模块中都有定义，使用别名区分
pub use analyzer::PerformancePrediction as AnalyzerPerformancePrediction;
pub use collector::{
    MetricsCollector, ToolMetrics, MemoryMetrics, AgentPerformance, ResourceUsage, MetricsSummary,
};
pub use alert::{
    SmartAlertEngine, AlertEngineConfig, AlertRule, AlertSeverity, AlertCondition, Alert,
    AlertStatistics, EscalationConfig, AutomationExecutor, AutomationAction, AutomationActionType,
    DefaultAutomationExecutor, AutomationConfig,
};
pub use monitor::{
    EnterprisePerformanceMonitor, PerformanceMonitorConfig, PerformanceThresholds,
    PredictionConfig, AutoOptimizationConfig, OptimizationStrategy, CurrentPerformanceMetrics,
    ResponseTimeMetrics, ThroughputMetrics, ResourceUsageMetrics, ErrorMetrics,
    OptimizationSuggestion, PerformanceSummary, PerformancePrediction, MonitoringStatistics,
};
pub use otel::{
    HttpOtlpExporter, OtelSpan, OtelMetric, DataPoint, AttributeValue, DataPointValue, SpanKind,
    SpanStatus, SpanEvent,
};

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 遥测数据接收器 trait
#[async_trait]
pub trait TelemetrySink: Send + Sync + std::fmt::Debug {
    /// 记录指标
    async fn record_metric(&self, name: &str, value: f64, tags: Option<HashMap<String, String>>);

    /// 开始追踪
    async fn start_trace(&self, name: &str) -> String;

    /// 结束追踪
    async fn end_trace(&self, trace_id: &str, duration: Duration);

    /// 记录事件
    async fn record_event(&self, name: &str, data: Value);
}

/// 指标类型
#[derive(Debug, Clone)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Timer,
}

/// 指标数据
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub metric_type: MetricType,
    pub tags: HashMap<String, String>,
    pub timestamp: Instant,
}

/// 追踪信息
#[derive(Debug, Clone)]
pub struct Trace {
    pub id: String,
    pub name: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub duration: Option<Duration>,
    pub tags: HashMap<String, String>,
}

/// 内存遥测接收器（用于开发和测试）
#[derive(Debug, Default)]
pub struct InMemoryTelemetrySink {
    metrics: std::sync::Mutex<Vec<Metric>>,
    traces: std::sync::Mutex<Vec<Trace>>,
    events: std::sync::Mutex<Vec<(String, Value)>>,
}

impl InMemoryTelemetrySink {
    /// 创建新的内存遥测接收器
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取所有指标
    pub fn get_metrics(&self) -> Vec<Metric> {
        self.metrics.lock().unwrap().clone()
    }

    /// 获取所有追踪
    pub fn get_traces(&self) -> Vec<Trace> {
        self.traces.lock().unwrap().clone()
    }

    /// 获取所有事件
    pub fn get_events(&self) -> Vec<(String, Value)> {
        self.events.lock().unwrap().clone()
    }

    /// 清空所有数据
    pub fn clear(&self) {
        self.metrics.lock().unwrap().clear();
        self.traces.lock().unwrap().clear();
        self.events.lock().unwrap().clear();
    }
}

#[async_trait]
impl TelemetrySink for InMemoryTelemetrySink {
    async fn record_metric(&self, name: &str, value: f64, tags: Option<HashMap<String, String>>) {
        let metric = Metric {
            name: name.to_string(),
            value,
            metric_type: MetricType::Gauge,
            tags: tags.unwrap_or_default(),
            timestamp: Instant::now(),
        };

        self.metrics.lock().unwrap().push(metric);
    }

    async fn start_trace(&self, name: &str) -> String {
        let trace_id = uuid::Uuid::new_v4().to_string();
        let trace = Trace {
            id: trace_id.clone(),
            name: name.to_string(),
            start_time: Instant::now(),
            end_time: None,
            duration: None,
            tags: HashMap::new(),
        };

        self.traces.lock().unwrap().push(trace);
        trace_id
    }

    async fn end_trace(&self, trace_id: &str, duration: Duration) {
        let mut traces = self.traces.lock().unwrap();
        if let Some(trace) = traces.iter_mut().find(|t| t.id == trace_id) {
            trace.end_time = Some(Instant::now());
            trace.duration = Some(duration);
        }
    }

    async fn record_event(&self, name: &str, data: Value) {
        self.events.lock().unwrap().push((name.to_string(), data));
    }
}

/// 空遥测接收器（用于禁用遥测）
#[derive(Debug, Default)]
pub struct NoopTelemetrySink;

#[async_trait]
impl TelemetrySink for NoopTelemetrySink {
    async fn record_metric(
        &self,
        _name: &str,
        _value: f64,
        _tags: Option<HashMap<String, String>>,
    ) {
        // 不做任何操作
    }

    async fn start_trace(&self, _name: &str) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    async fn end_trace(&self, _trace_id: &str, _duration: Duration) {
        // 不做任何操作
    }

    async fn record_event(&self, _name: &str, _data: Value) {
        // 不做任何操作
    }
}

/// 创建默认的遥测接收器
pub fn default_telemetry_sink() -> Arc<dyn TelemetrySink> {
    Arc::new(InMemoryTelemetrySink::new())
}

/// 遥测管理器
#[derive(Debug)]
pub struct TelemetryManager {
    sink: Arc<dyn TelemetrySink>,
}

impl TelemetryManager {
    /// 创建新的遥测管理器
    pub fn new(sink: Arc<dyn TelemetrySink>) -> Self {
        Self { sink }
    }

    /// 记录指标
    pub async fn record_metric(
        &self,
        name: &str,
        value: f64,
        tags: Option<HashMap<String, String>>,
    ) {
        self.sink.record_metric(name, value, tags).await;
    }

    /// 开始追踪
    pub async fn start_trace(&self, name: &str) -> String {
        self.sink.start_trace(name).await
    }

    /// 结束追踪
    pub async fn end_trace(&self, trace_id: &str, duration: Duration) {
        self.sink.end_trace(trace_id, duration).await;
    }

    /// 记录事件
    pub async fn record_event(&self, name: &str, data: Value) {
        self.sink.record_event(name, data).await;
    }
}

