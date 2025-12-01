//! 指标收集器模块
//!
//! 提供指标收集和聚合功能

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::compat::AgentMetrics;

/// 指标收集器 trait
#[async_trait]
pub trait MetricsCollector: Send + Sync {
    /// 记录代理执行指标
    async fn record_agent_execution(
        &self,
        metrics: AgentMetrics,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 记录工具执行指标
    async fn record_tool_execution(
        &self,
        metrics: ToolMetrics,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 记录内存操作指标
    async fn record_memory_operation(
        &self,
        metrics: MemoryMetrics,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 获取代理性能数据
    async fn get_agent_performance(
        &self,
        agent_id: &str,
    ) -> std::result::Result<AgentPerformance, Box<dyn std::error::Error + Send + Sync>>;

    /// 获取指标摘要
    async fn get_metrics_summary(
        &self,
        agent_id: Option<&str>,
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> std::result::Result<MetricsSummary, Box<dyn std::error::Error + Send + Sync>>;
}

/// 工具指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetrics {
    /// 工具名称
    pub tool_name: String,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 成功标志
    pub success: bool,
    /// 输入大小
    pub input_size: Option<usize>,
    /// 输出大小
    pub output_size: Option<usize>,
    /// 时间戳
    pub timestamp: u64,
}

/// 内存指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// 操作类型
    pub operation_type: String,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 成功标志
    pub success: bool,
    /// 内存使用量（字节）
    pub memory_usage_bytes: Option<usize>,
    /// 时间戳
    pub timestamp: u64,
}

/// 代理性能数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformance {
    /// 代理名称
    pub agent_name: String,
    /// 24小时内执行次数
    pub executions_last_24h: u64,
    /// 24小时成功率
    pub success_rate_24h: f64,
    /// 24小时平均响应时间（毫秒）
    pub avg_response_time_24h: f64,
    /// 错误率趋势
    pub error_rate_trend: Vec<(u64, f64)>,
    /// 性能趋势
    pub performance_trend: Vec<(u64, f64)>,
    /// 热门工具
    pub top_tools: Vec<(String, u64)>,
    /// 资源使用情况
    pub resource_usage: ResourceUsage,
}

/// 资源使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// 平均内存使用（MB）
    pub avg_memory_mb: f64,
    /// 峰值内存使用（MB）
    pub peak_memory_mb: f64,
    /// CPU使用率（百分比）
    pub cpu_usage_percent: f64,
}

/// 指标摘要
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
    /// 总Token使用量
    pub total_tokens_used: u64,
    /// 平均每次执行Token数
    pub avg_tokens_per_execution: f64,
    /// 最小执行时间（毫秒）
    pub min_execution_time_ms: u64,
    /// 最大执行时间（毫秒）
    pub max_execution_time_ms: u64,
    /// 工具调用统计
    pub tool_call_stats: HashMap<String, u64>,
    /// 时间范围
    pub time_range: crate::telemetry::analyzer::TimeRange,
}

