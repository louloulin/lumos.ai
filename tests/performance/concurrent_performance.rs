// Concurrent performance tests for LumosAI framework
use crate::test_config::*;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[tokio::test]
async fn test_concurrent_agent_performance() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("concurrent_perf_agent").await.unwrap();
    
    // Test different concurrency levels
    let concurrency_levels = vec![1, 5, 10, 20, 50];
    let requests_per_level = 100;
    
    for concurrency in concurrency_levels {
        println!("Testing concurrency level: {}", concurrency);
        
        let semaphore = Arc::new(Semaphore::new(concurrency));
        let start_time = Instant::now();
        let completed_counter = Arc::new(AtomicUsize::new(0));
        let success_counter = Arc::new(AtomicUsize::new(0));
        
        let mut handles = Vec::new();
        
        for i in 0..requests_per_level {
            let agent_clone = agent.clone();
            let semaphore_clone = semaphore.clone();
            let completed_counter_clone = completed_counter.clone();
            let success_counter_clone = success_counter.clone();
            let message = format!("Concurrent request {} at level {}", i, concurrency);
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();
                
                let request_start = Instant::now();
                let result = agent_clone.generate_simple(&message).await;
                let request_duration = request_start.elapsed();
                
                completed_counter_clone.fetch_add(1, Ordering::Relaxed);
                
                if result.is_ok() {
                    success_counter_clone.fetch_add(1, Ordering::Relaxed);
                }
                
                request_duration
            });
            
            handles.push(handle);
        }
        
        // Collect results
        let mut request_durations = Vec::new();
        for handle in handles {
            let duration = handle.await.unwrap();
            request_durations.push(duration);
        }
        
        let total_time = start_time.elapsed();
        let completed = completed_counter.load(Ordering::Relaxed);
        let successful = success_counter.load(Ordering::Relaxed);
        
        // Calculate statistics
        request_durations.sort();
        let avg_request_time = request_durations.iter().sum::<Duration>() / request_durations.len() as u32;
        let p50 = request_durations[request_durations.len() / 2];
        let p95 = request_durations[request_durations.len() * 95 / 100];
        let p99 = request_durations[request_durations.len() * 99 / 100];
        
        let success_rate = successful as f64 / requests_per_level as f64;
        let throughput = successful as f64 / total_time.as_secs_f64();
        
        println!("  Results for concurrency {}:", concurrency);
        println!("    Completed: {}/{}", completed, requests_per_level);
        println!("    Success rate: {:.1}%", success_rate * 100.0);
        println!("    Total time: {:?}", total_time);
        println!("    Throughput: {:.2} req/sec", throughput);
        println!("    Avg request time: {:?}", avg_request_time);
        println!("    P50: {:?}, P95: {:?}, P99: {:?}", p50, p95, p99);
        
        // Performance assertions
        assert!(success_rate >= 0.9, "Success rate should be at least 90%");
        assert!(p95 < Duration::from_secs(10), "P95 latency should be under 10s");
        
        // Small delay between tests
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

#[tokio::test]
async fn test_concurrent_vector_operations() {
    init_test_env();
    
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    
    // Test concurrent document additions and searches
    let concurrent_operations = 50;
    let documents_per_operation = 5;
    
    println!("Testing concurrent vector operations ({} operations, {} docs each)", 
             concurrent_operations, documents_per_operation);
    
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    // Concurrent document additions
    for i in 0..concurrent_operations {
        let storage_clone = storage.clone();
        
        let handle = tokio::spawn(async move {
            let mut operation_success = true;
            
            for j in 0..documents_per_operation {
                let doc = format!("Concurrent document {} from operation {}", j, i);
                if storage_clone.add_document(&doc).await.is_err() {
                    operation_success = false;
                }
            }
            
            operation_success
        });
        
        handles.push(handle);
    }
    
    // Wait for all additions to complete
    let mut successful_additions = 0;
    for handle in handles {
        if handle.await.unwrap() {
            successful_additions += 1;
        }
    }
    
    let addition_time = start_time.elapsed();
    
    // Now test concurrent searches
    let search_start = Instant::now();
    let mut search_handles = Vec::new();
    
    for i in 0..concurrent_operations {
        let storage_clone = storage.clone();
        let query = format!("Concurrent document {}", i % 10);
        
        let handle = tokio::spawn(async move {
            storage_clone.search(&query, 5).await.is_ok()
        });
        
        search_handles.push(handle);
    }
    
    let mut successful_searches = 0;
    for handle in search_handles {
        if handle.await.unwrap() {
            successful_searches += 1;
        }
    }
    
    let search_time = search_start.elapsed();
    let total_time = start_time.elapsed();
    
    println!("Concurrent vector operations results:");
    println!("  Addition phase:");
    println!("    Successful operations: {}/{}", successful_additions, concurrent_operations);
    println!("    Time: {:?}", addition_time);
    println!("  Search phase:");
    println!("    Successful searches: {}/{}", successful_searches, concurrent_operations);
    println!("    Time: {:?}", search_time);
    println!("  Total time: {:?}", total_time);
    
    // Performance assertions
    assert!(
        successful_additions >= concurrent_operations * 8 / 10,
        "At least 80% of addition operations should succeed"
    );
    
    assert!(
        successful_searches >= concurrent_operations * 9 / 10,
        "At least 90% of search operations should succeed"
    );
}

