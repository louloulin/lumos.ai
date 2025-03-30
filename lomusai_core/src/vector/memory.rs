use std::collections::HashMap;
use std::sync::RwLock;
use async_trait::async_trait;
use uuid::Uuid;
use serde_json::Value;
use float_cmp::{approx_eq, ApproxEq};

use super::{VectorStorage, IndexStats, QueryResult, SimilarityMetric};
use super::filter::{FilterCondition, FilterInterpreter};
use crate::error::{Error, Result};

/// Vector index information
#[derive(Debug)]
struct VectorIndex {
    /// Number of dimensions
    dimension: usize,
    /// Similarity metric
    metric: SimilarityMetric,
    /// Vectors stored in the index
    vectors: HashMap<String, Vec<f32>>,
    /// Metadata associated with vectors
    metadata: HashMap<String, HashMap<String, Value>>,
}

/// In-memory vector storage implementation
pub struct MemoryVectorStorage {
    /// Indexes stored in memory
    indexes: RwLock<HashMap<String, VectorIndex>>,
    /// Filter interpreter
    filter_interpreter: FilterInterpreter,
}

impl MemoryVectorStorage {
    /// Create a new in-memory vector storage
    pub fn new(dimensions: usize, capacity: Option<usize>) -> Self {
        let indexes = if let Some(cap) = capacity {
            RwLock::new(HashMap::with_capacity(cap))
        } else {
            RwLock::new(HashMap::new())
        };
        
        let result = Self {
            indexes,
            filter_interpreter: FilterInterpreter::new(),
        };
        
        // Create a default index
        if let Ok(mut indexes) = result.indexes.write() {
            indexes.insert("default".to_string(), VectorIndex {
                dimension: dimensions,
                metric: SimilarityMetric::Cosine,
                vectors: HashMap::new(),
                metadata: HashMap::new(),
            });
        }
        
        result
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// Calculate Euclidean distance between two vectors
    fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y) * (x - y))
            .sum::<f32>()
            .sqrt()
    }

    /// Calculate dot product between two vectors
    fn dot_product(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// Calculate similarity score based on metric
    fn calculate_similarity(&self, a: &[f32], b: &[f32], metric: SimilarityMetric) -> f32 {
        match metric {
            SimilarityMetric::Cosine => Self::cosine_similarity(a, b),
            SimilarityMetric::Euclidean => {
                let dist = Self::euclidean_distance(a, b);
                1.0 / (1.0 + dist) // Convert distance to similarity score
            }
            SimilarityMetric::DotProduct => Self::dot_product(a, b),
        }
    }
}

#[async_trait]
impl VectorStorage for MemoryVectorStorage {
    async fn create_index(
        &self,
        index_name: &str,
        dimension: usize,
        metric: Option<SimilarityMetric>,
    ) -> Result<()> {
        let mut indexes = self.indexes.write().map_err(|_| Error::Storage("Failed to acquire write lock".into()))?;
        
        if indexes.contains_key(index_name) {
            return Err(Error::Storage(format!("Index {} already exists", index_name)));
        }

        indexes.insert(index_name.to_string(), VectorIndex {
            dimension,
            metric: metric.unwrap_or(SimilarityMetric::Cosine),
            vectors: HashMap::new(),
            metadata: HashMap::new(),
        });

        Ok(())
    }

    async fn list_indexes(&self) -> Result<Vec<String>> {
        let indexes = self.indexes.read().map_err(|_| Error::Storage("Failed to acquire read lock".into()))?;
        Ok(indexes.keys().cloned().collect())
    }

    async fn describe_index(&self, index_name: &str) -> Result<IndexStats> {
        let indexes = self.indexes.read().map_err(|_| Error::Storage("Failed to acquire read lock".into()))?;
        
        let index = indexes.get(index_name)
            .ok_or_else(|| Error::Storage(format!("Index {} not found", index_name)))?;

        Ok(IndexStats {
            dimension: index.dimension,
            count: index.vectors.len(),
            metric: index.metric,
        })
    }

    async fn delete_index(&self, index_name: &str) -> Result<()> {
        let mut indexes = self.indexes.write().map_err(|_| Error::Storage("Failed to acquire write lock".into()))?;
        indexes.remove(index_name);
        Ok(())
    }

