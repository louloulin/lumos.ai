//! Lumosai Core - Rust实现的AI应用框架核心库
//!
//! 提供了Agent、工作流、工具、LLM接口等核心功能
//!
//! 经过 v2.0 重构，lumosai_core 现在只包含 8 个核心模块：
//! - agent: Agent 核心功能
//! - workflow: 工作流引擎
//! - tool: 工具系统
//! - memory: 内存管理
//! - llm: LLM 抽象
//! - config: 配置管理
//! - error: 错误处理
//! - prelude: 便捷导入

// 核心模块
pub mod agent;
pub mod cache; // 新增：缓存系统
pub mod config;
pub mod error;
pub mod llm;
pub mod logger;
pub mod memory;
pub mod pool; // 新增：资源池系统
pub mod prelude;
pub mod telemetry;
pub mod tool;
pub mod workflow;

// 兼容性模块（临时）
pub mod compat;

// 保留的兼容性模块（将逐步移除）
pub mod app;
pub mod base;
pub mod lumosai;
pub mod rag;
pub mod types;
pub mod unified_api;
pub mod vector;

// 核心模块导出
pub use agent::{
    create_basic_agent, Agent, AgentBuilder, AgentConfig, AgentFactory, AgentGenerateOptions,
    AgentStreamOptions, BasicAgent,
};
pub use config::*;
pub use error::{Error, Result};
pub use llm::{AnthropicProvider, MockLlmProvider, OpenAiProvider, QwenProvider};
pub use llm::{LlmOptions, LlmProvider, Message, Role};
pub use memory::{Memory, WorkingMemory, WorkingMemoryContent};
pub use tool::Tool;

// 兼容性导出（将逐步移除）
pub use crate::app::LumosApp;
pub use crate::rag::{DocumentSource, QueryResult as RagQueryResult, RagPipeline};
pub use base::{Base, BaseComponent, ComponentConfig};
pub use lumosai::{Lumosai, LumosaiConfig};
pub use vector::{
    create_memory_vector_storage, create_random_embedding, create_vector_storage, Document,
    EmbeddingService, FilterCondition, IndexStats, MemoryVectorStorage,
    QueryResult as VectorQueryResult, SimilarityMetric, Vector, VectorStorage, VectorStorageConfig,
};

// SQLite features temporarily disabled due to dependency conflicts
// #[cfg(feature = "vector_sqlite")]
// pub use vector::{
//     sqlite::create_sqlite_vector_storage, sqlite::create_sqlite_vector_storage_in_memory,
//     SqliteVectorStorage,
// };

// 导出工作流模块但不重命名
// pub use crate::workflow;

// 工作流类型的便捷访问
pub mod workflow_types {
    pub use crate::workflow::basic::{
        create_basic_workflow, BasicWorkflow, StepCondition, StepResult, Workflow, WorkflowStep,
    };
}
