use anyhow::Result;
use lumosai::prelude::SimpleAgent;
use std::collections::HashMap;

/// 简单工作流示例 - 展示基本的工作流概念
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    println!("🔄 简单工作流示例");
    println!("{}", "=".repeat(50));
    
    // 创建工作流步骤的 Agent
    let researcher = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是专业研究员，擅长收集和分析信息。"
    ).await?;
    
    let writer = lumosai::agent::simple(
        "gpt-3.5-turbo", 
        "你是专业作家，擅长将研究结果转化为易读的内容。"
    ).await?;
    
    let reviewer = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是质量审查员，擅长检查内容质量和准确性。"
    ).await?;
    
    // 工作流输入
    let topic = "人工智能在教育中的应用";
    println!("\n📝 工作流主题: {}", topic);
    
    // 步骤 1: 研究
    println!("\n🔍 步骤 1: 研究阶段");
    let research_prompt = format!(
        "请研究以下主题并提供详细分析：{}\n请包括：\n1. 当前发展状况\n2. 主要应用场景\n3. 技术挑战\n4. 未来趋势",
        topic
    );
    
    let research_result = researcher.chat(&research_prompt).await
        .map_err(|e| anyhow::anyhow!(e))?;
    
    println!("✅ 研究完成");
    println!("研究结果摘要: {}...", 
        research_result.chars().take(100).collect::<String>());
    
    // 步骤 2: 写作
    println!("\n✍️ 步骤 2: 写作阶段");
    let writing_prompt = format!(
        "基于以下研究结果，写一篇结构清晰的文章：\n\n{}\n\n请确保文章：\n1. 结构清晰\n2. 语言流畅\n3. 观点明确\n4. 适合一般读者阅读",
        research_result
    );
    
    let article = writer.chat(&writing_prompt).await
        .map_err(|e| anyhow::anyhow!(e))?;
    
    println!("✅ 写作完成");
    println!("文章摘要: {}...", 
        article.chars().take(100).collect::<String>());
    
    // 步骤 3: 审查
    println!("\n🔍 步骤 3: 审查阶段");
    let review_prompt = format!(
        "请审查以下文章并提供改进建议：\n\n{}\n\n请从以下方面评估：\n1. 内容准确性\n2. 逻辑结构\n3. 语言表达\n4. 读者友好性\n\n请给出具体的改进建议。",
        article
    );
    
    let review_result = reviewer.chat(&review_prompt).await
        .map_err(|e| anyhow::anyhow!(e))?;
    
    println!("✅ 审查完成");
    println!("审查结果摘要: {}...", 
        review_result.chars().take(100).collect::<String>());
    
    // 工作流总结
    println!("\n🎉 工作流执行完成！");
    println!("{}", "=".repeat(50));
    println!("📊 执行统计:");
    println!("  - 总步骤数: 3");
    println!("  - 成功步骤: 3");
    println!("  - 失败步骤: 0");
    println!("  - 主题: {}", topic);
    
    // 保存结果到上下文
    let mut workflow_context = HashMap::new();
    workflow_context.insert("topic".to_string(), topic.to_string());
    workflow_context.insert("research".to_string(), research_result);
    workflow_context.insert("article".to_string(), article);
    workflow_context.insert("review".to_string(), review_result);
    
    println!("\n💾 工作流上下文已保存，包含 {} 个数据项", workflow_context.len());
    
    Ok(())
}
