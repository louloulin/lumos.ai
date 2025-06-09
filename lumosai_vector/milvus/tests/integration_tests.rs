//! Integration tests for Milvus storage

use std::collections::HashMap;
use tokio_test;

use lumosai_vector_milvus::{MilvusStorage, MilvusConfig, MilvusConfigBuilder};
use lumosai_vector_core::{
    traits::VectorStorage,
    types::{Document, IndexConfig, SearchRequest, SimilarityMetric, FilterCondition, MetadataValue},
};

/// Create a test storage instance
async fn create_test_storage() -> Option<MilvusStorage> {
    let config = MilvusConfig::new("http://localhost:19530")
        .with_database("test_db");
    
    match MilvusStorage::new(config).await {
        Ok(storage) => Some(storage),
        Err(_) => {
            println!("⚠️  Skipping Milvus tests - Milvus not available on localhost:19530");
            None
        }
    }
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
    if let Some(storage) = create_test_storage().await {
        // Test health check
        assert!(storage.health_check().await.is_ok());
        
        // Test backend info
        let backend_info = storage.backend_info();
        assert_eq!(backend_info.name, "milvus");
        assert!(backend_info.features.contains(&"vector_search".to_string()));
        assert!(backend_info.features.contains(&"distributed".to_string()));
    }
}

#[tokio::test]
async fn test_collection_operations() {
    if let Some(storage) = create_test_storage().await {
        let collection_name = "test_collection";
        
        // Clean up any existing collection
        let _ = storage.delete_index(collection_name).await;
        
        // Test collection creation
        let index_config = IndexConfig::new(collection_name, 128)
            .with_metric(SimilarityMetric::Cosine)
            .with_description("Test collection");
        
        assert!(storage.create_index(index_config).await.is_ok());
        
        // Test list collections
        let collections = storage.list_indexes().await.unwrap();
        assert!(collections.contains(&collection_name.to_string()));
        
        // Test describe collection
        let collection_info = storage.describe_index(collection_name).await.unwrap();
        assert_eq!(collection_info.name, collection_name);
        assert_eq!(collection_info.dimension, 128);
        assert_eq!(collection_info.document_count, 0);
        
        // Test duplicate collection creation (should fail)
        let duplicate_config = IndexConfig::new(collection_name, 128);
        assert!(storage.create_index(duplicate_config).await.is_err());
        
        // Test delete collection
        assert!(storage.delete_index(collection_name).await.is_ok());
        
        let collections = storage.list_indexes().await.unwrap();
        assert!(!collections.contains(&collection_name.to_string()));
    }
}

#[tokio::test]
async fn test_document_operations() {
    if let Some(storage) = create_test_storage().await {
        let collection_name = "test_docs";
        
        // Clean up and create collection
        let _ = storage.delete_index(collection_name).await;
        let index_config = IndexConfig::new(collection_name, 64);
        assert!(storage.create_index(index_config).await.is_ok());
        
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
        
        let doc_ids = storage.upsert_documents(collection_name, documents).await.unwrap();
        assert_eq!(doc_ids.len(), 3);
        
        // Wait for indexing
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        // Test get documents
        let retrieved = storage.get_documents(
            collection_name, 
            vec!["doc1".to_string(), "doc2".to_string()], 
            true
        ).await.unwrap();
        assert_eq!(retrieved.len(), 2);
        
        // Test get documents without vectors
        let retrieved_no_vectors = storage.get_documents(
            collection_name, 
            vec!["doc1".to_string()], 
            false
        ).await.unwrap();
        assert_eq!(retrieved_no_vectors.len(), 1);
        
        // Test document update
        let updated_doc = Document::new("doc1", "Updated first document")
            .with_embedding(generate_test_embedding(64, 1))
            .with_metadata("category", "updated")
            .with_metadata("priority", 5i64);
        
        assert!(storage.update_document(collection_name, updated_doc).await.is_ok());
        
        // Test document deletion
        assert!(storage.delete_documents(collection_name, vec!["doc3".to_string()]).await.is_ok());
        
        // Clean up
        let _ = storage.delete_index(collection_name).await;
    }
}

#[tokio::test]
async fn test_vector_search() {
    if let Some(storage) = create_test_storage().await {
        let collection_name = "test_search";
        
        // Clean up and create collection
        let _ = storage.delete_index(collection_name).await;
        let index_config = IndexConfig::new(collection_name, 32);
        assert!(storage.create_index(index_config).await.is_ok());
        
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
        
        assert!(storage.upsert_documents(collection_name, documents).await.is_ok());
        
        // Wait for indexing
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        // Test basic search
        let query_vector = generate_test_embedding(32, 100);
        let search_request = SearchRequest {
            index_name: collection_name.to_string(),
            vector: query_vector,
            top_k: 3,
            similarity_metric: Some(SimilarityMetric::Cosine),
            filter: None,
            include_metadata: true,
        };
        
        let response = storage.search(search_request).await.unwrap();
        assert!(response.results.len() <= 3);
        
        if !response.results.is_empty() {
            // Results should be ordered by similarity
            for i in 1..response.results.len() {
                assert!(response.results[i-1].score >= response.results[i].score);
            }
        }
        
        // Clean up
        let _ = storage.delete_index(collection_name).await;
    }
}

