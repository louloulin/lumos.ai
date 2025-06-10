//! # Lumos Vector Memory Storage
//!
//! High-performance in-memory vector storage implementation for Lumos.
//! This implementation provides fast vector operations with support for
//! multiple similarity metrics and complex filtering.
//!
//! ## Features
//!
//! - **High Performance**: Optimized for speed with minimal memory overhead
//! - **Multiple Metrics**: Support for cosine, euclidean, and dot product similarity
//! - **Advanced Filtering**: Complex filter conditions with AND/OR/NOT logic
//! - **Thread Safe**: Full async support with efficient locking
//! - **Memory Efficient**: Configurable capacity and memory management
//!
//! ## Example
//!
//! ```rust
//! use lumosai_vector_memory::MemoryVectorStorage;
//! use lumosai_vector_core::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> lumosai_vector_core::Result<()> {
//!     // Create storage with initial capacity
//!     let storage = MemoryVectorStorage::with_capacity(1000).await?;
//!
//!     // Create an index
//!     let config = IndexConfig::new("documents", 384)
//!         .with_metric(SimilarityMetric::Cosine);
//!     storage.create_index(config).await?;
//!
//!     // Insert documents
//!     let docs = vec![
//!         Document::new("doc1", "Hello world")
//!             .with_embedding(vec![0.1; 384])  // 384-dimensional vector
//!             .with_metadata("type", "greeting"),
//!     ];
//!     storage.upsert_documents("documents", docs).await?;
//!
//!     // Search for similar documents
//!     let request = SearchRequest::new("documents", vec![0.1; 384])
//!         .with_top_k(5)
//!         .with_include_metadata(true);
//!
//!     let response = storage.search(request).await?;
//!     println!("Found {} results", response.results.len());
//!
//!     Ok(())
//! }
//! ```
#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
#![allow(non_camel_case_types, ambiguous_glob_reexports, hidden_glob_reexports)]
#![allow(unexpected_cfgs, unused_assignments)]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use uuid::Uuid;

use lumosai_vector_core::prelude::*;

mod storage;
mod index;
mod utils;

pub use storage::MemoryVectorStorage;

// Type alias for compatibility
pub type MemoryVectorStore = MemoryVectorStorage;
pub use index::MemoryIndex;

/// Memory storage configuration
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Initial capacity for indexes
    pub initial_capacity: usize,
    /// Maximum number of vectors per index
    pub max_vectors_per_index: Option<usize>,
    /// Enable approximate search for large datasets
    pub enable_approximate: bool,
    /// Memory usage threshold for triggering cleanup
    pub memory_threshold_mb: Option<usize>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            initial_capacity: 1000,
            max_vectors_per_index: None,
            enable_approximate: false,
            memory_threshold_mb: None,
        }
    }
}

impl MemoryConfig {
    /// Create a new memory configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set initial capacity
    pub fn with_initial_capacity(mut self, capacity: usize) -> Self {
        self.initial_capacity = capacity;
        self
    }
    
    /// Set maximum vectors per index
    pub fn with_max_vectors(mut self, max_vectors: usize) -> Self {
        self.max_vectors_per_index = Some(max_vectors);
        self
    }
    
    /// Enable approximate search
    pub fn with_approximate_search(mut self, enable: bool) -> Self {
        self.enable_approximate = enable;
        self
    }
    
    /// Set memory threshold
    pub fn with_memory_threshold(mut self, threshold_mb: usize) -> Self {
        self.memory_threshold_mb = Some(threshold_mb);
        self
    }
}
