//! In-memory storage provider implementation

use std::collections::HashMap;
use std::sync::RwLock;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;

use crate::error::{Error, Result};
use crate::storage::types::*;
use crate::workflow::WorkflowState;

/// Extension trait for Option to simplify filter expressions
trait OptionExt<T> {
    fn is_none_or<F>(&self, f: F) -> bool
    where
        F: FnOnce(&T) -> bool;
}

impl<T> OptionExt<T> for Option<T> {
    fn is_none_or<F>(&self, f: F) -> bool
    where
        F: FnOnce(&T) -> bool,
    {
        match self {
            None => true,
            Some(v) => f(v),
        }
    }
}

/// In-memory storage implementation
pub struct MemoryStorage {
    /// Provider name
    name: String,
    /// Tables storing data in memory
    tables: RwLock<HashMap<String, Vec<Value>>>,
    /// Thread storage
    threads: RwLock<HashMap<String, Thread>>,
    /// Message storage
    messages: RwLock<HashMap<String, Message>>,
    /// Workflow snapshot storage
    workflow_snapshots: RwLock<HashMap<(String, String), WorkflowState>>,
    /// Eval results storage
    evals: RwLock<HashMap<String, Vec<EvalRow>>>,
    /// Traces storage
    traces: RwLock<Vec<Value>>,
}

