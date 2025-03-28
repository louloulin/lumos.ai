//! Embedding generation for RAG systems
//! 
//! This module provides functionality for converting text into vector representations.

mod provider;
mod openai;

pub use provider::EmbeddingProvider;
pub use openai::OpenAIEmbeddingProvider; 