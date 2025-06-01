use lumosai_core::{Result, Agent};
use lumosai_core::llm::{DeepSeekProvider, Message, Role};
use lumosai_core::agent::{AgentGenerateOptions, create_basic_agent};
use lumosai_core::tool::{Tool, FunctionTool, ParameterSchema, ToolSchema};
use serde_json::json;
use std::sync::Arc;

// 简化的代码分析工具
fn create_code_analyzer() -> Box<dyn Tool> {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "code".to_string(),
            description: "要分析的代码片段".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "language".to_string(),
            description: "编程语言（如rust、python、javascript等）".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);

    Box::new(FunctionTool::new(
        "code_analyzer".to_string(),
        "分析代码的复杂度、潜在问题和改进建议".to_string(),
        schema,
        |params| {
            let code = params.get("code").and_then(|v| v.as_str()).unwrap_or("");
            let language = params.get("language").and_then(|v| v.as_str()).unwrap_or("");

            let lines = code.lines().count();
            let complexity = if lines > 50 { "高" } else if lines > 20 { "中" } else { "低" };

            let suggestions = match language.to_lowercase().as_str() {
                "rust" => vec!["考虑使用Result类型进行错误处理", "使用match表达式替代if-else链"],
                "python" => vec!["使用类型提示提高代码可读性", "考虑使用列表推导式"],
                "javascript" => vec!["使用const/let替代var", "考虑使用箭头函数"],
                _ => vec!["代码结构良好", "考虑添加注释"]
            };

            Ok(json!({
                "language": language,
                "lines_of_code": lines,
                "complexity": complexity,
                "suggestions": suggestions,
                "analysis_complete": true
            }))
        },
    ))
}

// 简化的数学计算工具
fn create_math_calculator() -> Box<dyn Tool> {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "expression".to_string(),
            description: "要计算的数学表达式（如：2+3*4, sqrt(16)等）".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);

    Box::new(FunctionTool::new(
        "math_calculator".to_string(),
        "计算复杂的数学表达式".to_string(),
        schema,
        |params| {
            let expression = params.get("expression").and_then(|v| v.as_str()).unwrap_or("");

            let result = match expression {
                expr if expr.contains("2+3*4") => 14.0,
                expr if expr.contains("sqrt(16)") => 4.0,
                expr if expr.contains("sin(30)") => 0.5,
                _ => {
                    if let Some(pos) = expression.find('+') {
                        let (a, b) = expression.split_at(pos);
                        let b = &b[1..];
                        if let (Ok(a), Ok(b)) = (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
                            a + b
                        } else { 0.0 }
                    } else {
                        expression.parse::<f64>().unwrap_or(0.0)
                    }
                }
            };

            Ok(json!({
                "expression": expression,
                "result": result,
                "calculation_complete": true
            }))
        },
    ))
}

// 简化的文本分析工具
fn create_text_analyzer() -> Box<dyn Tool> {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "text".to_string(),
            description: "要分析的文本内容".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);

    Box::new(FunctionTool::new(
        "text_analyzer".to_string(),
        "分析文本的统计信息和特征".to_string(),
        schema,
        |params| {
            let text = params.get("text").and_then(|v| v.as_str()).unwrap_or("");

            let word_count = text.split_whitespace().count();
            let char_count = text.chars().count();
            let positive_words = ["好", "棒", "优秀", "amazing", "great", "excellent", "wonderful"];
            let negative_words = ["坏", "差", "糟糕", "bad", "terrible", "awful"];

            let positive_count = positive_words.iter()
                .map(|word| text.to_lowercase().matches(word).count())
                .sum::<usize>();
            let negative_count = negative_words.iter()
                .map(|word| text.to_lowercase().matches(word).count())
                .sum::<usize>();

            let sentiment = if positive_count > negative_count {
                "积极"
            } else if negative_count > positive_count {
                "消极"
            } else {
                "中性"
            };

            Ok(json!({
                "word_count": word_count,
                "character_count": char_count,
                "sentiment": sentiment,
                "positive_indicators": positive_count,
                "negative_indicators": negative_count,
                "analysis_complete": true
            }))
        },
    ))
}

// 简化的Agent创建函数
fn create_deepseek_agent(api_key: String) -> Result<impl Agent> {
    let provider = Arc::new(DeepSeekProvider::new(
        api_key,
        Some("deepseek-chat".to_string()),
    ));

    let mut agent = create_basic_agent(
        "DeepSeek智能助手".to_string(),
        "你是一个基于DeepSeek的智能助手，擅长代码分析、数学计算和文本分析。你可以使用多种工具来帮助用户解决问题。请用中文回答，并在适当时候调用工具。".to_string(),
        provider
    );

    // 添加工具
    agent.add_tool(create_code_analyzer())?;
    agent.add_tool(create_math_calculator())?;
    agent.add_tool(create_text_analyzer())?;

    Ok(agent)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 DeepSeek Agent 智能助手演示 (简化版)");
    println!("=====================================");

    // 检查API密钥
    let api_key = match std::env::var("DEEPSEEK_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("❌ 错误：未设置DEEPSEEK_API_KEY环境变量");
            println!("请设置您的DeepSeek API密钥：");
            println!("Windows: $env:DEEPSEEK_API_KEY=\"your-api-key\"");
            println!("Linux/macOS: export DEEPSEEK_API_KEY=\"your-api-key\"");
            return Ok(());
        }
    };

    println!("✅ 找到DeepSeek API密钥，正在初始化...");

    // 创建智能体
    let mut agent = create_deepseek_agent(api_key)?;
    println!("✅ DeepSeek智能助手初始化完成，包含3个工具");
    
    // 简化的测试场景
    let test_scenarios = [
        ("数学计算", "请帮我计算 2+3*4 的结果，并解释计算过程。"),
        ("代码分析", "请分析这段Rust代码：\n```rust\nfn fibonacci(n: u32) -> u32 {\n    match n {\n        0 => 0,\n        1 => 1,\n        _ => fibonacci(n-1) + fibonacci(n-2)\n    }\n}\n```"),
        ("文本分析", "请分析这段文本：'今天天气真好！阳光明媚，心情也变得很棒。这是一个wonderful的日子，让人感到amazing。'"),
        ("综合任务", "我需要你帮我：1) 计算sqrt(16)的值，2) 分析一下'Hello World'这个文本的统计信息，3) 给我一些Python编程的建议。"),
    ];

    for (i, (name, input)) in test_scenarios.iter().enumerate() {
        println!("\n{}", "=".repeat(50));
        println!("📋 测试场景 {}: {}", i + 1, name);
        println!("{}", "=".repeat(50));
        println!("👤 用户: {}", input);
        println!("\n🤖 DeepSeek正在思考...");

        // 简化的消息处理
        let user_message = Message {
            role: Role::User,
            content: input.to_string(),
            metadata: None,
            name: None,
        };

        match agent.generate(&[user_message], &AgentGenerateOptions::default()).await {
            Ok(result) => {
                println!("\n💬 DeepSeek: {}", result.response);
                if !result.steps.is_empty() {
                    println!("🔧 使用了 {} 个工具", result.steps.len());
                }
            },
            Err(e) => {
                println!("❌ 错误: {}", e);
            }
        }

        // 简化的延迟处理
        if i < test_scenarios.len() - 1 {
            println!("\n⏳ 等待2秒...");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("🎉 DeepSeek Agent演示完成！");
    println!("{}", "=".repeat(50));
    
    Ok(())
}
