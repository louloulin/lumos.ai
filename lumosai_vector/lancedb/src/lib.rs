//! # LumosAI LanceDB Integration
//!
//! This crate provides LanceDB integration for LumosAI vector storage,
//! offering high-performance columnar vector database capabilities.
//!
//! ## Features
//!
//! - **High Performance**: Columnar storage optimized for vector operations
//! - **ACID Transactions**: Full transaction support with consistency guarantees
//! - **Rich Indexing**: Multiple index types (IVF, HNSW, LSH)
//! - **Metadata Filtering**: Complex filtering with SQL-like expressions
//! - **Versioning**: Built-in dataset versioning and time travel
//! - **Compression**: Advanced compression for storage efficiency
//!
//! ## Quick Start
//!
//! ```rust
//! use lumosai_vector_lancedb::{LanceDbStorage, LanceDbConfig};
//! use lumosai_vector_core::traits::VectorStorage;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create LanceDB storage
//!     let config = LanceDbConfig::new("./lance_data");
//!     let storage = LanceDbStorage::new(config).await?;
//!     
//!     // Create an index
//!     let index_config = IndexConfig::new("documents", 384)
//!         .with_metric(SimilarityMetric::Cosine);
//!     storage.create_index(index_config).await?;
//!     
//!     // Insert documents
//!     let docs = vec![
//!         Document::new("doc1", "Hello world")
//!             .with_embedding(vec![0.1; 384])
//!             .with_metadata("category", "greeting"),
//!     ];
//!     storage.upsert_documents("documents", docs).await?;
//!     
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;

pub mod storage;
pub mod config;
pub mod error;
pub mod conversion;
pub mod index;

pub use storage::LanceDbStorage;
pub use config::{LanceDbConfig, LanceDbConfigBuilder};
pub use error::{LanceDbError, LanceDbResult};

// Re-export core types for convenience
pub use lumosai_vector_core::types::*;
pub use lumosai_vector_core::traits::VectorStorage;

/// LanceDB client for managing connections and databases
#[derive(Clone)]
pub struct LanceDbClient {
    /// Database connection
    db: Arc<lancedb::Connection>,
    
    /// Configuration
    config: LanceDbConfig,
}

impl LanceDbClient {
    /// Create a new LanceDB client
    pub async fn new(config: LanceDbConfig) -> LanceDbResult<Self> {
        let db = lancedb::connect(&config.uri).await
            .map_err(|e| LanceDbError::Connection(e.to_string()))?;
        
        Ok(Self {
            db: Arc::new(db),
            config,
        })
    }
    
    /// Get the database connection
    pub fn connection(&self) -> Arc<lancedb::Connection> {
        self.db.clone()
    }
    
    /// Get the configuration
    pub fn config(&self) -> &LanceDbConfig {
        &self.config
    }
    
    /// List all tables in the database
    pub async fn list_tables(&self) -> LanceDbResult<Vec<String>> {
        let tables = self.db.table_names().await
            .map_err(|e| LanceDbError::Database(e.to_string()))?;
        Ok(tables)
    }
    
    /// Check if a table exists
    pub async fn table_exists(&self, name: &str) -> LanceDbResult<bool> {
        let tables = self.list_tables().await?;
        Ok(tables.contains(&name.to_string()))
    }
    
    /// Drop a table
    pub async fn drop_table(&self, name: &str) -> LanceDbResult<()> {
        self.db.drop_table(name).await
            .map_err(|e| LanceDbError::Database(e.to_string()))?;
        Ok(())
    }
    
    /// Get database statistics
    pub async fn stats(&self) -> LanceDbResult<DatabaseStats> {
        let tables = self.list_tables().await?;
        let mut total_rows = 0;
        let mut total_size = 0;
        
        for table_name in &tables {
            if let Ok(table) = self.db.open_table(table_name).await {
                if let Ok(count) = table.count_rows(None).await {
                    total_rows += count;
                }
                // Note: LanceDB doesn't provide direct size info in current API
                // This would need to be calculated from file system or other means
            }
        }
        
        Ok(DatabaseStats {
            table_count: tables.len(),
            total_rows,
            total_size_bytes: total_size,
            metadata: HashMap::new(),
        })
    }
}

/// Database statistics
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DatabaseStats {
    /// Number of tables
    pub table_count: usize,
    /// Total number of rows across all tables
    pub total_rows: usize,
    /// Total size in bytes
    pub total_size_bytes: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Create a new LanceDB storage instance
pub async fn create_lancedb_storage(uri: &str) -> LanceDbResult<LanceDbStorage> {
    let config = LanceDbConfig::new(uri);
    LanceDbStorage::new(config).await
}

/// Create a new LanceDB storage instance with configuration
pub async fn create_lancedb_storage_with_config(config: LanceDbConfig) -> LanceDbResult<LanceDbStorage> {
    LanceDbStorage::new(config).await
}

/// Utility functions for LanceDB operations
pub mod utils {
    pub use crate::conversion::*;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_client_creation() {
        let temp_dir = TempDir::new().unwrap();
        let uri = format!("file://{}", temp_dir.path().display());
        
        let config = LanceDbConfig::new(&uri);
        let client = LanceDbClient::new(config).await;
        
        assert!(client.is_ok());
    }
    
    #[tokio::test]
    async fn test_table_operations() {
        let temp_dir = TempDir::new().unwrap();
        let uri = format!("file://{}", temp_dir.path().display());
        
        let config = LanceDbConfig::new(&uri);
        let client = LanceDbClient::new(config).await.unwrap();
        
        // Initially no tables
        let tables = client.list_tables().await.unwrap();
        assert!(tables.is_empty());
        
        // Check non-existent table
        let exists = client.table_exists("test_table").await.unwrap();
        assert!(!exists);
    }
    
    #[test]
    fn test_document_schema() {
        let schema = utils::create_document_schema(384);
        assert_eq!(schema.fields().len(), 4);
        assert_eq!(schema.field(0).name(), "id");
        assert_eq!(schema.field(1).name(), "content");
        assert_eq!(schema.field(2).name(), "vector");
        assert_eq!(schema.field(3).name(), "metadata");
    }
}