impl MemoryStorage {
    /// Create a new in-memory storage provider
    pub fn new(name: String) -> Self {
        Self {
            name,
            tables: RwLock::new(HashMap::new()),
            threads: RwLock::new(HashMap::new()),
            messages: RwLock::new(HashMap::new()),
            workflow_snapshots: RwLock::new(HashMap::new()),
            evals: RwLock::new(HashMap::new()),
            traces: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    fn name(&self) -> &str {
        &self.name
    }

    async fn init(&self) -> Result<()> {
        Ok(()) // Nothing to initialize for in-memory storage
    }

    async fn create_table(&self, table_name: &str, _schema: HashMap<String, ColumnDefinition>) -> Result<()> {
        let mut tables = self.tables.write().map_err(|_| Error::Storage("Failed to acquire write lock".into()))?;
        if !tables.contains_key(table_name) {
            tables.insert(table_name.to_string(), Vec::new());
        }
        Ok(())
    }

    async fn clear_table(&self, table_name: &str) -> Result<()> {
        let mut tables = self.tables.write().map_err(|_| Error::Storage("Failed to acquire write lock".into()))?;
        if let Some(table) = tables.get_mut(table_name) {
            table.clear();
        }
        Ok(())
    }

    async fn insert(&self, table_name: &str, record: Value) -> Result<()> {
        let mut tables = self.tables.write().map_err(|_| Error::Storage("Failed to acquire write lock".into()))?;
        if let Some(table) = tables.get_mut(table_name) {
            table.push(record);
            Ok(())
        } else {
            Err(Error::Storage(format!("Table {} not found", table_name)))
        }
    }

    async fn batch_insert(&self, table_name: &str, records: Vec<Value>) -> Result<()> {
        let mut tables = self.tables.write().map_err(|_| Error::Storage("Failed to acquire write lock".into()))?;
        if let Some(table) = tables.get_mut(table_name) {
            table.extend(records);
            Ok(())
        } else {
            Err(Error::Storage(format!("Table {} not found", table_name)))
        }
    }

    async fn load(&self, table_name: &str, keys: HashMap<String, String>) -> Result<Option<Value>> {
        let tables = self.tables.read().map_err(|_| Error::Storage("Failed to acquire read lock".into()))?;
        if let Some(table) = tables.get(table_name) {
            Ok(table.iter().find(|record| {
                if let Value::Object(obj) = record {
                    keys.iter().all(|(key, value)| {
                        obj.get(key)
                            .and_then(|v| v.as_str())
                            .map(|v| v == value)
                            .unwrap_or(false)
                    })
                } else {
                    false
                }
            }).cloned())
        } else {
            Err(Error::Storage(format!("Table {} not found", table_name)))
        }
    }

    async fn get_thread_by_id(&self, thread_id: &str) -> Result<Option<Thread>> {
        let threads = self.threads.read().map_err(|_| Error::Storage("Failed to acquire read lock".into()))?;
        Ok(threads.get(thread_id).cloned())
    }

    async fn get_threads_by_resource_id(&self, resource_id: &str) -> Result<Vec<Thread>> {
        let threads = self.threads.read().map_err(|_| Error::Storage("Failed to acquire read lock".into()))?;
        Ok(threads.values()
            .filter(|thread| thread.resource_id == resource_id)
            .cloned()
            .collect())
    }

    async fn save_thread(&self, thread: Thread) -> Result<Thread> {
        let mut threads = self.threads.write().map_err(|_| Error::Storage("Failed to acquire write lock".into()))?;
        let thread_clone = thread.clone();
        threads.insert(thread.id.clone(), thread);
        Ok(thread_clone)
    }

    async fn update_thread(&self, id: &str, title: &str, metadata: Value) -> Result<Thread> {
        let mut threads = self.threads.write().map_err(|_| Error::Storage("Failed to acquire write lock".into()))?;
        
        if let Some(thread) = threads.get_mut(id) {
            thread.title = title.to_string();
            thread.metadata = Some(metadata);
            thread.updated_at = Utc::now();
            Ok(thread.clone())
        } else {
            Err(Error::Storage(format!("Thread {} not found", id)))
        }
    }

    async fn delete_thread(&self, thread_id: &str) -> Result<()> {
        let mut threads = self.threads.write().map_err(|_| Error::Storage("Failed to acquire write lock".into()))?;
        threads.remove(thread_id);
        Ok(())
    }

    async fn get_messages(&self, args: GetMessagesArgs) -> Result<Vec<Message>> {
        let messages = self.messages.read().map_err(|_| Error::Storage("Failed to acquire read lock".into()))?;
        
        let mut result: Vec<Message> = messages.values()
            .filter(|msg| msg.thread_id == args.thread_id)
            .cloned()
            .collect();

        // Apply selection criteria if provided
        if let Some(select) = args.select_by {
            if let Some(last) = select.last {
                result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                result.truncate(last);
            }
            
            // Handle includes if specified
            if let Some(includes) = select.include {
                let mut included_messages = Vec::new();
                for include in includes {
                    if let Some(msg) = messages.get(&include.id) {
                        included_messages.push(msg.clone());
                        
                        // Add previous messages if requested
                        if let Some(prev_count) = include.with_previous_messages {
                            let mut prev_msgs: Vec<_> = messages.values()
                                .filter(|m| m.thread_id == msg.thread_id && m.created_at < msg.created_at)
                                .cloned()
                                .collect();
                            prev_msgs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                            prev_msgs.truncate(prev_count);
                            included_messages.extend(prev_msgs);
                        }
                        
                        // Add next messages if requested
                        if let Some(next_count) = include.with_next_messages {
                            let mut next_msgs: Vec<_> = messages.values()
                                .filter(|m| m.thread_id == msg.thread_id && m.created_at > msg.created_at)
                                .cloned()
                                .collect();
                            next_msgs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
                            next_msgs.truncate(next_count);
                            included_messages.extend(next_msgs);
                        }
                    }
                }
                result = included_messages;
            }
        }

        result.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(result)
    }

    async fn save_messages(&self, messages: Vec<Message>) -> Result<Vec<Message>> {
        let mut storage = self.messages.write().map_err(|_| Error::Storage("Failed to acquire write lock".into()))?;
        for message in &messages {
            storage.insert(message.id.clone(), message.clone());
        }
        Ok(messages)
    }

    async fn get_evals_by_agent_name(&self, agent_name: &str, eval_type: Option<&str>) -> Result<Vec<EvalRow>> {
        let evals = self.evals.read().map_err(|_| Error::Storage("Failed to acquire read lock".into()))?;
        
        Ok(evals.get(agent_name)
            .map(|rows| {
                if let Some(etype) = eval_type {
                    rows.iter()
                        .filter(|row| row.test_info.as_ref().is_some_and(|info| 
                            info.get("type").and_then(|v| v.as_str()) == Some(etype)))
                        .cloned()
                        .collect()
                } else {
                    rows.clone()
                }
            })
            .unwrap_or_default())
    }

    async fn get_traces(&self, 
        name: Option<&str>,
        scope: Option<&str>,
        page: usize,
        per_page: usize,
        attributes: Option<HashMap<String, String>>
    ) -> Result<Vec<Value>> {
        let traces = self.traces.read().map_err(|_| Error::Storage("Failed to acquire read lock".into()))?;
        
        let filtered: Vec<_> = traces.iter()
            .filter(|trace| {
                if let Value::Object(obj) = trace {
                    let name_match = name.is_none_or(|n| 
                        obj.get("name").and_then(|v| v.as_str()) == Some(n));
                    let scope_match = scope.is_none_or(|s| 
                        obj.get("scope").and_then(|v| v.as_str()) == Some(s));
                    let attrs_match = attributes.as_ref().is_none_or(|attrs| 
                        attrs.iter().all(|(k, v)| 
                            obj.get("attributes")
                               .and_then(|a| a.as_object())
                               .and_then(|a| a.get(k))
                               .and_then(|a| a.as_str()) == Some(v)));
                    name_match && scope_match && attrs_match
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        let start = page.saturating_mul(per_page);
        let end = start.saturating_add(per_page).min(filtered.len());
        
        Ok(filtered[start..end].to_vec())
    }

    async fn persist_workflow_snapshot(&self, workflow_name: &str, run_id: &str, snapshot: &WorkflowState) -> Result<()> {
        let mut snapshots = self.workflow_snapshots.write()
            .map_err(|_| Error::Storage("Failed to acquire write lock".into()))?;
        snapshots.insert((workflow_name.to_string(), run_id.to_string()), snapshot.clone());
        Ok(())
    }

    async fn load_workflow_snapshot(&self, workflow_name: &str, run_id: &str) -> Result<Option<WorkflowState>> {
        let snapshots = self.workflow_snapshots.read()
            .map_err(|_| Error::Storage("Failed to acquire read lock".into()))?;
        Ok(snapshots.get(&(workflow_name.to_string(), run_id.to_string())).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    #[tokio::test]
    async fn test_thread_operations() {
        let storage = MemoryStorage::new("test".to_string());
        
        // Create a test thread
        let thread = Thread {
            id: "test-thread".to_string(),
            resource_id: "test-resource".to_string(),
            title: "Test Thread".to_string(),
            metadata: Some(json!({"key": "value"})),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Test save_thread
        let saved = storage.save_thread(thread.clone()).await.unwrap();
        assert_eq!(saved.id, thread.id);
        
        // Test get_thread_by_id
        let retrieved = storage.get_thread_by_id("test-thread").await.unwrap().unwrap();
        assert_eq!(retrieved.id, thread.id);
        
        // Test get_threads_by_resource_id
        let threads = storage.get_threads_by_resource_id("test-resource").await.unwrap();
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].id, thread.id);
        
        // Test update_thread
        let updated = storage.update_thread(
            "test-thread",
            "Updated Title",
            json!({"key": "new_value"})
        ).await.unwrap();
        assert_eq!(updated.title, "Updated Title");
        
        // Test delete_thread
        storage.delete_thread("test-thread").await.unwrap();
        let deleted = storage.get_thread_by_id("test-thread").await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_message_operations() {
        let storage = MemoryStorage::new("test".to_string());
        
        // Create test messages
        let messages = vec![
            Message {
                id: "msg1".to_string(),
                thread_id: "thread1".to_string(),
                content: "Message 1".to_string(),
                role: "user".to_string(),
                message_type: "text".to_string(),
                created_at: Utc::now(),
            },
            Message {
                id: "msg2".to_string(),
                thread_id: "thread1".to_string(),
                content: "Message 2".to_string(),
                role: "assistant".to_string(),
                message_type: "text".to_string(),
                created_at: Utc::now(),
            },
        ];
        
        // Test save_messages
        let saved = storage.save_messages(messages.clone()).await.unwrap();
        assert_eq!(saved.len(), 2);
        
        // Test get_messages
        let retrieved = storage.get_messages(GetMessagesArgs {
            thread_id: "thread1".to_string(),
            resource_id: None,
            select_by: None,
            thread_config: None,
        }).await.unwrap();
        
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].id, "msg1");
        assert_eq!(retrieved[1].id, "msg2");
        
        // Test get_messages with selection
        let selected = storage.get_messages(GetMessagesArgs {
            thread_id: "thread1".to_string(),
            resource_id: None,
            select_by: Some(MessageSelection {
                vector_search_string: None,
                last: Some(1),
                include: None,
            }),
            thread_config: None,
        }).await.unwrap();
        
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "msg2");
    }

    #[tokio::test]
    async fn test_workflow_snapshot_operations() {
        let storage = MemoryStorage::new("test".to_string());
        
        // Create a test workflow state
        let state = WorkflowState::default(); // Assuming WorkflowState has a Default impl
        
        // Test persist_workflow_snapshot
        storage.persist_workflow_snapshot("test-workflow", "run1", &state).await.unwrap();
        
        // Test load_workflow_snapshot
        let loaded = storage.load_workflow_snapshot("test-workflow", "run1").await.unwrap();
        assert!(loaded.is_some());
    }
} 