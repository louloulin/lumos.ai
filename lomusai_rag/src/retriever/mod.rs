//! Retriever module for RAG systems
//! 
//! This module provides functionality for storing and retrieving documents based on queries.

mod vector_store;
mod in_memory;

pub use vector_store::VectorStore;
pub use in_memory::InMemoryVectorStore; 