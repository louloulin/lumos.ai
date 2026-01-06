use lumosai_core::vector::memory::MemoryVectorStorage;
use lumosai_core::vector::{SimilarityMetric, VectorStorage};
use lumosai_vector::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;

/// 向量数据库系统全面验证测试
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚀 LumosAI 向量数据库系统验证测试");
    println!("========================================");

    // 测试1: 内存向量存储验证
    println!("\n📋 测试1: 内存向量存储验证");
    test_memory_vector_storage().await?;

    // 测试2: 向量搜索验证
    println!("\n📋 测试2: 向量搜索验证");
    test_vector_search().await?;

    // 测试3: 向量性能基准测试
    println!("\n📋 测试3: 向量性能基准测试");
    test_vector_performance().await?;

    println!("\n✅ 所有向量数据库系统验证测试完成！");
    Ok(())
}

async fn test_memory_vector_storage() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试内存向量存储...");

    let vector_storage = MemoryVectorStorage::new(5, Some(1000));
    println!("✅ 内存向量存储创建成功");

    // 创建索引
    let index_name = "test_index";
    let start_time = Instant::now();
    vector_storage
        .create_index(index_name, 5, Some(SimilarityMetric::Cosine))
        .await?;
    let duration = start_time.elapsed();

    println!("✅ 索引创建成功! 耗时: {:?}", duration);

    // 测试向量存储
    let test_vectors = vec![
        vec![0.1, 0.2, 0.3, 0.4, 0.5],
        vec![0.2, 0.3, 0.4, 0.5, 0.6],
        vec![0.3, 0.4, 0.5, 0.6, 0.7],
        vec![0.4, 0.5, 0.6, 0.7, 0.8],
        vec![0.5, 0.6, 0.7, 0.8, 0.9],
    ];

    let test_ids = vec!["doc1", "doc2", "doc3", "doc4", "doc5"];
    let test_metadata: Vec<HashMap<String, serde_json::Value>> = vec![
        [
            ("title".to_string(), json!("文档1")),
            ("content".to_string(), json!("这是第一个测试文档")),
        ]
        .into(),
        [
            ("title".to_string(), json!("文档2")),
            ("content".to_string(), json!("这是第二个测试文档")),
        ]
        .into(),
        [
            ("title".to_string(), json!("文档3")),
            ("content".to_string(), json!("这是第三个测试文档")),
        ]
        .into(),
        [
            ("title".to_string(), json!("文档4")),
            ("content".to_string(), json!("这是第四个测试文档")),
        ]
        .into(),
        [
            ("title".to_string(), json!("文档5")),
            ("content".to_string(), json!("这是第五个测试文档")),
        ]
        .into(),
    ];

    let start_time = Instant::now();
    let inserted_ids = vector_storage
        .upsert(
            index_name,
            test_vectors,
            Some(test_ids.iter().map(|s| s.to_string()).collect()),
            Some(test_metadata),
        )
        .await?;
    let duration = start_time.elapsed();

    println!("✅ 向量批量存储成功! 耗时: {:?}", duration);
    println!("📊 插入的向量数量: {}", inserted_ids.len());

    // 测试索引统计
    let start_time = Instant::now();
    let stats = vector_storage.describe_index(index_name).await?;
    let duration = start_time.elapsed();

    println!("✅ 索引统计获取成功! 耗时: {:?}", duration);
    println!("📊 索引维度: {}", stats.dimension);
    println!("📊 向量数量: {}", stats.count);
    println!("📊 相似度度量: {:?}", stats.metric);

    Ok(())
}

