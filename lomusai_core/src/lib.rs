//! Lomusai Core - Rust实现的AI应用框架核心库
//! 
//! 提供了Agent、工作流、工具、LLM接口等核心功能

pub mod agent;
pub mod error;
pub mod llm;
pub mod memory;
pub mod storage;
pub mod telemetry;
pub mod tool;
pub mod types;
pub mod vector;
pub mod workflow;

/// Re-export common types and traits
pub use error::{Error, Result};
pub use llm::{LlmProvider, LlmOptions, Message, Role};
pub use agent::AgentConfig;
pub use memory::Memory;
pub use storage::{Storage, create_memory_storage};
pub use tool::Tool;
pub use vector::{
    VectorStorage, 
    MemoryVectorStorage, 
    SimilarityMetric, 
    VectorStorageConfig, 
    create_vector_storage, 
    create_memory_vector_storage,
    EmbeddingService,
    create_random_embedding
}; 

#[cfg(feature = "vector_sqlite")]
pub use vector::SqliteVectorStorage; 