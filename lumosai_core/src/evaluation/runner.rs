//! # 评估运行器
//!
//! 执行评估测试并生成报告。

use crate::evaluation::{
    dataset::{TestDataset, TestCase, TestCaseResult},
    report::EvaluationReport,
    Evaluator, EvaluationError,
};
use std::sync::Arc;
use std::time::Instant;

/// 评估配置
#[derive(Debug, Clone)]
pub struct EvaluationConfig {
    /// 是否并行执行
    pub parallel: bool,
    /// 并发数
    pub concurrency: usize,
    /// 超时时间 (秒)
    pub timeout_seconds: u64,
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            parallel: true,
            concurrency: 10,
            timeout_seconds: 300,
        }
    }
}

/// 评估运行器
pub struct EvaluationRunner {
    evaluators: Vec<Arc<dyn Evaluator>>,
    config: EvaluationConfig,
}

impl EvaluationRunner {
    /// 创建新的评估运行器
    pub fn new() -> Self {
        Self {
            evaluators: Vec::new(),
            config: EvaluationConfig::default(),
        }
    }

    /// 添加评估器
    pub fn with_evaluator(mut self, evaluator: Arc<dyn Evaluator>) -> Self {
        self.evaluators.push(evaluator);
        self
    }

    /// 设置配置
    pub fn with_config(mut self, config: EvaluationConfig) -> Self {
        self.config = config;
        self
    }

    /// 运行评估
    ///
    /// 注意: 这是一个简化的实现,实际使用时需要集成 Agent 执行
    pub async fn evaluate(
        &self,
        dataset: &TestDataset,
        results: &[TestCaseResult],
    ) -> Result<EvaluationReport, EvaluationError> {
        if dataset.len() != results.len() {
            return Err(EvaluationError::DatasetError(format!(
                "Dataset size mismatch: {} cases vs {} results",
                dataset.len(),
                results.len()
            )));
        }

        let start_time = Instant::now();
        let mut case_metrics = Vec::new();

        // 评估每个测试用例
        for (i, (case, result)) in dataset.iter().zip(results.iter()).enumerate() {
            let mut metrics = std::collections::HashMap::new();

            for evaluator in &self.evaluators {
                match evaluator.evaluate_case(case, result) {
                    Ok(metric) => {
                        metrics.insert(evaluator.name().to_string(), metric);
                    }
                    Err(e) => {
                        eprintln!(
                            "Evaluator {} failed for case {}: {}",
                            evaluator.name(),
                            case.id,
                            e
                        );
                    }
                }
            }

            case_metrics.push((case.id.clone(), metrics));
        }

        let duration = start_time.elapsed();

        // 生成报告
        Ok(EvaluationReport {
            dataset_name: dataset.name.clone(),
            total_cases: dataset.len(),
            successful_cases: results.iter().filter(|r| r.success).count(),
            failed_cases: results.iter().filter(|r| !r.success).count(),
            evaluators: self.evaluators.iter().map(|e| e.name().to_string()).collect(),
            case_metrics,
            duration_ms: duration.as_millis() as u64,
        })
    }

    /// 模拟 Agent 执行并评估
    ///
    /// 这个方法用于演示,实际使用时应该集成真实的 Agent
    pub async fn evaluate_with_mock_agent<F>(
        &self,
        dataset: &TestDataset,
        agent_fn: F,
    ) -> Result<EvaluationReport, EvaluationError>
    where
        F: Fn(&str) -> TestCaseResult + Send + Sync,
    {
        let results: Vec<TestCaseResult> = dataset
            .iter()
            .map(|case| agent_fn(&case.input))
            .collect();

        self.evaluate(dataset, &results).await
    }
}

impl Default for EvaluationRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_mock_dataset() -> TestDataset {
        TestDataset::new("Test Dataset")
            .add_case(TestCase::new("1", "2+2", "4"))
            .add_case(TestCase::new("2", "3+3", "6"))
            .add_case(TestCase::new("3", "What is the capital of France?", "Paris"))
    }

    #[test]
    fn test_runner_creation() {
        let runner = EvaluationRunner::new();
        assert_eq!(runner.evaluators.len(), 0);
    }

    #[tokio::test]
    async fn test_evaluation() {
        use crate::evaluation::accuracy::AccuracyEvaluator;

        let dataset = create_mock_dataset();

        // 创建模拟结果
        let results = vec![
            TestCaseResult::success("4", 100, 10),
            TestCaseResult::success("6", 150, 12),
            TestCaseResult::success("Paris", 200, 15),
        ];

        let runner = EvaluationRunner::new()
            .with_evaluator(Arc::new(AccuracyEvaluator::new()));

        let report = runner.evaluate(&dataset, &results).await.unwrap();

        assert_eq!(report.total_cases, 3);
        assert_eq!(report.successful_cases, 3);
        assert_eq!(report.failed_cases, 0);
        assert_eq!(report.evaluators.len(), 1);
    }

    #[tokio::test]
    async fn test_evaluation_with_mismatch() {
        use crate::evaluation::accuracy::AccuracyEvaluator;

        let dataset = create_mock_dataset();
        let results = vec![
            TestCaseResult::success("4", 100, 10),
            TestCaseResult::success("6", 150, 12),
        ];

        let runner = EvaluationRunner::new()
            .with_evaluator(Arc::new(AccuracyEvaluator::new()));

        let result = runner.evaluate(&dataset, &results).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_agent_evaluation() {
        use crate::evaluation::latency::LatencyEvaluator;

        let dataset = create_mock_dataset();

        // 简单的模拟 Agent - 直接返回输入
        let mock_agent = |input: &str| TestCaseResult::success(input, 100, 10);

        let runner = EvaluationRunner::new()
            .with_evaluator(Arc::new(LatencyEvaluator::new()));

        let report = runner
            .evaluate_with_mock_agent(&dataset, mock_agent)
            .await
            .unwrap();

        assert_eq!(report.total_cases, 3);
        assert!(report.duration_ms > 0);
    }
}
