//! 自定义工具开发示例 - 展示如何创建和使用自定义工具
//! 
//! 这个示例展示了多种创建自定义工具的方法，从简单的函数工具到复杂的异步工具。
//! 
//! 运行方式:
//! ```bash
//! cargo run --example custom_tools
//! ```

use lumosai_core::prelude::*;
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions, FunctionTool, ToolSchema, ParameterSchema};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 LumosAI 自定义工具开发示例");
    println!("==============================");
    
    // 创建LLM提供者
    let llm = Arc::new(MockLlmProvider::new(vec![
        "我已经使用天气工具获取了天气信息。".to_string(),
        "我已经使用翻译工具完成了翻译。".to_string(),
        "我已经使用数据库工具查询了数据。".to_string(),
        "我已经使用文件处理工具完成了操作。".to_string(),
    ]));
    
    // 1. 简单函数工具
    println!("\n1️⃣ 简单函数工具");
    println!("----------------");
    
    let weather_tool = create_weather_tool();
    let simple_agent = quick_agent("simple_assistant", "你是一个天气助手")
        .model(llm.clone())
        .tools(vec![weather_tool])
        .build()?;
    
    println!("🌤️ 天气助手创建成功，可用工具:");
    for tool in simple_agent.get_tools() {
        println!("   - {}: {}", tool.name(), tool.description());
    }
    
    let weather_response = simple_agent.generate("请查询北京的天气").await?;
    println!("🤖 天气助手: {}", weather_response.content);
    
    // 2. 结构化参数工具
    println!("\n2️⃣ 结构化参数工具");
    println!("------------------");
    
    let translator_tool = create_translator_tool();
    let translator_agent = quick_agent("translator", "你是一个翻译助手")
        .model(llm.clone())
        .tools(vec![translator_tool])
        .build()?;
    
    println!("🌍 翻译助手创建成功，可用工具:");
    for tool in translator_agent.get_tools() {
        println!("   - {}: {}", tool.name(), tool.description());
    }
    
    let translate_response = translator_agent.generate("请将'Hello World'翻译成中文").await?;
    println!("🤖 翻译助手: {}", translate_response.content);
    
    // 3. 异步工具
    println!("\n3️⃣ 异步工具");
    println!("------------");
    
    let database_tool = create_database_tool();
    let db_agent = quick_agent("database_assistant", "你是一个数据库助手")
        .model(llm.clone())
        .tools(vec![database_tool])
        .build()?;
    
    println!("🗄️ 数据库助手创建成功，可用工具:");
    for tool in db_agent.get_tools() {
        println!("   - {}: {}", tool.name(), tool.description());
    }
    
    let db_response = db_agent.generate("请查询用户表中的所有数据").await?;
    println!("🤖 数据库助手: {}", db_response.content);
    
    // 4. 复杂状态工具
    println!("\n4️⃣ 复杂状态工具");
    println!("----------------");
    
    let file_processor = Arc::new(FileProcessor::new());
    let file_tool = Box::new(file_processor.clone()) as Box<dyn Tool>;
    
    let file_agent = quick_agent("file_processor", "你是一个文件处理助手")
        .model(llm.clone())
        .tools(vec![file_tool])
        .build()?;
    
    println!("📄 文件处理助手创建成功，可用工具:");
    for tool in file_agent.get_tools() {
        println!("   - {}: {}", tool.name(), tool.description());
    }
    
    let file_response = file_agent.generate("请处理文件 'example.txt'").await?;
    println!("🤖 文件处理助手: {}", file_response.content);
    
    // 5. 组合工具演示
    println!("\n5️⃣ 组合工具演示");
    println!("----------------");
    
    let multi_tool_agent = quick_agent("multi_tool_assistant", "你是一个多功能助手")
        .model(llm.clone())
        .tools(vec![
            create_weather_tool(),
            create_translator_tool(),
            create_database_tool(),
        ])
        .build()?;
    
    println!("🎯 多功能助手创建成功，工具数量: {}", multi_tool_agent.get_tools().len());
    
    // 6. 工具性能测试
    println!("\n6️⃣ 工具性能测试");
    println!("------------------");
    
    let start = std::time::Instant::now();
    
    // 测试工具创建性能
    let mut tools = Vec::new();
    for i in 0..100 {
        let tool = create_simple_tool(&format!("tool_{}", i));
        tools.push(tool);
    }
    
    let creation_duration = start.elapsed();
    println!("⏱️ 创建100个工具耗时: {:?}", creation_duration);
    
    // 测试工具执行性能
    let test_tool = create_weather_tool();
    let context = ToolExecutionContext::new(
        json!({"city": "北京"}),
        ToolExecutionOptions::default(),
    );
    
    let exec_start = std::time::Instant::now();
    let _result = test_tool.execute(context).await?;
    let exec_duration = exec_start.elapsed();
    println!("⏱️ 工具执行耗时: {:?}", exec_duration);
    
    println!("\n🎉 自定义工具开发示例完成!");
    println!("\n📚 下一步学习:");
    println!("   - examples/02_intermediate/workflows.rs - 学习工作流");
    println!("   - examples/03_advanced/complex_tools.rs - 学习复杂工具");
    println!("   - docs/best-practices/tool-development.md - 工具开发最佳实践");
    
    Ok(())
}