    async fn upsert(
        &self,
        index_name: &str,
        vectors: Vec<Vec<f32>>,
        ids: Option<Vec<String>>,
        metadata: Option<Vec<HashMap<String, serde_json::Value>>>,
    ) -> Result<Vec<String>> {
        let mut indexes = self.indexes.write().map_err(|_| Error::Storage("Failed to acquire write lock".into()))?;
        
        let index = indexes.get_mut(index_name)
            .ok_or_else(|| Error::Storage(format!("Index {} not found", index_name)))?;

        let vector_ids = ids.unwrap_or_else(|| vectors.iter().map(|_| Uuid::new_v4().to_string()).collect());

        if vector_ids.len() != vectors.len() {
            return Err(Error::Storage("Number of IDs must match number of vectors".into()));
        }

        if let Some(meta) = &metadata {
            if meta.len() != vectors.len() {
                return Err(Error::Storage("Number of metadata entries must match number of vectors".into()));
            }
        }

        for (i, (id, vector)) in vector_ids.iter().zip(vectors.iter()).enumerate() {
            if vector.len() != index.dimension {
                return Err(Error::Storage(format!(
                    "Vector dimension mismatch: expected {}, got {}",
                    index.dimension,
                    vector.len()
                )));
            }

            index.vectors.insert(id.clone(), vector.clone());
            
            if let Some(meta) = metadata.as_ref().and_then(|m| m.get(i)) {
                index.metadata.insert(id.clone(), meta.clone());
            }
        }

        Ok(vector_ids)
    }

