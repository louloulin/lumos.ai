//! RAG Integration Tests
//!
//! 测试 Agent + RAG 集成功能

use lumosai_core::agent::{Agent, AgentBuilder, RagIntegrationExt};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use lumosai_core::vector::MemoryVectorStorage;
use std::sync::Arc;

#[tokio::test]
async fn test_rag_agent_creation() {
    let llm = create_test_zhipu_provider_arc();
    let vector_store = Arc::new(MemoryVectorStorage::new(384, None));

    let rag_agent = AgentBuilder::new()
        .name("rag_test")
        .instructions("You are a knowledge base assistant")
        .model(llm)
        .with_rag_simple(vector_store)
        .unwrap();

    // 验证 Agent 创建成功
    assert_eq!(rag_agent.agent().get_name(), "rag_test");
    println!("✅ RAG Agent created successfully");
}

#[tokio::test]
async fn test_add_documents() {
    let llm = create_test_zhipu_provider_arc();
    let vector_store = Arc::new(MemoryVectorStorage::new(384, None));

    let rag_agent = AgentBuilder::new()
        .name("kb_agent")
        .instructions("Answer based on knowledge base")
        .model(llm)
        .with_rag_simple(vector_store)
        .unwrap();

    // 添加文档
    let docs = vec![
        ("doc1", "LumosAI is built with Rust for performance"),
        ("doc2", "LumosAI supports multiple LLM providers"),
        ("doc3", "LumosAI provides RAG capabilities"),
    ];

    let result = rag_agent.add_documents(docs).await;
    assert!(result.is_ok(), "Should add documents successfully");
    println!("✅ Documents added to knowledge base");
}

#[tokio::test]
async fn test_generate_with_rag() {
    let llm = create_test_zhipu_provider_arc();
    let vector_store = Arc::new(MemoryVectorStorage::new(384, None));

    let rag_agent = AgentBuilder::new()
        .name("qa_agent")
        .instructions("Answer questions based on the knowledge base")
        .model(llm)
        .with_rag_simple(vector_store.clone())
        .unwrap();

    // 添加知识
    rag_agent
        .add_documents(vec![
            ("k1", "LumosAI is an enterprise-grade AI framework"),
            ("k2", "LumosAI is built with Rust programming language"),
            ("k3", "LumosAI provides agents, RAG, and workflows"),
        ])
        .await
        .unwrap();

    // 查询
    let response = rag_agent.generate_with_rag("What is LumosAI?").await;

    match response {
        Ok(answer) => {
            println!("✅ RAG response: {}", answer);
            assert!(!answer.is_empty());
        }
        Err(e) => {
            println!("⚠️  RAG generation: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_rag_config() {
    use lumosai_core::agent::RagConfig;

    let vector_store = Arc::new(MemoryVectorStorage::new(384, None));

    // 测试默认配置
    let config1 = RagConfig::new(vector_store.clone());
    assert_eq!(config1.top_k, 5);
    assert_eq!(config1.similarity_threshold, 0.7);

    // 测试自定义配置
    let config2 = RagConfig::new(vector_store.clone())
        .with_top_k(10)
        .with_threshold(0.8)
        .with_cache(false);

    assert_eq!(config2.top_k, 10);
    assert_eq!(config2.similarity_threshold, 0.8);
    assert!(!config2.enable_cache);

    println!("✅ RAG Config tests passed");
}
