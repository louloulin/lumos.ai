//! 向量数据库集成测试
//!
//! 这些测试验证所有向量数据库的完整功能

use lumosai_vector_core::{
    VectorStorage as VectorStorageTrait, Document, DocumentId, IndexConfig,
    SearchRequest, SearchQuery, SearchResponse, MetadataValue, FilterCondition,
    IndexInfo, BackendInfo, Result
};
use std::collections::HashMap;
use tokio;

/// 测试数据生成器
struct TestDataGenerator;

impl TestDataGenerator {
    /// 生成测试向量
    fn generate_vector(dimension: usize, seed: u64) -> Vec<f32> {
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
    fn generate_document(id: &str, dimension: usize, seed: u64) -> Document {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), MetadataValue::String(format!("category_{}", seed % 5)));
        metadata.insert("score".to_string(), MetadataValue::Float((seed % 100) as f64 / 10.0));
        metadata.insert("active".to_string(), MetadataValue::Boolean(seed % 2 == 0));
        metadata.insert("count".to_string(), MetadataValue::Integer((seed % 50) as i64));
        
        Document {
            id: DocumentId::from(id),
            content: format!("Test document content for {}", id),
            embedding: Some(Self::generate_vector(dimension, seed)),
            metadata,
        }
    }
    
    /// 生成测试索引配置
    fn generate_index_config(name: &str, dimension: usize) -> IndexConfig {
        IndexConfig::new(name, dimension)
            .with_option("description", format!("Test index for {}", name))
    }
}

/// 基础向量存储测试套件
async fn run_basic_storage_tests<T: VectorStorageTrait + ?Sized>(storage: &T) -> Result<()> {
    let index_name = "test_basic_index";
    let dimension = 384;
    
    // 1. 测试索引创建
    let config = TestDataGenerator::generate_index_config(index_name, dimension);
    storage.create_index(config).await?;
    
    // 2. 测试索引列表
    let indexes = storage.list_indexes().await?;
    assert!(indexes.contains(&index_name.to_string()), "Index should be listed");
    
    // 3. 测试索引描述
    let index_info = storage.describe_index(index_name).await?;
    assert_eq!(index_info.name, index_name);
    assert_eq!(index_info.dimension, dimension);
    
    // 4. 测试文档插入
    let documents = vec![
        TestDataGenerator::generate_document("doc1", dimension, 1),
        TestDataGenerator::generate_document("doc2", dimension, 2),
        TestDataGenerator::generate_document("doc3", dimension, 3),
    ];
    
    let doc_ids = storage.upsert_documents(index_name, documents.clone()).await?;
    assert_eq!(doc_ids.len(), 3, "Should return 3 document IDs");
    
    // 5. 测试文档检索
    let retrieved_docs = storage.get_documents(
        index_name, 
        doc_ids.clone(), 
        true
    ).await?;
    assert_eq!(retrieved_docs.len(), 3, "Should retrieve 3 documents");
    
    // 6. 测试向量搜索
    let query_vector = TestDataGenerator::generate_vector(dimension, 1);
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
    assert!(!search_response.results.is_empty(), "Should return search results");
    assert!(search_response.results.len() <= 2, "Should respect limit");
    
    // 7. 测试文档更新
    let mut updated_doc = documents[0].clone();
    updated_doc.content = "Updated content".to_string();
    storage.update_document(index_name, updated_doc).await?;
    
    // 8. 测试文档删除
    storage.delete_documents(index_name, vec![doc_ids[0].clone()]).await?;
    
    let remaining_docs = storage.get_documents(
        index_name, 
        doc_ids.clone(), 
        false
    ).await?;
    assert_eq!(remaining_docs.len(), 2, "Should have 2 documents after deletion");
    
    // 9. 测试索引删除
    storage.delete_index(index_name).await?;
    
    let indexes_after = storage.list_indexes().await?;
    assert!(!indexes_after.contains(&index_name.to_string()), "Index should be deleted");
    
    Ok(())
}

/// 内存向量存储集成测试
#[tokio::test]
async fn test_memory_vector_storage_integration() {
    let storage = lumosai_vector::memory::MemoryVectorStorage::new().await.expect("Failed to create storage");

    println!("Testing Memory Vector Storage...");

    // 运行基础测试
    run_basic_storage_tests(&storage).await.expect("Basic tests failed");
    println!("✅ Memory storage basic tests passed");

    // 测试健康检查
    storage.health_check().await.expect("Health check failed");
    println!("✅ Memory storage health check passed");

    // 测试后端信息
    let backend_info = storage.backend_info();
    assert_eq!(backend_info.name, "Memory");
    println!("✅ Memory storage backend info: {:?}", backend_info);
}

/// 错误处理测试
#[tokio::test]
async fn test_error_handling() {
    let storage = lumosai_vector::memory::MemoryVectorStorage::new().await.expect("Failed to create storage");

    // 测试不存在的索引
    let result = storage.describe_index("nonexistent_index").await;
    assert!(result.is_err(), "Should fail for nonexistent index");

    println!("✅ Error handling tests passed");
}
