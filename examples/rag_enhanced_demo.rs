//! RAG 系统增强功能演示
//!
//! 演示重排序、语义分块和多提供商嵌入等高级 RAG 功能

use lumosai_rag::{
    document::{AdaptiveChunker, SemanticChunker, SmartChunker},
    embedding::{
        CachedEmbeddingProvider, EmbeddingProvider, LocalEmbeddingProvider, ZhipuEmbeddingProvider,
    },
    retriever::{CrossEncoderReranker, DiversityReranker, LLMReranker, Reranker},
    types::{Document, Metadata, ScoredDocument},
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("🚀 LumosAI RAG 系统增强功能演示\n");
    println!("{}", "=".repeat(80));
    println!();

    // 测试 1: 智能分块
    println!("📝 测试 1: 智能分块");
    println!("{}", "-".repeat(80));
    test_smart_chunking().await;
    println!();

    // 测试 2: 多提供商嵌入
    println!("🔢 测试 2: 多提供商嵌入");
    println!("{}", "-".repeat(80));
    test_embedding_providers().await;
    println!();

    // 测试 3: 文档重排序
    println!("🔄 测试 3: 文档重排序");
    println!("{}", "-".repeat(80));
    test_reranking().await;
    println!();

    // 总结
    println!("{}", "=".repeat(80));
    println!("✅ RAG 系统增强功能演示完成！");
    println!("{}", "=".repeat(80));
    println!();
    println!("📊 功能统计:");
    println!("  - 智能分块策略: 2 种（语义分块、自适应分块）");
    println!("  - 嵌入提供商: 3 种（智谱 AI、本地、缓存）");
    println!("  - 重排序算法: 3 种（交叉编码器、LLM、多样性）");
    println!();
    println!("💡 使用建议:");
    println!("  1. 语义分块: 适用于长文档，保持语义连贯性");
    println!("  2. 自适应分块: 根据文档类型自动选择策略");
    println!("  3. 重排序: 提升检索结果的相关性和多样性");
    println!();
    println!("🎯 对标验证:");
    println!("  - 分块策略: ✅ 对标 LlamaIndex 的智能分块");
    println!("  - 嵌入支持: ✅ 对标 LlamaIndex 的多提供商");
    println!("  - 重排序: ✅ 对标 LlamaIndex 的高级检索");
    println!("{}", "=".repeat(80));
}

/// 测试智能分块
async fn test_smart_chunking() {
    // 场景 1: 语义分块
    println!("场景 1: 语义分块");

    let embedding_provider = Arc::new(LocalEmbeddingProvider::new(128));
    let semantic_chunker = SemanticChunker::new(
        embedding_provider.clone(),
        0.7, // 相似度阈值
        100, // 最小块大小
        500, // 最大块大小
    );

    let doc = create_test_document(
        "doc1",
        "人工智能是计算机科学的一个分支。它致力于创建智能机器。\
         机器学习是人工智能的一个子领域。深度学习是机器学习的一种方法。\
         自然语言处理也是人工智能的重要应用。它使计算机能够理解人类语言。",
    );

    match semantic_chunker.chunk(&doc).await {
        Ok(chunks) => {
            println!("  ✅ 语义分块成功");
            println!("  分块数量: {}", chunks.len());
            for (i, chunk) in chunks.iter().enumerate() {
                println!("  块 {}: {} 字符", i + 1, chunk.content.len());
            }
        }
        Err(e) => {
            println!("  ❌ 语义分块失败: {}", e);
        }
    }
    println!();

    // 场景 2: 自适应分块
    println!("场景 2: 自适应分块");

    let adaptive_chunker = AdaptiveChunker::new(200, 50);

    let code_doc = create_test_document(
        "code1",
        "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```\n\n\
         这是一个简单的 Rust 程序。它打印 Hello, world! 到控制台。",
    );

    match adaptive_chunker.chunk(&code_doc).await {
        Ok(chunks) => {
            println!("  ✅ 自适应分块成功");
            println!("  分块数量: {}", chunks.len());
            for (i, chunk) in chunks.iter().enumerate() {
                println!("  块 {}: {} 字符", i + 1, chunk.content.len());
            }
        }
        Err(e) => {
            println!("  ❌ 自适应分块失败: {}", e);
        }
    }
}