#[tokio::test]
async fn test_concurrent_rag_operations() {
    init_test_env();
    
    let rag = TestUtils::create_test_rag().await.unwrap();
    
    // Test concurrent RAG operations: document addition and retrieval
    let concurrent_users = 20;
    let operations_per_user = 10;
    
    println!("Testing concurrent RAG operations ({} users, {} ops each)", 
             concurrent_users, operations_per_user);
    
    let start_time = Instant::now();
    let success_counter = Arc::new(AtomicUsize::new(0));
    let operation_counter = Arc::new(AtomicUsize::new(0));
    
    let mut handles = Vec::new();
    
    for user_id in 0..concurrent_users {
        let rag_clone = rag.clone();
        let success_counter_clone = success_counter.clone();
        let operation_counter_clone = operation_counter.clone();
        
        let handle = tokio::spawn(async move {
            let mut user_operations = 0;
            let mut user_successes = 0;
            
            for op_id in 0..operations_per_user {
                operation_counter_clone.fetch_add(1, Ordering::Relaxed);
                user_operations += 1;
                
                if op_id % 3 == 0 {
                    // Add document operation
                    let doc = format!("User {} document {} about topic {}", user_id, op_id, op_id % 5);
                    if rag_clone.add_document(&doc).await.is_ok() {
                        user_successes += 1;
                    }
                } else {
                    // Search operation
                    let query = format!("topic {}", (user_id + op_id) % 5);
                    if rag_clone.search(&query, 3).await.is_ok() {
                        user_successes += 1;
                    }
                }
                
                // Small delay to simulate realistic usage
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            
            success_counter_clone.fetch_add(user_successes, Ordering::Relaxed);
            (user_operations, user_successes)
        });
        
        handles.push(handle);
    }
    
    // Wait for all users to complete
    let mut user_results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        user_results.push(result);
    }
    
    let total_time = start_time.elapsed();
    let total_operations = operation_counter.load(Ordering::Relaxed);
    let total_successes = success_counter.load(Ordering::Relaxed);
    let success_rate = total_successes as f64 / total_operations as f64;
    let throughput = total_successes as f64 / total_time.as_secs_f64();
    
    println!("Concurrent RAG operations results:");
    println!("  Total operations: {}", total_operations);
    println!("  Total successes: {}", total_successes);
    println!("  Success rate: {:.1}%", success_rate * 100.0);
    println!("  Total time: {:?}", total_time);
    println!("  Throughput: {:.2} ops/sec", throughput);
    
    // Analyze per-user performance
    let avg_user_success_rate = user_results.iter()
        .map(|(ops, successes)| *successes as f64 / *ops as f64)
        .sum::<f64>() / user_results.len() as f64;
    
    println!("  Average user success rate: {:.1}%", avg_user_success_rate * 100.0);
    
    // Performance assertions
    assert!(success_rate >= 0.85, "Overall success rate should be at least 85%");
    assert!(avg_user_success_rate >= 0.8, "Average user success rate should be at least 80%");
    assert!(throughput >= 10.0, "Should handle at least 10 operations per second");
}

