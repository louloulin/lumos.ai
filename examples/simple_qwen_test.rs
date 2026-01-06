use lumosai_core::llm::{LlmOptions, LlmProvider, Message, QwenApiType, QwenProvider, Role};
use std::time::Instant;

/// 简单的Qwen验证测试，不依赖向量存储
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 LumosAI Qwen 简单验证测试");
    println!("========================================");

    // 测试配置
    let api_key = "sk-bc977c4e31e542f1a34159cb42478198";
    let model = "qwen3-30b-a3b";

    // 测试1: OpenAI兼容API（修复后）
    println!("\n📋 测试1: OpenAI兼容API（修复enable_thinking问题）");
    test_openai_compatible_fixed(api_key, model).await?;

    // 测试2: 基础功能验证
    println!("\n📋 测试2: 基础功能验证");
    test_basic_functionality(api_key, model).await?;

    // 测试3: 提供商信息验证
    test_provider_info();

    // 测试4: 构造函数验证
    test_constructors();

    println!("\n✅ 所有Qwen简单验证测试完成！");
    Ok(())
}

async fn test_openai_compatible_fixed(
    api_key: &str,
    model: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试修复后的OpenAI兼容API...");

    let provider = QwenProvider::new_with_api_type(
        api_key,
        model,
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    // 基础文本生成测试
    let start_time = Instant::now();
    let result = provider
        .generate("你好，请简单介绍一下你自己。", &LlmOptions::default())
        .await;
    let duration = start_time.elapsed();

    match result {
        Ok(response) => {
            println!("✅ OpenAI兼容API调用成功!");
            println!("📝 响应内容: {}", response);
            println!("⏱️ 响应时间: {:?}", duration);

            // 验证响应质量
            if response.len() > 10 {
                println!("✅ 响应长度合理: {} 字符", response.len());
            } else {
                println!("⚠️ 响应过短，可能有问题");
            }
        }
        Err(e) => {
            println!("❌ OpenAI兼容API调用失败: {}", e);
            return Err(format!("OpenAI兼容API测试失败: {}", e).into());
        }
    }

    // 英文对话测试
    let start_time = Instant::now();
    let result = provider
        .generate(
            "Hello, please introduce yourself briefly.",
            &LlmOptions::default(),
        )
        .await;
    let duration = start_time.elapsed();

    match result {
        Ok(response) => {
            println!("✅ 英文对话测试成功!");
            println!("📝 响应内容: {}", response);
            println!("⏱️ 响应时间: {:?}", duration);
        }
        Err(e) => {
            println!("❌ 英文对话测试失败: {}", e);
            return Err(format!("英文对话测试失败: {}", e).into());
        }
    }

    Ok(())
}

async fn test_basic_functionality(
    api_key: &str,
    model: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试基础功能...");

    let provider = QwenProvider::new_with_api_type(
        api_key,
        model,
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    // 多轮对话测试
    let messages = vec![
        Message {
            role: Role::System,
            content: "你是一个专业的AI助手，擅长回答技术问题。".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "什么是Rust编程语言？".to_string(),
            metadata: None,
            name: None,
        },
    ];

    let start_time = Instant::now();
    let result = provider
        .generate_with_messages(&messages, &LlmOptions::default())
        .await;
    let duration = start_time.elapsed();

    match result {
        Ok(response) => {
            println!("✅ 多轮对话测试成功!");
            println!("📝 响应内容: {}", response);
            println!("⏱️ 响应时间: {:?}", duration);

            // 检查回复质量
            let quality_keywords = ["Rust", "编程", "语言", "系统", "安全"];
            let found_keywords: Vec<&str> = quality_keywords
                .iter()
                .filter(|&&kw| response.contains(kw))
                .copied()
                .collect();

            println!(
                "📊 回复质量指标: 包含关键词 {}/{}",
                found_keywords.len(),
                quality_keywords.len()
            );
            if found_keywords.len() >= 2 {
                println!("✅ 回复质量良好");
            } else {
                println!("⚠️ 回复质量可能需要改进");
            }
        }
        Err(e) => {
            println!("❌ 多轮对话测试失败: {}", e);
            return Err(format!("多轮对话测试失败: {}", e).into());
        }
    }

    Ok(())
}

/// 验证提供商基本信息
fn test_provider_info() {
    println!("\n🧪 测试提供商基本信息...");

    let provider = QwenProvider::new_with_defaults("test-key", "qwen3-30b-a3b");

    println!("📋 提供商名称: {}", provider.name());

    if provider.name() == "qwen" {
        println!("✅ 提供商名称正确");
    } else {
        println!("❌ 提供商名称不正确");
    }
}

/// 测试不同的构造函数
fn test_constructors() {
    println!("\n🧪 测试不同的构造函数...");

    // 测试默认构造函数
    let _provider1 = QwenProvider::new_with_defaults("test-key", "qwen3-30b-a3b");
    println!("✅ new_with_defaults 构造函数正常");

    // 测试OpenAI兼容构造函数
    let _provider2 =
        QwenProvider::new_openai_compatible("test-key", "qwen3-30b-a3b", None::<String>);
    println!("✅ new_openai_compatible 构造函数正常");

    // 测试Qwen 2.5构造函数
    let _provider3 = QwenProvider::new_qwen25("test-key", "qwen3-30b-a3b");
    println!("✅ new_qwen25 构造函数正常");

    println!("✅ 所有构造函数测试通过");
}
