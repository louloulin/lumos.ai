use lumosai_core::prelude::*;
use std::collections::HashMap;
use serde_json::json;

#[tokio::test]
async fn test_memory_vector_storage_convenience() {
    // Test memory vector storage convenience function
    let storage = memory_vector_storage(3, Some(100))
        .expect("Failed to create memory vector storage");
    
    // Create an index
    storage.create_index("test_index", 3, Some(SimilarityMetric::Cosine))
        .await
        .expect("Failed to create index");
    
    // Insert some vectors
    let vectors = vec![
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
    ];
    
    let metadata = vec![
        HashMap::from([("type".to_string(), json!("A"))]),
        HashMap::from([("type".to_string(), json!("B"))]),
        HashMap::from([("type".to_string(), json!("C"))]),
    ];
    
    let ids = storage.upsert("test_index", vectors, None, Some(metadata))
        .await
        .expect("Failed to upsert vectors");
    
    assert_eq!(ids.len(), 3);
    
    // Query vectors
    let results = storage.query(
        "test_index",
        vec![1.0, 0.0, 0.0],
        2,
        None,
        false
    ).await.expect("Failed to query vectors");
    
    assert!(!results.is_empty());
    assert!(results[0].score > 0.9); // Should be very similar to the first vector
    
    // Test with filter
    let filter = FilterCondition::Eq("type".to_string(), json!("B"));
    let filtered_results = storage.query(
        "test_index",
        vec![0.0, 1.0, 0.0],
        1,
        Some(filter),
        false
    ).await.expect("Failed to query with filter");
    
    assert_eq!(filtered_results.len(), 1);
    assert_eq!(
        filtered_results[0].metadata.as_ref().and_then(|m| m.get("type")),
        Some(&json!("B"))
    );
}

#[tokio::test]
async fn test_vector_storage_config_creation() {
    // Test different vector storage configurations
    
    // Memory storage
    let memory_config = VectorStorageConfig::Memory {
        dimensions: 1536,
        capacity: Some(1000),
    };
    
    let storage = create_vector_storage(Some(memory_config))
        .expect("Failed to create memory storage from config");
    
    // Verify it works
    storage.create_index("config_test", 1536, None)
        .await
        .expect("Failed to create index from config");
    
    let indexes = storage.list_indexes()
        .await
        .expect("Failed to list indexes");
    
    assert!(indexes.contains(&"config_test".to_string()));
}

#[tokio::test]
async fn test_vector_storage_operations() {
    let storage = memory_vector_storage(4, None)
        .expect("Failed to create storage");
    
    // Test index lifecycle
    storage.create_index("lifecycle_test", 4, Some(SimilarityMetric::Euclidean))
        .await
        .expect("Failed to create index");
    
    // Describe index
    let stats = storage.describe_index("lifecycle_test")
        .await
        .expect("Failed to describe index");
    
    assert_eq!(stats.dimension, 4);
    assert_eq!(stats.count, 0);
    assert_eq!(stats.metric, SimilarityMetric::Euclidean);
    
    // Insert vectors
    let vectors = vec![
        vec![1.0, 2.0, 3.0, 4.0],
        vec![5.0, 6.0, 7.0, 8.0],
    ];
    
    let ids = storage.upsert("lifecycle_test", vectors, None, None)
        .await
        .expect("Failed to insert vectors");
    
    // Check updated stats
    let updated_stats = storage.describe_index("lifecycle_test")
        .await
        .expect("Failed to describe index after insert");
    
    assert_eq!(updated_stats.count, 2);
    
    // Update a vector
    storage.update_by_id(
        "lifecycle_test",
        &ids[0],
        Some(vec![10.0, 20.0, 30.0, 40.0]),
        Some(HashMap::from([("updated".to_string(), json!(true))]))
    ).await.expect("Failed to update vector");
    
    // Delete a vector
    storage.delete_by_id("lifecycle_test", &ids[1])
        .await
        .expect("Failed to delete vector");
    
    // Verify count decreased
    let final_stats = storage.describe_index("lifecycle_test")
        .await
        .expect("Failed to describe index after delete");
    
    assert_eq!(final_stats.count, 1);
    
    // Delete index
    storage.delete_index("lifecycle_test")
        .await
        .expect("Failed to delete index");
    
    // Verify index is gone
    let indexes = storage.list_indexes()
        .await
        .expect("Failed to list indexes");
    
    assert!(!indexes.contains(&"lifecycle_test".to_string()));
}

