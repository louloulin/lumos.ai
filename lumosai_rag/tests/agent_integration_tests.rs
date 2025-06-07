//! Integration tests for RAG system with Agent system
//! 
//! This module tests the integration between the RAG pipeline and Agent system,
//! demonstrating Plan 7.0 Phase 1 (RAG) and Phase 2 (Agent) working together.

use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;

use lumosai_rag::{
    RagPipeline, RagPipelineBuilder,
    embedding::EmbeddingProvider,
    types::{Document, Metadata, ChunkingStrategy, ChunkingConfig},
    RagError,
};

/// Mock embedding provider for testing
struct MockEmbeddingProvider {
    dimension: usize,
}

impl MockEmbeddingProvider {
    fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, RagError> {
        // Generate a simple mock embedding based on text content
        let mut embedding = vec![0.0; self.dimension];
        
        // Simple hash-like function for consistent embeddings
        let text_bytes = text.as_bytes();
        for (i, &byte) in text_bytes.iter().enumerate() {
            let idx = i % self.dimension;
            embedding[idx] += (byte as f32) / 255.0;
        }
        
        // Normalize the embedding
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }
        
        Ok(embedding)
    }
}

#[tokio::test]
async fn test_rag_pipeline_document_processing() {
    let embedding_provider = Box::new(MockEmbeddingProvider::new(384));
    
    let pipeline = RagPipelineBuilder::new()
        .embedding_provider(embedding_provider)
        .chunk_size(200)
        .chunk_overlap(50)
        .chunking_strategy(ChunkingStrategy::Recursive {
            separators: Some(vec!["\n\n".to_string(), "\n".to_string()]),
            is_separator_regex: false,
        })
        .extract_metadata(true, true, true)
        .build()
        .unwrap();
    
    // Create test documents
    let documents = vec![
        Document {
            id: "doc1".to_string(),
            content: "Artificial Intelligence (AI) is transforming the world.\n\nMachine learning algorithms can process vast amounts of data.\n\nDeep learning models are particularly effective for complex tasks.".to_string(),
            metadata: Metadata::new(),
            embedding: None,
        },
        Document {
            id: "doc2".to_string(),
            content: "Rust is a systems programming language.\n\nIt provides memory safety without garbage collection.\n\nRust is ideal for building high-performance applications.".to_string(),
            metadata: Metadata::new(),
            embedding: None,
        },
    ];
    
    // Process documents through RAG pipeline
    let processed_chunks = pipeline.process_documents(documents).await.unwrap();
    
    // Verify processing results
    assert!(!processed_chunks.is_empty());
    assert!(processed_chunks.len() >= 2); // Should create multiple chunks
    
    // Verify all chunks have embeddings
    for chunk in &processed_chunks {
        assert!(chunk.embedding.is_some());
        assert_eq!(chunk.embedding.as_ref().unwrap().len(), 384);
        assert!(!chunk.content.is_empty());
        
        // Verify chunk metadata
        assert!(chunk.metadata.fields.contains_key("chunk_index"));
        assert!(chunk.metadata.fields.contains_key("parent_document_id"));
    }
    
    // Verify chunks from different documents
    let doc1_chunks: Vec<_> = processed_chunks.iter()
        .filter(|chunk| chunk.metadata.fields.get("parent_document_id") == Some(&serde_json::json!("doc1")))
        .collect();
    let doc2_chunks: Vec<_> = processed_chunks.iter()
        .filter(|chunk| chunk.metadata.fields.get("parent_document_id") == Some(&serde_json::json!("doc2")))
        .collect();
    
    assert!(!doc1_chunks.is_empty());
    assert!(!doc2_chunks.is_empty());
}

