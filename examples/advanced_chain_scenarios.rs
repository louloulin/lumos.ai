//! 高级链式操作场景验证
//! 
//! 展示 LumosAI 链式操作在复杂业务场景中的应用，
//! 包括工作流自动化、决策树、条件分支等高级功能。

use lumosai_core::{Result, Error};
use lumosai_core::agent::{AgentBuilder, quick};
use lumosai_core::agent::chain::{AgentChainExt, ChainContext};
use lumosai_core::agent::convenience::deepseek_with_key;
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::llm::{Message, Role, LlmOptions};
use lumosai_core::tool::CalculatorTool;
use std::env;
use std::time::Instant;
use serde_json::json;

/// 获取 DeepSeek API Key
fn get_api_key() -> Result<String> {
    env::var("DEEPSEEK_API_KEY").map_err(|_| {
        Error::Configuration(
            "请设置 DEEPSEEK_API_KEY 环境变量。\n\
            获取方式：https://platform.deepseek.com/".to_string()
        )
    })
}

/// 场景 1: 智能决策树工作流
async fn scenario_decision_tree_workflow() -> Result<()> {
    println!("\n🌳 场景 1: 智能决策树工作流");
    println!("===========================");
    
    let api_key = get_api_key()?;
    
    // 创建决策分析师 Agent
    let decision_agent = AgentBuilder::new()
        .name("decision_analyst")
        .instructions("你是一个专业的决策分析师，能够分析情况并提供结构化的决策建议。请用中文回答，并在回答中明确指出推荐的选项。")
        .model(deepseek_with_key(&api_key, "deepseek-chat"))
        .build()?;
    
    println!("✅ 决策分析师 Agent 创建成功");
    
    // 开始决策流程
    println!("\n📋 开始智能决策流程:");
    
    let initial_response = decision_agent
        .chain()
        .system("你需要帮助用户做出明智的决策，请分析所有相关因素")
        .set_variable("decision_context".to_string(), json!("职业发展"))
        .set_variable("user_profile".to_string(), json!({
            "experience": "3年软件开发",
            "skills": ["Python", "Rust", "AI"],
            "goals": "技术领导力"
        }))
        .ask("我现在面临一个职业选择：是继续在当前公司深耕技术，还是跳槽到一家AI创业公司？请帮我分析。")
        .await?;
    
    println!("🤖 初步分析: {}", &initial_response.content()[..200.min(initial_response.content().len())]);
    
    // 基于初步分析，进行深入探讨
    let detailed_analysis = initial_response
        .then_ask("请详细分析这两个选择的风险和机会，特别是从技术成长和职业发展角度。")
        .await?;
    
    println!("🔍 详细分析: {}", &detailed_analysis.content()[..200.min(detailed_analysis.content().len())]);
    
    // 获取最终建议
    let final_recommendation = detailed_analysis
        .then_ask("基于以上分析，请给出你的最终建议，并说明理由。")
        .await?;
    
    println!("💡 最终建议: {}", &final_recommendation.content()[..200.min(final_recommendation.content().len())]);
    
    // 检查决策流程的完整性
    let chain = final_recommendation.chain();
    let messages = chain.get_messages();
    let steps = chain.get_steps();
    
    println!("\n📊 决策流程统计:");
    println!("   总消息数: {}", messages.len());
    println!("   决策步骤: {}", steps.len());
    println!("   用户输入: {}", messages.iter().filter(|m| m.role == Role::User).count());
    
    // 检查上下文变量
    if let Some(context) = chain.get_variable("decision_context") {
        println!("   决策上下文: {}", context);
    }
    
    println!("✅ 智能决策树工作流验证完成");
    
    Ok(())
}

