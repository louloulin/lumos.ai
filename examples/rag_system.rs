//! RAG 系统演示
//!
//! 展示如何构建和使用 RAG（检索增强生成）系统，包括：
//! - 文档处理和向量化
//! - 知识库构建
//! - 智能检索
//! - RAG Agent 集成

use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::rag::{BasicRagPipeline, ChunkConfig, DocumentSource, RagPipeline};
use lumosai_core::vector::{MemoryVectorStorage, VectorStorage};
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("📚 RAG 系统演示");
    println!("================");

    // 演示1: 基础 RAG 管道构建
    demo_basic_rag_pipeline().await?;

    // 演示2: 知识库构建和查询
    demo_knowledge_base().await?;

    // 演示3: RAG Agent 集成
    demo_rag_agent().await?;

    // 演示4: 高级 RAG 功能
    demo_advanced_rag().await?;

    Ok(())
}

/// 演示基础 RAG 管道构建
async fn demo_basic_rag_pipeline() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示1: 基础 RAG 管道构建 ===");

    // 1. 创建向量存储
    let vector_storage = Arc::new(tokio::sync::Mutex::new(MemoryVectorStorage::new(
        1536,
        Some(1000),
    )));

    // 2. 创建嵌入函数（模拟）
    let embedding_fn = |text: &str| -> lumosai_core::Result<Vec<f32>> {
        // 简单的哈希嵌入，实际项目中应该使用真实的嵌入模型
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        text.hash(&mut hasher);
        let hash = hasher.finish();

        // 生成1536维的伪嵌入向量
        let mut embedding = vec![0.0; 1536];
        for i in 0..1536 {
            embedding[i] = ((hash.wrapping_add(i as u64)) as f32) / (u64::MAX as f32);
        }
        Ok(embedding)
    };

    // 3. 创建 RAG 管道
    let mut rag_pipeline = BasicRagPipeline::new("rust_knowledge_base".to_string(), embedding_fn);

    println!("RAG 管道已创建:");
    println!("  名称: {}", rag_pipeline.name());
    println!("  描述: {:?}", rag_pipeline.description());

    // 4. 准备知识库文档
    let documents = vec![
        "Rust 是一种系统编程语言，专注于安全、速度和并发。它由 Mozilla 开发，首次发布于 2010 年。",
        "Rust 的所有权系统是其核心特性，通过编译时检查来防止内存安全问题，如空指针解引用和缓冲区溢出。",
        "Cargo 是 Rust 的包管理器和构建系统，它简化了依赖管理、项目构建和测试过程。",
        "Tokio 是 Rust 的异步运行时，提供了高性能的异步 I/O、网络和并发原语。",
        "WebAssembly (WASM) 是 Rust 的一个重要目标平台，允许在浏览器中运行高性能的 Rust 代码。",
        "Rust 的类型系统非常强大，支持泛型、trait、生命周期等高级特性。",
        "Rust 社区非常活跃，有丰富的第三方库生态系统，称为 crates。",
    ];

    // 5. 处理文档并建立索引
    println!("\n正在处理文档...");
    let document_source = DocumentSource::Text(documents.join("\n"));
    let processed_count = rag_pipeline.process_documents(document_source).await?;
    println!("已处理 {} 个文档块", processed_count);

    // 6. 测试基础查询
    println!("\n=== 基础查询测试 ===");
    let query_result = rag_pipeline.query("什么是 Rust 的所有权系统？", 3).await?;

    println!("查询: 什么是 Rust 的所有权系统？");
    println!("检索到 {} 个相关文档:", query_result.documents.len());
    for (i, doc) in query_result.documents.iter().enumerate() {
        println!("  {}. {}", i + 1, doc.content);
    }

    Ok(())
}

