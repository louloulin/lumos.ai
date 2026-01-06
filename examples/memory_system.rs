//! 记忆系统演示
//!
//! 展示如何使用记忆系统，包括：
//! - 工作记忆配置
//! - 多轮对话记忆
//! - 记忆内容管理
//! - 记忆检索和总结

use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::llm::{Message, MockLlmProvider, Role};
use lumosai_core::memory::{
    working::{create_working_memory, WorkingMemoryConfig},
    MemoryConfig,
};
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("💾 记忆系统演示");
    println!("================");

    // 演示1: 基础记忆配置
    demo_basic_memory().await?;

    // 演示2: 多轮对话记忆
    demo_conversation_memory().await?;

    // 演示3: 记忆管理功能
    demo_memory_management().await?;

    // 演示4: 记忆检索和总结
    demo_memory_retrieval().await?;

    Ok(())
}

/// 演示基础记忆配置
async fn demo_basic_memory() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示1: 基础记忆配置 ===");

    // 创建记忆配置
    let memory_config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(4000),
    };

    println!("记忆配置:");
    println!("  启用状态: {}", memory_config.enabled);
    println!("  内容类型: {:?}", memory_config.content_type);
    println!("  最大容量: {:?}", memory_config.max_capacity);

    // 创建工作记忆实例
    let working_memory = create_working_memory(&memory_config)?;

    // 添加一些测试消息到工作内存
    let messages = vec![
        Message::new(Role::User, "你好，我是新用户".to_string(), None, None),
        Message::new(
            Role::Assistant,
            "你好！欢迎使用我们的服务。我是你的AI助手。".to_string(),
            None,
            None,
        ),
        Message::new(Role::User, "我想了解一下你的功能".to_string(), None, None),
        Message::new(
            Role::Assistant,
            "我可以帮助你回答问题、提供信息、协助完成任务等。".to_string(),
            None,
            None,
        ),
    ];

    // 将消息存储到工作内存中
    for message in messages {
        let mut content = working_memory.get().await?;
        let mut messages_array = if let Some(msgs) = content.content.get("messages") {
            msgs.as_array().unwrap_or(&vec![]).clone()
        } else {
            vec![]
        };

        messages_array.push(serde_json::to_value(&message)?);

        if let serde_json::Value::Object(ref mut map) = content.content {
            map.insert(
                "messages".to_string(),
                serde_json::Value::Array(messages_array),
            );
        }

        working_memory.update(content).await?;
    }

    println!("\n当前记忆状态:");
    let content = working_memory.get().await?;
    if let Some(messages) = content.content.get("messages") {
        if let Some(msgs_array) = messages.as_array() {
            println!("  消息数量: {}", msgs_array.len());
        }
    }

    Ok(())
}

/// 演示多轮对话记忆
async fn demo_conversation_memory() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示2: 多轮对话记忆 ===");

    // 创建对话响应序列
    let conversation_responses = vec![
        "你好张三！很高兴认识你。25岁正是学习和成长的好年龄。我会记住你的信息。".to_string(),
        "编程和阅读都是很棒的爱好！编程可以锻炼逻辑思维，阅读可以拓宽知识面。你主要使用什么编程语言呢？".to_string(),
        "Rust是一门很棒的语言！它的内存安全特性和性能表现都很出色。你学习Rust多长时间了？".to_string(),
        "当然记得！你是张三，今年25岁，爱好是编程和阅读，特别喜欢Rust语言。我们刚才还聊到了你的学习经历呢。".to_string(),
        "根据我们的对话，我了解到你是一个25岁的程序员，名叫张三，热爱编程（特别是Rust）和阅读。你似乎是一个很有学习热情的人。".to_string(),
    ];
    let llm_provider = Arc::new(MockLlmProvider::new(conversation_responses));

    // 创建记忆配置
    let working_memory_config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(8000),
    };

    let memory_config = MemoryConfig {
        store_id: None,
        namespace: None,
        enabled: true,
        working_memory: Some(working_memory_config),
        semantic_recall: None,
        last_messages: None,
        query: None,
    };

    // 创建带记忆的 Agent
    let memory_agent = AgentBuilder::new()
        .name("memory_agent")
        .instructions("你是一个有记忆的助手，能记住之前的对话内容，并在后续对话中引用这些信息。请在回复中体现出你记住了用户的信息。")
        .model(llm_provider)
        .memory_config(memory_config)
        .build()?;

    // 模拟多轮对话
    println!("开始多轮对话演示:");

    let conversations = vec![
        "我叫张三，今年25岁",
        "我的爱好是编程和阅读",
        "我特别喜欢Rust编程语言",
        "请告诉我，你还记得我的名字和年龄吗？",
        "请总结一下你对我的了解",
    ];

    for (i, input) in conversations.iter().enumerate() {
        let response = memory_agent.generate_simple(input).await?;
        println!("\n第{}轮对话:", i + 1);
        println!("用户: {}", input);
        println!("AI: {}", response);

        // 显示当前记忆状态
        if let Some(_memory) = memory_agent.get_memory() {
            println!("记忆状态: 已配置内存系统");
        }
    }

    Ok(())
}

