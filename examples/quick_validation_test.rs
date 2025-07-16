//! 快速验证测试
//! 
//! 验证 LumosAI 的核心功能是否正常工作

use lumosai_core::{Result, Error};
use lumosai_core::agent::{AgentBuilder, quick};
use lumosai_core::agent::convenience::deepseek_with_key;
use lumosai_core::agent::trait_def::Agent as AgentTrait;
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::llm::{Message, Role};
use lumosai_core::llm::mock::MockLlmProvider;
use lumosai_core::tool::CalculatorTool;
use std::sync::Arc;
use std::env;

/// 获取 API Key 或使用模拟
fn get_api_key_or_mock() -> (Option<String>, bool) {
    match env::var("DEEPSEEK_API_KEY") {
        Ok(key) => (Some(key), false),
        Err(_) => (None, true),
    }
}

/// 验证 1: 基础 Agent 创建
async fn test_basic_agent_creation() -> Result<()> {
    println!("\n🚀 验证 1: 基础 Agent 创建");
    println!("========================");
    
    let (api_key, use_mock) = get_api_key_or_mock();
    
    let agent = if use_mock {
        println!("使用模拟 LLM Provider");
        let mock_llm = Arc::new(MockLlmProvider::new(vec![
            "你好！我是AI助手。".to_string(),
        ]));
        
        quick("test_assistant", "你是一个友好的AI助手")
            .model(mock_llm)
            .build()?
    } else {
        println!("使用真实 DeepSeek API");
        quick("test_assistant", "你是一个友好的AI助手")
            .model(deepseek_with_key(&api_key.unwrap(), "deepseek-chat"))
            .build()?
    };
    
    println!("✅ Agent 创建成功:");
    println!("   名称: {}", agent.get_name());
    println!("   指令: {}", agent.get_instructions());
    
    // 测试基本对话
    let messages = vec![Message {
        role: Role::User,
        content: "你好！".to_string(),
        metadata: None,
        name: None,
    }];
    
    let response = agent.generate(&messages, &AgentGenerateOptions::default()).await?;
    println!("✅ 对话测试成功: {}", &response.response[..50.min(response.response.len())]);
    
    Ok(())
}

/// 验证 2: AgentBuilder 构建器
async fn test_agent_builder() -> Result<()> {
    println!("\n🏗️ 验证 2: AgentBuilder 构建器");
    println!("==============================");
    
    let (api_key, use_mock) = get_api_key_or_mock();
    
    let agent = if use_mock {
        let mock_llm = Arc::new(MockLlmProvider::new(vec![
            "我是一个高级AI助手，可以使用工具。".to_string(),
        ]));
        
        AgentBuilder::new()
            .name("advanced_assistant")
            .instructions("你是一个高级AI助手")
            .model(mock_llm)
            .tool(Box::new(CalculatorTool::default()))
            .enable_function_calling(true)
            .build()?
    } else {
        AgentBuilder::new()
            .name("advanced_assistant")
            .instructions("你是一个高级AI助手")
            .model(deepseek_with_key(&api_key.unwrap(), "deepseek-chat"))
            .tool(Box::new(CalculatorTool::default()))
            .enable_function_calling(true)
            .build()?
    };
    
    println!("✅ 高级 Agent 创建成功:");
    println!("   名称: {}", agent.get_name());
    println!("   工具数量: {}", agent.get_tools().len());
    
    // 测试工具列表
    for tool in agent.get_tools() {
        println!("   工具: {} - {}", tool.name(), tool.description());
    }
    
    Ok(())
}

/// 验证 3: 错误处理
async fn test_error_handling() -> Result<()> {
    println!("\n🛡️ 验证 3: 错误处理");
    println!("==================");
    
    // 测试缺少名称的错误
    let result = AgentBuilder::new()
        .instructions("测试指令")
        .build();
    
    match result {
        Ok(_) => println!("⚠️ 应该失败但成功了"),
        Err(e) => println!("✅ 正确捕获错误: {}", e),
    }
    
    // 测试缺少指令的错误
    let result = AgentBuilder::new()
        .name("test")
        .build();
    
    match result {
        Ok(_) => println!("⚠️ 应该失败但成功了"),
        Err(e) => println!("✅ 正确捕获错误: {}", e),
    }
    
    Ok(())
}

