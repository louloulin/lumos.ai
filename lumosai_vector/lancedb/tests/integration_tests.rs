//! Integration tests for LanceDB storage

use std::collections::HashMap;
use tempfile::TempDir;
use tokio_test;

use lumosai_vector_lancedb::{LanceDbStorage, LanceDbConfig, LanceDbConfigBuilder};
use lumosai_vector_core::{
    traits::VectorStorage,
    types::{Document, IndexConfig, SearchRequest, SimilarityMetric, FilterCondition, MetadataValue},
};

/// Create a test storage instance with temporary directory
async fn create_test_storage() -> (LanceDbStorage, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let uri = format!("file://{}", temp_dir.path().display());
    
    let config = LanceDbConfig::new(&uri);
    let storage = LanceDbStorage::new(config).await.expect("Failed to create storage");
    
    (storage, temp_dir)
}

/// Generate a test embedding vector
fn generate_test_embedding(dimension: usize, seed: u64) -> Vec<f32> {
    let mut embedding = Vec::with_capacity(dimension);
    let mut value = seed as f32;
    
    for _ in 0..dimension {
        embedding.push((value % 2.0) - 1.0);
        value = (value * 1.1) % 100.0;
    }
    
    // Normalize
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for x in &mut embedding {
            *x /= magnitude;
        }
    }
    
    embedding
}

#[tokio::test]
async fn test_storage_creation() {
    let (storage, _temp_dir) = create_test_storage().await;
    
    // Test health check
    storage.health_check().await.expect("Health check failed");
    
    // Test backend info
    let backend_info = storage.backend_info();
    assert_eq!(backend_info.name, "lancedb");
    assert!(backend_info.features.contains(&"vector_search".to_string()));
}

#[tokio::test]
async fn test_index_operations() {
    let (storage, _temp_dir) = create_test_storage().await;
    
    // Test index creation
    let index_config = IndexConfig::new("test_index", 128)
        .with_metric(SimilarityMetric::Cosine)
        .with_description("Test index");
    
    storage.create_index(index_config).await.expect("Failed to create index");
    
    // Test list indexes
    let indexes = storage.list_indexes().await.expect("Failed to list indexes");
    assert!(indexes.contains(&"test_index".to_string()));
    
    // Test describe index
    let index_info = storage.describe_index("test_index").await.expect("Failed to describe index");
    assert_eq!(index_info.name, "test_index");
    assert_eq!(index_info.dimension, 128);
    assert_eq!(index_info.document_count, 0);
    
    // Test duplicate index creation (should fail)
    let duplicate_config = IndexConfig::new("test_index", 128);
    let result = storage.create_index(duplicate_config).await;
    assert!(result.is_err());
    
    // Test delete index
    storage.delete_index("test_index").await.expect("Failed to delete index");
    
    let indexes = storage.list_indexes().await.expect("Failed to list indexes");
    assert!(!indexes.contains(&"test_index".to_string()));
}

#[tokio::test]
async fn test_document_operations() {
    let (storage, _temp_dir) = create_test_storage().await;
    
    // Create index
    let index_config = IndexConfig::new("docs", 64);
    storage.create_index(index_config).await.expect("Failed to create index");
    
    // Test document insertion
    let documents = vec![
        Document::new("doc1", "First document")
            .with_embedding(generate_test_embedding(64, 1))
            .with_metadata("category", "test")
            .with_metadata("priority", 1i64),
        
        Document::new("doc2", "Second document")
            .with_embedding(generate_test_embedding(64, 2))
            .with_metadata("category", "test")
            .with_metadata("priority", 2i64),
        
        Document::new("doc3", "Third document")
            .with_embedding(generate_test_embedding(64, 3))
            .with_metadata("category", "other")
            .with_metadata("priority", 1i64),
    ];
    
    let doc_ids = storage.upsert_documents("docs", documents).await.expect("Failed to insert documents");
    assert_eq!(doc_ids.len(), 3);
    
    // Test get documents
    let retrieved = storage.get_documents("docs", vec!["doc1".to_string(), "doc2".to_string()], true).await
        .expect("Failed to get documents");
    assert_eq!(retrieved.len(), 2);
    assert!(retrieved[0].embedding.is_some());
    
    // Test get documents without vectors
    let retrieved_no_vectors = storage.get_documents("docs", vec!["doc1".to_string()], false).await
        .expect("Failed to get documents");
    assert_eq!(retrieved_no_vectors.len(), 1);
    assert!(retrieved_no_vectors[0].embedding.is_none());
    
    // Test document update
    let updated_doc = Document::new("doc1", "Updated first document")
        .with_embedding(generate_test_embedding(64, 1))
        .with_metadata("category", "updated")
        .with_metadata("priority", 5i64);
    
    storage.update_document("docs", updated_doc).await.expect("Failed to update document");
    
    let updated = storage.get_documents("docs", vec!["doc1".to_string()], false).await
        .expect("Failed to get updated document");
    assert_eq!(updated[0].content.as_deref(), Some("Updated first document"));
    
    // Test document deletion
    storage.delete_documents("docs", vec!["doc3".to_string()]).await.expect("Failed to delete document");
    
    let remaining = storage.get_documents("docs", vec!["doc3".to_string()], false).await
        .expect("Failed to check deleted document");
    assert!(remaining.is_empty());
}

