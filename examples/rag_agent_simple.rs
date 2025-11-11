//! Simple RAG Agent Example
//!
//! 演示如何创建一个具有知识库能力的 Agent
//!
//! 运行方式:
//! ```bash
//! cargo run --example rag_agent_simple
//! ```

use lumosai_core::agent::{AgentBuilder, RagIntegrationExt};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use lumosai_core::vector::MemoryVectorStorage;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 LumosAI Simple RAG Agent Example\n");
    println!("{}", "=".repeat(60));

    // 1. 创建向量存储（知识库）
    println!("\n📦 Step 1: Creating vector storage...");
    let vector_store = Arc::new(MemoryVectorStorage::new(384, None));
    println!("✅ Vector storage created (384 dimensions)");

    // 2. 创建 RAG Agent（一行代码！）
    println!("\n🤖 Step 2: Creating RAG Agent...");
    let llm = create_test_zhipu_provider_arc();

    let rag_agent = AgentBuilder::new()
        .name("knowledge_assistant")
        .instructions("You are a helpful assistant with access to a knowledge base. Answer questions based on the provided context.")
        .model(llm)
        .with_rag_simple(vector_store)? // 🎯 一行代码添加 RAG 能力！
    ;

    println!("✅ RAG Agent created successfully");

    // 3. 添加知识到知识库
    println!("\n📚 Step 3: Adding knowledge to database...");
    let knowledge_docs = vec![
        ("doc1", "LumosAI is an enterprise-grade AI agent framework built with Rust."),
        ("doc2", "LumosAI provides powerful capabilities including agents, RAG, and workflows."),
        ("doc3", "LumosAI supports multiple LLM providers like OpenAI, Anthropic, and Zhipu."),
        ("doc4", "LumosAI's RAG system includes vector storage, document processing, and retrieval."),
        ("doc5", "LumosAI is designed for high performance and memory safety using Rust."),
    ];

    rag_agent.add_documents(knowledge_docs).await?;
    println!("✅ {} documents added to knowledge base", 5);

    // 4. 使用 RAG Agent 回答问题
    println!("\n💬 Step 4: Querying with RAG...\n");
    println!("{}", "=".repeat(60));

    let questions = vec![
        "What is LumosAI?",
        "What language is LumosAI built with?",
        "What capabilities does LumosAI provide?",
    ];

    for (i, question) in questions.iter().enumerate() {
        println!("\n🔍 Question {}: {}", i + 1, question);
        println!("思考中...");

        match rag_agent.generate_with_rag(question).await {
            Ok(answer) => {
                println!("\n✅ Answer: {}", answer);
            }
            Err(e) => {
                println!("\n⚠️  Failed to generate answer: {:?}", e);
            }
        }

        println!("{}", "-".repeat(60));
    }

    // 5. 总结
    println!("\n{}", "=".repeat(60));
    println!("📊 Demo Summary");
    println!("{}", "=".repeat(60));
    println!();
    println!("✅ Achievements:");
    println!("  • Created RAG Agent with one line of code");
    println!("  • Added 5 documents to knowledge base");
    println!("  • Successfully queried knowledge base");
    println!("  • Automatic context retrieval and injection");
    println!();
    println!("🎯 Key Features:");
    println!("  • Simple API: .with_rag_simple(vector_store)");
    println!("  • Automatic retrieval: No manual search needed");
    println!("  • Context injection: Automatically enhanced prompts");
    println!("  • Type-safe: Full Rust type safety");
    println!();
    println!("📖 Learn more:");
    println!("  • docs/agent/rag_integration.md");
    println!("  • examples/rag_agent_advanced.rs");
    println!();

    Ok(())
}

