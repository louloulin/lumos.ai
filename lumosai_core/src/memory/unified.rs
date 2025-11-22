//! 统一内存系统 - LumosAI v2.0 第四周任务
//!
//! 提供统一的内存接口，简化内存系统的使用和配置
//!
//! # 设计目标
//! - 统一内存接口，减少抽象层次
//! - 提供简单的构造函数：basic(), semantic(), working()
//! - 支持 CompositeMemory 构建器模式
//! - 智能默认配置，开箱即用
//! - 保持向后兼容性

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::error::Result;
use crate::llm::{LlmProvider, Message};
use crate::memory::{
    create_semantic_memory, create_working_memory,
    processor::MemoryProcessor,
    semantic_memory::{SemanticMemoryTrait, SemanticSearchOptions},
    thread::{
        CreateThreadParams, GetMessagesParams, MemoryThread, MemoryThreadManager,
        MemoryThreadStorage, ThreadStats, UpdateThreadParams,
    },
    BasicMemory, Memory as MemoryTrait, MemoryConfig, SemanticRecallConfig, WorkingMemory,
    WorkingMemoryConfig,
};

/// 内存类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryType {
    /// 基础内存 - 简单的消息存储和检索
    Basic,
    /// 语义内存 - 基于向量相似度的智能检索
    Semantic,
    /// 工作内存 - 临时数据存储，支持容量限制
    Working { size: usize },
    /// 混合内存 - 结合多种内存类型
    Hybrid {
        working_size: Option<usize>,
        enable_semantic: bool,
    },
}

/// 内存实现的内部枚举
enum MemoryImpl {
    /// 基础内存实现
    Basic(BasicMemory),
    /// 语义内存实现  
    Semantic(Arc<dyn SemanticMemoryTrait>),
    /// 工作内存实现
    Working(Box<dyn WorkingMemory>),
    /// 混合内存实现
    Hybrid {
        basic: BasicMemory,
        working: Option<Box<dyn WorkingMemory>>,
        semantic: Option<Arc<dyn SemanticMemoryTrait>>,
    },
}

/// 统一内存结构体
///
/// 这是 LumosAI v2.0 推荐的内存API，提供简化的接口和智能默认配置
///
/// # 示例
///
/// ```rust
/// use lumosai_core::memory::UnifiedMemory;
///
/// // 创建基础内存
/// let memory = UnifiedMemory::basic();
///
/// // 创建语义内存
/// let memory = UnifiedMemory::semantic();
///
/// // 创建工作内存（指定大小）
/// let memory = UnifiedMemory::working(1000);
/// ```
pub struct Memory {
    /// 内部实现
    inner: MemoryImpl,
    /// 内存类型
    memory_type: MemoryType,
    /// 线程存储
    thread_storage: Option<Arc<dyn MemoryThreadStorage>>,
    /// Processor 列表
    processors: Vec<Arc<dyn MemoryProcessor>>,
    /// Processor 是否已同步到基础内存
    processors_registered: AtomicBool,
}

impl Memory {
    /// 为内存设置线程存储
    pub fn with_thread_storage(mut self, storage: Arc<dyn MemoryThreadStorage>) -> Self {
        self.thread_storage = Some(storage.clone());
        match &mut self.inner {
            MemoryImpl::Basic(basic) => basic.set_thread_storage(Some(storage)),
            MemoryImpl::Hybrid { basic, .. } => basic.set_thread_storage(Some(storage)),
            _ => {}
        }
        self.processors_registered.store(false, Ordering::SeqCst);
        self
    }

    /// 添加 Processor
    pub fn add_processor(mut self, processor: Arc<dyn MemoryProcessor>) -> Self {
        self.processors.push(processor.clone());
        self.processors_registered.store(false, Ordering::SeqCst);
        self
    }

    /// 创建基础内存
    ///
    /// 基础内存提供简单的消息存储和检索功能，适合大多数应用场景
    ///
    /// # 示例
    ///
    /// ```rust
    /// let memory = Memory::basic();
    /// ```
    pub fn basic() -> Self {
        let basic_memory = BasicMemory::new(None, None);

        Self {
            inner: MemoryImpl::Basic(basic_memory),
            memory_type: MemoryType::Basic,
            thread_storage: None,
            processors: Vec::new(),
            processors_registered: AtomicBool::new(false),
        }
    }

