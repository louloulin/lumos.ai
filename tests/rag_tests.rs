use std::sync::Arc;
use std::time::{Duration, Instant};
use lumosai_core::vector::{MemoryVectorStorage, VectorStorage};
use lumosai_rag::document::chunker::TextChunker;
use lumosai_rag::types::ChunkingConfig;

mod common;
use common::{TestAssertions, TestDataSets};

/// RAG文档处理测试
#[tokio::test]
async fn test_document_chunking_strategies() {
    // 测试不同分块策略
    let config = ChunkingConfig::default();
    let chunker = TextChunker::new(config);
    
    // 测试固定大小分块
    let test_text = "This is a long text that needs to be chunked into smaller pieces for processing.";
    let chunks = chunker.chunk_text(test_text).unwrap();
    assert!(!chunks.is_empty());
    
    // 验证分块结果
    for chunk in &chunks {
        assert!(!chunk.is_empty());
        assert!(chunk.len() <= 1000); // 默认最大块大小
    }
}

#[tokio::test]
async fn test_document_metadata_extraction() {
    // 测试元数据提取
    let chunker = TextChunker::new(ChunkingConfig::default());
    
    let text_with_metadata = "# Title\n\nThis is content under the title.\n\n## Subtitle\n\nMore content here.";
    let chunks = chunker.chunk_text(text_with_metadata).unwrap();
    
    assert!(!chunks.is_empty());
    // 验证分块包含了原始内容
    let combined = chunks.join(" ");
    assert!(combined.contains("Title"));
    assert!(combined.contains("content"));
}

#[tokio::test]
async fn test_document_format_support() {
    // 测试不同文档格式支持
    let chunker = TextChunker::new(ChunkingConfig::default());
    
    // 测试Markdown格式
    let markdown_text = "# Header\n\n**Bold text** and *italic text*.\n\n- List item 1\n- List item 2";
    let chunks = chunker.chunk_text(markdown_text).unwrap();
    assert!(!chunks.is_empty());
    
    // 测试HTML格式（如果支持）
    let html_text = "<h1>Header</h1><p>This is a paragraph with <strong>bold</strong> text.</p>";
    let html_chunks = chunker.chunk_text(html_text).unwrap();
    assert!(!html_chunks.is_empty());
}

#[tokio::test]
async fn test_document_preprocessing() {
    // 测试文档预处理
    let chunker = TextChunker::new(ChunkingConfig::default());
    
    // 测试包含特殊字符的文本
    let special_text = "Text with\ttabs\nand\r\nnewlines\u{00A0}and unicode spaces.";
    let chunks = chunker.chunk_text(special_text).unwrap();
    assert!(!chunks.is_empty());
    
    // 验证预处理后的文本
    for chunk in &chunks {
        assert!(!chunk.trim().is_empty());
    }
}

#[tokio::test]
async fn test_large_document_handling() {
    // 测试大文档处理
    let chunker = TextChunker::new(ChunkingConfig::default());
    
    let large_text = "Large document content. ".repeat(1000);
    let chunks = chunker.chunk_text(&large_text).unwrap();
    
    assert!(!chunks.is_empty());
    assert!(chunks.len() > 1); // 大文档应该被分成多个块
    
    // 验证所有块的总长度不超过原文档
    let total_length: usize = chunks.iter().map(|c| c.len()).sum();
    assert!(total_length <= large_text.len() * 2); // 允许一些重叠
}

/// RAG向量存储测试
#[tokio::test]
async fn test_vector_storage_crud() {
    // 测试向量存储的CRUD操作
    let storage = MemoryVectorStorage::new(384, None);
    
    // 创建索引
    storage.create_index("test", 384, Some(lumosai_core::vector::SimilarityMetric::Cosine)).await.unwrap();
    
    // 插入向量
    let vectors = vec![vec![0.1; 384]];
    let ids = storage.upsert("test", vectors, None, None).await.unwrap();
    assert_eq!(ids.len(), 1);
    
    // 查询向量
    let results = storage.query("test", vec![0.1; 384], 5, None, true).await.unwrap();
    assert!(!results.is_empty());
    
    // 验证查询结果
    assert_eq!(results.len(), 1);
    assert!(results[0].score >= 0.0);
}

