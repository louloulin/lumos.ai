//! # Lumosai RAG (Retrieval Augmented Generation)
//!
//! This module provides functionality for implementing Retrieval Augmented Generation
//! systems, which enhance LLM outputs with relevant information retrieved from
//! document collections.
//!
//! The main components are:
//! - Document processing: loading, parsing, and chunking documents
//! - Embedding generation: converting text to vector representations
//! - Retrieval: storing and retrieving relevant documents based on queries

pub mod context;
pub mod document;
pub mod embedding;
pub mod error;
pub mod pipeline;
pub mod retriever;
pub mod types;

// Add missing modules for compatibility
pub mod chunking {
    pub use crate::document::chunker::*;
}

pub mod retrieval {
    pub use crate::retriever::*;
}

pub use error::RagError;
pub use pipeline::{RagPipeline, RagPipelineBuilder};
pub use types::*;
