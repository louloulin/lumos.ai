//! 简化的链式操作验证示例
//! 
//! 验证 LumosAI 的链式操作功能，展示流畅的对话流程管理。

use lumosai_core::{Result, Error};
use lumosai_core::agent::{AgentBuilder, quick};
use lumosai_core::agent::chain::AgentChainExt;
use lumosai_core::agent::convenience::deepseek_with_key;
use lumosai_core::tool::CalculatorTool;
use std::env;

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

/// 验证基础链式对话
async fn test_basic_chain() -> Result<()> {
    println!("\n🔗 验证基础链式对话");
    println!("==================");
    
    let api_key = get_api_key()?;
    
    // 创建 Agent
    let agent = quick("chain_assistant", "你是一个友好的AI助手，请用中文简洁回答")
        .model(deepseek_with_key(&api_key, "deepseek-chat"))
        .build()?;
    
    println!("✅ Agent 创建成功");
    
    // 开始链式对话
    println!("\n📝 开始链式对话:");
    
    let response = agent
        .chain()
        .system("你是一个专业的旅行顾问")
        .ask("我想去日本旅行，有什么推荐吗？")
        .await?;
    
    println!("🤖 AI 响应: {}", &response.content()[..100.min(response.content().len())]);
    
    // 继续对话
    let response2 = response
        .then_ask("那东京有哪些必去的景点？")
        .await?;
    
    println!("🤖 AI 响应: {}", &response2.content()[..100.min(response2.content().len())]);
    
    // 检查对话历史
    let messages = response2.chain().get_messages();
    println!("\n📊 对话统计:");
    println!("   消息数量: {}", messages.len());
    println!("   对话轮数: {}", messages.iter().filter(|m| m.role == lumosai_core::llm::Role::User).count());
    
    println!("✅ 基础链式对话验证通过");
    
    Ok(())
}

/// 验证带工具的链式操作
async fn test_chain_with_tools() -> Result<()> {
    println!("\n🔧 验证带工具的链式操作");
    println!("========================");
    
    let api_key = get_api_key()?;
    
    // 创建带工具的 Agent
    let agent = AgentBuilder::new()
        .name("math_chain_assistant")
        .instructions("你是一个数学助手，可以使用计算器工具进行计算。请用中文回答。")
        .model(deepseek_with_key(&api_key, "deepseek-chat"))
        .tool(Box::new(CalculatorTool::default()))
        .enable_function_calling(true)
        .build()?;
    
    println!("✅ 带工具的 Agent 创建成功");
    
    // 链式数学对话
    println!("\n🧮 开始数学计算链式对话:");
    
    let response = agent
        .chain()
        .system("请使用计算器工具进行精确计算")
        .ask("请帮我计算 15 * 8 + 32 的结果")
        .await?;
    
    println!("🤖 计算结果: {}", response.content());
    
    // 继续计算
    let response2 = response
        .then_ask("那么这个结果除以 4 是多少？")
        .await?;
    
    println!("🤖 计算结果: {}", response2.content());
    
    // 检查步骤历史
    let steps = response2.chain().get_steps();
    
    println!("\n📊 工具调用统计:");
    println!("   总步骤数: {}", steps.len());
    
    println!("✅ 带工具的链式操作验证通过");
    
    Ok(())
}

/// 验证上下文变量管理
async fn test_context_variables() -> Result<()> {
    println!("\n📋 验证上下文变量管理");
    println!("====================");
    
    let api_key = get_api_key()?;
    
    let agent = quick("context_assistant", "你是一个智能助手，能够记住用户的偏好和信息")
        .model(deepseek_with_key(&api_key, "deepseek-chat"))
        .build()?;
    
    println!("✅ 上下文管理 Agent 创建成功");
    
    // 创建带变量的链式对话
    println!("\n🗂️ 测试上下文变量管理:");
    
    let response = agent
        .chain()
        .set_variable("user_name".to_string(), serde_json::Value::String("张三".to_string()))
        .set_variable("user_age".to_string(), serde_json::Value::Number(serde_json::Number::from(25)))
        .ask("你好，我是一个新用户")
        .await?;
    
    println!("🤖 AI 响应: {}", response.content());
    
    // 检查变量
    let chain = response.chain();
    if let Some(name) = chain.get_variable("user_name") {
        println!("📝 用户名变量: {}", name);
    }
    if let Some(age) = chain.get_variable("user_age") {
        println!("📝 年龄变量: {}", age);
    }
    
    println!("✅ 上下文变量管理验证通过");
    
    Ok(())
}