#[tokio::test]
async fn test_vector_similarity_metrics() {
    // 测试不同相似度计算方法
    let storage = MemoryVectorStorage::new(384, None);
    
    // 测试余弦相似度
    storage.create_index("cosine_test", 384, Some(lumosai_core::vector::SimilarityMetric::Cosine)).await.unwrap();
    
    // 创建384维向量
    let mut vector1 = vec![0.0; 384];
    vector1[0] = 1.0;

    let mut vector2 = vec![0.0; 384];
    vector2[1] = 1.0;

    let mut vector3 = vec![0.0; 384];
    vector3[0] = 1.0; // 与vector1相同
    
    // 插入向量
    let vectors = vec![vector1.clone(), vector2.clone()];
    let _ids = storage.upsert("cosine_test", vectors, None, None).await.unwrap();
    
    // 查询相似向量
    let results = storage.query("cosine_test", vector3, 2, None, true).await.unwrap();
    assert_eq!(results.len(), 2);
    
    // 验证相似度排序（相同向量应该排在前面）
    assert!(results[0].score >= results[1].score);
}

#[tokio::test]
async fn test_vector_storage_performance() {
    // 测试向量存储性能
    let storage = MemoryVectorStorage::new(384, None);
    storage.create_index("perf_test", 384, Some(lumosai_core::vector::SimilarityMetric::Cosine)).await.unwrap();
    
    // 批量插入向量
    let batch_size = 100;
    let vectors: Vec<Vec<f32>> = (0..batch_size)
        .map(|i| vec![i as f32 / batch_size as f32; 384])
        .collect();
    
    let start_time = Instant::now();
    let _ids = storage.upsert("perf_test", vectors, None, None).await.unwrap();
    let insert_duration = start_time.elapsed();
    
    // 验证插入性能
    TestAssertions::assert_response_time(insert_duration, Duration::from_secs(5));
    
    // 测试查询性能
    let query_vector = vec![0.5; 384];
    let start_time = Instant::now();
    let results = storage.query("perf_test", query_vector, 10, None, true).await.unwrap();
    let query_duration = start_time.elapsed();
    
    assert_eq!(results.len(), 10);
    TestAssertions::assert_response_time(query_duration, Duration::from_millis(100));
}

#[tokio::test]
async fn test_vector_storage_concurrency() {
    // 测试并发访问
    let storage = Arc::new(MemoryVectorStorage::new(384, None));
    storage.create_index("concurrent_test", 384, Some(lumosai_core::vector::SimilarityMetric::Cosine)).await.unwrap();
    
    // 并发插入
    let tasks: Vec<_> = (0..5).map(|i| {
        let storage_clone = storage.clone();
        tokio::spawn(async move {
            let vectors = vec![vec![i as f32; 384]];
            storage_clone.upsert("concurrent_test", vectors, None, None).await
        })
    }).collect();
    
    let results = futures::future::join_all(tasks).await;
    
    // 验证所有并发操作都成功
    for result in results {
        let ids = result.unwrap().unwrap();
        assert_eq!(ids.len(), 1);
    }
    
    // 验证数据完整性
    let query_results = storage.query("concurrent_test", vec![0.0; 384], 10, None, true).await.unwrap();
    assert_eq!(query_results.len(), 5);
}

/// RAG检索测试
#[tokio::test]
async fn test_rag_query_accuracy() {
    // 测试检索准确性
    let storage = MemoryVectorStorage::new(384, None);
    storage.create_index("accuracy_test", 384, Some(lumosai_core::vector::SimilarityMetric::Cosine)).await.unwrap();
    
    // 模拟文档嵌入 (384维)
    let doc_embeddings = vec![
        {
            let mut vec = vec![0.0; 384];
            vec[0] = 1.0; // 文档1: "Python programming"
            vec
        },
        {
            let mut vec = vec![0.0; 384];
            vec[1] = 1.0; // 文档2: "Machine learning"
            vec
        },
        {
            let mut vec = vec![0.0; 384];
            vec[2] = 1.0; // 文档3: "Data science"
            vec
        },
    ];
    
    let _ids = storage.upsert("accuracy_test", doc_embeddings, None, None).await.unwrap();
    
    // 查询相关文档 (384维)
    let mut query_embedding = vec![0.0; 384];
    query_embedding[0] = 0.9;
    query_embedding[1] = 0.1; // 更接近文档1
    let results = storage.query("accuracy_test", query_embedding, 3, None, true).await.unwrap();
    
    assert_eq!(results.len(), 3);
    // 验证最相关的文档排在前面
    assert!(results[0].score >= results[1].score);
    assert!(results[1].score >= results[2].score);
}

