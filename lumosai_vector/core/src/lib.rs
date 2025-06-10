//! # Lumos Vector Core
//!
//! Core abstractions and traits for the Lumos vector storage system.
//! This crate provides the foundational types and interfaces that all
//! vector storage implementations must follow.
//!
//! ## Design Philosophy
//!
//! This crate is inspired by the excellent designs of:
//! - **Rig**: Strong typing and trait-based abstractions
//! - **Mastra**: Declarative configuration and modularity
//! - **Lumos**: Enterprise features and performance
//!
//! ## Core Concepts
//!
//! - **VectorStorage**: The main trait for vector storage backends
//! - **EmbeddingModel**: Trait for embedding generation
//! - **Document**: Unified document representation with embedding support
//! - **SearchRequest/Response**: Structured query interface
//!
//! ## Example
//!
//! ```rust
//! use lumosai_vector_core::prelude::*;
//! use std::collections::HashMap;
//!
//! // Create a document with embedding support
//! let doc = Document::new("doc1", "This is a test document")
//!     .with_embedding(vec![0.1, 0.2, 0.3])
//!     .with_metadata("type", "article");
//!
//! // Create an index configuration
//! let config = IndexConfig::new("test_index", 384)
//!     .with_metric(SimilarityMetric::Cosine);
//! ```
#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
#![allow(non_camel_case_types, ambiguous_glob_reexports, hidden_glob_reexports)]
#![allow(unexpected_cfgs, unused_assignments)]

pub mod error;
pub mod types;
pub mod traits;
pub mod config;
pub mod performance;

#[cfg(test)]
mod tests;

// Re-export core types for convenience
pub use error::{VectorError, Result};
pub use types::*;
pub use traits::*;
pub use config::*;
pub use performance::*;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::error::{VectorError, Result};
    pub use crate::types::*;
    pub use crate::traits::*;
    pub use crate::config::*;
    pub use crate::performance::*;
}
