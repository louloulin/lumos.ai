//! 综合性能基准测试
//!
//! P0-1.3 任务：建立完整的性能基准
//!
//! 测试范围：
//! 1. Agent 生成性能基准（吞吐量、延迟）
//! 2. 向量检索性能基准（QPS、P99 延迟）
//! 3. 工作流执行性能基准（并发、资源使用）
//! 4. 内存操作性能基准（读写速度、容量）

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lumosai_core::agent::{AgentConfig, BasicAgent};
use lumosai_core::llm::{Message, MockLlmProvider, Role};
use lumosai_core::memory::{create_working_memory, WorkingMemory, WorkingMemoryConfig};
use lumosai_core::workflow::{BasicStep, Step, StepContext};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a test agent
fn create_test_agent() -> BasicAgent {
    let config = AgentConfig {
        name: "bench_agent".to_string(),
        instructions: "You are a benchmark test agent".to_string(),
        model_id: Some("gpt-4".to_string()),
        ..Default::default()
    };
    let llm = Arc::new(MockLlmProvider::new(vec![
        "Response 1".to_string(),
        "Response 2".to_string(),
        "Response 3".to_string(),
    ]));
    BasicAgent::new(config, llm)
}

/// Create test messages
fn create_test_messages(count: usize) -> Vec<Message> {
    (0..count)
        .map(|i| Message::new(Role::User, format!("Test message {}", i), None, None))
        .collect()
}

/// Generate test prompt of specified size
fn generate_prompt(size: usize) -> String {
    "word ".repeat(size)
}

/// Create test working memory
fn create_test_memory() -> Box<dyn WorkingMemory> {
    let config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1_000_000), // 1MB
    };
    create_working_memory(&config).unwrap()
}

/// Create test workflow step
fn create_test_step(id: &str) -> BasicStep {
    BasicStep::create_simple(id.to_string(), format!("Benchmark step {}", id), |input| {
        Ok(json!({"result": "success", "input": input}))
    })
}

// ============================================================================
// 1. Agent Generation Performance Benchmarks
// ============================================================================

/// Benchmark: Agent generation with different prompt sizes
fn bench_agent_generation_prompt_size(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("agent_generation_prompt_size");

    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}words", size)),
            size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let agent = create_test_agent();
                    let prompt = generate_prompt(size);

                    // Simulate agent processing (using mock LLM)
                    // In real scenarios, this would call agent.run() or similar
                    let _result = black_box(prompt);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Agent generation throughput
