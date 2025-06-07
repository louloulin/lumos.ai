//! Core traits for the Lumos vector storage system

use async_trait::async_trait;
use std::collections::HashMap;

use crate::{
    error::{Result, VectorError},
    types::*,
};

/// Core trait for vector storage backends
/// 
/// This trait defines the interface that all vector storage implementations
/// must provide. It's designed to be storage-agnostic and support various
/// backends like in-memory, SQLite, Qdrant, MongoDB, etc.
#[async_trait]
pub trait VectorStorage: Send + Sync {
    /// Storage-specific configuration type
    type Config: Send + Sync;
    
    /// Create a new vector index
    async fn create_index(&self, config: IndexConfig) -> Result<()>;
    
    /// List all available indexes
    async fn list_indexes(&self) -> Result<Vec<String>>;
    
    /// Get information about a specific index
    async fn describe_index(&self, index_name: &str) -> Result<IndexInfo>;
    
    /// Delete an index and all its vectors
    async fn delete_index(&self, index_name: &str) -> Result<()>;
    
    /// Insert or update documents in the index
    async fn upsert_documents(&self, index_name: &str, documents: Vec<Document>) -> Result<Vec<DocumentId>>;
    
    /// Search for similar vectors
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse>;
    
    /// Update a specific document
    async fn update_document(&self, index_name: &str, document: Document) -> Result<()>;
    
    /// Delete documents by IDs
    async fn delete_documents(&self, index_name: &str, ids: Vec<DocumentId>) -> Result<()>;
    
    /// Get documents by IDs
    async fn get_documents(&self, index_name: &str, ids: Vec<DocumentId>, include_vectors: bool) -> Result<Vec<Document>>;
    
    /// Check if the storage backend is healthy
    async fn health_check(&self) -> Result<()>;
    
    /// Get storage backend information
    fn backend_info(&self) -> BackendInfo;
}

/// Trait for embedding models
/// 
/// This trait abstracts over different embedding providers like OpenAI,
/// Ollama, local models, etc.
#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    /// Model-specific configuration type
    type Config: Send + Sync;
    
    /// Generate embedding for a single text
    async fn embed_text(&self, text: &str) -> Result<Vector>;
    
    /// Generate embeddings for multiple texts (batch processing)
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vector>>;
    
    /// Get the dimension of embeddings produced by this model
    fn dimensions(&self) -> usize;
    
    /// Get the model name/identifier
    fn model_name(&self) -> &str;
    
    /// Get the maximum input length supported by the model
    fn max_input_length(&self) -> Option<usize>;
    
    /// Check if the model is available/healthy
    async fn health_check(&self) -> Result<()>;
}

/// Trait for documents that can be embedded
/// 
/// This trait is inspired by Rig's Embed trait and allows automatic
/// embedding generation for structured data.
pub trait Embeddable {
    /// Extract the text content that should be embedded
    fn embed_content(&self) -> String;
    
    /// Get the document ID
    fn document_id(&self) -> DocumentId;
    
    /// Get additional metadata
    fn metadata(&self) -> Metadata {
        HashMap::new()
    }
}

/// Trait for vector similarity calculation
pub trait SimilarityCalculator: Send + Sync {
    /// Calculate similarity between two vectors
    fn calculate_similarity(&self, a: &[f32], b: &[f32]) -> Result<f32>;
    
    /// Get the similarity metric used
    fn metric(&self) -> SimilarityMetric;
    
    /// Normalize a vector if required by the metric
    fn normalize_vector(&self, _vector: &mut [f32]) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }
}

/// Trait for filter evaluation
pub trait FilterEvaluator: Send + Sync {
    /// Evaluate a filter condition against metadata
    fn evaluate(&self, filter: &FilterCondition, metadata: &Metadata) -> Result<bool>;
}

/// Backend information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BackendInfo {
    /// Backend name (e.g., "memory", "sqlite", "qdrant")
    pub name: String,
    /// Backend version
    pub version: String,
    /// Supported features
    pub features: Vec<String>,
    /// Additional backend-specific information
    pub metadata: Metadata,
}

impl BackendInfo {
    /// Create new backend info
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            features: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a supported feature
    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<MetadataValue>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Default similarity calculator implementations
pub mod similarity {
    use super::*;
    
    /// Cosine similarity calculator
    pub struct CosineSimilarity;
    
    impl SimilarityCalculator for CosineSimilarity {
        fn calculate_similarity(&self, a: &[f32], b: &[f32]) -> Result<f32> {
            if a.len() != b.len() {
                return Err(VectorError::dimension_mismatch(a.len(), b.len()));
            }
            
            let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
            let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
            
            if norm_a == 0.0 || norm_b == 0.0 {
                Ok(0.0)
            } else {
                Ok(dot_product / (norm_a * norm_b))
            }
        }
        
        fn metric(&self) -> SimilarityMetric {
            SimilarityMetric::Cosine
        }
    }
    
    /// Euclidean distance calculator (converted to similarity)
    pub struct EuclideanSimilarity;
    
