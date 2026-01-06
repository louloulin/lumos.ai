//! # Hello World - 最简单的 LumosAI Agent 示例
//! 
//! 这个示例展示了如何创建和使用最基本的 AI Agent。
//! 
//! ## 功能特性
//! - 创建基本的 Agent
//! - 发送消息并获取回复
//! - 基础错误处理
//! 
//! ## 使用方法
//! ```bash
//! export OPENAI_API_KEY="your-api-key"
//! cargo run
//! ```

use lumosai::prelude::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();
    
    println!("🤖 LumosAI Hello World 示例");
    println!("{}", "=".repeat(40));
    
    // 创建最简单的 Agent
    println!("📝 创建 Agent...");
    let agent = lumosai::agent::simple("gpt-3.5-turbo", "你是一个友好的助手，用中文回答问题。请保持回答简洁明了。").await?;
    
    println!("✅ Agent 创建成功: {}", agent.name());
    
    // 发送第一条消息
    println!("\n💬 发送消息: 你好！");
    let response = agent.chat("你好！").await?;
    println!("🤖 Agent 回复: {}", response);
    
    // 发送第二条消息
    println!("\n💬 发送消息: 你能做什么？");
    let response = agent.chat("你能做什么？").await?;
    println!("🤖 Agent 回复: {}", response);
    
    // 发送第三条消息
    println!("\n💬 发送消息: 请用一句话介绍 LumosAI");
    let response = agent.chat("请用一句话介绍 LumosAI").await?;
    println!("🤖 Agent 回复: {}", response);
    
    println!("\n🎉 Hello World 示例完成！");
    println!("📚 接下来可以尝试:");
    println!("   - cargo run --example interactive  # 交互式对话");
    println!("   - cargo run --example configured   # 配置示例");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_agent_creation() {
        let agent = lumosai::agent::simple("gpt-3.5-turbo", "你是一个测试助手").await;

        assert!(agent.is_ok());
        let agent = agent.unwrap();
        assert_eq!(agent.name(), "SimpleAgent");
    }
    
    #[tokio::test]
    async fn test_agent_generate() {
        // 注意：这个测试需要有效的 API 密钥
        if std::env::var("OPENAI_API_KEY").is_err() {
            println!("跳过测试：未设置 OPENAI_API_KEY");
            return;
        }
        
        let agent = lumosai::agent::simple("gpt-3.5-turbo", "你是一个测试助手，请简短回答。").await
            .unwrap();

        let response = agent.chat("测试").await;
        assert!(response.is_ok());
        
        let response = response.unwrap();
        assert!(!response.is_empty());
        println!("测试回复: {}", response);
    }
}
