//! Memory module for storing and retrieving context information
//!
//! This module provides a comprehensive memory system for LumosAI agents,
//! supporting multiple memory types and storage backends.
//!
//! # Overview
//!
//! The memory system consists of several components:
//!
//! - **Basic Memory**: Simple message storage and retrieval
//! - **Working Memory**: Short-term memory with limited capacity
//! - **Semantic Memory**: Long-term memory with vector-based search
//! - **Memory Processors**: Message filtering and transformation
//! - **Memory Threads**: Conversation thread management
//! - **Memory Sessions**: Session-based context management
//!
//! # Quick Start
//!
//! ## Using Unified Memory (Recommended)
//!
//! ```rust
//! use lumosai_core::memory::UnifiedMemory;
//!
//! // Create basic memory
//! let memory = UnifiedMemory::basic();
//!
//! // Create semantic memory
//! let memory = UnifiedMemory::semantic();
//!
//! // Create working memory with capacity
//! let memory = UnifiedMemory::working(1000);
//! ```
//!
//! ## Using Memory Trait Directly
//!
//! ```rust
//! use lumosai_core::memory::{BasicMemory, Memory, MemoryConfig};
//! use lumosai_core::llm::{Message, Role};
//!
//! # async fn example() -> lumosai_core::Result<()> {
//! let memory = BasicMemory::new(None, None);
//!
//! // Store a message
//! let message = Message {
//!     role: Role::User,
//!     content: "Hello!".to_string(),
//!     metadata: None,
//!     name: None,
//! };
//! memory.store(&message).await?;
//!
//! // Retrieve messages
//! let config = MemoryConfig::default();
//! let messages = memory.retrieve(&config).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Memory Types
//!
//! ## Basic Memory
//!
//! Simple in-memory storage for messages. Suitable for most applications.
//!
//! ## Working Memory
//!
//! Short-term memory with limited capacity (LRU eviction). Useful for
//! maintaining recent conversation context.
//!
//! ## Semantic Memory
//!
//! Long-term memory with vector-based semantic search. Enables retrieval
//! of relevant past conversations based on meaning.
//!
//! # See Also
//!
//! - [`Memory`] - Core memory trait
//! - [`WorkingMemory`] - Working memory trait
//! - [`SemanticMemory`] - Semantic memory trait
//! - [`MemoryProcessor`] - Message processing trait
#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
#![allow(non_camel_case_types, ambiguous_glob_reexports, hidden_glob_reexports)]
#![allow(unexpected_cfgs, unused_assignments)]

use crate::llm::Message;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// 导入统一内存系统
pub mod unified;

/// Configuration for semantic recall (memory retrieval)
///
/// Semantic recall uses vector embeddings to find relevant past messages
/// based on semantic similarity rather than exact keyword matching.
///
/// # Examples
///
/// ```rust
/// use lumosai_core::memory::SemanticRecallConfig;
///
/// let config = SemanticRecallConfig {
///     top_k: 5,
///     message_range: None,
///     generate_summaries: false,
///     use_embeddings: true,
///     max_capacity: Some(1000),
///     max_results: Some(10),
///     relevance_threshold: Some(0.7),
///     template: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRecallConfig {
    /// Maximum number of results to return
    pub top_k: usize,
    /// Message range to include around each result
    pub message_range: Option<MessageRange>,
    /// Whether to generate summaries of retrieved messages
    #[serde(default)]
    pub generate_summaries: bool,
    /// Whether to use embedding vectors for search
    #[serde(default = "default_use_embeddings")]
    pub use_embeddings: bool,
    /// Maximum capacity of the semantic memory
    pub max_capacity: Option<usize>,
    /// Maximum number of results to return
    pub max_results: Option<usize>,
    /// Minimum relevance score threshold (0.0-1.0)
    pub relevance_threshold: Option<f32>,
    /// Template for formatting results
    pub template: Option<String>,
}

/// Message range configuration for context retrieval
///
/// Specifies how many messages before and after a target message
/// should be included in the results.
///
/// # Examples
///
/// ```rust
/// use lumosai_core::memory::MessageRange;
///
/// // Include 2 messages before and 2 after
/// let range = MessageRange {
///     before: 2,
///     after: 2,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRange {
    /// Number of messages before the target message
    pub before: usize,
    /// Number of messages after the target message
    pub after: usize,
}

