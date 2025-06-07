//! Context compression implementations

use async_trait::async_trait;
use std::collections::HashSet;

use crate::{
    types::ScoredDocument,
    error::Result,
};

/// Trait for context compression strategies
#[async_trait]
pub trait ContextCompressor: Send + Sync {
    /// Compress context documents
    async fn compress_context(&self, documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>>;
}

/// Deduplication compressor - removes redundant content
pub struct DeduplicationCompressor {
    similarity_threshold: f32,
}

impl DeduplicationCompressor {
    pub fn new() -> Self {
        Self {
            similarity_threshold: 0.8,
        }
    }

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.similarity_threshold = threshold;
        self
    }

    /// Calculate simple text similarity based on word overlap
    fn calculate_similarity(&self, text1: &str, text2: &str) -> f32 {
        let words1: HashSet<&str> = text1.split_whitespace().collect();
        let words2: HashSet<&str> = text2.split_whitespace().collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
}

#[async_trait]
impl ContextCompressor for DeduplicationCompressor {
    async fn compress_context(&self, documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>> {
        let mut result: Vec<ScoredDocument> = Vec::new();

        for doc in documents {
            let mut is_duplicate = false;

            // Check against already selected documents
            for existing in &result {
                let similarity = self.calculate_similarity(
                    &doc.document.content,
                    &existing.document.content,
                );

                if similarity >= self.similarity_threshold {
                    is_duplicate = true;
                    break;
                }
            }

            if !is_duplicate {
                result.push(doc);
            }
        }

        Ok(result)
    }
}

/// Extraction compressor - extracts key sentences
pub struct ExtractionCompressor {
    max_sentences: usize,
}

impl ExtractionCompressor {
    pub fn new(max_sentences: usize) -> Self {
        Self { max_sentences }
    }

    /// Extract key sentences based on position and length
    fn extract_key_sentences(&self, text: &str) -> String {
        let sentences: Vec<&str> = text.split('.').collect();
        if sentences.len() <= self.max_sentences {
            return text.to_string();
        }

        let mut selected = Vec::new();
        
        // Always include first sentence (often contains key info)
        if !sentences.is_empty() {
            selected.push(sentences[0]);
        }
        
        // Select sentences from middle and end
        let remaining = self.max_sentences.saturating_sub(1);
        if remaining > 0 && sentences.len() > 1 {
            let step = sentences.len() / remaining;
            for i in (step..sentences.len()).step_by(step) {
                if selected.len() < self.max_sentences {
                    selected.push(sentences[i]);
                }
            }
        }
        
        selected.join(".") + "."
    }
}

#[async_trait]
impl ContextCompressor for ExtractionCompressor {
    async fn compress_context(&self, documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>> {
        let mut result = Vec::new();
        
        for doc in documents {
            let compressed_content = self.extract_key_sentences(&doc.document.content);
            
            let mut compressed_doc = doc.clone();
            compressed_doc.document.content = compressed_content;
            result.push(compressed_doc);
        }
        
        Ok(result)
    }
}

/// Summarization compressor - creates summaries of content
pub struct SummarizationCompressor {
    max_length: usize,
}

impl SummarizationCompressor {
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }

