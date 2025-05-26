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

#[cfg(test)]
pub mod tests;

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
    OtelConfig, OtelMetricsCollector, HttpOtlpExporter, OtelSpan, OtelMetric
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