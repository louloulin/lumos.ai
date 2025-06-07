//! Retriever module for RAG systems
//!
//! This module provides functionality for storing and retrieving documents based on queries.

use async_trait::async_trait;
use crate::{
    types::{RetrievalRequest, RetrievalResult},
    error::Result,
};

/// Trait for document retrieval systems
#[async_trait]
pub trait Retriever: Send + Sync {
    /// Retrieve documents based on a query
    async fn retrieve(&self, request: &RetrievalRequest) -> Result<RetrievalResult>;
}

mod vector_store;
mod in_memory;
pub mod hybrid;
pub mod bm25;

pub use vector_store::VectorStore;
pub use in_memory::InMemoryVectorStore;
pub use hybrid::{HybridRetriever, HybridSearchConfig, RerankStrategy, KeywordRetriever};
pub use bm25::{BM25Retriever, BM25Config, BM25Stats};