/// Memory configuration for retrieval operations
///
/// Controls how messages are retrieved from memory, including filtering,
/// limits, and search parameters.
///
/// # Examples
///
/// ## Basic Configuration
///
/// ```rust
/// use lumosai_core::memory::MemoryConfig;
///
/// let config = MemoryConfig {
///     enabled: true,
///     last_messages: Some(10),
///     ..Default::default()
/// };
/// ```
///
/// ## With Semantic Search
///
/// ```rust
/// use lumosai_core::memory::MemoryConfig;
///
/// let config = MemoryConfig {
///     enabled: true,
///     query: Some("What did we discuss about AI?".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Storage backend identifier
    pub store_id: Option<String>,
    /// Namespace for memory isolation (multi-tenancy)
    pub namespace: Option<String>,
    /// Whether memory is enabled
    #[serde(default = "default_memory_enabled")]
    pub enabled: bool,
    /// Working memory configuration
    pub working_memory: Option<working::WorkingMemoryConfig>,
    /// Semantic recall configuration
    pub semantic_recall: Option<SemanticRecallConfig>,
    /// Number of most recent messages to retrieve
    pub last_messages: Option<usize>,
    /// Query string for semantic search
    pub query: Option<String>,
}

fn default_memory_enabled() -> bool {
    true
}

/// Default embedding vector setting
fn default_use_embeddings() -> bool {
    true
}

impl Default for MemoryConfig {
    /// Creates a default memory configuration
    ///
    /// # Default Values
    ///
    /// - `enabled`: true
    /// - `store_id`: None
    /// - `namespace`: None
    /// - `working_memory`: None
    /// - `semantic_recall`: None
    /// - `last_messages`: Some(0)  // ⭐ 优化默认值：禁用历史消息提高性能
    /// - `query`: None
    fn default() -> Self {
        Self {
            store_id: None,
            namespace: None,
            enabled: true,
            working_memory: None,
            semantic_recall: None,
            last_messages: Some(0), // ⭐ 优化：默认禁用历史消息提高性能
            query: None,
        }
    }
}

/// Core trait for memory systems
///
/// The `Memory` trait defines the fundamental interface for all memory
/// implementations in LumosAI. It provides methods for storing and retrieving
/// messages, enabling agents to maintain conversation context.
///
/// # Overview
///
/// Memory systems allow agents to:
/// - Store conversation history
/// - Retrieve relevant past messages
/// - Maintain context across multiple interactions
/// - Support different storage backends
///
/// # Implementations
///
/// LumosAI provides several memory implementations:
///
/// - [`BasicMemory`] - Simple in-memory storage
/// - [`WorkingMemory`] - Short-term memory with capacity limits
/// - [`SemanticMemory`] - Long-term memory with vector search
/// - [`UnifiedMemory`] - Unified interface for all memory types (recommended)
///
/// # Examples
///
/// ## Using BasicMemory
///
/// ```rust
/// use lumosai_core::memory::{BasicMemory, Memory, MemoryConfig};
/// use lumosai_core::llm::{Message, Role};
///
/// # async fn example() -> lumosai_core::Result<()> {
/// let memory = BasicMemory::new(None, None);
///
/// // Store a message
/// let message = Message {
///     role: Role::User,
///     content: "What is Rust?".to_string(),
///     metadata: None,
///     name: None,
/// };
/// memory.store(&message).await?;
///
/// // Retrieve all messages
/// let config = MemoryConfig::default();
/// let messages = memory.retrieve(&config).await?;
/// println!("Retrieved {} messages", messages.len());
/// # Ok(())
/// # }
/// ```
///
/// ## Retrieving Recent Messages
///
/// ```rust
/// use lumosai_core::memory::{Memory, MemoryConfig};
///
/// # async fn example(memory: &dyn Memory) -> lumosai_core::Result<()> {
/// let config = MemoryConfig {
///     enabled: true,
///     last_messages: Some(10), // Get last 10 messages
///     ..Default::default()
/// };
///
/// let messages = memory.retrieve(&config).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Semantic Search
///
/// ```rust
/// use lumosai_core::memory::{Memory, MemoryConfig};
///
/// # async fn example(memory: &dyn Memory) -> lumosai_core::Result<()> {
/// let config = MemoryConfig {
///     enabled: true,
///     query: Some("Tell me about AI".to_string()),
///     ..Default::default()
/// };
///
/// let relevant_messages = memory.retrieve(&config).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Thread Safety
///
/// All memory implementations must be `Send + Sync`, allowing them to be
/// safely shared across threads and used in async contexts.
///
/// # See Also
///
/// - [`BasicMemory`] - Simple memory implementation
/// - [`WorkingMemory`] - Working memory trait
/// - [`SemanticMemory`] - Semantic memory trait
/// - [`UnifiedMemory`] - Unified memory interface
#[async_trait::async_trait]
pub trait Memory: Send + Sync {
    /// Stores a message in memory
    ///
    /// Persists a message to the memory backend for later retrieval.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to store
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The storage backend is unavailable
    /// - The message cannot be serialized
    /// - Storage capacity is exceeded
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::memory::Memory;
    /// use lumosai_core::llm::{Message, Role};
    ///
    /// # async fn example(memory: &dyn Memory) -> lumosai_core::Result<()> {
    /// let message = Message {
    ///     role: Role::User,
    ///     content: "Hello, AI!".to_string(),
    ///     metadata: None,
    ///     name: None,
    /// };
    ///
    /// memory.store(&message).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn store(&self, message: &Message) -> Result<()>;

