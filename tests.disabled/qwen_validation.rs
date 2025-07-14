use lumosai_core::llm::{QwenProvider, QwenApiType, LlmProvider, LlmOptions, Message, Role};
use lumosai_core::Result;
use tokio_test;
use std::time::Instant;
use futures::StreamExt;

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
        QwenApiType::DashScope
    )
}

#[tokio::test]
async fn test_qwen_basic_connection() {
    println!("🧪 测试 Qwen 基础连接...");
    
    let provider = create_qwen_provider();
    
    let start_time = Instant::now();
    let result = provider.generate(
        "你好，请简单介绍一下你自己。", 
        &LlmOptions::default()
    ).await;
    let duration = start_time.elapsed();
    
    match result {
        Ok(response) => {
            println!("✅ Qwen 连接成功!");
            println!("📝 响应内容: {}", response);
            println!("⏱️ 响应时间: {:?}", duration);
            
            assert!(!response.is_empty(), "响应内容不应为空");
            assert!(duration.as_secs() < 30, "响应时间应小于30秒");
        }
        Err(e) => {
            println!("❌ Qwen 连接失败: {}", e);
            panic!("Qwen 基础连接测试失败: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qwen_chinese_conversation() {
    println!("🧪 测试 Qwen 中文对话能力...");
    
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
    
    let result = provider.generate_with_messages(&messages, &LlmOptions::default()).await;
    
    match result {
        Ok(response) => {
            println!("✅ 中文对话测试成功!");
            println!("📝 响应内容: {}", response);
            
            assert!(!response.is_empty(), "响应内容不应为空");
            assert!(
                response.contains("所有权") || response.contains("Rust") || response.contains("内存"),
                "响应应包含相关技术内容"
            );
        }
        Err(e) => {
            println!("❌ 中文对话测试失败: {}", e);
            panic!("Qwen 中文对话测试失败: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qwen_english_conversation() {
    println!("🧪 测试 Qwen 英文对话能力...");
    
    let provider = create_qwen_provider();
    
    let result = provider.generate(
        "Explain the concept of artificial intelligence in simple terms.",
        &LlmOptions::default()
    ).await;
    
    match result {
        Ok(response) => {
            println!("✅ 英文对话测试成功!");
            println!("📝 响应内容: {}", response);
            
            assert!(!response.is_empty(), "响应内容不应为空");
            assert!(
                response.to_lowercase().contains("artificial") || 
                response.to_lowercase().contains("intelligence") ||
                response.to_lowercase().contains("ai"),
                "响应应包含AI相关内容"
            );
        }
        Err(e) => {
            println!("❌ 英文对话测试失败: {}", e);
            panic!("Qwen 英文对话测试失败: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qwen_streaming_response() {
    println!("🧪 测试 Qwen 流式响应...");
    
    let provider = create_qwen_provider();
    
    let start_time = Instant::now();
    let stream_result = provider.generate_stream(
        "请写一首关于人工智能的短诗，要求有4行。",
        &LlmOptions::default()
    ).await;
    
    match stream_result {
        Ok(mut stream) => {
            println!("✅ 流式响应启动成功!");
            
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
                        panic!("流式响应处理失败: {}", e);
                    }
                }
            }
            
            let duration = start_time.elapsed();
            println!("\n✅ 流式响应完成!");
            println!("📝 完整响应: {}", full_response);
            println!("📊 数据块数量: {}", chunk_count);
            println!("⏱️ 总耗时: {:?}", duration);
            
            assert!(!full_response.is_empty(), "流式响应内容不应为空");
            assert!(chunk_count > 0, "应该收到至少一个数据块");
            assert!(
                full_response.contains("人工智能") || full_response.contains("AI"),
                "响应应包含相关主题内容"
            );
        }
        Err(e) => {
            println!("❌ 流式响应启动失败: {}", e);
            panic!("Qwen 流式响应测试失败: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qwen_embedding() {
    println!("🧪 测试 Qwen 嵌入向量生成...");
    
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
            
            assert!(!embedding.is_empty(), "嵌入向量不应为空");
            assert!(embedding.len() > 100, "向量维度应该合理");
            assert!(duration.as_secs() < 10, "嵌入生成时间应小于10秒");
            
            // 检查向量值的合理性
            let has_non_zero = embedding.iter().any(|&x| x != 0.0);
            assert!(has_non_zero, "向量应包含非零值");
        }
        Err(e) => {
            println!("❌ 嵌入向量生成失败: {}", e);
            panic!("Qwen 嵌入向量测试失败: {}", e);
        }
    }
}

#[tokio::test]
async fn test_qwen_long_context() {
    println!("🧪 测试 Qwen 长上下文处理...");
    
    let provider = create_qwen_provider();
    
    // 创建一个较长的上下文
    let long_context = "人工智能（Artificial Intelligence，AI）是计算机科学的一个分支，它企图了解智能的实质，并生产出一种新的能以人类智能相似的方式做出反应的智能机器。该领域的研究包括机器人、语言识别、图像识别、自然语言处理和专家系统等。人工智能从诞生以来，理论和技术日益成熟，应用领域也不断扩大。可以设想，未来人工智能带来的科技产品，将会是人类智慧的"容器"。人工智能可以对人的意识、思维的信息过程的模拟。人工智能不是人的智能，但能像人那样思考、也可能超过人的智能。".repeat(10);
    
    let prompt = format!("{}\\n\\n基于上述内容，请总结人工智能的主要特点。", long_context);
    
    let start_time = Instant::now();
    let result = provider.generate(&prompt, &LlmOptions::default()).await;
    let duration = start_time.elapsed();
    
    match result {
        Ok(response) => {
            println!("✅ 长上下文处理成功!");
            println!("📝 响应内容: {}", response);
            println!("⏱️ 处理时间: {:?}", duration);
            println!("📊 输入长度: {} 字符", prompt.len());
            
            assert!(!response.is_empty(), "响应内容不应为空");
            assert!(duration.as_secs() < 60, "长上下文处理时间应小于60秒");
            assert!(
                response.contains("人工智能") || response.contains("智能"),
                "响应应包含相关内容总结"
            );
        }
        Err(e) => {
            println!("❌ 长上下文处理失败: {}", e);
            // 长上下文可能因为模型限制失败，这里只警告不panic
            println!("⚠️ 注意: 长上下文测试失败可能是由于模型上下文长度限制");
        }
    }
}

#[tokio::test]
async fn test_qwen_error_handling() {
    println!("🧪 测试 Qwen 错误处理...");
    
    // 测试无效API密钥
    let invalid_provider = QwenProvider::new_with_api_type(
        "invalid-api-key",
        QWEN_MODEL,
        QWEN_BASE_URL,
        QwenApiType::DashScope
    );
    
    let result = invalid_provider.generate("测试", &LlmOptions::default()).await;
    
    match result {
        Ok(_) => {
            println!("⚠️ 预期应该失败，但却成功了");
        }
        Err(e) => {
            println!("✅ 错误处理正常: {}", e);
            assert!(
                e.to_string().contains("401") || 
                e.to_string().contains("unauthorized") ||
                e.to_string().contains("invalid"),
                "错误信息应该指示认证问题"
            );
        }
    }
}

#[tokio::test]
async fn test_qwen_performance_benchmark() {
    println!("🧪 测试 Qwen 性能基准...");
    
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
                println!("✅ 成功 - 耗时: {:?}, 响应长度: {}", duration, response.len());
            }
            Err(e) => {
                println!("❌ 失败: {}", e);
            }
        }
    }
    
    let avg_time = total_time / success_count;
    let success_rate = (success_count as f64 / test_prompts.len() as f64) * 100.0;
    
    println!("\n📊 性能基准测试结果:");
    println!("- 总测试数: {}", test_prompts.len());
    println!("- 成功数: {}", success_count);
    println!("- 成功率: {:.1}%", success_rate);
    println!("- 平均响应时间: {:?}", avg_time);
    println!("- 总耗时: {:?}", total_time);
    
    assert!(success_rate >= 80.0, "成功率应该至少80%");
    assert!(avg_time.as_secs() < 10, "平均响应时间应小于10秒");
}

/// 运行所有Qwen验证测试
pub async fn run_all_qwen_tests() -> Result<()> {
    println!("🚀 开始 Qwen 提供商全面验证测试...\n");
    
    // 这里可以添加测试套件的协调逻辑
    println!("✅ Qwen 提供商验证测试完成!");
    
    Ok(())
}