    /// 创建语义内存
    ///
    /// 语义内存基于向量相似度进行智能检索，适合需要语义理解的应用
    ///
    /// 注意：需要提供 LLM 提供者来生成嵌入向量
    ///
    /// # 示例
    ///
    /// ```rust
    /// let memory = Memory::semantic();
    /// ```
    pub fn semantic() -> Self {
        // 创建默认配置
        let config = MemoryConfig {
            store_id: None,
            namespace: Some("semantic".to_string()),
            enabled: true,
            working_memory: None,
            semantic_recall: Some(SemanticRecallConfig {
                top_k: 10,
                message_range: None,
                generate_summaries: false,
                use_embeddings: true,
                max_capacity: Some(1000),
                max_results: Some(10),
                relevance_threshold: Some(0.7),
                template: None,
            }),
            last_messages: None,
            query: None,
        };

        // 注意：这里我们创建一个占位符实现
        // 在实际使用时，用户需要通过 with_llm() 方法提供 LLM 提供者
        let basic_memory = BasicMemory::new(None, None);

        Self {
            inner: MemoryImpl::Basic(basic_memory),
            memory_type: MemoryType::Semantic,
            thread_storage: None,
            processors: Vec::new(),
            processors_registered: AtomicBool::new(false),
        }
    }

    /// 创建工作内存
    ///
    /// 工作内存提供临时数据存储，支持容量限制和自动清理
    ///
    /// # 参数
    ///
    /// * `size` - 内存容量限制
    ///
    /// # 示例
    ///
    /// ```rust
    /// let memory = Memory::working(1000);
    /// ```
    pub fn working(size: usize) -> Self {
        let config = WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("application/json".to_string()),
            max_capacity: Some(size),
        };

        // 创建工作内存实例
        let working_memory = create_working_memory(&config).unwrap_or_else(|_| {
            // 如果创建失败，使用基础内存作为后备
            Box::new(crate::memory::working::BasicWorkingMemory::new(
                config.clone(),
            ))
        });

        Self {
            inner: MemoryImpl::Working(working_memory),
            memory_type: MemoryType::Working { size },
            thread_storage: None,
            processors: Vec::new(),
            processors_registered: AtomicBool::new(false),
        }
    }

    /// 创建混合内存
    ///
    /// 混合内存结合了多种内存类型的优势
    ///
    /// # 参数
    ///
    /// * `working_size` - 工作内存大小（可选）
    /// * `enable_semantic` - 是否启用语义内存
    ///
    /// # 示例
    ///
    /// ```rust
    /// let memory = Memory::hybrid(Some(1000), true);
    /// ```
    pub fn hybrid(working_size: Option<usize>, enable_semantic: bool) -> Self {
        let basic_memory = BasicMemory::new(None, None);

        // 创建工作内存（如果指定了大小）
        let working_memory = working_size.map(|size| {
            let config = WorkingMemoryConfig {
                enabled: true,
                template: None,
                content_type: Some("application/json".to_string()),
                max_capacity: Some(size),
            };
            create_working_memory(&config).unwrap_or_else(|_| {
                Box::new(crate::memory::working::BasicWorkingMemory::new(
                    config.clone(),
                ))
            })
        });

        Self {
            inner: MemoryImpl::Hybrid {
                basic: basic_memory,
                working: working_memory,
                semantic: None, // 将在 with_llm() 中初始化
            },
            memory_type: MemoryType::Hybrid {
                working_size,
                enable_semantic,
            },
            thread_storage: None,
            processors: Vec::new(),
            processors_registered: AtomicBool::new(false),
        }
    }

    /// 获取内存类型
    pub fn memory_type(&self) -> &MemoryType {
        &self.memory_type
    }

    /// 为语义内存配置 LLM 提供者
    ///
    /// 这个方法允许为语义内存提供 LLM 提供者来生成嵌入向量
    ///
    /// # 参数
    ///
    /// * `llm` - LLM 提供者
    ///
    /// # 示例
    ///
    /// ```rust
    /// let memory = Memory::semantic().with_llm(llm_provider);
    /// ```
    pub fn with_llm(mut self, llm: Arc<dyn LlmProvider>) -> Result<Self> {
        match &mut self.inner {
            MemoryImpl::Basic(basic) if matches!(self.memory_type, MemoryType::Semantic) => {
                // 为语义内存创建实际的语义内存实现
                let config = MemoryConfig {
                    namespace: Some("semantic".to_string()),
                    enabled: true,
                    semantic_recall: Some(SemanticRecallConfig {
                        top_k: 10,
                        message_range: None,
                        generate_summaries: false,
                        use_embeddings: true,
                        max_capacity: Some(1000),
                        max_results: Some(10),
                        relevance_threshold: Some(0.7),
                        template: None,
                    }),
                    ..Default::default()
                };

                let semantic_memory = create_semantic_memory(&config, llm.clone())?;
                basic.set_thread_storage(self.thread_storage.clone());
                self.inner = MemoryImpl::Semantic(semantic_memory);
            }
            MemoryImpl::Hybrid { semantic, .. }
                if matches!(
                    self.memory_type,
                    MemoryType::Hybrid {
                        enable_semantic: true,
                        ..
                    }
                ) =>
            {
                // 为混合内存创建语义内存组件
                let config = MemoryConfig {
                    namespace: Some("hybrid_semantic".to_string()),
                    enabled: true,
                    semantic_recall: Some(SemanticRecallConfig {
                        top_k: 10,
                        message_range: None,
                        generate_summaries: false,
                        use_embeddings: true,
                        max_capacity: Some(1000),
                        max_results: Some(10),
                        relevance_threshold: Some(0.7),
                        template: None,
                    }),
                    ..Default::default()
                };

                *semantic = Some(create_semantic_memory(&config, llm)?);
            }
            _ => {
                // 对于其他类型，不需要 LLM 提供者
            }
        }

        Ok(self)
    }
}