    impl SimilarityCalculator for EuclideanSimilarity {
        fn calculate_similarity(&self, a: &[f32], b: &[f32]) -> Result<f32> {
            if a.len() != b.len() {
                return Err(VectorError::dimension_mismatch(a.len(), b.len()));
            }
            
            let distance: f32 = a.iter()
                .zip(b.iter())
                .map(|(x, y)| (x - y) * (x - y))
                .sum::<f32>()
                .sqrt();
            
            // Convert distance to similarity score
            Ok(1.0 / (1.0 + distance))
        }
        
        fn metric(&self) -> SimilarityMetric {
            SimilarityMetric::Euclidean
        }
    }
    
    /// Dot product similarity calculator
    pub struct DotProductSimilarity;
    
    impl SimilarityCalculator for DotProductSimilarity {
        fn calculate_similarity(&self, a: &[f32], b: &[f32]) -> Result<f32> {
            if a.len() != b.len() {
                return Err(VectorError::dimension_mismatch(a.len(), b.len()));
            }
            
            Ok(a.iter().zip(b.iter()).map(|(x, y)| x * y).sum())
        }
        
        fn metric(&self) -> SimilarityMetric {
            SimilarityMetric::DotProduct
        }
    }
    
    /// Create a similarity calculator for the given metric
    pub fn create_calculator(metric: SimilarityMetric) -> Box<dyn SimilarityCalculator> {
        match metric {
            SimilarityMetric::Cosine => Box::new(CosineSimilarity),
            SimilarityMetric::Euclidean => Box::new(EuclideanSimilarity),
            SimilarityMetric::DotProduct => Box::new(DotProductSimilarity),
            _ => Box::new(CosineSimilarity), // Default fallback
        }
    }
}

/// Default filter evaluator implementation
pub mod filter {
    use super::*;
    
    /// Standard filter evaluator
    pub struct StandardFilterEvaluator;
    
    impl FilterEvaluator for StandardFilterEvaluator {
        fn evaluate(&self, filter: &FilterCondition, metadata: &Metadata) -> Result<bool> {
            match filter {
                FilterCondition::Eq(field, value) => {
                    Ok(metadata.get(field).map_or(false, |v| v == value))
                },
                FilterCondition::Ne(field, value) => {
                    Ok(metadata.get(field).map_or(true, |v| v != value))
                },
                FilterCondition::Gt(field, value) => {
                    self.compare_numeric(metadata, field, value, |a, b| a > b)
                },
                FilterCondition::Gte(field, value) => {
                    self.compare_numeric(metadata, field, value, |a, b| a >= b)
                },
                FilterCondition::Lt(field, value) => {
                    self.compare_numeric(metadata, field, value, |a, b| a < b)
                },
                FilterCondition::Lte(field, value) => {
                    self.compare_numeric(metadata, field, value, |a, b| a <= b)
                },
                FilterCondition::In(field, values) => {
                    Ok(metadata.get(field).map_or(false, |v| values.contains(v)))
                },
                FilterCondition::NotIn(field, values) => {
                    Ok(metadata.get(field).map_or(true, |v| !values.contains(v)))
                },
                FilterCondition::Exists(field) => {
                    Ok(metadata.contains_key(field))
                },
                FilterCondition::NotExists(field) => {
                    Ok(!metadata.contains_key(field))
                },
                FilterCondition::Contains(field, substring) => {
                    self.string_operation(metadata, field, |s| s.contains(substring))
                },
                FilterCondition::StartsWith(field, prefix) => {
                    self.string_operation(metadata, field, |s| s.starts_with(prefix))
                },
                FilterCondition::EndsWith(field, suffix) => {
                    self.string_operation(metadata, field, |s| s.ends_with(suffix))
                },
                FilterCondition::Regex(field, pattern) => {
                    // For now, just do a simple contains check
                    // In a real implementation, you'd use a regex library
                    self.string_operation(metadata, field, |s| s.contains(pattern))
                },
                FilterCondition::And(conditions) => {
                    for condition in conditions {
                        if !self.evaluate(condition, metadata)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                },
                FilterCondition::Or(conditions) => {
                    for condition in conditions {
                        if self.evaluate(condition, metadata)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                },
                FilterCondition::Not(condition) => {
                    Ok(!self.evaluate(condition, metadata)?)
                },
            }
        }
    }
    
    impl StandardFilterEvaluator {
        fn compare_numeric<F>(&self, metadata: &Metadata, field: &str, value: &MetadataValue, op: F) -> Result<bool>
        where
            F: Fn(f64, f64) -> bool,
        {
            let field_value = metadata.get(field);
            let filter_value = self.extract_numeric(value)?;
            
            if let Some(field_val) = field_value {
                let field_numeric = self.extract_numeric(field_val)?;
                Ok(op(field_numeric, filter_value))
            } else {
                Ok(false)
            }
        }
        
        fn string_operation<F>(&self, metadata: &Metadata, field: &str, op: F) -> Result<bool>
        where
            F: Fn(&str) -> bool,
        {
            if let Some(MetadataValue::String(s)) = metadata.get(field) {
                Ok(op(s))
            } else {
                Ok(false)
            }
        }
        
        fn extract_numeric(&self, value: &MetadataValue) -> Result<f64> {
            match value {
                MetadataValue::Integer(i) => Ok(*i as f64),
                MetadataValue::Float(f) => Ok(*f),
                _ => Err(VectorError::InvalidFilter("Expected numeric value".to_string())),
            }
        }
    }
}
