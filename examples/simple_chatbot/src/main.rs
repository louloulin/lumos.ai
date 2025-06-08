//! 简单聊天机器人示例
//! 
//! 这个示例展示了如何使用 LumosAI 创建一个简单的命令行聊天机器人。
//! 
//! 运行方式:
//! ```bash
//! cargo run --example simple_chatbot
//! ```

use anyhow::Result;
use clap::Parser;
use lumosai::prelude::*;
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "simple_chatbot")]
#[command(about = "一个使用 LumosAI 构建的简单聊天机器人")]
struct Args {
    /// LLM 模型名称
    #[arg(short, long, default_value = "gpt-3.5-turbo")]
    model: String,
    
    /// 系统提示
    #[arg(short, long, default_value = "你是一个友善的AI助手，请用中文回答问题。")]
    system_prompt: String,
    
    /// 温度参数 (0.0-2.0)
    #[arg(short, long, default_value = "0.7")]
    temperature: f32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("🤖 LumosAI 聊天机器人");
    println!("模型: {}", args.model);
    println!("输入 'quit' 或 'exit' 退出\n");
    
    // 创建 Agent
    let agent = create_agent(&args).await?;
    
    // 开始聊天循环
    chat_loop(agent).await?;
    
    Ok(())
}

async fn create_agent(args: &Args) -> Result<impl Agent> {
    // 这里使用简化的 API 创建 Agent
    // 在实际实现中，这会调用我们的统一 API
    let agent = lumosai::agent::builder()
        .model(&args.model)
        .system_prompt(&args.system_prompt)
        .temperature(args.temperature)
        .build()
        .await?;
    
    Ok(agent)
}

async fn chat_loop(agent: impl Agent) -> Result<()> {
    let mut conversation_history = Vec::new();
    
    loop {
        // 获取用户输入
        print!("👤 你: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        // 检查退出命令
        if input.is_empty() {
            continue;
        }
        
        if input == "quit" || input == "exit" {
            println!("👋 再见！");
            break;
        }
        
        // 特殊命令处理
        if input.starts_with('/') {
            handle_command(input, &agent).await?;
            continue;
        }
        
        // 添加用户消息到历史
        conversation_history.push(Message::user(input));
        
        // 获取 AI 回复
        print!("🤖 AI: ");
        io::stdout().flush()?;
        
        match agent.chat_with_history(input, &conversation_history).await {
            Ok(response) => {
                println!("{}", response);
                conversation_history.push(Message::assistant(&response));
            }
            Err(e) => {
                eprintln!("❌ 错误: {}", e);
            }
        }
        
        println!(); // 空行分隔
    }
    
    Ok(())
}

async fn handle_command(command: &str, agent: &impl Agent) -> Result<()> {
    match command {
        "/help" => {
            println!("📖 可用命令:");
            println!("  /help     - 显示帮助信息");
            println!("  /stats    - 显示统计信息");
            println!("  /clear    - 清除对话历史");
            println!("  /info     - 显示 Agent 信息");
        }
        "/stats" => {
            // 显示性能统计
            if let Ok(stats) = agent.get_stats().await {
                println!("📊 统计信息:");
                println!("  总对话数: {}", stats.total_conversations);
                println!("  平均响应时间: {:?}", stats.average_response_time);
                println!("  成功率: {:.2}%", stats.success_rate * 100.0);
            }
        }
        "/clear" => {
            println!("🧹 对话历史已清除");
        }
        "/info" => {
            println!("ℹ️ Agent 信息:");
            println!("  模型: {}", agent.model_name());
            println!("  温度: {}", agent.temperature());
            println!("  系统提示: {}", agent.system_prompt());
        }
        _ => {
            println!("❓ 未知命令: {}，输入 /help 查看帮助", command);
        }
    }
    
    Ok(())
}

// 模拟的 Agent trait 和相关类型
// 在实际实现中，这些会在 lumosai crate 中定义
trait Agent {
    async fn chat_with_history(&self, message: &str, history: &[Message]) -> Result<String>;
    async fn get_stats(&self) -> Result<AgentStats>;
    fn model_name(&self) -> &str;
    fn temperature(&self) -> f32;
    fn system_prompt(&self) -> &str;
}

#[derive(Debug)]
struct Message {
    role: String,
    content: String,
}

impl Message {
    fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
        }
    }
    
    fn assistant(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
        }
    }
}

#[derive(Debug)]
struct AgentStats {
    total_conversations: u64,
    average_response_time: std::time::Duration,
    success_rate: f64,
}

// 模拟的 Agent 实现
struct SimpleAgent {
    model: String,
    system_prompt: String,
    temperature: f32,
    stats: AgentStats,
}

impl Agent for SimpleAgent {
    async fn chat_with_history(&self, message: &str, _history: &[Message]) -> Result<String> {
        // 模拟 AI 回复
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        let responses = vec![
            "这是一个很有趣的问题！",
            "让我想想...",
            "根据我的理解，",
            "这个话题很复杂，",
            "我认为...",
        ];
        
        let response = responses[message.len() % responses.len()];
        Ok(format!("{} {}", response, message))
    }
    
    async fn get_stats(&self) -> Result<AgentStats> {
        Ok(AgentStats {
            total_conversations: 42,
            average_response_time: std::time::Duration::from_millis(750),
            success_rate: 0.95,
        })
    }
    
    fn model_name(&self) -> &str {
        &self.model
    }
    
    fn temperature(&self) -> f32 {
        self.temperature
    }
    
    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }
}

// 模拟的 lumosai API
mod lumosai {
    pub mod prelude {
        pub use super::agent::*;
    }
    
    pub mod agent {
        use super::super::*;
        
        pub fn builder() -> AgentBuilder {
            AgentBuilder::default()
        }
        
        #[derive(Default)]
        pub struct AgentBuilder {
            model: Option<String>,
            system_prompt: Option<String>,
            temperature: Option<f32>,
        }
        
        impl AgentBuilder {
            pub fn model(mut self, model: &str) -> Self {
                self.model = Some(model.to_string());
                self
            }
            
            pub fn system_prompt(mut self, prompt: &str) -> Self {
                self.system_prompt = Some(prompt.to_string());
                self
            }
            
            pub fn temperature(mut self, temp: f32) -> Self {
                self.temperature = Some(temp);
                self
            }
            
            pub async fn build(self) -> Result<SimpleAgent> {
                Ok(SimpleAgent {
                    model: self.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
                    system_prompt: self.system_prompt.unwrap_or_else(|| "You are a helpful assistant".to_string()),
                    temperature: self.temperature.unwrap_or(0.7),
                    stats: AgentStats {
                        total_conversations: 0,
                        average_response_time: std::time::Duration::from_millis(0),
                        success_rate: 1.0,
                    },
                })
            }
        }
    }
}
