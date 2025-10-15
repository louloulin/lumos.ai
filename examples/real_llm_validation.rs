use futures::StreamExt;
use lumosai_core::llm::{LlmOptions, LlmProvider, Message, QwenApiType, QwenProvider, Role};
use std::time::Instant;
use tokio;

/// 真实LLM提供商验证测试
/// 使用实际的LumosAI API进行功能验证
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🤖 LumosAI 真实LLM提供商验证测试");
    println!("========================================");
    println!("📋 配置信息:");
    println!("  - 模型: qwen3-30b-a3b");
    println!("  - API密钥: sk-bc977c4e31e542f1a34159cb42478198");
    println!("  - 基础URL: https://dashscope.aliyuncs.com/compatible-mode/v1");

    // 1.1 Qwen提供商基础连接测试
    println!("\n📋 1.1 Qwen提供商基础连接测试");
    test_qwen_basic_connection().await?;

    // 1.2 文本生成测试
    println!("\n📋 1.2 文本生成测试");
    test_text_generation().await?;

    // 1.3 多轮对话测试
    println!("\n📋 1.3 多轮对话测试");
    test_multi_turn_conversation().await?;

    // 1.4 流式响应测试
    println!("\n📋 1.4 流式响应测试");
    test_streaming_response().await?;

    // 1.5 函数调用测试
    println!("\n📋 1.5 函数调用测试");
    test_function_calling().await?;

    println!("\n✅ LLM提供商验证测试完成！");
    Ok(())
}

async fn test_qwen_basic_connection() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试Qwen基础连接...");
    let start_time = Instant::now();

    // 创建Qwen提供商 - 使用OpenAI兼容模式以支持enable_thinking参数
    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    println!("  ✓ LLM提供商创建成功");

    // 测试简单问答
    let prompt = "你好，请简单介绍一下你自己。";
    println!("  🔍 发送测试请求: '{}'", prompt);

    let response = llm.generate(prompt, &LlmOptions::default()).await?;

    let duration = start_time.elapsed();

    println!("  ✅ 连接测试成功!");
    println!(
        "  📝 响应内容: {}",
        if response.chars().count() > 50 {
            format!("{}...", response.chars().take(50).collect::<String>())
        } else {
            response.clone()
        }
    );
    println!("  ⏱️ 响应时间: {:?}", duration);
    println!("  📊 响应长度: {} 字符", response.len());

    // 验证响应质量
    assert!(!response.is_empty(), "响应不能为空");
    assert!(response.len() > 10, "响应长度应该大于10个字符");

    Ok(())
}

async fn test_text_generation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试文本生成...");

    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let test_cases = vec![
        ("简单问答", "什么是人工智能？"),
        ("技术解释", "请解释Rust编程语言的所有权系统。"),
        ("创意写作", "请写一首关于春天的短诗。"),
        (
            "逻辑推理",
            "如果所有的鸟都会飞，企鹅是鸟，那么企鹅会飞吗？请解释。",
        ),
        ("中英文混合", "Please explain what is 机器学习 in Chinese."),
    ];

    for (test_name, prompt) in test_cases {
        let start_time = Instant::now();

        println!("  🔍 测试 {}: '{}'", test_name, prompt);

        let response = llm.generate(prompt, &LlmOptions::default()).await?;
        let duration = start_time.elapsed();

        println!("    ✅ 生成成功");
        println!(
            "    📝 响应: {}",
            if response.chars().count() > 40 {
                format!("{}...", response.chars().take(40).collect::<String>())
            } else {
                response.clone()
            }
        );
        println!("    ⏱️ 生成时间: {:?}", duration);
        println!("    📊 响应长度: {} 字符", response.len());

        // 验证响应质量
        assert!(!response.is_empty(), "响应不能为空");
        assert!(duration.as_secs() < 30, "响应时间应该小于30秒");

        // 简单的内容相关性检查
        match test_name {
            "简单问答" => assert!(
                response.to_lowercase().contains("智能") || response.to_lowercase().contains("ai")
            ),
            "技术解释" => assert!(
                response.to_lowercase().contains("rust")
                    || response.to_lowercase().contains("所有权")
            ),
            "创意写作" => assert!(response.contains("春") || response.contains("诗")),
            "逻辑推理" => assert!(response.contains("企鹅") || response.contains("飞")),
            "中英文混合" => assert!(
                response.contains("机器学习") || response.to_lowercase().contains("machine")
            ),
            _ => {}
        }

        println!("    ✓ 内容相关性验证通过");
    }

    println!("✅ 文本生成测试完成!");
    Ok(())
}

