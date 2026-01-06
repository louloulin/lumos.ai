//! # LumosAI 聊天机器人示例
//! 
//! 这个示例展示如何创建一个基础的聊天机器人，包含：
//! - 交互式对话界面
//! - 对话历史管理
//! - 多轮对话支持
//! - 优雅的错误处理

use lumosai::prelude::*;
use anyhow::Result;
use std::io::{self, Write};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "chatbot")]
#[command(about = "LumosAI 聊天机器人示例")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动交互式聊天
    Chat {
        /// 聊天机器人的角色设定
        #[arg(short, long, default_value = "你是一个友好的AI助手，用中文回答问题。")]
        role: String,
        /// 使用的模型
        #[arg(short, long, default_value = "gpt-3.5-turbo")]
        model: String,
    },
    /// 单次问答
    Ask {
        /// 要问的问题
        question: String,
        /// 聊天机器人的角色设定
        #[arg(short, long, default_value = "你是一个友好的AI助手，用中文回答问题。")]
        role: String,
        /// 使用的模型
        #[arg(short, long, default_value = "gpt-3.5-turbo")]
        model: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Chat { role, model }) => {
            start_interactive_chat(&model, &role).await?;
        }
        Some(Commands::Ask { question, role, model }) => {
            ask_single_question(&model, &role, &question).await?;
        }
        None => {
            // 默认启动交互式聊天
            start_interactive_chat("gpt-3.5-turbo", "你是一个友好的AI助手，用中文回答问题。").await?;
        }
    }
    
    Ok(())
}

/// 启动交互式聊天
async fn start_interactive_chat(model: &str, role: &str) -> Result<()> {
    println!("🤖 LumosAI 聊天机器人");
    println!("{}", "=".repeat(50));
    println!("💡 输入 'quit' 或 'exit' 退出聊天");
    println!("💡 输入 'clear' 清空对话历史");
    println!("💡 输入 'help' 查看帮助");
    println!();
    
    // 创建聊天机器人
    println!("📝 正在初始化聊天机器人...");
    let agent = lumosai::agent::simple(model, role).await?;
    println!("✅ 聊天机器人已就绪！模型: {}", model);
    println!();
    
    let mut conversation_history = Vec::new();
    
    loop {
        // 获取用户输入
        print!("👤 你: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        // 处理特殊命令
        match input.to_lowercase().as_str() {
            "quit" | "exit" => {
                println!("👋 再见！");
                break;
            }
            "clear" => {
                conversation_history.clear();
                println!("🧹 对话历史已清空");
                continue;
            }
            "help" => {
                show_help();
                continue;
            }
            "" => continue,
            _ => {}
        }
        
        // 记录用户消息
        conversation_history.push(format!("用户: {}", input));
        
        // 发送消息并获取回复
        print!("🤖 机器人: ");
        io::stdout().flush()?;
        
        match agent.chat(input).await {
            Ok(response) => {
                println!("{}", response);
                conversation_history.push(format!("机器人: {}", response));
            }
            Err(e) => {
                println!("❌ 出错了: {}", e);
                println!("💡 请稍后重试");
            }
        }
        
        println!();
    }
    
    Ok(())
}

/// 单次问答
async fn ask_single_question(model: &str, role: &str, question: &str) -> Result<()> {
    println!("🤖 LumosAI 单次问答");
    println!("{}", "=".repeat(30));
    
    // 创建聊天机器人
    let agent = lumosai::agent::simple(model, role).await?;
    
    println!("👤 问题: {}", question);
    print!("🤖 回答: ");
    io::stdout().flush()?;
    
    match agent.chat(question).await {
        Ok(response) => {
            println!("{}", response);
        }
        Err(e) => {
            println!("❌ 出错了: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

/// 显示帮助信息
fn show_help() {
    println!("📚 聊天机器人帮助");
    println!("{}", "-".repeat(30));
    println!("可用命令:");
    println!("  quit/exit  - 退出聊天");
    println!("  clear      - 清空对话历史");
    println!("  help       - 显示此帮助");
    println!();
    println!("💡 提示:");
    println!("  - 直接输入消息开始对话");
    println!("  - 支持多轮对话和上下文理解");
    println!("  - 如遇到错误，请检查网络连接和API配置");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_creation() {
        let agent = lumosai::agent::simple("gpt-3.5-turbo", "你是一个测试助手").await;
        assert!(agent.is_ok());
    }

    #[tokio::test]
    async fn test_single_question() {
        let result = ask_single_question(
            "gpt-3.5-turbo", 
            "你是一个测试助手，请简短回答。", 
            "测试"
        ).await;
        
        // 注意：这个测试可能因为网络或API问题失败
        // 在实际环境中，你可能需要mock这些调用
        println!("测试结果: {:?}", result);
    }
}
