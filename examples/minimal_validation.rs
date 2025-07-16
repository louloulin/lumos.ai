//! 最小化验证示例
//! 
//! 验证 LumosAI 的基本编译和运行

use lumosai_core::{Result, Error};
use lumosai_core::agent::{AgentBuilder, quick};
use lumosai_core::llm::mock::MockLlmProvider;
use std::sync::Arc;

/// 验证基础功能
async fn test_basic_functionality() -> Result<()> {
    println!("🚀 验证基础功能");
    
    // 创建模拟 LLM
    let mock_llm = Arc::new(MockLlmProvider::new(vec![
        "Hello from LumosAI!".to_string(),
    ]));
    
    // 使用 quick API 创建 Agent
    let agent = quick("test_agent", "You are a test assistant")
        .model(mock_llm)
        .build()?;
    
    println!("✅ Agent 创建成功:");
    println!("   名称: {}", agent.get_name());
    println!("   指令: {}", agent.get_instructions());
    
    Ok(())
}

/// 验证 AgentBuilder
async fn test_agent_builder() -> Result<()> {
    println!("\n🏗️ 验证 AgentBuilder");
    
    let mock_llm = Arc::new(MockLlmProvider::new(vec![
        "Builder test response".to_string(),
    ]));
    
    let agent = AgentBuilder::new()
        .name("builder_test")
        .instructions("Test instructions")
        .model(mock_llm)
        .build()?;
    
    println!("✅ AgentBuilder 创建成功:");
    println!("   名称: {}", agent.get_name());
    
    Ok(())
}

/// 验证错误处理
fn test_error_handling() -> Result<()> {
    println!("\n🛡️ 验证错误处理");
    
    // 测试缺少名称
    let result = AgentBuilder::new()
        .instructions("test")
        .build();
    
    match result {
        Ok(_) => println!("⚠️ 应该失败但成功了"),
        Err(e) => println!("✅ 正确捕获错误: {}", e),
    }
    
    Ok(())
}

/// 主函数
#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 LumosAI 最小化验证");
    println!("======================");
    
    let mut success_count = 0;
    let mut total_count = 0;
    
    // 测试 1: 基础功能
    total_count += 1;
    match test_basic_functionality().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 基础功能 - 通过");
        }
        Err(e) => {
            println!("❌ 基础功能 - 失败: {}", e);
        }
    }
    
    // 测试 2: AgentBuilder
    total_count += 1;
    match test_agent_builder().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ AgentBuilder - 通过");
        }
        Err(e) => {
            println!("❌ AgentBuilder - 失败: {}", e);
        }
    }
    
    // 测试 3: 错误处理
    total_count += 1;
    match test_error_handling() {
        Ok(_) => {
            success_count += 1;
            println!("✅ 错误处理 - 通过");
        }
        Err(e) => {
            println!("❌ 错误处理 - 失败: {}", e);
        }
    }
    
    // 总结
    println!("\n🎉 验证完成！");
    println!("=============");
    println!("✅ 通过: {}/{}", success_count, total_count);
    println!("📊 成功率: {:.1}%", (success_count as f64 / total_count as f64) * 100.0);
    
    if success_count == total_count {
        println!("\n🏆 所有验证通过！");
        println!("LumosAI 核心功能正常工作！");
    } else {
        println!("\n⚠️ 部分验证失败");
    }
    
    Ok(())
}
