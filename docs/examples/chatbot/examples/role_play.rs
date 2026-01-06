//! 角色扮演聊天示例
//! 
//! 展示如何创建具有特定角色的聊天机器人

use lumosai::prelude::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    println!("🎭 角色扮演聊天示例");
    println!("{}", "=".repeat(40));
    
    // 创建不同角色的聊天机器人
    let roles = vec![
        (
            "编程导师",
            "你是一位经验丰富的编程导师，专门教授 Rust 编程语言。你的回答应该专业、耐心，并且包含实用的代码示例。"
        ),
        (
            "诗人",
            "你是一位富有创意的诗人，喜欢用优美的语言和诗意的表达来回答问题。你的回答应该充满想象力和艺术感。"
        ),
        (
            "科学家",
            "你是一位严谨的科学家，习惯用科学的方法和逻辑来分析问题。你的回答应该基于事实，逻辑清晰。"
        ),
    ];
    
    for (role_name, role_prompt) in roles {
        println!("\n🎭 当前角色: {}", role_name);
        println!("{}", "-".repeat(30));
        
        let agent = lumosai::agent::simple("gpt-3.5-turbo", role_prompt).await?;
        
        let question = "请介绍一下人工智能的发展历程";
        println!("👤 问题: {}", question);
        
        match agent.chat(question).await {
            Ok(response) => {
                println!("🤖 {}: {}", role_name, response);
            }
            Err(e) => {
                println!("❌ 错误: {}", e);
            }
        }
        
        // 添加延迟
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    
    println!("\n✅ 角色扮演示例完成！");
    println!("💡 你可以尝试创建更多有趣的角色！");
    
    Ok(())
}
