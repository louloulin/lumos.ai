//! LumosAI 评估框架基础功能演示
//!
//! 这个示例展示了如何使用 lumosai_evals 模块进行 AI 模型输出的评估。
//! 包括准确性评估、相关性评估、一致性评估等多种评估指标。

use lumosai_evals::{
    evaluator::{Evaluator, MetricBasedEvaluator},
    metrics::{Metric, MetricResult},
    types::{EvalOptions, EvalResult, TestInfo},
    Result,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio;

/// 模拟指标，用于演示评估框架的基本功能
struct MockMetric {
    name: String,
    description: String,
}

impl MockMetric {
    fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

#[async_trait::async_trait]
impl Metric for MockMetric {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn measure(&self, input: &str, output: &str) -> Result<MetricResult> {
        println!("� 计算 {} 指标:", self.name);
        println!("   输入: {}", input);
        println!("   输出: {}", output);

        // 模拟评估逻辑：基于输出长度和关键词匹配计算得分
        let mut score: f64 = 0.5; // 基础分数

        // 如果输出不为空，加分
        if !output.is_empty() {
            score += 0.2;
        }

        // 如果输出包含输入中的关键词，加分
        if output.contains("AI") || output.contains("人工智能") || output.contains("机器学习")
        {
            score += 0.2;
        }

        // 如果输出长度合理，加分
        if output.len() > 10 && output.len() < 1000 {
            score += 0.1;
        }

        // 确保分数在 0-1 范围内
        score = score.min(1.0).max(0.0);

        // 创建附加信息
        let mut info = HashMap::new();
        info.insert(
            "length_score".to_string(),
            json!(output.len() as f64 / 100.0),
        );
        info.insert("keyword_match".to_string(), json!(output.contains("AI")));
        info.insert("input_length".to_string(), json!(input.len()));
        info.insert("output_length".to_string(), json!(output.len()));

        println!("   得分: {:.3}", score);

        Ok(MetricResult { score, info })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 LumosAI 评估框架基础功能演示");
    println!("==================================================");

    // 1. 创建评估指标
    println!("\n📊 1. 创建评估指标");
    let accuracy_metric = Arc::new(MockMetric::new(
        "accuracy",
        "准确性指标：评估输出与期望结果的匹配程度",
    ));
    let relevance_metric = Arc::new(MockMetric::new(
        "relevance",
        "相关性指标：评估输出与输入的相关程度",
    ));
    let coherence_metric = Arc::new(MockMetric::new(
        "coherence",
        "连贯性指标：评估输出的逻辑连贯性",
    ));

    println!("✅ 创建了 3 个评估指标:");
    println!(
        "   - {}: {}",
        accuracy_metric.name(),
        accuracy_metric.description()
    );
    println!(
        "   - {}: {}",
        relevance_metric.name(),
        relevance_metric.description()
    );
    println!(
        "   - {}: {}",
        coherence_metric.name(),
        coherence_metric.description()
    );

    // 2. 创建基于指标的评估器
    println!("\n🔍 2. 创建基于指标的评估器");
    let accuracy_evaluator = MetricBasedEvaluator::new("准确性评估器", accuracy_metric.clone());
    let relevance_evaluator = MetricBasedEvaluator::new("相关性评估器", relevance_metric.clone());
    let coherence_evaluator = MetricBasedEvaluator::new("连贯性评估器", coherence_metric.clone());

    println!("✅ 创建了 3 个评估器:");
    println!("   - {}", accuracy_evaluator.name());
    println!("   - {}", relevance_evaluator.name());
    println!("   - {}", coherence_evaluator.name());

    // 3. 准备测试数据
    println!("\n🧪 3. 准备测试数据");
    let test_info = TestInfo {
        test_name: Some("AI 基础知识测试".to_string()),
        test_path: Some("tests/ai_basics.json".to_string()),
        tags: vec!["AI".to_string(), "基础".to_string(), "知识".to_string()],
        description: Some("测试 AI 模型对基础概念的理解".to_string()),
    };

    let test_cases = vec![
        (
            "什么是人工智能？",
            "人工智能是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的系统。",
        ),
        (
            "解释机器学习的基本概念",
            "机器学习是人工智能的一个子集，它使计算机能够从数据中学习并做出预测或决策。",
        ),
        (
            "深度学习有什么应用？",
            "深度学习在图像识别、自然语言处理、语音识别等领域有广泛应用。",
        ),
    ];

    println!("✅ 准备了 {} 个测试案例", test_cases.len());
    for (i, (input, _)) in test_cases.iter().enumerate() {
        println!("   {}. {}", i + 1, input);
    }

    // 4. 模拟 AI 模型输出
    println!("\n🤖 4. 模拟 AI 模型输出");
    let model_outputs = vec![
        "人工智能是一门计算机科学分支，专注于创建智能系统。",
        "机器学习确实是 AI 的一部分，它让计算机能够从数据中学习。",
        "深度学习的应用很广泛，包括图像处理等领域。",
    ];

    println!("✅ 生成了 {} 个模型输出", model_outputs.len());

    // 5. 执行评估
    println!("\n� 5. 执行评估");
    let eval_options = EvalOptions {
        global_run_id: Some("demo_run_001".to_string()),
        run_id: Some("eval_run_001".to_string()),
        target_name: Some("MockAI".to_string()),
        test_info: Some(test_info.clone()),
        instructions: Some("请评估 AI 模型输出的质量".to_string()),
        log_results: true,
    };

    let mut total_score = 0.0;
    let mut evaluation_results = Vec::new();

    for (i, ((input, _expected), output)) in test_cases.iter().zip(model_outputs.iter()).enumerate()
    {
        println!("\n--- 测试案例 {} ---", i + 1);
        println!("输入: {}", input);
        println!("输出: {}", output);

        // 使用准确性评估器
        let eval_result = accuracy_evaluator
            .evaluate(input, output, &eval_options)
            .await?;

        println!("\n📈 评估结果:");
        println!("   评估器: {}", eval_result.evaluator_name);
        println!("   指标: {}", eval_result.metric_name);
        println!("   总分: {:.3}", eval_result.score);
        println!("   评估ID: {}", eval_result.id);
        println!("   运行ID: {}", eval_result.run_id);

        total_score += eval_result.score;
        evaluation_results.push(eval_result);

        // 直接测试指标
        println!("\n📊 直接指标测试:");
        let metric_result = accuracy_metric.measure(input, output).await?;
        println!("   指标得分: {:.3}", metric_result.score);
        println!("   附加信息: {:?}", metric_result.info);
    }

    // 6. 汇总结果
    println!("\n📈 6. 评估结果汇总");
    println!("==================================================");
    let average_score = total_score / test_cases.len() as f64;
    println!("🎯 平均评估分数: {:.3}", average_score);
    println!("📊 总测试案例数: {}", test_cases.len());
    println!("✅ 评估完成案例数: {}", evaluation_results.len());

    // 评估等级
    let grade = if average_score >= 0.9 {
        "优秀 (A)"
    } else if average_score >= 0.8 {
        "良好 (B)"
    } else if average_score >= 0.7 {
        "中等 (C)"
    } else if average_score >= 0.6 {
        "及格 (D)"
    } else {
        "不及格 (F)"
    };

    println!("🏆 整体评估等级: {}", grade);

    // 7. 生成评估报告
    println!("\n📋 7. 评估报告");
    println!("==================================================");
    println!("评估框架: LumosAI Evals v0.2.0");
    println!(
        "评估时间: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("评估器数量: 3 个");
    println!("指标数量: 3 个");
    println!("测试案例: {} 个", test_cases.len());
    println!("平均分数: {:.3}", average_score);
    println!("评估等级: {}", grade);

    println!("\n✅ 所有评估功能演示完成！");
    println!("🎉 LumosAI 评估框架现在可以用于评估 AI 模型的输出质量。");

    Ok(())
}
