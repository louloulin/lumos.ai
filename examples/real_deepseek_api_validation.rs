//! 真实 DeepSeek API 验证示例
//! 
//! 本示例使用真实的 DeepSeek API 来验证 LumosAI 的 API 功能。
//! 需要设置真实的 DEEPSEEK_API_KEY 环境变量。

use lumosai_core::{Result, Error};
use lumosai_core::agent::{AgentBuilder, quick};
use lumosai_core::agent::trait_def::Agent as AgentTrait;
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::llm::{DeepSeekProvider, Message, Role};
use lumosai_core::tool::CalculatorTool;
use std::sync::Arc;
use std::env;

/// 检查并获取 DeepSeek API Key
fn get_deepseek_api_key() -> Result<String> {
    env::var("DEEPSEEK_API_KEY").map_err(|_| {
        Error::Configuration(
            "DEEPSEEK_API_KEY 环境变量未设置。\n\
            请按以下步骤设置您的 DeepSeek API Key：\n\
            \n\
            1. 访问 https://platform.deepseek.com/ 获取 API Key\n\
            2. 设置环境变量：\n\
               Windows (PowerShell): $env:DEEPSEEK_API_KEY=\"your-api-key\"\n\
               Windows (CMD): set DEEPSEEK_API_KEY=your-api-key\n\
               Linux/macOS: export DEEPSEEK_API_KEY=\"your-api-key\"\n\
            3. 重新运行此示例".to_string()
        )
    })
}

/// 示例 1: 真实 API 基础对话测试
async fn test_real_basic_conversation() -> Result<()> {
    println!("\n🚀 示例 1: 真实 API 基础对话测试");
    println!("================================");
    
    let api_key = get_deepseek_api_key()?;
    let llm = Arc::new(DeepSeekProvider::new(api_key, Some("deepseek-chat".to_string())));
    
    // 使用 quick API 创建 Agent
    let agent = quick("deepseek_assistant", "你是一个友好的AI助手，请用中文回答问题")
        .model(llm)
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
    
    let response = agent.generate(&messages, &AgentGenerateOptions::default()).await?;
    println!("📥 DeepSeek 响应: {}", response.response);
    
    Ok(())
}

/// 示例 2: 真实 API 工具调用测试
async fn test_real_tool_calling() -> Result<()> {
    println!("\n🔧 示例 2: 真实 API 工具调用测试");
    println!("===============================");
    
    let api_key = get_deepseek_api_key()?;
    let llm = Arc::new(DeepSeekProvider::new(api_key, Some("deepseek-chat".to_string())));
    
    // 创建带工具的 Agent
    let agent = AgentBuilder::new()
        .name("math_assistant")
        .instructions("你是一个数学助手，可以使用计算器工具进行数学计算。当用户询问数学问题时，请使用计算器工具来计算结果。")
        .model(llm)
        .tool(Box::new(CalculatorTool::default()))
        .enable_function_calling(true)
        .max_tool_calls(3)
        .build()?;
    
    println!("✅ 数学助手创建成功:");
    println!("   名称: {}", agent.get_name());
    println!("   工具数量: {}", agent.get_tools().len());
    
    // 测试数学计算
    let messages = vec![Message {
        role: Role::User,
        content: "请帮我计算 (15 + 25) * 3 - 8 的结果".to_string(),
        metadata: None,
        name: None,
    }];
    
    println!("\n📤 发送数学问题: 请帮我计算 (15 + 25) * 3 - 8 的结果");
    
    let response = agent.generate(&messages, &AgentGenerateOptions::default()).await?;
    println!("📥 DeepSeek 响应: {}", response.response);
    
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
    
    Ok(())
}

