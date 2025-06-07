//! Vector storage implementations

use async_trait::async_trait;
use std::collections::HashMap;

use crate::{
    Result, Vector, FilterCondition, QueryResult, IndexStats, SimilarityMetric
};

/// Unified vector storage trait
#[async_trait]
pub trait VectorStorage: Send + Sync {
    /// Create a new vector index
    async fn create_index(
        &self,
        index_name: &str,
        dimension: usize,
        metric: Option<SimilarityMetric>,
    ) -> Result<()>;

    /// List all available indexes
    async fn list_indexes(&self) -> Result<Vec<String>>;

    /// Get index statistics
    async fn describe_index(&self, index_name: &str) -> Result<IndexStats>;

    /// Delete an index
    async fn delete_index(&self, index_name: &str) -> Result<()>;

    /// Insert or update vectors
    async fn upsert(
        &self,
        index_name: &str,
        vectors: Vec<Vector>,
        ids: Option<Vec<String>>,
        metadata: Option<Vec<HashMap<String, serde_json::Value>>>,
    ) -> Result<Vec<String>>;

    /// Query similar vectors
    async fn query(
        &self,
        index_name: &str,
        query_vector: Vector,
        top_k: usize,
        filter: Option<FilterCondition>,
        include_vectors: bool,
    ) -> Result<Vec<QueryResult>>;

    /// Update vector by ID
    async fn update_by_id(
        &self,
        index_name: &str,
        id: &str,
        vector: Option<Vector>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<()>;

    /// Delete vector by ID
    async fn delete_by_id(&self, index_name: &str, id: &str) -> Result<()>;

    /// Get vector by ID
    async fn get_by_id(
        &self,
        index_name: &str,
        id: &str,
        include_vector: bool,
    ) -> Result<Option<QueryResult>>;

    /// Batch delete vectors by IDs
    async fn delete_by_ids(&self, index_name: &str, ids: Vec<String>) -> Result<()> {
        for id in ids {
            self.delete_by_id(index_name, &id).await?;
        }
        Ok(())
    }

    /// Check if index exists
    async fn index_exists(&self, index_name: &str) -> Result<bool> {
        let indexes = self.list_indexes().await?;
        Ok(indexes.contains(&index_name.to_string()))
    }

    /// Get vector count for an index
    async fn vector_count(&self, index_name: &str) -> Result<usize> {
        let stats = self.describe_index(index_name).await?;
        Ok(stats.vector_count)
    }
}

// Storage implementations
#[cfg(feature = "memory")]
pub mod memory;

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[cfg(feature = "qdrant")]
pub mod qdrant;

#[cfg(feature = "mongodb")]
pub mod mongodb;