    async fn query(
        &self,
        index_name: &str,
        query_vector: Vec<f32>,
        top_k: usize,
        filter: Option<FilterCondition>,
        include_vectors: bool,
    ) -> Result<Vec<QueryResult>> {
        let indexes = self.indexes.read().map_err(|_| Error::Storage("Failed to acquire read lock".into()))?;
        
        let index = indexes.get(index_name)
            .ok_or_else(|| Error::Storage(format!("Index {} not found", index_name)))?;

        if query_vector.len() != index.dimension {
            return Err(Error::Storage(format!(
                "Query vector dimension mismatch: expected {}, got {}",
                index.dimension,
                query_vector.len()
            )));
        }

        let mut results: Vec<QueryResult> = index.vectors.iter()
            .filter(|(id, _)| {
                if let Some(filter) = &filter {
                    if let Some(metadata) = index.metadata.get(*id) {
                        self.filter_interpreter.evaluate(filter, metadata)
                    } else {
                        false
                    }
                } else {
                    true
                }
            })
            .map(|(id, vector)| {
                let score = self.calculate_similarity(&query_vector, vector, index.metric);
                QueryResult {
                    id: id.clone(),
                    score,
                    vector: if include_vectors { Some(vector.clone()) } else { None },
                    metadata: index.metadata.get(id).cloned(),
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);

        Ok(results)
    }

    async fn update_by_id(
        &self,
        index_name: &str,
        id: &str,
        vector: Option<Vec<f32>>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<()> {
        let mut indexes = self.indexes.write().map_err(|_| Error::Storage("Failed to acquire write lock".into()))?;
        
        let index = indexes.get_mut(index_name)
            .ok_or_else(|| Error::Storage(format!("Index {} not found", index_name)))?;

        if !index.vectors.contains_key(id) {
            return Err(Error::Storage(format!("Vector with ID {} not found", id)));
        }

        if let Some(new_vector) = vector {
            if new_vector.len() != index.dimension {
                return Err(Error::Storage(format!(
                    "Vector dimension mismatch: expected {}, got {}",
                    index.dimension,
                    new_vector.len()
                )));
            }
            index.vectors.insert(id.to_string(), new_vector);
        }

        if let Some(meta) = metadata {
            index.metadata.insert(id.to_string(), meta);
        }

        Ok(())
    }

    async fn delete_by_id(&self, index_name: &str, id: &str) -> Result<()> {
        let mut indexes = self.indexes.write().map_err(|_| Error::Storage("Failed to acquire write lock".into()))?;
        
        let index = indexes.get_mut(index_name)
            .ok_or_else(|| Error::Storage(format!("Index {} not found", index_name)))?;

        index.vectors.remove(id);
        index.metadata.remove(id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLOAT_EPSILON: f32 = 1e-6;

    #[tokio::test]
    async fn test_vector_operations() {
        let storage = MemoryVectorStorage::new(3, None);
        
        // Create index with default metric (Cosine)
        storage.create_index("test_index", 3, None).await.unwrap();
        
        // Insert vectors
        let vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
        ];
        let metadata = Some(vec![
            HashMap::from([("type".to_string(), serde_json::json!("A"))]),
            HashMap::from([("type".to_string(), serde_json::json!("B"))]),
        ]);
        
        let ids = storage.upsert("test_index", vectors, None, metadata).await.unwrap();
        assert_eq!(ids.len(), 2);
        
        // Query vectors
        let results = storage.query(
            "test_index",
            vec![1.0, 0.0, 0.0],
            2,
            Some(FilterCondition::Eq("type".to_string(), serde_json::json!("A"))),
            true,
        ).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert!(approx_eq!(f32, results[0].score, 1.0, epsilon = FLOAT_EPSILON));
        assert_eq!(results[0].vector.as_ref().unwrap(), &vec![1.0, 0.0, 0.0]);
        
        // Update vector
        storage.update_by_id(
            "test_index",
            &ids[0],
            Some(vec![0.0, 0.0, 1.0]),
            None,
        ).await.unwrap();
        
        // Delete vector
        storage.delete_by_id("test_index", &ids[0]).await.unwrap();
        
        // Check index stats
        let stats = storage.describe_index("test_index").await.unwrap();
        assert_eq!(stats.dimension, 3);
        assert_eq!(stats.count, 1);
        assert_eq!(stats.metric, SimilarityMetric::Cosine);
        
        // Delete index
        storage.delete_index("test_index").await.unwrap();
        
        let indexes = storage.list_indexes().await.unwrap();
        assert!(indexes.is_empty());
    }

    #[tokio::test]
    async fn test_similarity_metrics() {
        let storage = MemoryVectorStorage::new(3, None);
        let test_vectors = vec![
            vec![1.0, 0.0, 0.0],  // Vector A
            vec![0.0, 1.0, 0.0],  // Vector B
            vec![1.0, 1.0, 0.0],  // Vector C
        ];
        
        // Test Cosine similarity
        storage.create_index("cosine_index", 3, Some(SimilarityMetric::Cosine)).await.unwrap();
        storage.upsert("cosine_index", test_vectors.clone(), None, None).await.unwrap();
        let cosine_results = storage.query(
            "cosine_index",
            vec![1.0, 1.0, 0.0],  // Query vector (same as C)
            3,
            None,
            false,
        ).await.unwrap();
        assert!(approx_eq!(f32, cosine_results[0].score, 1.0, epsilon = FLOAT_EPSILON)); // C should match perfectly
        assert!(cosine_results[1].score < 1.0); // A and B should have lower scores
        
        // Test Euclidean similarity
        storage.create_index("euclidean_index", 3, Some(SimilarityMetric::Euclidean)).await.unwrap();
        storage.upsert("euclidean_index", test_vectors.clone(), None, None).await.unwrap();
        let euclidean_results = storage.query(
            "euclidean_index",
            vec![1.0, 1.0, 0.0],  // Query vector (same as C)
            3,
            None,
            false,
        ).await.unwrap();
        assert!(approx_eq!(f32, euclidean_results[0].score, 1.0, epsilon = FLOAT_EPSILON)); // C should match perfectly
        assert!(euclidean_results[1].score < 1.0); // A and B should have lower scores
        
        // Test Dot product similarity
        storage.create_index("dot_index", 3, Some(SimilarityMetric::DotProduct)).await.unwrap();
        storage.upsert("dot_index", test_vectors.clone(), None, None).await.unwrap();
        let dot_results = storage.query(
            "dot_index",
            vec![1.0, 1.0, 0.0],  // Query vector (same as C)
            3,
            None,
            false,
        ).await.unwrap();
        assert!(approx_eq!(f32, dot_results[0].score, 2.0, epsilon = FLOAT_EPSILON)); // C should have dot product of 2
        assert!(dot_results[1].score < 2.0); // A and B should have lower scores
    }
} 