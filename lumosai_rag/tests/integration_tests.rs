use async_trait::async_trait;
use lumosai_rag::{
    document::{DocumentChunker, EnhancedChunker},
    embedding::EmbeddingProvider,
    pipeline::{RagPipeline, RagPipelineBuilder},
    types::{ChunkingConfig, ChunkingStrategy, Document, Metadata, ProcessingConfig},
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
        // Generate a simple mock embedding based on text length and content
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
async fn test_enhanced_chunker_recursive_strategy() {
    let chunker = EnhancedChunker::new();
    
    let document = Document {
        id: "test-doc".to_string(),
        content: "This is the first paragraph.\n\nThis is the second paragraph.\n\nThis is the third paragraph with more content to test chunking.".to_string(),
        metadata: Metadata::new(),
        embedding: None,
    };
    
    let config = ChunkingConfig {
        chunk_size: 50,
        chunk_overlap: 10,
        strategy: ChunkingStrategy::Recursive {
            separators: None,
            is_separator_regex: false,
        },
        ..Default::default()
    };
    
    let chunks = chunker.chunk(document, &config).await.unwrap();
    
    assert!(!chunks.is_empty());
    assert!(chunks.len() >= 2); // Should create multiple chunks
    
    // Verify chunk metadata
    for (i, chunk) in chunks.iter().enumerate() {
        assert_eq!(chunk.metadata.fields.get("chunk_index").unwrap(), &serde_json::json!(i as i64));
        assert_eq!(chunk.metadata.fields.get("parent_document_id").unwrap(), &serde_json::json!("test-doc"));
        assert!(!chunk.content.is_empty());
    }
}

#[tokio::test]
async fn test_enhanced_chunker_markdown_strategy() {
    let chunker = EnhancedChunker::new();
    
    let document = Document {
        id: "markdown-doc".to_string(),
        content: "# Main Title\n\nThis is the introduction.\n\n## Section 1\n\nContent of section 1.\n\n## Section 2\n\nContent of section 2.".to_string(),
        metadata: Metadata::new(),
        embedding: None,
    };
    
    let config = ChunkingConfig {
        chunk_size: 100,
        chunk_overlap: 0,
        strategy: ChunkingStrategy::Markdown {
            headers: Some(vec!["#".to_string(), "##".to_string()]),
            return_each_line: false,
            strip_headers: false,
        },
        ..Default::default()
    };
    
    let chunks = chunker.chunk(document, &config).await.unwrap();
    
    assert!(!chunks.is_empty());
    
    // Verify that chunks contain header information
    let content_combined: String = chunks.iter().map(|c| c.content.clone()).collect::<Vec<_>>().join(" ");
    assert!(content_combined.contains("Main Title") || content_combined.contains("Section"));
}

#[tokio::test]
async fn test_enhanced_chunker_token_strategy() {
    let chunker = EnhancedChunker::new();
    
    let document = Document {
        id: "token-doc".to_string(),
        content: "The quick brown fox jumps over the lazy dog. This is a test sentence for token-based chunking.".to_string(),
        metadata: Metadata::new(),
        embedding: None,
    };
    
    let config = ChunkingConfig {
        chunk_size: 5, // 5 words per chunk
        chunk_overlap: 1,
        strategy: ChunkingStrategy::Token {
            encoding_name: None,
            model_name: None,
        },
        ..Default::default()
    };
    
    let chunks = chunker.chunk(document, &config).await.unwrap();
    
    assert!(!chunks.is_empty());
    assert!(chunks.len() >= 2); // Should create multiple chunks
    
    // Verify that each chunk has approximately the right number of words
    for chunk in &chunks {
        let word_count = chunk.content.split_whitespace().count();
        assert!(word_count <= 6); // Allow some flexibility due to overlap
    }
}

#[tokio::test]
async fn test_rag_pipeline_basic_processing() {
    let embedding_provider = Box::new(MockEmbeddingProvider::new(384));
    let pipeline = RagPipeline::new(embedding_provider);
    
    let document = Document {
        id: "pipeline-test".to_string(),
        content: "This is a test document for the RAG pipeline. It should be chunked and embedded properly.".to_string(),
        metadata: Metadata::new(),
        embedding: None,
    };
    
    let processed_chunks = pipeline.process_document(document).await.unwrap();
    
    assert!(!processed_chunks.is_empty());
    
    // Verify that all chunks have embeddings
    for chunk in &processed_chunks {
        assert!(chunk.embedding.is_some());
        assert_eq!(chunk.embedding.as_ref().unwrap().len(), 384);
        assert!(!chunk.content.is_empty());
    }
}

#[tokio::test]
async fn test_rag_pipeline_builder() {
    let embedding_provider = Box::new(MockEmbeddingProvider::new(512));
    
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
    
    let document = Document {
        id: "builder-test".to_string(),
        content: "First paragraph with some content.\n\nSecond paragraph with different content.\n\nThird paragraph for testing.".to_string(),
        metadata: Metadata::new(),
        embedding: None,
    };
    
    let processed_chunks = pipeline.process_document(document).await.unwrap();
    
    assert!(!processed_chunks.is_empty());
    
    // Verify embeddings have correct dimension
    for chunk in &processed_chunks {
        assert!(chunk.embedding.is_some());
        assert_eq!(chunk.embedding.as_ref().unwrap().len(), 512);
    }
}

#[tokio::test]
async fn test_rag_pipeline_multiple_documents() {
    let embedding_provider = Box::new(MockEmbeddingProvider::new(256));
    let pipeline = RagPipeline::new(embedding_provider);
    
    let documents = vec![
        Document {
            id: "doc1".to_string(),
            content: "First document content for testing.".to_string(),
            metadata: Metadata::new(),
            embedding: None,
        },
        Document {
            id: "doc2".to_string(),
            content: "Second document with different content for testing the pipeline.".to_string(),
            metadata: Metadata::new(),
            embedding: None,
        },
    ];
    
    let processed_chunks = pipeline.process_documents(documents).await.unwrap();
    
    assert!(!processed_chunks.is_empty());
    
    // Verify that we have chunks from both documents
    let doc1_chunks: Vec<_> = processed_chunks.iter()
        .filter(|chunk| chunk.metadata.fields.get("parent_document_id") == Some(&serde_json::json!("doc1")))
        .collect();
    let doc2_chunks: Vec<_> = processed_chunks.iter()
        .filter(|chunk| chunk.metadata.fields.get("parent_document_id") == Some(&serde_json::json!("doc2")))
        .collect();
    
    assert!(!doc1_chunks.is_empty());
    assert!(!doc2_chunks.is_empty());
    
    // Verify all chunks have embeddings
    for chunk in &processed_chunks {
        assert!(chunk.embedding.is_some());
        assert_eq!(chunk.embedding.as_ref().unwrap().len(), 256);
    }
}

#[tokio::test]
async fn test_json_chunking_strategy() {
    let chunker = EnhancedChunker::new();
    
    let json_content = r#"
    {
        "users": [
            {"id": 1, "name": "Alice", "email": "alice@example.com"},
            {"id": 2, "name": "Bob", "email": "bob@example.com"}
        ],
        "settings": {
            "theme": "dark",
            "notifications": true
        }
    }
    "#;
    
    let document = Document {
        id: "json-doc".to_string(),
        content: json_content.to_string(),
        metadata: Metadata::new(),
        embedding: None,
    };
    
    let config = ChunkingConfig {
        chunk_size: 200,
        chunk_overlap: 0,
        strategy: ChunkingStrategy::Json {
            ensure_ascii: false,
            convert_lists: true,
        },
        ..Default::default()
    };
    
    let chunks = chunker.chunk(document, &config).await.unwrap();
    
    assert!(!chunks.is_empty());
    
    // Verify that chunks contain JSON structure information
    let all_content: String = chunks.iter().map(|c| c.content.clone()).collect::<Vec<_>>().join(" ");
    assert!(all_content.contains("users") || all_content.contains("settings"));
}

#[tokio::test]
async fn test_pipeline_error_handling() {
    // Test with missing embedding provider
    let result = RagPipelineBuilder::new()
        .chunk_size(100)
        .build();

    assert!(result.is_err());
    if let Err(error) = result {
        assert!(matches!(error, RagError::Configuration(_)));
    }
}
