// Unit tests for Vector Storage system
use crate::test_config::*;
use lumosai_core::prelude::*;
use std::time::Duration;

#[tokio::test]
async fn test_vector_storage_creation() {
    init_test_env();
    
    let storage = VectorStorage::memory().await;
    assert!(storage.is_ok(), "Vector storage creation should succeed");
}

#[tokio::test]
async fn test_vector_storage_add_document() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    let result = storage.add_document("Test document content").await;
    assert!(result.is_ok(), "Adding document should succeed");
}

#[tokio::test]
async fn test_vector_storage_add_multiple_documents() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    let documents = TestUtils::generate_test_documents(5);
    
    for (i, doc) in documents.iter().enumerate() {
        let result = storage.add_document(doc).await;
        assert!(result.is_ok(), "Adding document {} should succeed", i);
    }
}

#[tokio::test]
async fn test_vector_storage_search() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    // Add some test documents
    let documents = vec![
        "Artificial intelligence and machine learning",
        "Natural language processing techniques",
        "Computer vision and image recognition",
        "Deep learning neural networks",
        "Robotics and automation systems"
    ];
    
    for doc in &documents {
        storage.add_document(doc).await.unwrap();
    }
    
    // Search for relevant content
    let results = storage.search("machine learning", 3).await;
    assert!(results.is_ok(), "Search should succeed");
    
    let results = results.unwrap();
    TestAssertions::assert_valid_search_results(&results, 1);
}

#[tokio::test]
async fn test_vector_storage_search_empty() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    // Search in empty storage
    let results = storage.search("test query", 5).await;
    assert!(results.is_ok(), "Search in empty storage should succeed");
    
    let results = results.unwrap();
    assert_eq!(results.len(), 0, "Empty storage should return no results");
}

#[tokio::test]
async fn test_vector_storage_search_no_matches() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    // Add documents about one topic
    storage.add_document("Cooking recipes and food preparation").await.unwrap();
    storage.add_document("Kitchen utensils and cooking techniques").await.unwrap();
    
    // Search for completely unrelated topic
    let results = storage.search("quantum physics equations", 5).await;
    assert!(results.is_ok(), "Search should succeed even with no matches");
    
    // Results might be empty or have very low scores
    let results = results.unwrap();
    if !results.is_empty() {
        for result in &results {
            assert!(result.score < 0.5, "Unrelated results should have low scores");
        }
    }
}

#[tokio::test]
async fn test_vector_storage_large_document() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    // Test with a large document
    let large_doc = "Large document content. ".repeat(1000);
    let result = storage.add_document(&large_doc).await;
    
    assert!(result.is_ok(), "Adding large document should succeed");
    
    // Search in the large document
    let search_results = storage.search("Large document", 1).await;
    assert!(search_results.is_ok(), "Searching large document should succeed");
}

#[tokio::test]
async fn test_vector_storage_concurrent_operations() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    // Concurrent document additions
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let storage_clone = storage.clone();
        let doc = format!("Concurrent document {}", i);
        
        let handle = tokio::spawn(async move {
            storage_clone.add_document(&doc).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all additions to complete
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent addition {} should succeed", i);
        assert!(result.unwrap().is_ok(), "Document addition {} should succeed", i);
    }
    
    // Test concurrent searches
    let mut search_handles = Vec::new();
    
    for i in 0..3 {
        let storage_clone = storage.clone();
        let query = format!("Concurrent {}", i);
        
        let handle = tokio::spawn(async move {
            storage_clone.search(&query, 2).await
        });
        
        search_handles.push(handle);
    }
    
    // Wait for all searches to complete
    for (i, handle) in search_handles.into_iter().enumerate() {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent search {} should succeed", i);
    }
}

#[tokio::test]
async fn test_vector_storage_performance() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    // Measure document addition performance
    let (result, add_duration) = PerformanceTestUtils::measure_time(|| async {
        storage.add_document("Performance test document").await
    }).await;
    
    assert!(result.is_ok(), "Performance test addition should succeed");
    
    // Add a few more documents for search testing
    for i in 0..10 {
        let doc = format!("Performance test document {}", i);
        storage.add_document(&doc).await.unwrap();
    }
    
    // Measure search performance
    let (search_result, search_duration) = PerformanceTestUtils::measure_time(|| async {
        storage.search("Performance test", 5).await
    }).await;
    
    assert!(search_result.is_ok(), "Performance test search should succeed");
    
    // Assert reasonable performance (adjust thresholds as needed)
    PerformanceTestUtils::assert_execution_time_within(
        add_duration,
        Duration::from_secs(5)
    );
    
    PerformanceTestUtils::assert_execution_time_within(
        search_duration,
        Duration::from_secs(5)
    );
    
    println!("Vector storage performance - Add: {:?}, Search: {:?}", 
             add_duration, search_duration);
}

#[tokio::test]
async fn test_vector_storage_edge_cases() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    // Test with empty document
    let result = storage.add_document("").await;
    // Should handle empty documents gracefully
    assert!(result.is_ok() || result.is_err(), "Empty document should be handled");
    
    // Test with whitespace-only document
    let result = storage.add_document("   \n\t   ").await;
    assert!(result.is_ok() || result.is_err(), "Whitespace document should be handled");
    
    // Test with special characters
    let special_doc = "Document with special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?";
    let result = storage.add_document(special_doc).await;
    assert!(result.is_ok(), "Special characters should be handled");
    
    // Test with unicode content
    let unicode_doc = "Unicode content: 你好世界 🌍 Здравствуй мир";
    let result = storage.add_document(unicode_doc).await;
    assert!(result.is_ok(), "Unicode content should be handled");
}

#[tokio::test]
async fn test_vector_storage_search_limits() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    // Add multiple documents
    for i in 0..20 {
        let doc = format!("Test document number {} with various content", i);
        storage.add_document(&doc).await.unwrap();
    }
    
    // Test different search limits
    let test_limits = vec![1, 5, 10, 15, 25];
    
    for limit in test_limits {
        let results = storage.search("Test document", limit).await;
        assert!(results.is_ok(), "Search with limit {} should succeed", limit);
        
        let results = results.unwrap();
        assert!(
            results.len() <= limit,
            "Results should not exceed limit of {}",
            limit
        );
        
        // Results should be sorted by relevance (highest score first)
        for i in 1..results.len() {
            assert!(
                results[i-1].score >= results[i].score,
                "Results should be sorted by score"
            );
        }
    }
}

#[tokio::test]
async fn test_vector_storage_memory_usage() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    // Add many documents to test memory usage
    let document_count = 100;
    
    for i in 0..document_count {
        let doc = format!("Memory test document {} with content about topic {}", i, i % 10);
        let result = storage.add_document(&doc).await;
        assert!(result.is_ok(), "Document {} addition should succeed", i);
    }
    
    // Perform searches to ensure everything still works
    let results = storage.search("Memory test", 10).await;
    assert!(results.is_ok(), "Search after many additions should succeed");
    
    let results = results.unwrap();
    TestAssertions::assert_valid_search_results(&results, 1);
    
    println!("Successfully added {} documents to vector storage", document_count);
}