// 1. 简单天气工具
fn create_weather_tool() -> Box<dyn Tool> {
    let schema = ToolSchema {
        name: "get_weather".to_string(),
        description: "获取指定城市的天气信息".to_string(),
        parameters: ParameterSchema {
            type_: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("city".to_string(), json!({
                    "type": "string",
                    "description": "城市名称"
                }));
                props
            },
            required: vec!["city".to_string()],
        },
    };
    
    Box::new(FunctionTool::new(
        "get_weather".to_string(),
        "获取天气信息".to_string(),
        schema,
        Box::new(|params| Box::pin(async move {
            let city = params.get("city")
                .and_then(|v| v.as_str())
                .unwrap_or("未知城市");
            
            Ok(json!({
                "city": city,
                "temperature": "22°C",
                "condition": "晴天",
                "humidity": "65%",
                "wind": "微风"
            }))
        }))
    ))
}

// 2. 结构化参数翻译工具
#[derive(Serialize, Deserialize)]
struct TranslateRequest {
    text: String,
    from_lang: String,
    to_lang: String,
}

#[derive(Serialize, Deserialize)]
struct TranslateResponse {
    original_text: String,
    translated_text: String,
    from_language: String,
    to_language: String,
}

fn create_translator_tool() -> Box<dyn Tool> {
    let schema = ToolSchema {
        name: "translate".to_string(),
        description: "翻译文本到指定语言".to_string(),
        parameters: ParameterSchema {
            type_: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("text".to_string(), json!({
                    "type": "string",
                    "description": "要翻译的文本"
                }));
                props.insert("from_lang".to_string(), json!({
                    "type": "string",
                    "description": "源语言",
                    "default": "auto"
                }));
                props.insert("to_lang".to_string(), json!({
                    "type": "string",
                    "description": "目标语言"
                }));
                props
            },
            required: vec!["text".to_string(), "to_lang".to_string()],
        },
    };
    
    Box::new(FunctionTool::new(
        "translate".to_string(),
        "文本翻译工具".to_string(),
        schema,
        Box::new(|params| Box::pin(async move {
            let text = params.get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let from_lang = params.get("from_lang")
                .and_then(|v| v.as_str())
                .unwrap_or("auto");
            let to_lang = params.get("to_lang")
                .and_then(|v| v.as_str())
                .unwrap_or("zh");
            
            // 模拟翻译逻辑
            let translated = match text {
                "Hello World" => "你好世界",
                "Good morning" => "早上好",
                "Thank you" => "谢谢",
                _ => "翻译结果",
            };
            
            let response = TranslateResponse {
                original_text: text.to_string(),
                translated_text: translated.to_string(),
                from_language: from_lang.to_string(),
                to_language: to_lang.to_string(),
            };
            
            Ok(serde_json::to_value(response)?)
        }))
    ))
}