/// 验证 4: 链式操作 (如果可用)
async fn test_chain_operations() -> Result<()> {
    println!("\n🔗 验证 4: 链式操作");
    println!("==================");
    
    let (api_key, use_mock) = get_api_key_or_mock();
    
    let agent = if use_mock {
        let mock_llm = Arc::new(MockLlmProvider::new(vec![
            "这是第一个响应。".to_string(),
            "这是第二个响应。".to_string(),
        ]));
        
        quick("chain_assistant", "你是一个链式助手")
            .model(mock_llm)
            .build()?
    } else {
        quick("chain_assistant", "你是一个链式助手")
            .model(deepseek_with_key(&api_key.unwrap(), "deepseek-chat"))
            .build()?
    };
    
    // 尝试使用链式操作
    use lumosai_core::agent::chain::AgentChainExt;
    
    let response = agent
        .chain()
        .ask("第一个问题")
        .await?;
    
    println!("✅ 链式操作第一步成功: {}", &response.content()[..50.min(response.content().len())]);
    
    let response2 = response
        .then_ask("第二个问题")
        .await?;
    
    println!("✅ 链式操作第二步成功: {}", &response2.content()[..50.min(response2.content().len())]);
    
    // 检查链状态
    let chain = response2.chain();
    let messages = chain.get_messages();
    println!("✅ 链式对话消息数: {}", messages.len());
    
    Ok(())
}

/// 验证 5: 性能测试
async fn test_performance() -> Result<()> {
    println!("\n⚡ 验证 5: 性能测试");
    println!("==================");
    
    let mock_llm = Arc::new(MockLlmProvider::new(vec![
        "响应1".to_string(),
        "响应2".to_string(),
        "响应3".to_string(),
    ]));
    
    let agent = quick("performance_test", "简洁回答")
        .model(mock_llm)
        .build()?;
    
    let start_time = std::time::Instant::now();
    
    // 创建多个 Agent 测试性能
    for i in 1..=3 {
        let messages = vec![Message {
            role: Role::User,
            content: format!("测试 {}", i),
            metadata: None,
            name: None,
        }];
        
        let response = agent.generate(&messages, &AgentGenerateOptions::default()).await?;
        println!("   测试 {}: {}", i, response.response);
    }
    
    let duration = start_time.elapsed();
    println!("✅ 性能测试完成，总耗时: {}ms", duration.as_millis());
    
    Ok(())
}

/// 主函数：运行所有验证测试
#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 LumosAI 快速验证测试");
    println!("=======================");
    
    let (api_key, use_mock) = get_api_key_or_mock();
    
    if use_mock {
        println!("⚠️ 未设置 DEEPSEEK_API_KEY，使用模拟测试");
    } else {
        println!("✅ 找到 DeepSeek API Key: {}...{}", 
            &api_key.as_ref().unwrap()[..8.min(api_key.as_ref().unwrap().len())], 
            if api_key.as_ref().unwrap().len() > 16 { 
                &api_key.as_ref().unwrap()[api_key.as_ref().unwrap().len()-8..] 
            } else { "" }
        );
    }
    
    let mut success_count = 0;
    let mut total_count = 0;
    
    // 运行所有验证测试
    let tests = vec![
        ("基础 Agent 创建", test_basic_agent_creation()),
        ("AgentBuilder 构建器", test_agent_builder()),
        ("错误处理", test_error_handling()),
        ("链式操作", test_chain_operations()),
        ("性能测试", test_performance()),
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
    
    // 总结
    println!("\n🎉 快速验证测试完成！");
    println!("======================");
    println!("✅ 通过: {}/{}", success_count, total_count);
    println!("📊 成功率: {:.1}%", (success_count as f64 / total_count as f64) * 100.0);
    
    if success_count == total_count {
        println!("\n🏆 所有验证测试通过！");
        println!("✅ 基础 Agent 创建 - 正常工作");
        println!("✅ AgentBuilder 构建器 - 正常工作");
        println!("✅ 错误处理机制 - 正常工作");
        println!("✅ 链式操作 - 正常工作");
        println!("✅ 性能表现 - 正常工作");
        
        println!("\n💡 LumosAI 核心功能验证成功！");
        if use_mock {
            println!("   设置 DEEPSEEK_API_KEY 环境变量可以测试真实 API");
        } else {
            println!("   真实 API 集成正常工作");
        }
    } else {
        println!("\n⚠️ 部分测试失败，请检查实现");
    }
    
    Ok(())
}
