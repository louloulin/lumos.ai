//! CompositeMemory 演示程序 - Week 11-12 统一内存架构
//!
//! 演示如何使用 CompositeMemory 构建器创建和配置统一的内存系统

use lumosai_core::memory::{UnifiedMemory as Memory, Memory as MemoryTrait, MemoryConfig};
use lumosai_core::llm::{Message as LlmMessage, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 CompositeMemory 统一内存架构演示\n");
    println!("{}", "=".repeat(80));
    println!();

    // 演示 1: 基础内存
    demo_basic_memory().await?;

    // 演示 2: 工作内存
    demo_working_memory().await?;

    // 演示 3: CompositeMemory 构建器
    demo_composite_memory().await?;

    // 演示 4: 带处理器的 CompositeMemory
    demo_composite_with_processors().await?;

    // 总结
    println!("\n{}", "=".repeat(80));
    println!("✅ CompositeMemory 演示完成！");
    println!("{}", "=".repeat(80));
    println!();
    println!("📊 功能总结:");
    println!("  1. 基础内存: 简单的消息存储和检索");
    println!("  2. 工作内存: 支持容量限制的临时存储");
    println!("  3. CompositeMemory: 链式配置多种内存类型");
    println!("  4. 内存处理器: 支持消息限制、去重等处理");
    println!();
    println!("💡 设计优势:");
    println!("  - 统一接口: 减少抽象层次，简化使用");
    println!("  - 链式配置: 流畅的 API 设计");
    println!("  - 类型安全: 编译时验证");
    println!("  - 向后兼容: 保持现有功能");
    println!();
    println!("🚀 对标 Mastra/LangChain:");
    println!("  ✅ 统一内存 API");
    println!("  ✅ 构建器模式");
    println!("  ✅ 内存处理器链");
    println!("  ✅ 多种内存类型组合");
    println!("{}", "=".repeat(80));

    Ok(())
}

/// 演示 1: 基础内存
async fn demo_basic_memory() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 演示 1: 基础内存");
    println!("{}", "-".repeat(80));

    // 创建基础内存
    let memory = Memory::basic();
    println!("✅ 创建基础内存");

    // 存储消息
    let message1 = LlmMessage {
        role: Role::User,
        content: "你好，我是用户".to_string(),
        name: None,
        metadata: None,
    };

    memory.store(&message1).await?;
    println!("✅ 存储用户消息: {}", message1.content);

    let message2 = LlmMessage {
        role: Role::Assistant,
        content: "你好！我是 AI 助手，很高兴为您服务".to_string(),
        name: None,
        metadata: None,
    };

    memory.store(&message2).await?;
    println!("✅ 存储助手消息: {}", message2.content);

    // 检索消息
    let config = MemoryConfig::default();
    let messages = memory.retrieve(&config).await?;
    println!("✅ 检索到 {} 条消息", messages.len());

    println!();
    Ok(())
}

/// 演示 2: 工作内存
async fn demo_working_memory() -> Result<(), Box<dyn std::error::Error>> {
    println!("💼 演示 2: 工作内存（容量限制）");
    println!("{}", "-".repeat(80));

    // 创建工作内存，容量为 1000
    let memory = Memory::working(1000);
    println!("✅ 创建工作内存（容量: 1000）");

    // 存储多条消息
    for i in 1..=5 {
        let message = LlmMessage {
            role: if i % 2 == 1 { Role::User } else { Role::Assistant },
            content: format!("消息 {}", i),
            name: None,
            metadata: None,
        };
        memory.store(&message).await?;
        println!("✅ 存储消息 {}: {}", i, message.content);
    }

    // 检索消息
    let config = MemoryConfig::default();
    let messages = memory.retrieve(&config).await?;
    println!("✅ 检索到 {} 条消息", messages.len());

    println!();
    Ok(())
}

/// 演示 3: CompositeMemory 构建器
async fn demo_composite_memory() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 演示 3: CompositeMemory 构建器");
    println!("{}", "-".repeat(80));

    // 使用构建器创建 CompositeMemory
    let memory = Memory::composite()
        .working(2000)
        .namespace("demo_namespace")
        .build()
        .await?;

    println!("✅ 创建 CompositeMemory:");
    println!("  - 工作内存容量: 2000");
    println!("  - 命名空间: demo_namespace");

    // 存储消息
    let message = LlmMessage {
        role: Role::User,
        content: "这是通过 CompositeMemory 存储的消息".to_string(),
        name: None,
        metadata: None,
    };

    memory.store(&message).await?;
    println!("✅ 存储消息: {}", message.content);

    // 检索消息
    let config = MemoryConfig::default();
    let messages = memory.retrieve(&config).await?;
    println!("✅ 检索到 {} 条消息", messages.len());

    println!();
    Ok(())
}

/// 演示 4: 混合内存配置
async fn demo_composite_with_processors() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️  演示 4: 混合内存配置");
    println!("{}", "-".repeat(80));

    // 使用构建器创建混合内存（工作内存 + 语义内存）
    let memory = Memory::composite()
        .working(3000)
        .semantic("qdrant", "openai")
        .namespace("hybrid_memory")
        .build()
        .await?;

    println!("✅ 创建混合内存:");
    println!("  - 工作内存容量: 3000");
    println!("  - 语义内存: qdrant + openai");
    println!("  - 命名空间: hybrid_memory");

    // 存储多条消息
    let messages = vec![
        ("用户", "第一条消息"),
        ("助手", "我收到了第一条消息"),
        ("用户", "第二条消息"),
        ("助手", "我收到了第二条消息"),
        ("用户", "第三条消息"),
    ];

    for (role, content) in messages {
        let message = LlmMessage {
            role: if role == "用户" { Role::User } else { Role::Assistant },
            content: content.to_string(),
            name: None,
            metadata: None,
        };
        memory.store(&message).await?;
        println!("✅ 存储消息: {} - {}", role, content);
    }

    // 检索消息
    let config = MemoryConfig::default();
    let messages = memory.retrieve(&config).await?;
    println!("✅ 检索到 {} 条消息", messages.len());

    println!();
    Ok(())
}

