//! Memory Thread implementation for conversation management
//!
//! This module provides thread-based memory management similar to Mastra's Memory Thread concept.
//! It enables persistent storage of conversations with session isolation and message history management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::Error;
use crate::llm::Message;
use crate::memory::processor::{MemoryProcessor, MemoryProcessorOptions};
use crate::Result;

/// Memory thread for managing conversation history and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryThread {
    /// Unique thread identifier
    pub id: String,
    /// Human-readable title for the thread
    pub title: String,
    /// Optional agent ID that owns this thread
    pub agent_id: Option<String>,
    /// Resource ID that owns the thread (e.g., user ID, organization ID)
    pub resource_id: Option<String>,
    /// Additional metadata for the thread
    pub metadata: HashMap<String, Value>,
    /// When the thread was created
    pub created_at: DateTime<Utc>,
    /// When the thread was last updated
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct StoredMessage {
    id: String,
    message: Message,
    timestamp: DateTime<Utc>,
}

impl StoredMessage {
    fn new(message: &Message) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message: message.clone(),
            timestamp: Utc::now(),
        }
    }
}

/// Parameters for retrieving messages from a thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMessagesParams {
    /// Maximum number of messages to retrieve
    pub limit: Option<usize>,
    /// Cursor for pagination
    pub cursor: Option<String>,
    /// Filter criteria for messages
    pub filter: Option<MessageFilter>,
    /// Whether to include message content in response
    #[serde(default = "default_true")]
    pub include_content: bool,
    /// Whether to reverse the order (newest first)
    #[serde(default)]
    pub reverse_order: bool,
}

/// Filter criteria for message retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFilter {
    /// Filter by message role
    pub role: Option<String>,
    /// Filter by date range
    pub date_range: Option<DateRange>,
    /// Filter by content keywords
    pub keywords: Option<Vec<String>>,
    /// Filter by metadata
    pub metadata: Option<HashMap<String, Value>>,
}

/// Date range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    /// Start date (inclusive)
    pub start: DateTime<Utc>,
    /// End date (inclusive)
    pub end: DateTime<Utc>,
}

/// Parameters for creating a new thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateThreadParams {
    /// Optional thread ID (if not provided, one will be generated)
    pub id: Option<String>,
    /// Thread title
    pub title: String,
    /// Agent ID that owns this thread
    pub agent_id: Option<String>,
    /// Resource ID that owns the thread
    pub resource_id: Option<String>,
    /// Initial metadata
    pub metadata: Option<HashMap<String, Value>>,
}

/// Parameters for updating a thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateThreadParams {
    /// New title
    pub title: Option<String>,
    /// Updated metadata
    pub metadata: Option<HashMap<String, Value>>,
}

/// Memory options for thread operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryOptions {
    /// Whether to save messages to persistent storage
    #[serde(default = "default_true")]
    pub save_to_memory: bool,
    /// Whether to load context from thread history
    #[serde(default = "default_true")]
    pub load_context: bool,
    /// Maximum number of historical messages to include in context
    pub context_limit: Option<usize>,
    /// Whether to use semantic search for context retrieval
    #[serde(default)]
    pub use_semantic_search: bool,
    /// Working memory configuration
    pub working_memory: Option<crate::memory::working::WorkingMemoryConfig>,
}

/// Result of message operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageOperationResult {
    /// Number of messages affected
    pub affected_count: usize,
    /// Operation success status
    pub success: bool,
    /// Optional error message
    pub error_message: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for GetMessagesParams {
    fn default() -> Self {
        Self {
            limit: Some(50),
            cursor: None,
            filter: None,
            include_content: true,
            reverse_order: false,
        }
    }
}

impl Default for MemoryOptions {
    fn default() -> Self {
        Self {
            save_to_memory: true,
            load_context: true,
            context_limit: Some(10),
            use_semantic_search: false,
            working_memory: None,
        }
    }
}

