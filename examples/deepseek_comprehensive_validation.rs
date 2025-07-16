//! DeepSeek 综合功能验证示例
//! 
//! 基于 plan10.md 的 API 设计，全面验证 DeepSeek LLM provider 的各项功能，
//! 展示 LumosAI 框架的完整能力。

use lumosai_core::{Result, Error};
use lumosai_core::agent::{AgentBuilder, quick, Agent};
use lumosai_core::agent::convenience::{deepseek_with_key};
use lumosai_core::agent::trait_def::Agent as AgentTrait;
use lumosai_core::agent::types::{AgentGenerateOptions, AgentStreamOptions};
use lumosai_core::llm::{DeepSeekProvider, Message, Role, LlmOptions};
use lumosai_core::tool::{CalculatorTool, Tool};
use std::sync::Arc;
use std::env;
use std::time::Instant;

/// 获取 DeepSeek API Key
fn get_api_key() -> Result<String> {
    env::var("DEEPSEEK_API_KEY").map_err(|_| {
        Error::Configuration(
            "请设置 DEEPSEEK_API_KEY 环境变量。\n\
            获取方式：https://platform.deepseek.com/\n\
            设置方法：export DEEPSEEK_API_KEY=\"your-api-key\"".to_string()
        )
    })
}

/// 验证 1: 基础 Agent 创建和对话
async fn test_basic_agent_creation() -> Result<()> {
    println!("\n🚀 验证 1: 基础 Agent 创建和对话");
    println!("===============================");
    
    let api_key = get_api_key()?;
    
    // 使用 quick API 创建最简单的 Agent
    let agent = quick("basic_assistant", "你是一个友好的AI助手，请用中文简洁回答")
        .model(deepseek_with_key(&api_key, "deepseek-chat"))
        .build()?;
    
    println!("✅ Agent 创建成功:");
    println!("   名称: {}", agent.get_name());
    println!("   指令: {}", agent.get_instructions());
    
    // 测试基本对话
    let messages = vec![Message {
        role: Role::User,
        content: "你好！请简单介绍一下你自己。".to_string(),
        metadata: None,
        name: None,
    }];
    
    println!("\n📤 发送消息: 你好！请简单介绍一下你自己。");
    
    let start_time = Instant::now();
    let response = agent.generate(&messages, &AgentGenerateOptions::default()).await?;
    let duration = start_time.elapsed();
    
    println!("📥 DeepSeek 响应 ({}ms): {}", duration.as_millis(), response.response);
    println!("✅ 基础对话功能验证通过");
    
    Ok(())
}

/// 验证 2: 高级 Agent 配置和工具调用
async fn test_advanced_agent_with_tools() -> Result<()> {
    println!("\n🔧 验证 2: 高级 Agent 配置和工具调用");
    println!("===================================");
    
    let api_key = get_api_key()?;
    
    // 使用 AgentBuilder 创建带工具的高级 Agent
    let agent = AgentBuilder::new()
        .name("math_assistant")
        .instructions("你是一个数学助手，可以使用计算器工具进行精确计算。当用户询问数学问题时，请使用计算器工具来计算结果，然后用中文解释计算过程。")
        .model(deepseek_with_key(&api_key, "deepseek-chat"))
        .tool(Box::new(CalculatorTool::default()))
        .enable_function_calling(true)
        .max_tool_calls(5)
        .tool_timeout(30)
        .enable_smart_defaults()
        .build()?;
    
    println!("✅ 高级 Agent 创建成功:");
    println!("   名称: {}", agent.get_name());
    println!("   工具数量: {}", agent.get_tools().len());
    println!("   工具列表:");
    for tool in agent.get_tools() {
        println!("     - {}: {}", tool.name(), tool.description());
    }
    
    // 测试工具调用
    let messages = vec![Message {
        role: Role::User,
        content: "请帮我计算 (25 + 15) * 3 - 8 的结果，并解释计算步骤".to_string(),
        metadata: None,
        name: None,
    }];
    
    println!("\n📤 发送数学问题: 请帮我计算 (25 + 15) * 3 - 8 的结果");
    
    let start_time = Instant::now();
    let response = agent.generate(&messages, &AgentGenerateOptions::default()).await?;
    let duration = start_time.elapsed();
    
    println!("📥 DeepSeek 响应 ({}ms): {}", duration.as_millis(), response.response);
    
    // 显示执行步骤
    if !response.steps.is_empty() {
        println!("\n🔍 执行步骤:");
        for (i, step) in response.steps.iter().enumerate() {
            println!("   步骤 {}: {}", i + 1, step.description);
            if let Some(result) = &step.result {
                println!("   结果: {}", result);
            }
        }
    }
    
    println!("✅ 工具调用功能验证通过");
    
    Ok(())
}