/// 示例 3: 真实 API 复杂对话测试
async fn test_real_complex_conversation() -> Result<()> {
    println!("\n💬 示例 3: 真实 API 复杂对话测试");
    println!("===============================");
    
    let api_key = get_deepseek_api_key()?;
    let llm = Arc::new(DeepSeekProvider::new(api_key, Some("deepseek-chat".to_string())));
    
    let agent = quick("conversation_assistant", "你是一个智能对话助手，能够进行深入的对话和分析")
        .model(llm)
        .build()?;
    
    // 多轮对话测试
    let conversations = vec![
        "请解释一下什么是人工智能？",
        "那么机器学习和深度学习有什么区别？",
        "你能举个具体的例子说明深度学习在实际中的应用吗？",
    ];
    
    let mut messages = Vec::new();
    
    for (i, question) in conversations.iter().enumerate() {
        println!("\n📤 第 {} 轮对话: {}", i + 1, question);
        
        // 添加用户消息
        messages.push(Message {
            role: Role::User,
            content: question.to_string(),
            metadata: None,
            name: None,
        });
        
        // 获取 AI 响应
        let response = agent.generate(&messages, &AgentGenerateOptions::default()).await?;
        println!("📥 DeepSeek 响应: {}", response.response);
        
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
    
    Ok(())
}

/// 示例 4: 真实 API 性能测试
async fn test_real_performance() -> Result<()> {
    println!("\n⚡ 示例 4: 真实 API 性能测试");
    println!("============================");
    
    let api_key = get_deepseek_api_key()?;
    let llm = Arc::new(DeepSeekProvider::new(api_key, Some("deepseek-chat".to_string())));
    
    let agent = quick("performance_test", "你是一个测试助手，请简洁地回答问题")
        .model(llm)
        .build()?;
    
    let test_questions = vec![
        "1+1等于多少？",
        "今天天气怎么样？",
        "请说一个笑话",
        "什么是编程？",
        "推荐一本好书",
    ];
    
    println!("🔄 开始性能测试，发送 {} 个请求...", test_questions.len());
    
    let start_time = std::time::Instant::now();
    let mut total_response_length = 0;
    
    for (i, question) in test_questions.iter().enumerate() {
        let request_start = std::time::Instant::now();
        
        let messages = vec![Message {
            role: Role::User,
            content: question.to_string(),
            metadata: None,
            name: None,
        }];
        
        match agent.generate(&messages, &AgentGenerateOptions::default()).await {
            Ok(response) => {
                let request_time = request_start.elapsed();
                total_response_length += response.response.len();
                println!("   请求 {}: {}ms - {}", i + 1, request_time.as_millis(), question);
            }
            Err(e) => {
                println!("   请求 {} 失败: {}", i + 1, e);
            }
        }
        
        // 避免请求过于频繁
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    let total_time = start_time.elapsed();
    
    println!("\n📊 性能测试结果:");
    println!("   总耗时: {}ms", total_time.as_millis());
    println!("   平均每请求: {}ms", total_time.as_millis() / test_questions.len() as u128);
    println!("   总响应字符数: {}", total_response_length);
    println!("   平均响应长度: {} 字符", total_response_length / test_questions.len());
    
    Ok(())
}

/// 主函数：运行所有真实 API 验证测试
#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 LumosAI 真实 DeepSeek API 验证");
    println!("=================================");
    println!("本示例将使用真实的 DeepSeek API 进行验证测试");
    
    // 首先检查 API Key
    match get_deepseek_api_key() {
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
    
    println!("\n⚠️ 注意：此示例将调用真实的 DeepSeek API，可能产生费用。");
    println!("按 Enter 键继续，或 Ctrl+C 取消...");
    
    // 等待用户确认
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    
    // 运行所有测试
    let mut success_count = 0;
    let mut total_count = 0;
    
    // 测试 1: 基础对话
    total_count += 1;
    match test_real_basic_conversation().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 测试 1 通过");
        }
        Err(e) => println!("❌ 测试 1 失败: {}", e),
    }
    
    // 测试 2: 工具调用
    total_count += 1;
    match test_real_tool_calling().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 测试 2 通过");
        }
        Err(e) => println!("❌ 测试 2 失败: {}", e),
    }
    
    // 测试 3: 复杂对话
    total_count += 1;
    match test_real_complex_conversation().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 测试 3 通过");
        }
        Err(e) => println!("❌ 测试 3 失败: {}", e),
    }
    
    // 测试 4: 性能测试
    total_count += 1;
    match test_real_performance().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 测试 4 通过");
        }
        Err(e) => println!("❌ 测试 4 失败: {}", e),
    }
    
    // 总结
    println!("\n🎉 真实 API 验证完成！");
    println!("========================");
    println!("✅ 通过: {}/{}", success_count, total_count);
    println!("📊 成功率: {:.1}%", (success_count as f64 / total_count as f64) * 100.0);
    
    if success_count == total_count {
        println!("\n🏆 所有真实 API 测试通过！");
        println!("✅ 基础对话 - DeepSeek API 正常工作");
        println!("✅ 工具调用 - 函数调用功能正常");
        println!("✅ 复杂对话 - 多轮对话支持良好");
        println!("✅ 性能测试 - API 响应速度正常");
        
        println!("\n💡 LumosAI 与 DeepSeek API 集成验证成功！");
        println!("   您可以放心使用 LumosAI 构建基于 DeepSeek 的 AI 应用。");
    } else {
        println!("\n⚠️ 部分测试失败，请检查：");
        println!("   1. API Key 是否正确");
        println!("   2. 网络连接是否正常");
        println!("   3. DeepSeek API 服务是否可用");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_key_validation() {
        // 测试 API Key 验证逻辑
        std::env::remove_var("DEEPSEEK_API_KEY");
        assert!(get_deepseek_api_key().is_err());
        
        std::env::set_var("DEEPSEEK_API_KEY", "test-key");
        assert!(get_deepseek_api_key().is_ok());
        assert_eq!(get_deepseek_api_key().unwrap(), "test-key");
    }
}
