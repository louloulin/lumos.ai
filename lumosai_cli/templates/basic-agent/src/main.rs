//! {{project_name}} - {{description}}
//! 
//! 这是一个使用LumosAI创建的基础AI Agent项目。

use lumosai_core::prelude::*;
{{#if use_openai}}
use lumosai_core::llm::OpenAiProvider;
{{/if}}
{{#if use_mock}}
use lumosai_core::llm::MockLlmProvider;
{{/if}}
use std::sync::Arc;
use anyhow::Result;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 启动 {{project_name}}");

    // 创建LLM提供者
    {{#if use_openai}}
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("请设置OPENAI_API_KEY环境变量");
    let llm = Arc::new(OpenAiProvider::new(&api_key));
    {{else if use_mock}}
    let llm = Arc::new(MockLlmProvider::new(vec![
        "你好！我是{{agent_name}}，很高兴为您服务！".to_string(),
        "我可以帮助您处理各种任务。".to_string(),
        "有什么我可以帮助您的吗？".to_string(),
    ]));
    {{else}}
    // 默认使用Mock LLM用于演示
    let llm = Arc::new(MockLlmProvider::new(vec![
        "你好！我是{{agent_name}}，很高兴为您服务！".to_string(),
        "我可以帮助您处理各种任务。".to_string(),
        "有什么我可以帮助您的吗？".to_string(),
    ]));
    {{/if}}

    // 创建Agent
    let agent = quick_agent("{{agent_name}}", "{{agent_instructions}}")
        .model(llm)
        {{#if include_tools}}
        .tools(vec![
            calculator(),
            time_tool(),
            {{#if include_web_tools}}
            web_search(),
            {{/if}}
            {{#if include_file_tools}}
            file_reader(),
            {{/if}}
        ])
        {{/if}}
        .build()?;

    info!("✅ Agent '{}' 创建成功", agent.get_name());
    
    {{#if include_tools}}
    info!("🔧 可用工具数量: {}", agent.get_tools().len());
    for tool in agent.get_tools() {
        info!("   - {}: {}", tool.name(), tool.description());
    }
    {{/if}}

    // 示例对话
    let conversations = vec![
        "你好，请介绍一下自己",
        {{#if include_tools}}
        "请告诉我当前时间",
        {{#if include_web_tools}}
        "你能搜索网络信息吗？",
        {{/if}}
        {{#if include_file_tools}}
        "你能处理文件吗？",
        {{/if}}
        {{/if}}
        "谢谢你的帮助",
    ];

    for (i, input) in conversations.iter().enumerate() {
        println!("\n💬 对话 {}", i + 1);
        println!("👤 用户: {}", input);
        
        match agent.generate(input).await {
            Ok(response) => {
                println!("🤖 {}: {}", agent.get_name(), response.content);
            }
            Err(e) => {
                error!("❌ 生成响应失败: {}", e);
            }
        }
    }

    info!("🎉 {{project_name}} 运行完成");
    Ok(())
}

{{#if include_custom_functions}}
/// 自定义函数示例
async fn custom_processing(agent: &impl lumosai_core::agent::trait_def::Agent, input: &str) -> Result<String> {
    let response = agent.generate(input).await?;
    
    // 在这里添加自定义处理逻辑
    let processed = format!("处理后的响应: {}", response.content);
    
    Ok(processed)
}
{{/if}}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_creation() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
        
        let agent = quick_agent("test_agent", "Test instructions")
            .model(llm)
            .build();
        
        assert!(agent.is_ok());
        let agent = agent.unwrap();
        assert_eq!(agent.get_name(), "test_agent");
    }

    #[tokio::test]
    async fn test_agent_response() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello from test!".to_string()]));
        
        let agent = quick_agent("test_agent", "Test instructions")
            .model(llm)
            .build()
            .expect("Agent创建失败");
        
        let response = agent.generate("Hello").await;
        assert!(response.is_ok());
        
        let response = response.unwrap();
        assert_eq!(response.content, "Hello from test!");
    }
}
