//! # Lumosai Vector Storage System
//!
//! A unified, high-performance vector storage system for Lumos.ai that provides
//! a consistent interface across multiple storage backends.
//!
//! ## Features
//!
//! - **Unified Interface**: Single API for all vector storage backends
//! - **Multiple Backends**: Memory, Qdrant, PostgreSQL, MongoDB, and more
//! - **High Performance**: Optimized for speed and scalability
//! - **Type Safety**: Strong typing with comprehensive error handling
//! - **Async/Await**: Full async support with tokio
//! - **Extensible**: Easy to add new storage backends
//!
//! ## Quick Start
//!
//! ```rust
//! use lumosai_vector::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> lumosai_vector_core::Result<()> {
//!     // Create a memory storage instance
//!     let storage = lumosai_vector::memory::MemoryVectorStorage::new().await?;
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
//!     // Search
//!     let request = SearchRequest::new("documents", vec![0.1; 384])
//!         .with_top_k(5);
//!     let results = storage.search(request).await?;
//!
//!     println!("Found {} results", results.results.len());
//!     Ok(())
//! }
//! ```
//!
//! ## Storage Backends
//!
//! ### Memory Storage
//! Fast in-memory storage for development and testing:
//! ```rust
//! # use lumosai_vector::memory::MemoryVectorStorage;
//! # #[tokio::main]
//! # async fn main() -> lumosai_vector_core::Result<()> {
//! let storage = MemoryVectorStorage::new().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Qdrant Storage
//! High-performance vector database (requires `qdrant` feature):
//! ```rust,ignore
//! use lumosai_vector::qdrant::QdrantVectorStorage;
//!
//! let storage = QdrantVectorStorage::new("http://localhost:6334").await?;
//! ```
//!
//! ### PostgreSQL Storage
//! SQL database with pgvector extension (requires `postgres` feature):
//! ```rust,ignore
//! use lumosai_vector::postgres::PostgresVectorStorage;
//!
//! let storage = PostgresVectorStorage::new("postgresql://user:pass@localhost/db").await?;
//! ```
//!
//! ## Architecture
//!
//! The system is built on a layered architecture:
//!
//! 1. **Core Layer** (`lumosai-vector-core`): Defines traits and types
//! 2. **Implementation Layer**: Specific storage backends
//! 3. **Unified API**: This crate provides a single entry point
//!
//! All storage backends implement the `VectorStorage` trait, ensuring
//! consistent behavior and easy swapping between implementations.

// Re-export core types and traits
pub use lumosai_vector_core::prelude::*;

// Re-export storage implementations
#[cfg(feature = "memory")]
pub use lumosai_vector_memory as memory;

// TODO: Re-enable when API compatibility is fixed
// #[cfg(feature = "qdrant")]
// pub use lumosai_vector_qdrant as qdrant;

// TODO: Re-enable when implemented
// #[cfg(feature = "postgres")]
// pub use lumosai_vector_postgres as postgres;

/// Prelude module for convenient imports
pub mod prelude {
    pub use lumosai_vector_core::prelude::*;

    #[cfg(feature = "memory")]
    pub use crate::memory::MemoryVectorStorage;

    // TODO: Re-enable when API compatibility is fixed
    // #[cfg(feature = "qdrant")]
    // pub use crate::qdrant::QdrantVectorStorage;

    // TODO: Re-enable when implemented
    // #[cfg(feature = "postgres")]
    // pub use crate::postgres::PostgresVectorStorage;
}

/// Utility functions for working with vector storage
pub mod utils {
    use crate::prelude::*;
    
    /// Create a memory storage instance with default configuration
    #[cfg(feature = "memory")]
    pub async fn create_memory_storage() -> Result<crate::memory::MemoryVectorStorage> {
        crate::memory::MemoryVectorStorage::new().await
    }
    
    // TODO: Re-enable when API compatibility is fixed
    // /// Create a Qdrant storage instance
    // #[cfg(feature = "qdrant")]
    // pub async fn create_qdrant_storage(url: &str) -> Result<crate::qdrant::QdrantVectorStorage> {
    //     crate::qdrant::QdrantVectorStorage::new(url).await
    // }

    // TODO: Re-enable when implemented
    // /// Create a PostgreSQL storage instance
    // #[cfg(feature = "postgres")]
    // pub async fn create_postgres_storage(database_url: &str) -> Result<crate::postgres::PostgresVectorStorage> {
    //     crate::postgres::PostgresVectorStorage::new(database_url).await
    // }
    
    /// Auto-detect and create the best available storage backend
    /// Returns a memory storage instance as the default implementation
    #[cfg(feature = "memory")]
    pub async fn create_auto_storage() -> Result<crate::memory::MemoryVectorStorage> {
        // For now, just return memory storage
        // In the future, we can add auto-detection logic
        create_memory_storage().await
    }

    /// Create the best available storage backend based on environment
    #[cfg(feature = "memory")]
    pub async fn create_best_available_storage() -> Result<crate::memory::MemoryVectorStorage> {
        // Try different backends in order of preference

        // For now, we only have memory storage fully implemented
        // TODO: Add Qdrant and PostgreSQL detection when they're ready

        // #[cfg(feature = "qdrant")]
        // {
        //     if let Ok(storage) = create_qdrant_storage("http://localhost:6334").await {
        //         return Ok(storage);
        //     }
        // }

        // #[cfg(feature = "postgres")]
        // {
        //     if let Ok(database_url) = std::env::var("DATABASE_URL") {
        //         if let Ok(storage) = create_postgres_storage(&database_url).await {
        //             return Ok(storage);
        //         }
        //     }
        // }

        // Fallback to memory storage
        create_memory_storage().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_storage_integration() {
        let storage = utils::create_memory_storage().await.unwrap();
        
        // Create index
        let config = IndexConfig::new("test", 3)
            .with_metric(SimilarityMetric::Cosine);
        storage.create_index(config).await.unwrap();
        
        // Insert document
        let doc = Document::new("test1", "test content")
            .with_embedding(vec![1.0, 0.0, 0.0])
            .with_metadata("category", "test");
        
        storage.upsert_documents("test", vec![doc]).await.unwrap();
        
        // Search
        let request = SearchRequest::new("test", vec![1.0, 0.0, 0.0])
            .with_top_k(1);
        let results = storage.search(request).await.unwrap();
        
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].id, "test1");
    }
    
    #[tokio::test]
    #[cfg(feature = "memory")]
    async fn test_auto_storage_creation() {
        let storage = utils::create_auto_storage().await.unwrap();

        // Should be able to perform basic operations
        let info = storage.backend_info();
        assert!(!info.name.is_empty());
        assert!(!info.version.is_empty());
    }
}