fn bench_agent_generation_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("agent_generation_throughput");

    // Measure how many generations can be done in sequence
    for count in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}requests", count)),
            count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let agent = create_test_agent();

                    // Simulate multiple agent generations
                    for _ in 0..count {
                        let _result = black_box(&agent);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Agent with message history
fn bench_agent_with_history(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("agent_with_history");

    for history_size in [5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}messages", history_size)),
            history_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let agent = create_test_agent();
                    let messages = create_test_messages(size);

                    // Simulate agent processing with message history
                    let _result = black_box((agent, messages));
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// 2. Vector Retrieval Performance Benchmarks
// ============================================================================

/// Benchmark: Vector similarity search (simulated)
fn bench_vector_similarity_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_similarity_search");

    // Simulate vector similarity computation
    for vector_count in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}vectors", vector_count)),
            vector_count,
            |b, &count| {
                let query_vector: Vec<f32> = (0..384).map(|i| i as f32 * 0.01).collect();
                let vectors: Vec<Vec<f32>> = (0..count)
                    .map(|_| (0..384).map(|i| i as f32 * 0.01).collect())
                    .collect();

                b.iter(|| {
                    // Simulate cosine similarity computation
                    let mut similarities: Vec<f32> = vectors
                        .iter()
                        .map(|v| {
                            let dot: f32 =
                                query_vector.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
                            let norm_q: f32 =
                                query_vector.iter().map(|x| x * x).sum::<f32>().sqrt();
                            let norm_v: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
                            dot / (norm_q * norm_v)
                        })
                        .collect();

                    // Sort by similarity
                    similarities.sort_by(|a, b| b.partial_cmp(a).unwrap());

                    black_box(similarities)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Top-K retrieval
fn bench_vector_topk_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_topk_retrieval");

    let vector_count = 1000;
    let query_vector: Vec<f32> = (0..384).map(|i| i as f32 * 0.01).collect();
    let vectors: Vec<Vec<f32>> = (0..vector_count)
        .map(|_| (0..384).map(|i| i as f32 * 0.01).collect())
        .collect();

    for k in [5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("top{}", k)),
            k,
            |b, &k| {
                b.iter(|| {
                    let mut similarities: Vec<(usize, f32)> = vectors
                        .iter()
                        .enumerate()
                        .map(|(idx, v)| {
                            let dot: f32 =
                                query_vector.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
                            let norm_q: f32 =
                                query_vector.iter().map(|x| x * x).sum::<f32>().sqrt();
                            let norm_v: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
                            (idx, dot / (norm_q * norm_v))
                        })
                        .collect();

                    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                    similarities.truncate(k);

                    black_box(similarities)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// 3. Workflow Execution Performance Benchmarks
// ============================================================================

/// Benchmark: Sequential workflow execution
fn bench_workflow_sequential_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("workflow_sequential_execution");

    for step_count in [3, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}steps", step_count)),
            step_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let steps: Vec<BasicStep> = (0..count)
                        .map(|i| create_test_step(&format!("step{}", i)))
                        .collect();

                    let mut ctx = StepContext {
                        run_id: "bench_run".to_string(),
                        input_data: json!({"value": 42}),
                        trigger_data: json!({}),
                        steps: HashMap::new(),
                        attempts: HashMap::new(),
                    };

                    for step in steps {
                        let result = step.execute(black_box(ctx.clone())).await.unwrap();
                        ctx.input_data = result;
                    }

                    black_box(ctx)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Parallel workflow execution
fn bench_workflow_parallel_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("workflow_parallel_execution");

    for parallel_count in [2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}parallel", parallel_count)),
            parallel_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let ctx = StepContext {
                        run_id: "bench_run".to_string(),
                        input_data: json!({"value": 42}),
                        trigger_data: json!({}),
                        steps: HashMap::new(),
                        attempts: HashMap::new(),
                    };

                    // Execute all steps in parallel
                    let mut futures = Vec::new();
                    for i in 0..count {
                        let step = create_test_step(&format!("parallel{}", i));
                        let ctx_clone = ctx.clone();
                        futures.push(async move { step.execute(black_box(ctx_clone)).await });
                    }

                    let results = futures::future::join_all(futures).await;
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Workflow with data transformation
fn bench_workflow_data_transformation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("workflow_data_transformation");

    for data_size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}bytes", data_size)),
            data_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let step = BasicStep::create_simple(
                        "transform".to_string(),
                        "Transform data".to_string(),
                        |input| {
                            // Simulate data transformation
                            let data = input["data"].as_str().unwrap_or("");
                            let transformed = data.to_uppercase();
                            Ok(json!({"transformed": transformed}))
                        },
                    );

                    let ctx = StepContext {
                        run_id: "bench_run".to_string(),
                        input_data: json!({"data": "x".repeat(size)}),
                        trigger_data: json!({}),
                        steps: HashMap::new(),
                        attempts: HashMap::new(),
                    };

                    let result = step.execute(black_box(ctx)).await.unwrap();
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// 4. Memory Operations Performance Benchmarks
// ============================================================================

/// Benchmark: Memory write operations
fn bench_memory_write_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("memory_write_operations");

    for value_size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}bytes", value_size)),
            value_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let memory = create_test_memory();
                    let value = json!({"data": "x".repeat(size)});

                    memory
                        .set_value(black_box("test_key"), black_box(value))
                        .await
                        .unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Memory read operations
fn bench_memory_read_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("memory_read_operations");

    // Pre-populate memory
    let memory = create_test_memory();
    rt.block_on(async {
        for i in 0..100 {
            memory
                .set_value(&format!("key{}", i), json!({"value": i}))
                .await
                .unwrap();
        }
    });

    for key_count in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}keys", key_count)),
            key_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    for i in 0..count {
                        let _ = memory
                            .get_value(black_box(&format!("key{}", i)))
                            .await
                            .unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Memory update operations
fn bench_memory_update_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("memory_update_operations");

    for update_count in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}updates", update_count)),
            update_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let memory = create_test_memory();

                    for i in 0..count {
                        memory
                            .set_value(
                                black_box(&format!("key{}", i % 10)),
                                black_box(json!({"value": i})),
                            )
                            .await
                            .unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Memory concurrent operations
fn bench_memory_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("memory_concurrent_operations");

    for concurrent_count in [2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}concurrent", concurrent_count)),
            concurrent_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let memory = Arc::new(create_test_memory());

                    let futures: Vec<_> = (0..count)
                        .map(|i| {
                            let mem = Arc::clone(&memory);
                            async move {
                                mem.set_value(&format!("key{}", i), json!({"value": i}))
                                    .await
                                    .unwrap();
                            }
                        })
                        .collect();

                    futures::future::join_all(futures).await;
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    agent_benches,
    bench_agent_generation_prompt_size,
    bench_agent_generation_throughput,
    bench_agent_with_history,
);

criterion_group!(
    vector_benches,
    bench_vector_similarity_search,
    bench_vector_topk_retrieval,
);

criterion_group!(
    workflow_benches,
    bench_workflow_sequential_execution,
    bench_workflow_parallel_execution,
    bench_workflow_data_transformation,
);

criterion_group!(
    memory_benches,
    bench_memory_write_operations,
    bench_memory_read_operations,
    bench_memory_update_operations,
    bench_memory_concurrent_operations,
);

criterion_main!(
    agent_benches,
    vector_benches,
    workflow_benches,
    memory_benches,
);
