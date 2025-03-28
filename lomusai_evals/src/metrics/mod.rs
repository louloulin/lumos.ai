//! 评估指标模块
//!
//! 该模块提供了各种指标来评估AI模型输出的质量。

use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

pub mod accuracy;
pub mod relevance;
pub mod coherence;

/// 指标计算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricResult {
    /// 评估得分，通常在0到1之间
    pub score: f64,
    
    /// 有关得分的附加信息
    pub info: HashMap<String, serde_json::Value>,
}

impl Default for MetricResult {
    fn default() -> Self {
        Self {
            score: 0.0,
            info: HashMap::new(),
        }
    }
}

/// 评估指标接口
#[async_trait]
pub trait Metric: Send + Sync {
    /// 获取指标名称
    fn name(&self) -> &str;
    
    /// 获取指标描述
    fn description(&self) -> &str;
    
    /// 评估输入和输出，计算指标值
    async fn measure(&self, input: &str, output: &str) -> Result<MetricResult>;
}

// 重导出主要的指标实现，方便使用
pub use accuracy::AccuracyMetric;
pub use relevance::RelevanceMetric;
pub use coherence::CoherenceMetric; 