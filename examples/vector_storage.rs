//! 向量存储演示
//!
//! 展示如何使用不同的向量存储后端，包括：
//! - 内存向量存储
//! - 持久化向量存储
//! - 向量搜索和相似度计算
//! - 性能对比测试

use lumosai_core::vector::{MemoryVectorStorage, SimilarityMetric, VectorStorage};
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;
use tokio;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🔍 向量存储演示");
    println!("================");

    // 演示1: 内存向量存储
    demo_memory_storage().await?;

    // 演示2: 向量搜索功能
    demo_vector_search().await?;

    // 演示3: 批量操作
    demo_batch_operations().await?;

    // 演示4: 性能测试
    demo_performance_testing().await?;

    Ok(())
}

/// 演示内存向量存储
async fn demo_memory_storage() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示1: 内存向量存储 ===");

    // 创建内存向量存储
    let storage = MemoryVectorStorage::new(384, Some(1000));

    // 创建索引
    let index_name = "demo_index";
    let dimension = 384;
    storage
        .create_index(index_name, dimension, Some(SimilarityMetric::Cosine))
        .await?;

    println!("创建向量索引:");
    println!("  索引名称: {}", index_name);
    println!("  向量维度: {}", dimension);
    println!("  相似度度量: Cosine");

    // 准备测试向量和元数据
    let test_vectors = vec![
        generate_mock_vector(dimension, 1),
        generate_mock_vector(dimension, 2),
        generate_mock_vector(dimension, 3),
        generate_mock_vector(dimension, 4),
        generate_mock_vector(dimension, 5),
    ];

    let test_ids = vec![
        "doc_1".to_string(),
        "doc_2".to_string(),
        "doc_3".to_string(),
        "doc_4".to_string(),
        "doc_5".to_string(),
    ];

    let test_metadata = vec![
        create_metadata("文档1", "技术", "Rust编程基础"),
        create_metadata("文档2", "技术", "Python数据科学"),
        create_metadata("文档3", "技术", "JavaScript前端开发"),
        create_metadata("文档4", "商业", "市场分析报告"),
        create_metadata("文档5", "教育", "机器学习入门"),
    ];

    // 插入向量
    println!("\n插入测试向量...");
    let inserted_ids = storage
        .upsert(
            index_name,
            test_vectors.clone(),
            Some(test_ids.clone()),
            Some(test_metadata.clone()),
        )
        .await?;

    println!("成功插入 {} 个向量", inserted_ids.len());
    for (i, id) in inserted_ids.iter().enumerate() {
        println!("  {}. ID: {}", i + 1, id);
    }

    // 获取索引统计信息
    let stats = storage.describe_index(index_name).await?;
    println!("\n索引统计信息:");
    println!("  向量数量: {}", stats.count);
    println!("  向量维度: {}", stats.dimension);
    println!("  相似度度量: {:?}", stats.metric);

    // 列出所有索引
    let indexes = storage.list_indexes().await?;
    println!("\n所有索引:");
    for (i, index) in indexes.iter().enumerate() {
        println!("  {}. {}", i + 1, index);
    }

    Ok(())
}

/// 演示向量搜索功能
async fn demo_vector_search() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示2: 向量搜索功能 ===");

    let storage = MemoryVectorStorage::new(384, Some(1000));
    let index_name = "search_demo";
    let dimension = 128; // 使用较小的维度便于演示

    // 创建索引
    storage
        .create_index(index_name, dimension, Some(SimilarityMetric::Cosine))
        .await?;

    // 准备语义相关的向量（模拟）
    let documents = vec![
        ("rust_basics", "Rust编程语言基础教程", "技术"),
        ("rust_advanced", "Rust高级特性和最佳实践", "技术"),
        ("python_ml", "Python机器学习实战", "技术"),
        ("js_frontend", "JavaScript前端开发指南", "技术"),
        ("business_plan", "商业计划书模板", "商业"),
        ("market_analysis", "市场分析方法论", "商业"),
        ("ai_intro", "人工智能入门教程", "教育"),
        ("data_science", "数据科学基础知识", "教育"),
    ];

    let mut vectors = Vec::new();
    let mut ids = Vec::new();
    let mut metadata = Vec::new();

    for (id, title, category) in &documents {
        vectors.push(generate_semantic_vector(title, dimension));
        ids.push(id.to_string());
        metadata.push(create_metadata(title, category, title));
    }

    // 插入向量
    storage
        .upsert(index_name, vectors, Some(ids), Some(metadata))
        .await?;
    println!("已插入 {} 个语义向量", documents.len());

    // 测试不同类型的搜索
    let search_queries = vec![
        ("Rust编程", "寻找Rust相关内容"),
        ("机器学习", "寻找AI/ML相关内容"),
        ("商业分析", "寻找商业相关内容"),
        ("前端开发", "寻找前端技术内容"),
    ];

    println!("\n=== 语义搜索测试 ===");
    for (query, description) in search_queries {
        println!("\n搜索查询: {} ({})", query, description);

        // 生成查询向量
        let query_vector = generate_semantic_vector(query, dimension);

        // 执行搜索
        let results = storage
            .query(
                index_name,
                query_vector,
                3,    // top_k
                None, // filter
                true, // include_vectors
            )
            .await?;

        println!("搜索结果 (top 3):");
        for (i, result) in results.iter().enumerate() {
            let title = result
                .metadata
                .as_ref()
                .and_then(|m| m.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("未知");
            let category = result
                .metadata
                .as_ref()
                .and_then(|m| m.get("category"))
                .and_then(|v| v.as_str())
                .unwrap_or("未知");

            println!(
                "  {}. {} [{}] (相似度: {:.3})",
                i + 1,
                title,
                category,
                result.score
            );
        }
    }

    Ok(())
}