#[tokio::test]
async fn test_vector_search() {
    let (storage, _temp_dir) = create_test_storage().await;
    
    // Create index and insert documents
    let index_config = IndexConfig::new("search_test", 32);
    storage.create_index(index_config).await.expect("Failed to create index");
    
    let documents = vec![
        Document::new("similar1", "Similar document one")
            .with_embedding(generate_test_embedding(32, 100))
            .with_metadata("group", "A"),
        
        Document::new("similar2", "Similar document two")
            .with_embedding(generate_test_embedding(32, 101))
            .with_metadata("group", "A"),
        
        Document::new("different", "Different document")
            .with_embedding(generate_test_embedding(32, 200))
            .with_metadata("group", "B"),
    ];
    
    storage.upsert_documents("search_test", documents).await.expect("Failed to insert documents");
    
    // Test basic search
    let query_vector = generate_test_embedding(32, 100);
    let search_request = SearchRequest {
        index_name: "search_test".to_string(),
        vector: query_vector,
        top_k: 3,
        similarity_metric: Some(SimilarityMetric::Cosine),
        filter: None,
        include_metadata: true,
    };
    
    let response = storage.search(search_request).await.expect("Failed to search");
    assert_eq!(response.results.len(), 3);
    
    // First result should be most similar (similar1)
    assert_eq!(response.results[0].id, "similar1");
    assert!(response.results[0].score > response.results[1].score);
    
    // Test search with filter
    let filter = FilterCondition::Eq("group".to_string(), MetadataValue::String("A".to_string()));
    let filtered_search = SearchRequest {
        index_name: "search_test".to_string(),
        vector: generate_test_embedding(32, 100),
        top_k: 3,
        similarity_metric: Some(SimilarityMetric::Cosine),
        filter: Some(filter),
        include_metadata: true,
    };
    
    let filtered_response = storage.search(filtered_search).await.expect("Failed to search with filter");
    assert_eq!(filtered_response.results.len(), 2);
    
    for result in &filtered_response.results {
        let group = result.metadata.as_ref()
            .and_then(|m| m.get("group"))
            .unwrap();
        assert_eq!(group, &MetadataValue::String("A".to_string()));
    }
}

#[tokio::test]
async fn test_complex_filters() {
    let (storage, _temp_dir) = create_test_storage().await;
    
    // Create index and insert test documents
    let index_config = IndexConfig::new("filter_test", 16);
    storage.create_index(index_config).await.expect("Failed to create index");
    
    let documents = vec![
        Document::new("doc1", "Document 1")
            .with_embedding(generate_test_embedding(16, 1))
            .with_metadata("category", "tech")
            .with_metadata("score", 85i64)
            .with_metadata("active", true),
        
        Document::new("doc2", "Document 2")
            .with_embedding(generate_test_embedding(16, 2))
            .with_metadata("category", "science")
            .with_metadata("score", 92i64)
            .with_metadata("active", true),
        
        Document::new("doc3", "Document 3")
            .with_embedding(generate_test_embedding(16, 3))
            .with_metadata("category", "tech")
            .with_metadata("score", 78i64)
            .with_metadata("active", false),
        
        Document::new("doc4", "Document 4")
            .with_embedding(generate_test_embedding(16, 4))
            .with_metadata("category", "business")
            .with_metadata("score", 88i64)
            .with_metadata("active", true),
    ];
    
    storage.upsert_documents("filter_test", documents).await.expect("Failed to insert documents");
    
    // Test AND filter
    let and_filter = FilterCondition::And(vec![
        FilterCondition::Eq("category".to_string(), MetadataValue::String("tech".to_string())),
        FilterCondition::Eq("active".to_string(), MetadataValue::Boolean(true)),
    ]);
    
    let and_search = SearchRequest {
        index_name: "filter_test".to_string(),
        vector: generate_test_embedding(16, 1),
        top_k: 10,
        similarity_metric: Some(SimilarityMetric::Cosine),
        filter: Some(and_filter),
        include_metadata: true,
    };
    
    let and_response = storage.search(and_search).await.expect("Failed to search with AND filter");
    assert_eq!(and_response.results.len(), 1);
    assert_eq!(and_response.results[0].id, "doc1");
    
    // Test OR filter
    let or_filter = FilterCondition::Or(vec![
        FilterCondition::Eq("category".to_string(), MetadataValue::String("science".to_string())),
        FilterCondition::Gt("score".to_string(), MetadataValue::Integer(90)),
    ]);
    
    let or_search = SearchRequest {
        index_name: "filter_test".to_string(),
        vector: generate_test_embedding(16, 1),
        top_k: 10,
        similarity_metric: Some(SimilarityMetric::Cosine),
        filter: Some(or_filter),
        include_metadata: true,
    };
    
    let or_response = storage.search(or_search).await.expect("Failed to search with OR filter");
    assert_eq!(or_response.results.len(), 1);
    assert_eq!(or_response.results[0].id, "doc2");
    
    // Test range filter
    let range_filter = FilterCondition::And(vec![
        FilterCondition::Gte("score".to_string(), MetadataValue::Integer(80)),
        FilterCondition::Lt("score".to_string(), MetadataValue::Integer(90)),
    ]);
    
    let range_search = SearchRequest {
        index_name: "filter_test".to_string(),
        vector: generate_test_embedding(16, 1),
        top_k: 10,
        similarity_metric: Some(SimilarityMetric::Cosine),
        filter: Some(range_filter),
        include_metadata: true,
    };
    
    let range_response = storage.search(range_search).await.expect("Failed to search with range filter");
    assert_eq!(range_response.results.len(), 2); // doc1 (85) and doc4 (88)
}

