//! 评估执行器模块
//!
//! 该模块提供了用于执行评估的接口和实现。

use crate::error::Result;
use crate::types::{EvalOptions, EvalResult, TestInfo};
use crate::metrics::Metric;

use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

pub mod llm_eval;
pub mod rule_eval;

// 重导出主要的评估器实现
pub use llm_eval::LlmEvaluator;
pub use rule_eval::RuleEvaluator;

/// 评估器接口
#[async_trait::async_trait]
pub trait Evaluator: Send + Sync {
    /// 获取评估器名称
    fn name(&self) -> &str;
    
    /// 执行评估
    async fn evaluate(&self, input: &str, output: &str, options: &EvalOptions) -> Result<EvalResult>;
}

/// 基于指标的评估器
pub struct MetricBasedEvaluator {
    /// 评估器名称
    name: String,
    
    /// 用于评估的指标
    metric: Arc<dyn Metric>,
}

impl MetricBasedEvaluator {
    /// 创建一个新的基于指标的评估器
    pub fn new(name: impl Into<String>, metric: Arc<dyn Metric>) -> Self {
        Self {
            name: name.into(),
            metric,
        }
    }
}

#[async_trait::async_trait]
impl Evaluator for MetricBasedEvaluator {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn evaluate(&self, input: &str, output: &str, options: &EvalOptions) -> Result<EvalResult> {
        // 使用指标测量输入和输出
        let metric_result = self.metric.measure(input, output).await?;
        
        // 创建评估结果
        let global_run_id = options.global_run_id.clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
            
        let run_id = options.run_id.clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
            
        let result = EvalResult {
            id: Uuid::new_v4().to_string(),
            global_run_id,
            run_id,
            input: input.to_string(),
            output: output.to_string(),
            score: metric_result.score,
            score_details: metric_result.info,
            created_at: Utc::now(),
            evaluator_name: self.name.clone(),
            metric_name: self.metric.name().to_string(),
            target_name: options.target_name.clone(),
            test_info: options.test_info.clone(),
            instructions: options.instructions.clone(),
        };
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    /*
    use super::*;
    use crate::metrics::MetricResult;
    use std::collections::HashMap;
    use mockall::predicate::*;
    use mockall::predicate::always;
    use mockall::mock;
    use async_trait::async_trait;
    
    mock! {
        MockMetric {}
        
        #[async_trait]
        impl Metric for MockMetric {
            fn name(&self) -> &str;
            fn description(&self) -> &str;
            async fn measure(&self, input: &str, output: &str) -> Result<MetricResult>;
        }
    }
    
    #[test]
    fn test_metric_based_evaluator() {
        let mut mock_metric = MockMetric::new();
        
        // 设置mock的行为
        mock_metric.expect_name()
            .return_const("mock_metric");
            
        mock_metric.expect_measure()
            .with(always(), always())
            .returning(|_, _| {
                Ok(MetricResult {
                    score: 0.75,
                    info: HashMap::new(),
                })
            });
            
        // 创建评估器
        let evaluator = MetricBasedEvaluator::new(
            "test_evaluator",
            Arc::new(mock_metric)
        );
        
        // 执行评估
        let options = EvalOptions::default();
        let result = evaluator.evaluate("test input", "test output", &options);
        
        assert!(result.is_ok());
        let eval_result = result.unwrap();
        assert_eq!(eval_result.score, 0.75);
        assert_eq!(eval_result.evaluator_name, "test_evaluator");
        assert_eq!(eval_result.metric_name, "mock_metric");
    }
    */
} 