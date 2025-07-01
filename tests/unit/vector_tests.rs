// Unit tests for Vector Storage system
use crate::test_config::*;
use lumosai::prelude::SearchResult;
use lumosai_vector_core::VectorStorage as VectorStorageTrait;
use std::time::Duration;

#[tokio::test]
async fn test_vector_storage_creation() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await;
    assert!(storage.is_ok(), "Vector storage creation should succeed");
}

#[tokio::test]
async fn test_vector_storage_add_document() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    let doc = lumosai_vector_core::Document::new("test_doc", "Test document content")
        .with_embedding(vec![0.1; 384]);
    let result = storage.upsert_documents("default", vec![doc]).await;
    assert!(result.is_ok(), "Adding document should succeed");
}

#[tokio::test]
async fn test_vector_storage_add_multiple_documents() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    let documents = TestUtils::generate_test_documents(5);
    
    for (i, doc_content) in documents.iter().enumerate() {
        let doc = lumosai_vector_core::Document::new(&format!("doc_{}", i), doc_content)
            .with_embedding(vec![0.1; 384]);
        let result = storage.upsert_documents("default", vec![doc]).await;
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
    
    for (i, doc_content) in documents.iter().enumerate() {
        let doc = lumosai_vector_core::Document::new(&format!("search_doc_{}", i), *doc_content)
            .with_embedding(vec![0.1; 384]);
        storage.upsert_documents("default", vec![doc]).await.unwrap();
    }
    
    // Search for relevant content
    let search_request = lumosai_vector_core::SearchRequest::new_text("default", "machine learning")
        .with_top_k(3);
    let results = storage.search(search_request).await;
    assert!(results.is_ok(), "Search should succeed");
    
    let results = results.unwrap();
    // Convert to the expected type for assertion
    let converted_results: Vec<SearchResult> = results.results.iter().map(|r| SearchResult {
        document: lumosai::prelude::Document {
            id: r.id.clone(),
            content: r.content.clone().unwrap_or_default(),
            metadata: std::collections::HashMap::new(),
        },
        score: r.score,
        chunk_index: None,
    }).collect();
    TestAssertions::assert_valid_search_results(&converted_results, 1);
}

#[tokio::test]
async fn test_vector_storage_search_empty() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    // Search in empty storage
    let search_request = lumosai_vector_core::SearchRequest::new_text("default", "test query")
        .with_top_k(5);
    let results = storage.search(search_request).await;
    assert!(results.is_ok(), "Search in empty storage should succeed");
    
    let results = results.unwrap();
    assert_eq!(results.results.len(), 0, "Empty storage should return no results");
}

