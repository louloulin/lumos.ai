use lumosai_core::Result;
use lumosai_core::llm::{DeepSeekProvider, Message, Role, LlmOptions};
use lumosai_core::llm::provider::LlmProvider;
use lumosai_core::llm::function_calling::{FunctionDefinition, ToolChoice};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("DeepSeek集成测试示例");
    
    // 注意：需要设置环境变量 DEEPSEEK_API_KEY
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .unwrap_or_else(|_| "demo-key".to_string());
    
    // 创建DeepSeek提供者
    let deepseek = DeepSeekProvider::new(api_key, None);
    
    println!("\n测试基本生成功能:");
    let options = LlmOptions::default();
    
    // 如果有真实的API key，可以进行实际测试
    if std::env::var("DEEPSEEK_API_KEY").is_ok() {
        match deepseek.generate("你好，请介绍一下自己", &options).await {
            Ok(response) => println!("DeepSeek响应: {}", response),
            Err(e) => println!("调用失败: {}", e),
        }
    } else {
        println!("未设置DEEPSEEK_API_KEY环境变量，跳过实际API调用");
    }
    
    println!("\n测试消息格式生成:");
    let messages = vec![
        Message {
            role: Role::System,
            content: "你是一个有用的助手".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "什么是人工智能？".to_string(),
            metadata: None,
            name: None,
        },
    ];
    
    if std::env::var("DEEPSEEK_API_KEY").is_ok() {
        match deepseek.generate_with_messages(&messages, &options).await {
            Ok(response) => println!("DeepSeek响应: {}", response),
            Err(e) => println!("调用失败: {}", e),
        }
    } else {
        println!("未设置DEEPSEEK_API_KEY环境变量，跳过实际API调用");
    }
    
    println!("\n测试Function Calling支持:");
    
    // 定义一个简单的function
    let get_weather_function = FunctionDefinition {
        name: "get_weather".to_string(),
        description: Some("获取指定城市的天气信息".to_string()),
        parameters: json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "城市名称"
                }
            },
            "required": ["city"]
        }),
    };
    
    let functions = vec![get_weather_function];
    let tool_choice = ToolChoice::Auto;
    
    let function_messages = vec![
        Message {
            role: Role::User,
            content: "北京今天天气怎么样？".to_string(),
            metadata: None,
            name: None,
        },
    ];
    
    if std::env::var("DEEPSEEK_API_KEY").is_ok() {
        match deepseek.generate_with_functions(&function_messages, &functions, &tool_choice, &options).await {
            Ok(response) => {
                println!("DeepSeek Function Calling响应:");
                if let Some(content) = response.content {
                    println!("内容: {}", content);
                }
                if !response.function_calls.is_empty() {
                    println!("函数调用:");
                    for call in response.function_calls {
                        println!("  - 函数: {}", call.name);
                        println!("    参数: {}", call.arguments);
                    }
                }
            },
            Err(e) => println!("Function calling调用失败: {}", e),
        }
    } else {
        println!("未设置DEEPSEEK_API_KEY环境变量，跳过Function Calling测试");
    }
    
    println!("\n测试Embedding（预期失败）:");
    match deepseek.get_embedding("测试文本").await {
        Ok(_) => println!("意外成功！"),
        Err(e) => println!("预期的错误: {}", e),
    }
    
    println!("\n测试功能特性:");
    println!("支持Function Calling: {}", deepseek.supports_function_calling());
    
    println!("\nDeepSeek集成测试完成");
    Ok(())
}
