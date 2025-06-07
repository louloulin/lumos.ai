//! Embedding generation for RAG systems
//! 
//! This module provides functionality for converting text into vector representations.

mod provider;
pub mod openai;

pub use provider::{EmbeddingProvider, utils};
pub use openai::OpenAIEmbeddingProvider; 