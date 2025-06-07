//! # Lumos Vector PostgreSQL
//!
//! PostgreSQL with pgvector implementation for the lumos-vector-core architecture.
//! 
//! This crate provides a PostgreSQL backend with pgvector extension for vector storage,
//! implementing the unified VectorStorage trait from lumos-vector-core.
//!
//! ## Features
//!
//! - **SQL Integration**: Leverages PostgreSQL's ACID properties
//! - **pgvector Extension**: High-performance vector operations
//! - **Rich Queries**: Complex SQL queries with vector similarity
//! - **Transactions**: Full transaction support
//! - **Indexing**: Multiple vector index types (IVFFlat, HNSW)
//!
//! ## Example
//!
//! ```rust
//! use lumos_vector_postgres::PostgresVectorStorage;
//! use lumos_vector_core::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create PostgreSQL storage
//!     let storage = PostgresVectorStorage::new("postgresql://user:pass@localhost/db").await?;
//!     
//!     // Create an index
//!     let config = IndexConfig::new("documents", 384)
//!         .with_metric(SimilarityMetric::Cosine);
//!     storage.create_index(config).await?;
//!     
//!     // Insert documents
//!     let docs = vec![
//!         Document::new("doc1", "Hello world")
//!             .with_embedding(vec![0.1; 384])
//!             .with_metadata("type", "greeting"),
//!     ];
//!     storage.upsert_documents("documents", docs).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod storage;
pub mod config;
pub mod error;

pub use storage::PostgresVectorStorage;
pub use config::PostgresConfig;
pub use error::{PostgresError, PostgresResult};

// Re-export core types for convenience
pub use lumosai_vector_core::prelude::*;

/// Create a new PostgreSQL vector storage instance
pub async fn create_postgres_storage(database_url: &str) -> Result<PostgresVectorStorage> {
    PostgresVectorStorage::new(database_url).await
}

/// Create a new PostgreSQL vector storage instance with configuration
pub async fn create_postgres_storage_with_config(config: PostgresConfig) -> Result<PostgresVectorStorage> {
    PostgresVectorStorage::with_config(config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_storage() {
        // This test requires a running PostgreSQL instance with pgvector
        // Skip if not available
        if std::env::var("DATABASE_URL").is_err() {
            return;
        }
        
        let url = std::env::var("DATABASE_URL").unwrap();
        let storage = create_postgres_storage(&url).await;
        
        // Should either succeed or fail with connection error
        match storage {
            Ok(_) => println!("Successfully connected to PostgreSQL"),
            Err(e) => println!("Failed to connect to PostgreSQL: {}", e),
        }
    }
}
