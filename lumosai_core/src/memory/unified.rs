//! 统一内存系统 - LumosAI v2.0 第四周任务
//!
//! 提供统一的内存接口，简化内存系统的使用和配置
//! 
//! # 设计目标
//! - 统一内存接口，减少抽象层次
//! - 提供简单的构造函数：basic(), semantic(), working()
//! - 智能默认配置，开箱即用
//! - 保持向后兼容性

use async_trait::async_trait;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::llm::{LlmProvider, Message};
use crate::memory::{
    BasicMemory, MemoryConfig, SemanticRecallConfig, WorkingMemoryConfig,
    create_working_memory, create_semantic_memory, WorkingMemory,
    semantic_memory::{SemanticMemoryTrait, SemanticSearchOptions},
    Memory as MemoryTrait,
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
}

impl Memory {
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
        let working_memory = create_working_memory(&config)
            .unwrap_or_else(|_| {
                // 如果创建失败，使用基础内存作为后备
                Box::new(crate::memory::working::BasicWorkingMemory::new(config.clone()))
            });

        Self {
            inner: MemoryImpl::Working(working_memory),
            memory_type: MemoryType::Working { size },
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
            create_working_memory(&config)
                .unwrap_or_else(|_| {
                    Box::new(crate::memory::working::BasicWorkingMemory::new(config.clone()))
                })
        });

        Self {
            inner: MemoryImpl::Hybrid {
                basic: basic_memory,
                working: working_memory,
                semantic: None, // 将在 with_llm() 中初始化
            },
            memory_type: MemoryType::Hybrid { working_size, enable_semantic },
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
            MemoryImpl::Basic(_) if matches!(self.memory_type, MemoryType::Semantic) => {
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
                
                let semantic_memory = create_semantic_memory(&config, llm)?;
                self.inner = MemoryImpl::Semantic(semantic_memory);
            },
            MemoryImpl::Hybrid { semantic, .. } if matches!(self.memory_type, MemoryType::Hybrid { enable_semantic: true, .. }) => {
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
            },
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
        match &self.inner {
            MemoryImpl::Basic(basic) => {
                basic.store(message).await
            },
            MemoryImpl::Semantic(semantic) => {
                semantic.add(message).await
            },
            MemoryImpl::Working(working) => {
                // 将消息序列化为JSON值存储到工作内存
                let message_value = serde_json::to_value(message)
                    .map_err(crate::error::Error::Json)?;
                working.set_value("last_message", message_value).await
            },
            MemoryImpl::Hybrid { basic, working, semantic } => {
                // 存储到基础内存
                basic.store(message).await?;

                // 存储到工作内存（如果存在）
                if let Some(working) = working {
                    let message_value = serde_json::to_value(message)
                        .map_err(crate::error::Error::Json)?;
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
        match &self.inner {
            MemoryImpl::Basic(basic) => {
                basic.retrieve(config).await
            },
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
            },
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
            },
            MemoryImpl::Hybrid { basic, working: _, semantic } => {
                // 优先从语义内存检索，然后是基础内存
                if let Some(semantic) = semantic {
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
                        return Ok(results.into_iter().map(|r| r.message).collect());
                    }
                }

                // 回退到基础内存
                basic.retrieve(config).await
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
            },
            _ => {
                // 对于其他类型的内存，目前不支持清空操作
                // 这是为了安全考虑，避免意外删除重要数据
                Err(crate::error::Error::UnsupportedOperation(
                    "清空操作仅支持工作内存".to_string()
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
