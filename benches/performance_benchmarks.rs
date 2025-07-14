use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, async_executor::FuturesExecutor};
use std::time::Duration;
use lumosai::agent::{Agent, AgentBuilder};
use lumosai_core::llm::{LLMProvider, mock::MockLlmProvider};
use lumosai_rag::document::chunker::TextChunker;
use lumosai_rag::types::ChunkingConfig;
use lumosai_vector::memory::MemoryVectorStorage;
use lumosai_core::workflow::enhanced::EnhancedWorkflow;
use lumosai_network::{AgentNode, AgentNetwork, AgentId};
use lumosai_network::network::AgentConfig;
use lumosai_core::agent::RuntimeContext;
use serde_json::json;

/// Agent性能基准测试
fn bench_agent_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("agent_creation");
    
    group.bench_function("basic_agent", |b| {
        b.iter(|| {
            let llm = Box::new(MockLlmProvider::new()) as Box<dyn LLMProvider>;
            let agent = AgentBuilder::new()
                .with_llm(llm)
                .build()
                .expect("创建Agent失败");
            black_box(agent)
        })
    });

    group.bench_function("agent_with_system_prompt", |b| {
        b.iter(|| {
            let llm = Box::new(MockLlmProvider::new()) as Box<dyn LLMProvider>;
            let agent = AgentBuilder::new()
                .with_llm(llm)
                .with_system_prompt("你是一个有用的AI助手")
                .build()
                .expect("创建Agent失败");
            black_box(agent)
        })
    });
    
    group.finish();
}

/// Agent文本生成性能基准测试
fn bench_agent_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("agent_generation");

    // 设置较长的测量时间，因为涉及异步操作
    group.measurement_time(Duration::from_secs(10));

    let llm = Box::new(MockLlmProvider::new()) as Box<dyn LLMProvider>;
    let agent = AgentBuilder::new()
        .with_llm(llm)
        .build()
        .expect("创建Agent失败");

    let test_inputs = vec![
        ("short", "你好"),
        ("medium", "请解释一下人工智能的基本概念和应用领域"),
        ("long", "请详细分析机器学习、深度学习和人工智能之间的关系，并举例说明它们在实际应用中的区别和联系"),
    ];

    for (name, input) in test_inputs {
        group.bench_with_input(BenchmarkId::new("generation", name), input, |b, input| {
            b.to_async(FuturesExecutor).iter(|| async {
                let result = agent.generate(input).await;
                black_box(result)
            })
        });
    }

    group.finish();
}

/// 文档分块性能基准测试
fn bench_document_chunking(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_chunking");

    let chunker = TextChunker::new(ChunkingConfig::default());

    let test_documents = vec![
        ("small", "这是一个简短的测试文档。"),
        ("medium", &"这是一个中等长度的测试文档。".repeat(50)),
        ("large", &"这是一个较长的测试文档。".repeat(200)),
    ];

    for (name, content) in test_documents {
        group.bench_with_input(BenchmarkId::new("chunk_document", name), content, |b, content| {
            b.iter(|| {
                let result = chunker.chunk_text(content);
                black_box(result)
            })
        });
    }

    group.finish();
}

/// 向量存储性能基准测试
fn bench_vector_storage(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_storage");

    let storage = MemoryVectorStorage::new();

    // 向量添加性能测试
    let vector_sizes = vec![128, 256, 512, 1024];

    for size in vector_sizes {
        group.bench_with_input(BenchmarkId::new("add_vector", size), &size, |b, &size| {
            b.to_async(FuturesExecutor).iter(|| async {
                let vector: Vec<f32> = (0..size).map(|i| i as f32 / size as f32).collect();
                let result = storage.add(
                    format!("vec_{}", size),
                    vector,
                    json!({"size": size})
                ).await;
                black_box(result)
            })
        });
    }

    // 向量搜索性能测试
    group.bench_function("search_vectors", |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            let query_vector: Vec<f32> = (0..256).map(|i| i as f32 / 256.0).collect();
            let result = storage.search(&query_vector, 10).await;
            black_box(result)
        })
    });

    group.finish();
}

/// 工作流性能基准测试
fn bench_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow");

    group.bench_function("workflow_creation", |b| {
        b.iter(|| {
            let workflow = EnhancedWorkflow::new("test_workflow".to_string(), None);
            black_box(workflow)
        })
    });

    group.bench_function("workflow_execution", |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            let workflow = EnhancedWorkflow::new("test_workflow".to_string(), None);
            let context = RuntimeContext::new();
            let input = json!({"test": "data"});
            let result = workflow.execute(input, &context).await;
            black_box(result)
        })
    });

    group.finish();
}

/// 网络性能基准测试
fn bench_network(c: &mut Criterion) {
    let mut group = c.benchmark_group("network");

    group.bench_function("agent_node_creation", |b| {
        b.iter(|| {
            let config = AgentConfig::default();
            let node = AgentNode::new(Some(AgentId::new()), config);
            black_box(node)
        })
    });

    group.bench_function("agent_network_creation", |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            let network = AgentNetwork::new().await;
            black_box(network)
        })
    });

    // 并发节点创建性能测试
    let concurrent_counts = vec![1, 5, 10];

    for count in concurrent_counts {
        group.bench_with_input(BenchmarkId::new("concurrent_nodes", count), &count, |b, &count| {
            b.to_async(FuturesExecutor).iter(|| async {
                let tasks: Vec<_> = (0..count).map(|_| {
                    tokio::spawn(async {
                        let config = AgentConfig::default();
                        AgentNode::new(Some(AgentId::new()), config)
                    })
                }).collect();

                let results = futures::future::join_all(tasks).await;
                black_box(results)
            })
        });
    }

    group.finish();
}

/// 内存使用基准测试
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    group.bench_function("agent_memory_footprint", |b| {
        b.iter(|| {
            let agents: Vec<_> = (0..10).map(|_| {
                let llm = Box::new(MockLlmProvider::new()) as Box<dyn LLMProvider>;
                AgentBuilder::new()
                    .with_llm(llm)
                    .build()
                    .expect("创建Agent失败")
            }).collect();
            black_box(agents)
        })
    });

    group.bench_function("vector_storage_memory", |b| {
        b.iter(|| {
            let storage = MemoryVectorStorage::new();
            black_box(storage)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_agent_creation,
    bench_agent_generation,
    bench_document_chunking,
    bench_vector_storage,
    bench_workflow,
    bench_network,
    bench_memory_usage
);

criterion_main!(benches);