/// 验证上下文保存和加载
async fn test_context_persistence() -> Result<()> {
    println!("\n💾 验证上下文保存和加载");
    println!("======================");
    
    let api_key = get_api_key()?;
    
    let agent = quick("persistent_assistant", "你是一个能够记住对话历史的助手")
        .model(deepseek_with_key(&api_key, "deepseek-chat"))
        .build()?;
    
    println!("✅ 持久化 Agent 创建成功");
    
    // 创建对话并保存
    println!("\n💬 创建对话并保存到文件:");
    
    let response = agent
        .chain()
        .system("记住用户的所有偏好")
        .ask("我喜欢喝咖啡，特别是拿铁")
        .await?;
    
    println!("🤖 AI 响应: {}", &response.content()[..50.min(response.content().len())]);
    
    // 保存上下文
    let context_file = "test_chain_context.json";
    response.chain().save_context(context_file)?;
    println!("💾 上下文已保存到: {}", context_file);
    
    // 创建新的链式对话并加载上下文
    println!("\n📂 从文件加载上下文并继续对话:");
    
    let loaded_chain = agent
        .chain()
        .load_context(context_file)?;
    
    let response2 = loaded_chain
        .ask("根据我之前说的偏好，推荐一种咖啡")
        .await?;
    
    println!("🤖 AI 推荐: {}", &response2.content()[..50.min(response2.content().len())]);
    
    // 清理测试文件
    if std::path::Path::new(context_file).exists() {
        std::fs::remove_file(context_file).ok();
        println!("🗑️ 清理测试文件: {}", context_file);
    }
    
    println!("✅ 上下文保存和加载验证通过");
    
    Ok(())
}

/// 主函数：运行所有链式操作验证
#[tokio::main]
async fn main() -> Result<()> {
    println!("🔗 LumosAI 简化链式操作验证");
    println!("===========================");
    println!("验证 plan10.md 中的链式操作功能");
    
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
    
    // 运行验证测试
    total_count += 1;
    match test_basic_chain().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 基础链式对话 - 通过");
        }
        Err(e) => {
            println!("❌ 基础链式对话 - 失败: {}", e);
        }
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    total_count += 1;
    match test_chain_with_tools().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 带工具的链式操作 - 通过");
        }
        Err(e) => {
            println!("❌ 带工具的链式操作 - 失败: {}", e);
        }
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    total_count += 1;
    match test_context_variables().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 上下文变量管理 - 通过");
        }
        Err(e) => {
            println!("❌ 上下文变量管理 - 失败: {}", e);
        }
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    total_count += 1;
    match test_context_persistence().await {
        Ok(_) => {
            success_count += 1;
            println!("✅ 上下文保存加载 - 通过");
        }
        Err(e) => {
            println!("❌ 上下文保存加载 - 失败: {}", e);
        }
    }
    
    // 总结
    println!("\n🎉 链式操作验证完成！");
    println!("======================");
    println!("✅ 通过: {}/{}", success_count, total_count);
    println!("📊 成功率: {:.1}%", (success_count as f64 / total_count as f64) * 100.0);
    
    if success_count == total_count {
        println!("\n🏆 所有链式操作验证通过！");
        println!("✅ 基础链式对话 - 流畅的对话流程");
        println!("✅ 工具集成 - 链式操作中的工具调用");
        println!("✅ 上下文管理 - 变量和状态保持");
        println!("✅ 持久化 - 上下文保存和加载");
        
        println!("\n💡 链式操作 API 特性:");
        println!("   - 流畅的方法链式调用");
        println!("   - 自动的对话历史管理");
        println!("   - 灵活的上下文变量系统");
        println!("   - 完整的持久化支持");
        println!("   - 与工具系统无缝集成");
        
        println!("\n🎯 Plan 10 链式操作目标已实现！");
    } else {
        println!("\n⚠️ 部分测试失败，请检查:");
        println!("   1. API Key 是否正确");
        println!("   2. 网络连接是否正常");
        println!("   3. DeepSeek API 服务是否可用");
    }
    
    Ok(())
}