/// 演示记忆管理功能
async fn demo_memory_management() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示3: 记忆管理功能 ===");

    // 创建记忆配置
    let memory_config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1000), // 较小的限制用于演示
    };

    let working_memory = create_working_memory(&memory_config)?;

    // 添加多个消息来触发记忆管理
    let test_messages = vec![
        ("用户", "第一条消息"),
        ("助手", "我收到了你的第一条消息"),
        ("用户", "第二条消息"),
        ("助手", "我收到了你的第二条消息"),
        ("用户", "第三条消息"),
        ("助手", "我收到了你的第三条消息"),
        ("用户", "第四条消息"),
        ("助手", "我收到了你的第四条消息"),
        ("用户", "第五条消息"),
        ("助手", "我收到了你的第五条消息"),
        ("用户", "第六条消息"), // 这条应该触发记忆管理
    ];

    for (i, (role, content)) in test_messages.iter().enumerate() {
        let message_role = if *role == "用户" {
            Role::User
        } else {
            Role::Assistant
        };
        let message = Message::new(message_role, content.to_string(), None, None);

        // 将消息添加到工作内存
        let mut memory_content = working_memory.get().await?;
        let mut messages_array = if let Some(msgs) = memory_content.content.get("messages") {
            msgs.as_array().unwrap_or(&vec![]).clone()
        } else {
            vec![]
        };

        messages_array.push(serde_json::to_value(&message)?);

        if let serde_json::Value::Object(ref mut map) = memory_content.content {
            map.insert(
                "messages".to_string(),
                serde_json::Value::Array(messages_array.clone()),
            );
        }

        working_memory.update(memory_content).await?;

        println!("添加第{}条消息后: {} 消息", i + 1, messages_array.len());

        // 检查是否触发了记忆管理
        if messages_array.len() < i + 1 {
            println!("  ⚠️ 触发了记忆管理，旧消息被清理或总结");
        }
    }

    // 显示最终记忆内容
    println!("\n最终记忆内容:");
    let final_content = working_memory.get().await?;
    if let Some(messages) = final_content.content.get("messages") {
        if let Some(msgs_array) = messages.as_array() {
            for (i, msg_value) in msgs_array.iter().enumerate() {
                if let Ok(message) = serde_json::from_value::<Message>(msg_value.clone()) {
                    println!("  {}. {:?}: {}", i + 1, message.role, message.content);
                }
            }
        }
    }

    Ok(())
}