#[tokio::test]
async fn test_configuration() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let uri = format!("file://{}", temp_dir.path().display());
    
    // Test configuration builder
    let config = LanceDbConfigBuilder::new(&uri)
        .batch_size(500)
        .enable_compression(true)
        .compression_level(8)
        .cache_size(1024 * 1024)
        .build()
        .expect("Failed to build config");
    
    assert_eq!(config.performance.batch_size, 500);
    assert!(config.performance.enable_compression);
    assert_eq!(config.performance.compression_level, Some(8));
    assert_eq!(config.performance.cache_size, Some(1024 * 1024));
    
    // Test storage creation with custom config
    let storage = LanceDbStorage::new(config).await.expect("Failed to create storage with custom config");
    storage.health_check().await.expect("Health check failed");
}

#[tokio::test]
async fn test_error_handling() {
    let (storage, _temp_dir) = create_test_storage().await;
    
    // Test operations on non-existent index
    let result = storage.describe_index("nonexistent").await;
    assert!(result.is_err());
    
    let result = storage.delete_index("nonexistent").await;
    assert!(result.is_err());
    
    let result = storage.upsert_documents("nonexistent", vec![]).await;
    assert!(result.is_err());
    
    // Test invalid document (missing embedding)
    let index_config = IndexConfig::new("error_test", 32);
    storage.create_index(index_config).await.expect("Failed to create index");
    
    let invalid_doc = Document::new("invalid", "No embedding");
    let result = storage.upsert_documents("error_test", vec![invalid_doc]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_batch_operations() {
    let (storage, _temp_dir) = create_test_storage().await;
    
    // Create index
    let index_config = IndexConfig::new("batch_test", 8);
    storage.create_index(index_config).await.expect("Failed to create index");
    
    // Insert large batch of documents
    let batch_size = 100;
    let mut documents = Vec::with_capacity(batch_size);
    
    for i in 0..batch_size {
        let doc = Document::new(&format!("batch_doc_{}", i), &format!("Batch document {}", i))
            .with_embedding(generate_test_embedding(8, i as u64))
            .with_metadata("batch_id", i as i64);
        documents.push(doc);
    }
    
    let doc_ids = storage.upsert_documents("batch_test", documents).await.expect("Failed to insert batch");
    assert_eq!(doc_ids.len(), batch_size);
    
    // Test batch search
    let search_request = SearchRequest {
        index_name: "batch_test".to_string(),
        vector: generate_test_embedding(8, 50),
        top_k: 10,
        similarity_metric: Some(SimilarityMetric::Cosine),
        filter: None,
        include_metadata: false,
    };
    
    let response = storage.search(search_request).await.expect("Failed to search batch");
    assert_eq!(response.results.len(), 10);
    
    // Test batch deletion
    let ids_to_delete: Vec<String> = (0..10).map(|i| format!("batch_doc_{}", i)).collect();
    storage.delete_documents("batch_test", ids_to_delete).await.expect("Failed to delete batch");
    
    // Verify deletion
    let remaining = storage.get_documents("batch_test", vec!["batch_doc_0".to_string()], false).await
        .expect("Failed to check deleted documents");
    assert!(remaining.is_empty());
}
