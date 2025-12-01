use futures::StreamExt;
use lumosai_core::agent::streaming::{AgentEvent, IntoStreaming};
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::agent::{AgentConfig, BasicAgent};
use lumosai_core::llm::{Message, QwenApiType, QwenProvider, Role};
use std::sync::Arc;
use std::time::Instant;
use tokio;

/// 真实流式处理验证测试
/// 使用实际的LumosAI API进行流式处理功能验证
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🌊 LumosAI 真实流式处理验证测试");
    println!("========================================");
    println!("📋 配置信息:");
    println!("  - 模型: qwen3-30b-a3b");
    println!("  - API密钥: sk-bc977c4e31e542f1a34159cb42478198");
    println!("  - 基础URL: https://dashscope.aliyuncs.com/compatible-mode/v1");

    // 6.1 基础流式响应测试
    println!("\n📋 6.1 基础流式响应测试");
    test_basic_streaming().await?;

    // 6.2 长文本流式处理测试
    println!("\n📋 6.2 长文本流式处理测试");
    test_long_text_streaming().await?;

    // 6.3 多轮对话流式测试
    println!("\n📋 6.3 多轮对话流式测试");
    test_multi_turn_streaming().await?;

    // 6.4 流式处理性能测试
    println!("\n📋 6.4 流式处理性能测试");
    test_streaming_performance().await?;

    // 6.5 流式错误处理测试
    println!("\n📋 6.5 流式错误处理测试");
    test_streaming_error_handling().await?;

    println!("\n✅ 流式处理验证测试完成！");
    Ok(())
}