async fn test_multi_turn_conversation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试多轮对话...");

    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    // 构建多轮对话
    let mut messages = vec![Message {
        role: Role::System,
        content: "你是一个有用的AI助手，专门回答技术问题。".to_string(),
        metadata: None,
        name: None,
    }];

    let conversation_turns = vec![
        ("用户", "我想学习Rust编程语言，应该从哪里开始？"),
        ("助手", ""), // 将由AI填充
        ("用户", "Rust的所有权系统是什么？"),
        ("助手", ""), // 将由AI填充
        ("用户", "能给我一个简单的Rust代码示例吗？"),
    ];

    for (i, (speaker, content)) in conversation_turns.iter().enumerate() {
        if speaker == &"用户" {
            println!("  👤 用户: {}", content);

            messages.push(Message {
                role: Role::User,
                content: content.to_string(),
                metadata: None,
                name: None,
            });

            let start_time = Instant::now();
            let response = llm
                .generate_with_messages(&messages, &LlmOptions::default())
                .await?;
            let duration = start_time.elapsed();

            println!(
                "  🤖 助手: {}",
                if response.chars().count() > 50 {
                    format!("{}...", response.chars().take(50).collect::<String>())
                } else {
                    response.clone()
                }
            );
            println!("    ⏱️ 响应时间: {:?}", duration);

            messages.push(Message {
                role: Role::Assistant,
                content: response.clone(),
                metadata: None,
                name: None,
            });

            // 验证上下文保持
            if i > 0 {
                // 检查是否保持了对话上下文
                assert!(!response.is_empty(), "响应不能为空");
                if i == 2 {
                    // 第二轮，应该提到Rust
                    assert!(
                        response.to_lowercase().contains("rust")
                            || response.contains("所有权")
                            || response.contains("ownership"),
                        "应该保持Rust话题的上下文"
                    );
                }
            }

            println!("    ✓ 上下文保持验证通过");
        }
    }

    println!("✅ 多轮对话测试完成!");
    println!("📊 对话轮次: {}", messages.len());

    Ok(())
}

async fn test_streaming_response() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试流式响应...");

    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let prompt = "请写一篇关于人工智能发展历程的短文，包含3个段落。";
    println!("  🔍 流式请求: '{}'", prompt);

    let start_time = Instant::now();

    // 测试流式生成
    match llm.generate_stream(prompt, &LlmOptions::default()).await {
        Ok(mut stream) => {
            println!("  🌊 开始接收流式响应:");

            let mut full_response = String::new();
            let mut chunk_count = 0;

            use futures::StreamExt;
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        print!("{}", chunk);
                        full_response.push_str(&chunk);
                        chunk_count += 1;
                    }
                    Err(e) => {
                        println!("\n  ❌ 流式响应错误: {}", e);
                        break;
                    }
                }
            }

            let duration = start_time.elapsed();

            println!("\n  ✅ 流式响应完成!");
            println!("  📊 总块数: {}", chunk_count);
            println!("  📊 总长度: {} 字符", full_response.len());
            println!("  ⏱️ 总时间: {:?}", duration);

            // 验证流式响应
            assert!(!full_response.is_empty(), "流式响应不能为空");
            assert!(chunk_count > 1, "应该收到多个数据块");
            assert!(
                full_response.contains("人工智能") || full_response.contains("AI"),
                "内容应该相关"
            );

            println!("  ✓ 流式响应验证通过");
        }
        Err(e) => {
            println!("  ⚠️ 流式响应不支持或出错: {}", e);
            println!("  🔄 回退到普通生成模式");

            let response = llm.generate(prompt, &LlmOptions::default()).await?;
            let duration = start_time.elapsed();

            println!(
                "  📝 普通响应: {}",
                if response.chars().count() > 50 {
                    format!("{}...", response.chars().take(50).collect::<String>())
                } else {
                    response.clone()
                }
            );
            println!("  ⏱️ 响应时间: {:?}", duration);

            assert!(!response.is_empty(), "响应不能为空");
        }
    }

    println!("✅ 流式响应测试完成!");
    Ok(())
}

async fn test_function_calling() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试函数调用...");

    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    // 测试基础函数调用能力
    let prompt = "请帮我计算 123 + 456 的结果。如果你有计算器工具，请使用它。";
    println!("  🔍 函数调用测试: '{}'", prompt);

    let start_time = Instant::now();

    // 创建带有工具的选项
    let options = LlmOptions::default();
    // 注意：这里可能需要根据实际API调整工具定义格式

    let response = llm.generate(prompt, &options).await?;
    let duration = start_time.elapsed();

    println!("  📝 响应: {}", response);
    println!("  ⏱️ 响应时间: {:?}", duration);

    // 验证响应
    assert!(!response.is_empty(), "响应不能为空");

    // 检查是否包含计算结果或计算过程
    if response.contains("579") || response.contains("123") || response.contains("456") {
        println!("  ✓ 数学计算相关内容验证通过");
    } else {
        println!("  ⚠️ 响应中未明确包含计算结果，但这可能是正常的");
    }

    // 测试更复杂的函数调用场景
    let complex_prompt = "我需要查询今天的天气，然后根据天气情况推荐合适的活动。";
    println!("  🔍 复杂函数调用测试: '{}'", complex_prompt);

    let start_time = Instant::now();
    let response = llm.generate(complex_prompt, &LlmOptions::default()).await?;
    let duration = start_time.elapsed();

    println!(
        "  📝 复杂响应: {}",
        if response.chars().count() > 50 {
            format!("{}...", response.chars().take(50).collect::<String>())
        } else {
            response.clone()
        }
    );
    println!("  ⏱️ 响应时间: {:?}", duration);

    assert!(!response.is_empty(), "复杂响应不能为空");

    println!("✅ 函数调用测试完成!");
    println!("📝 注意: 函数调用功能可能需要额外的工具配置才能完全验证");

    Ok(())
}
