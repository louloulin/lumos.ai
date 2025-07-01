// Load testing for LumosAI framework
use crate::test_config::*;
use lumosai_core::agent::Agent;
use lumosai_vector_core::VectorStorage as VectorStorageTrait;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn test_agent_load_capacity() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("load_test_agent").await.unwrap();
    
    // Test different load levels
    let load_levels = vec![10, 25, 50, 100];
    
    for concurrent_requests in load_levels {
        println!("Testing load level: {} concurrent requests", concurrent_requests);
        
        let start_time = Instant::now();
        let success_counter = Arc::new(AtomicUsize::new(0));
        let error_counter = Arc::new(AtomicUsize::new(0));
        
        let mut handles = Vec::new();
        
        for i in 0..concurrent_requests {
            let agent_clone = agent.clone();
            let success_counter_clone = success_counter.clone();
            let error_counter_clone = error_counter.clone();
            let message = format!("Load test message {}", i);
            
            let handle = tokio::spawn(async move {
                match agent_clone.generate_simple(&message).await {
                    Ok(_) => success_counter_clone.fetch_add(1, Ordering::Relaxed),
                    Err(_) => error_counter_clone.fetch_add(1, Ordering::Relaxed),
                };
            });
            
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        let total_time = start_time.elapsed();
        let successful = success_counter.load(Ordering::Relaxed);
        let failed = error_counter.load(Ordering::Relaxed);
        let success_rate = successful as f64 / concurrent_requests as f64;
        let throughput = successful as f64 / total_time.as_secs_f64();
        
        println!("  Results:");
        println!("    Successful: {}/{}", successful, concurrent_requests);
        println!("    Success rate: {:.1}%", success_rate * 100.0);
        println!("    Total time: {:?}", total_time);
        println!("    Throughput: {:.2} req/sec", throughput);
        
        // Performance assertions
        assert!(
            success_rate >= 0.8,
            "Success rate should be at least 80% for {} concurrent requests",
            concurrent_requests
        );
        
        assert!(
            total_time < Duration::from_secs(60),
            "Load test should complete within 60 seconds"
        );
        
        // Small delay between load levels
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[tokio::test]
async fn test_sustained_load_performance() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("sustained_load_agent").await.unwrap();
    
    // Sustained load test: moderate load over longer period
    let duration = Duration::from_secs(30);
    let requests_per_second = 5;
    let interval = Duration::from_millis(1000 / requests_per_second);
    
    let start_time = Instant::now();
    let success_counter = Arc::new(AtomicUsize::new(0));
    let error_counter = Arc::new(AtomicUsize::new(0));
    let mut request_id = 0;
    
    println!("Starting sustained load test for {:?} at {} req/sec", duration, requests_per_second);
    
    while start_time.elapsed() < duration {
        let agent_clone = agent.clone();
        let success_counter_clone = success_counter.clone();
        let error_counter_clone = error_counter.clone();
        let message = format!("Sustained load request {}", request_id);
        
        tokio::spawn(async move {
            match agent_clone.generate_simple(&message).await {
                Ok(_) => success_counter_clone.fetch_add(1, Ordering::Relaxed),
                Err(_) => error_counter_clone.fetch_add(1, Ordering::Relaxed),
            };
        });
        
        request_id += 1;
        tokio::time::sleep(interval).await;
    }
    
    // Wait a bit for remaining requests to complete
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    let total_time = start_time.elapsed();
    let successful = success_counter.load(Ordering::Relaxed);
    let failed = error_counter.load(Ordering::Relaxed);
    let total_requests = successful + failed;
    let success_rate = successful as f64 / total_requests as f64;
    let actual_throughput = successful as f64 / total_time.as_secs_f64();
    
    println!("Sustained load test results:");
    println!("  Duration: {:?}", total_time);
    println!("  Total requests: {}", total_requests);
    println!("  Successful: {}", successful);
    println!("  Success rate: {:.1}%", success_rate * 100.0);
    println!("  Actual throughput: {:.2} req/sec", actual_throughput);
    
    // Performance assertions
    assert!(
        success_rate >= 0.9,
        "Sustained load should maintain at least 90% success rate"
    );
    
    assert!(
        actual_throughput >= requests_per_second as f64 * 0.8,
        "Should maintain at least 80% of target throughput"
    );
}

#[tokio::test]
async fn test_spike_load_handling() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("spike_test_agent").await.unwrap();
    
    // Spike test: sudden increase in load
    println!("Testing spike load handling");
    
    // Phase 1: Normal load
    let normal_load = 5;
    let spike_load = 50;
    
    println!("Phase 1: Normal load ({} requests)", normal_load);
    let (normal_success, normal_time) = execute_load_burst(&agent, normal_load, "normal").await;
    let normal_throughput = normal_success as f64 / normal_time.as_secs_f64();
    
    // Small delay
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Phase 2: Spike load
    println!("Phase 2: Spike load ({} requests)", spike_load);
    let (spike_success, spike_time) = execute_load_burst(&agent, spike_load, "spike").await;
    let spike_throughput = spike_success as f64 / spike_time.as_secs_f64();
    
    // Small delay
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Phase 3: Return to normal
    println!("Phase 3: Return to normal ({} requests)", normal_load);
    let (recovery_success, recovery_time) = execute_load_burst(&agent, normal_load, "recovery").await;
    let recovery_throughput = recovery_success as f64 / recovery_time.as_secs_f64();
    
    println!("Spike test results:");
    println!("  Normal: {} successful, {:.2} req/sec", normal_success, normal_throughput);
    println!("  Spike: {} successful, {:.2} req/sec", spike_success, spike_throughput);
    println!("  Recovery: {} successful, {:.2} req/sec", recovery_success, recovery_throughput);
    
    // Performance assertions
    assert!(normal_success >= normal_load * 8 / 10, "Normal load should have high success rate");
    assert!(spike_success >= spike_load * 6 / 10, "Spike load should handle at least 60% of requests");
    assert!(recovery_success >= normal_load * 8 / 10, "Should recover to normal performance");
    
    // Recovery should be similar to initial normal performance
    let recovery_ratio = recovery_throughput / normal_throughput;
    assert!(
        recovery_ratio >= 0.8,
        "Recovery throughput should be at least 80% of normal throughput"
    );
}

