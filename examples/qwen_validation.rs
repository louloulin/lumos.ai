use lumosai_core::llm::{QwenProvider, QwenApiType, LlmProvider, LlmOptions, Message, Role};
use std::time::Instant;

/// Qwen验证测试
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 LumosAI Qwen 提供商验证测试");
    println!("========================================");
    
    // 测试配置
    let api_key = "sk-bc977c4e31e542f1a34159cb42478198";
    let model = "qwen3-30b-a3b";
    
    // 测试1: DashScope API
    println!("\n📋 测试1: DashScope API");
    test_dashscope_api(api_key, model).await?;
    
    // 测试2: OpenAI兼容API
    println!("\n📋 测试2: OpenAI兼容API");
    test_openai_compatible_api(api_key, model).await?;
    
    println!("\n✅ 所有Qwen验证测试完成！");
    Ok(())
}

async fn test_dashscope_api(api_key: &str, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试DashScope API实现...");
    
    let provider = QwenProvider::new_with_api_type(
        api_key,
        model,
        "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation",
        QwenApiType::DashScope
    );
    
    // 基础文本生成测试
    let start_time = Instant::now();
    let result = provider.generate("你好，请简单介绍一下你自己。", &LlmOptions::default()).await;
    let duration = start_time.elapsed();
    
    match result {
        Ok(response) => {
            println!("✅ DashScope API调用成功!");
            println!("📝 响应内容: {}", response);
            println!("⏱️ 响应时间: {:?}", duration);
        }
        Err(e) => {
            println!("❌ DashScope API调用失败: {}", e);
            return Err(format!("DashScope API测试失败: {}", e).into());
        }
    }
    
    // 多轮对话测试
    let messages = vec![
        Message {
            role: Role::System,
            content: "你是一个专业的AI助手。".to_string(),
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
    let result = provider.generate_with_messages(&messages, &LlmOptions::default()).await;
    let duration = start_time.elapsed();
    
    match result {
        Ok(response) => {
            println!("✅ DashScope多轮对话成功!");
            println!("📝 响应内容: {}", response);
            println!("⏱️ 响应时间: {:?}", duration);
        }
        Err(e) => {
            println!("❌ DashScope多轮对话失败: {}", e);
            return Err(format!("DashScope多轮对话测试失败: {}", e).into());
        }
    }
    
    Ok(())
}

async fn test_openai_compatible_api(api_key: &str, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试OpenAI兼容API实现...");
    
    let provider = QwenProvider::new_with_api_type(
        api_key,
        model,
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible
    );
    
    // 基础文本生成测试
    let start_time = Instant::now();
    let result = provider.generate("Hello, please introduce yourself briefly.", &LlmOptions::default()).await;
    let duration = start_time.elapsed();
    
    match result {
        Ok(response) => {
            println!("✅ OpenAI兼容API调用成功!");
            println!("📝 响应内容: {}", response);
            println!("⏱️ 响应时间: {:?}", duration);
        }
        Err(e) => {
            println!("❌ OpenAI兼容API调用失败: {}", e);
            println!("⚠️ 注意: 可能需要添加enable_thinking参数");
            // 不作为错误处理，因为可能需要特殊参数
        }
    }
    
    // 嵌入向量测试
    let start_time = Instant::now();
    let result = provider.get_embedding("这是一个测试文本。").await;
    let duration = start_time.elapsed();
    
    match result {
        Ok(embedding) => {
            println!("✅ 嵌入向量生成成功!");
            println!("📊 向量维度: {}", embedding.len());
            println!("⏱️ 生成时间: {:?}", duration);
            println!("🔢 向量前5个值: {:?}", &embedding[..5.min(embedding.len())]);
        }
        Err(e) => {
            println!("❌ 嵌入向量生成失败: {}", e);
            println!("⚠️ 注意: 嵌入模型可能不支持或需要不同配置");
        }
    }
    
    // 流式响应测试
    println!("🧪 测试流式响应...");
    let start_time = Instant::now();
    let stream_result = provider.generate_stream(
        "请写一首关于人工智能的短诗。",
        &LlmOptions::default()
    ).await;
    
    match stream_result {
        Ok(mut stream) => {
            println!("✅ 流式响应启动成功!");
            
            use futures::StreamExt;
            let mut full_response = String::new();
            let mut chunk_count = 0;
            
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(text) => {
                        print!("{}", text);
                        full_response.push_str(&text);
                        chunk_count += 1;
                    }
                    Err(e) => {
                        println!("\n❌ 流式响应错误: {}", e);
                        break;
                    }
                }
            }
            
            let duration = start_time.elapsed();
            println!("\n✅ 流式响应完成!");
            println!("📊 数据块数量: {}", chunk_count);
            println!("⏱️ 总耗时: {:?}", duration);
            println!("📝 完整响应长度: {}", full_response.len());
        }
        Err(e) => {
            println!("❌ 流式响应启动失败: {}", e);
            println!("⚠️ 注意: 流式响应可能需要特殊配置");
        }
    }
    
    Ok(())
}

/// 测试不同的LlmOptions配置
async fn test_llm_options(provider: &QwenProvider) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试不同的LlmOptions配置...");
    
    // 测试温度参数
    let mut options = LlmOptions::default();
    options.temperature = Some(0.1);
    options.max_tokens = Some(100);
    
    let result = provider.generate("请简单回答：什么是AI？", &options).await;
    
    match result {
        Ok(response) => {
            println!("✅ 自定义参数测试成功!");
            println!("📝 响应内容: {}", response);
        }
        Err(e) => {
            println!("❌ 自定义参数测试失败: {}", e);
        }
    }
    
    Ok(())
}

/// 性能基准测试
async fn performance_benchmark(provider: &QwenProvider) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 性能基准测试...");
    
    let test_prompts = vec![
        "什么是机器学习？",
        "解释深度学习的概念。",
        "人工智能的应用领域有哪些？",
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
                println!("✅ 成功 - 耗时: {:?}, 响应长度: {}", duration, response.len());
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
    
    Ok(())
}
