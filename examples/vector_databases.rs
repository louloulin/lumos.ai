//! 真实向量数据库集成示例
//! 
//! 本示例展示如何使用Lumos与真实的向量数据库（Qdrant、Weaviate、PostgreSQL）进行集成

use lumos::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::init();
    
    println!("🚀 Lumos Vector Database Integration Examples");
    println!("==============================================");
    
    // 1. 自动选择最佳向量数据库
    demo_auto_selection().await?;
    
    // 2. 手动指定向量数据库
    demo_manual_selection().await?;
    
    // 3. 使用构建器模式
    demo_builder_pattern().await?;
    
    // 4. 向量数据库特性对比
    demo_feature_comparison().await?;
    
    Ok(())
}

/// 演示自动选择向量数据库
async fn demo_auto_selection() -> Result<()> {
    println!("\n📋 1. 自动选择向量数据库");
    println!("------------------------");
    
    // 自动选择最佳可用的向量数据库
    let storage = lumos::vector::auto().await?;
    let backend_info = storage.backend_info();
    
    println!("✅ 自动选择的后端: {} v{}", backend_info.name, backend_info.version);
    println!("🔧 支持的特性: {:?}", backend_info.features);
    
    // 测试基本操作
    storage.health_check().await?;
    println!("💚 健康检查通过");
    
    Ok(())
}

/// 演示手动选择向量数据库
async fn demo_manual_selection() -> Result<()> {
    println!("\n🎯 2. 手动选择向量数据库");
    println!("-------------------------");
    
    // 内存存储（总是可用）
    println!("📝 测试内存存储...");
    let memory_storage = lumos::vector::memory().await?;
    println!("✅ 内存存储创建成功");
    
    // Qdrant（如果可用）
    if let Ok(qdrant_url) = env::var("QDRANT_URL") {
        println!("🔍 测试Qdrant存储...");
        match lumos::vector::qdrant(&qdrant_url).await {
            Ok(_) => println!("✅ Qdrant存储连接成功: {}", qdrant_url),
            Err(e) => println!("❌ Qdrant连接失败: {}", e),
        }
    } else {
        println!("⏭️  跳过Qdrant测试 (未设置QDRANT_URL)");
        println!("   提示: 设置环境变量 QDRANT_URL=http://localhost:6334");
    }
    
    // Weaviate（如果可用）
    if let Ok(weaviate_url) = env::var("WEAVIATE_URL") {
        println!("🕸️  测试Weaviate存储...");
        match lumos::vector::weaviate(&weaviate_url).await {
            Ok(_) => println!("✅ Weaviate存储连接成功: {}", weaviate_url),
            Err(e) => println!("❌ Weaviate连接失败: {}", e),
        }
    } else {
        println!("⏭️  跳过Weaviate测试 (未设置WEAVIATE_URL)");
        println!("   提示: 设置环境变量 WEAVIATE_URL=http://localhost:8080");
    }
    
    // PostgreSQL（如果可用）
    if env::var("DATABASE_URL").is_ok() || env::var("POSTGRES_PASSWORD").is_ok() {
        println!("🐘 测试PostgreSQL存储...");
        match lumos::vector::postgres().await {
            Ok(_) => println!("✅ PostgreSQL存储连接成功"),
            Err(e) => println!("❌ PostgreSQL连接失败: {}", e),
        }
    } else {
        println!("⏭️  跳过PostgreSQL测试 (未设置数据库环境变量)");
        println!("   提示: 设置 DATABASE_URL 或 POSTGRES_* 环境变量");
    }
    
    Ok(())
}

