use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};
use crate::llm::Message;
use crate::memory::processor::MemoryProcessor;
use crate::memory::semantic_memory::{SemanticMemoryTrait, SemanticSearchOptions};
use crate::memory::thread::{
    CreateThreadParams, GetMessagesParams, MemoryThread, MemoryThreadManager, MemoryThreadStorage,
    ThreadStats, UpdateThreadParams,
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
    /// 已注册的处理器
    processors: Arc<Mutex<Vec<Arc<dyn MemoryProcessor>>>>,
    /// 已同步到线程管理器的处理器数量
    applied_processors: AtomicUsize,
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
            processors: Arc::new(Mutex::new(Vec::new())),
            applied_processors: AtomicUsize::new(0),
        }
    }

    /// 设置线程存储（可在构造后调用）
    pub fn set_thread_storage(&mut self, thread_storage: Option<Arc<dyn MemoryThreadStorage>>) {
        self.thread_storage = thread_storage.clone();
        self.thread_manager = thread_storage
            .as_ref()
            .map(|storage| MemoryThreadManager::new(storage.clone()));
        self.applied_processors.store(0, Ordering::SeqCst);
    }

    /// 注册 Processor
    pub async fn add_processor(&self, processor: Arc<dyn MemoryProcessor>) -> Result<()> {
        {
            let mut processors = self.processors.lock().unwrap();
            processors.push(processor.clone());
        }

        if let Some(manager) = &self.thread_manager {
            manager.add_processor(processor).await;
            self.applied_processors.fetch_add(1, Ordering::SeqCst);
        }
        Ok(())
    }

    async fn apply_pending_processors(&self) -> Result<()> {
        if self.thread_manager.is_none() {
            return Ok(());
        }

        let applied = self.applied_processors.load(Ordering::SeqCst);
        let processors = {
            let processors = self.processors.lock().unwrap();
            processors.clone()
        };

        if applied >= processors.len() {
            return Ok(());
        }

        if let Some(manager) = &self.thread_manager {
            for processor in processors.iter().skip(applied) {
                manager.add_processor(processor.clone()).await;
                self.applied_processors.fetch_add(1, Ordering::SeqCst);
            }
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
}

#[async_trait]
impl Memory for BasicMemory {
    async fn store(&self, message: &Message) -> Result<()> {
        self.apply_pending_processors().await?;

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
        self.apply_pending_processors().await?;

        let mut thread_results = Vec::new();
        let mut semantic_results = Vec::new();

        // 先获取线程历史消息
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
                    params.reverse_order = true; // 获取最新的消息
                }

                if let Ok(messages) = thread_manager
                    .get_messages(&thread_id, &params, thread_owner.as_deref())
                    .await
                {
                    thread_results.extend(messages);
                }
            }
        }

        // 然后获取语义召回结果
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
                        semantic_results.push(result.message);
                    }
                }
            }
        }

        // 合并结果：线程历史消息在前（按时间顺序），语义召回结果在后
        let mut combined = thread_results;
        combined.extend(semantic_results);

        Ok(Self::dedup_messages(combined))
    }

    fn as_thread_storage(&self) -> Option<Arc<dyn MemoryThreadStorage>> {
        self.thread_storage.clone()
    }

    async fn create_thread(
        &self,
        params: CreateThreadParams,
    ) -> Result<MemoryThread> {
        self.thread_manager
            .as_ref()
            .ok_or_else(|| {
                Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?
            .create_thread(params)
            .await
    }

    async fn get_thread(
        &self,
        thread_id: &str,
        resource_id: Option<&str>,
    ) -> Result<Option<MemoryThread>> {
        self.thread_manager
            .as_ref()
            .ok_or_else(|| {
                Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?
            .get_thread(thread_id, resource_id)
            .await
    }

    async fn update_thread(
        &self,
        thread_id: &str,
        params: UpdateThreadParams,
        resource_id: Option<&str>,
    ) -> Result<MemoryThread> {
        self.thread_manager
            .as_ref()
            .ok_or_else(|| {
                Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?
            .update_thread(thread_id, params, resource_id)
            .await
    }

    async fn delete_thread(
        &self,
        thread_id: &str,
        resource_id: Option<&str>,
    ) -> Result<()> {
        self.thread_manager
            .as_ref()
            .ok_or_else(|| {
                Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?
            .delete_thread(thread_id, resource_id)
            .await
    }

    async fn list_threads(&self, resource_id: &str) -> Result<Vec<MemoryThread>> {
        self.thread_manager
            .as_ref()
            .ok_or_else(|| {
                Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?
            .list_threads(resource_id)
            .await
    }

    async fn get_thread_stats(
        &self,
        thread_id: &str,
        resource_id: Option<&str>,
    ) -> Result<ThreadStats> {
        self.thread_manager
            .as_ref()
            .ok_or_else(|| {
                Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?
            .get_thread_stats(thread_id, resource_id)
            .await
    }

    async fn semantic_recall(
        &self,
        query: &str,
        config: &crate::memory::SemanticRecallConfig,
        namespace: Option<String>,
    ) -> Result<Vec<Message>> {
        if let Some(ref semantic_memory) = self.semantic_memory {
            let mut options = SemanticSearchOptions::default();
            options.limit = config.top_k;
            options.threshold = config.relevance_threshold;
            options.namespace = namespace;
            if let Some(range) = &config.message_range {
                options.use_window = true;
                options.window_size = Some((range.before, range.after));
            } else {
                options.use_window = false;
                options.window_size = None;
            }
            let results = semantic_memory.search(query, &options).await?;
            Ok(results.into_iter().map(|r| r.message).collect())
        } else {
            Err(Error::UnsupportedOperation(
                "Semantic memory not enabled for this BasicMemory instance".to_string(),
            ))
        }
    }

    async fn add_processor(&self, processor: Arc<dyn MemoryProcessor>) -> Result<()> {
        // 调用 BasicMemory 的公共方法（避免递归）
        {
            let mut processors = self.processors.lock().unwrap();
            processors.push(processor.clone());
        }

        if let Some(manager) = &self.thread_manager {
            manager.add_processor(processor).await;
            self.applied_processors.fetch_add(1, Ordering::SeqCst);
        }
        Ok(())
    }

    async fn process_messages(&self, messages: Vec<Message>) -> Result<Vec<Message>> {
        use crate::memory::processor::MemoryProcessorOptions;
        
        // 如果有 thread_manager，使用它的 process_messages 方法
        if let Some(manager) = &self.thread_manager {
            let options = MemoryProcessorOptions::default();
            manager.process_messages(messages, options).await
        } else {
            // 否则直接返回消息
            Ok(messages)
        }
    }
}

impl BasicMemory {
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
    /// use lumosai_core::memory::{BasicMemory, CreateThreadParams};
    ///
    /// # async fn example(memory: BasicMemory) -> lumosai_core::Result<()> {
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
        self.thread_manager
            .as_ref()
            .ok_or_else(|| {
                Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?
            .create_thread(params)
            .await
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
    /// # async fn example(memory: BasicMemory) -> lumosai_core::Result<()> {
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
        self.thread_manager
            .as_ref()
            .ok_or_else(|| {
                Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?
            .get_thread(thread_id, resource_id)
            .await
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
    /// use lumosai_core::memory::UpdateThreadParams;
    ///
    /// # async fn example(memory: BasicMemory) -> lumosai_core::Result<()> {
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
        self.thread_manager
            .as_ref()
            .ok_or_else(|| {
                Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?
            .update_thread(thread_id, params, resource_id)
            .await
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
    /// # async fn example(memory: BasicMemory) -> lumosai_core::Result<()> {
    /// memory.delete_thread("thread-123", Some("user-123")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_thread(
        &self,
        thread_id: &str,
        resource_id: Option<&str>,
    ) -> Result<()> {
        self.thread_manager
            .as_ref()
            .ok_or_else(|| {
                Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?
            .delete_thread(thread_id, resource_id)
            .await
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
    /// # async fn example(memory: BasicMemory) -> lumosai_core::Result<()> {
    /// let threads = memory.list_threads("user-123").await?;
    /// println!("Found {} threads", threads.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_threads(&self, resource_id: &str) -> Result<Vec<MemoryThread>> {
        self.thread_manager
            .as_ref()
            .ok_or_else(|| {
                Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?
            .list_threads(resource_id)
            .await
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
    /// # async fn example(memory: BasicMemory) -> lumosai_core::Result<()> {
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
        self.thread_manager
            .as_ref()
            .ok_or_else(|| {
                Error::Configuration(
                    "Thread storage not configured. Use with_thread_storage() first.".to_string(),
                )
            })?
            .get_thread_stats(thread_id, resource_id)
            .await
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
    use crate::memory::thread::{GetMessagesParams, InMemoryThreadStorage};
    use crate::memory::{MemoryConfig, MessageRange, SemanticRecallConfig};
    use serde_json::json;
    use std::sync::Mutex;

    fn build_test_memory() -> (BasicMemory, Arc<dyn MemoryThreadStorage>) {
        let storage = Arc::new(InMemoryThreadStorage::new()) as Arc<dyn MemoryThreadStorage>;
        let memory = BasicMemory::with_thread_storage(None, None, Some(storage.clone()));
        (memory, storage)
    }

    fn build_memory_with_semantic(
        semantic: Arc<dyn SemanticMemoryTrait>,
    ) -> (BasicMemory, Arc<dyn MemoryThreadStorage>) {
        let storage = Arc::new(InMemoryThreadStorage::new()) as Arc<dyn MemoryThreadStorage>;
        let memory =
            BasicMemory::with_thread_storage(None, Some(semantic), Some(storage.clone()));
        (memory, storage)
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
            _query: &str,
            options: &SemanticSearchOptions,
        ) -> Result<Vec<SemanticSearchResult>> {
            let messages = self.messages.lock().unwrap();
            let mut results = Vec::new();
            for message in messages.iter().rev().take(options.limit) {
                results.push(SemanticSearchResult {
                    message: message.clone(),
                    score: 1.0,
                    context: None,
                });
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

    #[tokio::test]
    async fn processors_limit_messages() -> Result<()> {
        let (memory, _) = build_test_memory();

        let processor = Arc::new(MessageLimitProcessor::new(
            1,
            Arc::new(NoopLogger::default()),
        ));
        memory.add_processor(processor).await?;

        let thread_id = "thread-limit";

        let first = Message::new(Role::User, "first".to_string(), None, None)
            .with_metadata("thread_id", json!(thread_id));
        memory.store(&first).await?;

        let second = Message::new(Role::User, "second".to_string(), None, None)
            .with_metadata("thread_id", json!(thread_id));
        memory.store(&second).await?;

        let config = MemoryConfig {
            namespace: Some(thread_id.to_string()),
            last_messages: Some(10),
            ..Default::default()
        };

        let retrieved = memory.retrieve(&config).await?;
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].content, "second");
        Ok(())
    }

    #[tokio::test]
    async fn semantic_recall_merges_with_thread_history() -> Result<()> {
        let semantic = Arc::new(MockSemanticMemory::default()) as Arc<dyn SemanticMemoryTrait>;
        let (memory, _) = build_memory_with_semantic(semantic);

        let thread_id = "semantic-thread";
        let earlier = Message::new(Role::User, "vector embeddings overview".into(), None, None)
            .with_metadata("thread_id", json!(thread_id));
        let latest = Message::new(Role::Assistant, "recent update".into(), None, None)
            .with_metadata("thread_id", json!(thread_id));

        memory.store(&earlier).await?;
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
            semantic_recall: Some(recall),
            query: Some("vector".to_string()),
            ..Default::default()
        };

        let retrieved = memory.retrieve(&config).await?;
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].content, "recent update");
        assert_eq!(retrieved[1].content, "vector embeddings overview");
        Ok(())
    }

    // Remove duplicate methods from impl BasicMemory block since they're now in impl Memory
    // The methods below are kept for backward compatibility but delegate to Memory trait methods

    #[tokio::test]
    async fn test_basic_memory_thread_management() -> Result<()> {
        let storage = Arc::new(InMemoryThreadStorage::default());
        let mut memory = BasicMemory::with_thread_storage(None, None, Some(storage.clone()));

        // 创建线程
        let thread = memory
            .create_thread(CreateThreadParams {
                id: Some("test-thread".to_string()),
                title: "Test Thread".to_string(),
                agent_id: None,
                resource_id: Some("user-123".to_string()),
                metadata: None,
            })
            .await?;
        assert_eq!(thread.id, "test-thread");
        assert_eq!(thread.title, "Test Thread");

        // 获取线程
        let retrieved = memory.get_thread("test-thread", Some("user-123")).await?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test Thread");

        // 更新线程
        let updated = memory
            .update_thread(
                "test-thread",
                UpdateThreadParams {
                    title: Some("Updated Title".to_string()),
                    metadata: None,
                },
                Some("user-123"),
            )
            .await?;
        assert_eq!(updated.title, "Updated Title");

        // 列出线程
        let threads = memory.list_threads("user-123").await?;
        assert_eq!(threads.len(), 1);

        // 获取统计信息
        let stats = memory.get_thread_stats("test-thread", Some("user-123")).await?;
        assert_eq!(stats.message_count, 0);

        // 删除线程
        memory.delete_thread("test-thread", Some("user-123")).await?;
        let deleted = memory.get_thread("test-thread", Some("user-123")).await?;
        assert!(deleted.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_basic_memory_thread_management_no_storage() -> Result<()> {
        let memory = BasicMemory::new(None, None);

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