/// 演示记忆检索和总结
async fn demo_memory_retrieval() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示4: 记忆检索和总结 ===");

    // 创建记忆配置
    let memory_config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(3000),
    };

    let working_memory = create_working_memory(&memory_config)?;

    // 添加一个完整的对话历史
    let conversation_history = vec![
        (Role::User, "你好，我是李四"),
        (Role::Assistant, "你好李四！很高兴认识你。"),
        (Role::User, "我是一名软件工程师"),
        (Role::Assistant, "太好了！软件工程是一个很有前景的领域。"),
        (Role::User, "我主要做前端开发，使用React和TypeScript"),
        (Role::Assistant, "React和TypeScript是很好的技术选择！"),
        (Role::User, "我最近在学习Rust"),
        (Role::Assistant, "Rust是一门很棒的语言，特别适合系统编程。"),
        (Role::User, "我想用Rust开发一个Web服务"),
        (Role::Assistant, "可以考虑使用Axum或Warp框架。"),
        (Role::User, "你能推荐一些学习资源吗？"),
        (Role::Assistant, "推荐《Rust程序设计语言》和官方文档。"),
    ];

    // 将对话历史添加到工作内存
    let mut memory_content = working_memory.get().await?;
    let mut messages_array = vec![];

    for (role, content) in conversation_history {
        let message = Message::new(role, content.to_string(), None, None);
        messages_array.push(serde_json::to_value(&message)?);
    }

    if let serde_json::Value::Object(ref mut map) = memory_content.content {
        map.insert(
            "messages".to_string(),
            serde_json::Value::Array(messages_array.clone()),
        );
    }

    working_memory.update(memory_content).await?;

    println!("对话历史已添加到记忆中");
    println!("总消息数: {}", messages_array.len());

    // 检索最近的消息
    println!("\n最近5条消息:");
    let recent_count = std::cmp::min(5, messages_array.len());
    let recent_messages = &messages_array[messages_array.len() - recent_count..];
    for (i, msg_value) in recent_messages.iter().enumerate() {
        if let Ok(message) = serde_json::from_value::<Message>(msg_value.clone()) {
            println!("  {}. {:?}: {}", i + 1, message.role, message.content);
        }
    }

    // 检索所有消息
    println!("\n所有消息:");
    for (i, msg_value) in messages_array.iter().enumerate() {
        if let Ok(message) = serde_json::from_value::<Message>(msg_value.clone()) {
            println!("  {}. {:?}: {}", i + 1, message.role, message.content);
        }
    }

    // 模拟记忆总结
    println!("\n记忆总结:");
    let all_messages: Vec<Message> = messages_array
        .iter()
        .filter_map(|v| serde_json::from_value(v.clone()).ok())
        .collect();
    let summary = generate_memory_summary(&all_messages);
    println!("{}", summary);

    // 清理记忆
    println!("\n清理记忆...");
    working_memory.clear().await?;
    let cleared_content = working_memory.get().await?;
    let cleared_count = if let Some(msgs) = cleared_content.content.get("messages") {
        msgs.as_array().map(|arr| arr.len()).unwrap_or(0)
    } else {
        0
    };
    println!("记忆已清理，当前消息数: {}", cleared_count);

    Ok(())
}

/// 生成记忆总结
fn generate_memory_summary(messages: &[Message]) -> String {
    let user_messages: Vec<&Message> = messages
        .iter()
        .filter(|m| matches!(m.role, Role::User))
        .collect();

    let assistant_messages: Vec<&Message> = messages
        .iter()
        .filter(|m| matches!(m.role, Role::Assistant))
        .collect();

    format!(
        "对话总结:\n\
        - 总消息数: {}\n\
        - 用户消息: {}\n\
        - 助手消息: {}\n\
        - 主要话题: 用户李四是软件工程师，主要做前端开发，正在学习Rust\n\
        - 技术栈: React, TypeScript, Rust\n\
        - 学习目标: 使用Rust开发Web服务",
        messages.len(),
        user_messages.len(),
        assistant_messages.len()
    )
}

/// 辅助函数：创建测试消息
#[allow(dead_code)]
fn create_test_message(role: Role, content: &str) -> Message {
    Message::new(role, content.to_string(), None, None)
}

/// 辅助函数：打印记忆统计
#[allow(dead_code)]
async fn print_memory_stats(
    memory: &dyn lumosai_core::memory::working::WorkingMemory,
    label: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let content = memory.get().await?;
    let message_count = if let Some(msgs) = content.content.get("messages") {
        msgs.as_array().map(|arr| arr.len()).unwrap_or(0)
    } else {
        0
    };
    println!("{}: {} 消息", label, message_count);
    Ok(())
}
