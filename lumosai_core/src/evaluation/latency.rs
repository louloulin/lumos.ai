//! # 延迟评估器
//!
//! 评估 Agent 的响应延迟和性能指标。

use crate::evaluation::{Evaluator, EvaluationError, MetricValue, TestCase, TestCaseResult};

/// 延迟评估器
pub struct LatencyEvaluator {
    /// P50 阈值 (毫秒)
    p50_threshold: u64,
    /// P95 阈值 (毫秒)
    p95_threshold: u64,
    /// P99 阈值 (毫秒)
    p99_threshold: u64,
}

impl LatencyEvaluator {
    /// 创建新的延迟评估器
    pub fn new() -> Self {
        Self {
            p50_threshold: 500,
            p95_threshold: 1000,
            p99_threshold: 2000,
        }
    }

    /// 设置 P50 阈值
    pub fn with_p50_threshold(mut self, threshold: u64) -> Self {
        self.p50_threshold = threshold;
        self
    }

    /// 设置 P95 阈值
    pub fn with_p95_threshold(mut self, threshold: u64) -> Self {
        self.p95_threshold = threshold;
        self
    }

    /// 设置 P99 阈值
    pub fn with_p99_threshold(mut self, threshold: u64) -> Self {
        self.p99_threshold = threshold;
        self
    }
}

impl Default for LatencyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for LatencyEvaluator {
    fn evaluate_case(&self, _case: &TestCase, result: &TestCaseResult) -> Result<MetricValue, EvaluationError> {
        // 返回延迟值
        Ok(MetricValue::Duration(result.latency_ms))
    }

    fn name(&self) -> &str {
        "LatencyEvaluator"
    }

    fn description(&self) -> &str {
        "评估 Agent 响应延迟和性能"
    }
}

/// 延迟指标聚合
pub struct LatencyMetric {
    /// P50 延迟 (毫秒)
    pub p50: u64,
    /// P95 延迟 (毫秒)
    pub p95: u64,
    /// P99 延迟 (毫秒)
    pub p99: u64,
    /// 平均延迟 (毫秒)
    pub avg: f64,
    /// 最小延迟 (毫秒)
    pub min: u64,
    /// 最大延迟 (毫秒)
    pub max: u64,
    /// 总请求数
    pub total_requests: usize,
    /// 超过 P95 阈值的请求比例
    pub p95_violation_rate: f64,
}

impl LatencyMetric {
    /// 从延迟列表计算指标
    pub fn from_latencies(latencies: &[u64], p95_threshold: u64) -> Self {
        if latencies.is_empty() {
            return Self {
                p50: 0,
                p95: 0,
                p99: 0,
                avg: 0.0,
                min: 0,
                max: 0,
                total_requests: 0,
                p95_violation_rate: 0.0,
            };
        }

        let mut sorted = latencies.to_vec();
        sorted.sort_unstable();

        let len = sorted.len();
        let min = sorted[0];
        let max = sorted[len - 1];
        let sum: u64 = sorted.iter().sum();
        let avg = sum as f64 / len as f64;

        // 计算百分位数
        let p50 = sorted[len * 50 / 100];
        let p95 = sorted[len * 95 / 100];
        let p99 = sorted[len * 99 / 100];

        // 计算 P95 违规率
        let p95_violations = sorted.iter().filter(|&&l| l > p95_threshold).count();
        let p95_violation_rate = p95_violations as f64 / len as f64;

        Self {
            p50,
            p95,
            p99,
            avg,
            min,
            max,
            total_requests: len,
            p95_violation_rate,
        }
    }

    /// 格式化报告
    pub fn format_report(&self) -> String {
        format!(
            "Latency Metrics:\n\
             - P50: {} ms\n\
             - P95: {} ms\n\
             - P99: {} ms\n\
             - Average: {:.2} ms\n\
             - Min: {} ms\n\
             - Max: {} ms\n\
             - Total Requests: {}\n\
             - P95 Violation Rate: {:.2}%",
            self.p50, self.p95, self.p99, self.avg, self.min, self.max,
            self.total_requests, self.p95_violation_rate * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_latency_evaluation() {
        let evaluator = LatencyEvaluator::new();

        let case = TestCase {
            id: "test_1".to_string(),
            input: "test".to_string(),
            expected_output: "result".to_string(),
            metadata: HashMap::new(),
        };

        let result = TestCaseResult {
            output: "result".to_string(),
            latency_ms: 123,
            tokens_used: 50,
            success: true,
            error: None,
            metadata: HashMap::new(),
        };

        let metric = evaluator.evaluate_case(&case, &result).unwrap();
        assert_eq!(metric, MetricValue::Duration(123));
    }

    #[test]
    fn test_latency_metrics_calculation() {
        let latencies = vec![
            100, 150, 200, 250, 300,  // 5 个
            400, 500, 600, 700, 800,  // 10 个
            900, 1000, 1100, 1200, 1300,  // 15 个
            1400, 1500, 1600, 1700, 1800,  // 20 个
        ];

        let metric = LatencyMetric::from_latencies(&latencies, 1000);

        assert_eq!(metric.min, 100);
        assert_eq!(metric.max, 1800);
        assert_eq!(metric.total_requests, 20);
        assert!(metric.avg > 0.0);
        assert!(metric.p95 > metric.p50);
        assert!(metric.p99 >= metric.p95);
    }

    #[test]
    fn test_empty_latencies() {
        let latencies = vec![];
        let metric = LatencyMetric::from_latencies(&latencies, 1000);

        assert_eq!(metric.total_requests, 0);
        assert_eq!(metric.min, 0);
        assert_eq!(metric.max, 0);
    }

    #[test]
    fn test_p95_violation_rate() {
        // P95 阈值设为 1000ms
        let latencies = vec![
            100, 200, 300, 400, 500,  // 正常
            600, 700, 800, 900, 950,  // 正常
            1100, 1200, 1300, 1500, 2000,  // 超过阈值
        ];

        let metric = LatencyMetric::from_latencies(&latencies, 1000);

        // 15 个中有 5 个超过 1000ms
        assert!((metric.p95_violation_rate - 0.333).abs() < 0.01);
    }
}