impl MemoryThread {
    /// Create a new memory thread
    pub fn new(params: CreateThreadParams) -> Self {
        let now = Utc::now();
        Self {
            id: params.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            title: params.title,
            agent_id: params.agent_id,
            resource_id: params.resource_id,
            metadata: params.metadata.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Update thread metadata and title
    pub fn update(&mut self, params: UpdateThreadParams) -> Result<()> {
        if let Some(title) = params.title {
            self.title = title;
        }

        if let Some(metadata) = params.metadata {
            self.metadata.extend(metadata);
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Add a metadata entry
    pub fn add_metadata(&mut self, key: String, value: Value) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Remove a metadata entry
    pub fn remove_metadata(&mut self, key: &str) -> Option<Value> {
        self.updated_at = Utc::now();
        self.metadata.remove(key)
    }

    /// Get metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        self.metadata.get(key)
    }

    /// Check if thread is owned by the given resource
    pub fn is_owned_by(&self, resource_id: &str) -> bool {
        self.resource_id
            .as_ref()
            .is_some_and(|rid| rid == resource_id)
    }

    /// Check if thread belongs to the given agent
    pub fn belongs_to_agent(&self, agent_id: &str) -> bool {
        self.agent_id.as_ref().is_some_and(|aid| aid == agent_id)
    }
}

/// In-memory implementation of `MemoryThreadStorage`
#[derive(Debug, Clone, Default)]
pub struct InMemoryThreadStorage {
    threads: Arc<RwLock<HashMap<String, MemoryThread>>>,
    messages: Arc<RwLock<HashMap<String, Vec<StoredMessage>>>>,
}

impl InMemoryThreadStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self::default()
    }

    async fn ensure_thread_exists(&self, thread_id: &str) -> Result<()> {
        let threads = self.threads.read().await;
        if threads.contains_key(thread_id) {
            Ok(())
        } else {
            Err(Error::NotFound(format!(
                "Thread {thread_id} does not exist"
            )))
        }
    }

    fn matches_filter(stored: &StoredMessage, filter: &MessageFilter) -> bool {
        if let Some(role) = &filter.role {
            if stored.message.role.to_string().to_lowercase() != role.to_lowercase() {
                return false;
            }
        }

        if let Some(date_range) = &filter.date_range {
            if stored.timestamp < date_range.start || stored.timestamp > date_range.end {
                return false;
            }
        }

        if let Some(keywords) = &filter.keywords {
            let content_lower = stored.message.content.to_lowercase();
            let contains_keyword = keywords.iter().any(|keyword| {
                let keyword_lower = keyword.to_lowercase();
                content_lower.contains(&keyword_lower)
            });
            if !contains_keyword {
                return false;
            }
        }

        if let Some(metadata_filter) = &filter.metadata {
            let message_metadata = stored.message.metadata.as_ref();
            for (key, value) in metadata_filter {
                match message_metadata {
                    Some(metadata) if metadata.get(key) == Some(value) => {}
                    _ => return false,
                }
            }
        }

        true
    }

    fn apply_cursor(messages: Vec<StoredMessage>, cursor: &Option<String>) -> Vec<StoredMessage> {
        if let Some(cursor_id) = cursor {
            if cursor_id.is_empty() {
                return messages;
            }

            let position = messages.iter().position(|msg| &msg.id == cursor_id);
            if let Some(pos) = position {
                return messages.into_iter().skip(pos + 1).collect();
            }
        }
        messages
    }
}

/// Trait for memory thread storage
#[async_trait::async_trait]
pub trait MemoryThreadStorage: Send + Sync {
    /// Create a new thread
    async fn create_thread(&self, thread: &MemoryThread) -> Result<MemoryThread>;

    /// Get a thread by ID
    async fn get_thread(&self, thread_id: &str) -> Result<Option<MemoryThread>>;

    /// Update an existing thread
    async fn update_thread(&self, thread: &MemoryThread) -> Result<MemoryThread>;

    /// Delete a thread and all its messages
    async fn delete_thread(&self, thread_id: &str) -> Result<()>;

    /// List threads by resource ID
    async fn list_threads_by_resource(&self, resource_id: &str) -> Result<Vec<MemoryThread>>;

    /// List threads by agent ID
    async fn list_threads_by_agent(&self, agent_id: &str) -> Result<Vec<MemoryThread>>;

    /// Add a message to a thread
    async fn add_message(&self, thread_id: &str, message: &Message) -> Result<()>;

    /// Get messages from a thread
    async fn get_messages(
        &self,
        thread_id: &str,
        params: &GetMessagesParams,
    ) -> Result<Vec<Message>>;