async fn test_basic_streaming() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试基础流式响应...");
    let start_time = Instant::now();

    // 测试用例 6.1.1: 创建支持流式的Agent
    println!("    🤖 测试创建支持流式的Agent");

    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let agent_config = AgentConfig {
        name: "StreamingAgent".to_string(),
        instructions: "你是一个有用的AI助手，能够提供详细和有用的回答。".to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
        tenant_id: None,
        isolation_level: None,
    };

    let agent = BasicAgent::new(agent_config, Arc::new(llm));
    let streaming_agent = agent.into_streaming();

    println!("      ✓ 流式Agent创建成功");

    // 测试用例 6.1.2: 基础流式响应
    println!("    🌊 测试基础流式响应");

    let messages = vec![Message {
        role: Role::User,
        content: "请详细介绍一下人工智能的发展历史，包括主要的里程碑事件。".to_string(),
        name: None,
        metadata: None,
    }];

    let stream_start = Instant::now();
    let options = AgentGenerateOptions::default();
    let mut stream = streaming_agent.execute_streaming(&messages, &options);

    let mut response_chunks = Vec::new();
    let mut chunk_count = 0;
    let mut total_content = String::new();

    println!("      🔄 开始接收流式响应:");

    while let Some(event_result) = stream.next().await {
        match event_result {
            Ok(event) => {
                match event {
                    AgentEvent::TextDelta { delta, .. } => {
                        chunk_count += 1;
                        total_content.push_str(&delta);
                        response_chunks.push(delta.clone());

                        // 显示前几个块的内容（截断显示）
                        if chunk_count <= 5 {
                            let display_content = if delta.len() > 50 {
                                format!("{}...", &delta[..50])
                            } else {
                                delta
                            };
                            println!("        块 {}: '{}'", chunk_count, display_content);
                        }
                    }
                    AgentEvent::GenerationComplete { .. } => {
                        println!("        ✓ 生成完成");
                        break;
                    }
                    _ => {
                        // 忽略其他事件类型
                    }
                }
            }
            Err(e) => {
                println!("        ❌ 流式响应错误: {}", e);
                break;
            }
        }
    }

    let stream_duration = stream_start.elapsed();

    println!("      ✓ 流式响应完成");
    println!("      📊 总块数: {}", chunk_count);
    println!("      📊 总内容长度: {} 字符", total_content.len());
    println!("      📊 流式处理耗时: {:?}", stream_duration);

    // 验证流式响应
    assert!(chunk_count > 0, "应该收到至少一个响应块");
    assert!(!total_content.trim().is_empty(), "总响应内容不能为空");
    assert!(total_content.len() > 100, "响应内容应该足够详细");

    // 验证内容相关性
    let content_lower = total_content.to_lowercase();
    assert!(
        content_lower.contains("人工智能") || content_lower.contains("ai"),
        "响应应该包含相关内容"
    );

    println!("      ✓ 流式响应验证通过");

    let duration = start_time.elapsed();
    println!("  ✅ 基础流式响应测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_long_text_streaming() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试长文本流式处理...");
    let start_time = Instant::now();

    // 创建Agent
    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let agent_config = AgentConfig {
        name: "LongTextAgent".to_string(),
        instructions: "你是一个专业的技术写作助手，能够生成详细、结构化的技术文档。".to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        tenant_id: None,
        isolation_level: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
        tenant_id: None,
        isolation_level: None,
    };

    let agent = BasicAgent::new(agent_config, Arc::new(llm));
    let streaming_agent = agent.into_streaming();

    // 测试用例 6.2.1: 长文本生成
    println!("    📝 测试长文本生成");

    let messages = vec![Message {
        role: Role::User,
        content: r#"请写一篇关于Rust编程语言的详细技术文档，包括以下内容：
1. Rust的历史和设计理念
2. 所有权系统的详细解释
3. 借用和生命周期
4. 错误处理机制
5. 并发编程特性
6. 实际应用案例
请确保内容详细、准确，并包含代码示例。"#
            .to_string(),
        name: None,
        metadata: None,
    }];

    let long_stream_start = Instant::now();
    let options = AgentGenerateOptions::default();
    let mut stream = streaming_agent.execute_streaming(&messages, &options);

    let mut chunk_count = 0;
    let mut total_content = String::new();
    let mut chunk_times = Vec::new();
    let mut last_chunk_time = Instant::now();

    println!("      🔄 开始接收长文本流式响应:");

    while let Some(event_result) = stream.next().await {
        match event_result {
            Ok(event) => {
                match event {
                    AgentEvent::TextDelta { delta, .. } => {
                        let current_time = Instant::now();
                        let chunk_interval = current_time.duration_since(last_chunk_time);
                        chunk_times.push(chunk_interval);
                        last_chunk_time = current_time;

                        chunk_count += 1;
                        total_content.push_str(&delta);

                        // 每10个块显示一次进度
                        if chunk_count % 10 == 0 {
                            println!(
                                "        已接收 {} 个块，当前内容长度: {} 字符",
                                chunk_count,
                                total_content.len()
                            );
                        }
                    }
                    AgentEvent::GenerationComplete { .. } => {
                        println!("        ✓ 长文本生成完成");
                        break;
                    }
                    _ => {
                        // 忽略其他事件类型
                    }
                }
            }
            Err(e) => {
                println!("        ❌ 长文本流式响应错误: {}", e);
                break;
            }
        }
    }

    let long_stream_duration = long_stream_start.elapsed();

    println!("      ✓ 长文本流式响应完成");
    println!("      📊 总块数: {}", chunk_count);
    println!("      📊 总内容长度: {} 字符", total_content.len());
    println!("      📊 总耗时: {:?}", long_stream_duration);

    // 计算流式处理统计
    if !chunk_times.is_empty() {
        let avg_chunk_interval =
            chunk_times.iter().sum::<std::time::Duration>() / chunk_times.len() as u32;
        let max_chunk_interval = chunk_times.iter().max().unwrap();
        let min_chunk_interval = chunk_times.iter().min().unwrap();

        println!("      📊 平均块间隔: {:?}", avg_chunk_interval);
        println!("      📊 最大块间隔: {:?}", max_chunk_interval);
        println!("      📊 最小块间隔: {:?}", min_chunk_interval);
    }

    // 验证长文本响应
    assert!(chunk_count > 10, "长文本应该产生多个响应块");
    assert!(total_content.len() > 1000, "长文本响应应该足够详细");

    // 验证内容结构
    let content_lower = total_content.to_lowercase();
    assert!(content_lower.contains("rust"), "应该包含Rust相关内容");
    assert!(
        content_lower.contains("所有权") || content_lower.contains("ownership"),
        "应该包含所有权相关内容"
    );

    println!("      ✓ 长文本流式处理验证通过");

    let duration = start_time.elapsed();
    println!("  ✅ 长文本流式处理测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_multi_turn_streaming() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试多轮对话流式...");
    let start_time = Instant::now();

    // 创建Agent
    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let agent_config = AgentConfig {
        name: "MultiTurnAgent".to_string(),
        instructions: "你是一个有用的AI助手，能够进行连贯的多轮对话。".to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
        tenant_id: None,
        isolation_level: None,
    };

    let agent = BasicAgent::new(agent_config, Arc::new(llm));
    let streaming_agent = agent.into_streaming();

    // 测试用例 6.3.1: 多轮对话流式处理
    println!("    💬 测试多轮对话流式处理");

    let conversation_turns = vec![
        "你好，我想学习编程，有什么建议吗？",
        "我对Rust语言很感兴趣，它适合初学者吗？",
        "能给我推荐一些Rust的学习资源吗？",
    ];

    let mut conversation_history = Vec::new();

    for (turn_index, user_input) in conversation_turns.iter().enumerate() {
        println!("      🔄 第 {} 轮对话", turn_index + 1);
        println!("        用户: {}", user_input);

        // 添加用户消息到对话历史
        conversation_history.push(Message {
            role: Role::User,
            content: user_input.to_string(),
            name: None,
            metadata: None,
        });

        let turn_start = Instant::now();
        let options = AgentGenerateOptions::default();

        // 克隆对话历史以避免借用冲突
        let current_history = conversation_history.clone();
        let mut stream = streaming_agent.execute_streaming(&current_history, &options);

        let mut turn_response = String::new();
        let mut turn_chunk_count = 0;

        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(event) => {
                    match event {
                        AgentEvent::TextDelta { delta, .. } => {
                            turn_chunk_count += 1;
                            turn_response.push_str(&delta);
                        }
                        AgentEvent::GenerationComplete { .. } => {
                            break;
                        }
                        _ => {
                            // 忽略其他事件类型
                        }
                    }
                }
                Err(e) => {
                    println!("        ❌ 第 {} 轮流式响应错误: {}", turn_index + 1, e);
                    break;
                }
            }
        }

        let turn_duration = turn_start.elapsed();

        // 添加助手响应到对话历史
        conversation_history.push(Message {
            role: Role::Assistant,
            content: turn_response.clone(),
            name: None,
            metadata: None,
        });

        println!(
            "        助手: {}",
            if turn_response.chars().count() > 50 {
                format!("{}...", turn_response.chars().take(50).collect::<String>())
            } else {
                turn_response.clone()
            }
        );
        println!(
            "        📊 块数: {}, 耗时: {:?}",
            turn_chunk_count, turn_duration
        );

        // 验证每轮响应
        assert!(!turn_response.trim().is_empty(), "每轮响应不能为空");
        assert!(turn_chunk_count > 0, "每轮应该产生至少一个响应块");

        println!("        ✓ 第 {} 轮验证通过", turn_index + 1);
    }

    println!("      ✓ 多轮对话流式处理完成");
    println!("      📊 总对话轮数: {}", conversation_turns.len());
    println!(
        "      📊 对话历史长度: {} 条消息",
        conversation_history.len()
    );

    let duration = start_time.elapsed();
    println!("  ✅ 多轮对话流式测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_streaming_performance() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试流式处理性能...");
    let start_time = Instant::now();

    // 创建Agent
    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let agent_config = AgentConfig {
        name: "PerformanceAgent".to_string(),
        instructions: "你是一个高效的AI助手。".to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
        tenant_id: None,
        isolation_level: None,
    };

    let agent = BasicAgent::new(agent_config, Arc::new(llm));
    let streaming_agent = agent.into_streaming();

    // 测试用例 6.4.1: 流式处理延迟测试
    println!("    ⚡ 测试流式处理延迟");

    let messages = vec![Message {
        role: Role::User,
        content: "请简单介绍一下机器学习的基本概念。".to_string(),
        name: None,
        metadata: None,
    }];

    let latency_start = Instant::now();
    let options = AgentGenerateOptions::default();
    let mut stream = streaming_agent.execute_streaming(&messages, &options);

    let mut first_chunk_time = None;
    let mut chunk_count = 0;
    let mut total_content = String::new();

    while let Some(event_result) = stream.next().await {
        match event_result {
            Ok(event) => {
                match event {
                    AgentEvent::TextDelta { delta, .. } => {
                        if first_chunk_time.is_none() {
                            first_chunk_time = Some(Instant::now());
                        }
                        chunk_count += 1;
                        total_content.push_str(&delta);
                    }
                    AgentEvent::GenerationComplete { .. } => {
                        break;
                    }
                    _ => {
                        // 忽略其他事件类型
                    }
                }
            }
            Err(e) => {
                println!("        ❌ 性能测试流式响应错误: {}", e);
                break;
            }
        }
    }

    let total_duration = latency_start.elapsed();
    let first_chunk_latency = first_chunk_time
        .map(|t| t.duration_since(latency_start))
        .unwrap_or_default();

    println!("      ✓ 流式处理性能测试完成");
    println!("      📊 首块延迟: {:?}", first_chunk_latency);
    println!("      📊 总处理时间: {:?}", total_duration);
    println!("      📊 总块数: {}", chunk_count);
    println!(
        "      📊 平均块处理时间: {:?}",
        total_duration / chunk_count.max(1) as u32
    );

    // 性能验证
    assert!(
        first_chunk_latency.as_secs() < 10,
        "首块延迟应该在合理范围内"
    );
    assert!(chunk_count > 0, "应该收到响应块");

    println!("      ✓ 流式处理性能验证通过");

    let duration = start_time.elapsed();
    println!("  ✅ 流式处理性能测试完成! 耗时: {:?}", duration);

    Ok(())
}

