//! Context window management implementations

use async_trait::async_trait;
use crate::{
    types::ScoredDocument,
    error::Result,
};

/// Trait for context window management strategies
#[async_trait]
pub trait ContextWindow: Send + Sync {
    /// Apply window strategy to documents
    async fn apply_window(
        &self,
        documents: Vec<ScoredDocument>,
        max_documents: usize,
        max_tokens: usize,
    ) -> Result<Vec<ScoredDocument>>;
}

/// Fixed window implementation
pub struct FixedWindow;

impl FixedWindow {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ContextWindow for FixedWindow {
    async fn apply_window(
        &self,
        mut documents: Vec<ScoredDocument>,
        max_documents: usize,
        max_tokens: usize,
    ) -> Result<Vec<ScoredDocument>> {
        // Simple truncation to max documents
        documents.truncate(max_documents);
        
        // Apply token limit
        let mut total_tokens = 0;
        let mut result = Vec::new();
        
        for doc in documents {
            let doc_tokens = estimate_tokens(&doc.document.content);
            if total_tokens + doc_tokens <= max_tokens {
                total_tokens += doc_tokens;
                result.push(doc);
            } else {
                break;
            }
        }
        
        Ok(result)
    }
}

/// Sliding window with overlap
pub struct SlidingWindow {
    overlap: usize,
}

impl SlidingWindow {
    pub fn new(overlap: usize) -> Self {
        Self { overlap }
    }
}

#[async_trait]
impl ContextWindow for SlidingWindow {
    async fn apply_window(
        &self,
        documents: Vec<ScoredDocument>,
        max_documents: usize,
        max_tokens: usize,
    ) -> Result<Vec<ScoredDocument>> {
        if documents.len() <= max_documents {
            return Ok(documents);
        }
        
        // Create sliding windows with overlap
        let mut result = Vec::new();
        let mut current_tokens = 0;
        
        for (i, doc) in documents.iter().enumerate() {
            let doc_tokens = estimate_tokens(&doc.document.content);
            
            // Check if we should start a new window
            if i > 0 && i % (max_documents - self.overlap) == 0 {
                // Reset token count for new window
                current_tokens = 0;
                // Keep overlap documents
                let overlap_start = result.len().saturating_sub(self.overlap);
                result.truncate(overlap_start);
                current_tokens = result.iter()
                    .map(|d: &ScoredDocument| estimate_tokens(&d.document.content))
                    .sum();
            }
            
            if current_tokens + doc_tokens <= max_tokens {
                current_tokens += doc_tokens;
                result.push(doc.clone());
            } else {
                break;
            }
        }
        
        Ok(result)
    }
}

/// Adaptive window that adjusts based on content
pub struct AdaptiveWindow {
    min_size: usize,
    max_size: usize,
}

impl AdaptiveWindow {
    pub fn new(min_size: usize, max_size: usize) -> Self {
        Self { min_size, max_size }
    }
}

#[async_trait]
impl ContextWindow for AdaptiveWindow {
    async fn apply_window(
        &self,
        documents: Vec<ScoredDocument>,
        max_documents: usize,
        max_tokens: usize,
    ) -> Result<Vec<ScoredDocument>> {
        let mut result = Vec::new();
        let mut total_tokens = 0;
        
        // Calculate average document quality
        let avg_score = if !documents.is_empty() {
            documents.iter().map(|d| d.score).sum::<f32>() / documents.len() as f32
        } else {
            0.0
        };
        
        // Adaptive threshold based on quality
        let quality_threshold = avg_score * 0.8;
        
        for doc in documents {
            let doc_tokens = estimate_tokens(&doc.document.content);
            
            // Include high-quality documents even if they exceed normal limits
            let should_include = if doc.score >= quality_threshold {
                result.len() < self.max_size && total_tokens + doc_tokens <= max_tokens * 2
            } else {
                result.len() < max_documents && total_tokens + doc_tokens <= max_tokens
            };
            
            if should_include {
                total_tokens += doc_tokens;
                result.push(doc);
            } else if result.len() >= self.min_size {
                break;
            }
        }
        
        Ok(result)
    }
}

/// Hierarchical window with different levels
pub struct HierarchicalWindow {
    levels: Vec<usize>,
}

impl HierarchicalWindow {
    pub fn new(levels: Vec<usize>) -> Self {
        Self { levels }
    }
}

#[async_trait]
impl ContextWindow for HierarchicalWindow {
    async fn apply_window(
        &self,
        documents: Vec<ScoredDocument>,
        max_documents: usize,
        max_tokens: usize,
    ) -> Result<Vec<ScoredDocument>> {
        let mut result = Vec::new();
        let mut total_tokens = 0;
        let mut current_level = 0;
        
        // Sort documents by score (highest first)
        let mut sorted_docs = documents;
        sorted_docs.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        for (_i, doc) in sorted_docs.iter().enumerate() {
            // Move to next level if current level is full
            while current_level < self.levels.len() && 
                  result.len() >= self.levels[current_level] {
                current_level += 1;
            }
            
            // Stop if we've exceeded all levels
            if current_level >= self.levels.len() || result.len() >= max_documents {
                break;
            }
            
            let doc_tokens = estimate_tokens(&doc.document.content);
            if total_tokens + doc_tokens <= max_tokens {
                total_tokens += doc_tokens;
                result.push(doc.clone());
            } else {
                break;
            }
        }
        
        Ok(result)
    }
}

/// Estimate token count for text (simple word-based estimation)
fn estimate_tokens(text: &str) -> usize {
    // Simple estimation: ~1.3 tokens per word on average
    (text.split_whitespace().count() as f32 * 1.3) as usize
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
    async fn test_fixed_window() {
        let window = FixedWindow::new();
        let documents = vec![
            create_test_document("1", "Short doc", 0.9),
            create_test_document("2", "Another short document", 0.8),
            create_test_document("3", "This is a longer document with more content", 0.7),
        ];

        let result = window.apply_window(documents, 2, 100).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].document.id, "1");
        assert_eq!(result[1].document.id, "2");
    }

    #[tokio::test]
    async fn test_adaptive_window() {
        let window = AdaptiveWindow::new(1, 5);
        let documents = vec![
            create_test_document("1", "High quality doc", 0.95),
            create_test_document("2", "Medium quality doc", 0.7),
            create_test_document("3", "Low quality doc", 0.3),
        ];

        let result = window.apply_window(documents, 2, 100).await.unwrap();
        // Should include high quality document even if it exceeds normal limits
        assert!(!result.is_empty());
        assert_eq!(result[0].document.id, "1");
    }

    #[tokio::test]
    async fn test_hierarchical_window() {
        let window = HierarchicalWindow::new(vec![2, 3, 5]);
        let documents = vec![
            create_test_document("1", "Doc 1", 0.9),
            create_test_document("2", "Doc 2", 0.8),
            create_test_document("3", "Doc 3", 0.7),
            create_test_document("4", "Doc 4", 0.6),
        ];

        let result = window.apply_window(documents, 10, 1000).await.unwrap();
        assert_eq!(result.len(), 4);
        // Should be sorted by score
        assert_eq!(result[0].document.id, "1");
        assert_eq!(result[1].document.id, "2");
    }
}
