//! Document ranking implementations for context management

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::cmp::Ordering;

use crate::{
    types::ScoredDocument,
    error::Result,
};

/// Trait for document ranking strategies
#[async_trait]
pub trait DocumentRanker: Send + Sync {
    /// Rank documents according to the strategy
    async fn rank_documents(&self, documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>>;
}

/// Relevance-based ranking (uses existing scores)
pub struct RelevanceRanker;

impl RelevanceRanker {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DocumentRanker for RelevanceRanker {
    async fn rank_documents(&self, mut documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>> {
        // Sort by relevance score (highest first)
        documents.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal)
        });
        Ok(documents)
    }
}

/// Recency-based ranking (requires timestamp metadata)
pub struct RecencyRanker;

impl RecencyRanker {
    pub fn new() -> Self {
        Self
    }

    fn extract_timestamp(&self, document: &ScoredDocument) -> Option<DateTime<Utc>> {
        // Try to extract timestamp from metadata
        if let Some(created_at) = &document.document.metadata.created_at {
            Some(*created_at)
        } else if let Some(timestamp_str) = document.document.metadata.fields.get("timestamp") {
            // Try to parse timestamp from metadata fields
            if let Value::String(ts) = timestamp_str {
                DateTime::parse_from_rfc3339(ts)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[async_trait]
impl DocumentRanker for RecencyRanker {
    async fn rank_documents(&self, mut documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>> {
        // Sort by recency (most recent first)
        documents.sort_by(|a, b| {
            let timestamp_a = self.extract_timestamp(a);
            let timestamp_b = self.extract_timestamp(b);
            
            match (timestamp_a, timestamp_b) {
                (Some(ts_a), Some(ts_b)) => ts_b.cmp(&ts_a), // Most recent first
                (Some(_), None) => Ordering::Less,           // Documents with timestamps first
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,             // Maintain original order
            }
        });
        Ok(documents)
    }
}

/// Length-based ranking (shorter documents first)
pub struct LengthRanker;

impl LengthRanker {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DocumentRanker for LengthRanker {
    async fn rank_documents(&self, mut documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>> {
        // Sort by length (shorter first for better context utilization)
        documents.sort_by(|a, b| {
            a.document.content.len().cmp(&b.document.content.len())
        });
        Ok(documents)
    }
}

/// Custom ranking (placeholder for user-defined ranking)
pub struct CustomRanker;

impl CustomRanker {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DocumentRanker for CustomRanker {
    async fn rank_documents(&self, documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>> {
        // Placeholder implementation - maintain original order
        // In a real implementation, this would use a user-provided ranking function
        Ok(documents)
    }
}

/// Hybrid ranking combining multiple factors
pub struct HybridRanker {
    relevance_weight: f32,
    recency_weight: f32,
    length_weight: f32,
}

impl HybridRanker {
    pub fn new(relevance_weight: f32, recency_weight: f32, length_weight: f32) -> Self {
        Self {
            relevance_weight,
            recency_weight,
            length_weight,
        }
    }

    fn calculate_hybrid_score(&self, document: &ScoredDocument) -> f32 {
        let mut score = 0.0;

        // Relevance component (normalized to 0-1)
        score += document.score * self.relevance_weight;

        // Recency component
        if let Some(timestamp) = self.extract_timestamp(document) {
            let now = Utc::now();
            let age_hours = (now - timestamp).num_hours() as f32;
            // Decay function: more recent = higher score
            let recency_score = (-age_hours / (24.0 * 7.0)).exp(); // Week-based decay
            score += recency_score * self.recency_weight;
        }

        // Length component (shorter documents get higher scores)
        let length = document.document.content.len() as f32;
        let length_score = 1.0 / (1.0 + length / 1000.0); // Normalize around 1000 chars
        score += length_score * self.length_weight;

        score
    }

    fn extract_timestamp(&self, document: &ScoredDocument) -> Option<DateTime<Utc>> {
        if let Some(created_at) = &document.document.metadata.created_at {
            Some(*created_at)
        } else if let Some(timestamp_str) = document.document.metadata.fields.get("timestamp") {
            if let Value::String(ts) = timestamp_str {
                DateTime::parse_from_rfc3339(ts)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[async_trait]
impl DocumentRanker for HybridRanker {
    async fn rank_documents(&self, mut documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>> {
        // Calculate hybrid scores and sort
        documents.sort_by(|a, b| {
            let score_a = self.calculate_hybrid_score(a);
            let score_b = self.calculate_hybrid_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(Ordering::Equal)
        });
        Ok(documents)
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

    fn create_test_document_with_timestamp(
        id: &str, 
        content: &str, 
        score: f32, 
        timestamp: DateTime<Utc>
    ) -> ScoredDocument {
        let mut metadata = Metadata::new();
        metadata.created_at = Some(timestamp);
        
        ScoredDocument {
            document: Document {
                id: id.to_string(),
                content: content.to_string(),
                metadata,
                embedding: None,
            },
            score,
        }
    }

    #[tokio::test]
    async fn test_relevance_ranker() {
        let ranker = RelevanceRanker::new();
        let documents = vec![
            create_test_document("1", "Content 1", 0.7),
            create_test_document("2", "Content 2", 0.9),
            create_test_document("3", "Content 3", 0.5),
        ];

        let result = ranker.rank_documents(documents).await.unwrap();
        assert_eq!(result[0].document.id, "2"); // Highest score
        assert_eq!(result[1].document.id, "1");
        assert_eq!(result[2].document.id, "3"); // Lowest score
    }

    #[tokio::test]
    async fn test_recency_ranker() {
        let ranker = RecencyRanker::new();
        let now = Utc::now();
        let hour_ago = now - chrono::Duration::hours(1);
        let day_ago = now - chrono::Duration::days(1);

        let documents = vec![
            create_test_document_with_timestamp("1", "Old content", 0.8, day_ago),
            create_test_document_with_timestamp("2", "Recent content", 0.7, now),
            create_test_document_with_timestamp("3", "Medium content", 0.9, hour_ago),
        ];

        let result = ranker.rank_documents(documents).await.unwrap();
        assert_eq!(result[0].document.id, "2"); // Most recent
        assert_eq!(result[1].document.id, "3");
        assert_eq!(result[2].document.id, "1"); // Oldest
    }

    #[tokio::test]
    async fn test_length_ranker() {
        let ranker = LengthRanker::new();
        let documents = vec![
            create_test_document("1", "This is a very long document with lots of content", 0.8),
            create_test_document("2", "Short", 0.7),
            create_test_document("3", "Medium length document", 0.9),
        ];

        let result = ranker.rank_documents(documents).await.unwrap();
        assert_eq!(result[0].document.id, "2"); // Shortest
        assert_eq!(result[2].document.id, "1"); // Longest
    }

    #[tokio::test]
    async fn test_hybrid_ranker() {
        let ranker = HybridRanker::new(0.5, 0.3, 0.2);
        let now = Utc::now();
        let hour_ago = now - chrono::Duration::hours(1);

        let documents = vec![
            create_test_document_with_timestamp("1", "Long old content with lower relevance", 0.6, hour_ago),
            create_test_document_with_timestamp("2", "Short recent content", 0.8, now),
        ];

        let result = ranker.rank_documents(documents).await.unwrap();
        // Should favor the recent, short, high-relevance document
        assert_eq!(result[0].document.id, "2");
    }
}