// 3. 异步数据库工具
fn create_database_tool() -> Box<dyn Tool> {
    let schema = ToolSchema {
        name: "query_database".to_string(),
        description: "查询数据库".to_string(),
        parameters: ParameterSchema {
            type_: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("table".to_string(), json!({
                    "type": "string",
                    "description": "表名"
                }));
                props.insert("query".to_string(), json!({
                    "type": "string",
                    "description": "查询条件",
                    "default": "*"
                }));
                props
            },
            required: vec!["table".to_string()],
        },
    };
    
    Box::new(FunctionTool::new(
        "query_database".to_string(),
        "数据库查询工具".to_string(),
        schema,
        Box::new(|params| Box::pin(async move {
            let table = params.get("table")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let query = params.get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("*");
            
            // 模拟异步数据库查询
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            
            let mock_data = match table {
                "users" => json!([
                    {"id": 1, "name": "张三", "age": 25},
                    {"id": 2, "name": "李四", "age": 30}
                ]),
                "products" => json!([
                    {"id": 1, "name": "商品A", "price": 100},
                    {"id": 2, "name": "商品B", "price": 200}
                ]),
                _ => json!([])
            };
            
            Ok(json!({
                "table": table,
                "query": query,
                "results": mock_data,
                "count": mock_data.as_array().map(|a| a.len()).unwrap_or(0)
            }))
        }))
    ))
}

// 4. 复杂状态工具 - 文件处理器
#[derive(Clone)]
struct FileProcessor {
    processed_files: Arc<std::sync::Mutex<Vec<String>>>,
}

impl FileProcessor {
    fn new() -> Self {
        Self {
            processed_files: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl Tool for FileProcessor {
    fn name(&self) -> &str {
        "file_processor"
    }
    
    fn description(&self) -> &str {
        "处理文件并记录处理历史"
    }
    
    fn parameters(&self) -> ToolSchema {
        ToolSchema {
            name: "file_processor".to_string(),
            description: "文件处理工具".to_string(),
            parameters: ParameterSchema {
                type_: "object".to_string(),
                properties: {
                    let mut props = HashMap::new();
                    props.insert("filename".to_string(), json!({
                        "type": "string",
                        "description": "文件名"
                    }));
                    props.insert("operation".to_string(), json!({
                        "type": "string",
                        "description": "操作类型",
                        "enum": ["read", "write", "delete", "process"]
                    }));
                    props
                },
                required: vec!["filename".to_string(), "operation".to_string()],
            },
        }
    }
    
    async fn execute(&self, context: ToolExecutionContext) -> Result<Value> {
        let params = context.parameters;
        let filename = params.get("filename")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown.txt");
        let operation = params.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("read");
        
        // 记录处理历史
        {
            let mut files = self.processed_files.lock().unwrap();
            files.push(format!("{}:{}", filename, operation));
        }
        
        // 模拟文件处理
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        
        let history = {
            let files = self.processed_files.lock().unwrap();
            files.clone()
        };
        
        Ok(json!({
            "filename": filename,
            "operation": operation,
            "status": "success",
            "processed_files_count": history.len(),
            "history": history
        }))
    }
}

// 5. 简单工具创建辅助函数
fn create_simple_tool(name: &str) -> Box<dyn Tool> {
    let schema = ToolSchema {
        name: name.to_string(),
        description: format!("简单工具: {}", name),
        parameters: ParameterSchema {
            type_: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        },
    };
    
    let tool_name = name.to_string();
    Box::new(FunctionTool::new(
        name.to_string(),
        format!("简单工具: {}", name),
        schema,
        Box::new(move |_params| {
            let name = tool_name.clone();
            Box::pin(async move {
                Ok(json!({
                    "tool": name,
                    "result": "success",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            })
        })
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_custom_tools_example() {
        let result = main().await;
        assert!(result.is_ok(), "自定义工具示例应该成功运行");
    }
    
    #[tokio::test]
    async fn test_weather_tool() {
        let tool = create_weather_tool();
        let context = ToolExecutionContext::new(
            json!({"city": "北京"}),
            ToolExecutionOptions::default(),
        );
        
        let result = tool.execute(context).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["city"], "北京");
        assert!(response["temperature"].is_string());
    }
    
    #[tokio::test]
    async fn test_translator_tool() {
        let tool = create_translator_tool();
        let context = ToolExecutionContext::new(
            json!({
                "text": "Hello World",
                "from_lang": "en",
                "to_lang": "zh"
            }),
            ToolExecutionOptions::default(),
        );
        
        let result = tool.execute(context).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["original_text"], "Hello World");
        assert_eq!(response["translated_text"], "你好世界");
    }
    
    #[tokio::test]
    async fn test_file_processor() {
        let processor = FileProcessor::new();
        let context = ToolExecutionContext::new(
            json!({
                "filename": "test.txt",
                "operation": "read"
            }),
            ToolExecutionOptions::default(),
        );
        
        let result = processor.execute(context).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["filename"], "test.txt");
        assert_eq!(response["operation"], "read");
        assert_eq!(response["status"], "success");
    }
}
