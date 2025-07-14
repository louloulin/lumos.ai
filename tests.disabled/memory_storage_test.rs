//! 内存向量存储测试
//! 
//! 专门测试内存向量存储的功能

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
    metadata.insert("active".to_string(), MetadataValue::Boolean(seed % 2 == 0));
    metadata.insert("count".to_string(), MetadataValue::Integer((seed % 50) as i64));
    
    Document {
        id: DocumentId::from(id),
        content: format!("Test document content for {}", id),
        embedding: Some(generate_test_vector(dimension, seed)),
        metadata,
    }
}

/// 测试基础存储操作
#[tokio::test]
async fn test_basic_operations() -> Result<()> {
    let storage = MemoryVectorStorage::new().await?;
    let index_name = "test_index";
    let dimension = 384;
    
    // 1. 创建索引
    let config = IndexConfig::new(index_name, dimension);
    storage.create_index(config).await?;
    
    // 2. 列出索引
    let indexes = storage.list_indexes().await?;
    assert!(indexes.contains(&index_name.to_string()));
    
    // 3. 描述索引
    let index_info = storage.describe_index(index_name).await?;
    assert_eq!(index_info.name, index_name);
    assert_eq!(index_info.dimension, dimension);
    
    // 4. 插入文档
    let documents = vec![
        generate_test_document("doc1", dimension, 1),
        generate_test_document("doc2", dimension, 2),
        generate_test_document("doc3", dimension, 3),
    ];
    
    let doc_ids = storage.upsert_documents(index_name, documents.clone()).await?;
    assert_eq!(doc_ids.len(), 3);
    
    // 5. 检索文档
    let retrieved_docs = storage.get_documents(index_name, doc_ids.clone(), true).await?;
    assert_eq!(retrieved_docs.len(), 3);
    
    // 6. 向量搜索
    let query_vector = generate_test_vector(dimension, 1);
    let search_request = SearchRequest {
        index_name: index_name.to_string(),
        query: SearchQuery::Vector(query_vector),
        top_k: 2,
        filter: None,
        include_metadata: true,
        include_vectors: true,
        options: HashMap::new(),
    };
    
    let search_response = storage.search(search_request).await?;
    assert!(!search_response.results.is_empty());
    assert!(search_response.results.len() <= 2);
    
    // 7. 更新文档
    let mut updated_doc = documents[0].clone();
    updated_doc.content = "Updated content".to_string();
    storage.update_document(index_name, updated_doc).await?;
    
    // 8. 删除文档
    storage.delete_documents(index_name, vec![doc_ids[0].clone()]).await?;
    
    let remaining_docs = storage.get_documents(index_name, doc_ids.clone(), false).await?;
    assert_eq!(remaining_docs.len(), 2);
    
    // 9. 删除索引
    storage.delete_index(index_name).await?;
    
    let indexes_after = storage.list_indexes().await?;
    assert!(!indexes_after.contains(&index_name.to_string()));
    
    Ok(())
}

/// 测试健康检查
#[tokio::test]
async fn test_health_check() -> Result<()> {
    let storage = MemoryVectorStorage::new().await?;
    storage.health_check().await?;
    Ok(())
}

/// 测试后端信息
#[tokio::test]
async fn test_backend_info() -> Result<()> {
    let storage = MemoryVectorStorage::new().await?;
    let backend_info = storage.backend_info();
    assert_eq!(backend_info.name, "memory");
    Ok(())
}

/// 测试错误处理
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let storage = MemoryVectorStorage::new().await?;
    
    // 测试不存在的索引
    let result = storage.describe_index("nonexistent_index").await;
    assert!(result.is_err());
    
    Ok(())
}

/// 测试并发操作
#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let storage = MemoryVectorStorage::new().await?;
    let index_name = "concurrent_test_index";
    let dimension = 128;
    
    // 创建索引
    let config = IndexConfig::new(index_name, dimension);
    storage.create_index(config).await?;
    
    // 并发插入文档
    let mut handles = vec![];
    for i in 0..10 {
        let doc = generate_test_document(&format!("concurrent_doc_{}", i), dimension, i);
        let handle = tokio::spawn({
            let storage = storage.clone();
            let index_name = index_name.to_string();
            async move {
                storage.upsert_documents(&index_name, vec![doc]).await
            }
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap()?;
    }
    
    // 验证所有文档都已插入
    let all_docs = storage.get_documents(
        index_name, 
        (0..10).map(|i| DocumentId::from(format!("concurrent_doc_{}", i))).collect(),
        false
    ).await?;
    
    assert_eq!(all_docs.len(), 10);
    
    // 清理
    storage.delete_index(index_name).await?;
    
    Ok(())
}

/// 性能基准测试
#[tokio::test]
async fn test_performance_benchmark() -> Result<()> {
    let storage = MemoryVectorStorage::new().await?;
    let index_name = "perf_test_index";
    let dimension = 768;
    let doc_count = 100;
    
    // 创建索引
    let config = IndexConfig::new(index_name, dimension);
    storage.create_index(config).await?;
    
    // 批量插入测试
    let start = std::time::Instant::now();
    let documents: Vec<_> = (0..doc_count)
        .map(|i| generate_test_document(&format!("perf_doc_{}", i), dimension, i))
        .collect();
    
    storage.upsert_documents(index_name, documents).await?;
    let insert_duration = start.elapsed();
    
    println!("Inserted {} documents in {:?}", doc_count, insert_duration);
    
    // 搜索性能测试
    let query_vector = generate_test_vector(dimension, 999);
    let start = std::time::Instant::now();
    
    for _ in 0..10 {
        let search_request = SearchRequest {
            index_name: index_name.to_string(),
            query: SearchQuery::Vector(query_vector.clone()),
            top_k: 10,
            filter: None,
            include_metadata: true,
            include_vectors: false,
            options: HashMap::new(),
        };
        storage.search(search_request).await?;
    }
    
    let search_duration = start.elapsed();
    println!("Performed 10 searches in {:?}", search_duration);
    
    // 清理
    storage.delete_index(index_name).await?;
    
    Ok(())
}
