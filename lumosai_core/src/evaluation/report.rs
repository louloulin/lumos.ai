//! # 评估报告
//!
//! 表示和格式化评估结果。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::evaluation::MetricValue;

/// 评估报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationReport {
    /// 数据集名称
    pub dataset_name: String,
    /// 总测试用例数
    pub total_cases: usize,
    /// 成功的测试用例数
    pub successful_cases: usize,
    /// 失败的测试用例数
    pub failed_cases: usize,
    /// 使用的评估器名称列表
    pub evaluators: Vec<String>,
    /// 每个测试用例的指标
    pub case_metrics: Vec<(String, HashMap<String, MetricValue>)>,
    /// 评估总耗时 (毫秒)
    pub duration_ms: u64,
}

impl EvaluationReport {
    /// 计算成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_cases == 0 {
            return 0.0;
        }
        (self.successful_cases as f64 / self.total_cases as f64) * 100.0
    }

    /// 获取指定评估器的所有指标值
    pub fn get_evaluator_metrics(&self, evaluator_name: &str) -> Vec<MetricValue> {
        self.case_metrics
            .iter()
            .filter_map(|(_, metrics)| metrics.get(evaluator_name).cloned())
            .collect()
    }

    /// 格式化为可读报告
    pub fn format_report(&self) -> String {
        let mut output = String::new();

        output.push_str("========================================\n");
        output.push_str(&format!("Evaluation Report: {}\n", self.dataset_name));
        output.push_str("========================================\n\n");

        output.push_str(&format!("Total Cases: {}\n", self.total_cases));
        output.push_str(&format!("Successful: {}\n", self.successful_cases));
        output.push_str(&format!("Failed: {}\n", self.failed_cases));
        output.push_str(&format!("Success Rate: {:.2}%\n", self.success_rate()));
        output.push_str(&format!("Duration: {} ms\n\n", self.duration_ms));

        output.push_str("Evaluators:\n");
        for evaluator in &self.evaluators {
            output.push_str(&format!("  - {}\n", evaluator));
        }

        output.push_str("\nPer-Case Metrics:\n");
        for (case_id, metrics) in &self.case_metrics {
            output.push_str(&format!("\n[{}]\n", case_id));
            for (evaluator, value) in metrics {
                output.push_str(&format!("  {}: {}\n", evaluator, format_metric_value(value)));
            }
        }

        output.push_str("\n========================================\n");

        output
    }

    /// 生成摘要
    pub fn summary(&self) -> EvaluationSummary {
        let mut evaluator_summaries = HashMap::new();

        for evaluator_name in &self.evaluators {
            let metrics = self.get_evaluator_metrics(evaluator_name);

            let summary = if metrics.is_empty() {
                None
            } else {
                Some(calculate_metric_summary(&metrics))
            };

            evaluator_summaries.insert(evaluator_name.clone(), summary);
        }

        EvaluationSummary {
            dataset_name: self.dataset_name.clone(),
            success_rate: self.success_rate(),
            total_cases: self.total_cases,
            evaluator_summaries,
        }
    }
}

/// 格式化指标值
fn format_metric_value(value: &MetricValue) -> String {
    match value {
        MetricValue::Float(v) => format!("{:.6}", v),
        MetricValue::Integer(v) => format!("{}", v),
        MetricValue::String(v) => v.clone(),
        MetricValue::Boolean(v) => format!("{}", v),
        MetricValue::Percentage(v) => format!("{:.2}%", v),
        MetricValue::Duration(v) => format!("{} ms", v),
    }
}

/// 计算指标摘要
fn calculate_metric_summary(metrics: &[MetricValue]) -> MetricSummary {
    let mut float_values = Vec::new();
    let mut int_values = Vec::new();

    for metric in metrics {
        match metric {
            MetricValue::Float(v) | MetricValue::Percentage(v) => {
                float_values.push(*v);
            }
            MetricValue::Integer(v) => {
                int_values.push(*v);
            }
            MetricValue::Duration(v) => {
                int_values.push(*v as i64);
            }
            _ => {}
        }
    }

    MetricSummary {
        avg_float: if float_values.is_empty() {
            None
        } else {
            Some(float_values.iter().sum::<f64>() / float_values.len() as f64)
        },
        avg_int: if int_values.is_empty() {
            None
        } else {
            Some(int_values.iter().sum::<i64>() / int_values.len() as i64)
        },
        count: metrics.len(),
    }
}

/// 指标摘要
#[derive(Debug, Clone)]
pub struct MetricSummary {
    /// 平均浮点值
    pub avg_float: Option<f64>,
    /// 平均整数值
    pub avg_int: Option<i64>,
    /// 指标数量
    pub count: usize,
}

/// 评估摘要
#[derive(Debug, Clone)]
pub struct EvaluationSummary {
    /// 数据集名称
    pub dataset_name: String,
    /// 成功率
    pub success_rate: f64,
    /// 总测试用例数
    pub total_cases: usize,
    /// 评估器摘要
    pub evaluator_summaries: HashMap<String, Option<MetricSummary>>,
}

impl EvaluationSummary {
    /// 格式化为可读摘要
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str("========================================\n");
        output.push_str(&format!("Summary: {}\n", self.dataset_name));
        output.push_str("========================================\n\n");

        output.push_str(&format!("Total Cases: {}\n", self.total_cases));
        output.push_str(&format!("Success Rate: {:.2}%\n\n", self.success_rate));

        output.push_str("Evaluator Summaries:\n");
        for (evaluator, summary) in &self.evaluator_summaries {
            output.push_str(&format!("\n{}:\n", evaluator));
            if let Some(s) = summary {
                if let Some(avg) = s.avg_float {
                    output.push_str(&format!("  Average: {:.4}\n", avg));
                }
                if let Some(avg) = s.avg_int {
                    output.push_str(&format!("  Average: {}\n", avg));
                }
                output.push_str(&format!("  Count: {}\n", s.count));
            } else {
                output.push_str("  No metrics available\n");
            }
        }

        output.push_str("\n========================================\n");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_rate() {
        let report = EvaluationReport {
            dataset_name: "Test".to_string(),
            total_cases: 10,
            successful_cases: 8,
            failed_cases: 2,
            evaluators: vec!["Accuracy".to_string()],
            case_metrics: Vec::new(),
            duration_ms: 1000,
        };

        assert_eq!(report.success_rate(), 80.0);
    }

    #[test]
    fn test_empty_report() {
        let report = EvaluationReport {
            dataset_name: "Test".to_string(),
            total_cases: 0,
            successful_cases: 0,
            failed_cases: 0,
            evaluators: vec![],
            case_metrics: vec![],
            duration_ms: 0,
        };

        assert_eq!(report.success_rate(), 0.0);
    }
}
