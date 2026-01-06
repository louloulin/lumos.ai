//! # 评估框架 (Evaluation Framework)
//!
//! LumosAI 的 Agent 性能、准确性和成本评估系统。
//!
//! ## 核心功能
//!
//! - **准确性评估** - 答案质量、相关性、事实一致性
//! - **性能评估** - 延迟、吞吐量、资源使用
//! - **成本评估** - Token 消耗、API 调用成本
//! - **自定义评估** - 用户定义的评估指标
//!
//! ## 快速开始
//!
//! ```rust,no_run
//! use lumosai_core::evaluation::{EvaluationFramework, AccuracyEvaluator};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let evaluator = EvaluationFramework::new()
//!     .with_evaluator(Arc::new(AccuracyEvaluator::new()))
//!     .with_evaluator(Arc::new(LatencyEvaluator::new()));
//!
//! let report = evaluator.evaluate(&agent, &test_dataset).await?;
//!
//! println!("Accuracy: {:.2}%", report.accuracy);
//! println!("P95 Latency: {:?}", report.p95_latency);
//! # Ok(())
//! # }
//! ```

pub mod accuracy;
pub mod latency;
pub mod cost;
pub mod custom;
pub mod dataset;
pub mod runner;
pub mod report;
pub mod ab_test;

// ✅ Phase 1: Week 3-4 导出核心类型
pub use accuracy::{AccuracyEvaluator, AccuracyMetric};
pub use latency::{LatencyEvaluator, LatencyMetric};
pub use cost::{CostEvaluator, CostMetric};
pub use custom::{CustomEvaluator, CustomMetricFn};
pub use dataset::{TestDataset, TestCase, TestCaseResult};
pub use runner::{EvaluationRunner, EvaluationConfig};
pub use report::{EvaluationReport, EvaluationSummary};
pub use ab_test::{ABTestRunner, ABTestResult, StatisticalTest};

use std::sync::Arc;
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// 评估器 trait
pub trait Evaluator: Send + Sync {
    /// 评估单个测试用例
    fn evaluate_case(&self, case: &TestCase, result: &TestCaseResult) -> Result<MetricValue, EvaluationError>;

    /// 获取评估器名称
    fn name(&self) -> &str;

    /// 获取评估器描述
    fn description(&self) -> &str;
}

/// 评估错误
#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("评估失败: {0}")]
    EvaluationFailed(String),

    #[error("数据集错误: {0}")]
    DatasetError(String),

    #[error("指标计算错误: {0}")]
    MetricError(String),

    #[error("Agent 执行错误: {0}")]
    AgentError(String),
}

/// 指标值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricValue {
    /// 浮点数值 (如准确率、延迟)
    Float(f64),
    /// 整数值 (如 Token 数量)
    Integer(i64),
    /// 字符串值 (如分类结果)
    String(String),
    /// 布尔值 (如是否通过)
    Boolean(bool),
    /// 百分比值 (0-100)
    Percentage(f64),
    /// 持续时间 (毫秒)
    Duration(u64),
}

impl MetricValue {
    /// 转换为浮点数
    pub fn as_float(&self) -> Option<f64> {
        match self {
            MetricValue::Float(v) => Some(*v),
            MetricValue::Percentage(v) => Some(*v),
            _ => None,
        }
    }

    /// 转换为整数
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            MetricValue::Integer(v) => Some(*v),
            MetricValue::Duration(v) => Some(*v as i64),
            _ => None,
        }
    }

    /// 转换为布尔值
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            MetricValue::Boolean(v) => Some(*v),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_value_conversions() {
        let float_val = MetricValue::Float(0.95);
        assert_eq!(float_val.as_float(), Some(0.95));
        assert_eq!(float_val.as_integer(), None);

        let int_val = MetricValue::Integer(1000);
        assert_eq!(int_val.as_integer(), Some(1000));
        assert_eq!(int_val.as_float(), None);

        let bool_val = MetricValue::Boolean(true);
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(bool_val.as_float(), None);

        let percent_val = MetricValue::Percentage(95.5);
        assert_eq!(percent_val.as_float(), Some(95.5));
    }
}