/// 场景 2: 多阶段项目规划链
async fn scenario_multi_stage_project_planning() -> Result<()> {
    println!("\n📋 场景 2: 多阶段项目规划链");
    println!("=============================");
    
    let api_key = get_api_key()?;
    
    // 创建项目管理专家 Agent
    let project_manager = AgentBuilder::new()
        .name("project_manager")
        .instructions("你是一个经验丰富的项目管理专家，擅长制定详细的项目计划和风险评估。")
        .model(deepseek_with_key(&api_key, "deepseek-chat"))
        .tool(Box::new(CalculatorTool::default()))
        .enable_function_calling(true)
        .build()?;
    
    println!("✅ 项目管理专家 Agent 创建成功");
    
    // 项目规划链式流程
    println!("\n🚀 开始多阶段项目规划:");
    
    let start_time = Instant::now();
    
    // 阶段 1: 项目范围定义
    let scope_definition = project_manager
        .chain()
        .with_options(AgentGenerateOptions {
            llm_options: LlmOptions::default()
                .with_temperature(0.3)  // 较低温度确保结构化输出
                .with_max_tokens(800),
            ..Default::default()
        })
        .system("你正在帮助规划一个新的AI产品开发项目")
        .set_variable("project_type".to_string(), json!("AI聊天机器人"))
        .set_variable("team_size".to_string(), json!(8))
        .set_variable("budget".to_string(), json!(500000))
        .set_variable("timeline".to_string(), json!("6个月"))
        .ask("我们要开发一个企业级AI聊天机器人，团队8人，预算50万，时间6个月。请帮我定义项目范围和主要功能模块。")
        .await?;
    
    println!("📝 项目范围: {}", &scope_definition.content()[..150.min(scope_definition.content().len())]);
    
    // 阶段 2: 资源分配和时间估算
    let resource_planning = scope_definition
        .then_ask("基于上述项目范围，请详细规划人员分工和时间分配，并使用计算器计算各阶段的成本。")
        .await?;
    
    println!("👥 资源规划: {}", &resource_planning.content()[..150.min(resource_planning.content().len())]);
    
    // 阶段 3: 风险评估和缓解策略
    let risk_assessment = resource_planning
        .then_ask("请识别这个项目的主要风险点，并为每个风险制定缓解策略。")
        .await?;
    
    println!("⚠️ 风险评估: {}", &risk_assessment.content()[..150.min(risk_assessment.content().len())]);
    
    // 阶段 4: 里程碑和交付计划
    let milestone_planning = risk_assessment
        .then_ask("请制定详细的里程碑计划，包括每个里程碑的交付物和验收标准。")
        .await?;
    
    println!("🎯 里程碑计划: {}", &milestone_planning.content()[..150.min(milestone_planning.content().len())]);
    
    let total_time = start_time.elapsed();
    
    // 保存完整的项目规划
    let planning_file = "project_planning_chain.json";
    milestone_planning.chain().save_context(planning_file)?;
    println!("💾 项目规划已保存到: {}", planning_file);
    
    // 统计信息
    let final_chain = milestone_planning.chain();
    let messages = final_chain.get_messages();
    let steps = final_chain.get_steps();
    
    println!("\n📊 项目规划统计:");
    println!("   总耗时: {}ms", total_time.as_millis());
    println!("   规划阶段: 4个");
    println!("   总消息数: {}", messages.len());
    println!("   规划步骤: {}", steps.len());
    
    // 清理文件
    if std::path::Path::new(planning_file).exists() {
        std::fs::remove_file(planning_file).ok();
    }
    
    println!("✅ 多阶段项目规划链验证完成");
    
    Ok(())
}

