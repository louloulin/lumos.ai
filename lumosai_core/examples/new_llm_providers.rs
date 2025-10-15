//! 新LLM提供商示例
//!
//! 这个示例展示了如何使用新添加的LLM提供商：
//! - Cohere
//! - Gemini
//! - Ollama
//! - Together AI

use lumosai_core::llm::{
    cohere::CohereProvider, gemini::GeminiProvider, ollama::OllamaProvider,
    together::TogetherProvider, LlmOptions, LlmProvider, Message, Role,
};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚀 LumosAI 新LLM提供商示例");
    println!("========================================");

    // 测试Cohere提供商
    test_cohere_provider().await?;

    // 测试Gemini提供商
    test_gemini_provider().await?;

    // 测试Ollama提供商
    test_ollama_provider().await?;

    // 测试Together提供商
    test_together_provider().await?;

    println!("\n✅ 所有新LLM提供商测试完成！");
    Ok(())
}

async fn test_cohere_provider() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n🔵 测试 Cohere 提供商");
    println!("----------------------------------------");

    // 创建Cohere提供商
    let provider = CohereProvider::new("test-api-key".to_string(), "command-r-plus".to_string());

    println!("✓ 提供商名称: {}", provider.name());
    println!("✓ 支持函数调用: {}", provider.supports_function_calling());

    // 创建测试消息
    let messages = vec![
        Message {
            role: Role::System,
            content: "你是一个有用的AI助手。".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "你好！".to_string(),
            metadata: None,
            name: None,
        },
    ];

    // 创建选项
    let options = LlmOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(100);

    println!("✓ 消息数量: {}", messages.len());
    println!("✓ 温度: {:?}", options.temperature);
    println!("✓ 最大令牌: {:?}", options.max_tokens);

    // 注意：实际的API调用需要有效的API密钥
    println!("⚠️  实际API调用需要有效的COHERE_API_KEY环境变量");

    Ok(())
}

async fn test_gemini_provider() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n🟡 测试 Gemini 提供商");
    println!("----------------------------------------");

    // 创建Gemini提供商
    let provider = GeminiProvider::new("test-api-key".to_string(), "gemini-1.5-pro".to_string());

    println!("✓ 提供商名称: {}", provider.name());
    println!("✓ 支持函数调用: {}", provider.supports_function_calling());

    // 创建测试消息
    let messages = vec![
        Message {
            role: Role::System,
            content: "你是一个有用的AI助手。".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "解释一下量子计算的基本原理。".to_string(),
            metadata: None,
            name: None,
        },
    ];

    // 创建选项
    let options = LlmOptions::default()
        .with_temperature(0.8)
        .with_max_tokens(200);

    println!("✓ 消息数量: {}", messages.len());
    println!("✓ 温度: {:?}", options.temperature);
    println!("✓ 最大令牌: {:?}", options.max_tokens);

    // 注意：实际的API调用需要有效的API密钥
    println!("⚠️  实际API调用需要有效的GEMINI_API_KEY环境变量");

    Ok(())
}

async fn test_ollama_provider() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n🟢 测试 Ollama 提供商");
    println!("----------------------------------------");

    // 创建Ollama提供商（本地主机）
    let provider = OllamaProvider::localhost("llama2".to_string());

    println!("✓ 提供商名称: {}", provider.name());
    println!("✓ 支持函数调用: {}", provider.supports_function_calling());

    // 创建测试消息
    let messages = vec![
        Message {
            role: Role::System,
            content: "你是一个有用的AI助手。".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "写一首关于编程的短诗。".to_string(),
            metadata: None,
            name: None,
        },
    ];

    // 创建选项
    let options = LlmOptions::default()
        .with_temperature(0.9)
        .with_max_tokens(150);

    println!("✓ 消息数量: {}", messages.len());
    println!("✓ 温度: {:?}", options.temperature);
    println!("✓ 最大令牌: {:?}", options.max_tokens);

    // 注意：实际的API调用需要本地运行Ollama服务
    println!("⚠️  实际API调用需要本地运行Ollama服务");

    Ok(())
}

async fn test_together_provider() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n🟣 测试 Together AI 提供商");
    println!("----------------------------------------");

    // 创建Together提供商
    let provider = TogetherProvider::new(
        "test-api-key".to_string(),
        "meta-llama/Llama-2-7b-chat-hf".to_string(),
    );

    println!("✓ 提供商名称: {}", provider.name());
    println!("✓ 支持函数调用: {}", provider.supports_function_calling());

    // 创建测试消息
    let messages = vec![
        Message {
            role: Role::System,
            content: "你是一个有用的AI助手。".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "解释一下机器学习和深度学习的区别。".to_string(),
            metadata: None,
            name: None,
        },
    ];

    // 创建选项
    let options = LlmOptions::default()
        .with_temperature(0.6)
        .with_max_tokens(300);

    println!("✓ 消息数量: {}", messages.len());
    println!("✓ 温度: {:?}", options.temperature);
    println!("✓ 最大令牌: {:?}", options.max_tokens);

    // 注意：实际的API调用需要有效的API密钥
    println!("⚠️  实际API调用需要有效的TOGETHER_API_KEY环境变量");

    Ok(())
}

/// 演示如何使用trait对象来统一处理不同的提供商
#[allow(dead_code)]
async fn demo_provider_trait_usage() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 演示提供商trait统一使用");
    println!("----------------------------------------");

    // 创建不同的提供商
    let providers: Vec<Box<dyn LlmProvider>> = vec![
        Box::new(CohereProvider::new(
            "test".to_string(),
            "command-r".to_string(),
        )),
        Box::new(GeminiProvider::new(
            "test".to_string(),
            "gemini-pro".to_string(),
        )),
        Box::new(OllamaProvider::localhost("llama2".to_string())),
        Box::new(TogetherProvider::new(
            "test".to_string(),
            "meta-llama/Llama-2-7b-chat-hf".to_string(),
        )),
    ];

    // 统一处理所有提供商
    for (i, provider) in providers.iter().enumerate() {
        println!("提供商 {}: {}", i + 1, provider.name());
        println!("  支持函数调用: {}", provider.supports_function_calling());
    }

    Ok(())
}

/// 演示如何从环境变量创建提供商
#[allow(dead_code)]
async fn demo_env_provider_creation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n🌍 演示从环境变量创建提供商");
    println!("----------------------------------------");

    // 尝试从环境变量创建Cohere提供商
    match CohereProvider::from_env() {
        Ok(provider) => {
            println!("✓ 成功从环境变量创建Cohere提供商: {}", provider.name());
        }
        Err(e) => {
            println!("⚠️  无法从环境变量创建Cohere提供商: {}", e);
        }
    }

    // 尝试从环境变量创建Gemini提供商
    match GeminiProvider::from_env() {
        Ok(provider) => {
            println!("✓ 成功从环境变量创建Gemini提供商: {}", provider.name());
        }
        Err(e) => {
            println!("⚠️  无法从环境变量创建Gemini提供商: {}", e);
        }
    }

    // Ollama提供商（应该总是成功，因为有默认值）
    let ollama_provider = OllamaProvider::from_env();
    println!("✓ 成功创建Ollama提供商: {}", ollama_provider.name());

    // 尝试从环境变量创建Together提供商
    match TogetherProvider::from_env() {
        Ok(provider) => {
            println!("✓ 成功从环境变量创建Together提供商: {}", provider.name());
        }
        Err(e) => {
            println!("⚠️  无法从环境变量创建Together提供商: {}", e);
        }
    }

    Ok(())
}
