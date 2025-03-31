use lumosai_core::Result;
use lumosai_core::agent::{Agent, MockAgent};
use lumosai_core::eval::{EvalSuite, Metric};
use lumos_macro::eval_suite;

struct AccuracyMetric;
impl Metric for AccuracyMetric {
    fn name(&self) -> &str {
        "accuracy"
    }
    
    fn compute(&self, response: &str, expected: &str) -> f64 {
        // 简单示例：检查响应中是否包含期望的关键字
        if response.contains(expected) {
            1.0
        } else {
            0.0
        }
    }
}

struct RelevanceMetric;
impl Metric for RelevanceMetric {
    fn name(&self) -> &str {
        "relevance"
    }
    
    fn compute(&self, response: &str, _expected: &str) -> f64 {
        // 简单示例：检查响应长度是否适中
        let len = response.len();
        if len > 10 && len < 500 {
            1.0
        } else if len > 0 {
            0.5
        } else {
            0.0
        }
    }
}

struct CompletenessMetric;
impl Metric for CompletenessMetric {
    fn name(&self) -> &str {
        "completeness"
    }
    
    fn compute(&self, response: &str, expected: &str) -> f64 {
        // 简单示例：检查响应是否包含所有期望的关键字
        let keywords = expected.split(',').collect::<Vec<_>>();
        let matched = keywords.iter()
            .filter(|k| response.contains(*k))
            .count();
        
        matched as f64 / keywords.len() as f64
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("评估框架DSL示例");
    
    // 创建一个模拟代理用于评估
    let agent = MockAgent::new("test_agent")
        .with_response("这是一个关于Rust的回答，涵盖了语法和内存安全")
        .build();
    
    // 使用eval_suite!宏定义一个评估套件
    let suite = eval_suite! {
        name: "agent_performance",
        
        metrics: {
            accuracy: AccuracyMetric,
            relevance: RelevanceMetric,
            completeness: CompletenessMetric
        },
        
        test_cases: [
            {
                query: "Rust有哪些特点？",
                expected: "内存安全,并发,性能",
                weight: 1.0
            },
            {
                query: "如何在Rust中处理错误？",
                expected: "Result,Option,?运算符",
                weight: 0.8
            },
            {
                query: "Rust的所有权系统是什么？",
                expected: "所有权,借用,生命周期",
                weight: 1.2
            }
        ],
        
        thresholds: {
            accuracy: 0.8,
            relevance: 0.7,
            completeness: 0.6
        },
        
        reporting: {
            format: "markdown",
            output: "./reports/eval_results.md",
            include_responses: true
        }
    };

    // 运行评估
    println!("开始评估代理性能...");
    
    let results = suite.run(agent).await?;
    
    println!("评估完成！");
    println!("总分: {:.2}", results.overall_score);
    println!("各指标得分:");
    for (metric, score) in &results.metric_scores {
        println!("- {}: {:.2}", metric, score);
    }
    
    println!("\n测试用例结果:");
    for (i, case_result) in results.case_results.iter().enumerate() {
        println!("测试 #{}: 查询=\"{}\"", i + 1, case_result.query);
        println!("  得分: {:.2}", case_result.score);
        println!("  通过: {}", if case_result.passed { "是" } else { "否" });
    }
    
    // 高级评估配置
    let advanced_suite = eval_suite! {
        name: "advanced_evaluation",
        
        metrics: {
            accuracy: AccuracyMetric,
            relevance: RelevanceMetric,
            completeness: CompletenessMetric,
            custom: {
                name: "response_time",
                implementation: |response: &str, _: &str| -> f64 {
                    // 动态定义的指标实现
                    if response.len() > 100 { 0.7 } else { 1.0 }
                }
            }
        },
        
        test_cases: {
            source: "./tests/eval_test_cases.json",
            filter: r#"{ "category": "beginner" }"#
        },
        
        reporting: {
            format: "html",
            template: "./templates/report.html",
            webhook: "https://api.company.com/eval-results"
        },
        
        options: {
            parallel: true,
            timeout: 10000,
            retry_count: 2
        }
    };
    
    println!("\n高级评估配置已创建");
    
    Ok(())
} 