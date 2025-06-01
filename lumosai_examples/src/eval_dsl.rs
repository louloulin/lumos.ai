use lumosai_core::Result;
use lumosai_core::agent::{Agent, create_basic_agent};
use lumosai_core::llm::MockLlmProvider;
use lumosai_evals::{Metric, MetricResult};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;

struct AccuracyMetric;

#[async_trait]
impl Metric for AccuracyMetric {
    fn name(&self) -> &str {
        "accuracy"
    }

    fn description(&self) -> &str {
        "检查响应是否包含期望的关键字"
    }

    async fn measure(&self, input: &str, output: &str) -> lumosai_evals::Result<MetricResult> {
        // 简单示例：检查响应中是否包含期望的关键字
        let score = if output.contains(input) {
            1.0
        } else {
            0.0
        };

        Ok(MetricResult {
            score,
            info: HashMap::new(),
        })
    }
}

struct RelevanceMetric;

#[async_trait]
impl Metric for RelevanceMetric {
    fn name(&self) -> &str {
        "relevance"
    }

    fn description(&self) -> &str {
        "检查响应长度是否适中"
    }

    async fn measure(&self, _input: &str, output: &str) -> lumosai_evals::Result<MetricResult> {
        // 简单示例：检查响应长度是否适中
        let len = output.len();
        let score = if len > 10 && len < 500 {
            1.0
        } else if len > 0 {
            0.5
        } else {
            0.0
        };

        Ok(MetricResult {
            score,
            info: HashMap::new(),
        })
    }
}

struct CompletenessMetric;

#[async_trait]
impl Metric for CompletenessMetric {
    fn name(&self) -> &str {
        "completeness"
    }

    fn description(&self) -> &str {
        "检查响应是否包含所有期望的关键字"
    }

    async fn measure(&self, input: &str, output: &str) -> lumosai_evals::Result<MetricResult> {
        // 简单示例：检查响应是否包含所有期望的关键字
        let keywords = input.split(',').collect::<Vec<_>>();
        let matched = keywords.iter()
            .filter(|&keyword| output.contains(keyword.trim()))
            .count();
        let score = matched as f64 / keywords.len() as f64;

        Ok(MetricResult {
            score,
            info: HashMap::new(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("评估框架DSL示例");

    // 创建一个模拟代理用于评估
    let mock_responses = vec![
        "这是一个关于Rust的回答，涵盖了语法和内存安全".to_string(),
        "Rust使用Result和Option类型来处理错误".to_string(),
        "Rust的所有权系统包括所有权、借用和生命周期".to_string(),
    ];
    let llm_provider = Arc::new(MockLlmProvider::new(mock_responses));
    let agent = create_basic_agent(
        "test_agent".to_string(),
        "你是一个Rust专家".to_string(),
        llm_provider
    );

    // 手动创建评估指标
    let accuracy_metric = AccuracyMetric;
    let relevance_metric = RelevanceMetric;
    let completeness_metric = CompletenessMetric;

    // 测试用例
    let test_cases = vec![
        ("Rust有哪些特点？", "内存安全,并发,性能"),
        ("如何在Rust中处理错误？", "Result,Option,?运算符"),
        ("Rust的所有权系统是什么？", "所有权,借用,生命周期"),
    ];

    println!("开始评估代理性能...");

    let mut total_score = 0.0;
    let mut test_count = 0;

    for (i, (query, expected)) in test_cases.iter().enumerate() {
        println!("\n测试 #{}: {}", i + 1, query);

        // 模拟代理响应
        let response = match i {
            0 => "这是一个关于Rust的回答，涵盖了语法和内存安全",
            1 => "Rust使用Result和Option类型来处理错误",
            2 => "Rust的所有权系统包括所有权、借用和生命周期",
            _ => "默认响应",
        };

        // 评估各个指标
        let accuracy_result = accuracy_metric.measure(expected, response).await.map_err(|e| lumosai_core::Error::InvalidInput(e.to_string()))?;
        let relevance_result = relevance_metric.measure(expected, response).await.map_err(|e| lumosai_core::Error::InvalidInput(e.to_string()))?;
        let completeness_result = completeness_metric.measure(expected, response).await.map_err(|e| lumosai_core::Error::InvalidInput(e.to_string()))?;

        let case_score = (accuracy_result.score + relevance_result.score + completeness_result.score) / 3.0;
        total_score += case_score;
        test_count += 1;

        println!("  响应: {}", response);
        println!("  准确性: {:.2}", accuracy_result.score);
        println!("  相关性: {:.2}", relevance_result.score);
        println!("  完整性: {:.2}", completeness_result.score);
        println!("  总分: {:.2}", case_score);
    }

    let overall_score = total_score / test_count as f64;

    println!("\n评估完成！");
    println!("总体得分: {:.2}", overall_score);
    println!("通过阈值: {}", if overall_score >= 0.7 { "是" } else { "否" });

    // 展示评估框架的概念
    println!("\n评估框架概念演示:");
    println!("- 支持多种评估指标");
    println!("- 可配置的测试用例");
    println!("- 灵活的评分机制");
    println!("- 详细的结果报告");

    Ok(())
}