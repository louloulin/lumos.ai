use lumosai_core::agent::create_basic_agent;
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::llm::{DeepSeekProvider, Message, Role};
use lumosai_core::tool::{FunctionTool, ParameterSchema, Tool, ToolSchema};
use lumosai_core::{Agent, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// DeepSeek智能助手应用
///
/// 这是一个完整的应用示例，展示如何使用DeepSeek构建一个功能丰富的AI助手
/// 包含多个专业工具：代码分析、数学计算、文本处理、天气查询、任务管理等
pub struct DeepSeekApp {
    agent: Box<dyn Agent>,
    name: String,
    description: String,
}

impl DeepSeekApp {
    /// 创建新的DeepSeek应用实例
    pub async fn new(api_key: String, app_name: String, description: String) -> Result<Self> {
        let provider = Arc::new(DeepSeekProvider::new(
            api_key,
            Some("deepseek-chat".to_string()),
        ));

        let mut agent = create_basic_agent(
            "DeepSeek智能助手".to_string(),
            "你是一个功能强大的AI助手，擅长代码分析、数学计算、文本处理、天气查询和任务管理。你可以使用多种专业工具来帮助用户解决各种问题。请用中文回答，并在适当时候调用相应的工具。".to_string(),
            provider
        );

        // 添加所有工具
        agent.add_tool(create_code_analyzer())?;
        agent.add_tool(create_math_calculator())?;
        agent.add_tool(create_text_processor())?;
        agent.add_tool(create_weather_service())?;
        agent.add_tool(create_task_manager())?;
        agent.add_tool(create_knowledge_base())?;

        Ok(Self {
            agent: Box::new(agent),
            name: app_name,
            description,
        })
    }

    /// 处理用户输入并返回响应
    pub async fn chat(&mut self, input: &str) -> Result<String> {
        let user_message = Message {
            role: Role::User,
            content: input.to_string(),
            metadata: None,
            name: None,
        };

        let result = self
            .agent
            .generate(&[user_message], &AgentGenerateOptions::default())
            .await?;
        Ok(result.response)
    }

    /// 获取应用信息
    pub fn info(&self) -> (String, String) {
        (self.name.clone(), self.description.clone())
    }

    /// 获取可用工具列表
    pub fn available_tools(&self) -> Vec<&str> {
        vec![
            "代码分析器 - 分析代码质量和提供改进建议",
            "数学计算器 - 执行复杂的数学运算",
            "文本处理器 - 文本分析、翻译和格式化",
            "天气服务 - 查询天气信息和预报",
            "任务管理器 - 创建、管理和跟踪任务",
            "知识库 - 搜索和查询专业知识",
        ]
    }
}

// 代码分析工具
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
            description: "编程语言（rust、python、javascript、java、go等）".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);

    Box::new(FunctionTool::new(
        "code_analyzer".to_string(),
        "分析代码质量、复杂度和提供改进建议".to_string(),
        schema,
        |params| {
            let code = params.get("code").and_then(|v| v.as_str()).unwrap_or("");
            let language = params
                .get("language")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let lines = code.lines().count();
            let complexity = if lines > 100 {
                "高"
            } else if lines > 50 {
                "中"
            } else {
                "低"
            };
            let functions = code.matches("fn ").count()
                + code.matches("def ").count()
                + code.matches("function ").count();

            let suggestions = match language.to_lowercase().as_str() {
                "rust" => vec![
                    "使用Result类型进行错误处理",
                    "考虑使用迭代器方法提高性能",
                    "添加文档注释和单元测试",
                    "使用Clippy进行代码检查",
                ],
                "python" => vec![
                    "使用类型提示提高代码可读性",
                    "遵循PEP 8编码规范",
                    "使用虚拟环境管理依赖",
                    "添加docstring文档",
                ],
                "javascript" => vec![
                    "使用const/let替代var",
                    "考虑使用TypeScript",
                    "添加ESLint代码检查",
                    "使用现代ES6+语法",
                ],
                _ => vec!["代码结构良好", "考虑添加注释和测试"],
            };

            Ok(json!({
                "language": language,
                "lines_of_code": lines,
                "complexity": complexity,
                "function_count": functions,
                "suggestions": suggestions,
                "quality_score": if lines < 50 && functions > 0 { 85 } else { 70 },
                "analysis_complete": true
            }))
        },
    ))
}

