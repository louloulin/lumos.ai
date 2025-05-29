//! Memory Thread implementation for conversation management
//! 
//! This module provides thread-based memory management similar to Mastra's Memory Thread concept.
//! It enables persistent storage of conversations with session isolation and message history management.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::llm::Message;
use crate::Result;
use crate::error::Error;

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
        self.resource_id.as_ref().map_or(false, |rid| rid == resource_id)
    }

    /// Check if thread belongs to the given agent
    pub fn belongs_to_agent(&self, agent_id: &str) -> bool {
        self.agent_id.as_ref().map_or(false, |aid| aid == agent_id)
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
    async fn get_messages(&self, thread_id: &str, params: &GetMessagesParams) -> Result<Vec<Message>>;

    /// Delete messages from a thread
    async fn delete_messages(&self, thread_id: &str, message_ids: &[String]) -> Result<MessageOperationResult>;

    /// Search messages across threads
    async fn search_messages(&self, query: &str, filter: Option<&MessageFilter>) -> Result<Vec<Message>>;

    /// Get thread statistics
    async fn get_thread_stats(&self, thread_id: &str) -> Result<ThreadStats>;
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
#[derive(Debug)]
pub struct MemoryThreadManager<S: MemoryThreadStorage> {
    storage: S,
}

impl<S: MemoryThreadStorage> MemoryThreadManager<S> {
    /// Create a new memory thread manager
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Create a new thread
    pub async fn create_thread(&self, params: CreateThreadParams) -> Result<MemoryThread> {
        let thread = MemoryThread::new(params);
        self.storage.create_thread(&thread).await
    }

    /// Get a thread by ID with ownership validation
    pub async fn get_thread(&self, thread_id: &str, resource_id: Option<&str>) -> Result<Option<MemoryThread>> {
        match self.storage.get_thread(thread_id).await? {
            Some(thread) => {
                if let Some(resource_id) = resource_id {
                    if !thread.is_owned_by(resource_id) {
                        return Err(Error::AccessDenied(format!(
                            "Thread {} is not owned by resource {}",
                            thread_id, resource_id
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
            .ok_or_else(|| Error::NotFound(format!("Thread {} not found", thread_id)))?;

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
        self.storage.get_messages(thread_id, params).await
    }

    /// List threads for a resource
    pub async fn list_threads(&self, resource_id: &str) -> Result<Vec<MemoryThread>> {
        self.storage.list_threads_by_resource(resource_id).await
    }

    /// Get thread statistics
    pub async fn get_thread_stats(&self, thread_id: &str, resource_id: Option<&str>) -> Result<ThreadStats> {
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

        assert_eq!(thread.get_metadata("key1"), Some(&Value::String("value1".to_string())));
        assert_eq!(thread.get_metadata("key2"), Some(&Value::Number(42.into())));

        let removed = thread.remove_metadata("key1");
        assert_eq!(removed, Some(Value::String("value1".to_string())));
        assert_eq!(thread.get_metadata("key1"), None);
    }
}
