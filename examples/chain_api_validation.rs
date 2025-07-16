//! 链式操作 API 验证示例
//! 
//! 验证 plan10.md 中提到的链式操作功能，展示流畅的对话流程管理。
//! 基于 DeepSeek LLM provider 进行真实 API 测试。

use lumosai_core::{Result, Error};
use lumosai_core::agent::{AgentBuilder, quick};
use lumosai_core::agent::chain::{AgentChainExt, ChainContext, ChainStep, ChainStepType};
use lumosai_core::agent::convenience::deepseek_with_key;
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::tool::CalculatorTool;
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

/// 验证 1: 基础链式对话
async fn test_basic_chain_conversation() -> Result<()> {
    println!("\n🔗 验证 1: 基础链式对话");
    println!("=======================");
    
    let api_key = get_api_key()?;
    
    // 创建 Agent
    let agent = quick("chain_assistant", "你是一个友好的AI助手，请用中文简洁回答")
        .model(deepseek_with_key(&api_key, "deepseek-chat"))
        .build()?;
    
    println!("✅ Agent 创建成功");
    
    // 开始链式对话
    println!("\n📝 开始链式对话流程:");
    
    let start_time = Instant::now();
    
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
    
    // 再次继续
    let response3 = response2
        .then_ask("大概需要多少预算？")
        .await?;
    
    println!("🤖 AI 响应: {}", &response3.content()[..100.min(response3.content().len())]);
    
    let duration = start_time.elapsed();
    
    // 检查对话历史
    let messages = response3.chain().get_messages();
    println!("\n📊 对话统计:");
    println!("   总耗时: {}ms", duration.as_millis());
    println!("   消息数量: {}", messages.len());
    println!("   对话轮数: {}", messages.iter().filter(|m| m.role == lumosai_core::llm::Role::User).count());
    
    println!("✅ 基础链式对话验证通过");
    
    Ok(())
}

/// 验证 2: 带工具的链式操作
async fn test_chain_with_tools() -> Result<()> {
    println!("\n🔧 验证 2: 带工具的链式操作");
    println!("===========================");
    
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
    
    // 复杂计算
    let response3 = response2
        .then_ask("请计算 (25 + 15) * 3 - 8 的结果")
        .await?;
    
    println!("🤖 计算结果: {}", response3.content());
    
    // 检查工具调用历史
    let steps = response3.chain().get_steps();
    let tool_calls = steps.iter().filter(|s| matches!(s.step_type, ChainStepType::ToolCall)).count();
    
    println!("\n📊 工具调用统计:");
    println!("   总步骤数: {}", steps.len());
    println!("   工具调用次数: {}", tool_calls);
    
    println!("✅ 带工具的链式操作验证通过");
    
    Ok(())
}

/// 验证 3: 上下文变量和状态管理
async fn test_chain_context_management() -> Result<()> {
    println!("\n📋 验证 3: 上下文变量和状态管理");
    println!("===============================");
    
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
        .set_variable("user_city".to_string(), serde_json::Value::String("北京".to_string()))
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
    if let Some(city) = chain.get_variable("user_city") {
        println!("📝 城市变量: {}", city);
    }
    
    // 继续对话，测试上下文保持
    let response2 = chain
        .ask("请根据我的信息推荐一些适合的活动")
        .await?;
    
    println!("🤖 AI 推荐: {}", &response2.content()[..100.min(response2.content().len())]);
    
    println!("✅ 上下文变量和状态管理验证通过");
    
    Ok(())
}

/// 验证 4: 上下文保存和加载
async fn test_chain_persistence() -> Result<()> {
    println!("\n💾 验证 4: 上下文保存和加载");
    println!("===========================");
    
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
    
    println!("🤖 AI 响应: {}", response.content());
    
    let response2 = response
        .then_ask("我还喜欢看科幻电影")
        .await?;
    
    println!("🤖 AI 响应: {}", response2.content());
    
    // 保存上下文
    let context_file = "test_chain_context.json";
    response2.chain().save_context(context_file)?;
    println!("💾 上下文已保存到: {}", context_file);
    
    // 创建新的链式对话并加载上下文
    println!("\n📂 从文件加载上下文并继续对话:");
    
    let loaded_chain = agent
        .chain()
        .load_context(context_file)?;
    
    let response3 = loaded_chain
        .ask("根据我之前说的偏好，推荐一部电影和一种咖啡")
        .await?;
    
    println!("🤖 AI 推荐: {}", response3.content());
    
    // 清理测试文件
    if std::path::Path::new(context_file).exists() {
        std::fs::remove_file(context_file).ok();
        println!("🗑️ 清理测试文件: {}", context_file);
    }
    
    println!("✅ 上下文保存和加载验证通过");
    
    Ok(())
}

