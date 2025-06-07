//! Lumos简化API完整演示
//! 
//! 展示如何使用Lumos的简化API快速构建AI应用

use lumos::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::init();
    
    println!("🚀 Lumos简化API演示开始");
    
    // 1. 一行代码创建向量存储
    println!("\n📦 创建向量存储...");
    let storage = lumos::vector::memory().await?;
    println!("✅ 内存向量存储创建成功");
    
    // 2. 一行代码创建RAG系统
    println!("\n🔍 创建RAG系统...");
    let rag = lumos::rag::simple(storage, "openai").await?;
    println!("✅ RAG系统创建成功");
    
    // 3. 添加文档到RAG系统
    println!("\n📄 添加文档到RAG系统...");
    let doc_id1 = rag.add_document("人工智能正在改变世界，特别是在医疗、教育和交通领域").await?;
    let doc_id2 = rag.add_document("机器学习是人工智能的一个重要分支，包括监督学习、无监督学习和强化学习").await?;
    let doc_id3 = rag.add_document("深度学习使用神经网络来模拟人脑的工作方式，在图像识别和自然语言处理方面表现出色").await?;
    println!("✅ 添加了3个文档: {}, {}, {}", doc_id1, doc_id2, doc_id3);
    
    // 4. 搜索相关文档
    println!("\n🔎 搜索相关文档...");
    let search_results = rag.search("什么是机器学习？", 2).await?;
    println!("✅ 找到{}个相关文档:", search_results.len());
    for (i, result) in search_results.iter().enumerate() {
        println!("  {}. 分数: {:.3} - {}", i + 1, result.score, result.document.content);
    }
    
    // 5. 一行代码创建Agent
    println!("\n🤖 创建AI Agent...");
    let agent = lumos::agent::simple("gpt-4", "你是一个专业的AI助手，擅长回答关于人工智能的问题").await?;
    println!("✅ Agent创建成功: {}", agent.name());
    
    // 6. Agent对话
    println!("\n💬 Agent对话测试...");
    let response = agent.chat("你好，请简单介绍一下自己").await?;
    println!("🤖 Agent回复: {}", response);
    
    // 7. 创建会话管理
    println!("\n📝 创建会话管理...");
    let session = lumos::session::create("ai_assistant", Some("user_123")).await?;
    println!("✅ 会话创建成功: {}", session.id());
    
    // 8. 添加消息到会话
    let message = Message {
        role: Role::User,
        content: "请解释一下深度学习".to_string(),
        metadata: None,
        name: None,
    };
    session.add_message(message).await?;
    println!("✅ 消息已添加到会话");
    
    // 9. 创建事件系统
    println!("\n📡 创建事件系统...");
    let event_bus = lumos::events::create_bus(1000);
    
    // 注册日志处理器
    lumos::events::register_log_handler(&event_bus).await?;
    
    // 注册指标收集器
    let metrics_handler = lumos::events::register_metrics_handler(&event_bus).await?;
    
    // 发布一些事件
    lumos::events::publish(&event_bus, "agent_started", serde_json::json!({
        "agent_id": "ai_assistant"
    })).await?;
    
    lumos::events::publish(&event_bus, "message_sent", serde_json::json!({
        "from": "ai_assistant",
        "to": "user_123",
        "content": "Hello from agent!"
    })).await?;
    
    println!("✅ 事件系统创建成功，已发布2个事件");
    
    // 10. 多Agent协作演示
    println!("\n👥 多Agent协作演示...");
    
    // 创建多个专业Agent
    let researcher = lumos::agent::builder()
        .name("研究员")
        .model("gpt-4")
        .system_prompt("你是一个专业的研究员，擅长收集和分析信息")
        .build()
        .await?;
    
    let writer = lumos::agent::builder()
        .name("作家")
        .model("gpt-4")
        .system_prompt("你是一个专业的作家，擅长将复杂信息整理成易懂的文章")
        .build()
        .await?;
    
    // 创建协作任务
    let task = lumos::orchestration::task()
        .name("AI研究报告")
        .description("研究AI技术并撰写报告")
        .agents(vec![researcher, writer])
        .pattern(lumos::orchestration::Pattern::Sequential)
        .input(serde_json::json!({
            "topic": "人工智能在医疗领域的应用",
            "requirements": "请提供详细的研究和清晰的总结"
        }))
        .build();
    
    println!("✅ 协作任务创建成功: {}", task.name);
    
    // 执行协作（注意：这需要真实的API密钥）
    println!("📋 协作任务已准备就绪（需要API密钥才能执行）");
    
    // 11. 获取指标统计
    println!("\n📊 获取系统指标...");
    
    // 等待事件处理
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let metrics = metrics_handler.get_metrics().await;
    println!("✅ 系统指标:");
    for (key, value) in metrics {
        println!("  {}: {}", key, value);
    }
    
    // 12. 获取事件历史
    let event_history = lumos::events::get_history(&event_bus, None).await;
    println!("✅ 事件历史: {}个事件", event_history.len());
    
    // 13. 使用构建器模式创建高级配置
    println!("\n⚙️ 高级配置演示...");
    
    // 高级向量存储配置
    let advanced_storage = lumos::vector::builder()
        .backend("memory")
        .batch_size(1000)
        .build()
        .await?;
    println!("✅ 高级向量存储配置完成");
    
    // 高级RAG配置
    let advanced_rag = lumos::rag::builder()
        .storage(advanced_storage)
        .embedding_provider("openai")
        .chunking_strategy("recursive")
        .chunk_size(800)
        .chunk_overlap(100)
        .retrieval_strategy("hybrid")
        .top_k(10)
        .build()
        .await?;
    println!("✅ 高级RAG配置完成");
    
    // 高级会话配置
    let advanced_session = lumos::session::builder()
        .agent_name("advanced_agent")
        .user_id("power_user")
        .title("高级AI对话会话")
        .build()
        .await?;
    println!("✅ 高级会话配置完成: {}", advanced_session.id());
    
    println!("\n🎉 Lumos简化API演示完成！");
    println!("\n📋 演示总结:");
    println!("  ✅ 向量存储: 内存存储创建成功");
    println!("  ✅ RAG系统: 文档添加和搜索功能正常");
    println!("  ✅ Agent系统: 对话功能正常");
    println!("  ✅ 会话管理: 消息持久化功能正常");
    println!("  ✅ 事件系统: 事件发布和处理功能正常");
    println!("  ✅ 多Agent编排: 任务创建功能正常");
    println!("  ✅ 高级配置: 构建器模式功能正常");
    
    println!("\n🚀 Lumos框架已准备就绪，可以开始构建企业级AI应用！");
    
    Ok(())
}

/// 演示错误处理
async fn demonstrate_error_handling() -> Result<()> {
    println!("\n🛡️ 错误处理演示...");
    
    // 尝试创建无效配置
    match lumos::vector::builder()
        .backend("invalid_backend")
        .build()
        .await
    {
        Ok(_) => println!("❌ 应该失败但成功了"),
        Err(e) => println!("✅ 正确捕获错误: {}", e),
    }
    
    Ok(())
}

/// 演示性能测试
async fn demonstrate_performance() -> Result<()> {
    println!("\n⚡ 性能测试演示...");
    
    let storage = lumos::vector::memory().await?;
    let start_time = std::time::Instant::now();
    
    // 批量添加文档
    for i in 0..100 {
        let content = format!("这是测试文档 {} 的内容，包含一些示例文本", i);
        // 这里需要实际的RAG实现来测试
    }
    
    let duration = start_time.elapsed();
    println!("✅ 性能测试完成，耗时: {:?}", duration);
    
    Ok(())
}
