//! # Lumos Vector Weaviate
//!
//! Weaviate vector database implementation for the lumos-vector-core architecture.
//! 
//! This crate provides a high-performance Weaviate backend for vector storage,
//! implementing the unified VectorStorage trait from lumos-vector-core.
//!
//! ## Features
//!
//! - **Semantic Search**: Leverages Weaviate's semantic search capabilities
//! - **Schema Management**: Automatic schema creation and management
//! - **Rich Filtering**: Advanced metadata filtering with GraphQL
//! - **Batch Operations**: Efficient bulk insert and update operations
//! - **Multi-tenancy**: Support for Weaviate's multi-tenant features
//!
//! ## Example
//!
//! ```rust
//! use lumos_vector_weaviate::WeaviateVectorStorage;
//! use lumos_vector_core::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create Weaviate storage
//!     let storage = WeaviateVectorStorage::new("http://localhost:8080").await?;
//!     
//!     // Create an index (class in Weaviate)
//!     let config = IndexConfig::new("Documents", 384)
//!         .with_metric(SimilarityMetric::Cosine);
//!     storage.create_index(config).await?;
//!     
//!     // Insert documents
//!     let docs = vec![
//!         Document::new("doc1", "Hello world")
//!             .with_embedding(vec![0.1; 384])
//!             .with_metadata("type", "greeting"),
//!     ];
//!     storage.upsert_documents("Documents", docs).await?;
//!     
//!     // Search
//!     let request = SearchRequest::new("Documents", vec![0.1; 384])
//!         .with_top_k(5);
//!     let results = storage.search(request).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod storage;
pub mod config;
pub mod filter;
pub mod error;
pub mod schema;

pub use storage::WeaviateVectorStorage;
pub use config::WeaviateConfig;
pub use error::WeaviateError;

// Re-export core types for convenience
pub use lumosai_vector_core::prelude::*;

/// Create a new Weaviate vector storage instance
pub async fn create_weaviate_storage(url: &str) -> Result<WeaviateVectorStorage> {
    WeaviateVectorStorage::new(url).await
}

/// Create a new Weaviate vector storage instance with configuration
pub async fn create_weaviate_storage_with_config(config: WeaviateConfig) -> Result<WeaviateVectorStorage> {
    WeaviateVectorStorage::with_config(config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_storage() {
        // This test requires a running Weaviate instance
        // Skip if not available
        if std::env::var("WEAVIATE_URL").is_err() {
            return;
        }
        
        let url = std::env::var("WEAVIATE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
        let storage = create_weaviate_storage(&url).await;
        
        // Should either succeed or fail with connection error
        match storage {
            Ok(_) => println!("Successfully connected to Weaviate"),
            Err(e) => println!("Failed to connect to Weaviate: {}", e),
        }
    }
}
