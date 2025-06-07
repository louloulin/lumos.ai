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

pub mod document;
pub mod embedding;
pub mod retriever;
pub mod pipeline;
pub mod types;
pub mod error;

pub use error::RagError;
pub use types::*;
pub use pipeline::{RagPipeline, RagPipelineBuilder};