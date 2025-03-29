use lomusai_core::Result;
use lomusai_core::agent::{Agent, MockAgent};
use lomusai_core::tool::MockTool;
use lumos_macro::workflow;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Workflow DSL示例");
    
    // 创建一些模拟代理用于示例
    let researcher = MockAgent::new("researcher")
        .with_tool(MockTool::new("search", |_| Ok(serde_json::json!({"results": ["找到了相关研究资料"]}))))
        .build();
    
    let writer = MockAgent::new("writer")
        .with_tool(MockTool::new("write", |_| Ok(serde_json::json!({"content": "这是根据研究资料撰写的内容"}))))
        .build();
    
    let reviewer = MockAgent::new("reviewer")
        .with_tool(MockTool::new("review", |_| Ok(serde_json::json!({"approved": true, "comments": "内容质量很好"}))))
        .build();

    // 使用workflow!宏定义一个内容创建工作流
    let content_workflow = workflow! {
        name: "content_creation",
        description: "创建高质量的内容",
        steps: {
            {
                name: "research",
                agent: researcher,
                instructions: "进行深入的主题研究",
                input: { "topic": "Rust中的过程宏" }
            },
            {
                name: "writing",
                agent: writer,
                instructions: "将研究结果整理成文章",
                when: { completed("research") }
            },
            {
                name: "review",
                agent: reviewer,
                instructions: "检查文章质量和准确性",
                when: { completed("writing") }
            },
            {
                name: "publish",
                agent: writer,
                instructions: "根据审阅意见发布最终文章",
                when: { 
                    completed("review") && 
                    output_contains("review", "approved", true) 
                }
            }
        }
    };

    // 执行工作流
    println!("开始执行工作流...");
    let input_data = serde_json::json!({
        "topic": "Rust宏系统指南",
        "target_audience": "Rust中级开发者"
    });
    
    let result = content_workflow.execute(input_data).await?;
    
    println!("工作流执行完成！");
    println!("结果: {}", serde_json::to_string_pretty(&result)?);
    
    Ok(())
} 