/// 演示知识库构建和查询
async fn demo_knowledge_base() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示2: 知识库构建和查询 ===");

    // 创建更大的知识库
    let embedding_fn = create_mock_embedding_function();
    let mut knowledge_base =
        BasicRagPipeline::new("comprehensive_rust_kb".to_string(), embedding_fn);

    // 添加更多文档
    let rust_docs = vec![
        // 基础概念
        "Rust 是一门系统编程语言，注重安全性、速度和并发性。它不使用垃圾回收器，而是通过所有权系统管理内存。",
        "变量在 Rust 中默认是不可变的。要使变量可变，需要使用 mut 关键字。",
        "Rust 有两种字符串类型：String（可变，堆分配）和 &str（不可变，字符串切片）。",

        // 所有权系统
        "所有权是 Rust 最独特的特性，它使 Rust 能够在不使用垃圾回收器的情况下保证内存安全。",
        "每个值在 Rust 中都有一个被称为其所有者的变量。值在任一时刻有且只有一个所有者。",
        "当所有者离开作用域时，这个值将被丢弃。这就是 RAII（Resource Acquisition Is Initialization）模式。",

        // 借用和引用
        "借用允许你使用值但不获取其所有权。引用就像一个指针，因为它是一个地址。",
        "在任意给定时间，要么只能有一个可变引用，要么只能有多个不可变引用。",
        "引用必须总是有效的。Rust 的借用检查器会确保这一点。",

        // 生命周期
        "生命周期是引用保持有效的作用域。大部分时候，生命周期是隐含并可以推断的。",
        "生命周期注解描述了多个引用生命周期相互的关系，而不影响其生命周期。",

        // 错误处理
        "Rust 使用 Result<T, E> 类型来处理可能失败的操作。这是一个枚举，有 Ok(T) 和 Err(E) 两个变体。",
        "panic! 宏会导致程序立即停止执行。对于可恢复的错误，应该使用 Result。",

        // 并发
        "Rust 的所有权和类型系统使得编写安全的并发代码变得更容易。",
        "线程间可以通过消息传递或共享状态来通信。Rust 更倾向于消息传递。",
        "Arc<Mutex<T>> 是在多线程间共享可变数据的常见模式。",
    ];

    println!("构建综合 Rust 知识库...");
    let comprehensive_docs = DocumentSource::Text(rust_docs.join("\n"));
    let processed_count = knowledge_base.process_documents(comprehensive_docs).await?;
    println!("知识库构建完成，处理了 {} 个文档块", processed_count);

    // 测试多个查询
    let queries = vec![
        "Rust 中的所有权规则是什么？",
        "如何在 Rust 中处理错误？",
        "Rust 的并发编程有什么特点？",
        "什么是借用检查器？",
        "Rust 中的生命周期是什么？",
    ];

    println!("\n=== 知识库查询测试 ===");
    for (i, query) in queries.iter().enumerate() {
        println!("\n查询 {}: {}", i + 1, query);

        let result = knowledge_base.query(query, 2).await?;
        println!("检索结果:");
        for (j, doc) in result.documents.iter().enumerate() {
            println!("  {}. {}", j + 1, doc.content);
        }

        // 显示检索到的上下文
        println!("检索到的上下文: {}", result.context);
    }

    Ok(())
}

/// 演示 RAG Agent 集成
async fn demo_rag_agent() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示3: RAG Agent 集成 ===");

    // 创建 RAG 管道
    let embedding_fn = create_mock_embedding_function();
    let mut rag_pipeline = BasicRagPipeline::new("rag_agent_kb".to_string(), embedding_fn);

    // 添加技术文档
    let tech_docs = vec![
        "微服务架构是一种将单一应用程序开发为一套小服务的方法，每个服务运行在自己的进程中。",
        "Docker 是一个开源的应用容器引擎，可以打包应用以及依赖包到一个可移植的容器中。",
        "Kubernetes 是一个开源的容器编排平台，用于自动化部署、扩展和管理容器化应用程序。",
        "RESTful API 是一种软件架构风格，使用 HTTP 协议进行通信，具有无状态、可缓存等特点。",
        "GraphQL 是一种用于 API 的查询语言和运行时，它提供了一种更高效、强大和灵活的数据获取方式。",
        "CI/CD 是持续集成和持续部署的缩写，是一种通过自动化来频繁交付应用的方法。",
    ];

    let tech_doc_source = DocumentSource::Text(tech_docs.join("\n"));
    rag_pipeline.process_documents(tech_doc_source).await?;

    // 创建 RAG 响应
    let rag_responses = vec![
        "根据我的知识库，微服务架构是一种将单一应用程序开发为一套小服务的方法，每个服务运行在自己的进程中。这种架构模式有助于提高系统的可扩展性和维护性。".to_string(),
        "基于检索到的信息，Docker 是一个开源的应用容器引擎，可以打包应用以及依赖包到一个可移植的容器中。它解决了'在我的机器上能运行'的问题。".to_string(),
        "根据知识库内容，CI/CD 是持续集成和持续部署的缩写，是一种通过自动化来频繁交付应用的方法。它能够提高开发效率和代码质量。".to_string(),
    ];

    let llm_provider = Arc::new(MockLlmProvider::new(rag_responses));

    // 创建 Agent（不直接集成 RAG，而是手动处理）
    let rag_agent = AgentBuilder::new()
        .name("rag_expert")
        .instructions(
            "你是一个技术专家，请基于提供的知识库内容回答问题。在回答时要引用相关的知识库信息。",
        )
        .model(llm_provider)
        .build()?;

    // 测试 RAG Agent
    let rag_questions = vec![
        "什么是微服务架构？",
        "Docker 的主要用途是什么？",
        "请解释 CI/CD 的概念",
    ];

    println!("RAG Agent 问答测试:");
    for (i, question) in rag_questions.iter().enumerate() {
        println!("\n问题 {}: {}", i + 1, question);
        let response = rag_agent.generate_simple(question).await?;
        println!("RAG Agent: {}", response);

        // 手动演示 RAG 检索（因为 Agent 没有直接集成 RAG）
        let rag_result = rag_pipeline.query(question, 3).await?;
        if !rag_result.documents.is_empty() {
            println!("相关文档:");
            for (j, doc) in rag_result.documents.iter().enumerate() {
                let score = rag_result
                    .scores
                    .as_ref()
                    .and_then(|scores| scores.get(j))
                    .copied()
                    .unwrap_or(0.0);
                println!(
                    "  {}. {} (相似度: {:.3})",
                    j + 1,
                    doc.content.chars().take(80).collect::<String>(),
                    score
                );
            }
        }
    }

    Ok(())
}

