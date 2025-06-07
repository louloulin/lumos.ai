//! # Lumos Vector Storage
//!
//! A unified vector storage abstraction layer for Lumos.ai that provides:
//! - Multiple storage backends (memory, SQLite, Qdrant, MongoDB)
//! - Unified API for vector operations
//! - Flexible filtering and querying
//! - High performance and scalability
//!
//! ## Quick Start
//!
//! ```rust
//! use lumos_vector::{VectorStorage, MemoryVectorStorage, SimilarityMetric};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a memory-based vector storage
//!     let storage = MemoryVectorStorage::new().await?;
//!     
//!     // Create an index
//!     storage.create_index("my_index", 384, Some(SimilarityMetric::Cosine)).await?;
//!     
//!     // Insert vectors
//!     let vectors = vec![vec![0.1; 384]];
//!     let ids = storage.upsert("my_index", vectors, None, None).await?;
//!
//!     // Query similar vectors
//!     let query_vector = vec![0.1; 384];
//!     let results = storage.query("my_index", query_vector, 10, None, false).await?;
//!     
//!     println!("Found {} similar vectors", results.len());
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod types;
pub mod storage;

// Re-export main types
pub use error::{VectorError, Result};
pub use types::{
    Vector, IndexConfig, FilterCondition, SearchParams, 
    QueryResult, IndexStats, SimilarityMetric
};
pub use storage::VectorStorage;

// Re-export storage implementations
#[cfg(feature = "memory")]
pub use storage::memory::MemoryVectorStorage;

#[cfg(feature = "sqlite")]
pub use storage::sqlite::SqliteVectorStorage;

#[cfg(feature = "qdrant")]
pub use storage::qdrant::QdrantVectorStorage;

#[cfg(feature = "mongodb")]
pub use storage::mongodb::MongoVectorStorage;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        VectorStorage, VectorError, Result,
        Vector, IndexConfig, FilterCondition, SearchParams,
        QueryResult, IndexStats, SimilarityMetric,
    };
    
    #[cfg(feature = "memory")]
    pub use crate::MemoryVectorStorage;
    
    #[cfg(feature = "sqlite")]
    pub use crate::SqliteVectorStorage;
    
    #[cfg(feature = "qdrant")]
    pub use crate::QdrantVectorStorage;
    
    #[cfg(feature = "mongodb")]
    pub use crate::MongoVectorStorage;
}
