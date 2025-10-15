use lumosai_core::llm::{LlmOptions, LlmProvider, Message, QwenApiType, QwenProvider, Role};
use std::time::Instant;

/// Qwen API配置
const QWEN_API_KEY: &str = "sk-bc977c4e31e542f1a34159cb42478198";
const QWEN_MODEL: &str = "qwen3-30b-a3b";
const QWEN_BASE_URL: &str = "https://dashscope.aliyuncs.com/compatible-mode/v1";

/// 创建Qwen提供商实例
fn create_qwen_provider() -> QwenProvider {
    QwenProvider::new_with_api_type(
        QWEN_API_KEY,
        QWEN_MODEL,
        QWEN_BASE_URL,
        QwenApiType::DashScope,
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 LumosAI Qwen 提供商验证测试");
    println!("========================================");

    // 测试1: 基础连接测试
    test_basic_connection().await?;

    // 测试2: 中文对话测试
    test_chinese_conversation().await?;

    // 测试3: 英文对话测试
    test_english_conversation().await?;

    // 测试4: 嵌入向量测试
    test_embedding().await?;

    // 测试5: 性能基准测试
    test_performance_benchmark().await?;

    println!("\n✅ 所有Qwen验证测试完成！");
    Ok(())
}

async fn test_basic_connection() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 测试1: Qwen 基础连接...");

    let provider = create_qwen_provider();

    let start_time = Instant::now();
    let result = provider
        .generate("你好，请简单介绍一下你自己。", &LlmOptions::default())
        .await;
    let duration = start_time.elapsed();

    match result {
        Ok(response) => {
            println!("✅ Qwen 连接成功!");
            println!("📝 响应内容: {}", response);
            println!("⏱️ 响应时间: {:?}", duration);

            if response.is_empty() {
                return Err("响应内容为空".into());
            }

            if duration.as_secs() > 30 {
                return Err("响应时间过长".into());
            }
        }
        Err(e) => {
            println!("❌ Qwen 连接失败: {}", e);
            return Err(format!("Qwen 基础连接测试失败: {}", e).into());
        }
    }

    Ok(())
}

async fn test_chinese_conversation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 测试2: Qwen 中文对话能力...");

    let provider = create_qwen_provider();

    let messages = vec![
        Message {
            role: Role::System,
            content: "你是一个专业的AI助手，擅长回答技术问题。".to_string(),
            metadata: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "请解释一下什么是Rust编程语言的所有权系统？".to_string(),
            metadata: None,
            name: None,
        },
    ];

    let result = provider
        .generate_with_messages(&messages, &LlmOptions::default())
        .await;

    match result {
        Ok(response) => {
            println!("✅ 中文对话测试成功!");
            println!("📝 响应内容: {}", response);

            if response.is_empty() {
                return Err("响应内容为空".into());
            }

            if !response.contains("所有权")
                && !response.contains("Rust")
                && !response.contains("内存")
            {
                println!("⚠️ 警告: 响应可能不包含预期的技术内容");
            }
        }
        Err(e) => {
            println!("❌ 中文对话测试失败: {}", e);
            return Err(format!("Qwen 中文对话测试失败: {}", e).into());
        }
    }

    Ok(())
}

async fn test_english_conversation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 测试3: Qwen 英文对话能力...");

    let provider = create_qwen_provider();

    let result = provider
        .generate(
            "Explain the concept of artificial intelligence in simple terms.",
            &LlmOptions::default(),
        )
        .await;

    match result {
        Ok(response) => {
            println!("✅ 英文对话测试成功!");
            println!("📝 响应内容: {}", response);

            if response.is_empty() {
                return Err("响应内容为空".into());
            }

            let response_lower = response.to_lowercase();
            if !response_lower.contains("artificial")
                && !response_lower.contains("intelligence")
                && !response_lower.contains("ai")
            {
                println!("⚠️ 警告: 响应可能不包含预期的AI相关内容");
            }
        }
        Err(e) => {
            println!("❌ 英文对话测试失败: {}", e);
            return Err(format!("Qwen 英文对话测试失败: {}", e).into());
        }
    }

    Ok(())
}

async fn test_embedding() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 测试4: Qwen 嵌入向量生成...");

    let provider = create_qwen_provider();

    let test_text = "这是一个测试文本，用于生成嵌入向量。";

    let start_time = Instant::now();
    let result = provider.get_embedding(test_text).await;
    let duration = start_time.elapsed();

    match result {
        Ok(embedding) => {
            println!("✅ 嵌入向量生成成功!");
            println!("📊 向量维度: {}", embedding.len());
            println!("⏱️ 生成时间: {:?}", duration);
            println!("🔢 向量前5个值: {:?}", &embedding[..5.min(embedding.len())]);

            if embedding.is_empty() {
                return Err("嵌入向量为空".into());
            }

            if embedding.len() < 100 {
                return Err("向量维度过小".into());
            }

            if duration.as_secs() > 10 {
                return Err("嵌入生成时间过长".into());
            }

            // 检查向量值的合理性
            let has_non_zero = embedding.iter().any(|&x| x != 0.0);
            if !has_non_zero {
                return Err("向量全为零值".into());
            }
        }
        Err(e) => {
            println!("❌ 嵌入向量生成失败: {}", e);
            return Err(format!("Qwen 嵌入向量测试失败: {}", e).into());
        }
    }

    Ok(())
}

async fn test_performance_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 测试5: Qwen 性能基准...");

    let provider = create_qwen_provider();
    let test_prompts = vec![
        "什么是机器学习？",
        "解释深度学习的概念。",
        "人工智能的应用领域有哪些？",
        "神经网络是如何工作的？",
        "自然语言处理的主要任务是什么？",
    ];

    let mut total_time = std::time::Duration::new(0, 0);
    let mut success_count = 0;

    for (i, prompt) in test_prompts.iter().enumerate() {
        println!("📝 测试提示 {}: {}", i + 1, prompt);

        let start_time = Instant::now();
        let result = provider.generate(prompt, &LlmOptions::default()).await;
        let duration = start_time.elapsed();

        match result {
            Ok(response) => {
                success_count += 1;
                total_time += duration;
                println!(
                    "✅ 成功 - 耗时: {:?}, 响应长度: {}",
                    duration,
                    response.len()
                );
            }
            Err(e) => {
                println!("❌ 失败: {}", e);
            }
        }

        // 避免请求过于频繁
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    let avg_time = if success_count > 0 {
        total_time / success_count
    } else {
        std::time::Duration::new(0, 0)
    };
    let success_rate = (success_count as f64 / test_prompts.len() as f64) * 100.0;

    println!("\n📊 性能基准测试结果:");
    println!("- 总测试数: {}", test_prompts.len());
    println!("- 成功数: {}", success_count);
    println!("- 成功率: {:.1}%", success_rate);
    println!("- 平均响应时间: {:?}", avg_time);
    println!("- 总耗时: {:?}", total_time);

    if success_rate < 80.0 {
        return Err(format!("成功率过低: {:.1}%", success_rate).into());
    }

    if avg_time.as_secs() > 10 {
        return Err(format!("平均响应时间过长: {:?}", avg_time).into());
    }

    Ok(())
}