/// 演示构建器模式
async fn demo_builder_pattern() -> Result<()> {
    println!("\n🏗️  3. 构建器模式");
    println!("----------------");
    
    // 使用构建器创建内存存储
    let storage = lumos::vector::builder()
        .backend("memory")
        .batch_size(1000)
        .build()?;
    
    println!("✅ 使用构建器创建内存存储成功");
    
    // 如果有Qdrant URL，尝试构建Qdrant存储
    if let Ok(qdrant_url) = env::var("QDRANT_URL") {
        match lumos::vector::builder()
            .backend("qdrant")
            .url(&qdrant_url)
            .batch_size(500)
            .build()
        {
            Ok(_) => println!("✅ 使用构建器创建Qdrant存储成功"),
            Err(e) => println!("❌ 构建器创建Qdrant存储失败: {}", e),
        }
    }
    
    // 如果有Weaviate URL，尝试构建Weaviate存储
    if let Ok(weaviate_url) = env::var("WEAVIATE_URL") {
        match lumos::vector::builder()
            .backend("weaviate")
            .url(&weaviate_url)
            .batch_size(200)
            .build()
        {
            Ok(_) => println!("✅ 使用构建器创建Weaviate存储成功"),
            Err(e) => println!("❌ 构建器创建Weaviate存储失败: {}", e),
        }
    }
    
    Ok(())
}

/// 演示向量数据库特性对比
async fn demo_feature_comparison() -> Result<()> {
    println!("\n📊 4. 向量数据库特性对比");
    println!("------------------------");
    
    let mut backends = Vec::new();
    
    // 收集所有可用的后端信息
    backends.push(("Memory", lumos::vector::memory().await?));
    
    if let Ok(url) = env::var("QDRANT_URL") {
        if let Ok(storage) = lumos::vector::qdrant(&url).await {
            backends.push(("Qdrant", storage));
        }
    }
    
    if let Ok(url) = env::var("WEAVIATE_URL") {
        if let Ok(storage) = lumos::vector::weaviate(&url).await {
            backends.push(("Weaviate", storage));
        }
    }
    
    if let Ok(storage) = lumos::vector::postgres().await {
        backends.push(("PostgreSQL", storage));
    }
    
    // 显示对比表
    println!("┌─────────────┬─────────┬──────────────────────────────────┐");
    println!("│ 后端        │ 版本    │ 特性                             │");
    println!("├─────────────┼─────────┼──────────────────────────────────┤");
    
    for (name, storage) in backends {
        let info = storage.backend_info();
        let features = info.features.join(", ");
        println!("│ {:11} │ {:7} │ {:32} │", name, info.version, 
                 if features.len() > 32 { &features[..29].to_string() + "..." } else { &features });
    }
    
    println!("└─────────────┴─────────┴──────────────────────────────────┘");
    
    Ok(())
}

/// 演示实际的向量操作（需要真实数据库）
#[allow(dead_code)]
async fn demo_vector_operations() -> Result<()> {
    println!("\n🔬 5. 实际向量操作演示");
    println!("---------------------");
    
    // 获取一个可用的存储
    let storage = lumos::vector::auto().await?;
    
    // 创建索引
    let index_config = IndexConfig::new("demo_docs", 384)
        .with_metric(SimilarityMetric::Cosine);
    
    // 注意：这会修改数据库状态，在示例中谨慎使用
    if let Err(e) = storage.create_index(index_config).await {
        println!("⚠️  创建索引失败（可能已存在）: {}", e);
    } else {
        println!("✅ 创建索引成功");
    }
    
    // 插入文档
    let documents = vec![
        Document::new("doc1", "Hello world")
            .with_embedding(vec![0.1; 384])
            .with_metadata("category", "greeting"),
        Document::new("doc2", "Goodbye world")
            .with_embedding(vec![0.2; 384])
            .with_metadata("category", "farewell"),
    ];
    
    match storage.upsert_documents("demo_docs", documents).await {
        Ok(ids) => println!("✅ 插入文档成功: {:?}", ids),
        Err(e) => println!("❌ 插入文档失败: {}", e),
    }
    
    // 搜索
    let search_request = SearchRequest::new("demo_docs", vec![0.15; 384])
        .with_top_k(5);
    
    match storage.search(search_request).await {
        Ok(results) => {
            println!("✅ 搜索成功，找到 {} 个结果", results.results.len());
            for (i, result) in results.results.iter().enumerate() {
                println!("  {}. ID: {}, Score: {:.4}", i + 1, result.id, result.score);
            }
        }
        Err(e) => println!("❌ 搜索失败: {}", e),
    }
    
    Ok(())
}