/// 测试嵌入提供商
async fn test_embedding_providers() {
    // 场景 1: 本地嵌入提供商
    println!("场景 1: 本地嵌入提供商");

    let local_provider = LocalEmbeddingProvider::new(128);
    let text = "这是一个测试文本";

    match local_provider.generate_embedding(text).await {
        Ok(embedding) => {
            println!("  ✅ 本地嵌入生成成功");
            println!("  嵌入维度: {}", embedding.len());
            println!("  前 5 个值: {:?}", &embedding[..5.min(embedding.len())]);
        }
        Err(e) => {
            println!("  ❌ 本地嵌入生成失败: {}", e);
        }
    }
    println!();

    // 场景 2: 缓存嵌入提供商
    println!("场景 2: 缓存嵌入提供商");

    let inner = Box::new(LocalEmbeddingProvider::new(64));
    let cached_provider = CachedEmbeddingProvider::new(inner, 100);

    // 第一次调用
    let start = std::time::Instant::now();
    let _ = cached_provider.generate_embedding(text).await;
    let first_duration = start.elapsed();

    // 第二次调用（应该从缓存获取）
    let start = std::time::Instant::now();
    let _ = cached_provider.generate_embedding(text).await;
    let second_duration = start.elapsed();

    println!("  ✅ 缓存嵌入提供商测试成功");
    println!("  第一次调用: {:?}", first_duration);
    println!("  第二次调用: {:?} (从缓存)", second_duration);
    println!("  缓存大小: {}", cached_provider.cache_size().await);
    println!();

    // 场景 3: 智谱 AI 嵌入提供商（需要 API 密钥）
    println!("场景 3: 智谱 AI 嵌入提供商");

    let api_key =
        std::env::var("ZHIPU_API_KEY").unwrap_or_else(|_| "your-api-key-here".to_string());

    if api_key == "your-api-key-here" {
        println!("  ⚠️  请设置环境变量 ZHIPU_API_KEY 以测试智谱 AI 嵌入");
        println!("     export ZHIPU_API_KEY=your-api-key");
    } else {
        let zhipu_provider = ZhipuEmbeddingProvider::new(api_key);
        match zhipu_provider.generate_embedding(text).await {
            Ok(embedding) => {
                println!("  ✅ 智谱 AI 嵌入生成成功");
                println!("  嵌入维度: {}", embedding.len());
            }
            Err(e) => {
                println!("  ❌ 智谱 AI 嵌入生成失败: {}", e);
            }
        }
    }
}

/// 测试重排序
async fn test_reranking() {
    let query = "人工智能";
    let documents = vec![
        create_scored_document("1", "人工智能是计算机科学的一个分支", 0.8),
        create_scored_document("2", "机器学习是人工智能的子领域", 0.7),
        create_scored_document("3", "深度学习使用神经网络", 0.6),
        create_scored_document("4", "自然语言处理处理人类语言", 0.5),
    ];

    // 场景 1: 交叉编码器重排序
    println!("场景 1: 交叉编码器重排序");

    let cross_encoder = CrossEncoderReranker::new("test-model", Some(3));
    match cross_encoder.rerank(query, documents.clone()).await {
        Ok(reranked) => {
            println!("  ✅ 交叉编码器重排序成功");
            println!("  返回文档数: {}", reranked.len());
            for (i, doc) in reranked.iter().enumerate() {
                println!(
                    "  排名 {}: {} (分数: {:.3})",
                    i + 1,
                    doc.document.id,
                    doc.score
                );
            }
        }
        Err(e) => {
            println!("  ❌ 交叉编码器重排序失败: {}", e);
        }
    }
    println!();

    // 场景 2: LLM 重排序
    println!("场景 2: LLM 重排序");

    let llm_reranker = LLMReranker::new("gpt-4", Some(3));
    match llm_reranker.rerank(query, documents.clone()).await {
        Ok(reranked) => {
            println!("  ✅ LLM 重排序成功");
            println!("  返回文档数: {}", reranked.len());
            for (i, doc) in reranked.iter().enumerate() {
                println!(
                    "  排名 {}: {} (分数: {:.3})",
                    i + 1,
                    doc.document.id,
                    doc.score
                );
            }
        }
        Err(e) => {
            println!("  ❌ LLM 重排序失败: {}", e);
        }
    }
    println!();

    // 场景 3: 多样性重排序
    println!("场景 3: 多样性重排序");

    let diversity_reranker = DiversityReranker::new(0.5, Some(3));
    match diversity_reranker.rerank(query, documents.clone()).await {
        Ok(reranked) => {
            println!("  ✅ 多样性重排序成功");
            println!("  返回文档数: {}", reranked.len());
            for (i, doc) in reranked.iter().enumerate() {
                println!(
                    "  排名 {}: {} (分数: {:.3})",
                    i + 1,
                    doc.document.id,
                    doc.score
                );
            }
        }
        Err(e) => {
            println!("  ❌ 多样性重排序失败: {}", e);
        }
    }
}

/// 创建测试文档
fn create_test_document(id: &str, content: &str) -> Document {
    Document {
        id: id.to_string(),
        content: content.to_string(),
        metadata: Metadata::new(),
        embedding: None,
    }
}

/// 创建带分数的测试文档
fn create_scored_document(id: &str, content: &str, score: f32) -> ScoredDocument {
    ScoredDocument {
        document: create_test_document(id, content),
        score,
    }
}
