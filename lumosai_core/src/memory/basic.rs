use async_trait::async_trait;
use serde_json::Value;
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
    /// 原始线程存储（可选，用于暴露给外部）
    thread_storage: Option<Arc<dyn MemoryThreadStorage>>,
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
        let thread_manager = thread_storage
            .as_ref()
            .map(|storage| MemoryThreadManager::new(storage.clone()));
        Self {
            working_memory,
            semantic_memory,
            thread_storage,
            thread_manager,
        }
    }

    /// 设置线程存储（可在构造后调用）
    pub fn set_thread_storage(&mut self, thread_storage: Option<Arc<dyn MemoryThreadStorage>>) {
        self.thread_storage = thread_storage.clone();
        self.thread_manager = thread_storage
            .as_ref()
            .map(|storage| MemoryThreadManager::new(storage.clone()));
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
        let thread_id = format!("thread-{resource_id}");
        if manager
            .get_thread(&thread_id, Some(resource_id))
            .await?
            .is_none()
        {
            manager
                .create_thread(CreateThreadParams {
                    id: Some(thread_id.clone()),
                    title: format!("Conversation with {resource_id}"),
                    agent_id: None,
                    resource_id: Some(resource_id.to_string()),
                    metadata: None,
                })
                .await?;
        }
        Ok(thread_id)
    }

    async fn ensure_thread_exists(
        manager: &MemoryThreadManager<Arc<dyn MemoryThreadStorage>>,
        thread_id: &str,
        resource_id: Option<&str>,
    ) -> Result<()> {
        if manager.get_thread(thread_id, resource_id).await?.is_none() {
            manager
                .create_thread(CreateThreadParams {
                    id: Some(thread_id.to_string()),
                    title: format!("Thread {thread_id}"),
                    agent_id: None,
                    resource_id: resource_id.map(|id| id.to_string()),
                    metadata: None,
                })
                .await?;
        }
        Ok(())
    }

    fn value_to_string(value: &Value) -> Option<String> {
        value.as_str().map(|s| s.to_string())
    }
}

#[async_trait]
impl Memory for BasicMemory {
    async fn store(&self, message: &Message) -> Result<()> {
        if let Some(thread_manager) = &self.thread_manager {
            let (thread_id_metadata, resource_id_metadata) = message
                .metadata
                .as_ref()
                .map(|meta| {
                    let thread_id = meta
                        .get("thread_id")
                        .and_then(Self::value_to_string);
                    let resource_id = meta
                        .get("resource_id")
                        .and_then(Self::value_to_string);
                    (thread_id, resource_id)
                })
                .unwrap_or((None, None));

            let resolved_thread_id = if let Some(thread_id) = thread_id_metadata.clone() {
                Self::ensure_thread_exists(
                    thread_manager,
                    &thread_id,
                    resource_id_metadata.as_deref(),
                )
                .await?;
                Some(thread_id)
            } else if let Some(resource_id) = resource_id_metadata.as_deref() {
                Some(
                    Self::get_or_create_thread_id(thread_manager, resource_id)
                        .await?,
                )
            } else {
                None
            };

            if let Some(thread_id) = resolved_thread_id {
                thread_manager
                    .add_message(&thread_id, message, resource_id_metadata.as_deref())
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
            let mut thread_id: Option<String> = None;
            let mut thread_owner: Option<String> = None;

            if let Some(target) = config.namespace.as_deref() {
                if let Some(thread) = thread_manager.get_thread(target, None).await? {
                    thread_owner = thread.resource_id.clone();
                    thread_id = Some(thread.id);
                } else if let Ok(threads) = thread_manager.list_threads(target).await {
                    if let Some(thread) = threads.first() {
                        thread_owner = thread.resource_id.clone();
                        thread_id = Some(thread.id.clone());
                    }
                }
            }

            if thread_id.is_none() {
                if let Some(resource_id) = config.store_id.as_deref() {
                    if let Ok(threads) = thread_manager.list_threads(resource_id).await {
                        if let Some(thread) = threads.first() {
                            thread_owner = thread.resource_id.clone();
                            thread_id = Some(thread.id.clone());
                        }
                    }
                }
            }

            if let Some(thread_id) = thread_id {
                let mut params = GetMessagesParams::default();
                if let Some(limit) = config
                    .last_messages
                    .and_then(|limit| if limit == 0 { None } else { Some(limit) })
                {
                    params.limit = Some(limit);
                }

                if let Ok(messages) = thread_manager
                    .get_messages(&thread_id, &params, thread_owner.as_deref())
                    .await
                {
                    results.extend(messages);
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

    fn as_thread_storage(&self) -> Option<Arc<dyn MemoryThreadStorage>> {
        self.thread_storage.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{Message, Role};
    use crate::memory::thread::{GetMessagesParams, InMemoryThreadStorage};
    use serde_json::json;

    fn build_test_memory() -> (BasicMemory, Arc<dyn MemoryThreadStorage>) {
        let storage = Arc::new(InMemoryThreadStorage::new()) as Arc<dyn MemoryThreadStorage>;
        let memory = BasicMemory::with_thread_storage(None, None, Some(storage.clone()));
        (memory, storage)
    }

    #[tokio::test]
    async fn store_message_with_thread_metadata() -> Result<()> {
        let (memory, storage) = build_test_memory();
        let message = Message::new(Role::User, "hello".to_string(), None, None)
            .with_metadata("thread_id", json!("thread-123"))
            .with_metadata("resource_id", json!("user-1"));

        memory.store(&message).await?;

        let manager = MemoryThreadManager::new(storage);
        let messages = manager
            .get_messages("thread-123", &GetMessagesParams::default(), Some("user-1"))
            .await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "hello");
        Ok(())
    }

    #[tokio::test]
    async fn store_message_with_only_resource_id_creates_thread() -> Result<()> {
        let (memory, storage) = build_test_memory();
        let message = Message::new(Role::User, "hello again".to_string(), None, None)
            .with_metadata("resource_id", json!("user-2"));

        memory.store(&message).await?;

        let manager = MemoryThreadManager::new(storage.clone());
        let threads = manager.list_threads("user-2").await?;
        assert_eq!(threads.len(), 1);
        let stored = manager
            .get_messages(&threads[0].id, &GetMessagesParams::default(), Some("user-2"))
            .await?;
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].content, "hello again");
        Ok(())
    }

    #[tokio::test]
    async fn retrieve_uses_thread_namespace() -> Result<()> {
        let (memory, storage) = build_test_memory();
        let message = Message::new(Role::User, "context".to_string(), None, None)
            .with_metadata("thread_id", json!("thread-abc"));
        memory.store(&message).await?;

        let config = MemoryConfig {
            namespace: Some("thread-abc".to_string()),
            last_messages: Some(10),
            ..Default::default()
        };

        let retrieved = memory.retrieve(&config).await?;
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].content, "context");

        // Ensure thread storage still exposes messages
        let manager = MemoryThreadManager::new(storage);
        let stored = manager
            .get_messages("thread-abc", &GetMessagesParams::default(), None)
            .await?;
        assert_eq!(stored.len(), 1);
        Ok(())
    }
}
