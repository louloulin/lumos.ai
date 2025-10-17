//! # 交互式 Hello World 示例
//! 
//! 这个示例展示了如何创建一个交互式的对话系统。

use lumosai::prelude::*;
use std::io::{self, Write};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 LumosAI 交互式对话示例");
    println!("=" .repeat(40));
    println!("输入 'quit' 或 '退出' 来结束对话\n");
    
    // 创建 Agent
    let agent = Agent::builder()
        .name("交互助手")
        .instructions(
            "你是一个友好的交互助手。\
            请用中文回答问题，保持回答简洁有用。\
            如果用户问候你，请友好回应。"
        )
        .model("gpt-3.5-turbo")
        .temperature(0.7)
        .build()?;
    
    println!("✅ {} 已准备就绪！\n", agent.name());
    
    // 开始交互循环
    loop {
        // 获取用户输入
        print!("👤 用户: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        // 检查退出条件
        if input.eq_ignore_ascii_case("quit") 
            || input.eq_ignore_ascii_case("退出") 
            || input.eq_ignore_ascii_case("exit") {
            println!("👋 再见！感谢使用 LumosAI！");
            break;
        }
        
        // 跳过空输入
        if input.is_empty() {
            continue;
        }
        
        // 生成回复
        print!("🤖 {} 正在思考", agent.name());
        io::stdout().flush()?;
        
        // 显示思考动画
        for _ in 0..3 {
            print!(".");
            io::stdout().flush()?;
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        print!("\r");
        
        match agent.generate(input).await {
            Ok(response) => {
                println!("🤖 {}: {}\n", agent.name(), response);
            }
            Err(e) => {
                println!("❌ 错误: {}\n", e);
                println!("💡 提示: 请检查您的 API 密钥是否正确设置");
            }
        }
    }
    
    Ok(())
}