/// 演示批量操作
async fn demo_batch_operations() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示3: 批量操作 ===");

    let storage = MemoryVectorStorage::new(384, Some(1000));
    let index_name = "batch_demo";
    let dimension = 256;

    storage
        .create_index(index_name, dimension, Some(SimilarityMetric::Euclidean))
        .await?;

    // 生成大批量测试数据
    let batch_size = 1000;
    println!("生成 {} 个测试向量...", batch_size);

    let start_time = Instant::now();

    let mut all_vectors = Vec::new();
    let mut all_ids = Vec::new();
    let mut all_metadata = Vec::new();

    for i in 0..batch_size {
        all_vectors.push(generate_mock_vector(dimension, i as u64));
        all_ids.push(format!("batch_doc_{}", i));
        all_metadata.push(create_metadata(
            &format!("批量文档 {}", i),
            if i % 3 == 0 {
                "类型A"
            } else if i % 3 == 1 {
                "类型B"
            } else {
                "类型C"
            },
            &format!("这是第 {} 个批量测试文档", i),
        ));
    }

    let generation_time = start_time.elapsed();
    println!("向量生成耗时: {:?}", generation_time);

    // 批量插入
    println!("\n执行批量插入...");
    let insert_start = Instant::now();

    let inserted_ids = storage
        .upsert(index_name, all_vectors, Some(all_ids), Some(all_metadata))
        .await?;

    let insert_time = insert_start.elapsed();
    println!("批量插入完成:");
    println!("  插入数量: {}", inserted_ids.len());
    println!("  插入耗时: {:?}", insert_time);
    println!(
        "  平均速度: {:.2} 向量/秒",
        batch_size as f64 / insert_time.as_secs_f64()
    );

    // 批量搜索测试
    println!("\n执行批量搜索测试...");
    let search_start = Instant::now();
    let search_count = 100;

    for i in 0..search_count {
        let query_vector = generate_mock_vector(dimension, (i * 7) as u64); // 使用不同的种子
        let _results = storage
            .query(index_name, query_vector, 5, None, false)
            .await?;
    }

    let search_time = search_start.elapsed();
    println!("批量搜索完成:");
    println!("  搜索次数: {}", search_count);
    println!("  总耗时: {:?}", search_time);
    println!("  平均搜索时间: {:?}", search_time / search_count);

    // 获取最终统计
    let final_stats = storage.describe_index(index_name).await?;
    println!("\n最终索引统计:");
    println!("  向量数量: {}", final_stats.count);
    println!("  向量维度: {}", final_stats.dimension);
    println!("  相似度度量: {:?}", final_stats.metric);

    Ok(())
}

/// 演示性能测试
async fn demo_performance_testing() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示4: 性能测试 ===");

    // 测试不同维度的性能
    let dimensions = vec![128, 256, 512, 1024];
    let test_size = 500;

    println!("性能测试配置:");
    println!("  测试向量数量: {}", test_size);
    println!("  测试维度: {:?}", dimensions);

    for dimension in dimensions {
        println!("\n--- 测试维度: {} ---", dimension);

        let storage = MemoryVectorStorage::new(384, Some(1000));
        let index_name = &format!("perf_test_{}", dimension);

        // 创建索引
        storage
            .create_index(index_name, dimension, Some(SimilarityMetric::Cosine))
            .await?;

        // 生成测试数据
        let vectors: Vec<Vec<f32>> = (0..test_size)
            .map(|i| generate_mock_vector(dimension, i as u64))
            .collect();

        let ids: Vec<String> = (0..test_size).map(|i| format!("perf_doc_{}", i)).collect();

        // 测试插入性能
        let insert_start = Instant::now();
        storage
            .upsert(index_name, vectors.clone(), Some(ids), None)
            .await?;
        let insert_time = insert_start.elapsed();

        // 测试搜索性能
        let search_start = Instant::now();
        let search_iterations = 50;

        for i in 0..search_iterations {
            let query_vector = generate_mock_vector(dimension, (i * 13) as u64);
            storage
                .query(index_name, query_vector, 10, None, false)
                .await?;
        }

        let search_time = search_start.elapsed();

        // 输出性能结果
        println!("  插入性能:");
        println!("    总时间: {:?}", insert_time);
        println!(
            "    速度: {:.2} 向量/秒",
            test_size as f64 / insert_time.as_secs_f64()
        );

        println!("  搜索性能:");
        println!("    总时间: {:?}", search_time);
        println!("    平均时间: {:?}", search_time / search_iterations);
        println!(
            "    QPS: {:.2}",
            search_iterations as f64 / search_time.as_secs_f64()
        );

        // 内存使用估算
        let vector_size = dimension * 4; // 4 bytes per f32
        let total_memory = test_size * vector_size;
        println!("  内存使用:");
        println!("    每向量: {} bytes", vector_size);
        println!("    总计: {} KB", total_memory / 1024);
    }

    // 相似度度量对比
    println!("\n=== 相似度度量对比 ===");
    demo_similarity_metrics().await?;

    Ok(())
}