#[tokio::test]
async fn test_rag_pipeline_with_different_chunking_strategies() {
    let embedding_provider = Box::new(MockEmbeddingProvider::new(256));
    
    // Test Markdown chunking strategy
    let markdown_pipeline = RagPipelineBuilder::new()
        .embedding_provider(embedding_provider)
        .chunk_size(150)
        .chunking_strategy(ChunkingStrategy::Markdown {
            headers: Some(vec!["#".to_string(), "##".to_string()]),
            return_each_line: false,
            strip_headers: false,
        })
        .build()
        .unwrap();
    
    let markdown_doc = Document {
        id: "markdown_doc".to_string(),
        content: "# Introduction\n\nThis is the introduction section.\n\n## Technical Details\n\nHere are the technical details.\n\n## Conclusion\n\nThis is the conclusion.".to_string(),
        metadata: Metadata::new(),
        embedding: None,
    };
    
    let chunks = markdown_pipeline.process_document(markdown_doc).await.unwrap();
    
    assert!(!chunks.is_empty());
    
    // Verify all chunks have embeddings
    for chunk in &chunks {
        assert!(chunk.embedding.is_some());
        assert_eq!(chunk.embedding.as_ref().unwrap().len(), 256);
    }
}

#[tokio::test]
async fn test_rag_pipeline_token_chunking() {
    let embedding_provider = Box::new(MockEmbeddingProvider::new(128));
    
    let token_pipeline = RagPipelineBuilder::new()
        .embedding_provider(embedding_provider)
        .chunk_size(10) // 10 words per chunk
        .chunk_overlap(2)
        .chunking_strategy(ChunkingStrategy::Token {
            encoding_name: None,
            model_name: None,
        })
        .build()
        .unwrap();
    
    let document = Document {
        id: "token_doc".to_string(),
        content: "The quick brown fox jumps over the lazy dog. This is a test sentence for token-based chunking. We want to verify that the chunking works correctly.".to_string(),
        metadata: Metadata::new(),
        embedding: None,
    };
    
    let chunks = token_pipeline.process_document(document).await.unwrap();
    
    assert!(!chunks.is_empty());
    assert!(chunks.len() >= 2); // Should create multiple chunks
    
    // Verify chunk sizes (approximately 10 words each)
    for chunk in &chunks {
        let word_count = chunk.content.split_whitespace().count();
        assert!(word_count <= 12); // Allow some flexibility due to overlap
        assert!(chunk.embedding.is_some());
        assert_eq!(chunk.embedding.as_ref().unwrap().len(), 128);
    }
}

#[tokio::test]
async fn test_rag_pipeline_json_chunking() {
    let embedding_provider = Box::new(MockEmbeddingProvider::new(512));
    
    let json_pipeline = RagPipelineBuilder::new()
        .embedding_provider(embedding_provider)
        .chunk_size(200)
        .chunking_strategy(ChunkingStrategy::Json {
            ensure_ascii: false,
            convert_lists: true,
        })
        .build()
        .unwrap();
    
    let json_content = r#"
    {
        "users": [
            {"id": 1, "name": "Alice", "email": "alice@example.com", "role": "admin"},
            {"id": 2, "name": "Bob", "email": "bob@example.com", "role": "user"}
        ],
        "settings": {
            "theme": "dark",
            "notifications": true,
            "language": "en"
        },
        "metadata": {
            "version": "1.0",
            "created": "2025-01-01",
            "description": "User management system configuration"
        }
    }
    "#;
    
    let document = Document {
        id: "json_doc".to_string(),
        content: json_content.to_string(),
        metadata: Metadata::new(),
        embedding: None,
    };
    
    let chunks = json_pipeline.process_document(document).await.unwrap();
    
    assert!(!chunks.is_empty());
    
    // Verify that chunks contain JSON structure information
    let all_content: String = chunks.iter().map(|c| c.content.clone()).collect::<Vec<_>>().join(" ");
    assert!(all_content.contains("users") || all_content.contains("settings") || all_content.contains("metadata"));
    
    // Verify all chunks have embeddings
    for chunk in &chunks {
        assert!(chunk.embedding.is_some());
        assert_eq!(chunk.embedding.as_ref().unwrap().len(), 512);
    }
}