    /// Retrieves messages from memory
    ///
    /// Fetches messages based on the provided configuration, which can
    /// include filters, limits, and search queries.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for retrieval (filters, limits, queries)
    ///
    /// # Returns
    ///
    /// A vector of messages matching the configuration criteria.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The storage backend is unavailable
    /// - The query is invalid
    /// - Deserialization fails
    ///
    /// # Examples
    ///
    /// ## Retrieve All Messages
    ///
    /// ```rust
    /// use lumosai_core::memory::{Memory, MemoryConfig};
    ///
    /// # async fn example(memory: &dyn Memory) -> lumosai_core::Result<()> {
    /// let config = MemoryConfig::default();
    /// let messages = memory.retrieve(&config).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Retrieve Last N Messages
    ///
    /// ```rust
    /// use lumosai_core::memory::{Memory, MemoryConfig};
    ///
    /// # async fn example(memory: &dyn Memory) -> lumosai_core::Result<()> {
    /// let config = MemoryConfig {
    ///     last_messages: Some(5),
    ///     ..Default::default()
    /// };
    /// let messages = memory.retrieve(&config).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Semantic Search
    ///
    /// ```rust
    /// use lumosai_core::memory::{Memory, MemoryConfig};
    ///
    /// # async fn example(memory: &dyn Memory) -> lumosai_core::Result<()> {
    /// let config = MemoryConfig {
    ///     query: Some("machine learning".to_string()),
    ///     ..Default::default()
    /// };
    /// let messages = memory.retrieve(&config).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn retrieve(&self, config: &MemoryConfig) -> Result<Vec<Message>>;

    /// Returns the memory as a thread storage if supported
    ///
    /// Some memory implementations support thread-based storage, which
    /// allows organizing messages into separate conversation threads.
    ///
    /// # Returns
    ///
    /// - `Some(storage)` if the memory supports thread storage
    /// - `None` if thread storage is not supported (default)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::memory::Memory;
    ///
    /// # fn example(memory: &dyn Memory) {
    /// if let Some(thread_storage) = memory.as_thread_storage() {
    ///     println!("Memory supports thread storage");
    /// } else {
    ///     println!("Memory does not support thread storage");
    /// }
    /// # }
    /// ```
    fn as_thread_storage(&self) -> Option<Arc<dyn thread::MemoryThreadStorage>> {
        None // Default implementation returns None
    }

    /// 创建新线程（可选功能）
    ///
    /// 默认实现返回错误，表示不支持线程管理。
    /// 支持线程管理的 Memory 实现应该覆盖此方法。
    async fn create_thread(
        &self,
        _params: thread::CreateThreadParams,
    ) -> Result<thread::MemoryThread> {
        Err(crate::error::Error::UnsupportedOperation(
            "Thread management is not supported by this memory implementation".to_string(),
        ))
    }

    /// 获取线程信息（可选功能）
    ///
    /// 默认实现返回错误，表示不支持线程管理。
    async fn get_thread(
        &self,
        _thread_id: &str,
        _resource_id: Option<&str>,
    ) -> Result<Option<thread::MemoryThread>> {
        Err(crate::error::Error::UnsupportedOperation(
            "Thread management is not supported by this memory implementation".to_string(),
        ))
    }