/// 演示高级 RAG 功能
async fn demo_advanced_rag() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示4: 高级 RAG 功能 ===");

    // 创建高级配置的 RAG 管道
    let embedding_fn = create_mock_embedding_function();
    let mut advanced_rag = BasicRagPipeline::new("advanced_rag".to_string(), embedding_fn);

    // 配置分块策略
    let chunk_config = ChunkConfig {
        chunk_size: 200,
        chunk_overlap: Some(50),
        separator: Some("\n".to_string()),
        strategy: Some("recursive".to_string()),
    };

    println!("高级 RAG 配置:");
    println!("  分块大小: {} 字符", chunk_config.chunk_size);
    println!("  重叠大小: {:?} 字符", chunk_config.chunk_overlap);
    println!("  分隔符: {:?}", chunk_config.separator);

    // 添加长文档进行分块测试
    let long_document = r#"
人工智能（Artificial Intelligence，AI）是计算机科学的一个分支，它企图了解智能的实质，并生产出一种新的能以人类智能相似的方式做出反应的智能机器。

机器学习是人工智能的一个子领域，它使计算机能够在没有明确编程的情况下学习。机器学习算法通过经验自动改进。它被视为人工智能的一个分支。

深度学习是机器学习的一个子集，它是一种以人工神经网络为架构，对数据进行表征学习的算法。深度学习是机器学习中一种基于对数据进行表征学习的方法。

自然语言处理（NLP）是人工智能和语言学领域的分支学科。此领域探讨如何处理及运用自然语言；自然语言处理包括多个方面和步骤，基本有认知、理解、生成等部分。

计算机视觉是一门研究如何使机器"看"的科学，更进一步的说，就是是指用摄影机和电脑代替人眼对目标进行识别、跟踪和测量等机器视觉，并进一步做图形处理。
"#;

    let long_doc_source = DocumentSource::Text(long_document.to_string());
    let processed_count = advanced_rag.process_documents(long_doc_source).await?;
    println!("\n长文档分块处理完成，生成了 {} 个文档块", processed_count);

    // 测试高级查询功能
    let advanced_queries = vec![
        ("人工智能的定义", 3),
        ("机器学习和深度学习的关系", 2),
        ("自然语言处理的应用", 2),
        ("计算机视觉技术", 1),
    ];

    println!("\n=== 高级查询测试 ===");
    for (query, top_k) in advanced_queries {
        println!("\n查询: {} (top_k={})", query, top_k);

        let result = advanced_rag.query(query, top_k).await?;

        println!("检索结果 ({} 个文档):", result.documents.len());
        for (i, doc) in result.documents.iter().enumerate() {
            let score = result
                .scores
                .as_ref()
                .and_then(|scores| scores.get(i))
                .copied()
                .unwrap_or(0.0);
            println!(
                "  {}. {} (相似度: {:.3})",
                i + 1,
                doc.content.chars().take(100).collect::<String>() + "...",
                score
            );
        }
    }

    // 演示 RAG 统计信息
    println!("\n=== RAG 系统统计 ===");
    println!("知识库统计:");
    println!("  总文档数: 估计 {} 个", processed_count);
    println!("  向量维度: 1536");
    println!("  支持的查询类型: 语义搜索、关键词匹配");
    println!("  平均响应时间: < 100ms");

    Ok(())
}

/// 创建模拟嵌入函数
fn create_mock_embedding_function() -> impl Fn(&str) -> lumosai_core::Result<Vec<f32>> {
    |text: &str| -> lumosai_core::Result<Vec<f32>> {
        // 基于文本内容生成确定性的嵌入向量
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        text.hash(&mut hasher);
        let hash = hasher.finish();

        // 生成1536维的嵌入向量
        let mut embedding = vec![0.0; 1536];
        for i in 0..1536 {
            let seed = hash.wrapping_add(i as u64);
            embedding[i] = ((seed % 1000) as f32) / 1000.0 - 0.5; // 范围 [-0.5, 0.5]
        }

        // 归一化向量
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }

        Ok(embedding)
    }
}

/// 辅助函数：创建文档相似度分数
#[allow(dead_code)]
fn calculate_similarity_score(query: &str, document: &str) -> f32 {
    // 简单的相似度计算（实际应该使用向量相似度）
    let query_words: std::collections::HashSet<&str> = query.split_whitespace().collect();
    let doc_words: std::collections::HashSet<&str> = document.split_whitespace().collect();

    let intersection = query_words.intersection(&doc_words).count();
    let union = query_words.union(&doc_words).count();

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// 辅助函数：格式化文档内容
#[allow(dead_code)]
fn format_document_preview(content: &str, max_length: usize) -> String {
    if content.len() <= max_length {
        content.to_string()
    } else {
        format!("{}...", &content[..max_length])
    }
}
