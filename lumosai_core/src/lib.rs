//! Lumosai Core - Rust实现的AI应用框架核心库
//!
//! 提供了Agent、工作流、工具、LLM接口等核心功能

pub mod agent;
pub mod app;
pub mod auth;
pub mod base;
pub mod billing;
pub mod bindings;
pub mod cache;
pub mod cli;
pub mod cloud;
pub mod config;
pub mod data_processing;
pub mod debug;
pub mod distributed;
pub mod documentation;
pub mod error;
pub mod llm;
pub mod logger;
pub mod logging;
pub mod lumosai;
pub mod marketplace;
pub mod memory;
pub mod monitoring;
pub mod plugin;
pub mod prelude;
pub mod rag;
pub mod security;
pub mod storage;
pub mod telemetry;
pub mod tool;
pub mod types;
pub mod unified_api;
pub mod vector;
pub mod voice;
pub mod workflow;

pub use agent::{
    create_basic_agent, AgentConfig, AgentFactory, AgentGenerateOptions, AgentStreamOptions,
    Agent, BasicAgent,
};
pub use base::{Base, BaseComponent, ComponentConfig};
/// Re-export common types and traits
pub use error::{Error, Result};
pub use llm::{AnthropicProvider, MockLlmProvider, OpenAiProvider, QwenProvider};
pub use llm::{LlmOptions, LlmProvider, Message, Role};
pub use logger::{create_logger, create_noop_logger, Component as LogComponent, LogLevel, Logger};
pub use lumosai::{Lumosai, LumosaiConfig};
pub use memory::{Memory, WorkingMemory, WorkingMemoryContent};
pub use storage::{create_memory_storage, Storage};
pub use tool::Tool;
pub use vector::{
    create_memory_vector_storage, create_random_embedding, create_vector_storage, Document,
    EmbeddingService, FilterCondition, IndexStats, MemoryVectorStorage,
    QueryResult as VectorQueryResult, SimilarityMetric, Vector, VectorStorage, VectorStorageConfig,
};

#[cfg(feature = "vector_sqlite")]
pub use vector::{
    sqlite::create_sqlite_vector_storage, sqlite::create_sqlite_vector_storage_in_memory,
    SqliteVectorStorage,
};

pub use crate::app::LumosApp;
pub use crate::rag::{DocumentSource, QueryResult as RagQueryResult, RagPipeline};
pub use voice::{
    providers::{MockVoice, OpenAIVoice},
    CompositeVoice, ListenOptions, VoiceOptions, VoiceProvider,
};

// 导出工作流模块但不重命名
// pub use crate::workflow;

// 工作流类型的便捷访问
pub mod workflow_types {
    pub use crate::workflow::basic::{
        create_basic_workflow, BasicWorkflow, StepCondition, StepResult, Workflow, WorkflowStep,
    };
}