    /// 更新线程（可选功能）
    ///
    /// 默认实现返回错误，表示不支持线程管理。
    async fn update_thread(
        &self,
        _thread_id: &str,
        _params: thread::UpdateThreadParams,
        _resource_id: Option<&str>,
    ) -> Result<thread::MemoryThread> {
        Err(crate::error::Error::UnsupportedOperation(
            "Thread management is not supported by this memory implementation".to_string(),
        ))
    }

    /// 删除线程（可选功能）
    ///
    /// 默认实现返回错误，表示不支持线程管理。
    async fn delete_thread(&self, _thread_id: &str, _resource_id: Option<&str>) -> Result<()> {
        Err(crate::error::Error::UnsupportedOperation(
            "Thread management is not supported by this memory implementation".to_string(),
        ))
    }

    /// 列出资源的所有线程（可选功能）
    ///
    /// 默认实现返回错误，表示不支持线程管理。
    async fn list_threads(&self, _resource_id: &str) -> Result<Vec<thread::MemoryThread>> {
        Err(crate::error::Error::UnsupportedOperation(
            "Thread management is not supported by this memory implementation".to_string(),
        ))
    }

    /// 获取线程统计信息（可选功能）
    ///
    /// 默认实现返回错误，表示不支持线程管理。
    async fn get_thread_stats(
        &self,
        _thread_id: &str,
        _resource_id: Option<&str>,
    ) -> Result<thread::ThreadStats> {
        Err(crate::error::Error::UnsupportedOperation(
            "Thread management is not supported by this memory implementation".to_string(),
        ))
    }

    /// 语义召回（可选功能）
    ///
    /// 执行语义搜索并返回相关消息，支持命名空间过滤。
    /// 默认实现返回错误，表示不支持语义召回。
    /// 支持语义召回的 Memory 实现应该覆盖此方法。
    ///
    /// # 参数
    ///
    /// * `query` - 搜索查询字符串
    /// * `config` - 语义召回配置
    /// * `namespace` - 可选的命名空间过滤
    ///
    /// # 示例
    ///
    /// ```rust
    /// use lumosai_core::memory::{Memory, SemanticRecallConfig};
    ///
    /// # async fn example(memory: &dyn Memory) -> lumosai_core::Result<()> {
    /// let config = SemanticRecallConfig {
    ///     top_k: 5,
    ///     ..Default::default()
    /// };
    /// let results = memory.semantic_recall("AI", &config, Some("namespace".to_string())).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn semantic_recall(
        &self,
        _query: &str,
        _config: &SemanticRecallConfig,
        _namespace: Option<String>,
    ) -> Result<Vec<Message>> {
        Err(crate::error::Error::UnsupportedOperation(
            "Semantic recall is not supported by this memory implementation".to_string(),
        ))
    }
}

// 模块声明
pub mod basic;
pub mod enhanced;
pub mod processor;
pub mod semantic;
pub mod semantic_memory;
pub mod session;
pub mod thread;
pub mod working;

#[cfg(test)]
mod real_api_tests;

// #[cfg(test)]
// mod processor_tests;

// 重新导出
pub use basic::BasicMemory;
pub use enhanced::{
    EnhancedMemory, ImportanceProcessor, MemoryEntry, MemoryEntryType, MemoryQueryOptions,
};
pub use processor::{
    create_default_processor_chain, CompositeProcessor, DeduplicationProcessor, MemoryProcessor,
    MemoryProcessorOptions, MessageLimitProcessor, RoleFilterProcessor,
};
pub use semantic_memory::{
    create_semantic_memory, SemanticMemoryTrait as SemanticMemory, SemanticSearchOptions,
    SemanticSearchResult,
};
pub use session::{
    ActionItem, CreateSessionParams, Priority, Session, SessionConfig, SessionContext,
    SessionManager, SessionState, SessionStats, UpdateSessionParams,
};
pub use thread::{
    CreateThreadParams, GetMessagesParams, MemoryOptions, MemoryThread, MemoryThreadManager,
    MemoryThreadStorage, MessageFilter, ThreadStats, UpdateThreadParams,
};
pub use working::{
    create_working_memory, BasicWorkingMemory, WorkingMemory, WorkingMemoryConfig,
    WorkingMemoryContent,
};

// 导出统一内存系统 - 这是新的推荐API
pub use unified::{Memory as UnifiedMemory, MemoryType};

/// 添加兼容函数，用于创建基本工作内存
#[inline]
pub fn create_basic_working_memory(config: &WorkingMemoryConfig) -> Result<Box<dyn WorkingMemory>> {
    create_working_memory(config)
}
