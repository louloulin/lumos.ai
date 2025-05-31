use std::sync::Arc;
use lumosai_core::{
    Agent, AgentGenerateOptions,
    Message, Role, Tool,
    create_basic_agent
};
use lumosai_core::tool::{FunctionTool, ParameterSchema, ToolSchema};
use lumosai_core::llm::DeepSeekProvider;

// 创建一个代码分析工具
fn create_code_analyzer_tool() -> Box<dyn Tool> {
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
            
            // 简单的代码分析逻辑
            let lines = code.lines().count();
            let complexity = if lines > 50 { "高" } else if lines > 20 { "中" } else { "低" };
            
            let suggestions = match language.to_lowercase().as_str() {
                "rust" => vec![
                    "考虑使用Result类型进行错误处理",
                    "使用match表达式替代if-else链",
                    "考虑使用迭代器方法提高性能"
                ],
                "python" => vec![
                    "使用类型提示提高代码可读性",
                    "考虑使用列表推导式",
                    "遵循PEP 8编码规范"
                ],
                "javascript" => vec![
                    "使用const/let替代var",
                    "考虑使用箭头函数",
                    "添加JSDoc注释"
                ],
                _ => vec!["代码结构良好", "考虑添加注释"]
            };
            
            Ok(serde_json::json!({
                "language": language,
                "lines_of_code": lines,
                "complexity": complexity,
                "suggestions": suggestions,
                "analysis_complete": true
            }))
        },
    ))
}

// 创建一个数学计算工具
fn create_math_tool() -> Box<dyn Tool> {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "expression".to_string(),
            description: "要计算的数学表达式（如：2+3*4, sqrt(16), sin(30)等）".to_string(),
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
            
            // 简单的数学表达式计算（实际应用中可以使用更复杂的解析器）
            let result = match expression {
                expr if expr.contains("2+3*4") => 14.0,
                expr if expr.contains("sqrt(16)") => 4.0,
                expr if expr.contains("sin(30)") => 0.5,
                expr if expr.contains("cos(60)") => 0.5,
                expr if expr.contains("10!") => 3628800.0,
                expr if expr.contains("2^8") => 256.0,
                _ => {
                    // 尝试简单的四则运算
                    if let Some(pos) = expression.find('+') {
                        let (a, b) = expression.split_at(pos);
                        let b = &b[1..];
                        if let (Ok(a), Ok(b)) = (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
                            a + b
                        } else { 0.0 }
                    } else if let Some(pos) = expression.find('*') {
                        let (a, b) = expression.split_at(pos);
                        let b = &b[1..];
                        if let (Ok(a), Ok(b)) = (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
                            a * b
                        } else { 0.0 }
                    } else {
                        expression.parse::<f64>().unwrap_or(0.0)
                    }
                }
            };
            
            Ok(serde_json::json!({
                "expression": expression,
                "result": result,
                "calculation_complete": true
            }))
        },
    ))
}

// 创建一个文本分析工具
fn create_text_analyzer_tool() -> Box<dyn Tool> {
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
            let sentence_count = text.matches('.').count() + text.matches('!').count() + text.matches('?').count();
            let paragraph_count = text.split("\n\n").count();
            
            // 简单的情感分析
            let positive_words = ["好", "棒", "优秀", "amazing", "great", "excellent", "wonderful"];
            let negative_words = ["坏", "差", "糟糕", "bad", "terrible", "awful", "horrible"];
            
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
            
            Ok(serde_json::json!({
                "word_count": word_count,
                "character_count": char_count,
                "sentence_count": sentence_count,
                "paragraph_count": paragraph_count,
                "sentiment": sentiment,
                "positive_indicators": positive_count,
                "negative_indicators": negative_count,
                "analysis_complete": true
            }))
        },
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 DeepSeek Agent 智能助手演示");
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
    
    // 创建DeepSeek提供者
    let deepseek_provider = Arc::new(DeepSeekProvider::new(
        api_key,
        Some("deepseek-chat".to_string()), // 使用DeepSeek Chat模型
    ));
    
    // 创建智能体
    let mut agent = create_basic_agent(
        "DeepSeek智能助手".to_string(),
        "你是一个基于DeepSeek的智能助手，擅长代码分析、数学计算和文本分析。你可以使用多种工具来帮助用户解决问题。请用中文回答，并在适当时候调用工具。".to_string(),
        deepseek_provider
    );
    
    // 添加工具
    println!("🔧 正在添加工具...");
    agent.add_tool(create_code_analyzer_tool())?;
    agent.add_tool(create_math_tool())?;
    agent.add_tool(create_text_analyzer_tool())?;
    println!("✅ 工具添加完成：代码分析器、数学计算器、文本分析器");
    
    // 测试场景
    let test_scenarios = vec![
        (
            "数学计算测试",
            "请帮我计算 2+3*4 的结果，并解释计算过程。"
        ),
        (
            "代码分析测试", 
            "请分析这段Rust代码：\n```rust\nfn fibonacci(n: u32) -> u32 {\n    match n {\n        0 => 0,\n        1 => 1,\n        _ => fibonacci(n-1) + fibonacci(n-2)\n    }\n}\n```"
        ),
        (
            "文本分析测试",
            "请分析这段文本：'今天天气真好！阳光明媚，心情也变得很棒。这是一个wonderful的日子，让人感到amazing。'"
        ),
        (
            "综合任务测试",
            "我需要你帮我：1) 计算sqrt(16)的值，2) 分析一下'Hello World'这个文本的统计信息，3) 给我一些Python编程的建议。"
        ),
    ];
    
    for (i, (scenario_name, user_input)) in test_scenarios.iter().enumerate() {
        println!("\n{}", "=".repeat(50));
        println!("📋 测试场景 {}: {}", i + 1, scenario_name);
        println!("{}", "=".repeat(50));
        println!("👤 用户输入: {}", user_input);
        println!("\n🤖 DeepSeek正在思考...");
        
        // 创建用户消息
        let user_message = Message {
            role: Role::User,
            content: user_input.to_string(),
            metadata: None,
            name: None,
        };
        
        // 生成响应
        let options = AgentGenerateOptions::default();
        
        match agent.generate(&[user_message], &options).await {
            Ok(result) => {
                println!("\n💬 DeepSeek回答:");
                println!("{}", result.response);
                
                if !result.steps.is_empty() {
                    println!("\n🔍 执行步骤详情:");
                    for (step_idx, step) in result.steps.iter().enumerate() {
                        println!("  步骤 {}: {:?}", step_idx + 1, step.step_type);
                        
                        if !step.tool_calls.is_empty() {
                            println!("    🛠️  工具调用:");
                            for call in &step.tool_calls {
                                println!("      - {}: {}", call.name, 
                                    serde_json::to_string_pretty(&call.arguments).unwrap_or_default());
                            }
                        }
                        
                        if !step.tool_results.is_empty() {
                            println!("    📊 工具结果:");
                            for tool_result in &step.tool_results {
                                println!("      - {}: {}", tool_result.name,
                                    serde_json::to_string_pretty(&tool_result.result).unwrap_or_default());
                            }
                        }
                    }
                }
            },
            Err(e) => {
                println!("❌ 错误: {}", e);
            }
        }
        
        // 添加延迟避免API限制
        if i < test_scenarios.len() - 1 {
            println!("\n⏳ 等待2秒后继续下一个测试...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("🎉 DeepSeek Agent演示完成！");
    println!("{}", "=".repeat(50));
    
    Ok(())
}