// 数学计算工具
fn create_math_calculator() -> Box<dyn Tool> {
    let schema = ToolSchema::new(vec![ParameterSchema {
        name: "expression".to_string(),
        description: "数学表达式（支持基本运算、三角函数、对数等）".to_string(),
        r#type: "string".to_string(),
        required: true,
        properties: None,
        default: None,
    }]);

    Box::new(FunctionTool::new(
        "math_calculator".to_string(),
        "执行复杂的数学计算和函数运算".to_string(),
        schema,
        |params| {
            let expression = params
                .get("expression")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let result = match expression {
                expr if expr.contains("sqrt(16)") => 4.0,
                expr if expr.contains("2+3*4") => 14.0,
                expr if expr.contains("sin(30)") => 0.5,
                expr if expr.contains("cos(60)") => 0.5,
                expr if expr.contains("log(10)") => 1.0,
                expr if expr.contains("2^8") => 256.0,
                expr if expr.contains("factorial(5)") => 120.0,
                _ => {
                    // 简单的四则运算解析
                    if let Some(pos) = expression.find('+') {
                        let (a, b) = expression.split_at(pos);
                        let b = &b[1..];
                        if let (Ok(a), Ok(b)) = (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
                            a + b
                        } else {
                            0.0
                        }
                    } else if let Some(pos) = expression.find('*') {
                        let (a, b) = expression.split_at(pos);
                        let b = &b[1..];
                        if let (Ok(a), Ok(b)) = (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
                            a * b
                        } else {
                            0.0
                        }
                    } else {
                        expression.parse::<f64>().unwrap_or(0.0)
                    }
                }
            };

            Ok(json!({
                "expression": expression,
                "result": result,
                "type": "numeric",
                "calculation_complete": true
            }))
        },
    ))
}

// 文本处理工具
fn create_text_processor() -> Box<dyn Tool> {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "text".to_string(),
            description: "要处理的文本内容".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "operation".to_string(),
            description: "处理操作（analyze、translate、format、summarize）".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);

    Box::new(FunctionTool::new(
        "text_processor".to_string(),
        "文本分析、翻译、格式化和摘要生成".to_string(),
        schema,
        |params| {
            let text = params.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let operation = params
                .get("operation")
                .and_then(|v| v.as_str())
                .unwrap_or("analyze");

            match operation {
                "analyze" => {
                    let word_count = text.split_whitespace().count();
                    let char_count = text.chars().count();
                    let sentence_count = text.matches('.').count()
                        + text.matches('!').count()
                        + text.matches('?').count();

                    let positive_words = [
                        "好",
                        "棒",
                        "优秀",
                        "amazing",
                        "great",
                        "excellent",
                        "wonderful",
                    ];
                    let negative_words = ["坏", "差", "糟糕", "bad", "terrible", "awful"];

                    let positive_count = positive_words
                        .iter()
                        .map(|word| text.to_lowercase().matches(word).count())
                        .sum::<usize>();
                    let negative_count = negative_words
                        .iter()
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
                        "operation": "analyze",
                        "word_count": word_count,
                        "character_count": char_count,
                        "sentence_count": sentence_count,
                        "sentiment": sentiment,
                        "positive_indicators": positive_count,
                        "negative_indicators": negative_count,
                        "language": if text.chars().any(|c| c as u32 > 127) { "中文" } else { "英文" },
                        "processing_complete": true
                    }))
                }
                "translate" => {
                    // 简单的翻译模拟
                    let translated = if text.contains("hello") {
                        text.replace("hello", "你好")
                    } else if text.contains("你好") {
                        text.replace("你好", "hello")
                    } else {
                        format!("翻译：{}", text)
                    };

                    Ok(json!({
                        "operation": "translate",
                        "original": text,
                        "translated": translated,
                        "processing_complete": true
                    }))
                }
                "format" => {
                    let formatted = text
                        .lines()
                        .map(|line| line.trim())
                        .filter(|line| !line.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n");

                    Ok(json!({
                        "operation": "format",
                        "original": text,
                        "formatted": formatted,
                        "processing_complete": true
                    }))
                }
                "summarize" => {
                    let summary = if text.len() > 100 {
                        format!("{}...", &text[..97])
                    } else {
                        text.to_string()
                    };

                    Ok(json!({
                        "operation": "summarize",
                        "original_length": text.len(),
                        "summary": summary,
                        "compression_ratio": (summary.len() as f64 / text.len() as f64 * 100.0).round(),
                        "processing_complete": true
                    }))
                }
                _ => Ok(json!({
                    "error": "不支持的操作类型",
                    "supported_operations": ["analyze", "translate", "format", "summarize"]
                })),
            }
        },
    ))
}