    /// Delete messages from a thread
    async fn delete_messages(
        &self,
        thread_id: &str,
        message_ids: &[String],
    ) -> Result<MessageOperationResult>;

    /// Search messages across threads
    async fn search_messages(
        &self,
        query: &str,
        filter: Option<&MessageFilter>,
    ) -> Result<Vec<Message>>;

    /// Get thread statistics
    async fn get_thread_stats(&self, thread_id: &str) -> Result<ThreadStats>;
}

#[async_trait::async_trait]
impl<T> MemoryThreadStorage for Arc<T>
where
    T: MemoryThreadStorage + ?Sized,
{
    async fn create_thread(&self, thread: &MemoryThread) -> Result<MemoryThread> {
        (**self).create_thread(thread).await
    }

    async fn get_thread(&self, thread_id: &str) -> Result<Option<MemoryThread>> {
        (**self).get_thread(thread_id).await
    }

    async fn update_thread(&self, thread: &MemoryThread) -> Result<MemoryThread> {
        (**self).update_thread(thread).await
    }

    async fn delete_thread(&self, thread_id: &str) -> Result<()> {
        (**self).delete_thread(thread_id).await
    }

    async fn list_threads_by_resource(&self, resource_id: &str) -> Result<Vec<MemoryThread>> {
        (**self).list_threads_by_resource(resource_id).await
    }

    async fn list_threads_by_agent(&self, agent_id: &str) -> Result<Vec<MemoryThread>> {
        (**self).list_threads_by_agent(agent_id).await
    }

    async fn add_message(&self, thread_id: &str, message: &Message) -> Result<()> {
        (**self).add_message(thread_id, message).await
    }

    async fn get_messages(
        &self,
        thread_id: &str,
        params: &GetMessagesParams,
    ) -> Result<Vec<Message>> {
        (**self).get_messages(thread_id, params).await
    }

    async fn delete_messages(
        &self,
        thread_id: &str,
        message_ids: &[String],
    ) -> Result<MessageOperationResult> {
        (**self).delete_messages(thread_id, message_ids).await
    }

    async fn search_messages(
        &self,
        query: &str,
        filter: Option<&MessageFilter>,
    ) -> Result<Vec<Message>> {
        (**self).search_messages(query, filter).await
    }

    async fn get_thread_stats(&self, thread_id: &str) -> Result<ThreadStats> {
        (**self).get_thread_stats(thread_id).await
    }
}

#[async_trait::async_trait]
impl MemoryThreadStorage for InMemoryThreadStorage {
    async fn create_thread(&self, thread: &MemoryThread) -> Result<MemoryThread> {
        let mut threads = self.threads.write().await;
        let stored = thread.clone();
        threads.insert(stored.id.clone(), stored.clone());
        Ok(stored)
    }

    async fn get_thread(&self, thread_id: &str) -> Result<Option<MemoryThread>> {
        let threads = self.threads.read().await;
        Ok(threads.get(thread_id).cloned())
    }

    async fn update_thread(&self, thread: &MemoryThread) -> Result<MemoryThread> {
        let mut threads = self.threads.write().await;
        if !threads.contains_key(&thread.id) {
            return Err(Error::NotFound(format!("Thread {} not found", thread.id)));
        }
        let mut updated = thread.clone();
        updated.updated_at = Utc::now();
        threads.insert(updated.id.clone(), updated.clone());
        Ok(updated)
    }

    async fn delete_thread(&self, thread_id: &str) -> Result<()> {
        let mut threads = self.threads.write().await;
        if threads.remove(thread_id).is_none() {
            return Err(Error::NotFound(format!("Thread {thread_id} not found")));
        }

        let mut messages = self.messages.write().await;
        messages.remove(thread_id);
        Ok(())
    }

    async fn list_threads_by_resource(&self, resource_id: &str) -> Result<Vec<MemoryThread>> {
        let threads = self.threads.read().await;
        Ok(threads
            .values()
            .filter(|thread| {
                thread
                    .resource_id
                    .as_ref()
                    .is_some_and(|rid| rid == resource_id)
            })
            .cloned()
            .collect())
    }

    async fn list_threads_by_agent(&self, agent_id: &str) -> Result<Vec<MemoryThread>> {
        let threads = self.threads.read().await;
        Ok(threads
            .values()
            .filter(|thread| thread.agent_id.as_ref().is_some_and(|aid| aid == agent_id))
            .cloned()
            .collect())
    }

