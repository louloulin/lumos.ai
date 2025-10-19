//! Embedding generation for RAG systems
//!
//! This module provides functionality for converting text into vector representations.

pub mod openai;
pub mod provider;
pub mod zhipu;

pub use openai::OpenAIEmbeddingProvider;
pub use provider::{utils, EmbeddingProvider};
pub use zhipu::{CachedEmbeddingProvider, LocalEmbeddingProvider, ZhipuEmbeddingProvider};
