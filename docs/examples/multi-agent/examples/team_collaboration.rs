use lumosai::prelude::SimpleAgent;
use std::collections::HashMap;
use futures::future::join_all;

/// 团队协作示例
/// 展示多个 Agent 如何协作解决复杂问题

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    println!("🤝 团队协作示例");
    println!("场景：为新产品制定营销策略\n");

    // 创建专业团队
    let mut team = HashMap::new();
    
    // 市场分析师
    let market_analyst = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是一名资深市场分析师，擅长市场调研、竞争分析和趋势预测。\
         请提供专业的市场洞察和数据分析。"
    ).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
    team.insert("market_analyst", market_analyst);

    // 产品经理
    let product_manager = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是一名经验丰富的产品经理，负责产品规划、功能定义和用户体验设计。\
         请从产品角度提供专业建议。"
    ).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
    team.insert("product_manager", product_manager);

    // 营销专家
    let marketing_expert = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是一名创意营销专家，擅长品牌推广、内容营销和用户获取策略。\
         请提供创新的营销方案和执行建议。"
    ).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
    team.insert("marketing_expert", marketing_expert);

    let product_description = "一款基于 AI 的智能代码审查工具，\
        能够自动检测代码质量问题、安全漏洞和性能优化建议";

    println!("📋 产品描述：{}\n", product_description);

    // 第一阶段：各自分析
    println!("🔍 第一阶段：专业分析");
    
    let market_analysis = team["market_analyst"].chat(&format!(
        "请分析这个产品的市场前景：{}。包括目标市场、竞争对手和市场机会。",
        product_description
    )).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
    
    let product_analysis = team["product_manager"].chat(&format!(
        "请从产品角度分析：{}。包括核心功能、用户价值和产品定位。",
        product_description
    )).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
    
    let marketing_analysis = team["marketing_expert"].chat(&format!(
        "请为这个产品制定初步营销策略：{}。包括目标用户、传播渠道和核心卖点。",
        product_description
    )).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;

    println!("📊 市场分析师的观点：\n{}\n", market_analysis);
    println!("🎯 产品经理的观点：\n{}\n", product_analysis);
    println!("📢 营销专家的观点：\n{}\n", marketing_analysis);

    // 第二阶段：协作讨论
    println!("💬 第二阶段：团队协作讨论");
    
    let discussion_context = format!(
        "团队讨论背景：\n\
         产品：{}\n\n\
         市场分析师的观点：{}\n\n\
         产品经理的观点：{}\n\n\
         营销专家的观点：{}\n\n",
        product_description, market_analysis, product_analysis, marketing_analysis
    );

    // 并行讨论
    let discussion_futures = vec![
        team["market_analyst"].chat(&format!(
            "{}基于以上讨论，请补充你的市场建议，特别是针对其他同事提出的观点。",
            discussion_context
        )),
        team["product_manager"].chat(&format!(
            "{}基于团队讨论，请调整产品策略，并回应市场和营销方面的建议。",
            discussion_context
        )),
        team["marketing_expert"].chat(&format!(
            "{}结合市场分析和产品定位，请完善营销策略，并提出具体的执行计划。",
            discussion_context
        )),
    ];

    let discussion_results: Vec<Result<String, lumosai::Error>> = join_all(discussion_futures).await;
    
    for (i, result) in discussion_results.iter().enumerate() {
        let role = match i {
            0 => "市场分析师",
            1 => "产品经理",
            2 => "营销专家",
            _ => "未知角色",
        };
        
        match result {
            Ok(response) => println!("🗣️ {} 的补充建议：\n{}\n", role, response),
            Err(e) => println!("❌ {} 响应失败：{}\n", role, e),
        }
    }

    // 第三阶段：最终整合
    println!("🎯 第三阶段：策略整合");
    
    let final_strategy = team["product_manager"].chat(&format!(
        "{}请作为项目负责人，整合所有团队成员的建议，\
         制定一个完整的产品营销策略，包括：\
         1. 产品定位和核心价值\
         2. 目标市场和用户画像\
         3. 营销策略和执行计划\
         4. 成功指标和里程碑",
        discussion_context
    )).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;

    println!("📋 最终营销策略：\n{}", final_strategy);

    println!("\n✅ 团队协作完成！");
    println!("💡 通过多个专业 Agent 的协作，我们得到了更全面、更专业的营销策略。");

    Ok(())
}
