//! Hello World - 最简单的LumosAI Agent示例
//! 
//! 这个示例展示了如何用最少的代码创建一个AI Agent并生成响应。
//! 
//! 运行方式:
//! ```bash
//! cargo run --example hello_world
//! ```

use lumosai_core::prelude::*;
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::agent::trait_def::Agent;
use lumosai_core::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 LumosAI Hello World 示例");
    println!("================================");
    
    // 创建一个Mock LLM提供者用于演示
    // 在实际应用中，您会使用真实的LLM如OpenAI、Anthropic等
    let llm = Arc::new(MockLlmProvider::new(vec![
        "你好！我是您的AI助手，很高兴为您服务！".to_string(),
        "我可以帮助您解答问题、处理任务和提供建议。".to_string(),
        "有什么我可以帮助您的吗？".to_string(),
    ]));
    
    // 使用最简单的API创建Agent
    // quick_agent() 是最快速的Agent创建方式
    let agent = quick_agent("assistant", "你是一个友好的AI助手")
        .model(llm)
        .build()?;
    
    println!("✅ Agent创建成功!");
    println!("   - 名称: {}", agent.get_name());
    println!("   - 指令: {}", agent.get_instructions());
    
    // 生成第一个响应
    println!("\n🤖 开始对话...");
    let response = agent.generate_simple("你好，请介绍一下自己").await?;
    
    println!("👤 用户: 你好，请介绍一下自己");
    println!("🤖 助手: {}", response);

    // 继续对话
    let response2 = agent.generate_simple("你能做什么？").await?;
    
    println!("\n👤 用户: 你能做什么？");
    println!("🤖 助手: {}", response2);
    
    println!("\n🎉 Hello World 示例完成!");
    println!("\n📚 下一步学习:");
    println!("   - examples/01_getting_started/quick_api.rs - 学习快速API");
    println!("   - examples/01_getting_started/basic_tools.rs - 学习工具使用");
    println!("   - docs/tutorials/beginner/ - 查看完整教程");
    
    Ok(())
}

// 如果您有OpenAI API密钥，可以使用这个版本
#[allow(dead_code)]
async fn hello_world_with_openai() -> Result<()> {
    use lumosai_core::llm::OpenAiProvider;
    
    // 从环境变量获取API密钥
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("请设置OPENAI_API_KEY环境变量");

    let llm = Arc::new(OpenAiProvider::new(api_key, "gpt-3.5-turbo".to_string()));
    
    let agent = quick_agent("assistant", "你是一个友好的AI助手")
        .model(llm)
        .build()?;
    
    let response = agent.generate_simple("解释一下什么是人工智能").await?;
    println!("🤖 Agent回复: {}", response);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hello_world() {
        let result = main().await;
        assert!(result.is_ok(), "Hello World示例应该成功运行");
    }
    
    #[tokio::test]
    async fn test_agent_creation() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
        
        let agent = quick_agent("test", "Test assistant")
            .model(llm)
            .build();
        
        assert!(agent.is_ok(), "Agent创建应该成功");
        
        let agent = agent.unwrap();
        assert_eq!(agent.get_name(), "test");
        assert_eq!(agent.get_instructions(), "Test assistant");
    }
    
    #[tokio::test]
    async fn test_agent_response() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello from test!".to_string()]));
        
        let agent = quick_agent("test", "Test assistant")
            .model(llm)
            .build()
            .expect("Agent创建失败");
        
        let response = agent.generate_simple("Hello").await;
        assert!(response.is_ok(), "响应生成应该成功");

        let response = response.unwrap();
        assert_eq!(response, "Hello from test!");
    }
}