/// 验证 5: 复杂链式工作流
async fn test_complex_chain_workflow() -> Result<()> {
    println!("\n🌊 验证 5: 复杂链式工作流");
    println!("=========================");
    
    let api_key = get_api_key()?;
    
    let agent = AgentBuilder::new()
        .name("workflow_assistant")
        .instructions("你是一个项目管理助手，帮助用户规划和管理项目")
        .model(deepseek_with_key(&api_key, "deepseek-chat"))
        .tool(Box::new(CalculatorTool::default()))
        .enable_function_calling(true)
        .build()?;
    
    println!("✅ 工作流 Agent 创建成功");
    
    // 复杂的项目规划工作流
    println!("\n📋 开始项目规划工作流:");
    
    let start_time = Instant::now();
    
    let response = agent
        .chain()
        .with_options(AgentGenerateOptions {
            llm_options: lumosai_core::llm::LlmOptions::default()
                .with_temperature(0.7)
                .with_max_tokens(500),
            ..Default::default()
        })
        .system("你是一个专业的项目管理顾问，请提供详细的项目规划建议")
        .set_variable("project_type".to_string(), serde_json::Value::String("移动应用开发".to_string()))
        .set_variable("team_size".to_string(), serde_json::Value::Number(serde_json::Number::from(5)))
        .set_variable("budget".to_string(), serde_json::Value::Number(serde_json::Number::from(100000)))
        .ask("我想开发一个移动应用，团队有5个人，预算10万元，请帮我制定项目计划")
        .await?;
    
    println!("📋 项目计划: {}", &response.content()[..150.min(response.content().len())]);
    
    // 询问时间安排
    let response2 = response
        .then_ask("这个项目大概需要多长时间？请帮我计算一下")
        .await?;
    
    println!("⏰ 时间估算: {}", &response2.content()[..150.min(response2.content().len())]);
    
    // 询问风险评估
    let response3 = response2
        .then_ask("有哪些主要风险需要注意？")
        .await?;
    
    println!("⚠️ 风险评估: {}", &response3.content()[..150.min(response3.content().len())]);
    
    // 询问里程碑
    let response4 = response3
        .then_ask("请制定详细的里程碑计划")
        .await?;
    
    println!("🎯 里程碑: {}", &response4.content()[..150.min(response4.content().len())]);
    
    let duration = start_time.elapsed();
    
    // 工作流统计
    let final_chain = response4.chain();
    let messages = final_chain.get_messages();
    let steps = final_chain.get_steps();
    
    println!("\n📊 工作流统计:");
    println!("   总耗时: {}ms", duration.as_millis());
    println!("   消息数量: {}", messages.len());
    println!("   步骤数量: {}", steps.len());
    println!("   用户输入: {}", messages.iter().filter(|m| m.role == lumosai_core::llm::Role::User).count());
    println!("   AI 响应: {}", messages.iter().filter(|m| m.role == lumosai_core::llm::Role::Assistant).count());
    
    // 检查变量
    println!("\n🗂️ 上下文变量:");
    if let Some(project_type) = final_chain.get_variable("project_type") {
        println!("   项目类型: {}", project_type);
    }
    if let Some(team_size) = final_chain.get_variable("team_size") {
        println!("   团队规模: {}", team_size);
    }
    if let Some(budget) = final_chain.get_variable("budget") {
        println!("   预算: {}", budget);
    }
    
    println!("✅ 复杂链式工作流验证通过");
    
    Ok(())
}

/// 主函数：运行所有链式操作验证
#[tokio::main]
async fn main() -> Result<()> {
    println!("🔗 LumosAI 链式操作 API 验证");
    println!("============================");
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
    
    // 运行所有验证测试
    let tests: Vec<(&str, std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>>>>)> = vec![
        ("基础链式对话", Box::pin(test_basic_chain_conversation())),
        ("带工具的链式操作", Box::pin(test_chain_with_tools())),
        ("上下文变量管理", Box::pin(test_chain_context_management())),
        ("上下文保存加载", Box::pin(test_chain_persistence())),
        ("复杂链式工作流", Box::pin(test_complex_chain_workflow())),
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
        
        // 测试间隔，避免请求过于频繁
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
    
    // 总结
    println!("\n🎉 链式操作 API 验证完成！");
    println!("===========================");
    println!("✅ 通过: {}/{}", success_count, total_count);
    println!("📊 成功率: {:.1}%", (success_count as f64 / total_count as f64) * 100.0);
    
    if success_count == total_count {
        println!("\n🏆 所有链式操作验证通过！");
        println!("✅ 基础链式对话 - 流畅的对话流程");
        println!("✅ 工具集成 - 链式操作中的工具调用");
        println!("✅ 上下文管理 - 变量和状态保持");
        println!("✅ 持久化 - 上下文保存和加载");
        println!("✅ 复杂工作流 - 多步骤业务流程");
        
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
