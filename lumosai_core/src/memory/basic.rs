use async_trait::async_trait;
use std::sync::Arc;

use crate::error::Result;
use crate::llm::Message;
use crate::memory::processor::MemoryProcessor;
use crate::memory::semantic_memory::{SemanticMemoryTrait, SemanticSearchOptions};
use crate::memory::thread::{
    CreateThreadParams, GetMessagesParams, MemoryThread, MemoryThreadManager, MemoryThreadStorage,
};
use crate::memory::working::WorkingMemory;
use crate::memory::{Memory, MemoryConfig};

/// BasicMemory 是一个简单的内存实现
pub struct BasicMemory {
    /// 内部工作内存
    working_memory: Option<Arc<dyn WorkingMemory>>,
    /// 内部语义内存
    semantic_memory: Option<Arc<dyn SemanticMemoryTrait>>,
    /// 线程管理器（可选）
    thread_manager: Option<MemoryThreadManager<Arc<dyn MemoryThreadStorage>>>,
}

impl BasicMemory {
    /// 创建一个新的基本内存实例
    pub fn new(
        working_memory: Option<Arc<dyn WorkingMemory>>,
        semantic_memory: Option<Arc<dyn SemanticMemoryTrait>>,
    ) -> Self {
        Self::with_thread_storage(working_memory, semantic_memory, None)
    }

    /// 创建带线程存储的基本内存实例
    pub fn with_thread_storage(
        working_memory: Option<Arc<dyn WorkingMemory>>,
        semantic_memory: Option<Arc<dyn SemanticMemoryTrait>>,
        thread_storage: Option<Arc<dyn MemoryThreadStorage>>,
    ) -> Self {
        Self {
            working_memory,
            semantic_memory,
            thread_manager: thread_storage.map(MemoryThreadManager::new),
        }
    }

    /// 设置线程存储（可在构造后调用）
    pub fn set_thread_storage(&mut self, thread_storage: Option<Arc<dyn MemoryThreadStorage>>) {
        self.thread_manager = thread_storage.map(MemoryThreadManager::new);
    }

    /// 注册 Processor
    pub async fn add_processor(&self, processor: Arc<dyn MemoryProcessor>) -> Result<()> {
        if let Some(manager) = &self.thread_manager {
            manager.add_processor(processor).await;
        }
        Ok(())
    }

    async fn get_or_create_thread_id(
        manager: &MemoryThreadManager<Arc<dyn MemoryThreadStorage>>,
        resource_id: &str,
    ) -> Result<String> {
        let existing = manager.list_threads(resource_id).await?;
        if let Some(thread) = existing.first() {
            Ok(thread.id.clone())
        } else {
            let thread = manager
                .create_thread(CreateThreadParams {
                    id: None,
                    title: format!("thread-{resource_id}"),
                    agent_id: None,
                    resource_id: Some(resource_id.to_string()),
                    metadata: None,
                })
                .await?;
            Ok(thread.id)
        }
    }
}

#[async_trait]
impl Memory for BasicMemory {
    async fn store(&self, message: &Message) -> Result<()> {
        if let Some(thread_manager) = &self.thread_manager {
            // Default behavior: store message in a default thread (per resource)
            if let Some(resource_id) = message
                .metadata
                .as_ref()
                .and_then(|meta| meta.get("resource_id"))
                .and_then(|value| value.as_str())
            {
                let params = CreateThreadParams {
                    id: None,
                    title: format!("default-thread-{}", resource_id),
                    agent_id: None,
                    resource_id: Some(resource_id.to_string()),
                    metadata: None,
                };

                let thread = thread_manager.create_thread(params).await?;
                thread_manager
                    .add_message(&thread.id, message, Some(resource_id))
                    .await?;
            }
        }

        if let Some(ref working_memory) = self.working_memory {
            // 将消息序列化为JSON值
            let message_value = serde_json::to_value(message).map_err(crate::error::Error::Json)?;

            working_memory
                .set_value("last_message", message_value)
                .await?;
        }

        if let Some(ref semantic_memory) = self.semantic_memory {
            semantic_memory.add(message).await?;
        }

        Ok(())
    }

    async fn retrieve(&self, config: &MemoryConfig) -> Result<Vec<Message>> {
        let mut results = Vec::new();

        if let Some(thread_manager) = &self.thread_manager {
            if let Some(resource_id) = config.namespace.as_deref() {
                let params = GetMessagesParams::default();
                if let Ok(thread_list) = thread_manager.list_threads(resource_id).await {
                    if let Some(thread) = thread_list.first() {
                        if let Ok(messages) = thread_manager
                            .get_messages(&thread.id, &params, Some(resource_id))
                            .await
                        {
                            results.extend(messages);
                        }
                    }
                }
            }
        }

        if let Some(ref semantic_memory) = self.semantic_memory {
            if let Some(ref semantic_recall) = config.semantic_recall {
                if let Some(ref query) = config.query {
                    // 使用默认选项创建搜索配置
                    let mut options = SemanticSearchOptions::default();

                    // 使用MemoryConfig中的相关配置
                    options.limit = semantic_recall.top_k;
                    options.threshold = semantic_recall.relevance_threshold;
                    options.namespace = config.namespace.clone();

                    // 执行搜索
                    let search_results = semantic_memory.search(query, &options).await?;

                    for result in search_results {
                        results.push(result.message);
                    }
                }
            }
        }

        Ok(results)
    }
}
