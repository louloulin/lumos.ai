//! Document processing for RAG systems
//!
//! This module provides functionality for loading, parsing, and chunking documents.

pub mod chunker;
mod loader;
mod parser;
pub mod semantic_chunker;

pub use chunker::{DocumentChunker, EnhancedChunker, TextChunker};
pub use loader::{DocumentLoader, FileLoader};
pub use parser::{DocumentParser, MarkdownParser, TextParser};
pub use semantic_chunker::{AdaptiveChunker, SemanticChunker, SmartChunker};
