//! In-memory storage implementation for Memory Threads
//! 
//! This module provides a simple in-memory storage implementation for testing
//! and development purposes. For production use, implement MemoryThreadStorage
//! with a persistent storage backend.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::llm::Message;
use crate::Result;
use crate::error::LumosError;
use super::thread::{
    MemoryThread, MemoryThreadStorage, GetMessagesParams, MessageFilter, 
    MessageOperationResult, ThreadStats
};

/// In-memory storage for memory threads
#[derive(Debug, Default)]
pub struct InMemoryThreadStorage {
    threads: Arc<RwLock<HashMap<String, MemoryThread>>>,
    messages: Arc<RwLock<HashMap<String, Vec<StoredMessage>>>>, // thread_id -> messages
}

/// Stored message with additional metadata
#[derive(Debug, Clone)]
struct StoredMessage {
    message: Message,
    stored_at: DateTime<Utc>,
    message_id: String,
}

impl InMemoryThreadStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all data (useful for testing)
    pub fn clear(&self) {
        self.threads.write().unwrap().clear();
        self.messages.write().unwrap().clear();
    }

    /// Get the number of stored threads
    pub fn thread_count(&self) -> usize {
        self.threads.read().unwrap().len()
    }

    /// Get the total number of stored messages
    pub fn total_message_count(&self) -> usize {
        self.messages.read().unwrap().values().map(|v| v.len()).sum()
    }

    fn matches_filter(&self, message: &StoredMessage, filter: &MessageFilter) -> bool {
        // Check role filter
        if let Some(ref role_filter) = filter.role {
            if message.message.role.to_string().to_lowercase() != role_filter.to_lowercase() {
                return false;
            }
        }

        // Check date range filter
        if let Some(ref date_range) = filter.date_range {
            if message.stored_at < date_range.start || message.stored_at > date_range.end {
                return false;
            }
        }

        // Check keywords filter
        if let Some(ref keywords) = filter.keywords {
            let content = message.message.content.to_lowercase();
            if !keywords.iter().any(|keyword| content.contains(&keyword.to_lowercase())) {
                return false;
            }
        }

        // TODO: Check metadata filter when Message supports metadata

        true
    }
}

#[async_trait]
impl MemoryThreadStorage for InMemoryThreadStorage {
    async fn create_thread(&self, thread: &MemoryThread) -> Result<MemoryThread> {
        let mut threads = self.threads.write().unwrap();
        
        if threads.contains_key(&thread.id) {
            return Err(LumosError::InvalidOperation(format!(
                "Thread with ID {} already exists",
                thread.id
            )));
        }

        threads.insert(thread.id.clone(), thread.clone());
        
        // Initialize empty message list for this thread
        self.messages.write().unwrap().insert(thread.id.clone(), Vec::new());
        
        Ok(thread.clone())
    }

    async fn get_thread(&self, thread_id: &str) -> Result<Option<MemoryThread>> {
        let threads = self.threads.read().unwrap();
        Ok(threads.get(thread_id).cloned())
    }

    async fn update_thread(&self, thread: &MemoryThread) -> Result<MemoryThread> {
        let mut threads = self.threads.write().unwrap();
        
        if !threads.contains_key(&thread.id) {
            return Err(LumosError::NotFound(format!(
                "Thread with ID {} not found",
                thread.id
            )));
        }

        threads.insert(thread.id.clone(), thread.clone());
        Ok(thread.clone())
    }

    async fn delete_thread(&self, thread_id: &str) -> Result<()> {
        let mut threads = self.threads.write().unwrap();
        let mut messages = self.messages.write().unwrap();
        
        if threads.remove(thread_id).is_none() {
            return Err(LumosError::NotFound(format!(
                "Thread with ID {} not found",
                thread_id
            )));
        }

        messages.remove(thread_id);
        Ok(())
    }

    async fn list_threads_by_resource(&self, resource_id: &str) -> Result<Vec<MemoryThread>> {
        let threads = self.threads.read().unwrap();
        Ok(threads
            .values()
            .filter(|thread| thread.is_owned_by(resource_id))
            .cloned()
            .collect())
    }

    async fn list_threads_by_agent(&self, agent_id: &str) -> Result<Vec<MemoryThread>> {
        let threads = self.threads.read().unwrap();
        Ok(threads
            .values()
            .filter(|thread| thread.belongs_to_agent(agent_id))
            .cloned()
            .collect())
    }

