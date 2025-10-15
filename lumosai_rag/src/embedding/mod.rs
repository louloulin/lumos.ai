//! Embedding generation for RAG systems
//!
//! This module provides functionality for converting text into vector representations.

pub mod openai;
mod provider;

pub use openai::OpenAIEmbeddingProvider;
pub use provider::{utils, EmbeddingProvider};
