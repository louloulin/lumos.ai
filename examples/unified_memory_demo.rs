//! LumosAI v2.0 统一内存系统演示
//!
//! 这个示例演示了第四周任务的成果：统一内存系统
//! 
//! 运行方式：
//! ```bash
//! cargo run --example unified_memory_demo
//! ```

use lumosai_core::error::Result;
use lumosai_core::llm::types::{user_message, assistant_message};
use lumosai_core::memory::UnifiedMemory;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧠 LumosAI 统一内存系统演示");
    println!("============================\n");

    // 演示1: 基础内存
    println!("📝 演示1: 基础内存");
    println!("------------------");
    demo_basic_memory().await?;
    println!();

    // 演示2: 工作内存
    println!("💾 演示2: 工作内存");
    println!("------------------");
    demo_working_memory().await?;
    println!();

    // 演示3: 混合内存
    println!("🔄 演示3: 混合内存");
    println!("------------------");
    demo_hybrid_memory().await?;
    println!();

    // 演示4: 内存统计和管理
    println!("📊 演示4: 内存统计和管理");
    println!("------------------------");
    demo_memory_stats().await?;
    println!();

    println!("🎉 统一内存系统演示完成！");
    println!("\n📋 总结:");
    println!("  - ✅ 基础内存: 简单易用的消息存储");
    println!("  - ✅ 工作内存: 支持容量限制的临时存储");
    println!("  - ✅ 混合内存: 结合多种内存类型的优势");
    println!("  - ✅ 统一接口: 一致的API设计");
    println!("  - ✅ 智能默认值: 开箱即用的配置");
    println!("  - ✅ 便利方法: 简化常用操作");

    Ok(())
}

/// 演示基础内存的使用
async fn demo_basic_memory() -> Result<()> {
    // 创建基础内存 - 一行代码即可
    let memory = UnifiedMemory::basic();
    
    println!("✅ 基础内存创建成功");
    println!("🎯 内存类型: {:?}", memory.memory_type());

    // 添加一些消息
    let messages = vec![
        user_message("你好，我是用户"),
        assistant_message("你好！我是AI助手，很高兴为您服务"),
        user_message("请介绍一下LumosAI"),
        assistant_message("LumosAI是一个企业级AI应用开发框架，使用Rust构建"),
    ];

    for (i, message) in messages.iter().enumerate() {
        memory.add_message(message.clone()).await?;
        println!("📝 已存储消息 {}: {}", i + 1,
            message.content.chars().take(30).collect::<String>() + "...");
    }

    // 检索最近的消息
    let recent = memory.get_recent_messages(2).await?;
    println!("🔍 检索到 {} 条最近消息", recent.len());

    // 检查内存统计
    let stats = memory.get_stats().await?;
    println!("📊 {}", stats);

    Ok(())
}

/// 演示工作内存的使用
async fn demo_working_memory() -> Result<()> {
    // 创建工作内存，指定容量为100
    let memory = UnifiedMemory::working(100);
    
    println!("✅ 工作内存创建成功 (容量: 100)");
    println!("🎯 内存类型: {:?}", memory.memory_type());

    // 添加消息
    let message = user_message("这是一条存储在工作内存中的消息");
    memory.add_message(message.clone()).await?;
    println!("📝 已存储消息: {}", message.content);

    // 检索消息
    let retrieved = memory.get_recent_messages(1).await?;
    if let Some(msg) = retrieved.first() {
        println!("🔍 检索到消息: {}", msg.content);
    }

    // 检查是否为空
    let is_empty = memory.is_empty().await?;
    println!("❓ 内存是否为空: {}", if is_empty { "是" } else { "否" });

    // 清空工作内存
    match memory.clear().await {
        Ok(_) => println!("🗑️ 工作内存已清空"),
        Err(e) => println!("⚠️ 清空失败: {}", e),
    }

    // 再次检查是否为空
    let is_empty_after_clear = memory.is_empty().await?;
    println!("❓ 清空后内存是否为空: {}", if is_empty_after_clear { "是" } else { "否" });

    Ok(())
}

/// 演示混合内存的使用
async fn demo_hybrid_memory() -> Result<()> {
    // 创建混合内存：工作内存大小为500，不启用语义内存
    let memory = UnifiedMemory::hybrid(Some(500), false);
    
    println!("✅ 混合内存创建成功 (工作内存: 500, 语义内存: 禁用)");
    println!("🎯 内存类型: {:?}", memory.memory_type());

    // 添加多条消息
    let messages = vec![
        user_message("混合内存测试消息1"),
        assistant_message("这是助手的回复1"),
        user_message("混合内存测试消息2"),
        assistant_message("这是助手的回复2"),
        user_message("混合内存测试消息3"),
    ];

    for (i, message) in messages.iter().enumerate() {
        memory.add_message(message.clone()).await?;
        println!("📝 已存储到混合内存 {}: {}", i + 1, message.content);
    }

    // 检索所有消息
    let all_messages = memory.get_recent_messages(10).await?;
    println!("🔍 从混合内存检索到 {} 条消息", all_messages.len());

    // 显示统计信息
    let stats = memory.get_stats().await?;
    println!("📊 {}", stats);

    Ok(())
}

/// 演示内存统计和管理功能
async fn demo_memory_stats() -> Result<()> {
    println!("创建不同类型的内存并比较统计信息...\n");

    // 创建三种不同类型的内存
    let basic_memory = UnifiedMemory::basic();
    let working_memory = UnifiedMemory::working(200);
    let hybrid_memory = UnifiedMemory::hybrid(Some(300), false);

    let memories = vec![
        ("基础内存", &basic_memory),
        ("工作内存", &working_memory),
        ("混合内存", &hybrid_memory),
    ];

    // 为每种内存添加测试数据
    for (name, memory) in &memories {
        println!("📝 为{}添加测试数据...", name);
        
        for i in 1..=3 {
            let message = user_message(&format!("{}的测试消息{}", name, i));
            memory.add_message(message).await?;
        }

        // 显示统计信息
        let stats = memory.get_stats().await?;
        println!("📊 {}: {}", name, stats);
        
        // 检查是否为空
        let is_empty = memory.is_empty().await?;
        println!("❓ {}是否为空: {}\n", name, if is_empty { "是" } else { "否" });
    }

    // 演示内存类型比较
    println!("🔍 内存类型比较:");
    for (name, memory) in &memories {
        println!("  - {}: {:?}", name, memory.memory_type());
    }

    Ok(())
}