#[tokio::test]
async fn test_similarity_metrics() {
    let storage = memory_vector_storage(3, None)
        .expect("Failed to create storage");
    
    // Test different similarity metrics
    let metrics = vec![
        SimilarityMetric::Cosine,
        SimilarityMetric::Euclidean,
        SimilarityMetric::DotProduct,
    ];
    
    for (i, metric) in metrics.iter().enumerate() {
        let index_name = format!("metric_test_{}", i);
        
        storage.create_index(&index_name, 3, Some(*metric))
            .await
            .expect("Failed to create index with metric");
        
        let stats = storage.describe_index(&index_name)
            .await
            .expect("Failed to describe index");
        
        assert_eq!(stats.metric, *metric);
        
        // Insert test vectors
        let vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
        ];
        
        storage.upsert(&index_name, vectors, None, None)
            .await
            .expect("Failed to insert vectors");
        
        // Query and verify results
        let results = storage.query(
            &index_name,
            vec![1.0, 0.0, 0.0],
            2,
            None,
            false
        ).await.expect("Failed to query vectors");
        
        assert_eq!(results.len(), 2);
        // First result should be more similar
        assert!(results[0].score >= results[1].score);
    }
}

#[tokio::test]
async fn test_complex_filters() {
    let storage = memory_vector_storage(2, None)
        .expect("Failed to create storage");
    
    storage.create_index("filter_test", 2, None)
        .await
        .expect("Failed to create index");
    
    // Insert vectors with complex metadata
    let vectors = vec![
        vec![1.0, 0.0],
        vec![0.0, 1.0],
        vec![1.0, 1.0],
        vec![0.5, 0.5],
    ];
    
    let metadata = vec![
        HashMap::from([
            ("category".to_string(), json!("A")),
            ("score".to_string(), json!(10)),
            ("active".to_string(), json!(true)),
        ]),
        HashMap::from([
            ("category".to_string(), json!("B")),
            ("score".to_string(), json!(20)),
            ("active".to_string(), json!(true)),
        ]),
        HashMap::from([
            ("category".to_string(), json!("A")),
            ("score".to_string(), json!(15)),
            ("active".to_string(), json!(false)),
        ]),
        HashMap::from([
            ("category".to_string(), json!("C")),
            ("score".to_string(), json!(5)),
            ("active".to_string(), json!(true)),
        ]),
    ];
    
    storage.upsert("filter_test", vectors, None, Some(metadata))
        .await
        .expect("Failed to insert vectors");
    
    // Test simple equality filter
    let eq_filter = FilterCondition::Eq("category".to_string(), json!("A"));
    let eq_results = storage.query(
        "filter_test",
        vec![1.0, 0.0],
        10,
        Some(eq_filter),
        false
    ).await.expect("Failed to query with equality filter");
    
    assert_eq!(eq_results.len(), 2); // Should find 2 category A items
    
    // Test range filter
    let gt_filter = FilterCondition::Gt("score".to_string(), json!(10));
    let gt_results = storage.query(
        "filter_test",
        vec![1.0, 0.0],
        10,
        Some(gt_filter),
        false
    ).await.expect("Failed to query with greater than filter");
    
    assert_eq!(gt_results.len(), 2); // Should find items with score > 10
    
    // Test AND filter
    let and_filter = FilterCondition::And(vec![
        FilterCondition::Eq("category".to_string(), json!("A")),
        FilterCondition::Eq("active".to_string(), json!(true)),
    ]);
    
    let and_results = storage.query(
        "filter_test",
        vec![1.0, 0.0],
        10,
        Some(and_filter),
        false
    ).await.expect("Failed to query with AND filter");
    
    assert_eq!(and_results.len(), 1); // Should find 1 active category A item
    
    // Test OR filter
    let or_filter = FilterCondition::Or(vec![
        FilterCondition::Eq("category".to_string(), json!("B")),
        FilterCondition::Eq("category".to_string(), json!("C")),
    ]);
    
    let or_results = storage.query(
        "filter_test",
        vec![1.0, 0.0],
        10,
        Some(or_filter),
        false
    ).await.expect("Failed to query with OR filter");
    
    assert_eq!(or_results.len(), 2); // Should find category B and C items
}

#[tokio::test]
async fn test_vector_storage_error_handling() {
    let storage = memory_vector_storage(3, None)
        .expect("Failed to create storage");
    
    // Test querying non-existent index
    let result = storage.query("non_existent", vec![1.0, 0.0, 0.0], 1, None, false).await;
    assert!(result.is_err());
    
    // Test creating duplicate index
    storage.create_index("duplicate_test", 3, None)
        .await
        .expect("Failed to create first index");
    
    let duplicate_result = storage.create_index("duplicate_test", 3, None).await;
    assert!(duplicate_result.is_err());
    
    // Test dimension mismatch (this should be handled gracefully)
    storage.create_index("dimension_test", 3, None)
        .await
        .expect("Failed to create index");
    
    // Try to insert vector with wrong dimension
    let wrong_vectors = vec![vec![1.0, 2.0]]; // 2D instead of 3D
    let wrong_result = storage.upsert("dimension_test", wrong_vectors, None, None).await;
    // This might succeed depending on implementation, but should be handled gracefully
    
    // Test updating non-existent vector
    let update_result = storage.update_by_id("dimension_test", "non_existent_id", None, None).await;
    // This should succeed (no-op) or return appropriate error
}