async fn test_streaming_error_handling() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试流式错误处理...");
    let start_time = Instant::now();

    // 测试用例 6.5.1: 正常流式处理（作为对照）
    println!("    ✅ 测试正常流式处理（对照组）");

    let llm = QwenProvider::new_with_api_type(
        "sk-bc977c4e31e542f1a34159cb42478198",
        "qwen3-30b-a3b",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        QwenApiType::OpenAICompatible,
    );

    let agent_config = AgentConfig {
        name: "ErrorTestAgent".to_string(),
        instructions: "你是一个AI助手。".to_string(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(false),
        context: None,
        metadata: None,
        max_tool_calls: None,
        tool_timeout: None,
        tenant_id: None,
        isolation_level: None,
    };

    let agent = BasicAgent::new(agent_config, Arc::new(llm));
    let streaming_agent = agent.into_streaming();

    let messages = vec![Message {
        role: Role::User,
        content: "你好".to_string(),
        name: None,
        metadata: None,
    }];

    let normal_start = Instant::now();
    let options = AgentGenerateOptions::default();
    let mut stream = streaming_agent.execute_streaming(&messages, &options);

    let mut normal_chunk_count = 0;
    let mut normal_success = false;

    while let Some(event_result) = stream.next().await {
        match event_result {
            Ok(event) => {
                match event {
                    AgentEvent::TextDelta { .. } => {
                        normal_chunk_count += 1;
                        normal_success = true;
                    }
                    AgentEvent::GenerationComplete { .. } => {
                        break;
                    }
                    _ => {
                        // 忽略其他事件类型
                    }
                }
            }
            Err(e) => {
                println!("        ❌ 正常流式处理出现错误: {}", e);
                break;
            }
        }
    }

    let normal_duration = normal_start.elapsed();

    println!("      ✓ 正常流式处理完成");
    println!("      📊 正常处理块数: {}", normal_chunk_count);
    println!("      📊 正常处理耗时: {:?}", normal_duration);

    // 验证正常处理
    assert!(normal_success, "正常流式处理应该成功");
    assert!(normal_chunk_count > 0, "正常处理应该产生响应块");

    println!("      ✓ 正常流式处理验证通过");

    // 测试用例 6.5.2: 流式处理鲁棒性测试
    println!("    🛡️ 测试流式处理鲁棒性");

    // 测试空消息处理
    let empty_messages = vec![];

    let mut empty_stream = streaming_agent.execute_streaming(&empty_messages, &options);
    let mut empty_chunk_count = 0;

    while let Some(event_result) = empty_stream.next().await {
        match event_result {
            Ok(event) => match event {
                AgentEvent::TextDelta { .. } => empty_chunk_count += 1,
                AgentEvent::GenerationComplete { .. } => break,
                _ => {}
            },
            Err(e) => {
                println!("      ⚠️ 空消息处理错误（预期）: {}", e);
                break;
            }
        }
    }

    println!("      📊 空消息处理块数: {}", empty_chunk_count);

    println!("      ✓ 流式处理鲁棒性测试完成");

    let duration = start_time.elapsed();
    println!("  ✅ 流式错误处理测试完成! 耗时: {:?}", duration);

    Ok(())
}
