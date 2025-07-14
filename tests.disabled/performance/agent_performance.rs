// Performance tests for Agent system
use crate::test_config::*;
use lumosai_core::prelude::*;
use std::time::Duration;

#[tokio::test]
async fn benchmark_agent_creation() {
    init_test_env();
    
    let iterations = 100;
    
    let durations = PerformanceTestUtils::benchmark(
        "agent_creation",
        iterations,
        || async {
            let _agent = Agent::builder()
                .name("benchmark-agent")
                .model_name("gpt-4")
                .instructions("You are a benchmark test assistant")
                .build()
                .await
                .unwrap();
        }
    ).await;
    
    // Calculate statistics
    let avg_duration = durations.iter().sum::<Duration>() / iterations as u32;
    let min_duration = *durations.iter().min().unwrap();
    let max_duration = *durations.iter().max().unwrap();
    
    // Performance assertions
    assert!(
        avg_duration < Duration::from_millis(100),
        "Average agent creation should be under 100ms, got {:?}",
        avg_duration
    );
    
    assert!(
        max_duration < Duration::from_millis(500),
        "Max agent creation should be under 500ms, got {:?}",
        max_duration
    );
    
    println!("Agent creation benchmark:");
    println!("  Average: {:?}", avg_duration);
    println!("  Min: {:?}", min_duration);
    println!("  Max: {:?}", max_duration);
}

#[tokio::test]
async fn benchmark_agent_generation() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("perf-agent").await.unwrap();
    let iterations = 50;
    
    let durations = PerformanceTestUtils::benchmark(
        "agent_generation",
        iterations,
        || async {
            let _response = agent.generate_simple("Performance test message").await.unwrap();
        }
    ).await;
    
    // Calculate statistics
    let avg_duration = durations.iter().sum::<Duration>() / iterations as u32;
    let min_duration = *durations.iter().min().unwrap();
    let max_duration = *durations.iter().max().unwrap();
    
    // Performance assertions (adjust based on your requirements)
    assert!(
        avg_duration < Duration::from_secs(5),
        "Average generation should be under 5s, got {:?}",
        avg_duration
    );
    
    println!("Agent generation benchmark:");
    println!("  Average: {:?}", avg_duration);
    println!("  Min: {:?}", min_duration);
    println!("  Max: {:?}", max_duration);
    println!("  Throughput: {:.2} generations/sec", 1.0 / avg_duration.as_secs_f64());
}

#[tokio::test]
async fn benchmark_agent_concurrent_generation() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("concurrent-perf-agent").await.unwrap();
    let concurrent_requests = 10;
    
    let start_time = std::time::Instant::now();
    
    // Launch concurrent generations
    let mut handles = Vec::new();
    for i in 0..concurrent_requests {
        let agent_clone = agent.clone();
        let message = format!("Concurrent performance test {}", i);
        
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            let result = agent_clone.generate_simple(&message).await;
            let duration = start.elapsed();
            (result, duration)
        });
        
        handles.push(handle);
    }
    
    // Collect results
    let mut successful_requests = 0;
    let mut total_request_time = Duration::ZERO;
    
    for handle in handles {
        let (result, duration) = handle.await.unwrap();
        if result.is_ok() {
            successful_requests += 1;
            total_request_time += duration;
        }
    }
    
    let total_wall_time = start_time.elapsed();
    let avg_request_time = total_request_time / successful_requests;
    
    // Performance assertions
    assert!(
        successful_requests >= concurrent_requests * 8 / 10, // At least 80% success
        "Should have at least 80% successful concurrent requests"
    );
    
    assert!(
        total_wall_time < Duration::from_secs(30),
        "Concurrent requests should complete within 30s, took {:?}",
        total_wall_time
    );
    
    println!("Concurrent generation benchmark:");
    println!("  Concurrent requests: {}", concurrent_requests);
    println!("  Successful requests: {}", successful_requests);
    println!("  Total wall time: {:?}", total_wall_time);
    println!("  Average request time: {:?}", avg_request_time);
    println!("  Effective throughput: {:.2} req/sec", 
             successful_requests as f64 / total_wall_time.as_secs_f64());
}

