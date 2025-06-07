//! BM25 keyword retrieval implementation
//! 
//! This module provides a BM25-based keyword retrieval system for exact term matching
//! and traditional information retrieval scoring.

use async_trait::async_trait;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::{
    types::Document,
    retriever::hybrid::{KeywordRetriever, ScoredDocument},
    error::Result,
};

/// Configuration for BM25 algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BM25Config {
    /// k1 parameter (term frequency saturation)
    pub k1: f32,
    /// b parameter (length normalization)
    pub b: f32,
    /// Minimum term frequency to consider
    pub min_term_freq: usize,
    /// Maximum number of terms to consider per document
    pub max_terms_per_doc: usize,
}

impl Default for BM25Config {
    fn default() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
            min_term_freq: 1,
            max_terms_per_doc: 1000,
        }
    }
}

/// BM25-based keyword retriever
pub struct BM25Retriever {
    documents: Vec<Document>,
    term_frequencies: HashMap<String, HashMap<String, usize>>, // doc_id -> term -> freq
    document_frequencies: HashMap<String, usize>, // term -> doc_count
    document_lengths: HashMap<String, usize>, // doc_id -> length
    average_document_length: f32,
    config: BM25Config,
}

impl BM25Retriever {
    /// Create a new BM25 retriever
    pub fn new(documents: Vec<Document>, config: BM25Config) -> Result<Self> {
        let mut retriever = Self {
            documents: Vec::new(),
            term_frequencies: HashMap::new(),
            document_frequencies: HashMap::new(),
            document_lengths: HashMap::new(),
            average_document_length: 0.0,
            config,
        };

        retriever.index_documents(documents)?;
        Ok(retriever)
    }

    /// Create with default configuration
    pub fn with_default_config(documents: Vec<Document>) -> Result<Self> {
        Self::new(documents, BM25Config::default())
    }

    /// Index documents for BM25 search
    fn index_documents(&mut self, documents: Vec<Document>) -> Result<()> {
        self.documents = documents;
        self.term_frequencies.clear();
        self.document_frequencies.clear();
        self.document_lengths.clear();

        let mut total_length = 0;

        // First pass: calculate term frequencies and document lengths
        for document in &self.documents {
            let terms = self.tokenize(&document.content);
            let doc_length = terms.len();
            total_length += doc_length;

            self.document_lengths.insert(document.id.clone(), doc_length);

            let mut term_freq: HashMap<String, usize> = HashMap::new();
            for term in terms {
                if term.len() >= 2 { // Filter out very short terms
                    *term_freq.entry(term).or_insert(0) += 1;
                }
            }

            // Limit terms per document
            if term_freq.len() > self.config.max_terms_per_doc {
                let mut sorted_terms: Vec<_> = term_freq.into_iter().collect();
                sorted_terms.sort_by(|a, b| b.1.cmp(&a.1));
                term_freq = sorted_terms
                    .into_iter()
                    .take(self.config.max_terms_per_doc)
                    .collect();
            }

            self.term_frequencies.insert(document.id.clone(), term_freq);
        }

        // Calculate average document length
        self.average_document_length = if self.documents.is_empty() {
            0.0
        } else {
            total_length as f32 / self.documents.len() as f32
        };

        // Second pass: calculate document frequencies
        for term_freq_map in self.term_frequencies.values() {
            for (term, freq) in term_freq_map {
                if *freq >= self.config.min_term_freq {
                    *self.document_frequencies.entry(term.clone()).or_insert(0) += 1;
                }
            }
        }

        Ok(())
    }

