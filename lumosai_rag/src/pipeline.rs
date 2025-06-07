//! RAG Pipeline for document processing and retrieval
//!
//! This module provides a high-level pipeline for implementing RAG systems,
//! similar to Mastra's document processing pipeline.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::document::{DocumentChunker, EnhancedChunker};
use crate::embedding::EmbeddingProvider;
use crate::error::{RagError, Result};
use crate::types::{Document, ProcessingConfig, RetrievalOptions, RetrievalResult};

/// RAG Pipeline for processing documents and performing retrieval
pub struct RagPipeline {
    chunker: Box<dyn DocumentChunker>,
    embedding_provider: Box<dyn EmbeddingProvider>,
    config: ProcessingConfig,
}

impl RagPipeline {
    /// Create a new RAG pipeline with default configuration
    pub fn new(embedding_provider: Box<dyn EmbeddingProvider>) -> Self {
        Self {
            chunker: Box::new(EnhancedChunker::new()),
            embedding_provider,
            config: ProcessingConfig::default(),
        }
    }
    
    /// Create a new RAG pipeline with custom configuration
    pub fn with_config(
        embedding_provider: Box<dyn EmbeddingProvider>,
        config: ProcessingConfig,
    ) -> Self {
        Self {
            chunker: Box::new(EnhancedChunker::new()),
            embedding_provider,
            config,
        }
    }
    
    /// Create a builder for the RAG pipeline
    pub fn builder() -> RagPipelineBuilder {
        RagPipelineBuilder::new()
    }
    
    /// Process a single document through the RAG pipeline
    pub async fn process_document(&self, document: Document) -> Result<Vec<Document>> {
        // Step 1: Clean and normalize text if configured
        let cleaned_doc = if self.config.clean_text {
            self.clean_document(document)?
        } else {
            document
        };
        
        // Step 2: Chunk the document
        let chunks = self.chunker.chunk(cleaned_doc, &self.config.chunking).await?;
        
        // Step 3: Generate embeddings for each chunk
        let mut processed_chunks = Vec::with_capacity(chunks.len());
        
        for chunk in chunks {
            let embedding = self.embedding_provider
                .generate_embedding(&chunk.content)
                .await?;
            
            let mut processed_chunk = chunk;
            processed_chunk.embedding = Some(embedding);
            processed_chunks.push(processed_chunk);
        }
        
        Ok(processed_chunks)
    }
    
    /// Process multiple documents through the RAG pipeline
    pub async fn process_documents(&self, documents: Vec<Document>) -> Result<Vec<Document>> {
        let mut all_chunks = Vec::new();
        
        for document in documents {
            let chunks = self.process_document(document).await?;
            all_chunks.extend(chunks);
        }
        
        Ok(all_chunks)
    }
    
    /// Extract metadata from a document (title, summary, keywords, etc.)
    pub async fn extract_metadata(&self, document: &mut Document) -> Result<()> {
        if !self.config.extraction.extract_title &&
           !self.config.extraction.extract_summary &&
           !self.config.extraction.extract_keywords &&
           !self.config.extraction.extract_questions {
            return Ok(());
        }
        
        // TODO: Implement metadata extraction using LLM
        // For now, we'll add basic metadata
        if self.config.extraction.extract_title {
            let title = self.extract_title(&document.content)?;
            document.metadata.add("extracted_title", title);
        }
        
        if self.config.extraction.extract_summary {
            let summary = self.extract_summary(&document.content)?;
            document.metadata.add("extracted_summary", summary);
        }
        
        if self.config.extraction.extract_keywords {
            let keywords = self.extract_keywords(&document.content)?;
            document.metadata.add("extracted_keywords", keywords);
        }
        
        Ok(())
    }
    
    /// Clean and normalize document text
    fn clean_document(&self, mut document: Document) -> Result<Document> {
        // Remove extra whitespace
        document.content = document.content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        
        // Normalize unicode if configured
        if self.config.normalize_unicode {
            // Simple normalization - in a real implementation, use unicode-normalization crate
            document.content = document.content.chars().collect();
        }
        
        Ok(document)
    }
    