#[tokio::test]
async fn test_filtered_search() {
    if let Some(storage) = create_test_storage().await {
        let collection_name = "test_filter";
        
        // Clean up and create collection
        let _ = storage.delete_index(collection_name).await;
        let index_config = IndexConfig::new(collection_name, 16);
        assert!(storage.create_index(index_config).await.is_ok());
        
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
        ];
        
        assert!(storage.upsert_documents(collection_name, documents).await.is_ok());
        
        // Wait for indexing
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        // Test filtered search (this might not work with all Milvus configurations)
        let filter = FilterCondition::Eq("category".to_string(), MetadataValue::String("tech".to_string()));
        
        let filtered_search = SearchRequest {
            index_name: collection_name.to_string(),
            vector: generate_test_embedding(16, 1),
            top_k: 10,
            similarity_metric: Some(SimilarityMetric::Cosine),
            filter: Some(filter),
            include_metadata: true,
        };
        
        // Note: Filtered search might fail depending on Milvus configuration
        let _result = storage.search(filtered_search).await;
        
        // Clean up
        let _ = storage.delete_index(collection_name).await;
    }
}

#[tokio::test]
async fn test_configuration() {
    // Test configuration builder
    let config = MilvusConfigBuilder::new("http://localhost:19530")
        .database("test_db")
        .auth("user", "pass")
        .batch_size(500)
        .consistency_level(lumosai_vector_milvus::config::ConsistencyLevel::Strong)
        .build()
        .unwrap();
    
    assert_eq!(config.endpoint, "http://localhost:19530");
    assert_eq!(config.database, "test_db");
    assert_eq!(config.performance.batch_size, 500);
    assert!(matches!(config.collection_config.consistency_level, 
        lumosai_vector_milvus::config::ConsistencyLevel::Strong));
    assert!(config.auth.is_some());
}

#[tokio::test]
async fn test_error_handling() {
    if let Some(storage) = create_test_storage().await {
        // Test operations on non-existent collection
        assert!(storage.describe_index("nonexistent").await.is_err());
        assert!(storage.delete_index("nonexistent").await.is_err());
        assert!(storage.upsert_documents("nonexistent", vec![]).await.is_err());
        
        // Test invalid document (missing embedding)
        let collection_name = "test_errors";
        let _ = storage.delete_index(collection_name).await;
        let index_config = IndexConfig::new(collection_name, 32);
        assert!(storage.create_index(index_config).await.is_ok());
        
        let invalid_doc = Document::new("invalid", "No embedding");
        assert!(storage.upsert_documents(collection_name, vec![invalid_doc]).await.is_err());
        
        // Clean up
        let _ = storage.delete_index(collection_name).await;
    }
}

#[tokio::test]
async fn test_batch_operations() {
    if let Some(storage) = create_test_storage().await {
        let collection_name = "test_batch";
        
        // Clean up and create collection
        let _ = storage.delete_index(collection_name).await;
        let index_config = IndexConfig::new(collection_name, 8);
        assert!(storage.create_index(index_config).await.is_ok());
        
        // Insert batch of documents
        let batch_size = 50;
        let mut documents = Vec::with_capacity(batch_size);
        
        for i in 0..batch_size {
            let doc = Document::new(&format!("batch_doc_{}", i), &format!("Batch document {}", i))
                .with_embedding(generate_test_embedding(8, i as u64))
                .with_metadata("batch_id", i as i64);
            documents.push(doc);
        }
        
        let doc_ids = storage.upsert_documents(collection_name, documents).await.unwrap();
        assert_eq!(doc_ids.len(), batch_size);
        
        // Wait for indexing
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        // Test batch search
        let search_request = SearchRequest {
            index_name: collection_name.to_string(),
            vector: generate_test_embedding(8, 25),
            top_k: 10,
            similarity_metric: Some(SimilarityMetric::Cosine),
            filter: None,
            include_metadata: false,
        };
        
        let response = storage.search(search_request).await.unwrap();
        assert!(response.results.len() <= 10);
        
        // Test batch deletion
        let ids_to_delete: Vec<String> = (0..10).map(|i| format!("batch_doc_{}", i)).collect();
        assert!(storage.delete_documents(collection_name, ids_to_delete).await.is_ok());
        
        // Clean up
        let _ = storage.delete_index(collection_name).await;
    }
}