#[tokio::test]
async fn test_concurrent_memory_operations() {
    init_test_env();
    
    // Test concurrent memory operations
    let concurrent_threads = 25;
    let operations_per_thread = 20;
    
    println!("Testing concurrent memory operations ({} threads, {} ops each)", 
             concurrent_threads, operations_per_thread);
    
    let memory = Arc::new(TestMemory::new());
    let start_time = Instant::now();
    let operation_counter = Arc::new(AtomicUsize::new(0));
    let success_counter = Arc::new(AtomicUsize::new(0));
    
    let mut handles = Vec::new();
    
    for thread_id in 0..concurrent_threads {
        let memory_clone = memory.clone();
        let operation_counter_clone = operation_counter.clone();
        let success_counter_clone = success_counter.clone();
        
        let handle = tokio::spawn(async move {
            let mut thread_successes = 0;
            
            for op_id in 0..operations_per_thread {
                operation_counter_clone.fetch_add(1, Ordering::Relaxed);
                
                let key = format!("thread_{}_key_{}", thread_id, op_id);
                let value = format!("thread_{}_value_{}", thread_id, op_id);
                
                // Perform different operations
                match op_id % 4 {
                    0 => {
                        // Store operation
                        if memory_clone.store(&key, &value).await.is_ok() {
                            thread_successes += 1;
                        }
                    }
                    1 => {
                        // Retrieve operation
                        if memory_clone.retrieve(&key).await.is_ok() {
                            thread_successes += 1;
                        }
                    }
                    2 => {
                        // Update operation
                        let new_value = format!("{}_updated", value);
                        if memory_clone.update(&key, &new_value).await.is_ok() {
                            thread_successes += 1;
                        }
                    }
                    3 => {
                        // Delete operation
                        if memory_clone.delete(&key).await.is_ok() {
                            thread_successes += 1;
                        }
                    }
                    _ => unreachable!(),
                }
            }
            
            success_counter_clone.fetch_add(thread_successes, Ordering::Relaxed);
            thread_successes
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    let mut thread_results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        thread_results.push(result);
    }
    
    let total_time = start_time.elapsed();
    let total_operations = operation_counter.load(Ordering::Relaxed);
    let total_successes = success_counter.load(Ordering::Relaxed);
    let success_rate = total_successes as f64 / total_operations as f64;
    let throughput = total_successes as f64 / total_time.as_secs_f64();
    
    println!("Concurrent memory operations results:");
    println!("  Total operations: {}", total_operations);
    println!("  Total successes: {}", total_successes);
    println!("  Success rate: {:.1}%", success_rate * 100.0);
    println!("  Total time: {:?}", total_time);
    println!("  Throughput: {:.2} ops/sec", throughput);
    
    // Analyze thread performance distribution
    let min_thread_success = *thread_results.iter().min().unwrap();
    let max_thread_success = *thread_results.iter().max().unwrap();
    let avg_thread_success = thread_results.iter().sum::<usize>() / thread_results.len();
    
    println!("  Thread performance:");
    println!("    Min: {} successes", min_thread_success);
    println!("    Max: {} successes", max_thread_success);
    println!("    Avg: {} successes", avg_thread_success);
    
    // Performance assertions
    assert!(success_rate >= 0.9, "Memory operations should have at least 90% success rate");
    assert!(throughput >= 50.0, "Should handle at least 50 memory operations per second");
    
    // Check for fairness (no thread should be starved)
    let fairness_ratio = min_thread_success as f64 / max_thread_success as f64;
    assert!(fairness_ratio >= 0.5, "Thread performance should be reasonably fair");
}

#[tokio::test]
async fn test_mixed_concurrent_workload() {
    init_test_env();
    
    // Test mixed workload with different types of operations
    let agent = TestUtils::create_test_agent("mixed_workload_agent").await.unwrap();
    let storage = TestUtils::create_test_vector_storage().await.unwrap();
    let rag = TestUtils::create_test_rag().await.unwrap();
    
    let total_operations = 100;
    let operation_types = 4; // agent, storage, rag, mixed
    
    println!("Testing mixed concurrent workload ({} operations)", total_operations);
    
    let start_time = Instant::now();
    let mut handles = Vec::new();
    let results = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    
    for i in 0..total_operations {
        let operation_type = i % operation_types;
        let results_clone = results.clone();
        
        match operation_type {
            0 => {
                // Agent operation
                let agent_clone = agent.clone();
                let handle = tokio::spawn(async move {
                    let start = Instant::now();
                    let result = agent_clone.generate_simple(&format!("Mixed workload agent request {}", i)).await;
                    let duration = start.elapsed();
                    
                    let mut results = results_clone.lock().await;
                    results.push(("agent", result.is_ok(), duration));
                });
                handles.push(handle);
            }
            1 => {
                // Storage operation
                let storage_clone = storage.clone();
                let handle = tokio::spawn(async move {
                    let start = Instant::now();
                    let result = storage_clone.add_document(&format!("Mixed workload document {}", i)).await;
                    let duration = start.elapsed();
                    
                    let mut results = results_clone.lock().await;
                    results.push(("storage", result.is_ok(), duration));
                });
                handles.push(handle);
            }
            2 => {
                // RAG operation
                let rag_clone = rag.clone();
                let handle = tokio::spawn(async move {
                    let start = Instant::now();
                    let result = rag_clone.search(&format!("Mixed workload query {}", i), 3).await;
                    let duration = start.elapsed();
                    
                    let mut results = results_clone.lock().await;
                    results.push(("rag", result.is_ok(), duration));
                });
                handles.push(handle);
            }
            3 => {
                // Mixed operation (agent + storage)
                let agent_clone = agent.clone();
                let storage_clone = storage.clone();
                let handle = tokio::spawn(async move {
                    let start = Instant::now();
                    
                    // First add document
                    let doc_result = storage_clone.add_document(&format!("Mixed operation document {}", i)).await;
                    
                    // Then generate response
                    let agent_result = agent_clone.generate_simple(&format!("Mixed operation request {}", i)).await;
                    
                    let duration = start.elapsed();
                    let success = doc_result.is_ok() && agent_result.is_ok();
                    
                    let mut results = results_clone.lock().await;
                    results.push(("mixed", success, duration));
                });
                handles.push(handle);
            }
            _ => unreachable!(),
        }
    }
    
    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    let total_time = start_time.elapsed();
    let results = results.lock().await;
    
    // Analyze results by operation type
    let mut type_stats = std::collections::HashMap::new();
    
    for (op_type, success, duration) in results.iter() {
        let entry = type_stats.entry(op_type).or_insert((0, 0, Duration::ZERO));
        entry.0 += 1; // total count
        if *success {
            entry.1 += 1; // success count
        }
        entry.2 += *duration; // total duration
    }
    
    println!("Mixed concurrent workload results:");
    println!("  Total time: {:?}", total_time);
    println!("  Total operations: {}", results.len());
    
    for (op_type, (total, successful, total_duration)) in type_stats {
        let success_rate = successful as f64 / total as f64;
        let avg_duration = total_duration / total as u32;
        
        println!("  {}: {}/{} successful ({:.1}%), avg time: {:?}", 
                 op_type, successful, total, success_rate * 100.0, avg_duration);
    }
    
    let overall_success_rate = results.iter().filter(|(_, success, _)| *success).count() as f64 / results.len() as f64;
    let throughput = results.len() as f64 / total_time.as_secs_f64();
    
    println!("  Overall success rate: {:.1}%", overall_success_rate * 100.0);
    println!("  Overall throughput: {:.2} ops/sec", throughput);
    
    // Performance assertions
    assert!(overall_success_rate >= 0.8, "Mixed workload should have at least 80% success rate");
    assert!(throughput >= 5.0, "Should handle at least 5 operations per second");
}

// Mock memory implementation for concurrent testing
#[derive(Clone)]
struct TestMemory {
    data: Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
}

impl TestMemory {
    fn new() -> Self {
        Self {
            data: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    async fn store(&self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut data = self.data.write().await;
        data.insert(key.to_string(), value.to_string());
        Ok(())
    }
    
    async fn retrieve(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }
    
    async fn update(&self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.store(key, value).await
    }
    
    async fn delete(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut data = self.data.write().await;
        data.remove(key);
        Ok(())
    }
}