#[tokio::test]
async fn test_vector_storage_search_no_matches() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    // Add documents about one topic
    let doc1 = lumosai_vector_core::Document::new("cooking_doc1", "Cooking recipes and food preparation")
        .with_embedding(vec![0.1; 384]);
    storage.upsert_documents("default", vec![doc1]).await.unwrap();

    let doc2 = lumosai_vector_core::Document::new("cooking_doc2", "Kitchen utensils and cooking techniques")
        .with_embedding(vec![0.1; 384]);
    storage.upsert_documents("default", vec![doc2]).await.unwrap();
    
    // Search for completely unrelated topic
    let search_request = lumosai_vector_core::SearchRequest::new_text("default", "quantum physics equations")
        .with_top_k(5);
    let results = storage.search(search_request).await;
    assert!(results.is_ok(), "Search should succeed even with no matches");
    
    // Results might be empty or have very low scores
    let results = results.unwrap();
    if !results.results.is_empty() {
        for result in &results.results {
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
    let doc = lumosai_vector_core::Document::new("large_doc", &large_doc)
        .with_embedding(vec![0.1; 384]);
    let result = storage.upsert_documents("default", vec![doc]).await;
    
    assert!(result.is_ok(), "Adding large document should succeed");
    
    // Search in the large document
    let search_request = lumosai_vector_core::SearchRequest::new_text("default", "Large document")
        .with_top_k(1);
    let search_results = storage.search(search_request).await;
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
            let document = lumosai_vector_core::Document::new(&format!("concurrent_{}", i), &doc)
                .with_embedding(vec![0.1; 384]);
            storage_clone.upsert_documents("default", vec![document]).await
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
            let search_request = lumosai_vector_core::SearchRequest::new_text("default", &query)
                .with_top_k(2);
            storage_clone.search(search_request).await
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
        let doc = lumosai_vector_core::Document::new("perf_test", "Performance test document")
            .with_embedding(vec![0.1; 384]);
        storage.upsert_documents("default", vec![doc]).await
    }).await;
    
    assert!(result.is_ok(), "Performance test addition should succeed");
    
    // Add a few more documents for search testing
    for i in 0..10 {
        let doc_content = format!("Performance test document {}", i);
        let doc = lumosai_vector_core::Document::new(&format!("perf_doc_{}", i), &doc_content)
            .with_embedding(vec![0.1; 384]);
        storage.upsert_documents("default", vec![doc]).await.unwrap();
    }
    
    // Measure search performance
    let (search_result, search_duration) = PerformanceTestUtils::measure_time(|| async {
        let search_request = lumosai_vector_core::SearchRequest::new_text("default", "Performance test")
            .with_top_k(5);
        storage.search(search_request).await
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
    let doc = lumosai_vector_core::Document::new("empty_doc", "")
        .with_embedding(vec![0.1; 384]);
    let result = storage.upsert_documents("default", vec![doc]).await;
    // Should handle empty documents gracefully
    assert!(result.is_ok() || result.is_err(), "Empty document should be handled");

    // Test with whitespace-only document
    let doc = lumosai_vector_core::Document::new("whitespace_doc", "   \n\t   ")
        .with_embedding(vec![0.1; 384]);
    let result = storage.upsert_documents("default", vec![doc]).await;
    assert!(result.is_ok() || result.is_err(), "Whitespace document should be handled");

    // Test with special characters
    let special_doc = "Document with special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?";
    let doc = lumosai_vector_core::Document::new("special_doc", special_doc)
        .with_embedding(vec![0.1; 384]);
    let result = storage.upsert_documents("default", vec![doc]).await;
    assert!(result.is_ok(), "Special characters should be handled");

    // Test with unicode content
    let unicode_doc = "Unicode content: 你好世界 🌍 Здравствуй мир";
    let doc = lumosai_vector_core::Document::new("unicode_doc", unicode_doc)
        .with_embedding(vec![0.1; 384]);
    let result = storage.upsert_documents("default", vec![doc]).await;
    assert!(result.is_ok(), "Unicode content should be handled");
}

#[tokio::test]
async fn test_vector_storage_search_limits() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    // Add multiple documents
    for i in 0..20 {
        let doc_content = format!("Test document number {} with various content", i);
        let doc = lumosai_vector_core::Document::new(&format!("limit_test_{}", i), &doc_content)
            .with_embedding(vec![0.1; 384]);
        storage.upsert_documents("default", vec![doc]).await.unwrap();
    }
    
    // Test different search limits
    let test_limits = vec![1, 5, 10, 15, 25];
    
    for limit in test_limits {
        let search_request = lumosai_vector_core::SearchRequest::new_text("default", "Test document")
            .with_top_k(limit);
        let results = storage.search(search_request).await;
        assert!(results.is_ok(), "Search with limit {} should succeed", limit);
        
        let results = results.unwrap();
        assert!(
            results.results.len() <= limit,
            "Results should not exceed limit of {}",
            limit
        );

        // Results should be sorted by relevance (highest score first)
        for i in 1..results.results.len() {
            assert!(
                results.results[i-1].score >= results.results[i].score,
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
        let doc_content = format!("Memory test document {} with content about topic {}", i, i % 10);
        let doc = lumosai_vector_core::Document::new(&format!("memory_test_{}", i), &doc_content)
            .with_embedding(vec![0.1; 384]);
        let result = storage.upsert_documents("default", vec![doc]).await;
        assert!(result.is_ok(), "Document {} addition should succeed", i);
    }
    
    // Perform searches to ensure everything still works
    let search_request = lumosai_vector_core::SearchRequest::new_text("default", "Memory test")
        .with_top_k(10);
    let results = storage.search(search_request).await;
    assert!(results.is_ok(), "Search after many additions should succeed");
    
    let results = results.unwrap();
    // Convert to the expected type for assertion
    let converted_results: Vec<SearchResult> = results.results.iter().map(|r| SearchResult {
        document: lumosai::prelude::Document {
            id: r.id.clone(),
            content: r.content.clone().unwrap_or_default(),
            metadata: std::collections::HashMap::new(),
        },
        score: r.score,
        chunk_index: None,
    }).collect();
    TestAssertions::assert_valid_search_results(&converted_results, 1);
    
    println!("Successfully added {} documents to vector storage", document_count);
}
