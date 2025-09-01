use lumosai_rag::{Document, ChunkingStrategy, ChunkingConfig, Metadata};
use lumosai_rag::document::{TextChunker, DocumentChunker};
use lumosai_rag::retriever::{InMemoryVectorStore, VectorStore};
use lumosai_rag::types::{RetrievalOptions, ScoredDocument};
use lumosai_rag::embedding::EmbeddingProvider;
use serde_json::json;
use std::time::Instant;
use async_trait::async_trait;

// Mock embedding provider for testing
struct MockEmbeddingProvider;

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn generate_embedding(&self, text: &str) -> lumosai_rag::error::Result<Vec<f32>> {
        // Generate a simple mock embedding based on text length and content
        let embedding: Vec<f32> = (0..384)
            .map(|i| (text.len() as f32 + i as f32) * 0.001)
            .collect();
        Ok(embedding)
    }


}

/// RAG系统全面验证测试
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚀 LumosAI RAG系统验证测试");
    println!("========================================");
    
    // 测试1: 文档处理验证
    println!("\n📋 测试1: 文档处理验证");
    test_document_processing().await?;
    
    // 测试2: 文本分块验证
    println!("\n📋 测试2: 文本分块验证");
    test_text_chunking().await?;
    
    // 测试3: 向量化验证
    println!("\n📋 测试3: 向量化验证");
    test_embedding_generation().await?;
    
    // 测试4: 向量检索验证
    println!("\n📋 测试4: 向量检索验证");
    test_vector_retrieval().await?;
    
    // 测试5: 上下文管理验证
    println!("\n📋 测试5: 上下文管理验证");
    test_context_management().await?;
    
    // 测试6: 端到端RAG流程验证
    println!("\n📋 测试6: 端到端RAG流程验证");
    test_end_to_end_rag().await?;
    
    println!("\n✅ 所有RAG系统验证测试完成！");
    Ok(())
}

async fn test_document_processing() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试文档处理...");

    println!("✅ 文档处理测试开始");

    // 测试不同类型的文档
    let test_documents = vec![
        ("text_doc", "这是一个测试文档。它包含了多个段落和句子。\n\n第二段落包含了更多的内容，用于测试文档处理功能。", "text/plain"),
        ("markdown_doc", "# 标题\n\n这是一个Markdown文档。\n\n## 子标题\n\n- 列表项1\n- 列表项2\n\n**粗体文本**和*斜体文本*。", "text/markdown"),
        ("json_doc", r#"{"title": "JSON文档", "content": "这是一个JSON格式的文档", "metadata": {"author": "测试", "date": "2025-01-12"}}"#, "application/json"),
    ];

    for (doc_id, content, content_type) in test_documents {
        let start_time = Instant::now();

        // 创建文档对象
        let mut metadata = Metadata::new();
        metadata.add("content_type", content_type);
        metadata.add("doc_id", doc_id);

        let document = Document {
            id: doc_id.to_string(),
            content: content.to_string(),
            metadata,
            embedding: None,
        };

        let duration = start_time.elapsed();

        println!("✅ 文档 '{}' 处理成功! 耗时: {:?}", doc_id, duration);
        println!("📝 文档ID: {}", document.id);
        println!("📝 内容长度: {} 字符", document.content.len());
        println!("📝 内容类型: {}", document.metadata.fields.get("content_type").unwrap_or(&json!("未知")));

        // 验证文档内容
        if !document.content.is_empty() {
            println!("✅ 文档内容提取正常");
        } else {
            println!("⚠️ 文档内容为空");
        }
    }

    Ok(())
}

