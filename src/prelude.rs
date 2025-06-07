//! Lumos Prelude - 最常用的API重导出
//!
//! 这个模块包含了Lumos框架中最常用的类型和函数，
//! 通过`use lumos::prelude::*;`可以一次性导入所有常用API。

// 核心错误和结果类型
pub use crate::{Error, Result, Message, Role};

// 向量存储相关
pub use crate::vector::{VectorStorage, MemoryStorage, PostgresStorage};

// RAG系统相关
pub use crate::rag::{RagSystem, SimpleRag, Document, SearchResult};

// Agent相关
pub use crate::agent::{SimpleAgent, AgentBuilder, AgentResponse};

// 会话管理
pub use crate::session::{Session, SessionManager, SessionState};

// 事件系统
pub use crate::events::{EventBus, AgentEvent, EventHandler};

// 编排系统
pub use crate::orchestration::{
    OrchestrationPattern as Pattern,
    CollaborationTask,
    AgentOrchestrator,
    BasicOrchestrator,
};

// 核心trait重导出
pub use lumosai_core::agent::trait_def::Agent as AgentTrait;
pub use lumosai_core::llm::LlmProvider;
pub use lumosai_core::tool::Tool;

// 向量存储trait
pub use lumosai_vector::core::{VectorStore, VectorStoreIndex};

// RAG trait
pub use lumosai_rag::{
    chunking::ChunkingStrategy,
    embedding::EmbeddingProvider,
    retrieval::RetrievalStrategy,
};

// 常用宏
#[cfg(feature = "macros")]
pub use lumos_macro::*;

// 异步运行时相关
pub use tokio;
pub use futures;

// 序列化相关
pub use serde_json;
pub use serde::{Serialize, Deserialize};

// 时间相关
pub use chrono::{DateTime, Utc};

// UUID生成
pub use uuid::Uuid;

// 常用集合类型
pub use std::collections::HashMap;
pub use std::sync::Arc;
