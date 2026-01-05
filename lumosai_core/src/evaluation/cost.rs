//! # 成本评估器
//!
//! 评估 Agent 的 Token 消耗和 API 调用成本。

use crate::evaluation::{Evaluator, EvaluationError, MetricValue, TestCase, TestCaseResult};
use std::collections::HashMap;

/// 成本评估器
pub struct CostEvaluator {
    /// 输入 Token 单价 (每 1K tokens)
    input_price_per_1k: f64,
    /// 输出 Token 单价 (每 1K tokens)
    output_price_per_1k: f64,
    /// 假设输入输出比例 (用于估算)
    assumed_input_ratio: f64,
}

impl CostEvaluator {
    /// 创建新的成本评估器
    ///
    /// 默认使用 GPT-4 的定价:
    /// - 输入: $0.03 / 1K tokens
    /// - 输出: $0.06 / 1K tokens
    pub fn new() -> Self {
        Self {
            input_price_per_1k: 0.03,
            output_price_per_1k: 0.06,
            assumed_input_ratio: 0.5,
        }
    }

    /// 设置输入 Token 单价
    pub fn with_input_price(mut self, price: f64) -> Self {
        self.input_price_per_1k = price;
        self
    }

    /// 设置输出 Token 单价
    pub fn with_output_price(mut self, price: f64) -> Self {
        self.output_price_per_1k = price;
        self
    }

    /// 设置假设的输入输出比例
    pub fn with_input_ratio(mut self, ratio: f64) -> Self {
        self.assumed_input_ratio = ratio;
        self
    }

    /// 计算单个请求的成本
    fn calculate_cost(&self, tokens_used: i64) -> f64 {
        if tokens_used <= 0 {
            return 0.0;
        }

        let input_tokens = (tokens_used as f64 * self.assumed_input_ratio) as i64;
        let output_tokens = tokens_used - input_tokens;

        let input_cost = (input_tokens as f64 / 1000.0) * self.input_price_per_1k;
        let output_cost = (output_tokens as f64 / 1000.0) * self.output_price_per_1k;

        input_cost + output_cost
    }
}

impl Default for CostEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for CostEvaluator {
    fn evaluate_case(&self, _case: &TestCase, result: &TestCaseResult) -> Result<MetricValue, EvaluationError> {
        let cost = self.calculate_cost(result.tokens_used);
        Ok(MetricValue::Float(cost))
    }

    fn name(&self) -> &str {
        "CostEvaluator"
    }

    fn description(&self) -> &str {
        "评估 Agent 的 Token 消耗和成本"
    }
}

/// 成本指标聚合
pub struct CostMetric {
    /// 总成本 (美元)
    pub total_cost: f64,
    /// 总 Token 数
    pub total_tokens: i64,
    /// 平均每次请求成本
    pub avg_cost_per_request: f64,
    /// 平均每次请求 Token 数
    pub avg_tokens_per_request: i64,
    /// 总请求数
    pub total_requests: usize,
    /// 成本最高的请求
    pub max_cost: f64,
    /// 成本最低的请求
    pub min_cost: f64,
}

impl CostMetric {
    /// 从成本列表计算指标
    pub fn from_costs(costs: &[f64], tokens: &[i64]) -> Self {
        if costs.is_empty() {
            return Self {
                total_cost: 0.0,
                total_tokens: 0,
                avg_cost_per_request: 0.0,
                avg_tokens_per_request: 0,
                total_requests: 0,
                max_cost: 0.0,
                min_cost: 0.0,
            };
        }

        let total_cost: f64 = costs.iter().sum();
        let total_tokens: i64 = tokens.iter().sum();
        let total_requests = costs.len();

        let avg_cost_per_request = total_cost / total_requests as f64;
        let avg_tokens_per_request = total_tokens / total_requests as i64;

        let max_cost = costs.iter().fold(0.0_f64, |a, &b| a.max(b));
        let min_cost = costs.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        Self {
            total_cost,
            total_tokens,
            avg_cost_per_request,
            avg_tokens_per_request,
            total_requests,
            max_cost,
            min_cost,
        }
    }

    /// 格式化报告
    pub fn format_report(&self) -> String {
        format!(
            "Cost Metrics:\n\
             - Total Cost: ${:.6}\n\
             - Total Tokens: {}\n\
             - Avg Cost/Request: ${:.6}\n\
             - Avg Tokens/Request: {}\n\
             - Max Cost: ${:.6}\n\
             - Min Cost: ${:.6}\n\
             - Total Requests: {}",
            self.total_cost,
            self.total_tokens,
            self.avg_cost_per_request,
            self.avg_tokens_per_request,
            self.max_cost,
            self.min_cost,
            self.total_requests
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_case(input: &str) -> TestCase {
        TestCase {
            id: "test_1".to_string(),
            input: input.to_string(),
            expected_output: "result".to_string(),
            metadata: HashMap::new(),
        }
    }

    fn create_test_result(tokens: i64) -> TestCaseResult {
        TestCaseResult {
            output: "result".to_string(),
            latency_ms: 100,
            tokens_used: tokens,
            success: true,
            error: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_cost_evaluation() {
        let evaluator = CostEvaluator::new();
        let case = create_test_case("test");
        let result = create_test_result(1000); // 1K tokens

        let metric = evaluator.evaluate_case(&case, &result).unwrap();

        if let MetricValue::Float(cost) = metric {
            // 1K tokens = 500 input + 500 output
            // 500/1000 * 0.03 + 500/1000 * 0.06 = 0.015 + 0.03 = 0.045
            assert!((cost - 0.045).abs() < 0.001);
        } else {
            panic!("Expected Float metric");
        }
    }

    #[test]
    fn test_zero_tokens() {
        let evaluator = CostEvaluator::new();
        let case = create_test_case("test");
        let result = create_test_result(0);

        let metric = evaluator.evaluate_case(&case, &result).unwrap();
        assert_eq!(metric, MetricValue::Float(0.0));
    }

    #[test]
    fn test_cost_metrics_aggregation() {
        let costs = vec![0.01, 0.02, 0.03, 0.04, 0.05];
        let tokens = vec![100, 200, 300, 400, 500];

        let metric = CostMetric::from_costs(&costs, &tokens);

        assert_eq!(metric.total_requests, 5);
        assert_eq!(metric.total_tokens, 1500);
        assert!((metric.total_cost - 0.15).abs() < 0.001);
        assert!((metric.avg_cost_per_request - 0.03).abs() < 0.001);
        assert_eq!(metric.avg_tokens_per_request, 300);
        assert_eq!(metric.max_cost, 0.05);
        assert_eq!(metric.min_cost, 0.01);
    }

    #[test]
    fn test_custom_pricing() {
        let evaluator = CostEvaluator::new()
            .with_input_price(0.01)
            .with_output_price(0.02);

        let case = create_test_case("test");
        let result = create_test_result(1000);

        let metric = evaluator.evaluate_case(&case, &result).unwrap();

        if let MetricValue::Float(cost) = metric {
            // 1K tokens = 500 input + 500 output
            // 500/1000 * 0.01 + 500/1000 * 0.02 = 0.005 + 0.01 = 0.015
            assert!((cost - 0.015).abs() < 0.001);
        } else {
            panic!("Expected Float metric");
        }
    }
}
