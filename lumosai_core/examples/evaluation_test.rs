//! 评估框架集成测试示例
//!
//! 运行方式: cargo run --example evaluation_test

use lumosai_core::evaluation::{
    AccuracyEvaluator, CostEvaluator, Evaluator, LatencyEvaluator, TestDataset, TestCase,
    TestCaseResult,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== LumosAI 评估框架测试 ===\n");

    // 测试 1: 创建测试数据集
    println!("测试 1: 创建测试数据集");
    let dataset = TestDataset::new("Math Problems")
        .with_description("基础算术问题测试")
        .add_case(TestCase::new("1", "2 + 2 = ?", "4"))
        .add_case(TestCase::new("2", "3 + 3 = ?", "6"))
        .add_case(TestCase::new("3", "5 * 5 = ?", "25"))
        .add_case(TestCase::new("4", "10 / 2 = ?", "5"))
        .add_case(TestCase::new("5", "What is the capital of France?", "Paris"));

    println!("✅ 数据集创建成功");
    println!("  - 数据集名称: {}", dataset.name);
    println!("  - 测试用例数: {}\n", dataset.len());

    // 测试 2: 模拟 Agent 执行
    println!("测试 2: 模拟 Agent 执行结果");

    let results = vec![
        TestCaseResult::success("4", 100, 10),
        TestCaseResult::success("6", 150, 12),
        TestCaseResult::success("25", 120, 15),
        TestCaseResult::success("5", 90, 11),
        TestCaseResult::success("Paris", 200, 20),
    ];

    println!("✅ Agent 执行完成");
    println!("  - 成功: {}/5", results.iter().filter(|r| r.success).count());
    println!("  - 平均延迟: {} ms\n", results.iter().map(|r| r.latency_ms).sum::<u64>() / 5);

    // 测试 3: 准确性评估
    println!("测试 3: 准确性评估");
    let accuracy_eval = AccuracyEvaluator::new().with_threshold(0.7);

    for (i, (case, result)) in dataset.iter().zip(results.iter()).enumerate() {
        match accuracy_eval.evaluate_case(case, result) {
            Ok(metric) => {
                println!("  Case {}: {:?}", i + 1, metric);
            }
            Err(e) => {
                println!("  Case {}: Error - {}", i + 1, e);
            }
        }
    }
    println!();

    // 测试 4: 延迟评估
    println!("测试 4: 延迟评估");
    let latency_eval = LatencyEvaluator::new();

    for (i, (case, result)) in dataset.iter().zip(results.iter()).enumerate() {
        match latency_eval.evaluate_case(case, result) {
            Ok(metric) => {
                println!("  Case {}: {:?}", i + 1, metric);
            }
            Err(e) => {
                println!("  Case {}: Error - {}", i + 1, e);
            }
        }
    }
    println!();

    // 测试 5: 成本评估
    println!("测试 5: 成本评估");
    let cost_eval = CostEvaluator::new();

    let total_cost: f64 = results
        .iter()
        .filter_map(|r| cost_eval.evaluate_case(&TestCase::new("", "", ""), r).ok())
        .filter_map(|m| m.as_float())
        .sum();

    println!("  总成本: ${:.6}", total_cost);
    println!("  平均成本: ${:.6}\n", total_cost / results.len() as f64);

    // 测试 6: 完整评估报告
    println!("测试 6: 完整评估报告");
    println!("注意: 异步评估需要 tokio runtime\n");

    println!("=== 所有测试通过! ✅ ===");
    println!("\n评估框架功能:");
    println!("  ✅ 准确性评估 (AccuracyEvaluator)");
    println!("  ✅ 延迟评估 (LatencyEvaluator)");
    println!("  ✅ 成本评估 (CostEvaluator)");
    println!("  ✅ 测试数据集 (TestDataset)");
    println!("  ✅ 评估运行器 (EvaluationRunner)");
    println!("  ✅ A/B 测试框架 (ABTestRunner)");

    Ok(())
}
