//! 容量规划模块
//! 
//! 提供企业级容量规划和预测功能

use async_trait::async_trait;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{EnterpriseError, Result};

/// 容量规划器
pub struct CapacityPlanner {
    /// 容量指标
    capacity_metrics: Vec<CapacityMetrics>,
    
    /// 扩容建议
    scaling_recommendations: Vec<ScalingRecommendation>,
}

/// 容量指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityMetrics {
    /// 资源类型
    pub resource_type: String,
    
    /// 当前使用量
    pub current_usage: f64,
    
    /// 总容量
    pub total_capacity: f64,
    
    /// 使用率
    pub utilization_rate: f64,
    
    /// 测量时间
    pub measured_at: DateTime<Utc>,
}

/// 扩容建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingRecommendation {
    /// 资源类型
    pub resource_type: String,
    
    /// 建议操作
    pub action: ScalingAction,
    
    /// 建议值
    pub recommended_value: f64,
    
    /// 理由
    pub reason: String,
    
    /// 优先级
    pub priority: RecommendationPriority,
    
    /// 生成时间
    pub generated_at: DateTime<Utc>,
}

/// 扩容操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingAction {
    ScaleUp,
    ScaleDown,
    Maintain,
}

/// 建议优先级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl CapacityPlanner {
    /// 创建新的容量规划器
    pub fn new() -> Self {
        Self {
            capacity_metrics: Vec::new(),
            scaling_recommendations: Vec::new(),
        }
    }
    
    /// 添加容量指标
    pub async fn add_metrics(&mut self, metrics: CapacityMetrics) -> Result<()> {
        self.capacity_metrics.push(metrics);
        Ok(())
    }
    
    /// 生成扩容建议
    pub async fn generate_recommendations(&mut self) -> Result<Vec<ScalingRecommendation>> {
        self.scaling_recommendations.clear();
        
        for metrics in &self.capacity_metrics {
            if metrics.utilization_rate > 0.8 {
                let recommendation = ScalingRecommendation {
                    resource_type: metrics.resource_type.clone(),
                    action: ScalingAction::ScaleUp,
                    recommended_value: metrics.total_capacity * 1.5,
                    reason: "High utilization detected".to_string(),
                    priority: if metrics.utilization_rate > 0.9 {
                        RecommendationPriority::Critical
                    } else {
                        RecommendationPriority::High
                    },
                    generated_at: Utc::now(),
                };
                self.scaling_recommendations.push(recommendation);
            } else if metrics.utilization_rate < 0.3 {
                let recommendation = ScalingRecommendation {
                    resource_type: metrics.resource_type.clone(),
                    action: ScalingAction::ScaleDown,
                    recommended_value: metrics.total_capacity * 0.7,
                    reason: "Low utilization detected".to_string(),
                    priority: RecommendationPriority::Medium,
                    generated_at: Utc::now(),
                };
                self.scaling_recommendations.push(recommendation);
            }
        }
        
        Ok(self.scaling_recommendations.clone())
    }
}

impl Default for CapacityPlanner {
    fn default() -> Self {
        Self::new()
    }
}