    async fn add_message(&self, thread_id: &str, message: &Message) -> Result<()> {
        self.ensure_thread_exists(thread_id).await?;

        let mut messages = self.messages.write().await;
        let entry = messages
            .entry(thread_id.to_string())
            .or_insert_with(Vec::new);
        entry.push(StoredMessage::new(message));
        Ok(())
    }

    async fn get_messages(
        &self,
        thread_id: &str,
        params: &GetMessagesParams,
    ) -> Result<Vec<Message>> {
        self.ensure_thread_exists(thread_id).await?;

        let messages_map = self.messages.read().await;
        let stored_messages = messages_map
            .get(thread_id)
            .cloned()
            .unwrap_or_else(Vec::new);
        drop(messages_map);

        let mut filtered: Vec<StoredMessage> = if let Some(filter) = &params.filter {
            stored_messages
                .into_iter()
                .filter(|msg| Self::matches_filter(msg, filter))
                .collect()
        } else {
            stored_messages
        };

        if params.reverse_order {
            filtered.reverse();
        }

        filtered = Self::apply_cursor(filtered, &params.cursor);

        if let Some(limit) = params.limit {
            if filtered.len() > limit {
                filtered.truncate(limit);
            }
        }

        let mut results = Vec::with_capacity(filtered.len());
        for stored in filtered {
            let mut message = stored.message.clone();
            if !params.include_content {
                message.content.clear();
            }
            results.push(message);
        }

        Ok(results)
    }

    async fn delete_messages(
        &self,
        thread_id: &str,
        message_ids: &[String],
    ) -> Result<MessageOperationResult> {
        self.ensure_thread_exists(thread_id).await?;
        let mut messages = self.messages.write().await;
        let entry = messages.entry(thread_id.to_string()).or_default();
        let before = entry.len();
        entry.retain(|message| !message_ids.contains(&message.id));
        let removed = before - entry.len();

        Ok(MessageOperationResult {
            affected_count: removed,
            success: removed > 0,
            error_message: if removed == 0 {
                Some("No messages were deleted".to_string())
            } else {
                None
            },
        })
    }

    async fn search_messages(
        &self,
        query: &str,
        filter: Option<&MessageFilter>,
    ) -> Result<Vec<Message>> {
        let messages = self.messages.read().await;
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for stored_messages in messages.values() {
            for stored in stored_messages {
                if !query_lower.is_empty()
                    && !stored.message.content.to_lowercase().contains(&query_lower)
                {
                    continue;
                }

                if let Some(filter) = filter {
                    if !Self::matches_filter(stored, filter) {
                        continue;
                    }
                }

                results.push(stored.message.clone());
            }
        }

        Ok(results)
    }

    async fn get_thread_stats(&self, thread_id: &str) -> Result<ThreadStats> {
        let threads = self.threads.read().await;
        let thread = threads
            .get(thread_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Thread {thread_id} not found")))?;
        drop(threads);

        let messages_map = self.messages.read().await;
        let stored_messages = messages_map
            .get(thread_id)
            .cloned()
            .unwrap_or_else(Vec::new);

        let mut message_count = 0;
        let mut user_count = 0;
        let mut assistant_count = 0;
        let mut size_bytes = 0;
        let mut last_message_at = None;

        for stored in stored_messages {
            message_count += 1;
            size_bytes += stored.message.content.len();
            last_message_at = Some(
                last_message_at.map_or(stored.timestamp, |current: DateTime<Utc>| {
                    current.max(stored.timestamp)
                }),
            );

            match stored.message.role {
                crate::llm::Role::User => user_count += 1,
                crate::llm::Role::Assistant => assistant_count += 1,
                _ => {}
            }
        }

        Ok(ThreadStats {
            message_count,
            user_message_count: user_count,
            assistant_message_count: assistant_count,
            created_at: thread.created_at,
            last_message_at,
            size_bytes,
        })
    }
}

/// Thread statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadStats {
    /// Total number of messages
    pub message_count: usize,
    /// Number of user messages
    pub user_message_count: usize,
    /// Number of assistant messages
    pub assistant_message_count: usize,
    /// Thread creation date
    pub created_at: DateTime<Utc>,
    /// Last message date
    pub last_message_at: Option<DateTime<Utc>>,
    /// Total size in bytes (approximate)
    pub size_bytes: usize,
}

/// Memory thread manager for high-level operations
pub struct MemoryThreadManager<S: MemoryThreadStorage> {
    storage: S,
    processors: Arc<RwLock<Vec<Arc<dyn MemoryProcessor>>>>,
}

impl<S: MemoryThreadStorage> fmt::Debug for MemoryThreadManager<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryThreadManager").finish()
    }
}

