//! Lumosai Core - Rust实现的AI应用框架核心库
//! 
//! 提供了Agent、工作流、工具、LLM接口等核心功能

pub mod agent;
pub mod base;
pub mod error;
pub mod llm;
pub mod logger;
pub mod lumosai;
pub mod memory;
pub mod storage;
pub mod telemetry;
pub mod tool;
pub mod types;
pub mod vector;
pub mod workflow;
pub mod app;
pub mod rag;
pub mod voice;

/// Re-export common types and traits
pub use error::{Error, Result};
pub use llm::{LlmProvider, LlmOptions, Message, Role};
pub use llm::{OpenAiProvider, AnthropicProvider, QwenProvider, MockLlmProvider};
pub use agent::{Agent, AgentConfig, BasicAgent, create_basic_agent, AgentGenerateOptions, AgentStreamOptions};
pub use base::{Base, ComponentConfig, BaseComponent};
pub use logger::{Logger, LogLevel, Component as LogComponent, create_logger, create_noop_logger};
pub use lumosai::{Lumosai, LumosaiConfig};
pub use memory::{Memory, WorkingMemory, WorkingMemoryContent};
pub use storage::{Storage, create_memory_storage};
pub use tool::{Tool};
pub use vector::{
    VectorStorage, 
    MemoryVectorStorage, 
    SimilarityMetric, 
    VectorStorageConfig, 
    create_vector_storage, 
    create_memory_vector_storage,
    EmbeddingService,
    create_random_embedding,
    Vector,
    IndexStats,
    QueryResult as VectorQueryResult,
    FilterCondition,
    Document
}; 

#[cfg(feature = "vector_sqlite")]
pub use vector::{
    SqliteVectorStorage,
    sqlite::create_sqlite_vector_storage,
    sqlite::create_sqlite_vector_storage_in_memory
}; 

pub use crate::app::LumosApp;
pub use crate::rag::{RagPipeline, QueryResult as RagQueryResult, DocumentSource};
pub use voice::{VoiceProvider, VoiceOptions, ListenOptions, CompositeVoice, providers::{MockVoice, OpenAIVoice}};

// 导出工作流模块但不重命名
// pub use crate::workflow;

// 工作流类型的便捷访问
pub mod workflow_types {
    pub use crate::workflow::basic::{
        Workflow, StepResult, StepCondition, WorkflowStep, BasicWorkflow, create_basic_workflow
    };
} 