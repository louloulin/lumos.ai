use std::collections::HashMap;
use std::sync::RwLock;
use async_trait::async_trait;
use uuid::Uuid;
use serde_json::Value;

use super::{VectorStorage, IndexStats, QueryResult, SimilarityMetric, FilterCondition};
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

    /// Normalize a vector to unit length
    fn normalize_vector(vector: &[f32]) -> Vec<f32> {
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm == 0.0 {
            return vector.to_vec(); // Return original if norm is zero
        }
        vector.iter().map(|x| x / norm).collect()
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        // For performance, we could pre-normalize vectors and just use dot product
        // But this implementation handles the general case
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
            SimilarityMetric::Cosine => {
                // Option to use pre-normalized vectors for better performance
                let a_norm = Self::normalize_vector(a);
                let b_norm = Self::normalize_vector(b);
                Self::dot_product(&a_norm, &b_norm)
            },
            SimilarityMetric::Euclidean => {
                let dist = Self::euclidean_distance(a, b);
                1.0 / (1.0 + dist) // Convert distance to similarity score
            }
            SimilarityMetric::DotProduct => Self::dot_product(a, b),
        }
    }

    /// Evaluate filter condition against metadata
    fn evaluate_filter(&self, filter: &FilterCondition, metadata: &HashMap<String, Value>) -> bool {
        match filter {
            FilterCondition::Eq(field, value) => {
                metadata.get(field).map_or(false, |v| v == value)
            },
            FilterCondition::Gt(field, value) => {
                if let (Some(field_value), Some(filter_value)) = (metadata.get(field), value.as_f64()) {
                    field_value.as_f64().map_or(false, |v| v > filter_value)
                } else {
                    false
                }
            },
            FilterCondition::Lt(field, value) => {
                if let (Some(field_value), Some(filter_value)) = (metadata.get(field), value.as_f64()) {
                    field_value.as_f64().map_or(false, |v| v < filter_value)
                } else {
                    false
                }
            },
            FilterCondition::In(field, values) => {
                if let Some(field_value) = metadata.get(field) {
                    values.contains(field_value)
                } else {
                    false
                }
            },
            FilterCondition::And(conditions) => {
                conditions.iter().all(|c| self.evaluate_filter(c, metadata))
            },
            FilterCondition::Or(conditions) => {
                conditions.iter().any(|c| self.evaluate_filter(c, metadata))
            },
            FilterCondition::Not(condition) => {
                !self.evaluate_filter(condition, metadata)
            },
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

    /// Insert or update vectors and their metadata
    /// 
    /// Returns a list of vector IDs
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

    /// Query the index for vectors similar to the query vector
    /// 
    /// Returns a list of results sorted by similarity score in descending order
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
                        self.evaluate_filter(filter, metadata)
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

    /// Update a vector and/or its metadata by ID
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
        
        // 在测试开始时确保清理可能存在的旧索引
        if let Ok(indexes) = storage.list_indexes().await {
            for index in indexes {
                let _ = storage.delete_index(&index).await;
            }
        }
        
        // 创建索引
        storage.create_index("test_index", 3, Some(SimilarityMetric::Cosine)).await.unwrap();
        
        // 添加向量
        let vectors = vec![vec![0.1, 0.2, 0.3]];
        let ids = Some(vec!["id1".to_string()]);
        let metadata = Some(vec![HashMap::from([("key".to_string(), serde_json::json!("value"))])]);
        
        storage.upsert("test_index", vectors.clone(), ids, metadata).await.unwrap();
        
        // 查询向量
        let results = storage.query(
            "test_index",
            vec![0.1, 0.2, 0.3],
            5,
            None,
            false
        ).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "id1");
        assert!(approx_eq!(f32, results[0].score, 1.0, epsilon = FLOAT_EPSILON));
        
        // 检查索引统计信息
        let stats = storage.describe_index("test_index").await.unwrap();
        assert_eq!(stats.dimension, 3);
        assert_eq!(stats.count, 1);
        assert_eq!(stats.metric, SimilarityMetric::Cosine);
        
        // 删除索引
        storage.delete_index("test_index").await.unwrap();
        
        // 等待一小段时间确保删除完成
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        let indexes = storage.list_indexes().await.unwrap();
        assert!(indexes.is_empty(), "预期索引列表为空，但包含了: {:?}", indexes);
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