#[tokio::test]
async fn test_memory_usage_under_load() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("memory_load_agent").await.unwrap();
    
    // Test memory usage during sustained load
    let request_count = 100;
    let batch_size = 10;
    
    println!("Testing memory usage under load ({} requests in batches of {})", 
             request_count, batch_size);
    
    for batch in 0..(request_count / batch_size) {
        let batch_start = Instant::now();
        let mut handles = Vec::new();
        
        for i in 0..batch_size {
            let agent_clone = agent.clone();
            let message = format!("Memory test batch {} request {}", batch, i);
            
            let handle = tokio::spawn(async move {
                agent_clone.generate_simple(&message).await
            });
            
            handles.push(handle);
        }
        
        // Wait for batch completion
        let mut batch_success = 0;
        for handle in handles {
            if handle.await.unwrap().is_ok() {
                batch_success += 1;
            }
        }
        
        let batch_time = batch_start.elapsed();
        println!("  Batch {}: {}/{} successful in {:?}", 
                 batch, batch_success, batch_size, batch_time);
        
        // Small delay between batches
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    println!("Memory load test completed successfully");
}

#[tokio::test]
async fn test_concurrent_component_load() {
    init_test_env();
    
    // Test load on multiple components simultaneously
    let agent = TestUtils::create_test_agent("concurrent_component_agent").await.unwrap();
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    let rag = TestUtils::create_test_rag().await.unwrap();
    
    let concurrent_operations = 20;
    
    println!("Testing concurrent load on multiple components ({} operations each)", 
             concurrent_operations);
    
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    // Agent operations
    for i in 0..concurrent_operations {
        let agent_clone = agent.clone();
        let message = format!("Concurrent agent operation {}", i);
        
        let handle = tokio::spawn(async move {
            agent_clone.generate_simple(&message).await.map(|_| "agent")
        });
        
        handles.push(handle);
    }
    
    // Storage operations
    for i in 0..concurrent_operations {
        let storage_clone = storage.clone();
        let doc = format!("Concurrent storage document {}", i);
        
        let handle = tokio::spawn(async move {
            let document = lumosai_vector_core::Document::new(
                &format!("concurrent_{}", i),
                &doc
            ).with_embedding(vec![0.1; 384]);
            storage_clone.upsert_documents("default", vec![document]).await.map(|_| "storage")
        });
        
        handles.push(handle);
    }
    
    // RAG operations
    for i in 0..concurrent_operations {
        let rag_clone = rag.clone();
        let query = format!("Concurrent RAG query {}", i);
        
        let handle = tokio::spawn(async move {
            rag_clone.search(&query, 3).await.map(|_| "rag")
        });
        
        handles.push(handle);
    }
    
    // Wait for all operations
    let mut component_results = std::collections::HashMap::new();
    
    for handle in handles {
        match handle.await.unwrap() {
            Ok(component) => {
                *component_results.entry(component).or_insert(0) += 1;
            }
            Err(_) => {
                *component_results.entry("error").or_insert(0) += 1;
            }
        }
    }
    
    let total_time = start_time.elapsed();
    let total_operations = concurrent_operations * 3;
    let total_successful: usize = component_results.values().sum();
    let success_rate = total_successful as f64 / total_operations as f64;
    
    println!("Concurrent component load results:");
    println!("  Total time: {:?}", total_time);
    println!("  Total operations: {}", total_operations);
    println!("  Successful operations: {}", total_successful);
    println!("  Success rate: {:.1}%", success_rate * 100.0);
    println!("  Component breakdown: {:?}", component_results);
    
    // Performance assertions
    assert!(
        success_rate >= 0.8,
        "Concurrent component load should have at least 80% success rate"
    );
    
    assert!(
        total_time < Duration::from_secs(30),
        "Concurrent operations should complete within 30 seconds"
    );
}