#[async_trait]
impl MemoryTrait for Memory {
    /// 存储消息到内存
    async fn store(&self, message: &Message) -> Result<()> {
        self.ensure_processors_registered().await?;

        match &self.inner {
            MemoryImpl::Basic(basic) => basic.store(message).await,
            MemoryImpl::Semantic(semantic) => semantic.add(message).await,
            MemoryImpl::Working(working) => {
                // 将消息序列化为JSON值存储到工作内存
                let message_value =
                    serde_json::to_value(message).map_err(crate::error::Error::Json)?;
                working.set_value("last_message", message_value).await
            }
            MemoryImpl::Hybrid {
                basic,
                working,
                semantic,
            } => {
                // 存储到基础内存
                basic.store(message).await?;

                // 存储到工作内存（如果存在）
                if let Some(working) = working {
                    let message_value =
                        serde_json::to_value(message).map_err(crate::error::Error::Json)?;
                    working.set_value("last_message", message_value).await?;
                }

                // 存储到语义内存（如果存在）
                if let Some(semantic) = semantic {
                    semantic.add(message).await?;
                }

                Ok(())
            }
        }
    }

    /// 从内存检索消息
    async fn retrieve(&self, config: &MemoryConfig) -> Result<Vec<Message>> {
        self.ensure_processors_registered().await?;

        match &self.inner {
            MemoryImpl::Basic(basic) => basic.retrieve(config).await,
            MemoryImpl::Semantic(semantic) => {
                // 从语义内存检索
                if let Some(semantic_config) = &config.semantic_recall {
                    let search_options = SemanticSearchOptions {
                        limit: semantic_config.top_k,
                        threshold: semantic_config.relevance_threshold,
                        namespace: config.namespace.clone(),
                        use_window: false,
                        window_size: None,
                        filter: None,
                    };

                    let query = config.query.as_deref().unwrap_or("");
                    let results = semantic.search(query, &search_options).await?;
                    Ok(results.into_iter().map(|r| r.message).collect())
                } else {
                    // 如果没有语义配置，返回空结果
                    Ok(vec![])
                }
            }
            MemoryImpl::Working(working) => {
                // 从工作内存检索最后的消息
                if let Ok(Some(message_value)) = working.get_value("last_message").await {
                    if let Ok(message) = serde_json::from_value::<Message>(message_value) {
                        Ok(vec![message])
                    } else {
                        Ok(vec![])
                    }
                } else {
                    Ok(vec![])
                }
            }
            MemoryImpl::Hybrid {
                basic,
                working: _,
                semantic,
            } => {
                let mut combined = Vec::new();

                // 先获取线程历史消息（按时间顺序，最新的在前）
                let mut base_messages = basic.retrieve(config).await?;
                combined.append(&mut base_messages);

                // 然后获取语义召回结果
                if let Some(semantic) = semantic {
                    if let Some(mut semantic_messages) =
                        Self::semantic_results(semantic, config).await?
                    {
                        combined.append(&mut semantic_messages);
                    }
                }

                Ok(Self::dedup_messages(combined))
            }
        }
    }
}

impl Memory {
    /// 便利方法：存储单个消息
    ///
    /// # 示例
    ///
    /// ```rust
    /// let message = Message::user("Hello, world!");
    /// memory.add_message(message).await?;
    /// ```
    pub async fn add_message(&self, message: Message) -> Result<()> {
        self.store(&message).await
    }

    /// 便利方法：检索最近的消息
    ///
    /// # 参数
    ///
    /// * `count` - 要检索的消息数量
    ///
    /// # 示例
    ///
    /// ```rust
    /// let recent_messages = memory.get_recent_messages(10).await?;
    /// ```
    pub async fn get_recent_messages(&self, count: usize) -> Result<Vec<Message>> {
        let config = MemoryConfig {
            last_messages: Some(count),
            ..Default::default()
        };
        self.retrieve(&config).await
    }

    /// 便利方法：语义搜索
    ///
    /// 仅适用于语义内存和混合内存
    ///
    /// # 参数
    ///
    /// * `query` - 搜索查询
    /// * `top_k` - 返回的最大结果数
    ///
    /// # 示例
    ///
    /// ```rust
    /// let results = memory.semantic_search("AI技术", 5).await?;
    /// ```
    pub async fn semantic_search(&self, query: &str, top_k: usize) -> Result<Vec<Message>> {
        let config = MemoryConfig {
            query: Some(query.to_string()),
            semantic_recall: Some(SemanticRecallConfig {
                top_k,
                message_range: None,
                generate_summaries: false,
                use_embeddings: true,
                max_capacity: Some(1000),
                max_results: Some(top_k),
                relevance_threshold: Some(0.7),
                template: None,
            }),
            ..Default::default()
        };
        self.retrieve(&config).await
    }