/// 演示不同相似度度量
async fn demo_similarity_metrics() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let dimension = 128;
    let test_vectors = vec![
        vec![1.0; dimension],                // 全1向量
        vec![0.0; dimension],                // 全0向量
        generate_mock_vector(dimension, 42), // 随机向量1
        generate_mock_vector(dimension, 84), // 随机向量2
    ];

    let metrics = vec![
        SimilarityMetric::Cosine,
        SimilarityMetric::Euclidean,
        SimilarityMetric::DotProduct,
    ];

    for metric in metrics {
        println!("\n测试相似度度量: {:?}", metric);

        let storage = MemoryVectorStorage::new(384, Some(1000));
        let index_name = &format!("metric_test_{:?}", metric);

        storage
            .create_index(index_name, dimension, Some(metric))
            .await?;

        // 插入测试向量
        let ids: Vec<String> = (0..test_vectors.len())
            .map(|i| format!("test_vec_{}", i))
            .collect();

        storage
            .upsert(index_name, test_vectors.clone(), Some(ids), None)
            .await?;

        // 使用第一个向量作为查询
        let query_vector = test_vectors[0].clone();
        let results = storage
            .query(index_name, query_vector, 4, None, false)
            .await?;

        println!("  搜索结果:");
        for (i, result) in results.iter().enumerate() {
            println!(
                "    {}. ID: {} (分数: {:.4})",
                i + 1,
                result.id,
                result.score
            );
        }
    }

    Ok(())
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 生成模拟向量
fn generate_mock_vector(dimension: usize, seed: u64) -> Vec<f32> {
    let mut vector = Vec::with_capacity(dimension);
    for i in 0..dimension {
        let value = ((seed.wrapping_add(i as u64) * 1103515245 + 12345) % (1 << 31)) as f32
            / (1 << 30) as f32
            - 1.0;
        vector.push(value);
    }

    // 归一化向量
    let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in &mut vector {
            *x /= norm;
        }
    }

    vector
}

/// 生成语义向量（基于文本内容）
fn generate_semantic_vector(text: &str, dimension: usize) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();

    let mut vector = Vec::with_capacity(dimension);
    for i in 0..dimension {
        let seed = hash.wrapping_add(i as u64);
        let value = ((seed.wrapping_mul(1103515245).wrapping_add(12345)) % (1 << 31)) as f32
            / (1 << 30) as f32
            - 1.0;
        vector.push(value);
    }

    // 根据文本内容调整向量（简单的语义模拟）
    if text.contains("Rust") || text.contains("rust") {
        for i in 0..10 {
            if i < vector.len() {
                vector[i] += 0.5;
            }
        }
    }
    if text.contains("Python") || text.contains("python") {
        for i in 10..20 {
            if i < vector.len() {
                vector[i] += 0.5;
            }
        }
    }
    if text.contains("机器学习") || text.contains("AI") || text.contains("人工智能") {
        for i in 20..30 {
            if i < vector.len() {
                vector[i] += 0.5;
            }
        }
    }
    if text.contains("商业") || text.contains("市场") {
        for i in 30..40 {
            if i < vector.len() {
                vector[i] += 0.5;
            }
        }
    }

    // 归一化
    let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in &mut vector {
            *x /= norm;
        }
    }

    vector
}

/// 创建元数据
fn create_metadata(
    title: &str,
    category: &str,
    description: &str,
) -> HashMap<String, serde_json::Value> {
    let mut metadata = HashMap::new();
    metadata.insert("title".to_string(), json!(title));
    metadata.insert("category".to_string(), json!(category));
    metadata.insert("description".to_string(), json!(description));
    metadata.insert(
        "timestamp".to_string(),
        json!(chrono::Utc::now().to_rfc3339()),
    );
    metadata
}

/// 格式化性能结果
#[allow(dead_code)]
fn format_performance_result(
    operation: &str,
    count: usize,
    duration: std::time::Duration,
) -> String {
    let rate = count as f64 / duration.as_secs_f64();
    format!(
        "{}: {} 操作, 耗时 {:?}, 速率 {:.2} ops/sec",
        operation, count, duration, rate
    )
}
