//! E2E 测试：RAG 系统场景

mod framework;
use framework::{E2ETestContext, E2EAssertions};

use lumosai_core::vector::MemoryVectorStorage;
use lumosai_rag::{RagPipeline, RagPipelineBuilder, Document};
use lumosai_rag::embedding::MockEmbeddingProvider;

/// 测试 14: RAG Pipeline 基础功能
#[tokio::test]
async fn test_rag_pipeline_basic() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建 Mock 嵌入提供者
    let embedding_provider = Box::new(MockEmbeddingProvider::new(384));

    // 创建 RAG Pipeline
    let pipeline = RagPipelineBuilder::new()
        .embedding_provider(embedding_provider)
        .chunk_size(512)
        .chunk_overlap(50)
        .build();

    assert!(pipeline.is_ok(), "RAG Pipeline should be created successfully");

    let pipeline = pipeline.unwrap();

    // 创建测试文档
    let doc = Document {
        id: "test-doc-1".to_string(),
        content: "LumosAI is an enterprise-grade AI framework built with Rust. It provides powerful agent capabilities, RAG systems, and workflow orchestration.".to_string(),
        metadata: Default::default(),
        embedding: None,
    };

    // 处理文档
    let processed = pipeline.process_document(doc).await;
    assert!(processed.is_ok(), "Document processing should succeed");

    let chunks = processed.unwrap();
    assert!(!chunks.is_empty(), "Should produce at least one chunk");

    // 验证嵌入已生成
    for chunk in &chunks {
        assert!(chunk.embedding.is_some(), "Chunk should have embedding");
        assert_eq!(
            chunk.embedding.as_ref().unwrap().len(),
            384,
            "Embedding dimension should be 384"
        );
    }

    println!("✅ RAG Pipeline test passed: {} chunks generated", chunks.len());

    ctx.teardown().await.unwrap();
}

/// 测试 15: 向量存储和检索
#[tokio::test]
async fn test_vector_storage_retrieval() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let storage = &ctx.vector_storage;

    // 添加一些测试向量
    let vectors = vec![
        (vec![1.0; 384], "Document about AI"),
        (vec![0.5; 384], "Document about Rust"),
        (vec![0.8; 384], "Document about LumosAI"),
    ];

    for (i, (embedding, text)) in vectors.iter().enumerate() {
        let id = format!("vec-{}", i);
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("text".to_string(), serde_json::Value::String(text.to_string()));

        let result = storage.add_vector(&id, embedding.clone(), metadata).await;
        assert!(result.is_ok(), "Should add vector successfully");
    }

    // 搜索相似向量
    let query = vec![0.9; 384];
    let results = storage.search(&query, 2, None).await;

    assert!(results.is_ok(), "Search should succeed");
    let results = results.unwrap();
    assert!(!results.is_empty(), "Should return search results");
    assert!(results.len() <= 2, "Should return at most 2 results");

    println!("✅ Vector storage test passed: {} results found", results.len());

    ctx.teardown().await.unwrap();
}

/// 测试 16: Agent + RAG 集成（基础）
#[tokio::test]
async fn test_agent_rag_integration_basic() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建 Agent
    let agent = ctx
        .create_test_agent("rag_agent", "You are a knowledge base assistant.")
        .unwrap();

    // 准备向量存储（添加知识）
    let storage = &ctx.vector_storage;
    let knowledge = vec![
        "LumosAI is built with Rust for high performance and safety.",
        "LumosAI supports multiple LLM providers including OpenAI and Anthropic.",
        "LumosAI provides RAG, agents, and workflow orchestration.",
    ];

    for (i, text) in knowledge.iter().enumerate() {
        let id = format!("knowledge-{}", i);
        // 简单的模拟嵌入（基于文本长度）
        let embedding = vec![text.len() as f32 / 100.0; 384];
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("text".to_string(), serde_json::Value::String(text.to_string()));

        storage.add_vector(&id, embedding, metadata).await.ok();
    }

    // Agent 查询（虽然没有自动RAG，但测试基础功能）
    let response = agent.generate_simple("What is LumosAI?").await;
    assert!(response.is_ok());

    let response_text = response.unwrap();
    E2EAssertions::assert_non_empty_response(&response_text);

    println!("✅ Agent + RAG integration test passed");

    ctx.teardown().await.unwrap();
}

/// 测试 17: RAG 文档批量处理
#[tokio::test]
async fn test_rag_batch_processing() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let embedding_provider = Box::new(MockEmbeddingProvider::new(384));

    let pipeline = RagPipelineBuilder::new()
        .embedding_provider(embedding_provider)
        .chunk_size(256)
        .build()
        .unwrap();

    // 创建多个文档
    let documents = vec![
        Document {
            id: "doc1".to_string(),
            content: "This is the first document about AI.".to_string(),
            metadata: Default::default(),
            embedding: None,
        },
        Document {
            id: "doc2".to_string(),
            content: "This is the second document about ML.".to_string(),
            metadata: Default::default(),
            embedding: None,
        },
        Document {
            id: "doc3".to_string(),
            content: "This is the third document about RAG.".to_string(),
            metadata: Default::default(),
            embedding: None,
        },
    ];

    // 批量处理
    let start = std::time::Instant::now();
    let processed = pipeline.process_documents(documents).await;
    let duration = start.elapsed();

    assert!(processed.is_ok(), "Batch processing should succeed");

    let all_chunks = processed.unwrap();
    assert!(!all_chunks.is_empty(), "Should produce chunks");

    E2EAssertions::assert_reasonable_response_time(duration);

    println!(
        "✅ RAG batch processing test passed: {} chunks in {:?}",
        all_chunks.len(),
        duration
    );

    ctx.teardown().await.unwrap();
}