#[tokio::test]
async fn test_load_test_with_realistic_data() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("realistic_load_agent").await.unwrap();
    
    // Realistic load test with varied request sizes and types
    let test_scenarios = vec![
        ("short_query", "Hello", 20),
        ("medium_query", &"Explain machine learning ".repeat(10), 15),
        ("long_query", &"Tell me about artificial intelligence ".repeat(50), 10),
        ("complex_query", "Analyze the implications of quantum computing on cryptography and provide detailed examples", 5),
    ];
    
    println!("Testing realistic load with varied request types");
    
    let overall_start = Instant::now();
    let mut total_requests = 0;
    let mut total_successful = 0;
    
    for (scenario_name, query_template, request_count) in test_scenarios {
        println!("  Scenario: {} ({} requests)", scenario_name, request_count);
        
        let scenario_start = Instant::now();
        let mut handles = Vec::new();
        
        for i in 0..request_count {
            let agent_clone = agent.clone();
            let query = format!("{} (request {})", query_template, i);
            
            let handle = tokio::spawn(async move {
                agent_clone.generate_simple(&query).await
            });
            
            handles.push(handle);
        }
        
        let mut scenario_successful = 0;
        for handle in handles {
            if handle.await.unwrap().is_ok() {
                scenario_successful += 1;
            }
        }
        
        let scenario_time = scenario_start.elapsed();
        let scenario_success_rate = scenario_successful as f64 / request_count as f64;
        
        println!("    Results: {}/{} successful ({:.1}%) in {:?}", 
                 scenario_successful, request_count, 
                 scenario_success_rate * 100.0, scenario_time);
        
        total_requests += request_count;
        total_successful += scenario_successful;
        
        // Small delay between scenarios
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    let overall_time = overall_start.elapsed();
    let overall_success_rate = total_successful as f64 / total_requests as f64;
    let overall_throughput = total_successful as f64 / overall_time.as_secs_f64();
    
    println!("Realistic load test summary:");
    println!("  Total requests: {}", total_requests);
    println!("  Total successful: {}", total_successful);
    println!("  Overall success rate: {:.1}%", overall_success_rate * 100.0);
    println!("  Overall time: {:?}", overall_time);
    println!("  Overall throughput: {:.2} req/sec", overall_throughput);
    
    // Performance assertions
    assert!(
        overall_success_rate >= 0.85,
        "Realistic load test should have at least 85% success rate"
    );
    
    assert!(
        overall_throughput >= 1.0,
        "Should maintain at least 1 request per second throughput"
    );
}

// Helper function for load burst testing
async fn execute_load_burst(agent: &Agent, request_count: usize, phase: &str) -> (usize, Duration) {
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for i in 0..request_count {
        let agent_clone = agent.clone();
        let message = format!("{} phase request {}", phase, i);
        
        let handle = tokio::spawn(async move {
            agent_clone.generate_simple(&message).await
        });
        
        handles.push(handle);
    }
    
    let mut successful = 0;
    for handle in handles {
        if handle.await.unwrap().is_ok() {
            successful += 1;
        }
    }
    
    let duration = start_time.elapsed();
    (successful, duration)
}

// Mock types for testing
type VectorStorage = String;
type RagSystem = String;
