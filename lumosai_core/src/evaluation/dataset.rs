//! # 测试数据集
//!
//! 定义测试用例和数据集结构。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 测试用例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// 测试用例 ID
    pub id: String,
    /// 输入
    pub input: String,
    /// 期望输出
    pub expected_output: String,
    /// 元数据
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl TestCase {
    /// 创建新的测试用例
    pub fn new(id: impl Into<String>, input: impl Into<String>, expected: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            input: input.into(),
            expected_output: expected.into(),
            metadata: HashMap::new(),
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// 测试用例执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    /// 实际输出
    pub output: String,
    /// 延迟 (毫秒)
    pub latency_ms: u64,
    /// 使用的 Token 数量
    pub tokens_used: i64,
    /// 是否成功
    pub success: bool,
    /// 错误信息 (如果失败)
    pub error: Option<String>,
    /// 元数据
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl TestCaseResult {
    /// 创建成功的结果
    pub fn success(output: impl Into<String>, latency_ms: u64, tokens_used: i64) -> Self {
        Self {
            output: output.into(),
            latency_ms,
            tokens_used,
            success: true,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// 创建失败的结果
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            output: String::new(),
            latency_ms: 0,
            tokens_used: 0,
            success: false,
            error: Some(error.into()),
            metadata: HashMap::new(),
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// 测试数据集
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDataset {
    /// 数据集名称
    pub name: String,
    /// 测试用例集合
    pub cases: Vec<TestCase>,
    /// 数据集描述
    pub description: String,
}

impl TestDataset {
    /// 创建新的测试数据集
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            cases: Vec::new(),
            description: String::new(),
        }
    }

    /// 添加测试用例
    pub fn add_case(mut self, case: TestCase) -> Self {
        self.cases.push(case);
        self
    }

    /// 添加多个测试用例
    pub fn add_cases(mut self, cases: Vec<TestCase>) -> Self {
        self.cases.extend(cases);
        self
    }

    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// 获取测试用例数量
    pub fn len(&self) -> usize {
        self.cases.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.cases.is_empty()
    }

    /// 获取迭代器
    pub fn iter(&self) -> impl Iterator<Item = &TestCase> {
        self.cases.iter()
    }
}

impl FromIterator<TestCase> for TestDataset {
    fn from_iter<T: IntoIterator<Item = TestCase>>(iter: T) -> Self {
        let mut dataset = Self::new("Unnamed");
        dataset.cases = iter.into_iter().collect();
        dataset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_case_creation() {
        let case = TestCase::new("test_1", "What is 2+2?", "4")
            .with_metadata("category", "math");

        assert_eq!(case.id, "test_1");
        assert_eq!(case.input, "What is 2+2?");
        assert_eq!(case.expected_output, "4");
        assert_eq!(case.metadata.get("category"), Some(&"math".to_string()));
    }

    #[test]
    fn test_test_case_result() {
        let success = TestCaseResult::success("The answer is 4", 100, 50);
        assert!(success.success);
        assert_eq!(success.latency_ms, 100);
        assert_eq!(success.tokens_used, 50);

        let failure = TestCaseResult::failure("API error");
        assert!(!failure.success);
        assert_eq!(failure.error, Some("API error".to_string()));
    }

    #[test]
    fn test_dataset_creation() {
        let dataset = TestDataset::new("Math Problems")
            .with_description("Basic arithmetic problems")
            .add_case(TestCase::new("1", "2+2", "4"))
            .add_case(TestCase::new("2", "3+3", "6"));

        assert_eq!(dataset.name, "Math Problems");
        assert_eq!(dataset.len(), 2);
        assert!(!dataset.is_empty());
    }

    #[test]
    fn test_dataset_from_iterator() {
        let cases = vec![
            TestCase::new("1", "test1", "result1"),
            TestCase::new("2", "test2", "result2"),
        ];

        let dataset: TestDataset = cases.into_iter().collect();
        assert_eq!(dataset.len(), 2);
    }
}