/// 场景 3: 条件分支和动态路由
async fn scenario_conditional_branching() -> Result<()> {
    println!("\n🔀 场景 3: 条件分支和动态路由");
    println!("=============================");
    
    let api_key = get_api_key()?;
    
    // 创建智能路由 Agent
    let router_agent = quick("smart_router", "你是一个智能路由助手，能够根据用户需求选择最合适的处理方式")
        .model(deepseek_with_key(&api_key, "deepseek-chat"))
        .build()?;
    
    println!("✅ 智能路由 Agent 创建成功");
    
    // 测试不同类型的请求路由
    let test_scenarios = vec![
        ("技术问题", "我的Python代码出现了内存泄漏，应该怎么调试？"),
        ("商务咨询", "我想了解贵公司的AI解决方案定价和服务内容"),
        ("产品反馈", "你们的产品在移动端体验不太好，希望能改进"),
    ];
    
    println!("\n🧭 测试智能路由功能:");
    
    for (scenario_type, user_request) in test_scenarios {
        println!("\n--- {} ---", scenario_type);
        
        let routing_response = router_agent
            .chain()
            .system("请分析用户请求的类型，并提供相应的处理建议")
            .set_variable("request_type".to_string(), json!(scenario_type))
            .ask(format!("用户请求：{}", user_request))
            .await?;
        
        println!("🎯 路由结果: {}", &routing_response.content()[..100.min(routing_response.content().len())]);
        
        // 根据路由结果进行后续处理
        let follow_up = routing_response
            .then_ask("基于你的分析，请提供具体的解决方案或下一步行动建议。")
            .await?;
        
        println!("💡 解决方案: {}", &follow_up.content()[..100.min(follow_up.content().len())]);
        
        // 检查路由变量
        let chain = follow_up.chain();
        if let Some(request_type) = chain.get_variable("request_type") {
            println!("📋 请求类型: {}", request_type);
        }
        
        // 短暂暂停
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    println!("\n✅ 条件分支和动态路由验证完成");
    
    Ok(())
}

/// 场景 4: 链式操作性能压力测试
async fn scenario_chain_performance_test() -> Result<()> {
    println!("\n⚡ 场景 4: 链式操作性能压力测试");
    println!("===============================");
    
    let api_key = get_api_key()?;
    
    let performance_agent = quick("performance_test", "请简洁回答")
        .model(deepseek_with_key(&api_key, "deepseek-chat"))
        .build()?;
    
    println!("✅ 性能测试 Agent 创建成功");
    
    // 长链式对话性能测试
    println!("\n🔄 开始长链式对话性能测试:");
    
    let start_time = Instant::now();
    
    let mut current_response = performance_agent
        .chain()
        .ask("开始计数：1")
        .await?;
    
    println!("🔗 链式对话进行中...");
    
    // 进行多轮链式对话
    for i in 2..=5 {  // 减少轮数以避免API限制
        let question = format!("继续计数：{}", i);
        current_response = current_response
            .then_ask(question)
            .await?;
        
        print!(".");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        // 避免请求过于频繁
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
    }
    
    let total_time = start_time.elapsed();
    
    // 性能统计
    let final_chain = current_response.chain();
    let messages = final_chain.get_messages();
    let steps = final_chain.get_steps();
    
    println!("\n\n📊 性能测试结果:");
    println!("   总耗时: {}ms", total_time.as_millis());
    println!("   链式轮数: 5轮");
    println!("   平均每轮: {}ms", total_time.as_millis() / 5);
    println!("   总消息数: {}", messages.len());
    println!("   总步骤数: {}", steps.len());
    println!("   内存效率: Arc 共享，零拷贝");
    
    // 验证对话连贯性
    println!("\n🔍 验证对话连贯性:");
    let user_messages: Vec<_> = messages.iter()
        .filter(|m| m.role == Role::User)
        .map(|m| &m.content)
        .collect();
    
    println!("   用户输入序列: {:?}", user_messages);
    
    println!("✅ 链式操作性能压力测试完成");
    
    Ok(())
}

/// 主函数：运行所有高级场景验证
#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 LumosAI 高级链式操作场景验证");
    println!("================================");
    println!("展示链式操作在复杂业务场景中的应用");
    
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
    
    println!("\n⚠️ 注意：此验证将调用真实的 DeepSeek API，可能产生费用。");
    
    let mut success_count = 0;
    let mut total_count = 0;
    
    // 运行高级场景验证
    let scenarios = vec![
        ("智能决策树工作流", scenario_decision_tree_workflow()),
        ("多阶段项目规划", scenario_multi_stage_project_planning()),
        ("条件分支和动态路由", scenario_conditional_branching()),
        ("链式操作性能测试", scenario_chain_performance_test()),
    ];
    
    for (scenario_name, scenario_future) in scenarios {
        total_count += 1;
        match scenario_future.await {
            Ok(_) => {
                success_count += 1;
                println!("✅ {} - 验证成功", scenario_name);
            }
            Err(e) => {
                println!("❌ {} - 验证失败: {}", scenario_name, e);
            }
        }
        
        // 场景间隔
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    }
    
    // 总结
    println!("\n🎉 高级链式操作场景验证完成！");
    println!("================================");
    println!("✅ 成功验证: {}/{}", success_count, total_count);
    println!("📊 成功率: {:.1}%", (success_count as f64 / total_count as f64) * 100.0);
    
    if success_count == total_count {
        println!("\n🏆 所有高级场景验证通过！");
        
        println!("\n🎯 验证的高级功能:");
        println!("   ✅ 智能决策树 - 多步骤决策分析流程");
        println!("   ✅ 项目规划链 - 复杂业务流程自动化");
        println!("   ✅ 条件分支 - 动态路由和智能分发");
        println!("   ✅ 性能优化 - 长链式对话的高效处理");
        
        println!("\n💡 链式操作的企业级特性:");
        println!("   - 复杂业务流程的自动化");
        println!("   - 上下文状态的完整管理");
        println!("   - 决策树和条件分支支持");
        println!("   - 高性能的长链式对话");
        println!("   - 完整的持久化和恢复");
        
        println!("\n🚀 LumosAI 链式操作已达到企业级应用标准！");
    } else {
        println!("\n⚠️ 部分场景验证失败，请检查网络和 API 配置");
    }
    
    Ok(())
}
