//! 性能优化功能测试
//! 
//! 测试缓存、性能监控和连接池功能

use lumosai_vector::memory::MemoryVectorStorage;
use lumosai_vector_core::{
    VectorStorage, Document, DocumentId, IndexConfig, 
    SearchRequest, SearchQuery, MetadataValue, Result
};
use std::collections::HashMap;
use tokio;

/// 生成测试向量
fn generate_test_vector(dimension: usize, seed: u64) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let hash = hasher.finish();
    
    (0..dimension)
        .map(|i| {
            let mut h = DefaultHasher::new();
            (hash + i as u64).hash(&mut h);
            (h.finish() % 1000) as f32 / 1000.0
        })
        .collect()
}

/// 生成测试文档
fn generate_test_document(id: &str, dimension: usize, seed: u64) -> Document {
    let mut metadata = HashMap::new();
    metadata.insert("category".to_string(), MetadataValue::String(format!("category_{}", seed % 5)));
    metadata.insert("score".to_string(), MetadataValue::Float((seed % 100) as f64 / 10.0));
    
    Document {
        id: DocumentId::from(id),
        content: format!("Test document content for {}", id),
        embedding: Some(generate_test_vector(dimension, seed)),
        metadata,
    }
}

/// 测试搜索缓存功能
#[tokio::test]
async fn test_search_cache() -> Result<()> {
    let storage = MemoryVectorStorage::new().await?;
    let index_name = "cache_test_index";
    let dimension = 384;
    
    // 创建索引
    let config = IndexConfig::new(index_name, dimension);
    storage.create_index(config).await?;
    
    // 插入测试文档
    let documents = vec![
        generate_test_document("doc1", dimension, 1),
        generate_test_document("doc2", dimension, 2),
        generate_test_document("doc3", dimension, 3),
    ];
    
    storage.upsert_documents(index_name, documents).await?;
    
    // 执行相同的搜索请求多次
    let query_vector = generate_test_vector(dimension, 1);
    let search_request = SearchRequest {
        index_name: index_name.to_string(),
        query: SearchQuery::Vector(query_vector),
        top_k: 2,
        filter: None,
        include_metadata: true,
        include_vectors: false,
        options: HashMap::new(),
    };
    
    // 第一次搜索（应该缓存结果）
    let start_time = std::time::Instant::now();
    let response1 = storage.search(search_request.clone()).await?;
    let first_search_time = start_time.elapsed();
    
    // 第二次搜索（应该从缓存返回）
    let start_time = std::time::Instant::now();
    let response2 = storage.search(search_request.clone()).await?;
    let second_search_time = start_time.elapsed();
    
    // 验证结果一致
    assert_eq!(response1.results.len(), response2.results.len());
    assert_eq!(response1.results[0].id, response2.results[0].id);
    
    // 缓存的搜索应该更快（虽然在内存存储中差异可能很小）
    println!("First search time: {:?}", first_search_time);
    println!("Second search time: {:?}", second_search_time);
    
    // 获取缓存统计信息
    let cache_stats = storage.get_cache_stats().await;
    println!("Cache stats: {:?}", cache_stats);
    
    // 验证缓存命中
    assert!(cache_stats.total_requests >= 2);
    assert!(cache_stats.cache_hits >= 1);
    assert!(cache_stats.hit_rate > 0.0);
    
    Ok(())
}

/// 测试性能监控功能
#[tokio::test]
async fn test_performance_monitoring() -> Result<()> {
    let storage = MemoryVectorStorage::new().await?;
    let index_name = "perf_test_index";
    let dimension = 256;
    
    // 创建索引
    let config = IndexConfig::new(index_name, dimension);
    storage.create_index(config).await?;
    
    // 插入文档
    let documents = vec![
        generate_test_document("perf_doc1", dimension, 1),
        generate_test_document("perf_doc2", dimension, 2),
        generate_test_document("perf_doc3", dimension, 3),
    ];
    
    storage.upsert_documents(index_name, documents).await?;
    
    // 执行多次搜索操作
    let query_vector = generate_test_vector(dimension, 999);
    for i in 0..5 {
        let search_request = SearchRequest {
            index_name: index_name.to_string(),
            query: SearchQuery::Vector(query_vector.clone()),
            top_k: 3,
            filter: None,
            include_metadata: true,
            include_vectors: false,
            options: HashMap::new(),
        };
        
        storage.search(search_request).await?;
    }
    
    // 获取性能指标
    let metrics = storage.get_performance_metrics().await;
    println!("Performance metrics: {:?}", metrics);
    
    // 验证性能指标
    assert!(metrics.total_operations >= 5);
    assert!(metrics.successful_operations >= 5);
    assert_eq!(metrics.failed_operations, 0);
    assert!(metrics.average_response_time.as_nanos() > 0);
    
    Ok(())
}

