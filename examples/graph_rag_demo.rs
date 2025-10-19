//! GraphRAG 演示程序 - 知识图谱检索增强生成
//!
//! 演示如何使用 GraphRAG 进行基于知识图谱的文档检索

use lumosai_rag::{
    retriever::{GraphRagConfig, GraphRagRetriever, Retriever},
    types::{Document, Metadata, RetrievalOptions, RetrievalRequest},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🕸️  GraphRAG 知识图谱检索演示\n");
    println!("{}", "=".repeat(80));
    println!();

    // 演示 1: 创建 GraphRAG 检索器
    demo_create_graph_rag().await?;

    // 演示 2: 添加文档并构建知识图谱
    demo_build_knowledge_graph().await?;

    // 演示 3: 图遍历检索
    demo_graph_traversal_retrieval().await?;

    // 总结
    println!("\n{}", "=".repeat(80));
    println!("✅ GraphRAG 演示完成！");
    println!("{}", "=".repeat(80));
    println!();
    println!("📊 功能总结:");
    println!("  1. 知识图谱构建: 从文档中提取实体和关系");
    println!("  2. 图遍历检索: 基于实体关系进行多跳检索");
    println!("  3. 上下文扩展: 通过图结构获取更丰富的上下文");
    println!();
    println!("💡 技术优势:");
    println!("  - 结构化知识: 将非结构化文本转换为知识图谱");
    println!("  - 关系推理: 利用实体间关系进行推理");
    println!("  - 多跳检索: 支持多层级的关系遍历");
    println!("  - 上下文丰富: 提供更全面的检索结果");
    println!();
    println!("🚀 对标 LlamaIndex:");
    println!("  ✅ 知识图谱构建");
    println!("  ✅ 实体关系提取");
    println!("  ✅ 图遍历检索");
    println!("  ✅ 社区检测（配置支持）");
    println!("{}", "=".repeat(80));

    Ok(())
}

/// 演示 1: 创建 GraphRAG 检索器
async fn demo_create_graph_rag() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 演示 1: 创建 GraphRAG 检索器");
    println!("{}", "-".repeat(80));

    // 创建配置
    let config = GraphRagConfig {
        max_traversal_depth: 2,
        enable_community_detection: false,
        entity_similarity_threshold: 0.7,
        max_results: 10,
    };

    println!("✅ 配置参数:");
    println!("  - 最大遍历深度: {}", config.max_traversal_depth);
    println!("  - 社区检测: {}", config.enable_community_detection);
    println!("  - 实体相似度阈值: {}", config.entity_similarity_threshold);
    println!("  - 最大结果数: {}", config.max_results);

    // 创建 GraphRAG 检索器
    let _retriever = GraphRagRetriever::new(config);
    println!("✅ 创建 GraphRAG 检索器成功");

    println!();
    Ok(())
}

/// 演示 2: 添加文档并构建知识图谱
async fn demo_build_knowledge_graph() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️  演示 2: 构建知识图谱");
    println!("{}", "-".repeat(80));

    let config = GraphRagConfig::default();
    let retriever = GraphRagRetriever::new(config);

    // 准备示例文档
    let documents = vec![
        Document {
            id: "doc1".to_string(),
            content: "LumosAI is a Rust-based AI framework developed by the Lumosai Team. It provides powerful tools for building AI agents.".to_string(),
            metadata: Metadata::default(),
            embedding: None,
        },
        Document {
            id: "doc2".to_string(),
            content: "Rust is a systems programming language that focuses on safety and performance. LumosAI leverages Rust for high-performance AI applications.".to_string(),
            metadata: Metadata::default(),
            embedding: None,
        },
        Document {
            id: "doc3".to_string(),
            content: "The Lumosai Team is dedicated to building enterprise-grade AI solutions. They focus on RAG systems and multi-agent collaboration.".to_string(),
            metadata: Metadata::default(),
            embedding: None,
        },
    ];

    println!("📄 添加文档到知识图谱:");
    for (idx, doc) in documents.iter().enumerate() {
        retriever.add_document(doc.clone()).await?;
        println!("  ✅ 文档 {}: {} 字符", idx + 1, doc.content.len());
    }

    println!("\n💡 知识图谱构建过程:");
    println!("  1. 实体提取: 识别文档中的关键实体（LumosAI, Rust, Lumosai Team）");
    println!("  2. 关系识别: 提取实体间的关系（developed_by, leverages, focuses_on）");
    println!("  3. 图谱构建: 将实体和关系组织成图结构");
    println!("  4. 索引优化: 建立邻接表和反向索引");

    println!();
    Ok(())
}

/// 演示 3: 图遍历检索
async fn demo_graph_traversal_retrieval() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 演示 3: 图遍历检索");
    println!("{}", "-".repeat(80));

    let config = GraphRagConfig {
        max_traversal_depth: 2,
        enable_community_detection: false,
        entity_similarity_threshold: 0.7,
        max_results: 5,
    };
    let retriever = GraphRagRetriever::new(config);

    // 添加文档
    let documents = vec![
        Document {
            id: "doc1".to_string(),
            content: "LumosAI is a Rust-based AI framework developed by the Lumosai Team. It provides powerful tools for building AI agents.".to_string(),
            metadata: Metadata::default(),
            embedding: None,
        },
        Document {
            id: "doc2".to_string(),
            content: "Rust is a systems programming language that focuses on safety and performance. LumosAI leverages Rust for high-performance AI applications.".to_string(),
            metadata: Metadata::default(),
            embedding: None,
        },
        Document {
            id: "doc3".to_string(),
            content: "The Lumosai Team is dedicated to building enterprise-grade AI solutions. They focus on RAG systems and multi-agent collaboration.".to_string(),
            metadata: Metadata::default(),
            embedding: None,
        },
        Document {
            id: "doc4".to_string(),
            content: "RAG systems combine retrieval and generation for enhanced AI responses. LumosAI implements advanced RAG capabilities including GraphRAG.".to_string(),
            metadata: Metadata::default(),
            embedding: None,
        },
    ];

    for doc in documents {
        retriever.add_document(doc).await?;
    }

    println!("📄 已添加 4 个文档到知识图谱");

    // 执行检索
    let queries = vec![
        "What is LumosAI?",
        "Tell me about Rust programming language",
        "What does the Lumosai Team do?",
    ];

    for (idx, query) in queries.iter().enumerate() {
        println!("\n🔎 查询 {}: {}", idx + 1, query);
        
        let request = RetrievalRequest {
            query: query.to_string(),
            options: RetrievalOptions {
                limit: Some(3),
                threshold: None,
                filter: None,
            },
        };

        let result = retriever.retrieve(&request).await?;
        
        println!("  ✅ 检索到 {} 个相关文档:", result.documents.len());
        for (i, scored_doc) in result.documents.iter().enumerate() {
            println!("    {}. [分数: {:.2}] {}", 
                i + 1, 
                scored_doc.score, 
                &scored_doc.document.content[..80.min(scored_doc.document.content.len())]
            );
        }
    }

    println!("\n💡 图遍历检索流程:");
    println!("  1. 实体识别: 从查询中提取关键实体");
    println!("  2. 图匹配: 在知识图谱中查找匹配的实体");
    println!("  3. 邻居扩展: 通过关系获取相关实体（多跳遍历）");
    println!("  4. 文档聚合: 收集所有相关实体关联的文档");
    println!("  5. 分数计算: 基于实体匹配度和关系强度计算分数");
    println!("  6. 结果排序: 按分数降序返回最相关的文档");

    println!();
    Ok(())
}

