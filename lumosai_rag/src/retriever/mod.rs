//! Retriever module for RAG systems
//!
//! This module provides functionality for storing and retrieving documents based on queries.

use crate::{
    error::Result,
    types::{RetrievalRequest, RetrievalResult},
};
use async_trait::async_trait;

/// Trait for document retrieval systems
#[async_trait]
pub trait Retriever: Send + Sync {
    /// Retrieve documents based on a query
    async fn retrieve(&self, request: &RetrievalRequest) -> Result<RetrievalResult>;
}

pub mod bm25;
pub mod graph_rag;
pub mod hybrid;
pub mod reranker;
mod in_memory;
mod vector_store;

pub use bm25::{BM25Config, BM25Retriever, BM25Stats};
pub use graph_rag::{Entity, GraphRagConfig, GraphRagRetriever, KnowledgeGraph, Relation};
pub use hybrid::{HybridRetriever, HybridSearchConfig, KeywordRetriever, RerankStrategy};
pub use in_memory::InMemoryVectorStore;
pub use reranker::{
    ChainReranker, CrossEncoderReranker, DiversityReranker, LLMReranker, Reranker,
};
pub use vector_store::VectorStore;
