//! Hybrid retrieval combining vector similarity and keyword search
//! 
//! This module implements hybrid search strategies that combine:
//! - Vector similarity search for semantic matching
//! - Keyword/BM25 search for exact term matching
//! - Re-ranking algorithms for optimal result ordering

use async_trait::async_trait;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::{
    types::{Document, RetrievalRequest, RetrievalResult, RetrievalOptions},
    retriever::Retriever,
    error::Result,
};

/// Configuration for hybrid search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchConfig {
    /// Weight for vector similarity scores (0.0 to 1.0)
    pub vector_weight: f32,
    /// Weight for keyword search scores (0.0 to 1.0)
    pub keyword_weight: f32,
    /// Minimum score threshold for results
    pub min_score_threshold: f32,
    /// Maximum number of candidates from each search method
    pub max_candidates_per_method: usize,
    /// Re-ranking strategy
    pub rerank_strategy: RerankStrategy,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            vector_weight: 0.7,
            keyword_weight: 0.3,
            min_score_threshold: 0.1,
            max_candidates_per_method: 100,
            rerank_strategy: RerankStrategy::WeightedSum,
        }
    }
}

/// Re-ranking strategies for combining search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RerankStrategy {
    /// Simple weighted sum of scores
    WeightedSum,
    /// Reciprocal Rank Fusion (RRF)
    ReciprocalRankFusion { k: f32 },
    /// Convex combination with learned weights
    ConvexCombination { alpha: f32 },
    /// Rank-based fusion
    RankBasedFusion,
}

/// Hybrid retriever that combines vector and keyword search
pub struct HybridRetriever {
    vector_retriever: Box<dyn Retriever>,
    keyword_retriever: Box<dyn KeywordRetriever>,
    config: HybridSearchConfig,
}

impl HybridRetriever {
    /// Create a new hybrid retriever
    pub fn new(
        vector_retriever: Box<dyn Retriever>,
        keyword_retriever: Box<dyn KeywordRetriever>,
        config: HybridSearchConfig,
    ) -> Self {
        Self {
            vector_retriever,
            keyword_retriever,
            config,
        }
    }

    /// Create with default configuration
    pub fn with_default_config(
        vector_retriever: Box<dyn Retriever>,
        keyword_retriever: Box<dyn KeywordRetriever>,
    ) -> Self {
        Self::new(vector_retriever, keyword_retriever, HybridSearchConfig::default())
    }

    /// Update configuration
    pub fn with_config(mut self, config: HybridSearchConfig) -> Self {
        self.config = config;
        self
    }

    /// Combine and re-rank search results
    fn combine_results(
        &self,
        vector_results: Vec<ScoredDocument>,
        keyword_results: Vec<ScoredDocument>,
    ) -> Result<Vec<ScoredDocument>> {
        match self.config.rerank_strategy {
            RerankStrategy::WeightedSum => {
                self.weighted_sum_fusion(vector_results, keyword_results)
            }
            RerankStrategy::ReciprocalRankFusion { k } => {
                self.reciprocal_rank_fusion(vector_results, keyword_results, k)
            }
            RerankStrategy::ConvexCombination { alpha } => {
                self.convex_combination_fusion(vector_results, keyword_results, alpha)
            }
            RerankStrategy::RankBasedFusion => {
                self.rank_based_fusion(vector_results, keyword_results)
            }
        }
    }

