//! In-memory vector storage implementation

use std::collections::HashMap;
use std::sync::RwLock;
use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    Result, VectorError, Vector, FilterCondition, QueryResult, 
    IndexStats, SimilarityMetric
};
use super::VectorStorage;

/// Vector index information
#[derive(Debug)]
struct VectorIndex {
    /// Number of dimensions
    dimension: usize,
    /// Similarity metric
    metric: SimilarityMetric,
    /// Vectors stored in the index
    vectors: HashMap<String, Vector>,
    /// Metadata associated with vectors
    metadata: HashMap<String, HashMap<String, serde_json::Value>>,
}

/// In-memory vector storage implementation
#[derive(Debug)]
pub struct MemoryVectorStorage {
    /// Indexes stored in memory
    indexes: RwLock<HashMap<String, VectorIndex>>,
}

impl MemoryVectorStorage {
    /// Create a new in-memory vector storage
    pub async fn new() -> Result<Self> {
        Ok(Self {
            indexes: RwLock::new(HashMap::new()),
        })
    }

    /// Create a new in-memory vector storage with capacity hint
    pub async fn with_capacity(capacity: usize) -> Result<Self> {
        Ok(Self {
            indexes: RwLock::new(HashMap::with_capacity(capacity)),
        })
    }

    /// Normalize a vector to unit length
    fn normalize_vector(vector: &[f32]) -> Vector {
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm == 0.0 {
            return vector.to_vec(); // Return original if norm is zero
        }
        vector.iter().map(|x| x / norm).collect()
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
    fn evaluate_filter(&self, filter: &FilterCondition, metadata: &HashMap<String, serde_json::Value>) -> bool {
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
        let mut indexes = self.indexes.write()
            .map_err(|_| VectorError::internal("Failed to acquire write lock"))?;
        
        if indexes.contains_key(index_name) {
            return Err(VectorError::IndexAlreadyExists(index_name.to_string()));
        }

        indexes.insert(index_name.to_string(), VectorIndex {
            dimension,
            metric: metric.unwrap_or_default(),
            vectors: HashMap::new(),
            metadata: HashMap::new(),
        });

        Ok(())
    }

    async fn list_indexes(&self) -> Result<Vec<String>> {
        let indexes = self.indexes.read()
            .map_err(|_| VectorError::internal("Failed to acquire read lock"))?;
        Ok(indexes.keys().cloned().collect())
    }

    async fn describe_index(&self, index_name: &str) -> Result<IndexStats> {
        let indexes = self.indexes.read()
            .map_err(|_| VectorError::internal("Failed to acquire read lock"))?;
        
        let index = indexes.get(index_name)
            .ok_or_else(|| VectorError::IndexNotFound(index_name.to_string()))?;

        Ok(IndexStats::new(
            index_name.to_string(),
            index.dimension,
            index.metric,
        )
        .with_vector_count(index.vectors.len())
        .with_size_bytes((index.vectors.len() * index.dimension * 4) as u64))
    }

    async fn delete_index(&self, index_name: &str) -> Result<()> {
        let mut indexes = self.indexes.write()
            .map_err(|_| VectorError::internal("Failed to acquire write lock"))?;
        
        if !indexes.contains_key(index_name) {
            return Err(VectorError::IndexNotFound(index_name.to_string()));
        }
        
        indexes.remove(index_name);
        Ok(())
    }

    async fn upsert(
        &self,
        index_name: &str,
        vectors: Vec<Vector>,
        ids: Option<Vec<String>>,
        metadata: Option<Vec<HashMap<String, serde_json::Value>>>,
    ) -> Result<Vec<String>> {
        let mut indexes = self.indexes.write()
            .map_err(|_| VectorError::internal("Failed to acquire write lock"))?;
        
        let index = indexes.get_mut(index_name)
            .ok_or_else(|| VectorError::IndexNotFound(index_name.to_string()))?;

        let vector_ids = ids.unwrap_or_else(|| {
            vectors.iter().map(|_| Uuid::new_v4().to_string()).collect()
        });

        if vector_ids.len() != vectors.len() {
            return Err(VectorError::InvalidOperation(
                "Number of IDs must match number of vectors".to_string()
            ));
        }

        if let Some(meta) = &metadata {
            if meta.len() != vectors.len() {
                return Err(VectorError::InvalidOperation(
                    "Number of metadata entries must match number of vectors".to_string()
                ));
            }
        }

        for (i, (id, vector)) in vector_ids.iter().zip(vectors.iter()).enumerate() {
            if vector.len() != index.dimension {
                return Err(VectorError::DimensionMismatch {
                    expected: index.dimension,
                    actual: vector.len(),
                });
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
        query_vector: Vector,
        top_k: usize,
        filter: Option<FilterCondition>,
        include_vectors: bool,
    ) -> Result<Vec<QueryResult>> {
        let indexes = self.indexes.read()
            .map_err(|_| VectorError::internal("Failed to acquire read lock"))?;
        
        let index = indexes.get(index_name)
            .ok_or_else(|| VectorError::IndexNotFound(index_name.to_string()))?;

        if query_vector.len() != index.dimension {
            return Err(VectorError::DimensionMismatch {
                expected: index.dimension,
                actual: query_vector.len(),
            });
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
                QueryResult::new(id.clone(), score)
                    .with_vector(if include_vectors { vector.clone() } else { vec![] })
                    .with_metadata(index.metadata.get(id).cloned().unwrap_or_default())
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
        vector: Option<Vector>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<()> {
        let mut indexes = self.indexes.write()
            .map_err(|_| VectorError::internal("Failed to acquire write lock"))?;
        
        let index = indexes.get_mut(index_name)
            .ok_or_else(|| VectorError::IndexNotFound(index_name.to_string()))?;

        if !index.vectors.contains_key(id) {
            return Err(VectorError::VectorNotFound(id.to_string()));
        }

        if let Some(new_vector) = vector {
            if new_vector.len() != index.dimension {
                return Err(VectorError::DimensionMismatch {
                    expected: index.dimension,
                    actual: new_vector.len(),
                });
            }
            index.vectors.insert(id.to_string(), new_vector);
        }

        if let Some(meta) = metadata {
            index.metadata.insert(id.to_string(), meta);
        }

        Ok(())
    }

    async fn delete_by_id(&self, index_name: &str, id: &str) -> Result<()> {
        let mut indexes = self.indexes.write()
            .map_err(|_| VectorError::internal("Failed to acquire write lock"))?;
        
        let index = indexes.get_mut(index_name)
            .ok_or_else(|| VectorError::IndexNotFound(index_name.to_string()))?;

        index.vectors.remove(id);
        index.metadata.remove(id);

        Ok(())
    }

    async fn get_by_id(
        &self,
        index_name: &str,
        id: &str,
        include_vector: bool,
    ) -> Result<Option<QueryResult>> {
        let indexes = self.indexes.read()
            .map_err(|_| VectorError::internal("Failed to acquire read lock"))?;
        
        let index = indexes.get(index_name)
            .ok_or_else(|| VectorError::IndexNotFound(index_name.to_string()))?;

        if let Some(vector) = index.vectors.get(id) {
            let result = QueryResult::new(id.to_string(), 1.0)
                .with_vector(if include_vector { vector.clone() } else { vec![] })
                .with_metadata(index.metadata.get(id).cloned().unwrap_or_default());
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}
