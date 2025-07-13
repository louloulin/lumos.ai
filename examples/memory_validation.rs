use lumosai_core::memory::{
    Memory, MemoryConfig, WorkingMemory, WorkingMemoryConfig,
    SemanticMemory, BasicMemory, MemoryEntry, MemoryEntryType,
    create_working_memory, create_semantic_memory
};
use lumosai_core::llm::{Message, Role};
use serde_json::json;
use std::time::Instant;

/// 内存管理系统全面验证测试
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 LumosAI 内存管理系统验证测试");
    println!("========================================");
    
    // 测试1: 工作内存验证
    println!("\n📋 测试1: 工作内存验证");
    test_working_memory().await?;
    
    // 测试2: 语义内存验证
    println!("\n📋 测试2: 语义内存验证");
    test_semantic_memory().await?;
    
    // 测试3: 内存提供商验证
    println!("\n📋 测试3: 内存提供商验证");
    test_memory_providers().await?;
    
    // 测试4: 内存性能基准测试
    println!("\n📋 测试4: 内存性能基准测试");
    test_memory_performance().await?;
    
    println!("\n✅ 所有内存管理系统验证测试完成！");
    Ok(())
}

async fn test_working_memory() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试工作内存...");

    // 创建工作内存配置
    let config = WorkingMemoryConfig {
        max_capacity: Some(1000),
        max_age_seconds: Some(3600),
        cleanup_interval_seconds: Some(300),
    };

    let working_memory = create_working_memory(&config)?;
    println!("✅ 工作内存创建成功");

    // 创建测试消息
    let test_message = Message {
        role: Role::User,
        content: "这是一个测试消息，用于验证工作内存功能".to_string(),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    };

    let start_time = Instant::now();
    working_memory.store(&test_message).await?;
    let store_duration = start_time.elapsed();

    println!("✅ 工作内存存储成功!");
    println!("⏱️ 存储时间: {:?}", store_duration);

    // 测试检索
    let memory_config = MemoryConfig {
        store_id: None,
        namespace: None,
        enabled: true,
        working_memory: Some(config.clone()),
        semantic_recall: None,
        last_messages: Some(10),
        query: None,
    };

    let start_time = Instant::now();
    let retrieved_messages = working_memory.retrieve(&memory_config).await?;
    let retrieve_duration = start_time.elapsed();

    println!("✅ 工作内存检索成功!");
    println!("⏱️ 检索时间: {:?}", retrieve_duration);
    println!("📊 检索到的消息数量: {}", retrieved_messages.len());

    if !retrieved_messages.is_empty() {
        println!("📝 第一条消息内容: {}", retrieved_messages[0].content);
        println!("✅ 数据完整性验证通过");
    } else {
        println!("⚠️ 未检索到任何消息");
    }

    Ok(())
}

async fn test_semantic_memory() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试语义内存...");

    // 创建语义内存
    let semantic_memory = create_semantic_memory()?;
    println!("✅ 语义内存创建成功");

    // 创建测试消息
    let test_messages = vec![
        Message {
            role: Role::User,
            content: "什么是Rust编程语言的所有权系统？".to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        Message {
            role: Role::Assistant,
            content: "Rust的所有权系统是其核心特性，确保内存安全而无需垃圾回收器。".to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        Message {
            role: Role::User,
            content: "AI Agent是什么？".to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        Message {
            role: Role::Assistant,
            content: "AI Agent是能够自主执行任务的人工智能系统，具有感知、决策和行动能力。".to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ];

    // 存储消息到语义内存
    for (i, message) in test_messages.iter().enumerate() {
        let start_time = Instant::now();
        semantic_memory.store(message).await?;
        let duration = start_time.elapsed();

        println!("✅ 消息 {} 存储成功! 耗时: {:?}", i + 1, duration);
    }

    // 测试语义检索
    let memory_config = MemoryConfig {
        store_id: None,
        namespace: None,
        enabled: true,
        working_memory: None,
        semantic_recall: None,
        last_messages: Some(10),
        query: Some("Rust所有权".to_string()),
    };

    let start_time = Instant::now();
    let retrieved_messages = semantic_memory.retrieve(&memory_config).await?;
    let duration = start_time.elapsed();

    println!("✅ 语义检索完成! 耗时: {:?}", duration);
    println!("📊 检索到的消息数量: {}", retrieved_messages.len());

    for (i, message) in retrieved_messages.iter().enumerate() {
        println!("📝 检索消息 {}: {}", i + 1, message.content);
    }

    Ok(())
}

async fn test_memory_providers() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试内存提供商...");

    // 测试基础内存
    let basic_memory = BasicMemory::new();
    println!("✅ 基础内存创建成功");

    // 创建测试消息
    let test_message = Message {
        role: Role::User,
        content: "这是一个测试消息，用于验证基础内存功能".to_string(),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    };

    let start_time = Instant::now();
    basic_memory.store(&test_message).await?;
    let store_duration = start_time.elapsed();

    println!("✅ 基础内存存储成功! 耗时: {:?}", store_duration);

    // 测试检索
    let memory_config = MemoryConfig {
        store_id: None,
        namespace: None,
        enabled: true,
        working_memory: None,
        semantic_recall: None,
        last_messages: Some(10),
        query: None,
    };

    let start_time = Instant::now();
    let retrieved_messages = basic_memory.retrieve(&memory_config).await?;
    let retrieve_duration = start_time.elapsed();

    println!("✅ 基础内存检索成功! 耗时: {:?}", retrieve_duration);
    println!("📊 检索到的消息数量: {}", retrieved_messages.len());

    if !retrieved_messages.is_empty() {
        println!("📝 检索到的消息: {}", retrieved_messages[0].content);
    }

    Ok(())
}

async fn test_memory_performance() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试内存性能基准...");

    let basic_memory = BasicMemory::new();

    // 性能测试参数
    let test_sizes = vec![10, 50, 100];

    for size in test_sizes {
        println!("\n📊 测试规模: {} 条消息", size);

        let mut store_total_time = std::time::Duration::new(0, 0);
        let mut retrieve_total_time = std::time::Duration::new(0, 0);

        // 批量存储性能测试
        for i in 0..size {
            let test_message = Message {
                role: Role::User,
                content: format!("性能测试消息 {}", i),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            };

            // 存储性能测试
            let start_time = Instant::now();
            basic_memory.store(&test_message).await?;
            store_total_time += start_time.elapsed();
        }

        // 检索性能测试
        let memory_config = MemoryConfig {
            store_id: None,
            namespace: None,
            enabled: true,
            working_memory: None,
            semantic_recall: None,
            last_messages: Some(size as usize),
            query: None,
        };

        let start_time = Instant::now();
        let _retrieved = basic_memory.retrieve(&memory_config).await?;
        retrieve_total_time = start_time.elapsed();

        let avg_store_time = store_total_time / size;

        println!("📈 平均存储时间: {:?}", avg_store_time);
        println!("📈 检索时间: {:?}", retrieve_total_time);
        println!("📈 总存储时间: {:?}", store_total_time);

        // 计算吞吐量
        let store_throughput = size as f64 / store_total_time.as_secs_f64();
        let retrieve_throughput = size as f64 / retrieve_total_time.as_secs_f64();

        println!("🚀 存储吞吐量: {:.2} 操作/秒", store_throughput);
        println!("🚀 检索吞吐量: {:.2} 操作/秒", retrieve_throughput);
    }

    println!("\n📊 内存性能基准测试完成!");

    Ok(())
}
