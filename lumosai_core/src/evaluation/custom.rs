//! # 自定义评估器
//!
//! 允许用户定义自定义的评估逻辑。

use crate::evaluation::{Evaluator, EvaluationError, MetricValue, TestCase, TestCaseResult};
use std::sync::Arc;

/// 自定义评估函数
pub type CustomMetricFn = Arc<dyn Fn(&TestCase, &TestCaseResult) -> Result<MetricValue, EvaluationError> + Send + Sync>;

/// 自定义评估器
pub struct CustomEvaluator {
    name: String,
    description: String,
    eval_fn: CustomMetricFn,
}

impl CustomEvaluator {
    /// 创建新的自定义评估器
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        eval_fn: CustomMetricFn,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            eval_fn,
        }
    }
}

impl Evaluator for CustomEvaluator {
    fn evaluate_case(&self, case: &TestCase, result: &TestCaseResult) -> Result<MetricValue, EvaluationError> {
        (self.eval_fn)(case, result)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// 自定义指标构建器
pub struct CustomMetricBuilder {
    name: String,
    description: String,
    eval_fn: Option<CustomMetricFn>,
}

impl CustomMetricBuilder {
    /// 创建新的构建器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            eval_fn: None,
        }
    }

    /// 设置描述
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// 设置评估函数
    pub fn evaluate_fn(mut self, fn_impl: CustomMetricFn) -> Self {
        self.eval_fn = Some(fn_impl);
        self
    }

    /// 构建评估器
    pub fn build(self) -> Result<CustomEvaluator, String> {
        let eval_fn = self.eval_fn.ok_or("Evaluation function not set")?;
        Ok(CustomEvaluator::new(self.name, self.description, eval_fn))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_custom_evaluator() {
        let evaluator = CustomEvaluator::new(
            "LengthCheck",
            "Check output length",
            Arc::new(|_case, result| {
                let len = result.output.len() as i64;
                Ok(MetricValue::Integer(len))
            }),
        );

        let case = TestCase {
            id: "test".to_string(),
            input: "test".to_string(),
            expected_output: "result".to_string(),
            metadata: HashMap::new(),
        };

        let result = TestCaseResult {
            output: "Hello, World!".to_string(),
            latency_ms: 100,
            tokens_used: 50,
            success: true,
            error: None,
            metadata: HashMap::new(),
        };

        let metric = evaluator.evaluate_case(&case, &result).unwrap();
        assert_eq!(metric, MetricValue::Integer(13));
    }

    #[test]
    fn test_custom_evaluator_builder() {
        let evaluator = CustomMetricBuilder::new("OutputLength")
            .description("Measures output length")
            .evaluate_fn(Arc::new(|_case, result| {
                Ok(MetricValue::Integer(result.output.len() as i64))
            }))
            .build()
            .unwrap();

        assert_eq!(evaluator.name(), "OutputLength");
        assert_eq!(evaluator.description(), "Measures output length");
    }
}