    async fn add_message(&self, thread_id: &str, message: &Message) -> Result<()> {
        // Verify thread exists
        {
            let threads = self.threads.read().unwrap();
            if !threads.contains_key(thread_id) {
                return Err(LumosError::NotFound(format!(
                    "Thread with ID {} not found",
                    thread_id
                )));
            }
        }

        let stored_message = StoredMessage {
            message: message.clone(),
            stored_at: Utc::now(),
            message_id: uuid::Uuid::new_v4().to_string(),
        };

        let mut messages = self.messages.write().unwrap();
        let thread_messages = messages.entry(thread_id.to_string()).or_insert_with(Vec::new);
        thread_messages.push(stored_message);

        Ok(())
    }

    async fn get_messages(&self, thread_id: &str, params: &GetMessagesParams) -> Result<Vec<Message>> {
        let messages = self.messages.read().unwrap();
        
        let thread_messages = match messages.get(thread_id) {
            Some(msgs) => msgs,
            None => return Ok(Vec::new()),
        };

        let mut filtered_messages: Vec<_> = thread_messages
            .iter()
            .filter(|msg| {
                if let Some(ref filter) = params.filter {
                    self.matches_filter(msg, filter)
                } else {
                    true
                }
            })
            .collect();

        // Sort by stored_at (chronological order)
        filtered_messages.sort_by(|a, b| a.stored_at.cmp(&b.stored_at));

        // Apply reverse order if requested
        if params.reverse_order {
            filtered_messages.reverse();
        }

        // Apply limit
        if let Some(limit) = params.limit {
            filtered_messages.truncate(limit);
        }

        // TODO: Handle cursor-based pagination

        Ok(filtered_messages
            .into_iter()
            .map(|stored_msg| stored_msg.message.clone())
            .collect())
    }

    async fn delete_messages(&self, thread_id: &str, message_ids: &[String]) -> Result<MessageOperationResult> {
        let mut messages = self.messages.write().unwrap();
        
        let thread_messages = match messages.get_mut(thread_id) {
            Some(msgs) => msgs,
            None => {
                return Ok(MessageOperationResult {
                    affected_count: 0,
                    success: false,
                    error_message: Some(format!("Thread {} not found", thread_id)),
                });
            }
        };

        let initial_count = thread_messages.len();
        thread_messages.retain(|msg| !message_ids.contains(&msg.message_id));
        let affected_count = initial_count - thread_messages.len();

        Ok(MessageOperationResult {
            affected_count,
            success: true,
            error_message: None,
        })
    }

    async fn search_messages(&self, query: &str, filter: Option<&MessageFilter>) -> Result<Vec<Message>> {
        let messages = self.messages.read().unwrap();
        let query_lower = query.to_lowercase();

        let mut matching_messages = Vec::new();

        for thread_messages in messages.values() {
            for stored_msg in thread_messages {
                // Check if message content contains the query
                if stored_msg.message.content.to_lowercase().contains(&query_lower) {
                    // Apply additional filter if provided
                    if let Some(filter) = filter {
                        if self.matches_filter(stored_msg, filter) {
                            matching_messages.push(stored_msg.message.clone());
                        }
                    } else {
                        matching_messages.push(stored_msg.message.clone());
                    }
                }
            }
        }

        Ok(matching_messages)
    }