    /// 语义召回方法
    ///
    /// 执行语义搜索并返回相关消息，支持命名空间过滤
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
    /// use lumosai_core::memory::{SemanticRecallConfig, MessageRange};
    ///
    /// let recall_config = SemanticRecallConfig {
    ///     top_k: 5,
    ///     message_range: Some(MessageRange { before: 1, after: 1 }),
    ///     ..Default::default()
    /// };
    /// let results = memory.semantic_recall("AI", &recall_config, Some("namespace".to_string())).await?;
    /// ```
    pub async fn semantic_recall(
        &self,
        query: &str,
        config: &SemanticRecallConfig,
        namespace: Option<String>,
    ) -> Result<Vec<Message>> {
        // 直接调用语义内存的 search 方法，不通过 retrieve 避免获取线程历史消息
        match &self.inner {
            MemoryImpl::Semantic(semantic) => {
                let mut options = SemanticSearchOptions::default();
                options.limit = config.top_k;
                options.threshold = config.relevance_threshold;
                options.namespace = namespace;
                if let Some(range) = &config.message_range {
                    options.use_window = true;
                    options.window_size = Some((range.before, range.after));
                }
                let results = semantic.search(query, &options).await?;
                Ok(results.into_iter().map(|r| r.message).collect())
            }
            MemoryImpl::Hybrid { semantic, .. } => {
                if let Some(semantic) = semantic {
                    let mut options = SemanticSearchOptions::default();
                    options.limit = config.top_k;
                    options.threshold = config.relevance_threshold;
                    options.namespace = namespace;
                    if let Some(range) = &config.message_range {
                        options.use_window = true;
                        options.window_size = Some((range.before, range.after));
                    }
                    let results = semantic.search(query, &options).await?;
                    Ok(results.into_iter().map(|r| r.message).collect())
                } else {
                    Ok(vec![])
                }
            }
            _ => Ok(vec![]), // 其他类型不支持语义召回
        }
    }

