use anyhow::Result;
use lumosai::prelude::*;

/// 演示网络研究功能
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    println!("🔍 LumosAI 网络研究演示");
    println!("{}", "=".repeat(40));

    // 创建研究助手
    let agent = lumosai::agent::simple(
        "gpt-3.5-turbo",
        r#"
你是一个专业的网络研究助手。你的任务是：
1. 分析用户提供的研究主题
2. 模拟搜索相关信息
3. 提供结构化的研究报告

请用中文回答，保持专业和客观。
"#,
    ).await?;

    // 研究主题列表
    let topics = vec![
        "人工智能在医疗领域的应用",
        "区块链技术的发展趋势",
        "可再生能源的未来前景",
    ];

    for (i, topic) in topics.iter().enumerate() {
        println!("\n📚 研究主题 {}: {}", i + 1, topic);
        println!("{}", "-".repeat(30));

        // 模拟搜索结果
        let search_context = format!(
            "请为主题 '{}' 提供一份研究报告，包括：\n\
            1. 主题概述\n\
            2. 当前发展状况\n\
            3. 主要挑战\n\
            4. 未来趋势\n\
            5. 关键参考资料\n\n\
            请保持报告的专业性和客观性。",
            topic
        );

        match agent.chat(&search_context).await {
            Ok(report) => {
                println!("📊 研究报告:");
                println!("{}", report);
            }
            Err(e) => {
                println!("❌ 生成报告失败: {}", e);
            }
        }

        if i < topics.len() - 1 {
            println!("\n{}", "=".repeat(50));
        }
    }

    println!("\n✅ 网络研究演示完成！");
    Ok(())
}