    async fn get_thread_stats(&self, thread_id: &str) -> Result<ThreadStats> {
        // Verify thread exists
        let thread = {
            let threads = self.threads.read().unwrap();
            threads.get(thread_id).cloned()
        };

        let thread = thread.ok_or_else(|| LumosError::NotFound(format!(
            "Thread with ID {} not found",
            thread_id
        )))?;

        let messages = self.messages.read().unwrap();
        let thread_messages = messages.get(thread_id).unwrap_or(&Vec::new());

        let message_count = thread_messages.len();
        let user_message_count = thread_messages
            .iter()
            .filter(|msg| msg.message.role.to_string().to_lowercase() == "user")
            .count();
        let assistant_message_count = thread_messages
            .iter()
            .filter(|msg| msg.message.role.to_string().to_lowercase() == "assistant")
            .count();

        let last_message_at = thread_messages
            .iter()
            .map(|msg| msg.stored_at)
            .max();

        // Rough estimation of size in bytes
        let size_bytes = thread_messages
            .iter()
            .map(|msg| msg.message.content.len() + 100) // +100 for metadata overhead
            .sum();

        Ok(ThreadStats {
            message_count,
            user_message_count,
            assistant_message_count,
            created_at: thread.created_at,
            last_message_at,
            size_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{Role, user_message, assistant_message};

    #[tokio::test]
    async fn test_thread_creation_and_retrieval() {
        let storage = InMemoryThreadStorage::new();
        
        let thread = MemoryThread::new(super::super::thread::CreateThreadParams {
            id: Some("test-thread".to_string()),
            title: "Test Thread".to_string(),
            agent_id: Some("agent-1".to_string()),
            resource_id: Some("user-123".to_string()),
            metadata: None,
        });

        // Create thread
        let created = storage.create_thread(&thread).await.unwrap();
        assert_eq!(created.id, "test-thread");

        // Retrieve thread
        let retrieved = storage.get_thread("test-thread").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test-thread");

        // Thread not found
        let not_found = storage.get_thread("nonexistent").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_message_operations() {
        let storage = InMemoryThreadStorage::new();
        
        let thread = MemoryThread::new(super::super::thread::CreateThreadParams {
            id: Some("test-thread".to_string()),
            title: "Test Thread".to_string()),
            agent_id: None,
            resource_id: None,
            metadata: None,
        });

        storage.create_thread(&thread).await.unwrap();

        // Add messages
        let user_msg = user_message("Hello, how are you?");
        let assistant_msg = assistant_message("I'm doing well, thank you!");

        storage.add_message("test-thread", &user_msg).await.unwrap();
        storage.add_message("test-thread", &assistant_msg).await.unwrap();

        // Get messages
        let params = GetMessagesParams::default();
        let messages = storage.get_messages("test-thread", &params).await.unwrap();

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "Hello, how are you?");
        assert_eq!(messages[1].content, "I'm doing well, thank you!");
    }

    #[tokio::test]
    async fn test_message_filtering() {
        let storage = InMemoryThreadStorage::new();
        
        let thread = MemoryThread::new(super::super::thread::CreateThreadParams {
            id: Some("test-thread".to_string()),
            title: "Test Thread".to_string(),
            agent_id: None,
            resource_id: None,
            metadata: None,
        });

        storage.create_thread(&thread).await.unwrap();

        // Add various messages
        storage.add_message("test-thread", &user_message("Hello")).await.unwrap();
        storage.add_message("test-thread", &assistant_message("Hi there")).await.unwrap();
        storage.add_message("test-thread", &user_message("How are you?")).await.unwrap();

        // Filter by role
        let params = GetMessagesParams {
            filter: Some(MessageFilter {
                role: Some("user".to_string()),
                date_range: None,
                keywords: None,
                metadata: None,
            }),
            ..Default::default()
        };

        let user_messages = storage.get_messages("test-thread", &params).await.unwrap();
        assert_eq!(user_messages.len(), 2);
        assert!(user_messages.iter().all(|msg| msg.role == Role::User));
    }

    #[tokio::test]
    async fn test_thread_stats() {
        let storage = InMemoryThreadStorage::new();
        
        let thread = MemoryThread::new(super::super::thread::CreateThreadParams {
            id: Some("test-thread".to_string()),
            title: "Test Thread".to_string(),
            agent_id: None,
            resource_id: None,
            metadata: None,
        });

        storage.create_thread(&thread).await.unwrap();

        // Add messages
        storage.add_message("test-thread", &user_message("Hello")).await.unwrap();
        storage.add_message("test-thread", &assistant_message("Hi")).await.unwrap();
        storage.add_message("test-thread", &user_message("Bye")).await.unwrap();

        let stats = storage.get_thread_stats("test-thread").await.unwrap();
        assert_eq!(stats.message_count, 3);
        assert_eq!(stats.user_message_count, 2);
        assert_eq!(stats.assistant_message_count, 1);
        assert!(stats.last_message_at.is_some());
    }

    #[tokio::test]
    async fn test_search_messages() {
        let storage = InMemoryThreadStorage::new();
        
        let thread = MemoryThread::new(super::super::thread::CreateThreadParams {
            id: Some("test-thread".to_string()),
            title: "Test Thread".to_string(),
            agent_id: None,
            resource_id: None,
            metadata: None,
        });

        storage.create_thread(&thread).await.unwrap();

        // Add messages
        storage.add_message("test-thread", &user_message("I love programming")).await.unwrap();
        storage.add_message("test-thread", &assistant_message("Programming is great!")).await.unwrap();
        storage.add_message("test-thread", &user_message("What about cooking?")).await.unwrap();

        // Search for messages containing "programming"
        let results = storage.search_messages("programming", None).await.unwrap();
        assert_eq!(results.len(), 2);
    }
}
