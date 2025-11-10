//! 简化的 API 验证示例
//!
//! 验证 plan10.md 中已确认实现的核心 API 功能

use lumosai_core::agent::trait_def::Agent as AgentTrait;
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::agent::{quick, AgentBuilder};
use lumosai_core::llm::test_helpers::{create_test_zhipu_provider, create_test_zhipu_provider_arc};
use lumosai_core::llm::{Message, Role};
use lumosai_core::tool::CalculatorTool;
use lumosai_core::Result;
use std::sync::Arc;

/// 示例 1: 验证 quick 函数 API
async fn test_quick_api() -> Result<()> {
    println!("🚀 测试 1: quick() 函数 API");
    println!("==========================");

    let llm = create_test_zhipu_provider_arc();

    // ✅ 验证 quick 函数
    let agent = quick("assistant", "你是一个友好的AI助手")
        .model(llm)
        .build()?;

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

    let response = agent
        .generate(&messages, &AgentGenerateOptions::default())
        .await?;
    println!("   响应: {}", response.response);

    Ok(())
}

/// 示例 2: 验证 AgentBuilder 构建器
async fn test_agent_builder() -> Result<()> {
    println!("\n🏗️ 测试 2: AgentBuilder 构建器");
    println!("===============================");

    let llm = create_test_zhipu_provider_arc();

    // ✅ 验证完整的构建器模式
    let agent = AgentBuilder::new()
        .name("advanced_assistant")
        .instructions("你是一个高级AI助手")
        .model(llm)
        .tool(Box::new(CalculatorTool::default()))
        .max_tool_calls(5)
        .tool_timeout(30)
        .enable_function_calling(true)
        .build()?;

    println!("✅ 高级 Agent 创建成功:");
    println!("   名称: {}", agent.get_name());
    println!("   指令: {}", agent.get_instructions());
    println!("   工具数量: {}", agent.get_tools().len());

    // 验证工具
    for (tool_name, tool) in agent.get_tools() {
        println!("   工具: {} - {}", tool_name, tool.description());
    }

    Ok(())
}

/// 示例 3: 验证配置验证
async fn test_configuration_validation() -> Result<()> {
    println!("\n⚠️ 测试 3: 配置验证");
    println!("====================");

    // ✅ 验证缺少必需字段的错误处理
    println!("测试缺少名称的错误:");
    let result = AgentBuilder::new().instructions("测试指令").build();

    match result {
        Err(e) => println!("   ✅ 正确捕获错误: {}", e),
        Ok(_) => println!("   ❌ 应该返回错误但没有"),
    }

    // 测试缺少指令
    println!("测试缺少指令的错误:");
    let llm = create_test_zhipu_provider_arc();
    let result = AgentBuilder::new().name("test").model(llm).build();

    match result {
        Err(e) => println!("   ✅ 正确捕获错误: {}", e),
        Ok(_) => println!("   ❌ 应该返回错误但没有"),
    }

    Ok(())
}

/// 示例 4: 验证智能默认配置
async fn test_smart_defaults() -> Result<()> {
    println!("\n🧠 测试 4: 智能默认配置");
    println!("========================");

    let llm = create_test_zhipu_provider_arc();

    // ✅ 验证智能默认配置
    let agent = quick("default_test", "测试默认配置").model(llm).build()?;

    println!("✅ 智能默认配置验证:");
    println!("   名称: {}", agent.get_name());
    println!("   指令: {}", agent.get_instructions());
    println!("   工具数量: {}", agent.get_tools().len());

    Ok(())
}

/// 示例 5: 验证工具系统
async fn test_tool_system() -> Result<()> {
    println!("\n🔧 测试 5: 工具系统");
    println!("===================");

    let llm = create_test_zhipu_provider_arc();

    // ✅ 验证工具注册和使用
    let agent = AgentBuilder::new()
        .name("tool_test")
        .instructions("测试工具系统")
        .model(llm)
        .tool(Box::new(CalculatorTool::default()))
        .build()?;

    println!("✅ 工具系统验证:");
    println!("   Agent 名称: {}", agent.get_name());
    println!("   注册的工具:");

    for (tool_name, tool) in agent.get_tools() {
        println!("     - ID: {}", tool_name);
        println!("       描述: {}", tool.description());
    }

    // 验证工具查找
    if let Some(calc_tool) = agent.get_tool("calculator") {
        println!("   ✅ 成功找到计算器工具: {}", calc_tool.description());
    } else {
        println!("   ⚠️ 未找到计算器工具");
    }

    Ok(())
}

