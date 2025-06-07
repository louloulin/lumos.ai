//! Integration tests for Lumos Vector Storage

use lumos_vector::prelude::*;
use std::collections::HashMap;

#[tokio::test]
async fn test_memory_storage_basic_operations() {
    let storage = MemoryVectorStorage::new().await.unwrap();
    
    // Test index creation
    storage.create_index("test", 3, Some(SimilarityMetric::Cosine)).await.unwrap();
    
    // Test index listing
    let indexes = storage.list_indexes().await.unwrap();
    assert!(indexes.contains(&"test".to_string()));
    
    // Test vector insertion
    let vectors = vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]];
    let ids = storage.upsert("test", vectors, None, None).await.unwrap();
    assert_eq!(ids.len(), 2);
    
    // Test querying
    let results = storage.query("test", vec![1.0, 0.0, 0.0], 5, None, false).await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results[0].score > results[1].score); // First vector should be more similar
    
    // Test index statistics
    let stats = storage.describe_index("test").await.unwrap();
    assert_eq!(stats.dimension, 3);
    assert_eq!(stats.vector_count, 2);
    
    // Test vector deletion
    storage.delete_by_id("test", &ids[0]).await.unwrap();
    let stats_after_delete = storage.describe_index("test").await.unwrap();
    assert_eq!(stats_after_delete.vector_count, 1);
    
    // Test index deletion
    storage.delete_index("test").await.unwrap();
    let indexes_after_delete = storage.list_indexes().await.unwrap();
    assert!(!indexes_after_delete.contains(&"test".to_string()));
}

#[tokio::test]
async fn test_filtering() {
    let storage = MemoryVectorStorage::new().await.unwrap();
    storage.create_index("filter_test", 2, Some(SimilarityMetric::Cosine)).await.unwrap();
    
    // Insert vectors with metadata
    let vectors = vec![
        vec![1.0, 0.0],
        vec![0.0, 1.0],
        vec![1.0, 1.0],
    ];
    
    let metadata = vec![
        HashMap::from([("type".to_string(), serde_json::json!("A"))]),
        HashMap::from([("type".to_string(), serde_json::json!("B"))]),
        HashMap::from([("type".to_string(), serde_json::json!("A"))]),
    ];
    
    storage.upsert("filter_test", vectors, None, Some(metadata)).await.unwrap();
    
    // Test equality filter
    let filter = FilterCondition::eq("type", "A");
    let results = storage.query("filter_test", vec![1.0, 0.0], 5, Some(filter), false).await.unwrap();
    assert_eq!(results.len(), 2); // Should only return vectors with type "A"
    
    // Test complex filter
    let complex_filter = FilterCondition::or(vec![
        FilterCondition::eq("type", "A"),
        FilterCondition::eq("type", "B"),
    ]);
    let complex_results = storage.query("filter_test", vec![1.0, 0.0], 5, Some(complex_filter), false).await.unwrap();
    assert_eq!(complex_results.len(), 3); // Should return all vectors
}

#[tokio::test]
async fn test_similarity_metrics() {
    let storage = MemoryVectorStorage::new().await.unwrap();
    
    // Test different similarity metrics
    let test_vectors = vec![
        vec![1.0, 0.0],
        vec![0.0, 1.0],
        vec![1.0, 1.0],
    ];
    
    // Test Cosine similarity
    storage.create_index("cosine", 2, Some(SimilarityMetric::Cosine)).await.unwrap();
    storage.upsert("cosine", test_vectors.clone(), None, None).await.unwrap();
    let cosine_results = storage.query("cosine", vec![1.0, 0.0], 3, None, false).await.unwrap();
    assert!(cosine_results[0].score >= cosine_results[1].score);
    
    // Test Euclidean similarity
    storage.create_index("euclidean", 2, Some(SimilarityMetric::Euclidean)).await.unwrap();
    storage.upsert("euclidean", test_vectors.clone(), None, None).await.unwrap();
    let euclidean_results = storage.query("euclidean", vec![1.0, 0.0], 3, None, false).await.unwrap();
    assert!(euclidean_results[0].score >= euclidean_results[1].score);
    
    // Test Dot Product similarity
    storage.create_index("dot", 2, Some(SimilarityMetric::DotProduct)).await.unwrap();
    storage.upsert("dot", test_vectors, None, None).await.unwrap();
    let dot_results = storage.query("dot", vec![1.0, 0.0], 3, None, false).await.unwrap();
    assert!(dot_results[0].score >= dot_results[1].score);
}

#[tokio::test]
async fn test_error_handling() {
    let storage = MemoryVectorStorage::new().await.unwrap();
    
    // Test querying non-existent index
    let result = storage.query("nonexistent", vec![1.0, 0.0], 5, None, false).await;
    assert!(matches!(result, Err(VectorError::IndexNotFound(_))));
    
    // Test creating duplicate index
    storage.create_index("duplicate", 2, None).await.unwrap();
    let duplicate_result = storage.create_index("duplicate", 2, None).await;
    assert!(matches!(duplicate_result, Err(VectorError::IndexAlreadyExists(_))));
    
    // Test dimension mismatch
    storage.create_index("dim_test", 3, None).await.unwrap();
    let wrong_vectors = vec![vec![1.0, 0.0]]; // 2D vector for 3D index
    let dim_result = storage.upsert("dim_test", wrong_vectors, None, None).await;
    assert!(matches!(dim_result, Err(VectorError::DimensionMismatch { .. })));
    
    // Test updating non-existent vector
    let update_result = storage.update_by_id("dim_test", "nonexistent", None, None).await;
    assert!(matches!(update_result, Err(VectorError::VectorNotFound(_))));
}

#[tokio::test]
async fn test_vector_operations() {
    let storage = MemoryVectorStorage::new().await.unwrap();
    storage.create_index("ops_test", 3, Some(SimilarityMetric::Cosine)).await.unwrap();
    
    // Insert a vector
    let vectors = vec![vec![1.0, 0.0, 0.0]];
    let ids = storage.upsert("ops_test", vectors, None, None).await.unwrap();
    let id = &ids[0];
    
    // Test get_by_id
    let retrieved = storage.get_by_id("ops_test", id, true).await.unwrap();
    assert!(retrieved.is_some());
    let vector_result = retrieved.unwrap();
    assert_eq!(vector_result.id, *id);
    assert!(vector_result.vector.is_some());
    
    // Test update_by_id
    let new_metadata = HashMap::from([("updated".to_string(), serde_json::json!(true))]);
    storage.update_by_id("ops_test", id, None, Some(new_metadata)).await.unwrap();
    
    let updated = storage.get_by_id("ops_test", id, false).await.unwrap().unwrap();
    assert_eq!(updated.metadata.unwrap().get("updated").unwrap(), &serde_json::json!(true));
    
    // Test delete_by_id
    storage.delete_by_id("ops_test", id).await.unwrap();
    let deleted = storage.get_by_id("ops_test", id, false).await.unwrap();
    assert!(deleted.is_none());
}