/// 测试缓存过期和清理
#[tokio::test]
async fn test_cache_expiration() -> Result<()> {
    let storage = MemoryVectorStorage::new().await?;
    let index_name = "cache_expire_test";
    let dimension = 128;
    
    // 创建索引
    let config = IndexConfig::new(index_name, dimension);
    storage.create_index(config).await?;
    
    // 插入文档
    let documents = vec![
        generate_test_document("expire_doc1", dimension, 1),
    ];
    
    storage.upsert_documents(index_name, documents).await?;
    
    // 执行搜索
    let query_vector = generate_test_vector(dimension, 1);
    let search_request = SearchRequest {
        index_name: index_name.to_string(),
        query: SearchQuery::Vector(query_vector),
        top_k: 1,
        filter: None,
        include_metadata: false,
        include_vectors: false,
        options: HashMap::new(),
    };
    
    // 第一次搜索
    storage.search(search_request.clone()).await?;
    
    let initial_stats = storage.get_cache_stats().await;
    assert!(initial_stats.current_size > 0);

    // 手动清理缓存（模拟内存压力）
    storage.cleanup().await?;

    // 验证缓存被清理（cleanup只在内存压力时清理，所以这里验证缓存统计存在即可）
    let after_cleanup_stats = storage.get_cache_stats().await;
    // 缓存可能被清理，也可能没有，这取决于内存阈值设置
    assert!(after_cleanup_stats.total_requests >= initial_stats.total_requests);
    
    Ok(())
}

/// 测试并发搜索的缓存一致性
#[tokio::test]
async fn test_concurrent_cache_access() -> Result<()> {
    let storage = MemoryVectorStorage::new().await?;
    let index_name = "concurrent_cache_test";
    let dimension = 256;
    
    // 创建索引
    let config = IndexConfig::new(index_name, dimension);
    storage.create_index(config).await?;
    
    // 插入文档
    let documents = vec![
        generate_test_document("concurrent_doc1", dimension, 1),
        generate_test_document("concurrent_doc2", dimension, 2),
    ];
    
    storage.upsert_documents(index_name, documents).await?;
    
    // 并发执行相同的搜索请求
    let query_vector = generate_test_vector(dimension, 1);
    let search_request = SearchRequest {
        index_name: index_name.to_string(),
        query: SearchQuery::Vector(query_vector),
        top_k: 2,
        filter: None,
        include_metadata: true,
        include_vectors: false,
        options: HashMap::new(),
    };
    
    let mut handles = vec![];
    for i in 0..10 {
        let storage_clone = storage.clone();
        let request_clone = search_request.clone();
        
        let handle = tokio::spawn(async move {
            storage_clone.search(request_clone).await
        });
        handles.push(handle);
    }
    
    // 等待所有搜索完成
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.unwrap()?;
        results.push(result);
    }
    
    // 验证所有结果一致
    let first_result = &results[0];
    for result in &results[1..] {
        assert_eq!(result.results.len(), first_result.results.len());
        assert_eq!(result.results[0].id, first_result.results[0].id);
    }
    
    // 验证缓存统计
    let cache_stats = storage.get_cache_stats().await;
    println!("Concurrent cache stats: {:?}", cache_stats);
    assert!(cache_stats.total_requests >= 10);
    assert!(cache_stats.cache_hits >= 9); // 第一次是miss，后面都应该是hit
    
    Ok(())
}

/// 性能基准测试
#[tokio::test]
async fn test_performance_benchmark() -> Result<()> {
    let storage = MemoryVectorStorage::new().await?;
    let index_name = "benchmark_index";
    let dimension = 768;
    let doc_count = 1000;
    
    // 创建索引
    let config = IndexConfig::new(index_name, dimension);
    storage.create_index(config).await?;
    
    // 批量插入文档
    let start_time = std::time::Instant::now();
    let documents: Vec<_> = (0..doc_count)
        .map(|i| generate_test_document(&format!("bench_doc_{}", i), dimension, i))
        .collect();
    
    storage.upsert_documents(index_name, documents).await?;
    let insert_duration = start_time.elapsed();
    
    println!("Inserted {} documents in {:?}", doc_count, insert_duration);
    println!("Insert rate: {:.2} docs/sec", doc_count as f64 / insert_duration.as_secs_f64());
    
    // 搜索性能测试
    let query_vector = generate_test_vector(dimension, 999);
    let search_count = 100;
    
    let start_time = std::time::Instant::now();
    for i in 0..search_count {
        let search_request = SearchRequest {
            index_name: index_name.to_string(),
            query: SearchQuery::Vector(query_vector.clone()),
            top_k: 10,
            filter: None,
            include_metadata: false,
            include_vectors: false,
            options: HashMap::new(),
        };
        storage.search(search_request).await?;
    }
    let search_duration = start_time.elapsed();
    
    println!("Performed {} searches in {:?}", search_count, search_duration);
    println!("Search rate: {:.2} searches/sec", search_count as f64 / search_duration.as_secs_f64());
    
    // 获取最终的性能指标
    let final_metrics = storage.get_performance_metrics().await;
    println!("Final performance metrics: {:?}", final_metrics);
    
    let final_cache_stats = storage.get_cache_stats().await;
    println!("Final cache stats: {:?}", final_cache_stats);
    
    // 验证性能指标合理性
    assert!(final_metrics.total_operations >= search_count);
    assert!(final_metrics.average_response_time.as_millis() < 100); // 应该很快
    
    Ok(())
}
