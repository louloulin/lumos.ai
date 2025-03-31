//! Lumosai Evaluation Framework
//!
//! 这个模块提供了一套用于评估AI模型输出的工具和指标。
//! 它包括LLM评估、基于规则的评估以及各种评估指标。

pub mod error;
pub mod types;
pub mod metrics;
pub mod evaluator;

// 重导出主要的类型和函数，使API更易用
pub use error::{Error, Result};
pub use types::{EvalOptions, EvalResult, TestInfo};
pub use metrics::{Metric, MetricResult};
pub use evaluator::Evaluator; 