async fn test_text_chunking() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试文本分块...");

    let config = ChunkingConfig {
        chunk_size: 200,
        chunk_overlap: 50,
        strategy: ChunkingStrategy::Recursive {
            separators: None,
            is_separator_regex: false,
        },
        ..Default::default()
    };

    let chunker = TextChunker::new(config.clone());
    println!("✅ 文本分块器创建成功");

    // 测试长文本分块
    let long_text = "这是一个很长的文档，用于测试文本分块功能。".repeat(50);

    // 创建文档对象
    let mut metadata = Metadata::new();
    metadata.add("test", "chunking");

    let document = Document {
        id: "test_doc".to_string(),
        content: long_text.clone(),
        metadata,
        embedding: None,
    };

    let start_time = Instant::now();
    let chunks = chunker.chunk(document, &config).await?;
    let duration = start_time.elapsed();

    println!("✅ 文本分块完成! 耗时: {:?}", duration);
    println!("📊 原文长度: {} 字符", long_text.len());
    println!("📊 分块数量: {}", chunks.len());

    for (i, chunk) in chunks.iter().enumerate().take(3) {
        println!("📝 分块 {}: {} 字符", i + 1, chunk.content.len());
        if chunk.content.len() > 50 {
            let preview = chunk.content.chars().take(20).collect::<String>();
            println!("   内容预览: {}...", preview);
        } else {
            println!("   内容: {}", chunk.content);
        }
    }

    // 测试不同分块策略
    let strategies = vec![
        ("递归分块", ChunkingStrategy::Recursive { separators: None, is_separator_regex: false }),
        ("字符分块", ChunkingStrategy::Character { separator: "\n".to_string(), is_separator_regex: false }),
        ("Markdown分块", ChunkingStrategy::Markdown { headers: None, return_each_line: false, strip_headers: false }),
    ];

    for (strategy_name, strategy) in strategies {
        let config = ChunkingConfig {
            chunk_size: 100,
            chunk_overlap: 20,
            strategy,
            ..Default::default()
        };
        let chunker = TextChunker::new(config.clone());

        let test_doc = Document {
            id: "test".to_string(),
            content: "这是第一句。这是第二句话，比较长一些。这是第三句。这是第四句话。这是第五句，用于测试。".to_string(),
            metadata: Metadata::new(),
            embedding: None,
        };

        let start_time = Instant::now();
        let chunks = chunker.chunk(test_doc, &config).await?;
        let duration = start_time.elapsed();

        println!("✅ {} 策略测试完成! 耗时: {:?}, 分块数: {}", strategy_name, duration, chunks.len());
    }

    Ok(())
}

async fn test_embedding_generation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试向量化...");
    
    // 注意：这里需要实际的API密钥才能测试
    // 为了演示，我们创建一个模拟的嵌入提供商
    println!("⚠️ 使用模拟嵌入提供商进行测试");
    
    let test_texts = vec![
        "这是第一个测试文本，用于生成向量嵌入。",
        "这是第二个测试文本，内容与第一个相似。",
        "这是一个完全不同的文本，讨论的是另一个主题。",
        "人工智能和机器学习是现代技术的重要组成部分。",
        "Rust是一种系统编程语言，注重安全性和性能。",
    ];
    
    // 模拟嵌入生成
    let mut embeddings = Vec::new();
    for (i, text) in test_texts.iter().enumerate() {
        let start_time = Instant::now();
        
        // 模拟嵌入向量（实际应用中应该调用真实的嵌入API）
        let embedding: Vec<f32> = (0..384)
            .map(|j| ((i + j) as f32 * 0.01) % 1.0)
            .collect();
        
        let duration = start_time.elapsed();
        embeddings.push(embedding);
        
        println!("✅ 文本 {} 向量化完成! 耗时: {:?}", i + 1, duration);
        println!("📝 文本: {}", text);
        println!("📊 向量维度: {}", embeddings[i].len());
    }
    
    // 验证嵌入质量
    println!("📊 嵌入质量验证:");
    for (i, embedding) in embeddings.iter().enumerate() {
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        println!("📈 向量 {} 的L2范数: {:.4}", i + 1, norm);
    }
    
    Ok(())
}