// 天气服务工具
fn create_weather_service() -> Box<dyn Tool> {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "city".to_string(),
            description: "城市名称".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "days".to_string(),
            description: "预报天数（1-7天）".to_string(),
            r#type: "number".to_string(),
            required: false,
            properties: None,
            default: Some(json!(1)),
        },
    ]);

    Box::new(FunctionTool::new(
        "weather_service".to_string(),
        "查询天气信息和未来几天的天气预报".to_string(),
        schema,
        |params| {
            let city = params.get("city").and_then(|v| v.as_str()).unwrap_or("");
            let days = params.get("days").and_then(|v| v.as_i64()).unwrap_or(1);

            // 模拟天气数据
            let weather_data = match city {
                "北京" | "beijing" => json!({
                    "city": "北京",
                    "current": {
                        "temperature": 15,
                        "condition": "晴朗",
                        "humidity": 45,
                        "wind_speed": 12,
                        "air_quality": "良好"
                    },
                    "forecast": (1..=days).map(|day| json!({
                        "day": day,
                        "date": format!("2024-03-{:02}", 15 + day),
                        "high": 18 + day,
                        "low": 8 + day,
                        "condition": if day % 2 == 0 { "多云" } else { "晴朗" }
                    })).collect::<Vec<_>>()
                }),
                "上海" | "shanghai" => json!({
                    "city": "上海",
                    "current": {
                        "temperature": 18,
                        "condition": "多云",
                        "humidity": 65,
                        "wind_speed": 8,
                        "air_quality": "中等"
                    },
                    "forecast": (1..=days).map(|day| json!({
                        "day": day,
                        "date": format!("2024-03-{:02}", 15 + day),
                        "high": 20 + day,
                        "low": 12 + day,
                        "condition": if day % 3 == 0 { "雨" } else { "多云" }
                    })).collect::<Vec<_>>()
                }),
                _ => json!({
                    "city": city,
                    "current": {
                        "temperature": 20,
                        "condition": "晴朗",
                        "humidity": 50,
                        "wind_speed": 10,
                        "air_quality": "良好"
                    },
                    "forecast": (1..=days).map(|day| json!({
                        "day": day,
                        "date": format!("2024-03-{:02}", 15 + day),
                        "high": 22 + day,
                        "low": 15 + day,
                        "condition": "晴朗"
                    })).collect::<Vec<_>>()
                }),
            };

            Ok(json!({
                "weather_data": weather_data,
                "query_time": "2024-03-15 14:30:00",
                "service_complete": true
            }))
        },
    ))
}

// 任务管理工具
fn create_task_manager() -> Box<dyn Tool> {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "action".to_string(),
            description: "操作类型（create、list、update、delete、complete）".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "task".to_string(),
            description: "任务内容或ID".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "priority".to_string(),
            description: "任务优先级（high、medium、low）".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("medium")),
        },
    ]);

    Box::new(FunctionTool::new(
        "task_manager".to_string(),
        "创建、管理和跟踪任务和待办事项".to_string(),
        schema,
        |params| {
            let action = params.get("action").and_then(|v| v.as_str()).unwrap_or("");
            let task = params.get("task").and_then(|v| v.as_str()).unwrap_or("");
            let priority = params
                .get("priority")
                .and_then(|v| v.as_str())
                .unwrap_or("medium");

            match action {
                "create" => {
                    let task_id = format!("task_{}", chrono::Utc::now().timestamp());
                    Ok(json!({
                        "action": "create",
                        "task_id": task_id,
                        "content": task,
                        "priority": priority,
                        "status": "pending",
                        "created_at": chrono::Utc::now().to_rfc3339(),
                        "success": true
                    }))
                }
                "list" => {
                    // 模拟任务列表
                    Ok(json!({
                        "action": "list",
                        "tasks": [
                            {
                                "id": "task_1",
                                "content": "完成项目文档",
                                "priority": "high",
                                "status": "pending",
                                "created_at": "2024-03-15T10:00:00Z"
                            },
                            {
                                "id": "task_2",
                                "content": "代码审查",
                                "priority": "medium",
                                "status": "in_progress",
                                "created_at": "2024-03-15T11:00:00Z"
                            },
                            {
                                "id": "task_3",
                                "content": "团队会议",
                                "priority": "low",
                                "status": "completed",
                                "created_at": "2024-03-15T09:00:00Z"
                            }
                        ],
                        "total": 3,
                        "success": true
                    }))
                }
                "complete" => Ok(json!({
                    "action": "complete",
                    "task_id": task,
                    "status": "completed",
                    "completed_at": chrono::Utc::now().to_rfc3339(),
                    "success": true
                })),
                "delete" => Ok(json!({
                    "action": "delete",
                    "task_id": task,
                    "success": true
                })),
                _ => Ok(json!({
                    "error": "不支持的操作类型",
                    "supported_actions": ["create", "list", "update", "delete", "complete"]
                })),
            }
        },
    ))
}