    /// Simple tokenization (split by whitespace and punctuation)
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|word| {
                // Remove punctuation from start and end
                word.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|word| !word.is_empty())
            .collect()
    }

    /// Calculate BM25 score for a document given query terms
    fn calculate_bm25_score(&self, doc_id: &str, query_terms: &[String]) -> f32 {
        let doc_length = *self.document_lengths.get(doc_id).unwrap_or(&0) as f32;
        let term_freq_map = match self.term_frequencies.get(doc_id) {
            Some(map) => map,
            None => return 0.0,
        };

        let mut score = 0.0;

        for term in query_terms {
            let term_freq = *term_freq_map.get(term).unwrap_or(&0) as f32;
            if term_freq == 0.0 {
                continue;
            }

            let doc_freq = *self.document_frequencies.get(term).unwrap_or(&0) as f32;
            if doc_freq == 0.0 {
                continue;
            }

            let total_docs = self.documents.len() as f32;
            
            // IDF calculation: log((N - df + 0.5) / (df + 0.5))
            let idf = ((total_docs - doc_freq + 0.5) / (doc_freq + 0.5)).ln();
            
            // TF calculation with BM25 normalization
            let tf_component = (term_freq * (self.config.k1 + 1.0)) / 
                (term_freq + self.config.k1 * (1.0 - self.config.b + 
                    self.config.b * (doc_length / self.average_document_length)));

            score += idf * tf_component;
        }

        score
    }

    /// Add new documents to the index
    pub fn add_documents(&mut self, mut new_documents: Vec<Document>) -> Result<()> {
        self.documents.append(&mut new_documents);
        self.index_documents(self.documents.clone())?;
        Ok(())
    }

    /// Remove documents from the index
    pub fn remove_documents(&mut self, doc_ids: &[String]) -> Result<()> {
        self.documents.retain(|doc| !doc_ids.contains(&doc.id));
        
        for doc_id in doc_ids {
            self.term_frequencies.remove(doc_id);
            self.document_lengths.remove(doc_id);
        }

        // Reindex to update document frequencies
        self.index_documents(self.documents.clone())?;
        Ok(())
    }

    /// Get document by ID
    pub fn get_document(&self, doc_id: &str) -> Option<&Document> {
        self.documents.iter().find(|doc| doc.id == doc_id)
    }

    /// Get index statistics
    pub fn get_stats(&self) -> BM25Stats {
        BM25Stats {
            total_documents: self.documents.len(),
            total_terms: self.document_frequencies.len(),
            average_document_length: self.average_document_length,
            total_term_occurrences: self.term_frequencies
                .values()
                .map(|tf| tf.values().sum::<usize>())
                .sum(),
        }
    }
}

#[async_trait]
impl KeywordRetriever for BM25Retriever {
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<ScoredDocument>> {
        let query_terms = self.tokenize(query);
        if query_terms.is_empty() {
            return Ok(Vec::new());
        }

        let mut scored_docs: Vec<ScoredDocument> = Vec::new();

        for document in &self.documents {
            let score = self.calculate_bm25_score(&document.id, &query_terms);
            if score > 0.0 {
                scored_docs.push(ScoredDocument {
                    document: document.clone(),
                    score,
                });
            }
        }

        // Sort by score (descending)
        scored_docs.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Apply limit
        scored_docs.truncate(limit);

        Ok(scored_docs)
    }
}

/// Statistics about the BM25 index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BM25Stats {
    pub total_documents: usize,
    pub total_terms: usize,
    pub average_document_length: f32,
    pub total_term_occurrences: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Metadata;

    fn create_test_documents() -> Vec<Document> {
        vec![
            Document {
                id: "doc1".to_string(),
                content: "The quick brown fox jumps over the lazy dog".to_string(),
                metadata: Metadata::new(),
                embedding: None,
            },
            Document {
                id: "doc2".to_string(),
                content: "A quick brown fox is very fast and agile".to_string(),
                metadata: Metadata::new(),
                embedding: None,
            },
            Document {
                id: "doc3".to_string(),
                content: "The lazy dog sleeps all day long".to_string(),
                metadata: Metadata::new(),
                embedding: None,
            },
        ]
    }

    #[tokio::test]
    async fn test_bm25_basic_search() {
        let documents = create_test_documents();
        let retriever = BM25Retriever::with_default_config(documents).unwrap();

        let results = retriever.search("quick fox", 10).await.unwrap();
        
        assert!(!results.is_empty());
        assert!(results[0].score > 0.0);
        
        // Documents with "quick" and "fox" should score higher
        let first_result = &results[0];
        assert!(first_result.document.content.contains("quick"));
        assert!(first_result.document.content.contains("fox"));
    }

    #[tokio::test]
    async fn test_bm25_no_results() {
        let documents = create_test_documents();
        let retriever = BM25Retriever::with_default_config(documents).unwrap();

        let results = retriever.search("nonexistent term", 10).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_bm25_scoring_order() {
        let documents = create_test_documents();
        let retriever = BM25Retriever::with_default_config(documents).unwrap();

        let results = retriever.search("fox", 10).await.unwrap();
        
        // Should return documents in descending score order
        for i in 1..results.len() {
            assert!(results[i-1].score >= results[i].score);
        }
    }

    #[tokio::test]
    async fn test_bm25_limit() {
        let documents = create_test_documents();
        let retriever = BM25Retriever::with_default_config(documents).unwrap();

        let results = retriever.search("the", 2).await.unwrap();
        assert!(results.len() <= 2);
    }

    #[test]
    fn test_bm25_stats() {
        let documents = create_test_documents();
        let retriever = BM25Retriever::with_default_config(documents).unwrap();

        let stats = retriever.get_stats();
        assert_eq!(stats.total_documents, 3);
        assert!(stats.total_terms > 0);
        assert!(stats.average_document_length > 0.0);
        assert!(stats.total_term_occurrences > 0);
    }
}
