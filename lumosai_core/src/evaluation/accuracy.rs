//! # 准确性评估器
//!
//! 评估 Agent 回答的准确性、相关性和质量。

use crate::evaluation::{Evaluator, EvaluationError, MetricValue, TestCase, TestCaseResult};
use std::collections::HashMap;

/// 准确性评估器
pub struct AccuracyEvaluator {
    /// 是否启用语义相似度检测
    semantic_similarity: bool,
    /// 准确性阈值
    threshold: f64,
}

impl AccuracyEvaluator {
    /// 创建新的准确性评估器
    pub fn new() -> Self {
        Self {
            semantic_similarity: false,
            threshold: 0.7,
        }
    }

    /// 设置语义相似度检测
    pub fn with_semantic_similarity(mut self, enabled: bool) -> Self {
        self.semantic_similarity = enabled;
        self
    }

    /// 设置准确性阈值
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    /// 计算字符串相似度 (简单的编辑距离算法)
    fn calculate_similarity(&self, expected: &str, actual: &str) -> f64 {
        if expected == actual {
            return 1.0;
        }

        let expected_len = expected.chars().count();
        let actual_len = actual.chars().count();

        if expected_len == 0 || actual_len == 0 {
            return 0.0;
        }

        let distance = edit_distance(expected, actual);
        let max_len = expected_len.max(actual_len);

        1.0 - (distance as f64 / max_len as f64)
    }

    /// 提取关键信息并比较
    fn extract_key_info(&self, text: &str) -> Vec<String> {
        // 简单的关键信息提取: 数字、日期、专有名词
        let mut info = Vec::new();

        // 提取数字
        for token in text.split_whitespace() {
            if token.parse::<f64>().is_ok() {
                info.push(token.to_string());
            }
        }

        info
    }
}

impl Default for AccuracyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for AccuracyEvaluator {
    fn evaluate_case(&self, case: &TestCase, result: &TestCaseResult) -> Result<MetricValue, EvaluationError> {
        let expected = &case.expected_output;
        let actual = &result.output;

        // 1. 完全匹配检查
        if expected == actual {
            return Ok(MetricValue::Percentage(100.0));
        }

        // 2. 字符串相似度
        let similarity = self.calculate_similarity(expected, actual);
        if similarity >= self.threshold {
            return Ok(MetricValue::Percentage(similarity * 100.0));
        }

        // 3. 关键信息匹配
        let expected_info = self.extract_key_info(expected);
        let actual_info = self.extract_key_info(actual);

        if !expected_info.is_empty() {
            let matched = expected_info.iter()
                .filter(|info| actual_info.contains(info))
                .count();

            let info_accuracy = matched as f64 / expected_info.len() as f64;
            return Ok(MetricValue::Percentage(info_accuracy * 100.0));
        }

        // 如果都不匹配,返回相似度分数
        Ok(MetricValue::Percentage(similarity * 100.0))
    }

    fn name(&self) -> &str {
        "AccuracyEvaluator"
    }

    fn description(&self) -> &str {
        "评估 Agent 回答的准确性和相关性"
    }
}

/// 准确性指标
#[derive(Debug, Clone)]
pub struct AccuracyMetric {
    /// 准确率 (0-100)
    pub accuracy: f64,
    /// 完全匹配的数量
    pub exact_matches: i64,
    /// 总测试用例数
    pub total_cases: i64,
    /// 平均相似度
    pub avg_similarity: f64,
}

/// 计算编辑距离 (Levenshtein 距离)
fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    // 初始化第一行和第一列
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    // 动态规划计算
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = [
                matrix[i - 1][j] + 1,      // 删除
                matrix[i][j - 1] + 1,      // 插入
                matrix[i - 1][j - 1] + cost, // 替换
            ].iter().min().copied().unwrap();
        }
    }

    matrix[a_len][b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_case(input: &str, expected: &str) -> TestCase {
        TestCase {
            id: "test_1".to_string(),
            input: input.to_string(),
            expected_output: expected.to_string(),
            metadata: HashMap::new(),
        }
    }

    fn create_test_result(output: &str) -> TestCaseResult {
        TestCaseResult {
            output: output.to_string(),
            latency_ms: 100,
            tokens_used: 50,
            success: true,
            error: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_exact_match() {
        let evaluator = AccuracyEvaluator::new();
        let case = create_test_case("What is 2+2?", "4");
        let result = create_test_result("4");

        let metric = evaluator.evaluate_case(&case, &result).unwrap();
        assert_eq!(metric, MetricValue::Percentage(100.0));
    }

    #[test]
    fn test_no_match() {
        let evaluator = AccuracyEvaluator::new();
        let case = create_test_case("What is 2+2?", "4");
        let result = create_test_result("5");

        let metric = evaluator.evaluate_case(&case, &result).unwrap();
        // "4" 和 "5" 相似度很低
        if let MetricValue::Percentage(score) = metric {
            assert!(score < 100.0);
        } else {
            panic!("Expected Percentage metric");
        }
    }

    #[test]
    fn test_similarity_calculation() {
        let evaluator = AccuracyEvaluator::new();
        let case = create_test_case("Calculate", "The result is 42");
        let result = create_test_result("The answer is 42");

        let metric = evaluator.evaluate_case(&case, &result).unwrap();
        // 应该有一定的相似度
        if let MetricValue::Percentage(score) = metric {
            assert!(score > 0.0);
        } else {
            panic!("Expected Percentage metric");
        }
    }

    #[test]
    fn test_key_info_extraction() {
        let evaluator = AccuracyEvaluator::new();

        let info = evaluator.extract_key_info("The price is $19.99 and quantity is 100");
        assert!(info.contains(&"19.99".to_string()));
        assert!(info.contains(&"100".to_string()));
    }

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("a", "a"), 0);
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("", "abc"), 3);
    }
}
