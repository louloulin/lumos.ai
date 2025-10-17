use lumosai::prelude::SimpleAgent;
use std::collections::HashMap;
use futures::future::join_all;

/// Agent 协调示例
/// 展示多个 Agent 如何通过协调者进行任务分配和结果整合

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    println!("🎯 Agent 协调示例");
    println!("场景：智能客服系统 - 多个专业 Agent 协作处理客户问题\n");

    // 创建协调者 Agent
    let coordinator = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是一个智能客服协调者，负责分析客户问题并分配给合适的专业 Agent。\
         你需要：1) 分析问题类型 2) 选择合适的专业 Agent 3) 整合各个 Agent 的回答"
    ).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;

    // 创建专业 Agent 团队
    let mut specialists = HashMap::new();
    
    // 技术支持专家
    let tech_support = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是技术支持专家，专门解决软件、硬件和网络技术问题。\
         请提供详细的技术解决方案和步骤指导。"
    ).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
    specialists.insert("tech_support", tech_support);

    // 账单专家
    let billing_expert = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是账单和付费专家，专门处理账单查询、付费问题和退款申请。\
         请提供准确的账单信息和付费指导。"
    ).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
    specialists.insert("billing", billing_expert);

    // 产品顾问
    let product_advisor = lumosai::agent::simple(
        "gpt-3.5-turbo",
        "你是产品顾问，专门介绍产品功能、使用方法和最佳实践。\
         请提供详细的产品使用指导和建议。"
    ).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
    specialists.insert("product", product_advisor);

    // 客户问题列表
    let customer_questions = vec![
        "我的软件无法启动，显示错误代码 0x80070005",
        "我想查询上个月的账单详情，并申请部分退款",
        "请介绍一下你们产品的高级功能，我想升级套餐",
        "我的网络连接不稳定，经常断线",
        "账单中有一笔我不认识的费用，请帮我查询",
    ];

    for (i, question) in customer_questions.iter().enumerate() {
        println!("📞 客户问题 {}: {}", i + 1, question);
        
        // 第一步：协调者分析问题
        let analysis_prompt = format!(
            "客户问题：{}\n\n请分析这个问题属于哪个类别：\n\
             1. tech_support - 技术支持问题\n\
             2. billing - 账单和付费问题\n\
             3. product - 产品咨询问题\n\n\
             请只回答类别名称（tech_support/billing/product）",
            question
        );
        
        let category = coordinator.chat(&analysis_prompt).await
            .map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
        
        let category = category.trim().to_lowercase();
        println!("🎯 协调者分析：问题类别 = {}", category);

        // 第二步：分配给专业 Agent
        if let Some(specialist) = specialists.get(category.as_str()) {
            let specialist_response = specialist.chat(&format!(
                "客户问题：{}\n\n请作为专业人员提供详细的解决方案。",
                question
            )).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
            
            println!("👨‍💼 专业 Agent 回答：\n{}", specialist_response);
            
            // 第三步：协调者整合和优化回答
            let final_response = coordinator.chat(&format!(
                "客户问题：{}\n\n专业 Agent 的回答：{}\n\n\
                 请作为客服协调者，将专业回答整理成更友好、更易懂的客户回复。\
                 保持专业性的同时，使用更温和的语气。",
                question, specialist_response
            )).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
            
            println!("✅ 最终客服回复：\n{}\n", final_response);
        } else {
            println!("❌ 未找到合适的专业 Agent 处理此问题\n");
        }
        
        println!("{}", "=".repeat(80));
    }

    // 演示并行处理能力
    println!("\n🚀 并行处理演示");
    println!("同时处理多个客户问题...\n");
    
    let parallel_questions = vec![
        "软件崩溃了怎么办？",
        "如何查看我的消费记录？",
        "产品有哪些新功能？",
    ];
    
    let parallel_futures: Vec<_> = parallel_questions.iter().enumerate().map(|(i, question)| {
        let coordinator = &coordinator;
        let specialists = &specialists;
        async move {
            println!("🔄 开始处理问题 {}: {}", i + 1, question);
            
            // 分析问题类别
            let analysis = coordinator.chat(&format!(
                "问题：{}\n请分析类别（tech_support/billing/product）",
                question
            )).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
            
            let category = analysis.trim().to_lowercase();
            
            // 获取专业回答
            if let Some(specialist) = specialists.get(category.as_str()) {
                let response = specialist.chat(&format!(
                    "问题：{}\n请提供简洁的解决方案。",
                    question
                )).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))?;
                
                Ok::<String, anyhow::Error>(format!(
                    "问题 {}: {} -> 类别: {} -> 回答: {}",
                    i + 1, question, category, response.chars().take(100).collect::<String>() + "..."
                ))
            } else {
                Ok(format!("问题 {}: {} -> 无法处理", i + 1, question))
            }
        }
    }).collect();
    
    let results = join_all(parallel_futures).await;
    
    for result in results {
        match result {
            Ok(summary) => println!("✅ {}", summary),
            Err(e) => println!("❌ 处理失败: {}", e),
        }
    }

    println!("\n🎉 Agent 协调演示完成！");
    println!("💡 通过协调者模式，我们实现了：");
    println!("   - 智能问题分类和路由");
    println!("   - 专业 Agent 的精准回答");
    println!("   - 统一的客户体验");
    println!("   - 并行处理能力");

    Ok(())
}
