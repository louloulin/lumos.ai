use anyhow::Result;
use futures::future::join_all;
use lumosai::prelude::SimpleAgent;
use std::time::Instant;

/// 并行工作流示例 - 展示如何并行执行多个工作流步骤
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    println!("⚡ 并行工作流示例");
    println!("{}", "=".repeat(50));
    
    // 创建不同专业的 Agent
    let market_analyst = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是市场分析专家，擅长市场趋势分析和竞争对手研究。"
    ).await?;
    
    let tech_analyst = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是技术分析专家，擅长技术评估和架构设计。"
    ).await?;
    
    let financial_analyst = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是财务分析专家，擅长成本分析和投资回报评估。"
    ).await?;
    
    let risk_analyst = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是风险分析专家，擅长识别和评估各种业务风险。"
    ).await?;
    
    // 项目信息
    let project = "开发一个基于 AI 的智能客服系统";
    println!("\n📋 项目: {}", project);
    
    println!("\n🚀 启动并行分析...");
    let start_time = Instant::now();
    
    // 创建并行任务
    let market_task = async {
        let prompt = format!(
            "请分析以下项目的市场前景：{}\n\n请包括：\n1. 目标市场规模\n2. 竞争对手分析\n3. 市场机会\n4. 市场挑战",
            project
        );
        
        println!("📊 开始市场分析...");
        let result = market_analyst.chat(&prompt).await
            .map_err(|e| anyhow::anyhow!("市场分析失败: {}", e))?;
        println!("✅ 市场分析完成");
        Ok::<String, anyhow::Error>(result)
    };
    
    let tech_task = async {
        let prompt = format!(
            "请分析以下项目的技术方案：{}\n\n请包括：\n1. 技术架构设计\n2. 关键技术选型\n3. 技术难点\n4. 实施方案",
            project
        );
        
        println!("🔧 开始技术分析...");
        let result = tech_analyst.chat(&prompt).await
            .map_err(|e| anyhow::anyhow!("技术分析失败: {}", e))?;
        println!("✅ 技术分析完成");
        Ok::<String, anyhow::Error>(result)
    };
    
    let financial_task = async {
        let prompt = format!(
            "请分析以下项目的财务状况：{}\n\n请包括：\n1. 开发成本估算\n2. 运营成本分析\n3. 收入预测\n4. 投资回报率",
            project
        );
        
        println!("💰 开始财务分析...");
        let result = financial_analyst.chat(&prompt).await
            .map_err(|e| anyhow::anyhow!("财务分析失败: {}", e))?;
        println!("✅ 财务分析完成");
        Ok::<String, anyhow::Error>(result)
    };
    
    let risk_task = async {
        let prompt = format!(
            "请分析以下项目的风险因素：{}\n\n请包括：\n1. 技术风险\n2. 市场风险\n3. 财务风险\n4. 运营风险\n5. 风险缓解策略",
            project
        );
        
        println!("⚠️ 开始风险分析...");
        let result = risk_analyst.chat(&prompt).await
            .map_err(|e| anyhow::anyhow!("风险分析失败: {}", e))?;
        println!("✅ 风险分析完成");
        Ok::<String, anyhow::Error>(result)
    };
    
    // 并行执行所有任务
    let results = join_all(vec![
        Box::pin(market_task),
        Box::pin(tech_task), 
        Box::pin(financial_task),
        Box::pin(risk_task),
    ]).await;
    
    let execution_time = start_time.elapsed();
    
    println!("\n⏱️ 并行执行完成，耗时: {:.2}秒", execution_time.as_secs_f64());
    
    // 处理结果
    let mut successful_analyses = Vec::new();
    let mut failed_analyses = Vec::new();
    
    let analysis_types = vec!["市场分析", "技术分析", "财务分析", "风险分析"];
    
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(analysis) => {
                successful_analyses.push((analysis_types[i], analysis));
            }
            Err(e) => {
                failed_analyses.push((analysis_types[i], e.to_string()));
            }
        }
    }
    
    // 显示结果统计
    println!("\n📊 执行结果统计:");
    println!("  ✅ 成功: {} 个分析", successful_analyses.len());
    println!("  ❌ 失败: {} 个分析", failed_analyses.len());
    
    // 显示成功的分析结果
    if !successful_analyses.is_empty() {
        println!("\n📋 分析结果摘要:");
        for (analysis_type, result) in &successful_analyses {
            println!("\n🔸 {}:", analysis_type);
            let summary = result.chars().take(150).collect::<String>();
            println!("   {}", summary);
            if result.len() > 150 {
                println!("   ...");
            }
        }
    }
    
    // 显示失败的分析
    if !failed_analyses.is_empty() {
        println!("\n❌ 失败的分析:");
        for (analysis_type, error) in &failed_analyses {
            println!("  - {}: {}", analysis_type, error);
        }
    }
    
    // 如果所有分析都成功，进行综合分析
    if successful_analyses.len() == 4 {
        println!("\n🔄 进行综合分析...");
        
        let synthesizer = lumosai::agent::simple(
            "gpt-3.5-turbo",
            "你是项目综合分析专家，擅长整合多维度分析结果并提供决策建议。"
        ).await?;
        
        let synthesis_prompt = format!(
            "请基于以下四个维度的分析结果，提供项目的综合评估和决策建议：\n\n市场分析：\n{}\n\n技术分析：\n{}\n\n财务分析：\n{}\n\n风险分析：\n{}\n\n请提供：\n1. 项目可行性评估\n2. 关键成功因素\n3. 主要风险点\n4. 实施建议\n5. 决策建议",
            successful_analyses[0].1,
            successful_analyses[1].1,
            successful_analyses[2].1,
            successful_analyses[3].1
        );
        
        match synthesizer.chat(&synthesis_prompt).await {
            Ok(synthesis) => {
                println!("✅ 综合分析完成");
                println!("\n🎯 项目综合评估:");
                println!("{}", synthesis);
            }
            Err(e) => {
                println!("❌ 综合分析失败: {}", e);
            }
        }
    }
    
    println!("\n🎉 并行工作流执行完成！");
    println!("总执行时间: {:.2}秒", execution_time.as_secs_f64());
    
    Ok(())
}
