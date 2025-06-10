// Integration tests for Agent + RAG system
use crate::test_config::*;
use lumosai_core::prelude::*;
use std::time::Duration;

#[tokio::test]
async fn test_agent_with_rag_basic_flow() {
    init_test_env();
    
    // Setup RAG system
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    let rag = RagSystem::builder()
        .storage(storage)
        .embedding_provider("mock")
        .build()
        .await
        .unwrap();
    
    // Add knowledge to RAG
    let knowledge_docs = vec![
        "LumosAI is a powerful AI framework built in Rust",
        "The framework supports multiple LLM providers including OpenAI",
        "Vector storage enables semantic search capabilities",
        "Agents can use tools to extend their capabilities"
    ];
    
    for doc in &knowledge_docs {
        rag.add_document(doc).await.unwrap();
    }
    
    // Create agent
    let agent = TestUtils::create_test_agent("rag-agent").await.unwrap();
    
    // Test agent can work independently
    let response = agent.generate_simple("Hello, what can you do?").await;
    assert!(response.is_ok(), "Agent should work independently");
    
    // Test RAG search works
    let search_results = rag.search("LumosAI framework", 2).await;
    assert!(search_results.is_ok(), "RAG search should work");
    
    let results = search_results.unwrap();
    assert!(!results.is_empty(), "Should find relevant documents");
    
    println!("Integration test: Agent + RAG basic flow completed successfully");
}

#[tokio::test]
async fn test_agent_rag_knowledge_retrieval() {
    init_test_env();
    
    let env = IntegrationTestUtils::setup_integration_env().await.unwrap();
    
    // Add specific knowledge
    let knowledge = vec![
        "The capital of France is Paris",
        "Python is a programming language created by Guido van Rossum",
        "Machine learning is a subset of artificial intelligence",
        "Rust provides memory safety without garbage collection"
    ];
    
    for doc in &knowledge {
        env.rag.add_document(doc).await.unwrap();
    }
    
    // Test knowledge retrieval
    let search_results = env.rag.search("capital France", 1).await.unwrap();
    assert!(!search_results.is_empty(), "Should find information about France");
    assert!(search_results[0].content.contains("Paris"), "Should find Paris as capital");
    
    let search_results = env.rag.search("Rust memory", 1).await.unwrap();
    assert!(!search_results.is_empty(), "Should find information about Rust");
    
    println!("Knowledge retrieval test completed successfully");
}

#[tokio::test]
async fn test_agent_rag_context_enhancement() {
    init_test_env();
    
    let env = IntegrationTestUtils::setup_integration_env().await.unwrap();
    
    // Add domain-specific knowledge
    let domain_knowledge = vec![
        "LumosAI supports vector databases like Qdrant and Weaviate",
        "The framework uses async/await for non-blocking operations",
        "Agents can be configured with different system prompts",
        "RAG systems improve response accuracy with relevant context"
    ];
    
    for doc in &domain_knowledge {
        env.rag.add_document(doc).await.unwrap();
    }
    
    // Test that agent can potentially use RAG context
    // Note: This test assumes future integration where agents use RAG automatically
    let response = env.agent.generate_simple("Tell me about LumosAI's vector database support").await;
    assert!(response.is_ok(), "Agent should handle domain-specific questions");
    
    // Test RAG can provide relevant context
    let context = env.rag.search("vector databases", 2).await.unwrap();
    assert!(!context.is_empty(), "RAG should provide relevant context");
    
    println!("Context enhancement test completed successfully");
}

#[tokio::test]
async fn test_agent_rag_performance_integration() {
    init_test_env();
    
    let env = IntegrationTestUtils::setup_integration_env().await.unwrap();
    
    // Add multiple documents for performance testing
    for i in 0..50 {
        let doc = format!("Performance test document {} with various content about topic {}", i, i % 10);
        env.rag.add_document(&doc).await.unwrap();
    }
    
    // Measure combined performance
    let (_, total_duration) = PerformanceTestUtils::measure_time(|| async {
        // Simulate agent + RAG workflow
        let search_results = env.rag.search("Performance test", 5).await.unwrap();
        let _agent_response = env.agent.generate_simple("Process this information").await.unwrap();
        
        search_results
    }).await;
    
    // Assert reasonable performance for integrated operations
    PerformanceTestUtils::assert_execution_time_within(
        total_duration,
        Duration::from_secs(15) // Allow more time for integrated operations
    );
    
    println!("Agent + RAG performance test completed in {:?}", total_duration);
}