    /// Simple extractive summarization
    fn summarize_text(&self, text: &str) -> String {
        if text.len() <= self.max_length {
            return text.to_string();
        }

        // Simple approach: take first part and last part
        let half_length = self.max_length / 2;
        let first_part = text.chars().take(half_length).collect::<String>();
        let last_part = text.chars().rev().take(half_length).collect::<String>()
            .chars().rev().collect::<String>();
        
        format!("{}...\n\n...{}", first_part.trim(), last_part.trim())
    }
}

#[async_trait]
impl ContextCompressor for SummarizationCompressor {
    async fn compress_context(&self, documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>> {
        let mut result = Vec::new();
        
        for doc in documents {
            let summarized_content = self.summarize_text(&doc.document.content);
            
            let mut summarized_doc = doc.clone();
            summarized_doc.document.content = summarized_content;
            result.push(summarized_doc);
        }
        
        Ok(result)
    }
}

/// Hybrid compressor combining multiple strategies
pub struct HybridCompressor {
    deduplicator: DeduplicationCompressor,
    extractor: ExtractionCompressor,
}

impl HybridCompressor {
    pub fn new() -> Self {
        Self {
            deduplicator: DeduplicationCompressor::new(),
            extractor: ExtractionCompressor::new(3), // Default to 3 sentences
        }
    }
}

#[async_trait]
impl ContextCompressor for HybridCompressor {
    async fn compress_context(&self, documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>> {
        // First apply deduplication
        let deduplicated = self.deduplicator.compress_context(documents).await?;
        
        // Then apply extraction
        let extracted = self.extractor.compress_context(deduplicated).await?;
        
        Ok(extracted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Document, Metadata};

    fn create_test_document(id: &str, content: &str, score: f32) -> ScoredDocument {
        ScoredDocument {
            document: Document {
                id: id.to_string(),
                content: content.to_string(),
                metadata: Metadata::new(),
                embedding: None,
            },
            score,
        }
    }

    #[tokio::test]
    async fn test_deduplication_compressor() {
        let compressor = DeduplicationCompressor::new().with_threshold(0.8);
        let documents = vec![
            create_test_document("1", "This is a document about cats and their behavior", 0.9),
            create_test_document("2", "This is a document about dogs and their training", 0.8),
            create_test_document("3", "This is a document about cats and their behavior", 0.7), // Exact duplicate
        ];

        let result = compressor.compress_context(documents).await.unwrap();

        // Should remove exact duplicate (doc3)
        assert_eq!(result.len(), 2, "Expected 2 documents after deduplication, got {}", result.len());

        // Check that we have the first two documents (different content)
        let ids: Vec<&str> = result.iter().map(|d| d.document.id.as_str()).collect();
        assert!(ids.contains(&"1"));
        assert!(ids.contains(&"2"));

        // Verify the duplicate was removed
        assert!(!ids.contains(&"3"));
    }

    #[tokio::test]
    async fn test_extraction_compressor() {
        let compressor = ExtractionCompressor::new(2);
        let documents = vec![
            create_test_document(
                "1", 
                "First sentence. Second sentence. Third sentence. Fourth sentence.", 
                0.9
            ),
        ];

        let result = compressor.compress_context(documents).await.unwrap();
        assert_eq!(result.len(), 1);
        
        // Should extract key sentences
        let content = &result[0].document.content;
        assert!(content.contains("First sentence"));
        assert!(content.len() < "First sentence. Second sentence. Third sentence. Fourth sentence.".len());
    }

    #[tokio::test]
    async fn test_summarization_compressor() {
        let compressor = SummarizationCompressor::new(50);
        let documents = vec![
            create_test_document(
                "1", 
                "This is a very long document that needs to be summarized because it contains too much information for the context window and we need to compress it down to a smaller size.", 
                0.9
            ),
        ];

        let result = compressor.compress_context(documents).await.unwrap();
        assert_eq!(result.len(), 1);
        
        // Should be shorter than original
        let content = &result[0].document.content;
        assert!(content.len() <= 50 + 10); // Allow some margin for formatting
    }

    #[tokio::test]
    async fn test_hybrid_compressor() {
        let compressor = HybridCompressor::new();
        let documents = vec![
            create_test_document("1", "First document. Second sentence. Third sentence.", 0.9),
            create_test_document("2", "First document. Second sentence. Third sentence.", 0.8), // Duplicate
            create_test_document("3", "Different document. With different content. And more sentences.", 0.7),
        ];

        let result = compressor.compress_context(documents).await.unwrap();
        assert_eq!(result.len(), 2); // Should remove duplicate
        
        // Should also apply extraction
        for doc in &result {
            assert!(doc.document.content.len() > 0);
        }
    }
}