    /// Weighted sum fusion strategy
    fn weighted_sum_fusion(
        &self,
        vector_results: Vec<ScoredDocument>,
        keyword_results: Vec<ScoredDocument>,
    ) -> Result<Vec<ScoredDocument>> {
        let mut combined_scores: HashMap<String, f32> = HashMap::new();
        let mut documents: HashMap<String, Document> = HashMap::new();

        // Add vector search scores
        for scored_doc in vector_results {
            let doc_id = scored_doc.document.id.clone();
            combined_scores.insert(
                doc_id.clone(),
                scored_doc.score * self.config.vector_weight,
            );
            documents.insert(doc_id, scored_doc.document);
        }

        // Add keyword search scores
        for scored_doc in keyword_results {
            let doc_id = scored_doc.document.id.clone();
            let existing_score = combined_scores.get(&doc_id).unwrap_or(&0.0);
            combined_scores.insert(
                doc_id.clone(),
                existing_score + (scored_doc.score * self.config.keyword_weight),
            );
            documents.insert(doc_id, scored_doc.document);
        }

        // Convert to sorted results
        let mut results: Vec<ScoredDocument> = combined_scores
            .into_iter()
            .filter_map(|(doc_id, score)| {
                if score >= self.config.min_score_threshold {
                    documents.remove(&doc_id).map(|doc| ScoredDocument {
                        document: doc,
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }

    /// Reciprocal Rank Fusion (RRF) strategy
    fn reciprocal_rank_fusion(
        &self,
        vector_results: Vec<ScoredDocument>,
        keyword_results: Vec<ScoredDocument>,
        k: f32,
    ) -> Result<Vec<ScoredDocument>> {
        let mut rrf_scores: HashMap<String, f32> = HashMap::new();
        let mut documents: HashMap<String, Document> = HashMap::new();

        // Process vector results
        for (rank, scored_doc) in vector_results.into_iter().enumerate() {
            let doc_id = scored_doc.document.id.clone();
            let rrf_score = 1.0 / (k + (rank as f32) + 1.0);
            rrf_scores.insert(doc_id.clone(), rrf_score * self.config.vector_weight);
            documents.insert(doc_id, scored_doc.document);
        }

        // Process keyword results
        for (rank, scored_doc) in keyword_results.into_iter().enumerate() {
            let doc_id = scored_doc.document.id.clone();
            let rrf_score = 1.0 / (k + (rank as f32) + 1.0);
            let existing_score = rrf_scores.get(&doc_id).unwrap_or(&0.0);
            rrf_scores.insert(
                doc_id.clone(),
                existing_score + (rrf_score * self.config.keyword_weight),
            );
            documents.insert(doc_id, scored_doc.document);
        }

        // Convert to sorted results
        let mut results: Vec<ScoredDocument> = rrf_scores
            .into_iter()
            .filter_map(|(doc_id, score)| {
                if score >= self.config.min_score_threshold {
                    documents.remove(&doc_id).map(|doc| ScoredDocument {
                        document: doc,
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }

    /// Convex combination fusion strategy
    fn convex_combination_fusion(
        &self,
        vector_results: Vec<ScoredDocument>,
        keyword_results: Vec<ScoredDocument>,
        alpha: f32,
    ) -> Result<Vec<ScoredDocument>> {
        // Normalize scores first
        let normalized_vector = self.normalize_scores(vector_results);
        let normalized_keyword = self.normalize_scores(keyword_results);

        let mut combined_scores: HashMap<String, f32> = HashMap::new();
        let mut documents: HashMap<String, Document> = HashMap::new();

        // Combine with convex combination: alpha * vector + (1-alpha) * keyword
        for scored_doc in normalized_vector {
            let doc_id = scored_doc.document.id.clone();
            combined_scores.insert(doc_id.clone(), alpha * scored_doc.score);
            documents.insert(doc_id, scored_doc.document);
        }

        for scored_doc in normalized_keyword {
            let doc_id = scored_doc.document.id.clone();
            let existing_score = combined_scores.get(&doc_id).unwrap_or(&0.0);
            combined_scores.insert(
                doc_id.clone(),
                existing_score + ((1.0 - alpha) * scored_doc.score),
            );
            documents.insert(doc_id, scored_doc.document);
        }

        // Convert to sorted results
        let mut results: Vec<ScoredDocument> = combined_scores
            .into_iter()
            .filter_map(|(doc_id, score)| {
                if score >= self.config.min_score_threshold {
                    documents.remove(&doc_id).map(|doc| ScoredDocument {
                        document: doc,
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }

    /// Rank-based fusion strategy
    fn rank_based_fusion(
        &self,
        vector_results: Vec<ScoredDocument>,
        keyword_results: Vec<ScoredDocument>,
    ) -> Result<Vec<ScoredDocument>> {
        let mut rank_scores: HashMap<String, f32> = HashMap::new();
        let mut documents: HashMap<String, Document> = HashMap::new();

        let vector_len = vector_results.len() as f32;
        let keyword_len = keyword_results.len() as f32;

        // Assign rank-based scores for vector results
        for (rank, scored_doc) in vector_results.into_iter().enumerate() {
            let doc_id = scored_doc.document.id.clone();
            let rank_score = (vector_len - rank as f32) / vector_len;
            rank_scores.insert(doc_id.clone(), rank_score * self.config.vector_weight);
            documents.insert(doc_id, scored_doc.document);
        }

        // Assign rank-based scores for keyword results
        for (rank, scored_doc) in keyword_results.into_iter().enumerate() {
            let doc_id = scored_doc.document.id.clone();
            let rank_score = (keyword_len - rank as f32) / keyword_len;
            let existing_score = rank_scores.get(&doc_id).unwrap_or(&0.0);
            rank_scores.insert(
                doc_id.clone(),
                existing_score + (rank_score * self.config.keyword_weight),
            );
            documents.insert(doc_id, scored_doc.document);
        }

        // Convert to sorted results
        let mut results: Vec<ScoredDocument> = rank_scores
            .into_iter()
            .filter_map(|(doc_id, score)| {
                if score >= self.config.min_score_threshold {
                    documents.remove(&doc_id).map(|doc| ScoredDocument {
                        document: doc,
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }

    /// Normalize scores to [0, 1] range
    fn normalize_scores(&self, results: Vec<ScoredDocument>) -> Vec<ScoredDocument> {
        if results.is_empty() {
            return results;
        }

        let max_score = results
            .iter()
            .map(|r| r.score)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_score = results
            .iter()
            .map(|r| r.score)
            .fold(f32::INFINITY, f32::min);

        let score_range = max_score - min_score;
        if score_range == 0.0 {
            return results;
        }

        results
            .into_iter()
            .map(|mut scored_doc| {
                scored_doc.score = (scored_doc.score - min_score) / score_range;
                scored_doc
            })
            .collect()
    }
}

#[async_trait]
impl Retriever for HybridRetriever {
    async fn retrieve(&self, request: &RetrievalRequest) -> Result<RetrievalResult> {
        // Perform vector search
        let vector_request = RetrievalRequest {
            query: request.query.clone(),
            options: RetrievalOptions {
                limit: Some(self.config.max_candidates_per_method),
                ..request.options.clone()
            },
        };
        let vector_result = self.vector_retriever.retrieve(&vector_request).await?;

        // Perform keyword search
        let keyword_results = self
            .keyword_retriever
            .search(&request.query, self.config.max_candidates_per_method)
            .await?;

        // Combine and re-rank results
        let combined_results = self.combine_results(
            vector_result.documents.into_iter().map(|doc| ScoredDocument {
                document: doc.document,
                score: doc.score,
            }).collect(),
            keyword_results,
        )?;

        // Apply final limit
        let final_limit = request.options.limit.unwrap_or(10);
        let final_documents: Vec<crate::types::ScoredDocument> = combined_results
            .into_iter()
            .take(final_limit)
            .map(|scored_doc| crate::types::ScoredDocument {
                document: scored_doc.document,
                score: scored_doc.score,
            })
            .collect();

        let total_count = final_documents.len();

        Ok(RetrievalResult {
            documents: final_documents,
            total_count,
        })
    }
}

/// Trait for keyword-based retrieval
#[async_trait]
pub trait KeywordRetriever: Send + Sync {
    /// Search for documents using keyword matching
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<ScoredDocument>>;
}

/// Simple scored document for internal use
#[derive(Debug, Clone)]
pub struct ScoredDocument {
    pub document: Document,
    pub score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Document, RetrievalRequest, RetrievalOptions};
    use async_trait::async_trait;

    // Mock vector retriever for testing
    struct MockVectorRetriever {
        documents: Vec<crate::types::ScoredDocument>,
    }

    #[async_trait]
    impl Retriever for MockVectorRetriever {
        async fn retrieve(&self, _request: &RetrievalRequest) -> Result<RetrievalResult> {
            Ok(RetrievalResult {
                documents: self.documents.clone(),
                total_count: self.documents.len(),
            })
        }
    }

    // Mock keyword retriever for testing
    struct MockKeywordRetriever {
        documents: Vec<ScoredDocument>,
    }

    #[async_trait]
    impl KeywordRetriever for MockKeywordRetriever {
        async fn search(&self, _query: &str, limit: usize) -> Result<Vec<ScoredDocument>> {
            Ok(self.documents.iter().take(limit).cloned().collect())
        }
    }

    fn create_test_document(id: &str, content: &str) -> Document {
        Document {
            id: id.to_string(),
            content: content.to_string(),
            metadata: crate::types::Metadata::new(),
            embedding: None,
        }
    }

    #[tokio::test]
    async fn test_hybrid_retriever_weighted_sum() {
        // Create mock retrievers
        let vector_retriever = Box::new(MockVectorRetriever {
            documents: vec![
                crate::types::ScoredDocument {
                    document: create_test_document("doc1", "vector content 1"),
                    score: 0.9,
                },
                crate::types::ScoredDocument {
                    document: create_test_document("doc2", "vector content 2"),
                    score: 0.7,
                },
            ],
        });

        let keyword_retriever = Box::new(MockKeywordRetriever {
            documents: vec![
                ScoredDocument {
                    document: create_test_document("doc2", "keyword content 2"),
                    score: 0.8,
                },
                ScoredDocument {
                    document: create_test_document("doc3", "keyword content 3"),
                    score: 0.6,
                },
            ],
        });

        let config = HybridSearchConfig {
            vector_weight: 0.7,
            keyword_weight: 0.3,
            min_score_threshold: 0.1,
            max_candidates_per_method: 10,
            rerank_strategy: RerankStrategy::WeightedSum,
        };

        let hybrid_retriever = HybridRetriever::new(vector_retriever, keyword_retriever, config);

        let request = RetrievalRequest {
            query: "test query".to_string(),
            options: RetrievalOptions {
                limit: Some(5),
                threshold: None,
                filter: None,
            },
        };

        let result = hybrid_retriever.retrieve(&request).await.unwrap();

        // Should have 3 documents (doc1, doc2, doc3)
        assert_eq!(result.documents.len(), 3);

        // doc2 should have the highest score (appears in both results)
        // doc2 score = 0.7 * 0.7 + 0.8 * 0.3 = 0.49 + 0.24 = 0.73
        assert_eq!(result.documents[0].document.id, "doc2");
        assert!((result.documents[0].score - 0.73).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_hybrid_retriever_rrf() {
        let vector_retriever = Box::new(MockVectorRetriever {
            documents: vec![
                crate::types::ScoredDocument {
                    document: create_test_document("doc1", "content 1"),
                    score: 0.9,
                },
                crate::types::ScoredDocument {
                    document: create_test_document("doc2", "content 2"),
                    score: 0.8,
                },
            ],
        });

        let keyword_retriever = Box::new(MockKeywordRetriever {
            documents: vec![
                ScoredDocument {
                    document: create_test_document("doc2", "content 2"),
                    score: 0.7,
                },
                ScoredDocument {
                    document: create_test_document("doc1", "content 1"),
                    score: 0.6,
                },
            ],
        });

        let config = HybridSearchConfig {
            vector_weight: 0.5,
            keyword_weight: 0.5,
            min_score_threshold: 0.0,
            max_candidates_per_method: 10,
            rerank_strategy: RerankStrategy::ReciprocalRankFusion { k: 60.0 },
        };

        let hybrid_retriever = HybridRetriever::new(vector_retriever, keyword_retriever, config);

        let request = RetrievalRequest {
            query: "test query".to_string(),
            options: RetrievalOptions::default(),
        };

        let result = hybrid_retriever.retrieve(&request).await.unwrap();

        // Should have 2 documents
        assert_eq!(result.documents.len(), 2);

        // Results should be sorted by RRF score
        assert!(result.documents[0].score >= result.documents[1].score);
    }

    #[tokio::test]
    async fn test_score_normalization() {
        let config = HybridSearchConfig::default();
        let vector_retriever = Box::new(MockVectorRetriever { documents: vec![] });
        let keyword_retriever = Box::new(MockKeywordRetriever { documents: vec![] });
        let hybrid_retriever = HybridRetriever::new(vector_retriever, keyword_retriever, config);

        let documents = vec![
            ScoredDocument {
                document: create_test_document("doc1", "content 1"),
                score: 10.0,
            },
            ScoredDocument {
                document: create_test_document("doc2", "content 2"),
                score: 5.0,
            },
            ScoredDocument {
                document: create_test_document("doc3", "content 3"),
                score: 0.0,
            },
        ];

        let normalized = hybrid_retriever.normalize_scores(documents);

        // Scores should be normalized to [0, 1]
        assert_eq!(normalized[0].score, 1.0); // (10-0)/(10-0) = 1
        assert_eq!(normalized[1].score, 0.5); // (5-0)/(10-0) = 0.5
        assert_eq!(normalized[2].score, 0.0); // (0-0)/(10-0) = 0
    }
}
