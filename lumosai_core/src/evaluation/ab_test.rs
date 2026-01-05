//! # A/B 测试框架
//!
//! 支持多个 Agent 版本的对比测试和统计显著性检验。

use crate::evaluation::{
    dataset::{TestDataset, TestCase},
    report::EvaluationReport,
    EvaluationError, Evaluator, MetricValue,
};
use std::collections::HashMap;

/// A/B 测试运行器
pub struct ABTestRunner {
    evaluators: Vec<Box<dyn Evaluator>>,
}

impl ABTestRunner {
    /// 创建新的 A/B 测试运行器
    pub fn new() -> Self {
        Self {
            evaluators: Vec::new(),
        }
    }

    /// 添加评估器
    pub fn with_evaluator(mut self, evaluator: Box<dyn Evaluator>) -> Self {
        self.evaluators.push(evaluator);
        self
    }

    /// 运行 A/B 测试
    ///
    /// 对比两个或多个 Agent 版本的性能
    pub async fn run_ab_test(
        &self,
        dataset: &TestDataset,
        version_a_results: &[crate::evaluation::dataset::TestCaseResult],
        version_b_results: &[crate::evaluation::dataset::TestCaseResult],
    ) -> Result<ABTestResult, EvaluationError> {
        if version_a_results.len() != dataset.len() || version_b_results.len() != dataset.len() {
            return Err(EvaluationError::DatasetError(
                "Results length mismatch".to_string(),
            ));
        }

        let mut version_a_metrics = HashMap::new();
        let mut version_b_metrics = HashMap::new();

        // 收集每个评估器的指标
        for evaluator in &self.evaluators {
            let name = evaluator.name();

            let mut a_values = Vec::new();
            let mut b_values = Vec::new();

            for (i, case) in dataset.iter().enumerate() {
                if let (Ok(a_metric), Ok(b_metric)) = (
                    evaluator.evaluate_case(case, &version_a_results[i]),
                    evaluator.evaluate_case(case, &version_b_results[i]),
                ) {
                    a_values.push(a_metric);
                    b_values.push(b_metric);
                }
            }

            version_a_metrics.insert(name.to_string(), a_values);
            version_b_metrics.insert(name.to_string(), b_values);
        }

        // 执行统计检验
        let statistical_tests = self.perform_statistical_tests(&version_a_metrics, &version_b_metrics);
        let winner = self.determine_winner(&statistical_tests);

        Ok(ABTestResult {
            dataset_name: dataset.name.clone(),
            version_a_metrics,
            version_b_metrics,
            statistical_tests,
            winner,
        })
    }

    /// 执行统计显著性检验
    fn perform_statistical_tests(
        &self,
        a_metrics: &HashMap<String, Vec<MetricValue>>,
        b_metrics: &HashMap<String, Vec<MetricValue>>,
    ) -> HashMap<String, StatisticalTest> {
        let mut results = HashMap::new();

        for (evaluator_name, a_values) in a_metrics {
            if let Some(b_values) = b_metrics.get(evaluator_name) {
                if let Some(test_result) = self.perform_t_test(a_values, b_values) {
                    results.insert(evaluator_name.clone(), test_result);
                }
            }
        }

        results
    }