    /// Extract title from document content (simple implementation)
    fn extract_title(&self, content: &str) -> Result<String> {
        // Simple heuristic: first non-empty line or first sentence
        let lines: Vec<&str> = content.lines().collect();
        
        for line in lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                // If it looks like a title (short, no periods), use it
                if trimmed.len() < 100 && !trimmed.contains('.') {
                    return Ok(trimmed.to_string());
                }
                // Otherwise, take first sentence
                if let Some(first_sentence) = trimmed.split('.').next() {
                    return Ok(first_sentence.trim().to_string());
                }
            }
        }
        
        Ok("Untitled Document".to_string())
    }
    
    /// Extract summary from document content (simple implementation)
    fn extract_summary(&self, content: &str) -> Result<String> {
        // Simple heuristic: first paragraph or first few sentences
        let paragraphs: Vec<&str> = content.split("\n\n").collect();
        
        if let Some(first_paragraph) = paragraphs.first() {
            let trimmed = first_paragraph.trim();
            if trimmed.len() <= 500 {
                return Ok(trimmed.to_string());
            }
            
            // Take first few sentences if paragraph is too long
            let sentences: Vec<&str> = trimmed.split('.').take(3).collect();
            return Ok(sentences.join(".") + ".");
        }
        
        Ok("No summary available".to_string())
    }
    
    /// Extract keywords from document content (simple implementation)
    fn extract_keywords(&self, content: &str) -> Result<Vec<String>> {
        // Simple keyword extraction based on word frequency
        let lowercase_content = content.to_lowercase();
        let words: Vec<&str> = lowercase_content
            .split_whitespace()
            .filter(|word| word.len() > 3) // Filter short words
            .collect();
        
        let mut word_counts: HashMap<&str, usize> = HashMap::new();
        for word in words {
            *word_counts.entry(word).or_insert(0) += 1;
        }
        
        // Get top 10 most frequent words
        let mut sorted_words: Vec<_> = word_counts.into_iter().collect();
        sorted_words.sort_by(|a, b| b.1.cmp(&a.1));
        
        let keywords: Vec<String> = sorted_words
            .into_iter()
            .take(10)
            .map(|(word, _)| word.to_string())
            .collect();
        
        Ok(keywords)
    }
}

/// Builder for RAG Pipeline
pub struct RagPipelineBuilder {
    embedding_provider: Option<Box<dyn EmbeddingProvider>>,
    chunker: Option<Box<dyn DocumentChunker>>,
    config: ProcessingConfig,
}

impl RagPipelineBuilder {
    pub fn new() -> Self {
        Self {
            embedding_provider: None,
            chunker: None,
            config: ProcessingConfig::default(),
        }
    }
    
    pub fn embedding_provider(mut self, provider: Box<dyn EmbeddingProvider>) -> Self {
        self.embedding_provider = Some(provider);
        self
    }
    
    pub fn chunker(mut self, chunker: Box<dyn DocumentChunker>) -> Self {
        self.chunker = Some(chunker);
        self
    }
    
    pub fn config(mut self, config: ProcessingConfig) -> Self {
        self.config = config;
        self
    }
    
    pub fn chunking_strategy(mut self, strategy: crate::types::ChunkingStrategy) -> Self {
        self.config.chunking.strategy = strategy;
        self
    }
    
    pub fn chunk_size(mut self, size: usize) -> Self {
        self.config.chunking.chunk_size = size;
        self
    }
    
    pub fn chunk_overlap(mut self, overlap: usize) -> Self {
        self.config.chunking.chunk_overlap = overlap;
        self
    }
    
    pub fn extract_metadata(mut self, extract_title: bool, extract_summary: bool, extract_keywords: bool) -> Self {
        self.config.extraction.extract_title = extract_title;
        self.config.extraction.extract_summary = extract_summary;
        self.config.extraction.extract_keywords = extract_keywords;
        self
    }
    
    pub fn build(self) -> Result<RagPipeline> {
        let embedding_provider = self.embedding_provider
            .ok_or_else(|| RagError::Configuration("Embedding provider is required".into()))?;
        
        let chunker = self.chunker.unwrap_or_else(|| Box::new(EnhancedChunker::new()));
        
        Ok(RagPipeline {
            chunker,
            embedding_provider,
            config: self.config,
        })
    }
}

impl Default for RagPipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