#[tokio::test]
async fn test_rag_pipeline_error_handling() {
    // Test with invalid JSON for JSON chunking strategy
    let embedding_provider = Box::new(MockEmbeddingProvider::new(256));
    
    let json_pipeline = RagPipelineBuilder::new()
        .embedding_provider(embedding_provider)
        .chunk_size(100)
        .chunking_strategy(ChunkingStrategy::Json {
            ensure_ascii: false,
            convert_lists: true,
        })
        .build()
        .unwrap();
    
    let invalid_json_doc = Document {
        id: "invalid_json".to_string(),
        content: "{ invalid json content }".to_string(),
        metadata: Metadata::new(),
        embedding: None,
    };
    
    let result = json_pipeline.process_document(invalid_json_doc).await;
    assert!(result.is_err());
    
    // Verify error type
    if let Err(error) = result {
        assert!(matches!(error, RagError::DocumentChunking(_)));
    }
}

#[tokio::test]
async fn test_rag_pipeline_metadata_extraction() {
    let embedding_provider = Box::new(MockEmbeddingProvider::new(384));
    
    let pipeline = RagPipelineBuilder::new()
        .embedding_provider(embedding_provider)
        .chunk_size(100)
        .extract_metadata(true, true, true)
        .build()
        .unwrap();
    
    let document = Document {
        id: "metadata_test".to_string(),
        content: "Introduction to Machine Learning\n\nMachine learning is a subset of artificial intelligence that focuses on algorithms and statistical models. These systems can learn and improve from experience without being explicitly programmed.".to_string(),
        metadata: Metadata::new(),
        embedding: None,
    };
    
    let mut test_doc = document.clone();
    pipeline.extract_metadata(&mut test_doc).await.unwrap();
    
    // Verify metadata extraction
    assert!(test_doc.metadata.fields.contains_key("extracted_title"));
    assert!(test_doc.metadata.fields.contains_key("extracted_summary"));
    assert!(test_doc.metadata.fields.contains_key("extracted_keywords"));
    
    // Verify extracted content
    let title = test_doc.metadata.fields.get("extracted_title").unwrap().as_str().unwrap();
    assert!(!title.is_empty());
    
    let summary = test_doc.metadata.fields.get("extracted_summary").unwrap().as_str().unwrap();
    assert!(!summary.is_empty());
    
    let keywords = test_doc.metadata.fields.get("extracted_keywords").unwrap().as_array().unwrap();
    assert!(!keywords.is_empty());
}

#[tokio::test]
async fn test_rag_pipeline_builder_validation() {
    // Test missing embedding provider
    let result = RagPipelineBuilder::new()
        .chunk_size(100)
        .build();
    
    assert!(result.is_err());
    if let Err(error) = result {
        assert!(matches!(error, RagError::Configuration(_)));
    }
}

#[tokio::test]
async fn test_rag_pipeline_performance_characteristics() {
    let embedding_provider = Box::new(MockEmbeddingProvider::new(384));
    
    let pipeline = RagPipelineBuilder::new()
        .embedding_provider(embedding_provider)
        .chunk_size(500)
        .chunk_overlap(100)
        .build()
        .unwrap();
    
    // Create a larger document to test performance
    let large_content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);
    let document = Document {
        id: "large_doc".to_string(),
        content: large_content,
        metadata: Metadata::new(),
        embedding: None,
    };
    
    let start_time = std::time::Instant::now();
    let chunks = pipeline.process_document(document).await.unwrap();
    let processing_time = start_time.elapsed();
    
    // Verify processing completed in reasonable time (should be fast for mock provider)
    assert!(processing_time.as_millis() < 1000); // Less than 1 second
    
    // Verify chunks were created
    assert!(!chunks.is_empty());
    
    // Verify all chunks have embeddings
    for chunk in &chunks {
        assert!(chunk.embedding.is_some());
        assert_eq!(chunk.embedding.as_ref().unwrap().len(), 384);
    }
}