async fn test_vector_search() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试向量搜索...");

    let vector_storage = MemoryVectorStorage::new(5, Some(1000));
    let index_name = "search_index";

    // 创建索引
    vector_storage
        .create_index(index_name, 5, Some(SimilarityMetric::Cosine))
        .await?;

    // 准备测试数据
    let test_vectors = vec![
        vec![1.0, 0.8, 0.6, 0.4, 0.2],
        vec![0.8, 1.0, 0.7, 0.5, 0.3],
        vec![0.6, 0.7, 1.0, 0.8, 0.4],
        vec![0.4, 0.5, 0.8, 1.0, 0.6],
        vec![0.2, 0.3, 0.4, 0.6, 1.0],
    ];

    let test_ids = vec!["rust_doc", "ai_doc", "web_doc", "db_doc", "ml_doc"];
    let test_metadata: Vec<HashMap<String, serde_json::Value>> = vec![
        [
            ("topic".to_string(), json!("Rust编程")),
            ("category".to_string(), json!("programming")),
        ]
        .into(),
        [
            ("topic".to_string(), json!("人工智能")),
            ("category".to_string(), json!("ai")),
        ]
        .into(),
        [
            ("topic".to_string(), json!("Web开发")),
            ("category".to_string(), json!("web")),
        ]
        .into(),
        [
            ("topic".to_string(), json!("数据库")),
            ("category".to_string(), json!("database")),
        ]
        .into(),
        [
            ("topic".to_string(), json!("机器学习")),
            ("category".to_string(), json!("ml")),
        ]
        .into(),
    ];

    // 插入测试数据
    vector_storage
        .upsert(
            index_name,
            test_vectors,
            Some(test_ids.iter().map(|s| s.to_string()).collect()),
            Some(test_metadata),
        )
        .await?;

    println!("✅ 测试数据准备完成");

    // 测试相似性搜索
    let query_vector = vec![0.9, 0.8, 0.7, 0.5, 0.3];

    let start_time = Instant::now();
    let search_results = vector_storage
        .query(index_name, query_vector, 3, None, true)
        .await?;
    let duration = start_time.elapsed();

    println!("✅ 向量搜索完成! 耗时: {:?}", duration);
    println!("📊 搜索结果数量: {}", search_results.len());

    for (i, result) in search_results.iter().enumerate() {
        println!(
            "📝 结果 {}: ID={}, 相似度={:.4}",
            i + 1,
            result.id,
            result.score
        );
        if let Some(metadata) = &result.metadata {
            println!("   元数据: {:?}", metadata);
        }
    }

    Ok(())
}

async fn test_vector_performance() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试向量性能基准...");

    // 性能测试参数
    let test_sizes = vec![100, 500];
    let vector_dim = 128;

    for size in test_sizes {
        println!("\n📊 测试规模: {} 个向量, 维度: {}", size, vector_dim);

        let vector_storage = MemoryVectorStorage::new(vector_dim, Some(size + 100));
        let index_name = &format!("perf_index_{}", size);

        // 创建索引
        let start_time = Instant::now();
        vector_storage
            .create_index(index_name, vector_dim, Some(SimilarityMetric::Cosine))
            .await?;
        let index_creation_time = start_time.elapsed();
        println!("📈 索引创建时间: {:?}", index_creation_time);

        // 准备批量数据
        let vectors: Vec<Vec<f32>> = (0..size)
            .map(|i| {
                (0..vector_dim)
                    .map(|j| (i as f32 + j as f32) / (size as f32 + vector_dim as f32))
                    .collect()
            })
            .collect();

        let ids: Vec<String> = (0..size).map(|i| format!("vec_{}", i)).collect();

        let metadata: Vec<HashMap<String, serde_json::Value>> = (0..size)
            .map(|i| {
                [
                    ("id".to_string(), json!(i)),
                    (
                        "category".to_string(),
                        json!(format!("category_{}", i % 10)),
                    ),
                ]
                .into()
            })
            .collect();

        // 批量存储性能测试
        let start_time = Instant::now();
        let _inserted_ids = vector_storage
            .upsert(index_name, vectors, Some(ids), Some(metadata))
            .await?;
        let store_total_time = start_time.elapsed();

        let avg_store_time = store_total_time / size as u32;
        println!("📈 平均存储时间: {:?}", avg_store_time);
        println!("📈 总存储时间: {:?}", store_total_time);

        // 搜索性能测试
        let search_iterations = 10;
        let mut search_total_time = std::time::Duration::new(0, 0);

        for _ in 0..search_iterations {
            let query_vector: Vec<f32> = (0..vector_dim)
                .map(|j| j as f32 / vector_dim as f32)
                .collect();

            let start_time = Instant::now();
            let _results = vector_storage
                .query(index_name, query_vector, 10, None, false)
                .await?;
            search_total_time += start_time.elapsed();
        }

        let avg_search_time = search_total_time / search_iterations as u32;
        println!("📈 平均搜索时间: {:?}", avg_search_time);
        println!("📈 总搜索时间: {:?}", search_total_time);

        // 计算吞吐量
        let store_throughput = size as f64 / store_total_time.as_secs_f64();
        let search_throughput = search_iterations as f64 / search_total_time.as_secs_f64();

        println!("🚀 存储吞吐量: {:.2} 向量/秒", store_throughput);
        println!("🚀 搜索吞吐量: {:.2} 查询/秒", search_throughput);

        // 内存使用估算
        let estimated_memory_mb = (size * vector_dim * 4) as f64 / (1024.0 * 1024.0);
        println!("💾 估算内存使用: {:.2} MB", estimated_memory_mb);
    }

    println!("\n📊 向量性能基准测试完成!");

    Ok(())
}