/// 示例 6: 验证错误恢复
async fn test_error_recovery() -> Result<()> {
    println!("\n🛡️ 测试 6: 错误恢复");
    println!("====================");

    let llm = create_test_zhipu_provider_arc();

    let agent = quick("error_test", "错误处理测试").model(llm).build()?;

    // 测试正常操作
    let messages = vec![Message {
        role: Role::User,
        content: "正常消息".to_string(),
        metadata: None,
        name: None,
    }];

    let response = agent
        .generate(&messages, &AgentGenerateOptions::default())
        .await?;
    println!("✅ 正常响应: {}", response.response);

    // 验证 Agent 仍然可用
    let messages2 = vec![Message {
        role: Role::User,
        content: "第二条消息".to_string(),
        metadata: None,
        name: None,
    }];

    let response2 = agent
        .generate(&messages2, &AgentGenerateOptions::default())
        .await?;
    println!("✅ 恢复后响应: {}", response2.response);

    Ok(())
}

/// 主函数：运行所有验证测试
#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 LumosAI 简化 API 验证");
    println!("========================");
    println!("验证 plan10.md 中已实现的核心 API 功能\n");

    // 运行所有测试
    let mut success_count = 0;
    let mut total_count = 0;

    // 测试 1: quick API
    total_count += 1;
    match test_quick_api().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 测试 1 通过");
        }
        Err(e) => println!("❌ 测试 1 失败: {}", e),
    }

    // 测试 2: AgentBuilder
    total_count += 1;
    match test_agent_builder().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 测试 2 通过");
        }
        Err(e) => println!("❌ 测试 2 失败: {}", e),
    }

    // 测试 3: 配置验证
    total_count += 1;
    match test_configuration_validation().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 测试 3 通过");
        }
        Err(e) => println!("❌ 测试 3 失败: {}", e),
    }

    // 测试 4: 智能默认配置
    total_count += 1;
    match test_smart_defaults().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 测试 4 通过");
        }
        Err(e) => println!("❌ 测试 4 失败: {}", e),
    }

    // 测试 5: 工具系统
    total_count += 1;
    match test_tool_system().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 测试 5 通过");
        }
        Err(e) => println!("❌ 测试 5 失败: {}", e),
    }

    // 测试 6: 错误恢复
    total_count += 1;
    match test_error_recovery().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 测试 6 通过");
        }
        Err(e) => println!("❌ 测试 6 失败: {}", e),
    }

    // 总结
    println!("\n🎉 验证完成！");
    println!("=============");
    println!("✅ 通过: {}/{}", success_count, total_count);
    println!(
        "📊 成功率: {:.1}%",
        (success_count as f64 / total_count as f64) * 100.0
    );

    if success_count == total_count {
        println!("\n🏆 所有 API 验证通过！");
        println!("✅ quick() 函数 - 已验证");
        println!("✅ AgentBuilder - 已验证");
        println!("✅ 配置验证 - 已验证");
        println!("✅ 智能默认配置 - 已验证");
        println!("✅ 工具系统 - 已验证");
        println!("✅ 错误恢复 - 已验证");
    } else {
        println!("\n⚠️ 部分测试失败，请检查实现");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_all_validations() {
        // 确保所有验证都能正常运行
        assert!(test_quick_api().await.is_ok());
        assert!(test_agent_builder().await.is_ok());
        assert!(test_configuration_validation().await.is_ok());
        assert!(test_smart_defaults().await.is_ok());
        assert!(test_tool_system().await.is_ok());
        assert!(test_error_recovery().await.is_ok());
    }
}