#[tokio::test]
async fn benchmark_agent_memory_usage() {
    init_test_env();
    
    // Create multiple agents to test memory usage
    let agent_count = 50;
    let mut agents = Vec::new();
    
    let start_time = std::time::Instant::now();
    
    for i in 0..agent_count {
        let agent = Agent::builder()
            .name(&format!("memory-test-agent-{}", i))
            .model_name("gpt-4")
            .instructions("You are a memory test assistant")
            .build()
            .await
            .unwrap();
        
        agents.push(agent);
    }
    
    let creation_time = start_time.elapsed();
    
    // Test that all agents work
    let mut successful_generations = 0;
    let generation_start = std::time::Instant::now();
    
    for (i, agent) in agents.iter().enumerate() {
        let message = format!("Memory test message {}", i);
        if agent.generate_simple(&message).await.is_ok() {
            successful_generations += 1;
        }
    }
    
    let generation_time = generation_start.elapsed();
    
    // Performance assertions
    assert!(
        creation_time < Duration::from_secs(10),
        "Creating {} agents should take under 10s, took {:?}",
        agent_count, creation_time
    );
    
    assert!(
        successful_generations >= agent_count * 8 / 10, // At least 80% success
        "At least 80% of agents should work properly"
    );
    
    println!("Memory usage benchmark:");
    println!("  Agents created: {}", agent_count);
    println!("  Creation time: {:?}", creation_time);
    println!("  Successful generations: {}", successful_generations);
    println!("  Generation time: {:?}", generation_time);
    println!("  Avg creation time per agent: {:?}", creation_time / agent_count);
}

#[tokio::test]
async fn benchmark_agent_stress_test() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("stress-test-agent").await.unwrap();
    let stress_iterations = 100;
    let batch_size = 10;
    
    let mut total_successful = 0;
    let mut total_time = Duration::ZERO;
    
    // Run stress test in batches
    for batch in 0..(stress_iterations / batch_size) {
        let batch_start = std::time::Instant::now();
        let mut batch_handles = Vec::new();
        
        // Launch batch of concurrent requests
        for i in 0..batch_size {
            let agent_clone = agent.clone();
            let message = format!("Stress test batch {} item {}", batch, i);
            
            let handle = tokio::spawn(async move {
                agent_clone.generate_simple(&message).await
            });
            
            batch_handles.push(handle);
        }
        
        // Wait for batch to complete
        let mut batch_successful = 0;
        for handle in batch_handles {
            if handle.await.unwrap().is_ok() {
                batch_successful += 1;
            }
        }
        
        let batch_time = batch_start.elapsed();
        total_successful += batch_successful;
        total_time += batch_time;
        
        println!("Stress test batch {}: {}/{} successful in {:?}", 
                 batch, batch_successful, batch_size, batch_time);
        
        // Small delay between batches to avoid overwhelming the system
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    let success_rate = total_successful as f64 / stress_iterations as f64;
    let avg_throughput = total_successful as f64 / total_time.as_secs_f64();
    
    // Performance assertions
    assert!(
        success_rate >= 0.7, // At least 70% success under stress
        "Should maintain at least 70% success rate under stress, got {:.1}%",
        success_rate * 100.0
    );
    
    assert!(
        avg_throughput >= 1.0, // At least 1 request per second
        "Should maintain at least 1 req/sec throughput, got {:.2}",
        avg_throughput
    );
    
    println!("Stress test results:");
    println!("  Total iterations: {}", stress_iterations);
    println!("  Successful: {}", total_successful);
    println!("  Success rate: {:.1}%", success_rate * 100.0);
    println!("  Total time: {:?}", total_time);
    println!("  Average throughput: {:.2} req/sec", avg_throughput);
}

#[tokio::test]
async fn benchmark_agent_latency_distribution() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("latency-test-agent").await.unwrap();
    let iterations = 100;
    
    let mut latencies = Vec::new();
    
    for i in 0..iterations {
        let message = format!("Latency test message {}", i);
        let start = std::time::Instant::now();
        
        let result = agent.generate_simple(&message).await;
        let latency = start.elapsed();
        
        if result.is_ok() {
            latencies.push(latency);
        }
    }
    
    // Sort latencies for percentile calculation
    latencies.sort();
    
    if !latencies.is_empty() {
        let p50 = latencies[latencies.len() * 50 / 100];
        let p90 = latencies[latencies.len() * 90 / 100];
        let p95 = latencies[latencies.len() * 95 / 100];
        let p99 = latencies[latencies.len() * 99 / 100];
        
        println!("Latency distribution:");
        println!("  P50: {:?}", p50);
        println!("  P90: {:?}", p90);
        println!("  P95: {:?}", p95);
        println!("  P99: {:?}", p99);
        
        // Performance assertions
        assert!(
            p95 < Duration::from_secs(10),
            "P95 latency should be under 10s, got {:?}",
            p95
        );
        
        assert!(
            p99 < Duration::from_secs(20),
            "P99 latency should be under 20s, got {:?}",
            p99
        );
    }
}