    /// 创建新线程
    ///
    /// # 参数
    ///
    /// * `params` - 线程创建参数
    ///
    /// # 错误
    ///
    /// 如果线程存储未配置，返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use lumosai_core::memory::{Memory, CreateThreadParams};
    ///
    /// # async fn example(memory: Memory) -> lumosai_core::Result<()> {
    /// let thread = memory.create_thread(CreateThreadParams {
    ///     id: None,
    ///     title: "New Conversation".to_string(),
    ///     agent_id: None,
    ///     resource_id: Some("user-123".to_string()),
    ///     metadata: None,
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_thread(
        &self,
        params: CreateThreadParams,
    ) -> Result<MemoryThread> {
        let storage = self
            .thread_storage
            .as_ref()
            .ok_or_else(|| {
                crate::error::Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?;
        let manager = MemoryThreadManager::new(storage.clone() as Arc<dyn MemoryThreadStorage>);
        manager.create_thread(params).await
    }

    /// 获取线程信息
    ///
    /// # 参数
    ///
    /// * `thread_id` - 线程ID
    /// * `resource_id` - 可选的资源ID，用于所有权验证
    ///
    /// # 示例
    ///
    /// ```rust
    /// # async fn example(memory: Memory) -> lumosai_core::Result<()> {
    /// if let Some(thread) = memory.get_thread("thread-123", Some("user-123")).await? {
    ///     println!("Thread title: {}", thread.title);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_thread(
        &self,
        thread_id: &str,
        resource_id: Option<&str>,
    ) -> Result<Option<MemoryThread>> {
        let storage = self
            .thread_storage
            .as_ref()
            .ok_or_else(|| {
                crate::error::Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?;
        let manager = MemoryThreadManager::new(storage.clone() as Arc<dyn MemoryThreadStorage>);
        manager.get_thread(thread_id, resource_id).await
    }

    /// 更新线程
    ///
    /// # 参数
    ///
    /// * `thread_id` - 线程ID
    /// * `params` - 更新参数
    /// * `resource_id` - 可选的资源ID，用于所有权验证
    ///
    /// # 示例
    ///
    /// ```rust
    /// use lumosai_core::memory::{Memory, UpdateThreadParams};
    ///
    /// # async fn example(memory: Memory) -> lumosai_core::Result<()> {
    /// let updated = memory.update_thread(
    ///     "thread-123",
    ///     UpdateThreadParams {
    ///         title: Some("Updated Title".to_string()),
    ///         metadata: None,
    ///     },
    ///     Some("user-123"),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_thread(
        &self,
        thread_id: &str,
        params: UpdateThreadParams,
        resource_id: Option<&str>,
    ) -> Result<MemoryThread> {
        let storage = self
            .thread_storage
            .as_ref()
            .ok_or_else(|| {
                crate::error::Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?;
        let manager = MemoryThreadManager::new(storage.clone() as Arc<dyn MemoryThreadStorage>);
        manager.update_thread(thread_id, params, resource_id).await
    }

    /// 删除线程
    ///
    /// # 参数
    ///
    /// * `thread_id` - 线程ID
    /// * `resource_id` - 可选的资源ID，用于所有权验证
    ///
    /// # 示例
    ///
    /// ```rust
    /// # async fn example(memory: Memory) -> lumosai_core::Result<()> {
    /// memory.delete_thread("thread-123", Some("user-123")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_thread(
        &self,
        thread_id: &str,
        resource_id: Option<&str>,
    ) -> Result<()> {
        let storage = self
            .thread_storage
            .as_ref()
            .ok_or_else(|| {
                crate::error::Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?;
        let manager = MemoryThreadManager::new(storage.clone() as Arc<dyn MemoryThreadStorage>);
        manager.delete_thread(thread_id, resource_id).await
    }

    /// 列出资源的所有线程
    ///
    /// # 参数
    ///
    /// * `resource_id` - 资源ID
    ///
    /// # 示例
    ///
    /// ```rust
    /// # async fn example(memory: Memory) -> lumosai_core::Result<()> {
    /// let threads = memory.list_threads("user-123").await?;
    /// println!("Found {} threads", threads.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_threads(&self, resource_id: &str) -> Result<Vec<MemoryThread>> {
        let storage = self
            .thread_storage
            .as_ref()
            .ok_or_else(|| {
                crate::error::Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?;
        let manager = MemoryThreadManager::new(storage.clone() as Arc<dyn MemoryThreadStorage>);
        manager.list_threads(resource_id).await
    }

    /// 获取线程统计信息
    ///
    /// # 参数
    ///
    /// * `thread_id` - 线程ID
    /// * `resource_id` - 可选的资源ID，用于所有权验证
    ///
    /// # 示例
    ///
    /// ```rust
    /// # async fn example(memory: Memory) -> lumosai_core::Result<()> {
    /// let stats = memory.get_thread_stats("thread-123", Some("user-123")).await?;
    /// println!("Thread has {} messages", stats.message_count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_thread_stats(
        &self,
        thread_id: &str,
        resource_id: Option<&str>,
    ) -> Result<ThreadStats> {
        let storage = self
            .thread_storage
            .as_ref()
            .ok_or_else(|| {
                crate::error::Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?;
        let manager = MemoryThreadManager::new(storage.clone() as Arc<dyn MemoryThreadStorage>);
        manager.get_thread_stats(thread_id, resource_id).await
    }

    /// 检查内存是否为空
    ///
    /// # 示例
    ///
    /// ```rust
    /// if memory.is_empty().await? {
    ///     println!("内存为空");
    /// }
    /// ```
    pub async fn is_empty(&self) -> Result<bool> {
        let messages = self.get_recent_messages(1).await?;
        Ok(messages.is_empty())
    }

    /// 获取内存统计信息
    ///
    /// # 示例
    ///
    /// ```rust
    /// let stats = memory.get_stats().await?;
    /// println!("内存中有 {} 条消息", stats.message_count);
    /// ```
    pub async fn get_stats(&self) -> Result<MemoryStats> {
        // 尝试获取最近100条消息来估算统计信息
        let messages = self.get_recent_messages(100).await?;

        Ok(MemoryStats {
            message_count: messages.len(),
            memory_type: self.memory_type.clone(),
            last_updated: chrono::Utc::now(),
        })
    }

    /// 清空内存
    ///
    /// 注意：此操作不可逆
    ///
    /// # 示例
    ///
    /// ```rust
    /// memory.clear().await?;
    /// ```
    pub async fn clear(&self) -> Result<()> {
        match &self.inner {
            MemoryImpl::Working(working) => {
                // 清空工作内存
                working.clear().await
            }
            _ => {
                // 对于其他类型的内存，目前不支持清空操作
                // 这是为了安全考虑，避免意外删除重要数据
                Err(crate::error::Error::UnsupportedOperation(
                    "清空操作仅支持工作内存".to_string(),
                ))
            }
        }
    }
}

/// 内存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// 消息数量
    pub message_count: usize,
    /// 内存类型
    pub memory_type: MemoryType,
    /// 最后更新时间
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl std::fmt::Display for MemoryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "内存统计: {} 条消息, 类型: {:?}, 更新时间: {}",
            self.message_count,
            self.memory_type,
            self.last_updated.format("%Y-%m-%d %H:%M:%S")
        )
    }
}

// ============================================================================
// CompositeMemory 构建器 - Week 11-12 统一内存架构
// ============================================================================

/// CompositeMemory 构建器配置
///
/// 支持链式配置多种内存类型和处理器
#[derive(Default)]
pub struct CompositeMemoryBuilder {
    /// 工作内存配置
    working_config: Option<WorkingMemoryConfig>,
    /// 语义内存配置
    semantic_config: Option<SemanticMemoryConfig>,
    /// 内存处理器列表
    processors: Vec<Arc<dyn MemoryProcessor>>,
    /// 命名空间
    namespace: Option<String>,
}

/// 语义内存配置
pub struct SemanticMemoryConfig {
    /// 向量存储后端名称
    pub vector_store: String,
    /// 嵌入模型名称
    pub embedding_model: String,
    /// 索引配置
    pub index_config: Option<String>,
}

impl Memory {
    async fn ensure_processors_registered(&self) -> Result<()> {
        if self.processors.is_empty()
            || self.processors_registered.load(Ordering::SeqCst)
        {
            return Ok(());
        }

        match &self.inner {
            MemoryImpl::Basic(basic) => {
                for processor in &self.processors {
                    basic.add_processor(processor.clone()).await?;
                }
            }
            MemoryImpl::Hybrid { basic, .. } => {
                for processor in &self.processors {
                    basic.add_processor(processor.clone()).await?;
                }
            }
            _ => {}
        }

        self.processors_registered.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn message_signature(message: &Message) -> String {
        let metadata_str = message
            .metadata
            .as_ref()
            .and_then(|meta| serde_json::to_string(meta).ok())
            .unwrap_or_default();
        let name = message.name.clone().unwrap_or_default();
        format!("{:?}::{name}::{}::{metadata_str}", message.role, message.content)
    }

    fn dedup_messages(messages: Vec<Message>) -> Vec<Message> {
        let mut seen = HashSet::new();
        let mut deduped = Vec::new();

        for message in messages {
            let signature = Self::message_signature(&message);
            if seen.insert(signature) {
                deduped.push(message);
            }
        }

        deduped
    }

    async fn semantic_results(
        semantic: &Arc<dyn SemanticMemoryTrait>,
        config: &MemoryConfig,
    ) -> Result<Option<Vec<Message>>> {
        let semantic_config = match &config.semantic_recall {
            Some(cfg) => cfg,
            None => return Ok(None),
        };

        let query = match config.query.as_deref() {
            Some(q) if !q.is_empty() => q,
            _ => return Ok(None),
        };

        let mut options = SemanticSearchOptions::default();
        options.limit = semantic_config.top_k;
        options.threshold = semantic_config.relevance_threshold;
        options.namespace = config.namespace.clone();

        if let Some(range) = &semantic_config.message_range {
            options.use_window = true;
            options.window_size = Some((range.before, range.after));
        } else {
            options.use_window = false;
            options.window_size = None;
        }

        let results = semantic.search(query, &options).await?;
        Ok(Some(results.into_iter().map(|r| r.message).collect()))
    }

    /// 创建 CompositeMemory 构建器
    ///
    /// # 示例
    ///
    /// ```rust
    /// use lumosai_core::memory::UnifiedMemory;
    ///
    /// let memory = UnifiedMemory::composite()
    ///     .working(1000)
    ///     .semantic("qdrant", "openai")
    ///     .build()
    ///     .await?;
    /// ```
    pub fn composite() -> CompositeMemoryBuilder {
        CompositeMemoryBuilder::default()
    }
}

impl CompositeMemoryBuilder {
    /// 配置工作内存
    ///
    /// # 参数
    /// - `capacity`: 工作内存容量（消息数量）
    ///
    /// # 示例
    ///
    /// ```rust
    /// let builder = Memory::composite().working(1000);
    /// ```
    pub fn working(mut self, capacity: usize) -> Self {
        self.working_config = Some(WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("buffer".to_string()),
            max_capacity: Some(capacity),
        });
        self
    }

    /// 配置语义内存
    ///
    /// # 参数
    /// - `vector_store`: 向量存储后端（如 "qdrant", "weaviate"）
    /// - `embedding_model`: 嵌入模型（如 "openai", "sentence-transformers"）
    ///
    /// # 示例
    ///
    /// ```rust
    /// let builder = Memory::composite()
    ///     .semantic("qdrant", "openai");
    /// ```
    pub fn semantic(mut self, vector_store: &str, embedding_model: &str) -> Self {
        self.semantic_config = Some(SemanticMemoryConfig {
            vector_store: vector_store.to_string(),
            embedding_model: embedding_model.to_string(),
            index_config: None,
        });
        self
    }

    /// 添加内存处理器
    ///
    /// # 参数
    /// - `processor`: 内存处理器实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use lumosai_core::memory::MessageLimitProcessor;
    ///
    /// let builder = Memory::composite()
    ///     .processor(Arc::new(MessageLimitProcessor::new(4000)));
    /// ```
    pub fn processor(mut self, processor: Arc<dyn MemoryProcessor>) -> Self {
        self.processors.push(processor);
        self
    }

    /// 批量添加内存处理器
    ///
    /// # 参数
    /// - `processors`: 处理器列表
    ///
    /// # 示例
    ///
    /// ```rust
    /// use lumosai_core::memory::{MessageLimitProcessor, DeduplicationProcessor};
    ///
    /// let builder = Memory::composite()
    ///     .processors(vec![
    ///         Arc::new(MessageLimitProcessor::new(4000)),
    ///         Arc::new(DeduplicationProcessor::new()),
    ///     ]);
    /// ```
    pub fn processors(mut self, processors: Vec<Arc<dyn MemoryProcessor>>) -> Self {
        self.processors.extend(processors);
        self
    }

    /// 设置命名空间
    ///
    /// # 参数
    /// - `namespace`: 命名空间名称
    pub fn namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    /// 构建 CompositeMemory 实例
    ///
    /// # 返回
    ///
    /// 返回配置好的 Memory 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// let memory = Memory::composite()
    ///     .working(1000)
    ///     .semantic("qdrant", "openai")
    ///     .build()
    ///     .await?;
    /// ```
    pub async fn build(self) -> Result<Memory> {
        // 创建工作内存（如果配置了）
        let working_memory_box = if let Some(config) = self.working_config.clone() {
            Some(create_working_memory(&config)?)
        } else {
            None
        };

        // 将 Box<dyn WorkingMemory> 转换为 Arc<dyn WorkingMemory>
        // 这里我们需要重新创建实例，因为 Box 和 Arc 不能直接转换
        let working_memory_arc = self.working_config.clone().map(|config| {
            Arc::new(crate::memory::working::BasicWorkingMemory::new(config))
                as Arc<dyn WorkingMemory>
        });

        // 创建语义内存（如果配置了）
        // 注意：这里需要实际的向量存储和嵌入提供商实例
        // 当前使用 None 作为占位符
        let semantic_memory: Option<Arc<dyn SemanticMemoryTrait>> = None;

        // 创建基础内存，组合工作内存和语义内存
        let basic_memory = BasicMemory::new(working_memory_arc.clone(), semantic_memory.clone());

        // 确定内存类型
        let memory_type = match (
            self.working_config.is_some(),
            self.semantic_config.is_some(),
        ) {
            (true, true) => MemoryType::Hybrid {
                working_size: self.working_config.as_ref().and_then(|c| c.max_capacity),
                enable_semantic: true,
            },
            (true, false) => MemoryType::Working {
                size: self
                    .working_config
                    .as_ref()
                    .and_then(|c| c.max_capacity)
                    .unwrap_or(1000),
            },
            (false, true) => MemoryType::Semantic,
            (false, false) => MemoryType::Basic,
        };

        Ok(Memory {
            inner: MemoryImpl::Hybrid {
                basic: basic_memory,
                working: working_memory_box,
                semantic: semantic_memory,
            },
            memory_type,
            thread_storage: None,
            processors: self.processors.clone(),
            processors_registered: AtomicBool::new(false),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{Message, Role};
    use crate::logger::NoopLogger;
    use crate::memory::processor::MessageLimitProcessor;
    use crate::memory::semantic_memory::{
        SemanticMemoryTrait, SemanticSearchOptions, SemanticSearchResult,
    };
    use crate::memory::thread::InMemoryThreadStorage;
    use crate::memory::{MemoryConfig, MessageRange, SemanticRecallConfig};
    use serde_json::json;
    use std::sync::Mutex;

    #[tokio::test]
    async fn unified_memory_applies_processors() -> Result<()> {
        let storage = Arc::new(InMemoryThreadStorage::new()) as Arc<dyn MemoryThreadStorage>;
        let processor = Arc::new(MessageLimitProcessor::new(
            1,
            Arc::new(NoopLogger::default()),
        ));
        let memory = Memory::basic()
            .add_processor(processor)
            .with_thread_storage(storage);

        let thread_id = "unified-thread";
        let first = Message::new(Role::User, "alpha".to_string(), None, None)
            .with_metadata("thread_id", json!(thread_id));
        memory.store(&first).await?;

        let second = Message::new(Role::User, "beta".to_string(), None, None)
            .with_metadata("thread_id", json!(thread_id));
        memory.store(&second).await?;

        let config = MemoryConfig {
            namespace: Some(thread_id.to_string()),
            last_messages: Some(10),
            ..Default::default()
        };

        let retrieved = memory.retrieve(&config).await?;
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].content, "beta");
        Ok(())
    }

    #[derive(Default)]
    struct MockSemanticMemory {
        messages: Mutex<Vec<Message>>,
    }

    #[async_trait::async_trait]
    impl SemanticMemoryTrait for MockSemanticMemory {
        async fn add(&self, message: &Message) -> Result<()> {
            let mut messages = self.messages.lock().unwrap();
            messages.push(message.clone());
            Ok(())
        }

        async fn search(
            &self,
            query: &str,
            options: &SemanticSearchOptions,
        ) -> Result<Vec<SemanticSearchResult>> {
            let messages = self.messages.lock().unwrap();
            let mut results = Vec::new();
            // 根据 query 过滤消息（简单的内容匹配）
            for message in messages.iter().rev() {
                if message.content.to_lowercase().contains(&query.to_lowercase()) {
                    results.push(SemanticSearchResult {
                        message: message.clone(),
                        score: 1.0,
                        context: None,
                    });
                    if results.len() >= options.limit {
                        break;
                    }
                }
            }
            Ok(results)
        }

        async fn get_recent(&self, limit: usize) -> Result<Vec<Message>> {
            let messages = self.messages.lock().unwrap();
            Ok(messages.iter().rev().take(limit).cloned().collect())
        }

        async fn get_context(
            &self,
            _message_id: &str,
            _before: usize,
            _after: usize,
        ) -> Result<Vec<Message>> {
            Ok(vec![])
        }

        async fn clear(&self) -> Result<()> {
            let mut messages = self.messages.lock().unwrap();
            messages.clear();
            Ok(())
        }
    }

    #[tokio::test]
    async fn hybrid_memory_combines_semantic_and_thread_messages() -> Result<()> {
        let storage = Arc::new(InMemoryThreadStorage::new()) as Arc<dyn MemoryThreadStorage>;
        let semantic = Arc::new(MockSemanticMemory::default()) as Arc<dyn SemanticMemoryTrait>;

        let memory = Memory {
            inner: MemoryImpl::Hybrid {
                basic: BasicMemory::with_thread_storage(None, None, Some(storage.clone())),
                working: None,
                semantic: Some(semantic.clone()),
            },
            memory_type: MemoryType::Hybrid {
                working_size: None,
                enable_semantic: true,
            },
            thread_storage: Some(storage.clone()),
            processors: Vec::new(),
            processors_registered: AtomicBool::new(true),
        };

        let thread_id = "hybrid-thread";
        let first = Message::new(Role::User, "vector reference".into(), None, None)
            .with_metadata("thread_id", json!(thread_id));
        let latest = Message::new(Role::Assistant, "latest summary".into(), None, None)
            .with_metadata("thread_id", json!(thread_id));

        memory.store(&first).await?;
        memory.store(&latest).await?;

        let recall = SemanticRecallConfig {
            top_k: 2,
            message_range: Some(MessageRange { before: 0, after: 0 }),
            generate_summaries: false,
            use_embeddings: true,
            max_capacity: None,
            max_results: None,
            relevance_threshold: None,
            template: None,
        };

        let config = MemoryConfig {
            namespace: Some(thread_id.to_string()),
            last_messages: Some(1),
            semantic_recall: Some(recall.clone()),
            query: Some("vector".to_string()),
            ..Default::default()
        };

        let retrieved = memory.retrieve(&config).await?;
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].content, "latest summary");
        assert_eq!(retrieved[1].content, "vector reference");

        let semantic_only = memory
            .semantic_recall("vector", &recall, Some(thread_id.to_string()))
            .await?;
        assert_eq!(semantic_only.len(), 1);
        assert_eq!(semantic_only[0].content, "vector reference");

        Ok(())
    }

    #[tokio::test]
    async fn unified_memory_thread_management() -> Result<()> {
        let storage = Arc::new(InMemoryThreadStorage::new()) as Arc<dyn MemoryThreadStorage>;
        let memory = Memory::basic().with_thread_storage(storage);

        // 创建线程
        let thread = memory
            .create_thread(CreateThreadParams {
                id: Some("unified-thread".to_string()),
                title: "Unified Test Thread".to_string(),
                agent_id: None,
                resource_id: Some("user-456".to_string()),
                metadata: None,
            })
            .await?;
        assert_eq!(thread.id, "unified-thread");
        assert_eq!(thread.title, "Unified Test Thread");

        // 获取线程
        let retrieved = memory.get_thread("unified-thread", Some("user-456")).await?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Unified Test Thread");

        // 更新线程
        let updated = memory
            .update_thread(
                "unified-thread",
                UpdateThreadParams {
                    title: Some("Updated Unified Title".to_string()),
                    metadata: None,
                },
                Some("user-456"),
            )
            .await?;
        assert_eq!(updated.title, "Updated Unified Title");

        // 列出线程
        let threads = memory.list_threads("user-456").await?;
        assert_eq!(threads.len(), 1);

        // 获取统计信息
        let stats = memory.get_thread_stats("unified-thread", Some("user-456")).await?;
        assert_eq!(stats.message_count, 0);

        // 删除线程
        memory.delete_thread("unified-thread", Some("user-456")).await?;
        let deleted = memory.get_thread("unified-thread", Some("user-456")).await?;
        assert!(deleted.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn unified_memory_thread_management_no_storage() -> Result<()> {
        let memory = Memory::basic();

        // 尝试在没有线程存储的情况下创建线程应该失败
        let result = memory
            .create_thread(CreateThreadParams {
                id: None,
                title: "Test".to_string(),
                agent_id: None,
                resource_id: None,
                metadata: None,
            })
            .await;
        assert!(result.is_err());

        Ok(())
    }
}
