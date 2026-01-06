//! Lumos Prelude - 最常用的API重导出
//!
//! 这个模块包含了Lumos框架中最常用的类型和函数，
//! 通过`use lumos::prelude::*;`可以一次性导入所有常用API。

// 核心错误和结果类型
pub use crate::{Error, Message, Result, Role};

// 向量存储相关
#[cfg(feature = "postgres")]
pub use crate::vector::PostgresStorage;
pub use crate::vector::VectorStorage; // MemoryStorage temporarily disabled
#[cfg(feature = "vector-memory")]
pub use crate::vector::MemoryStorage;

// RAG系统相关
pub use crate::rag::{Document, RagSystem, SearchResult}; // SimpleRag temporarily disabled

// Agent相关
pub use crate::agent::{AgentBuilder, AgentResponse, SimpleAgent};
pub use lumosai_core::agent::Agent;

// 会话管理
pub use crate::session::{Session, SessionManager, SessionState};

// 事件系统
pub use crate::events::{AgentEvent, EventBus, EventHandler};

// 编排系统
pub use crate::orchestration::{
    AgentOrchestrator, BasicOrchestrator, CollaborationTask, OrchestrationPattern as Pattern,
};

// 核心trait重导出
pub use lumosai_core::agent::trait_def::Agent as AgentTrait;
pub use lumosai_core::llm::LlmProvider;
pub use lumosai_core::tool::Tool;

// 向量存储trait
pub use lumosai_vector_core::prelude::IndexConfig;

// RAG trait
// pub use lumosai_rag::{embedding::EmbeddingProvider, types::ChunkingStrategy}; // Temporarily disabled - package excluded

// UI组件相关 (可选功能)
#[cfg(feature = "ui")]
pub use crate::ui::prelude::*;

// 常用宏
#[cfg(feature = "macros")]
pub use lumos_macro::*;

// 异步运行时相关
pub use futures;
pub use tokio;

// 序列化相关
pub use serde::{Deserialize, Serialize};
pub use serde_json;

// 时间相关
pub use chrono::{DateTime, Utc};

// UUID生成
pub use uuid::Uuid;

// 常用集合类型
pub use std::collections::HashMap;
pub use std::sync::Arc;