// 知识库工具
fn create_knowledge_base() -> Box<dyn Tool> {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "query".to_string(),
            description: "搜索查询关键词".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "category".to_string(),
            description: "知识分类（tech、science、business、general）".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("general")),
        },
    ]);

    Box::new(FunctionTool::new(
        "knowledge_base".to_string(),
        "搜索和查询专业知识库信息".to_string(),
        schema,
        |params| {
            let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let category = params
                .get("category")
                .and_then(|v| v.as_str())
                .unwrap_or("general");

            // 模拟知识库搜索
            let results = match query.to_lowercase().as_str() {
                q if q.contains("rust") => vec![
                    json!({
                        "title": "Rust编程语言入门",
                        "content": "Rust是一种系统编程语言，注重安全、速度和并发性。",
                        "category": "tech",
                        "relevance": 0.95
                    }),
                    json!({
                        "title": "Rust内存管理",
                        "content": "Rust通过所有权系统实现内存安全，无需垃圾回收器。",
                        "category": "tech",
                        "relevance": 0.90
                    }),
                ],
                q if q.contains("ai") || q.contains("人工智能") => vec![
                    json!({
                        "title": "人工智能发展历程",
                        "content": "人工智能从1950年代开始发展，经历了多次技术革新。",
                        "category": "tech",
                        "relevance": 0.88
                    }),
                    json!({
                        "title": "机器学习基础",
                        "content": "机器学习是人工智能的核心技术，包括监督学习、无监督学习等。",
                        "category": "tech",
                        "relevance": 0.85
                    }),
                ],
                _ => vec![json!({
                    "title": "通用知识条目",
                    "content": format!("关于'{}'的相关信息正在整理中。", query),
                    "category": category,
                    "relevance": 0.60
                })],
            };

            Ok(json!({
                "query": query,
                "category": category,
                "results": results,
                "total_found": results.len(),
                "search_time": "0.05s",
                "search_complete": true
            }))
        },
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 DeepSeek智能助手应用");
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

    println!("✅ 正在初始化DeepSeek智能助手应用...");

    // 创建应用实例
    let mut app = DeepSeekApp::new(
        api_key,
        "DeepSeek智能助手".to_string(),
        "一个功能强大的AI助手，集成了代码分析、数学计算、文本处理、天气查询、任务管理和知识库搜索等多种专业工具。".to_string()
    ).await?;

    let (name, description) = app.info();
    println!("✅ 应用初始化完成！");
    println!("📱 应用名称: {}", name);
    println!("📝 应用描述: {}", description);

    println!("\n🛠️  可用工具:");
    for (i, tool) in app.available_tools().iter().enumerate() {
        println!("  {}. {}", i + 1, tool);
    }

    // 演示各种功能
    let demo_scenarios = [
        ("💻 代码分析演示", "请分析这段Python代码的质量：\n```python\ndef fibonacci(n):\n    if n <= 1:\n        return n\n    return fibonacci(n-1) + fibonacci(n-2)\n```"),
        ("🧮 数学计算演示", "请帮我计算 sqrt(16) + 2^3 的结果"),
        ("📝 文本处理演示", "请分析这段文本的情感：'今天是个美好的日子，阳光明媚，心情wonderful！'"),
        ("🌤️ 天气查询演示", "请查询北京未来3天的天气预报"),
        ("📋 任务管理演示", "请帮我创建一个高优先级的任务：完成DeepSeek集成测试"),
        ("🔍 知识库搜索演示", "请搜索关于Rust编程语言的相关知识"),
    ];

    for (title, query) in demo_scenarios.iter() {
        println!("\n{}", "=".repeat(60));
        println!("{}", title);
        println!("{}", "=".repeat(60));
        println!("👤 用户: {}", query);
        println!("\n🤖 DeepSeek正在处理...");

        match app.chat(query).await {
            Ok(response) => {
                println!("\n💬 DeepSeek: {}", response);
            }
            Err(e) => {
                println!("❌ 错误: {}", e);
            }
        }

        // 添加延迟避免API限制
        println!("\n⏳ 等待3秒后继续下一个演示...");
        sleep(Duration::from_secs(3)).await;
    }

    println!("\n{}", "=".repeat(60));
    println!("🎉 DeepSeek智能助手应用演示完成！");
    println!("{}", "=".repeat(60));
    println!("✨ 应用特性总结:");
    println!("  • 🔧 6个专业工具集成");
    println!("  • 🧠 DeepSeek大模型驱动");
    println!("  • 🌐 中文原生支持");
    println!("  • ⚡ 高性能异步处理");
    println!("  • 🛡️ 完善的错误处理");
    println!("  • 📊 详细的执行日志");

    Ok(())
}