impl<S: MemoryThreadStorage> MemoryThreadManager<S> {
    /// Create a new memory thread manager
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            processors: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a memory processor to the pipeline
    pub async fn add_processor(&self, processor: Arc<dyn MemoryProcessor>) {
        let mut processors = self.processors.write().await;
        processors.push(processor);
    }

    /// Replace all processors with the provided list
    pub async fn set_processors(&self, new_processors: Vec<Arc<dyn MemoryProcessor>>) {
        let mut processors = self.processors.write().await;
        *processors = new_processors;
    }

    /// Apply registered processors to the provided messages
    async fn apply_processors(
        &self,
        messages: Vec<Message>,
        options: &MemoryProcessorOptions,
    ) -> Result<Vec<Message>> {
        let processors = self.processors.read().await.clone();
        let mut processed = messages;
        for processor in processors {
            processed = processor.process(processed, options).await?;
        }
        Ok(processed)
    }

    /// Public helper to process arbitrary message lists
    pub async fn process_messages(
        &self,
        messages: Vec<Message>,
        options: MemoryProcessorOptions,
    ) -> Result<Vec<Message>> {
        self.apply_processors(messages, &options).await
    }

    /// Create a new thread
    pub async fn create_thread(&self, params: CreateThreadParams) -> Result<MemoryThread> {
        let thread = MemoryThread::new(params);
        self.storage.create_thread(&thread).await
    }

    /// Get a thread by ID with ownership validation
    pub async fn get_thread(
        &self,
        thread_id: &str,
        resource_id: Option<&str>,
    ) -> Result<Option<MemoryThread>> {
        match self.storage.get_thread(thread_id).await? {
            Some(thread) => {
                if let Some(resource_id) = resource_id {
                    if !thread.is_owned_by(resource_id) {
                        return Err(Error::AccessDenied(format!(
                            "Thread {thread_id} is not owned by resource {resource_id}"
                        )));
                    }
                }
                Ok(Some(thread))
            }
            None => Ok(None),
        }
    }

    /// Update a thread with ownership validation
    pub async fn update_thread(
        &self,
        thread_id: &str,
        params: UpdateThreadParams,
        resource_id: Option<&str>,
    ) -> Result<MemoryThread> {
        let mut thread = self
            .get_thread(thread_id, resource_id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("Thread {thread_id} not found")))?;

