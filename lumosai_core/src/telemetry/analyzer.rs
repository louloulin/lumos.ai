//! 性能分析器模块
//!
//! 提供性能分析、异常检测和优化建议功能

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::compat::AgentMetrics;

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// 开始时间（毫秒时间戳）
    pub start: u64,
    /// 结束时间（毫秒时间戳）
    pub end: u64,
}

/// 性能分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// 总体性能评分（0-100）
    pub overall_score: f64,
    /// 性能瓶颈列表
    pub bottlenecks: Vec<PerformanceBottleneck>,
    /// 异常列表
    pub anomalies: Vec<PerformanceAnomaly>,
    /// 优化建议列表
    pub recommendations: Vec<OptimizationRecommendation>,
    /// 性能趋势
    pub trend: PerformanceTrend,
    /// 性能预测列表
    pub predictions: Vec<PerformancePrediction>,
    /// 时间范围
    pub time_range: TimeRange,
    /// 时间戳
    pub timestamp: u64,
}

/// 性能瓶颈
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    /// 瓶颈类型
    pub bottleneck_type: String,
    /// 严重程度
    pub severity: String,
    /// 描述
    pub description: String,
    /// 影响范围
    pub impact: String,
    /// 建议
    pub suggestion: String,
}

/// 性能异常
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnomaly {
    /// 异常类型
    pub anomaly_type: String,
    /// 严重程度
    pub severity: String,
    /// 描述
    pub description: String,
    /// 检测时间
    pub detected_at: u64,
    /// 指标值
    pub metric_value: f64,
    /// 预期值
    pub expected_value: f64,
}

/// 优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// 建议标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 预期改善（百分比）
    pub expected_improvement: f64,
    /// 实施难度
    pub implementation_difficulty: ImplementationDifficulty,
    /// 优先级
    pub priority: Priority,
    /// 相关指标
    pub related_metrics: Vec<String>,
}

/// 实施难度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationDifficulty {
    Low,
    Medium,
    High,
}

/// 优先级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// 性能趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTrend {
    /// 稳定趋势
    Stable { variance: f64 },
    /// 上升趋势
    Increasing { rate: f64 },
    /// 下降趋势
    Decreasing { rate: f64 },
    /// 波动趋势
    Volatile { variance: f64 },
}

/// 性能预测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePrediction {
    /// 预测指标名称
    pub metric_name: String,
    /// 预测值
    pub predicted_value: f64,
    /// 置信度（0-1）
    pub confidence: f64,
    /// 预测时间点
    pub prediction_time: u64,
    /// 置信区间下限
    pub confidence_lower: f64,
    /// 置信区间上限
    pub confidence_upper: f64,
}

/// 性能分析器 trait
#[async_trait]
pub trait PerformanceAnalyzer: Send + Sync {
    /// 分析性能指标
    async fn analyze(
        &self,
        metrics: &[AgentMetrics],
        time_range: TimeRange,
    ) -> std::result::Result<PerformanceAnalysis, Box<dyn std::error::Error + Send + Sync>>;

    /// 检测异常
    async fn detect_anomalies(
        &self,
        metrics: &[AgentMetrics],
    ) -> std::result::Result<Vec<PerformanceAnomaly>, Box<dyn std::error::Error + Send + Sync>>;

    /// 识别瓶颈
    async fn identify_bottlenecks(
        &self,
        metrics: &[AgentMetrics],
    ) -> std::result::Result<Vec<PerformanceBottleneck>, Box<dyn std::error::Error + Send + Sync>>;

    /// 生成优化建议
    async fn generate_recommendations(
        &self,
        analysis: &PerformanceAnalysis,
    ) -> std::result::Result<Vec<OptimizationRecommendation>, Box<dyn std::error::Error + Send + Sync>>;

    /// 预测趋势
    async fn predict_trends(
        &self,
        metrics: &[AgentMetrics],
    ) -> std::result::Result<Vec<PerformancePrediction>, Box<dyn std::error::Error + Send + Sync>>;
}

