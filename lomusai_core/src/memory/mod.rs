//! Memory module for storing and retrieving context information

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::llm::Message;
use crate::Result;

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum number of messages to retrieve
    pub max_messages: usize,
    
    /// Additional configuration options
    #[serde(flatten)]
    pub options: HashMap<String, serde_json::Value>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_messages: 10,
            options: HashMap::new(),
        }
    }
}

/// Basic memory trait
#[async_trait]
pub trait Memory: Send + Sync {
    /// Add a key-value pair to memory
    async fn add(&mut self, key: &str, value: &str) -> Result<()>;
    
    /// Get a value by key
    async fn get(&self, key: &str) -> Result<Option<String>>;
    
    /// Store a message in memory
    async fn store_message(&self, message: &Message, config: &MemoryConfig) -> Result<()>;
    
    /// Retrieve messages from memory
    async fn retrieve(&self, config: &MemoryConfig) -> Result<Vec<Message>>;
} 