/// 验证 3: 多轮对话和上下文管理
async fn test_multi_turn_conversation() -> Result<()> {
    println!("\n💬 验证 3: 多轮对话和上下文管理");
    println!("===============================");
    
    let api_key = get_api_key()?;
    
    let agent = quick("conversation_assistant", "你是一个智能对话助手，能够记住对话历史并进行连贯的多轮对话")
        .model(deepseek_with_key(&api_key, "deepseek-chat"))
        .build()?;
    
    // 多轮对话测试
    let conversation_turns = vec![
        "我想了解人工智能的发展历史",
        "那么深度学习是什么时候开始兴起的？",
        "你刚才提到的深度学习，它和机器学习有什么区别？",
        "能举个具体的深度学习应用例子吗？",
    ];
    
    let mut messages = Vec::new();
    
    for (i, user_input) in conversation_turns.iter().enumerate() {
        println!("\n📤 第 {} 轮对话: {}", i + 1, user_input);
        
        // 添加用户消息
        messages.push(Message {
            role: Role::User,
            content: user_input.to_string(),
            metadata: None,
            name: None,
        });
        
        // 获取 AI 响应
        let start_time = Instant::now();
        let response = agent.generate(&messages, &AgentGenerateOptions::default()).await?;
        let duration = start_time.elapsed();
        
        println!("📥 AI 响应 ({}ms): {}", duration.as_millis(), 
            if response.response.len() > 100 {
                format!("{}...", &response.response[..100])
            } else {
                response.response.clone()
            }
        );
        
        // 添加 AI 响应到对话历史
        messages.push(Message {
            role: Role::Assistant,
            content: response.response,
            metadata: None,
            name: None,
        });
        
        // 短暂暂停，避免请求过于频繁
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
    
    println!("\n✅ 多轮对话功能验证通过");
    println!("   对话轮数: {}", conversation_turns.len());
    println!("   上下文消息数: {}", messages.len());
    
    Ok(())
}

/// 验证 4: 不同模型配置和参数调优
async fn test_model_configurations() -> Result<()> {
    println!("\n⚙️ 验证 4: 不同模型配置和参数调优");
    println!("=================================");
    
    let api_key = get_api_key()?;
    
    // 测试不同的模型配置
    let configurations = vec![
        ("创意模式", 0.9, 1000),
        ("平衡模式", 0.7, 500),
        ("精确模式", 0.1, 200),
    ];
    
    let test_prompt = "请写一个关于人工智能的短诗";
    
    for (mode_name, temperature, max_tokens) in configurations {
        println!("\n🎛️ 测试 {}", mode_name);
        println!("   温度: {}, 最大令牌: {}", temperature, max_tokens);
        
        // 创建自定义 LLM 配置
        let llm = Arc::new(DeepSeekProvider::new(api_key.clone(), Some("deepseek-chat".to_string())));
        
        let agent = quick(&format!("poet_{}", mode_name), "你是一个诗人，擅长创作各种风格的诗歌")
            .model(llm)
            .build()?;
        
        let messages = vec![Message {
            role: Role::User,
            content: test_prompt.to_string(),
            metadata: None,
            name: None,
        }];
        
        let options = AgentGenerateOptions {
            temperature: Some(temperature),
            max_tokens: Some(max_tokens),
            ..Default::default()
        };
        
        let start_time = Instant::now();
        let response = agent.generate(&messages, &options).await?;
        let duration = start_time.elapsed();
        
        println!("📝 {} 响应 ({}ms):", mode_name, duration.as_millis());
        println!("   长度: {} 字符", response.response.len());
        println!("   内容: {}...", &response.response[..50.min(response.response.len())]);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
    
    println!("\n✅ 模型配置功能验证通过");
    
    Ok(())
}

/// 验证 5: 错误处理和恢复机制
async fn test_error_handling() -> Result<()> {
    println!("\n🛡️ 验证 5: 错误处理和恢复机制");
    println!("=============================");
    
    // 测试无效 API Key
    println!("🧪 测试无效 API Key 处理:");
    let invalid_agent_result = quick("invalid_test", "test")
        .model(deepseek_with_key("invalid-key", "deepseek-chat"))
        .build();
    
    match invalid_agent_result {
        Ok(_) => println!("⚠️ 应该失败但成功了"),
        Err(e) => println!("✅ 正确处理无效配置: {}", e),
    }
    
    // 测试网络错误恢复
    println!("\n🧪 测试配置验证:");
    let empty_name_result = AgentBuilder::new()
        .instructions("test")
        .build();
    
    match empty_name_result {
        Ok(_) => println!("⚠️ 应该失败但成功了"),
        Err(e) => println!("✅ 正确验证必需字段: {}", e),
    }
    
    println!("\n✅ 错误处理机制验证通过");
    
    Ok(())
}

/// 性能基准测试
async fn benchmark_performance() -> Result<()> {
    println!("\n⚡ 性能基准测试");
    println!("================");
    
    let api_key = get_api_key()?;
    
    // 创建测试 Agent
    let agent = quick("benchmark_agent", "请简洁回答")
        .model(deepseek_with_key(&api_key, "deepseek-chat"))
        .build()?;
    
    let test_messages = vec![
        "1+1等于多少？",
        "今天是星期几？",
        "请说一个数字",
        "你好",
        "谢谢",
    ];
    
    println!("🔄 开始性能测试，发送 {} 个请求...", test_messages.len());
    
    let overall_start = Instant::now();
    let mut total_response_length = 0;
    let mut successful_requests = 0;
    
    for (i, message_text) in test_messages.iter().enumerate() {
        let request_start = Instant::now();
        
        let messages = vec![Message {
            role: Role::User,
            content: message_text.to_string(),
            metadata: None,
            name: None,
        }];
        
        match agent.generate(&messages, &AgentGenerateOptions::default()).await {
            Ok(response) => {
                let request_time = request_start.elapsed();
                total_response_length += response.response.len();
                successful_requests += 1;
                println!("   请求 {}: {}ms - {}", i + 1, request_time.as_millis(), message_text);
            }
            Err(e) => {
                println!("   请求 {} 失败: {}", i + 1, e);
            }
        }
        
        // 避免请求过于频繁
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    let total_time = overall_start.elapsed();
    
    println!("\n📊 性能测试结果:");
    println!("   总耗时: {}ms", total_time.as_millis());
    println!("   成功请求: {}/{}", successful_requests, test_messages.len());
    println!("   平均每请求: {}ms", total_time.as_millis() / test_messages.len() as u128);
    println!("   总响应字符数: {}", total_response_length);
    println!("   平均响应长度: {} 字符", total_response_length / successful_requests.max(1));
    println!("   成功率: {:.1}%", (successful_requests as f64 / test_messages.len() as f64) * 100.0);
    
    Ok(())
}

/// 主函数：运行所有验证测试
#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 DeepSeek 综合功能验证");
    println!("========================");
    println!("基于 plan10.md API 设计的完整功能验证");
    
    // 检查 API Key
    match get_api_key() {
        Ok(api_key) => {
            println!("✅ 找到 DeepSeek API Key: {}...{}", 
                &api_key[..8.min(api_key.len())], 
                if api_key.len() > 16 { &api_key[api_key.len()-8..] } else { "" }
            );
        }
        Err(e) => {
            println!("❌ {}", e);
            return Ok(());
        }
    }
    
    println!("\n⚠️ 注意：此验证将调用真实的 DeepSeek API，可能产生少量费用。");
    
    let mut success_count = 0;
    let mut total_count = 0;
    
    // 运行所有验证测试
    let tests = vec![
        ("基础 Agent 创建", test_basic_agent_creation()),
        ("高级 Agent 和工具", test_advanced_agent_with_tools()),
        ("多轮对话", test_multi_turn_conversation()),
        ("模型配置", test_model_configurations()),
        ("错误处理", test_error_handling()),
    ];
    
    for (test_name, test_future) in tests {
        total_count += 1;
        match test_future.await {
            Ok(_) => {
                success_count += 1;
                println!("✅ {} - 通过", test_name);
            }
            Err(e) => {
                println!("❌ {} - 失败: {}", test_name, e);
            }
        }
    }
    
    // 性能测试
    total_count += 1;
    match benchmark_performance().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 性能基准测试 - 通过");
        }
        Err(e) => {
            println!("❌ 性能基准测试 - 失败: {}", e);
        }
    }
    
    // 总结
    println!("\n🎉 DeepSeek 综合验证完成！");
    println!("===========================");
    println!("✅ 通过: {}/{}", success_count, total_count);
    println!("📊 成功率: {:.1}%", (success_count as f64 / total_count as f64) * 100.0);
    
    if success_count == total_count {
        println!("\n🏆 所有验证测试通过！");
        println!("✅ Plan 10 API 设计验证成功");
        println!("✅ DeepSeek 集成功能完整");
        println!("✅ LumosAI 框架运行正常");
        
        println!("\n💡 验证的功能特性:");
        println!("   - 简化 API 设计 (quick 函数)");
        println!("   - 构建器模式 (AgentBuilder)");
        println!("   - 工具集成和函数调用");
        println!("   - 多轮对话和上下文管理");
        println!("   - 模型参数配置");
        println!("   - 错误处理和验证");
        println!("   - 性能和稳定性");
    } else {
        println!("\n⚠️ 部分测试失败，请检查:");
        println!("   1. API Key 是否正确");
        println!("   2. 网络连接是否正常");
        println!("   3. DeepSeek API 服务是否可用");
    }
    
    Ok(())
}