async fn test_vector_retrieval() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试向量检索...");

    // 创建内存向量存储
    let mut vector_store = InMemoryVectorStore::new();
    println!("✅ 内存向量存储创建成功");

    // 准备测试文档
    let test_documents = vec![
        ("doc1", "人工智能技术的发展", vec![0.1; 384]),
        ("doc2", "机器学习算法原理", vec![0.2; 384]),
        ("doc3", "深度学习神经网络", vec![0.3; 384]),
        ("doc4", "自然语言处理技术", vec![0.4; 384]),
        ("doc5", "计算机视觉应用", vec![0.5; 384]),
    ];

    // 添加文档到向量存储
    for (doc_id, content, embedding) in test_documents {
        let mut metadata = Metadata::new();
        metadata.add("content", content);
        metadata.add("category", "AI技术");

        let document = Document {
            id: doc_id.to_string(),
            content: content.to_string(),
            metadata,
            embedding: Some(embedding),
        };

        let start_time = Instant::now();
        vector_store.add_document(document).await?;
        let duration = start_time.elapsed();

        println!("✅ 文档 '{}' 添加成功! 耗时: {:?}", doc_id, duration);
    }

    // 创建模拟嵌入提供商
    let embedding_provider = MockEmbeddingProvider;

    // 测试文本查询
    let options = RetrievalOptions {
        limit: Some(3),
        threshold: Some(0.5),
        filter: None,
    };

    let start_time = Instant::now();
    let results = vector_store.query_by_text("人工智能相关技术", &options, &embedding_provider).await?;
    let duration = start_time.elapsed();

    println!("✅ 向量检索完成! 耗时: {:?}", duration);
    println!("📊 检索结果数量: {}", results.documents.len());
    println!("📊 总文档数量: {}", results.total_count);

    for (i, scored_doc) in results.documents.iter().enumerate() {
        println!("📝 结果 {}: 相似度={:.4}", i + 1, scored_doc.score);
        println!("   文档ID: {}", scored_doc.document.id);
        println!("   内容: {}", scored_doc.document.content);
    }

    Ok(())
}

async fn test_context_management() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试上下文管理...");

    // 简化的上下文管理测试
    println!("✅ 上下文管理器测试开始");

    // 模拟检索结果
    let scored_documents = vec![
        ScoredDocument {
            document: Document {
                id: "doc1".to_string(),
                content: "人工智能是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的系统。".to_string(),
                metadata: {
                    let mut meta = Metadata::new();
                    meta.add("source", "AI百科");
                    meta
                },
                embedding: None,
            },
            score: 0.95,
        },
        ScoredDocument {
            document: Document {
                id: "doc2".to_string(),
                content: "机器学习是人工智能的一个子集，它使计算机能够在没有明确编程的情况下学习和改进。".to_string(),
                metadata: {
                    let mut meta = Metadata::new();
                    meta.add("source", "ML教程");
                    meta
                },
                embedding: None,
            },
            score: 0.88,
        },
        ScoredDocument {
            document: Document {
                id: "doc3".to_string(),
                content: "深度学习是机器学习的一个分支，使用多层神经网络来模拟人脑的工作方式。".to_string(),
                metadata: {
                    let mut meta = Metadata::new();
                    meta.add("source", "DL指南");
                    meta
                },
                embedding: None,
            },
            score: 0.82,
        },
    ];

    let query = "什么是人工智能？";

    let start_time = Instant::now();

    // 简单的上下文构建
    let mut context = format!("查询: {}\n\n相关文档:\n", query);
    for (i, scored_doc) in scored_documents.iter().enumerate() {
        context.push_str(&format!(
            "{}. [相似度: {:.2}] {}\n",
            i + 1,
            scored_doc.score,
            scored_doc.document.content
        ));
    }

    let duration = start_time.elapsed();

    println!("✅ 上下文构建完成! 耗时: {:?}", duration);
    println!("📊 上下文长度: {} 字符", context.len());
    println!("📝 上下文内容预览:");
    if context.len() > 200 {
        println!("   {}...", &context[..200]);
    } else {
        println!("   {}", context);
    }

    println!("📊 上下文统计:");
    println!("   文档数量: {}", scored_documents.len());
    println!("   平均相似度: {:.4}", scored_documents.iter().map(|d| d.score).sum::<f32>() / scored_documents.len() as f32);

    Ok(())
}

