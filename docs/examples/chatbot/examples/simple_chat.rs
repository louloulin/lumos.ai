//! 简单聊天示例
//! 
//! 展示最基础的聊天机器人功能

use lumosai::prelude::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    println!("🤖 简单聊天示例");
    println!("{}", "=".repeat(30));
    
    // 创建一个简单的聊天机器人
    let agent = lumosai::agent::simple(
        "gpt-3.5-turbo", 
        "你是一个友好的AI助手，用中文简洁地回答问题。"
    ).await?;
    
    // 预定义的对话
    let conversations = vec![
        "你好！",
        "你能做什么？",
        "请介绍一下 LumosAI",
        "谢谢你的帮助！",
    ];
    
    for (i, message) in conversations.iter().enumerate() {
        println!("\n💬 对话 {}", i + 1);
        println!("👤 用户: {}", message);
        
        match agent.chat(message).await {
            Ok(response) => {
                println!("🤖 助手: {}", response);
            }
            Err(e) => {
                println!("❌ 错误: {}", e);
            }
        }
        
        // 添加延迟，模拟真实对话
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    println!("\n✅ 简单聊天示例完成！");
    
    Ok(())
}