    /// 执行 t 检验
    fn perform_t_test(&self, a_values: &[MetricValue], b_values: &[MetricValue]) -> Option<StatisticalTest> {
        // 提取数值
        let a_nums: Vec<f64> = a_values
            .iter()
            .filter_map(|v| match v {
                MetricValue::Float(f) | MetricValue::Percentage(f) => Some(*f),
                MetricValue::Integer(i) => Some(*i as f64),
                MetricValue::Duration(d) => Some(*d as f64),
                _ => None,
            })
            .collect();

        let b_nums: Vec<f64> = b_values
            .iter()
            .filter_map(|v| match v {
                MetricValue::Float(f) | MetricValue::Percentage(f) => Some(*f),
                MetricValue::Integer(i) => Some(*i as f64),
                MetricValue::Duration(d) => Some(*d as f64),
                _ => None,
            })
            .collect();

        if a_nums.len() < 2 || b_nums.len() < 2 {
            return None;
        }

        // 计算平均值和方差
        let a_mean: f64 = a_nums.iter().sum::<f64>() / a_nums.len() as f64;
        let b_mean: f64 = b_nums.iter().sum::<f64>() / b_nums.len() as f64;

        let a_var = variance(&a_nums, a_mean);
        let b_var = variance(&b_nums, b_mean);

        // 计算 t 统计量 (简化版,假设相等方差)
        let pooled_var = ((a_nums.len() as f64 - 1.0) * a_var + (b_nums.len() as f64 - 1.0) * b_var)
            / (a_nums.len() + b_nums.len() - 2) as f64;

        let std_error = (pooled_var / a_nums.len() as f64 + pooled_var / b_nums.len() as f64).sqrt();
        let t_statistic = (a_mean - b_mean) / std_error;

        // 简化的 p 值估计 (实际应该使用 t 分布)
        let p_value = if t_statistic.abs() > 1.96 { 0.05 } else { 0.10 };

        Some(StatisticalTest {
            test_name: "Independent t-test".to_string(),
            statistic: t_statistic,
            p_value,
            significant: p_value < 0.05,
            effect_size: (a_mean - b_mean).abs(),
        })
    }

    /// 确定胜者
    fn determine_winner(&self, tests: &HashMap<String, StatisticalTest>) -> Option<String> {
        let mut scores = HashMap::new();

        for (evaluator, test) in tests {
            if test.significant {
                let winner = if test.statistic > 0.0 { "A" } else { "B" };
                *scores.entry(winner.to_string()).or_insert(0) += 1;
            }
        }

        scores
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(winner, _)| winner)
    }
}

impl Default for ABTestRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// A/B 测试结果
#[derive(Debug, Clone)]
pub struct ABTestResult {
    /// 数据集名称
    pub dataset_name: String,
    /// 版本 A 的指标
    pub version_a_metrics: HashMap<String, Vec<MetricValue>>,
    /// 版本 B 的指标
    pub version_b_metrics: HashMap<String, Vec<MetricValue>>,
    /// 统计检验结果
    pub statistical_tests: HashMap<String, StatisticalTest>,
    /// 胜者 ("A", "B", 或 None)
    pub winner: Option<String>,
}

impl ABTestResult {
    /// 格式化报告
    pub fn format_report(&self) -> String {
        let mut output = String::new();

        output.push_str("========================================\n");
        output.push_str(&format!("A/B Test Report: {}\n", self.dataset_name));
        output.push_str("========================================\n\n");

        if let Some(winner) = &self.winner {
            output.push_str(&format!("Winner: Version {}\n\n", winner));
        } else {
            output.push_str("Winner: No significant difference\n\n");
        }

        output.push_str("Statistical Tests:\n");
        for (evaluator, test) in &self.statistical_tests {
            output.push_str(&format!(
                "\n{}:\n\
                 - Test: {}\n\
                 - t-statistic: {:.4}\n\
                 - p-value: {:.4}\n\
                 - Significant: {}\n\
                 - Effect size: {:.4}\n",
                evaluator,
                test.test_name,
                test.statistic,
                test.p_value,
                test.significant,
                test.effect_size
            ));
        }

        output.push_str("\n========================================\n");

        output
    }
}

/// 统计检验结果
#[derive(Debug, Clone)]
pub struct StatisticalTest {
    /// 检验名称
    pub test_name: String,
    /// 统计量
    pub statistic: f64,
    /// p 值
    pub p_value: f64,
    /// 是否显著 (p < 0.05)
    pub significant: bool,
    /// 效应大小
    pub effect_size: f64,
}

/// 计算方差
fn variance(values: &[f64], mean: f64) -> f64 {
    if values.len() <= 1 {
        return 0.0;
    }

    let sum_squared_diff: f64 = values.iter().map(|v| (v - mean).powi(2)).sum();
    sum_squared_diff / (values.len() - 1) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variance_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mean = 3.0;
        let var = variance(&values, mean);

        // 方差应该是 2.5
        assert!((var - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_ab_test_runner_creation() {
        let runner = ABTestRunner::new();
        assert_eq!(runner.evaluators.len(), 0);
    }
}
