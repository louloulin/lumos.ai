//! Memory module for storing and retrieving context information

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::llm::Message;
use crate::Result;

/// 工作内存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryConfig {
    /// 是否启用工作内存
    pub enabled: bool,
    /// 内存模板
    pub template: Option<String>,
    /// 内容类型
    pub content_type: Option<String>,
    /// 最大容量
    pub max_capacity: Option<usize>,
}

/// 语义回忆配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRecallConfig {
    /// 返回的最大结果数
    pub top_k: usize,
    /// 消息范围
    pub message_range: Option<MessageRange>,
    /// 是否生成摘要
    #[serde(default)]
    pub generate_summaries: bool,
    /// 是否使用嵌入向量
    #[serde(default = "default_use_embeddings")]
    pub use_embeddings: bool,
    /// 最大容量
    pub max_capacity: Option<usize>,
    /// 最大结果数
    pub max_results: Option<usize>,
    /// 相关性阈值
    pub relevance_threshold: Option<f32>,
    /// 模板
    pub template: Option<String>,
}

/// 消息范围配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRange {
    /// 选取目标消息前的消息数
    pub before: usize,
    /// 选取目标消息后的消息数
    pub after: usize,
}

/// 内存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// 存储ID（表示使用哪个存储后端）
    pub store_id: Option<String>,
    /// 命名空间（用于隔离不同的内存空间）
    pub namespace: Option<String>,
    /// 是否启用内存
    #[serde(default = "default_memory_enabled")]
    pub enabled: bool,
    /// 工作内存配置
    pub working_memory: Option<WorkingMemoryConfig>,
    /// 语义回忆配置
    pub semantic_recall: Option<SemanticRecallConfig>,
    /// 每次获取的最后消息数量
    pub last_messages: Option<usize>,
    /// 查询内容
    pub query: Option<String>,
}

fn default_memory_enabled() -> bool {
    true
}

/// 默认嵌入向量设置
fn default_use_embeddings() -> bool {
    true
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            store_id: None,
            namespace: None,
            enabled: true,
            working_memory: None,
            semantic_recall: None,
            last_messages: None,
            query: None,
        }
    }
}

/// Memory trait for storing and retrieving messages
#[async_trait::async_trait]
pub trait Memory: Send + Sync {
    /// Store a message in memory
    async fn store(&self, message: &Message) -> Result<()>;
    
    /// Retrieve messages from memory
    async fn retrieve(&self, config: &MemoryConfig) -> Result<Vec<Message>>;
}

pub mod working;
pub mod semantic;
pub mod semantic_memory;

// 重新导出工作内存类型
pub use working::{WorkingMemory, WorkingMemoryContent, create_working_memory}; 
pub use semantic_memory::{SemanticMemoryTrait as SemanticMemory, SemanticSearchOptions, SemanticSearchResult, create_semantic_memory}; 