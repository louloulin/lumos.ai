//! 中国LLM提供商示例
//! 
//! 这个示例展示了如何使用智谱AI和百度ERNIE等中国LLM提供商
//! 
//! 运行前需要设置环境变量:
//! - ZHIPU_API_KEY: 智谱AI的API密钥
//! - BAIDU_API_KEY: 百度的API Key
//! - BAIDU_SECRET_KEY: 百度的Secret Key

use lumosai_core::llm::{
    providers,
    types::{LlmOptions, Message, Role},
    function_calling::{FunctionDefinition, ToolChoice},
    provider::LlmProvider,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 中国LLM提供商示例");
    println!("=" .repeat(50));

    // 测试智谱AI
    if let Ok(zhipu) = providers::zhipu_from_env() {
        println!("\n📱 测试智谱AI (GLM)");
        println!("-".repeat(30));
        
        test_basic_generation(&zhipu, "智谱AI").await?;
        test_conversation(&zhipu, "智谱AI").await?;
        test_function_calling(&zhipu, "智谱AI").await?;
    } else {
        println!("⚠️  跳过智谱AI测试 - 未设置ZHIPU_API_KEY环境变量");
    }

    // 测试百度ERNIE
    if let Ok(baidu) = providers::baidu_from_env() {
        println!("\n🔵 测试百度ERNIE");
        println!("-".repeat(30));
        
        test_basic_generation(&baidu, "百度ERNIE").await?;
        test_conversation(&baidu, "百度ERNIE").await?;
        test_function_calling(&baidu, "百度ERNIE").await?;
    } else {
        println!("⚠️  跳过百度ERNIE测试 - 未设置BAIDU_API_KEY或BAIDU_SECRET_KEY环境变量");
    }

    // 演示自动provider选择
    println!("\n🤖 自动Provider选择");
    println!("-".repeat(30));
    
    match providers::auto_provider() {
        Ok(provider) => {
            println!("✅ 自动选择的provider: {}", provider.name());
            test_basic_generation(&*provider, "自动选择").await?;
        }
        Err(e) => {
            println!("❌ 无法自动选择provider: {}", e);
        }
    }

    println!("\n✨ 示例完成!");
    Ok(())
}

/// 测试基本文本生成
async fn test_basic_generation(
    provider: &dyn LlmProvider, 
    name: &str
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔤 基本文本生成 ({})", name);
    
    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(100);
    
    let prompt = "请用一句话介绍人工智能的发展历程";
    
    match provider.generate(prompt, &options).await {
        Ok(response) => {
            println!("✅ 响应: {}", response.trim());
        }
        Err(e) => {
            println!("❌ 错误: {}", e);
        }
    }
    
    Ok(())
}

/// 测试对话功能
async fn test_conversation(
    provider: &dyn LlmProvider, 
    name: &str
) -> Result<(), Box<dyn std::error::Error>> {
    println!("💬 对话测试 ({})", name);
    
    let messages = vec![
        Message {
            role: Role::System,
            content: "你是一个友好的AI助手，请用简洁的中文回答问题。".to_string(),
            name: None,
            metadata: None,
        },
        Message {
            role: Role::User,
            content: "什么是机器学习？".to_string(),
            name: None,
            metadata: None,
        },
    ];
    
    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(150);
    
    match provider.generate_with_messages(&messages, &options).await {
        Ok(response) => {
            println!("✅ 对话响应: {}", response.trim());
        }
        Err(e) => {
            println!("❌ 对话错误: {}", e);
        }
    }
    
    Ok(())
}

/// 测试函数调用功能
async fn test_function_calling(
    provider: &dyn LlmProvider, 
    name: &str
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 函数调用测试 ({})", name);
    
    if !provider.supports_function_calling() {
        println!("⚠️  {} 不支持函数调用", name);
        return Ok(());
    }
    
    // 定义一个简单的函数
    let functions = vec![
        FunctionDefinition {
            name: "get_weather".to_string(),
            description: "获取指定城市的天气信息".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "城市名称"
                    },
                    "unit": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"],
                        "description": "温度单位"
                    }
                },
                "required": ["city"]
            }),
        }
    ];
    
    let messages = vec![
        Message {
            role: Role::User,
            content: "北京今天的天气怎么样？".to_string(),
            name: None,
            metadata: None,
        },
    ];
    
    let options = LlmOptions::default()
        .with_temperature(0.1)
        .with_max_tokens(200);
    
    match provider.generate_with_functions(
        &messages, 
        &functions, 
        &ToolChoice::Auto, 
        &options
    ).await {
        Ok(response) => {
            if !response.function_calls.is_empty() {
                println!("✅ 函数调用: {:?}", response.function_calls);
            } else {
                println!("✅ 普通响应: {:?}", response.content);
            }
        }
        Err(e) => {
            println!("❌ 函数调用错误: {}", e);
        }
    }
    
    Ok(())
}

/// 演示流式生成 (如果支持)
#[allow(dead_code)]
async fn test_streaming(
    provider: &dyn LlmProvider, 
    name: &str
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 流式生成测试 ({})", name);
    
    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(100);
    
    let prompt = "请讲一个关于人工智能的小故事";
    
    match provider.generate_stream(prompt, &options).await {
        Ok(mut stream) => {
            use futures::StreamExt;
            
            print!("✅ 流式响应: ");
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(text) => print!("{}", text),
                    Err(e) => {
                        println!("\n❌ 流式错误: {}", e);
                        break;
                    }
                }
            }
            println!(); // 换行
        }
        Err(e) => {
            println!("❌ 流式生成错误: {}", e);
        }
    }
    
    Ok(())
}

/// 演示embedding功能
#[allow(dead_code)]
async fn test_embeddings(
    provider: &dyn LlmProvider, 
    name: &str
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔢 Embedding测试 ({})", name);
    
    let text = "人工智能是计算机科学的一个分支";
    
    match provider.get_embedding(text).await {
        Ok(embedding) => {
            println!("✅ Embedding维度: {}", embedding.len());
            println!("✅ 前5个值: {:?}", &embedding[..5.min(embedding.len())]);
        }
        Err(e) => {
            println!("❌ Embedding错误: {}", e);
        }
    }
    
    Ok(())
}
