//! MVP 示例 4: 带记忆的 Agent
//!
//! 这个示例展示如何为 Agent 添加记忆功能，实现上下文对话。
//!
//! 运行方式:
//! ```bash
//! cargo run --example mvp_04_agent_with_memory
//! ```

use lumosai_core::agent::{create_basic_agent, Agent};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use lumosai_core::llm::{LlmProvider, Message, Role};
use lumosai_core::memory::{create_working_memory, Memory, WorkingMemoryConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MVP 示例 4: 带记忆的 Agent\n");
    println!("=".repeat(50));
    
    // 步骤 1: 创建内存系统
    println!("\n📝 步骤 1: 创建内存系统");
    let memory = create_working_memory(WorkingMemoryConfig {
        max_messages: 10,
        max_tokens: 4000,
    });
    println!("✅ 内存系统创建成功");
    println!("   - 最大消息数: 10");
    println!("   - 最大 token 数: 4000");
    
    // 步骤 2: 创建 LLM 提供商
    println!("\n📝 步骤 2: 创建 LLM 提供商");
    let llm: Arc<dyn LlmProvider> = create_test_zhipu_provider_arc();
    println!("✅ LLM 提供商创建成功");
    
    // 步骤 3: 创建带内存的 Agent
    println!("\n📝 步骤 3: 创建带内存的 Agent");
    let mut agent = create_basic_agent(
        "memory_assistant",
        "You are a helpful assistant with memory. Remember user information and context from previous messages.",
        llm,
    );
    
    // 设置内存
    agent.set_memory(memory.clone());
    println!("✅ Agent '{}' 创建成功，已配置内存", agent.name());
    
    // 步骤 4: 多轮对话测试
    println!("\n📝 步骤 4: 多轮对话测试");
    println!("━".repeat(50));
    
    let conversations = vec![
        "Hi! My name is Alice and I'm a software engineer.",
        "What's my name?",
        "What's my profession?",
        "I'm working on a Rust project about AI agents.",
        "What am I working on?",
        "Can you summarize what you know about me?",
    ];
    
    for (i, message) in conversations.iter().enumerate() {
        println!("\n💬 对话 {}: {}", i + 1, message);
        
        // 添加用户消息到内存
        memory.add_message(Message {
            role: Role::User,
            content: message.to_string(),
            name: None,
            function_call: None,
        }).await?;
        
        // 生成响应
        match agent.generate_simple(message).await {
            Ok(response) => {
                println!("🤖 Agent: {}", response);
                
                // 添加 Agent 响应到内存
                memory.add_message(Message {
                    role: Role::Assistant,
                    content: response,
                    name: None,
                    function_call: None,
                }).await?;
            }
            Err(e) => {
                eprintln!("❌ 错误: {}", e);
            }
        }
    }
    
    // 步骤 5: 查看内存内容
    println!("\n📝 步骤 5: 查看内存内容");
    println!("━".repeat(50));
    
    let messages = memory.get_messages().await?;
    println!("\n📊 内存中的消息数量: {}", messages.len());
    println!("\n📜 完整对话历史:");
    println!("{}", "─".repeat(50));
    
    for (i, msg) in messages.iter().enumerate() {
        let role_icon = match msg.role {
            Role::User => "💬",
            Role::Assistant => "🤖",
            Role::System => "⚙️",
            Role::Function => "🔧",
        };
        println!("\n{} 消息 {} ({:?}):", role_icon, i + 1, msg.role);
        println!("{}", msg.content);
    }
    
    println!("\n{}", "─".repeat(50));
    
    // 步骤 6: 测试内存清理
    println!("\n📝 步骤 6: 测试内存清理");
    println!("━".repeat(50));
    
    println!("\n🗑️  清理内存...");
    memory.clear().await?;
    
    let messages_after_clear = memory.get_messages().await?;
    println!("✅ 内存已清理");
    println!("   - 清理前消息数: {}", messages.len());
    println!("   - 清理后消息数: {}", messages_after_clear.len());
    
    // 步骤 7: 测试内存限制
    println!("\n📝 步骤 7: 测试内存限制");
    println!("━".repeat(50));
    
    println!("\n📊 添加超过限制的消息...");
    for i in 1..=15 {
        memory.add_message(Message {
            role: Role::User,
            content: format!("Test message {}", i),
            name: None,
            function_call: None,
        }).await?;
    }
    
    let final_messages = memory.get_messages().await?;
    println!("✅ 消息添加完成");
    println!("   - 尝试添加: 15 条消息");
    println!("   - 实际保留: {} 条消息（受限于 max_messages=10）", final_messages.len());
    
    println!("\n" + &"=".repeat(50));
    println!("✅ 示例完成！");
    println!("\n💡 提示:");
    println!("   - Memory 系统可以保存对话历史");
    println!("   - 支持最大消息数和 token 数限制");
    println!("   - Agent 可以利用历史上下文生成更好的响应");
    println!("   - 下一步: 查看 mvp_05_workflow.rs 学习工作流编排");
    
    Ok(())
}