#[tokio::test]
async fn test_rag_ranking_quality() {
    // 测试结果排序质量
    let storage = MemoryVectorStorage::new(3, None);
    storage.create_index("ranking_test", 3, Some(lumosai_core::vector::SimilarityMetric::Cosine)).await.unwrap();
    
    // 插入不同相似度的向量
    let vectors = vec![
        vec![1.0, 0.0, 0.0], // 完全匹配
        vec![0.8, 0.6, 0.0], // 部分匹配
        vec![0.0, 0.0, 1.0], // 不匹配
    ];
    
    let _ids = storage.upsert("ranking_test", vectors, None, None).await.unwrap();
    
    // 查询
    let query = vec![1.0, 0.0, 0.0];
    let results = storage.query("ranking_test", query, 3, None, true).await.unwrap();
    
    // 验证排序质量
    assert_eq!(results.len(), 3);
    assert!(results[0].score > results[1].score);
    assert!(results[1].score > results[2].score);
    
    // 完全匹配应该有最高分数
    assert!(results[0].score > 0.9);
}

#[tokio::test]
async fn test_rag_context_window() {
    // 测试上下文窗口管理
    let storage = MemoryVectorStorage::new(384, None);
    storage.create_index("context_test", 384, Some(lumosai_core::vector::SimilarityMetric::Cosine)).await.unwrap();
    
    // 插入多个文档
    let num_docs = 20;
    let vectors: Vec<Vec<f32>> = (0..num_docs)
        .map(|i| vec![i as f32 / num_docs as f32; 384])
        .collect();
    
    let _ids = storage.upsert("context_test", vectors, None, None).await.unwrap();
    
    // 测试不同的top_k值
    for k in [1, 5, 10, 15] {
        let results = storage.query("context_test", vec![0.5; 384], k, None, true).await.unwrap();
        assert_eq!(results.len(), k.min(num_docs));
    }
}

#[tokio::test]
async fn test_chunking_edge_cases() {
    // 测试分块边界情况
    let chunker = TextChunker::new(ChunkingConfig::default());
    
    let test_data = TestDataSets::load();
    
    // 测试边界情况
    for edge_case in &test_data.edge_cases {
        let result = chunker.chunk_text(edge_case);
        
        match edge_case.trim().is_empty() {
            true => {
                // 空文本应该返回空结果或错误
                if result.is_ok() {
                    let chunks = result.unwrap();
                    assert!(chunks.is_empty() || chunks.iter().all(|c| c.trim().is_empty()));
                }
                // 如果返回错误也是可以接受的
            }
            false => {
                // 非空文本应该返回有效结果
                assert!(result.is_ok());
                let chunks = result.unwrap();
                if !chunks.is_empty() {
                    assert!(chunks.iter().any(|c| !c.trim().is_empty()));
                }
            }
        }
    }
}

#[tokio::test]
async fn test_multilingual_chunking() {
    // 测试多语言分块
    let chunker = TextChunker::new(ChunkingConfig::default());
    let test_data = TestDataSets::load();
    
    // 测试中文文本
    if let Some(chinese_texts) = test_data.multilingual_content.get("chinese") {
        for text in chinese_texts {
            let chunks = chunker.chunk_text(text).unwrap();
            assert!(!chunks.is_empty());
            
            // 验证中文字符被正确处理
            for chunk in &chunks {
                assert!(!chunk.trim().is_empty());
            }
        }
    }
    
    // 测试日文文本
    if let Some(japanese_texts) = test_data.multilingual_content.get("japanese") {
        for text in japanese_texts {
            let chunks = chunker.chunk_text(text).unwrap();
            assert!(!chunks.is_empty());
        }
    }
}
