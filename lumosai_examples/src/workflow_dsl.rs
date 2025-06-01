use lumosai_core::Result;
use lumosai_core::agent::{Agent, create_basic_agent};
use lumosai_core::llm::MockLlmProvider;
use std::sync::Arc;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Workflow DSL示例");

    // 创建一些模拟代理用于示例
    let researcher_responses = vec![
        "我找到了关于Rust宏系统的详细资料，包括声明式宏和过程宏的区别。".to_string(),
    ];
    let researcher_llm = Arc::new(MockLlmProvider::new(researcher_responses));
    let researcher = create_basic_agent(
        "researcher".to_string(),
        "你是一个专业的研究员，擅长收集和分析技术资料。".to_string(),
        researcher_llm
    );

    let writer_responses = vec![
        "基于研究资料，我撰写了一篇关于Rust宏系统的详细指南。".to_string(),
    ];
    let writer_llm = Arc::new(MockLlmProvider::new(writer_responses));
    let writer = create_basic_agent(
        "writer".to_string(),
        "你是一个技术写作专家，能够将复杂的技术概念转化为易懂的文章。".to_string(),
        writer_llm
    );

    let reviewer_responses = vec![
        "文章质量很好，内容准确，结构清晰，可以发布。".to_string(),
    ];
    let reviewer_llm = Arc::new(MockLlmProvider::new(reviewer_responses));
    let reviewer = create_basic_agent(
        "reviewer".to_string(),
        "你是一个技术审阅专家，负责检查文章的准确性和质量。".to_string(),
        reviewer_llm
    );

    // 模拟工作流执行步骤
    println!("开始执行内容创建工作流...");

    // 步骤1: 研究
    println!("\n步骤1: 研究阶段");
    println!("研究员正在收集关于 'Rust宏系统指南' 的资料...");
    let research_result = json!({
        "status": "completed",
        "findings": [
            "Rust有两种类型的宏：声明式宏和过程宏",
            "声明式宏使用macro_rules!定义",
            "过程宏有三种类型：derive、attribute和function-like"
        ],
        "sources": ["Rust官方文档", "The Rust Programming Language书籍"]
    });
    println!("研究完成: {}", serde_json::to_string_pretty(&research_result)?);

    // 步骤2: 写作
    println!("\n步骤2: 写作阶段");
    println!("写作专家正在基于研究结果撰写文章...");
    let writing_result = json!({
        "status": "completed",
        "content": {
            "title": "Rust宏系统完整指南",
            "sections": [
                "宏的基本概念",
                "声明式宏详解",
                "过程宏深入解析",
                "实际应用案例"
            ],
            "word_count": 2500
        }
    });
    println!("写作完成: {}", serde_json::to_string_pretty(&writing_result)?);

    // 步骤3: 审阅
    println!("\n步骤3: 审阅阶段");
    println!("审阅专家正在检查文章质量...");
    let review_result = json!({
        "status": "completed",
        "approved": true,
        "score": 9.2,
        "comments": [
            "内容准确性高",
            "结构清晰易懂",
            "示例代码质量好",
            "建议增加更多实际应用场景"
        ]
    });
    println!("审阅完成: {}", serde_json::to_string_pretty(&review_result)?);

    // 步骤4: 发布
    println!("\n步骤4: 发布阶段");
    println!("根据审阅意见进行最终发布...");
    let publish_result = json!({
        "status": "published",
        "url": "https://blog.example.com/rust-macro-guide",
        "publication_date": "2024-01-15",
        "final_score": 9.5
    });
    println!("发布完成: {}", serde_json::to_string_pretty(&publish_result)?);

    // 工作流总结
    let workflow_summary = json!({
        "workflow_name": "content_creation",
        "total_steps": 4,
        "execution_time": "45 minutes",
        "success": true,
        "final_output": publish_result
    });

    println!("\n工作流执行完成！");
    println!("工作流总结: {}", serde_json::to_string_pretty(&workflow_summary)?);

    // 展示工作流DSL的概念
    println!("\n工作流DSL概念演示:");
    println!("- 多代理协作");
    println!("- 步骤依赖管理");
    println!("- 条件执行逻辑");
    println!("- 结果传递机制");
    println!("- 错误处理和重试");

    Ok(())
}