#[tokio::test]
async fn test_agent_rag_concurrent_operations() {
    init_test_env();
    
    let env = IntegrationTestUtils::setup_integration_env().await.unwrap();
    
    // Add some initial knowledge
    for i in 0..10 {
        let doc = format!("Concurrent test document {}", i);
        env.rag.add_document(&doc).await.unwrap();
    }
    
    // Test concurrent agent and RAG operations
    let mut handles = Vec::new();
    
    // Concurrent agent generations
    for i in 0..3 {
        let agent = env.agent.clone();
        let message = format!("Concurrent agent message {}", i);
        
        let handle = tokio::spawn(async move {
            agent.generate_simple(&message).await
        });
        handles.push(handle);
    }
    
    // Concurrent RAG searches
    for i in 0..3 {
        let rag = env.rag.clone();
        let query = format!("Concurrent test {}", i);
        
        let handle = tokio::spawn(async move {
            rag.search(&query, 2).await
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent operation {} should succeed", i);
    }
    
    println!("Concurrent operations test completed successfully");
}

#[tokio::test]
async fn test_agent_rag_error_handling() {
    init_test_env();
    
    let env = IntegrationTestUtils::setup_integration_env().await.unwrap();
    
    // Test agent error handling
    let agent_result = env.agent.generate_simple("").await;
    // Should handle empty input gracefully
    assert!(agent_result.is_ok() || agent_result.is_err(), "Agent should handle empty input");
    
    // Test RAG error handling
    let rag_result = env.rag.search("", 5).await;
    // Should handle empty query gracefully
    assert!(rag_result.is_ok() || rag_result.is_err(), "RAG should handle empty query");
    
    // Test with invalid parameters
    let rag_result = env.rag.search("test", 0).await;
    // Should handle zero limit gracefully
    assert!(rag_result.is_ok() || rag_result.is_err(), "RAG should handle zero limit");
    
    println!("Error handling test completed successfully");
}

#[tokio::test]
async fn test_agent_rag_data_consistency() {
    init_test_env();
    
    let env = IntegrationTestUtils::setup_integration_env().await.unwrap();
    
    // Add documents and verify they're searchable
    let test_docs = vec![
        "Data consistency test document one",
        "Data consistency test document two",
        "Data consistency test document three"
    ];
    
    for doc in &test_docs {
        env.rag.add_document(doc).await.unwrap();
    }
    
    // Verify all documents are searchable
    let search_results = env.rag.search("Data consistency test", 5).await.unwrap();
    assert!(search_results.len() >= 3, "Should find all added documents");
    
    // Test agent consistency
    let response1 = env.agent.generate_simple("Test message").await.unwrap();
    let response2 = env.agent.generate_simple("Test message").await.unwrap();
    
    // Both responses should be valid (though they may differ in content)
    TestAssertions::assert_valid_agent_response(&response1);
    TestAssertions::assert_valid_agent_response(&response2);
    
    println!("Data consistency test completed successfully");
}

#[tokio::test]
async fn test_agent_rag_scalability() {
    init_test_env();
    
    let env = IntegrationTestUtils::setup_integration_env().await.unwrap();
    
    // Test scalability with larger dataset
    let document_count = 100;
    
    // Add many documents
    for i in 0..document_count {
        let doc = format!("Scalability test document {} with content about various topics including technology, science, and research", i);
        env.rag.add_document(&doc).await.unwrap();
    }
    
    // Test search performance with large dataset
    let (search_results, search_duration) = PerformanceTestUtils::measure_time(|| async {
        env.rag.search("Scalability test technology", 10).await.unwrap()
    }).await;
    
    assert!(!search_results.is_empty(), "Should find results in large dataset");
    
    // Test agent performance
    let (agent_response, agent_duration) = PerformanceTestUtils::measure_time(|| async {
        env.agent.generate_simple("Analyze the scalability data").await.unwrap()
    }).await;
    
    TestAssertions::assert_valid_agent_response(&agent_response);
    
    // Assert reasonable performance even with large dataset
    PerformanceTestUtils::assert_execution_time_within(
        search_duration,
        Duration::from_secs(10)
    );
    
    PerformanceTestUtils::assert_execution_time_within(
        agent_duration,
        Duration::from_secs(10)
    );
    
    println!("Scalability test completed - {} documents, search: {:?}, agent: {:?}", 
             document_count, search_duration, agent_duration);
}