        thread.update(params)?;
        self.storage.update_thread(&thread).await
    }

    /// Delete a thread with ownership validation
    pub async fn delete_thread(&self, thread_id: &str, resource_id: Option<&str>) -> Result<()> {
        if resource_id.is_some() {
            // Validate ownership before deletion
            self.get_thread(thread_id, resource_id).await?;
        }
        self.storage.delete_thread(thread_id).await
    }

    /// Add a message to a thread
    pub async fn add_message(
        &self,
        thread_id: &str,
        message: &Message,
        resource_id: Option<&str>,
    ) -> Result<()> {
        // Validate thread ownership if resource_id is provided
        if resource_id.is_some() {
            self.get_thread(thread_id, resource_id).await?;
        }
        self.storage.add_message(thread_id, message).await
    }

    /// Get messages from a thread with ownership validation
    pub async fn get_messages(
        &self,
        thread_id: &str,
        params: &GetMessagesParams,
        resource_id: Option<&str>,
    ) -> Result<Vec<Message>> {
        // Validate thread ownership if resource_id is provided
        if resource_id.is_some() {
            self.get_thread(thread_id, resource_id).await?;
        }
        let messages = self.storage.get_messages(thread_id, params).await?;
        let options = MemoryProcessorOptions {
            new_messages: Vec::new(),
            ..Default::default()
        };
        self.apply_processors(messages, &options).await
    }

    /// List threads for a resource
    pub async fn list_threads(&self, resource_id: &str) -> Result<Vec<MemoryThread>> {
        self.storage.list_threads_by_resource(resource_id).await
    }

    /// Get thread statistics
    pub async fn get_thread_stats(
        &self,
        thread_id: &str,
        resource_id: Option<&str>,
    ) -> Result<ThreadStats> {
        // Validate thread ownership if resource_id is provided
        if resource_id.is_some() {
            self.get_thread(thread_id, resource_id).await?;
        }
        self.storage.get_thread_stats(thread_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{Message, Role};
    use crate::logger::NoopLogger;
    use crate::memory::processor::{MessageLimitProcessor, RoleFilterProcessor};
    use serde_json::json;

    #[test]
    fn test_memory_thread_creation() {
        let params = CreateThreadParams {
            id: Some("test-thread".to_string()),
            title: "Test Thread".to_string(),
            agent_id: Some("agent-1".to_string()),
            resource_id: Some("user-123".to_string()),
            metadata: None,
        };

        let thread = MemoryThread::new(params);
        assert_eq!(thread.id, "test-thread");
        assert_eq!(thread.title, "Test Thread");
        assert_eq!(thread.agent_id, Some("agent-1".to_string()));
        assert_eq!(thread.resource_id, Some("user-123".to_string()));
    }

    #[test]
    fn test_thread_ownership() {
        let params = CreateThreadParams {
            id: None,
            title: "Test Thread".to_string(),
            agent_id: Some("agent-1".to_string()),
            resource_id: Some("user-123".to_string()),
            metadata: None,
        };

        let thread = MemoryThread::new(params);
        assert!(thread.is_owned_by("user-123"));
        assert!(!thread.is_owned_by("user-456"));
        assert!(thread.belongs_to_agent("agent-1"));
        assert!(!thread.belongs_to_agent("agent-2"));
    }

    #[test]
    fn test_thread_metadata() {
        let params = CreateThreadParams {
            id: None,
            title: "Test Thread".to_string(),
            agent_id: None,
            resource_id: None,
            metadata: None,
        };

        let mut thread = MemoryThread::new(params);
        thread.add_metadata("key1".to_string(), Value::String("value1".to_string()));
        thread.add_metadata("key2".to_string(), Value::Number(42.into()));

        assert_eq!(
            thread.get_metadata("key1"),
            Some(&Value::String("value1".to_string()))
        );
        assert_eq!(thread.get_metadata("key2"), Some(&Value::Number(42.into())));

        let removed = thread.remove_metadata("key1");
        assert_eq!(removed, Some(Value::String("value1".to_string())));
        assert_eq!(thread.get_metadata("key1"), None);
    }

    #[tokio::test]
    async fn test_in_memory_thread_storage_crud() -> Result<()> {
        let storage = InMemoryThreadStorage::new();
        let params = CreateThreadParams {
            id: None,
            title: "Thread One".to_string(),
            agent_id: Some("agent-a".to_string()),
            resource_id: Some("resource-1".to_string()),
            metadata: None,
        };

        let thread = MemoryThread::new(params);
        let created = storage.create_thread(&thread).await?;
        assert!(!created.id.is_empty());

        let fetched = storage.get_thread(&created.id).await?;
        assert!(fetched.is_some());

        let by_resource = storage.list_threads_by_resource("resource-1").await?;
        assert_eq!(by_resource.len(), 1);

        let updated = MemoryThread {
            title: "Updated Thread".to_string(),
            ..created.clone()
        };
        let updated_thread = storage.update_thread(&updated).await?;
        assert_eq!(updated_thread.title, "Updated Thread");

        storage.delete_thread(&created.id).await?;
        let missing = storage.get_thread(&created.id).await?;
        assert!(missing.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_in_memory_thread_storage_messages() -> Result<()> {
        let storage = InMemoryThreadStorage::new();
        let thread = MemoryThread::new(CreateThreadParams {
            id: None,
            title: "Message Thread".to_string(),
            agent_id: None,
            resource_id: Some("resource-2".to_string()),
            metadata: None,
        });
        storage.create_thread(&thread).await?;

        let user_message = Message {
            role: Role::User,
            content: "Hello assistant".to_string(),
            metadata: Some(HashMap::from([("topic".to_string(), json!("greeting"))])),
            name: None,
        };
        storage.add_message(&thread.id, &user_message).await?;

        let assistant_message = Message {
            role: Role::Assistant,
            content: "Hello user".to_string(),
            metadata: None,
            name: None,
        };
        storage.add_message(&thread.id, &assistant_message).await?;

        let messages = storage
            .get_messages(&thread.id, &GetMessagesParams::default())
            .await?;
        assert_eq!(messages.len(), 2);

        let user_only = storage
            .get_messages(
                &thread.id,
                &GetMessagesParams {
                    filter: Some(MessageFilter {
                        role: Some("user".to_string()),
                        date_range: None,
                        keywords: None,
                        metadata: None,
                    }),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(user_only.len(), 1);
        assert_eq!(user_only[0].role, Role::User);

        let stats = storage.get_thread_stats(&thread.id).await?;
        assert_eq!(stats.message_count, 2);
        assert_eq!(stats.user_message_count, 1);
        assert_eq!(stats.assistant_message_count, 1);
        assert!(stats.last_message_at.is_some());

        let keyword_search = storage
            .search_messages(
                "hello",
                Some(&MessageFilter {
                    role: Some("assistant".to_string()),
                    date_range: None,
                    keywords: None,
                    metadata: None,
                }),
            )
            .await?;
        assert_eq!(keyword_search.len(), 1);
        assert_eq!(keyword_search[0].role, Role::Assistant);

        let message_ids = {
            let messages_map = storage.messages.read().await;
            messages_map
                .get(&thread.id)
                .unwrap()
                .iter()
                .map(|msg| msg.id.clone())
                .collect::<Vec<_>>()
        };

        let result = storage
            .delete_messages(&thread.id, &[message_ids[0].clone()])
            .await?;
        assert_eq!(result.affected_count, 1);
        assert!(result.success);

        Ok(())
    }

    #[tokio::test]
    async fn test_memory_thread_manager_with_storage() -> Result<()> {
        let storage = InMemoryThreadStorage::new();
        let manager = MemoryThreadManager::new(storage.clone());

        let thread = manager
            .create_thread(CreateThreadParams {
                id: None,
                title: "Manager Thread".to_string(),
                agent_id: Some("agent-manager".to_string()),
                resource_id: Some("resource-manager".to_string()),
                metadata: None,
            })
            .await?;

        let retrieved = manager
            .get_thread(&thread.id, Some("resource-manager"))
            .await?;
        assert!(retrieved.is_some());

        // Ownership validation
        let err = manager
            .get_thread(&thread.id, Some("other-resource"))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::AccessDenied(_)));

        Ok(())
    }

    #[tokio::test]
    async fn test_memory_thread_manager_processors() -> Result<()> {
        let storage = InMemoryThreadStorage::new();
        let manager = MemoryThreadManager::new(storage.clone());
        let logger = Arc::new(NoopLogger::default());

        manager
            .add_processor(Arc::new(RoleFilterProcessor::new(
                vec![Role::User],
                logger.clone(),
            )))
            .await;
        manager
            .add_processor(Arc::new(MessageLimitProcessor::new(1, logger.clone())))
            .await;

        let thread = manager
            .create_thread(CreateThreadParams {
                id: None,
                title: "Processor Thread".to_string(),
                agent_id: None,
                resource_id: Some("resource-processor".to_string()),
                metadata: None,
            })
            .await?;

        let user_message = Message {
            role: Role::User,
            content: "First message".to_string(),
            metadata: None,
            name: None,
        };
        let assistant_message = Message {
            role: Role::Assistant,
            content: "Second message".to_string(),
            metadata: None,
            name: None,
        };

        storage.add_message(&thread.id, &user_message).await?;
        storage.add_message(&thread.id, &assistant_message).await?;

        let messages = manager
            .get_messages(&thread.id, &GetMessagesParams::default(), None)
            .await?;

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, Role::User);
        assert_eq!(messages[0].content, "First message");

        // Also test process_messages helper
        let processed = manager
            .process_messages(
                vec![assistant_message.clone(), user_message.clone()],
                MemoryProcessorOptions::default(),
            )
            .await?;
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].role, Role::User);

        Ok(())
    }
}
