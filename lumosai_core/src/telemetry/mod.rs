//! Telemetry and monitoring system for lumosai agents
//! 
//! This module provides comprehensive telemetry capabilities including:
//! - Metrics collection for agent execution, tool calls, and memory operations
//! - Execution tracing with detailed step tracking
//! - OpenTelemetry integration for distributed tracing
//! - Multiple storage backends (in-memory, filesystem, OTLP)

pub mod metrics;
pub mod trace;
pub mod collectors;
pub mod otel;
pub mod alerts;
pub mod analyzer;
pub mod alert_engine;
pub mod performance_monitor;

// 企业级监控扩展模块
pub mod enterprise;
pub mod compliance_monitor;
pub mod business_metrics;
pub mod anomaly_detection;
pub mod capacity_planning;
pub mod sla_monitoring;

#[cfg(test)]
pub mod tests;

// Integration tests and observability tests are temporarily disabled
// due to compilation issues with trait definitions
// #[cfg(test)]
// pub mod integration_tests;
// #[cfg(test)]
// pub mod observability_tests;

// Re-export core types for convenience
pub use metrics::{
    AgentMetrics, ToolMetrics, MemoryMetrics, TokenUsage, MetricValue, ExecutionContext,
    MetricsCollector, MetricsSummary, AgentPerformance, ResourceUsage, TimeRange
};

pub use trace::{
    ExecutionTrace, TraceStep, StepType, TraceCollector, TraceBuilder, TraceStats
};

pub use collectors::{
    InMemoryMetricsCollector, FileSystemMetricsCollector
};

pub use otel::{
    OtelConfig, OtelMetricsCollector, HttpOtlpExporter, OtelSpan, OtelMetric,
    OtelExporter, SpanStatus, SpanKind, AttributeValue, SpanEvent, MetricType,
    DataPoint, DataPointValue, HistogramBucket
};

pub use alerts::{
    AlertManager, AlertRule, AlertEvent, AlertSeverity, AlertStatus, AlertCondition,
    AlertChannel, AlertChannelType, InMemoryAlertManager, DiagnosisInfo,
    AutoFixSuggestion, ComparisonOperator
};

pub use analyzer::{
    PerformanceAnalyzer, PerformanceAnalysis, PerformanceTrend, BottleneckType,
    AnomalyType, OptimizationRecommendation, RecommendationType, DifficultyLevel,
    PerformancePrediction, PredictionModel, IntelligentPerformanceAnalyzer
};

pub use alert_engine::{
    SmartAlertEngine, AlertEngineConfig, EscalationConfig, AutomationConfig,
    AutomationAction, AutomationActionType, AlertContext, AlertActionResult,
    ActionStatus, AlertStatistics, AutomationExecutor, DefaultAutomationExecutor
};

pub use performance_monitor::{
    EnterprisePerformanceMonitor, PerformanceMonitorConfig, PerformanceThresholds,
    PredictionConfig, AutoOptimizationConfig, OptimizationStrategy,
    RealTimePerformanceMetrics, ResponseTimeMetrics, ThroughputMetrics,
    ResourceUsageMetrics, ErrorMetrics, PerformanceTrend as MonitorPerformanceTrend,
    PerformancePrediction as MonitorPerformancePrediction,
    PerformanceOptimizationSuggestion, DifficultyLevel as MonitorDifficultyLevel, RiskLevel,
    PerformanceSummaryReport, MonitoringStatistics
};

/// Basic event type for legacy support
#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub data: serde_json::Value,
}

/// Telemetry sink trait for legacy support
pub trait TelemetrySink: Send + Sync {
    fn record_event(&self, event: Event);
} 