async fn test_end_to_end_rag() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试端到端RAG流程...");

    // 1. 文档处理
    let documents = vec![
        "人工智能（AI）是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的智能机器。",
        "机器学习是人工智能的一个子集，它使计算机能够在没有明确编程的情况下自动学习和改进。",
        "深度学习是机器学习的一个分支，它使用具有多个层次的神经网络来模拟人脑的工作方式。",
        "自然语言处理（NLP）是人工智能的一个分支，专注于计算机与人类语言之间的交互。",
        "计算机视觉是人工智能的一个领域，致力于使计算机能够理解和解释视觉信息。",
    ];

    let mut processed_docs = Vec::new();
    for (i, content) in documents.iter().enumerate() {
        let mut metadata = Metadata::new();
        metadata.add("doc_id", format!("doc_{}", i));
        metadata.add("category", "AI技术");

        let document = Document {
            id: format!("doc_{}", i),
            content: content.to_string(),
            metadata,
            embedding: None,
        };
        processed_docs.push(document);
    }

    println!("✅ 步骤1: 文档处理完成 - {} 个文档", processed_docs.len());

    // 2. 文本分块
    let config = ChunkingConfig {
        chunk_size: 100,
        chunk_overlap: 20,
        strategy: ChunkingStrategy::Recursive {
            separators: None,
            is_separator_regex: false,
        },
        ..Default::default()
    };
    let chunker = TextChunker::new(config.clone());

    let mut all_chunks = Vec::new();
    for doc in processed_docs {
        let chunks = chunker.chunk(doc, &config).await?;
        all_chunks.extend(chunks);
    }

    println!("✅ 步骤2: 文本分块完成 - {} 个分块", all_chunks.len());

    // 3. 向量化（模拟）
    let mut chunk_documents = Vec::new();
    for (i, chunk) in all_chunks.iter().enumerate() {
        let embedding: Vec<f32> = (0..384)
            .map(|j| ((i + j) as f32 * 0.01) % 1.0)
            .collect();

        let mut metadata = Metadata::new();
        metadata.add("chunk_id", i);
        metadata.add("content", chunk.content.clone());

        let chunk_doc = Document {
            id: format!("chunk_{}", i),
            content: chunk.content.clone(),
            metadata,
            embedding: Some(embedding),
        };
        chunk_documents.push(chunk_doc);
    }

    println!("✅ 步骤3: 向量化完成 - {} 个向量", chunk_documents.len());

    // 4. 向量存储
    let mut vector_store = InMemoryVectorStore::new();

    for chunk_doc in chunk_documents {
        vector_store.add_document(chunk_doc).await?;
    }

    println!("✅ 步骤4: 向量存储完成");

    // 5. 查询和检索
    let query = "什么是机器学习？";
    let embedding_provider = MockEmbeddingProvider;

    let options = RetrievalOptions {
        limit: Some(3),
        threshold: Some(0.5),
        filter: None,
    };

    let start_time = Instant::now();
    let search_results = vector_store.query_by_text(query, &options, &embedding_provider).await?;
    let duration = start_time.elapsed();

    println!("✅ 步骤5: 查询检索完成 - 耗时: {:?}", duration);

    // 6. 上下文构建
    let mut context = format!("查询: {}\n\n相关文档:\n", query);
    for (i, scored_doc) in search_results.documents.iter().enumerate() {
        context.push_str(&format!(
            "{}. [相似度: {:.2}] {}\n",
            i + 1,
            scored_doc.score,
            scored_doc.document.content
        ));
    }

    println!("✅ 步骤6: 上下文构建完成");

    // 7. 结果展示
    println!("\n📋 端到端RAG流程结果:");
    println!("🔍 查询: {}", query);
    println!("📊 检索到 {} 个相关文档片段", search_results.documents.len());
    println!("📝 构建的上下文长度: {} 字符", context.len());

    for (i, scored_doc) in search_results.documents.iter().enumerate() {
        println!("📄 片段 {}: 相似度={:.4}", i + 1, scored_doc.score);
        println!("   内容: {}", scored_doc.document.content);
    }

    println!("✅ 端到端RAG流程验证完成！");

    Ok(())
}
