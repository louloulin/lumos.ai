use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::Value;

use crate::error::{Error, Result};
use super::{VectorStorage, IndexStats, QueryResult, SimilarityMetric, FilterCondition};

/// Adapter to bridge lumosai_stores::VectorStore to lumosai_core::VectorStorage
pub struct VectorStorageAdapter {
    store: Box<dyn lumosai_stores::VectorStore>,
}

impl VectorStorageAdapter {
    /// Create a new adapter
    pub fn new(store: Box<dyn lumosai_stores::VectorStore>) -> Self {
        Self { store }
    }
    
    /// Convert SimilarityMetric to string
    fn metric_to_string(metric: SimilarityMetric) -> String {
        match metric {
            SimilarityMetric::Cosine => "cosine".to_string(),
            SimilarityMetric::Euclidean => "euclidean".to_string(),
            SimilarityMetric::DotProduct => "dotproduct".to_string(),
        }
    }
    
    /// Convert string to SimilarityMetric
    fn string_to_metric(s: &str) -> SimilarityMetric {
        match s.to_lowercase().as_str() {
            "euclidean" => SimilarityMetric::Euclidean,
            "dotproduct" | "dot" => SimilarityMetric::DotProduct,
            _ => SimilarityMetric::Cosine,
        }
    }
    
    /// Convert FilterCondition to VectorFilter
    fn convert_filter(filter: FilterCondition) -> Option<lumosai_stores::vector::VectorFilter> {
        match filter {
            FilterCondition::Eq(field, value) => {
                Some(lumosai_stores::vector::VectorFilter::Field(
                    HashMap::from([(field, lumosai_stores::vector::FieldCondition::Value(value))])
                ))
            },
            FilterCondition::Ne(field, value) => {
                Some(lumosai_stores::vector::VectorFilter::Field(
                    HashMap::from([(field, lumosai_stores::vector::FieldCondition::Operator(
                        HashMap::from([("$ne".to_string(), value)])
                    ))])
                ))
            },
            FilterCondition::Gt(field, value) => {
                Some(lumosai_stores::vector::VectorFilter::Field(
                    HashMap::from([(field, lumosai_stores::vector::FieldCondition::Operator(
                        HashMap::from([("$gt".to_string(), value)])
                    ))])
                ))
            },
            FilterCondition::Lt(field, value) => {
                Some(lumosai_stores::vector::VectorFilter::Field(
                    HashMap::from([(field, lumosai_stores::vector::FieldCondition::Operator(
                        HashMap::from([("$lt".to_string(), value)])
                    ))])
                ))
            },
            FilterCondition::In(field, values) => {
                Some(lumosai_stores::vector::VectorFilter::Field(
                    HashMap::from([(field, lumosai_stores::vector::FieldCondition::Operator(
                        HashMap::from([("$in".to_string(), Value::Array(values))])
                    ))])
                ))
            },
            FilterCondition::And(conditions) => {
                let filters: Vec<_> = conditions.into_iter()
                    .filter_map(Self::convert_filter)
                    .collect();
                if filters.is_empty() {
                    None
                } else {
                    Some(lumosai_stores::vector::VectorFilter::And { and: filters })
                }
            },
            FilterCondition::Or(conditions) => {
                let filters: Vec<_> = conditions.into_iter()
                    .filter_map(Self::convert_filter)
                    .collect();
                if filters.is_empty() {
                    None
                } else {
                    Some(lumosai_stores::vector::VectorFilter::Or { or: filters })
                }
            },
        }
    }
}

#[async_trait]
impl VectorStorage for VectorStorageAdapter {
    async fn create_index(
        &self,
        index_name: &str,
        dimension: usize,
        metric: Option<SimilarityMetric>,
    ) -> Result<()> {
        let params = lumosai_stores::vector::CreateIndexParams {
            index_name: index_name.to_string(),
            dimension,
            metric: Self::metric_to_string(metric.unwrap_or(SimilarityMetric::Cosine)),
        };
        
        self.store.create_index(params).await
            .map_err(|e| Error::Storage(e.to_string()))
    }

    async fn list_indexes(&self) -> Result<Vec<String>> {
        self.store.list_indexes().await
            .map_err(|e| Error::Storage(e.to_string()))
    }

    async fn describe_index(&self, index_name: &str) -> Result<IndexStats> {
        let stats = self.store.describe_index(index_name).await
            .map_err(|e| Error::Storage(e.to_string()))?;
            
        Ok(IndexStats {
            dimension: stats.dimension,
            count: stats.count,
            metric: Self::string_to_metric(&stats.metric),
        })
    }

    async fn delete_index(&self, index_name: &str) -> Result<()> {
        self.store.delete_index(index_name).await
            .map_err(|e| Error::Storage(e.to_string()))
    }

    async fn upsert(
        &self,
        index_name: &str,
        vectors: Vec<Vec<f32>>,
        ids: Option<Vec<String>>,
        metadata: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let params = lumosai_stores::vector::UpsertParams {
            index_name: index_name.to_string(),
            vectors,
            metadata: metadata.unwrap_or_default(),
            ids,
        };
        
        self.store.upsert(params).await
            .map_err(|e| Error::Storage(e.to_string()))
    }

    async fn query(
        &self,
        index_name: &str,
        query_vector: Vec<f32>,
        top_k: usize,
        filter: Option<FilterCondition>,
        include_vectors: bool,
    ) -> Result<Vec<QueryResult>> {
        let params = lumosai_stores::vector::QueryParams {
            index_name: index_name.to_string(),
            query_vector,
            top_k,
            filter: filter.and_then(Self::convert_filter),
            include_vector: include_vectors,
        };
        
        let results = self.store.query(params).await
            .map_err(|e| Error::Storage(e.to_string()))?;
            
        Ok(results.into_iter().map(|r| QueryResult {
            id: r.id,
            score: r.score,
            metadata: r.metadata,
            vector: r.vector,
        }).collect())
    }

    async fn update_by_id(
        &self,
        index_name: &str,
        id: &str,
        vector: Option<Vec<f32>>,
        metadata: Option<HashMap<String, Value>>,
    ) -> Result<()> {
        self.store.update_vector_by_id(index_name, id, vector, metadata).await
            .map_err(|e| Error::Storage(e.to_string()))
    }

    async fn delete_by_id(&self, index_name: &str, id: &str) -> Result<()> {
        self.store.delete_vectors(index_name, &[id.to_string()]).await
            .map_err(|e| Error::Storage(e.to_string()))
    }
}
