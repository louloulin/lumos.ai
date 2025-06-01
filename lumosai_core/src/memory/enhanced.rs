//! Enhanced memory management system
//! 
//! Provides Mastra-like memory management features including semantic retrieval, working memory and memory processors

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::error::Result;
use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::llm::{Message, LlmProvider};
use crate::memory::{Memory, MemoryConfig};
use crate::vector::VectorStorage;
use crate::logger::{Component, Logger};

/// Memory entry type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryEntryType {
    /// User message
    UserMessage,
    /// Assistant reply
    AssistantMessage,
    /// System message
    SystemMessage,
    /// Tool call
    ToolCall,
    /// Tool result
    ToolResult,
    /// Fact information
    Fact,
    /// Context information
    Context,
    /// Custom type
    Custom(String),
}

/// Memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Entry ID
    pub id: String,
    /// Entry type
    pub entry_type: MemoryEntryType,
    /// Content
    pub content: String,
    /// Metadata
    pub metadata: HashMap<String, Value>,
    /// Creation timestamp
    pub created_at: u64,
    /// Update timestamp
    pub updated_at: u64,
    /// Importance score (0.0-1.0)
    pub importance: f32,
    /// Access count
    pub access_count: u32,
    /// Last access time
    pub last_accessed: u64,
    /// Thread ID
    pub thread_id: Option<String>,
    /// Resource ID
    pub resource_id: Option<String>,
}

/// Memory query options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQueryOptions {
    /// Query text
    pub query: String,
    /// Maximum return count
    pub limit: Option<usize>,
    /// Similarity threshold
    pub similarity_threshold: Option<f32>,
    /// Filter conditions
    pub filters: Option<HashMap<String, Value>>,
    /// Thread ID filter
    pub thread_id: Option<String>,
    /// Resource ID filter
    pub resource_id: Option<String>,
    /// Entry type filter
    pub entry_types: Option<Vec<MemoryEntryType>>,
    /// Time range filter (start_time, end_time)
    pub time_range: Option<(u64, u64)>,
    /// Importance threshold
    pub importance_threshold: Option<f32>,
}

/// Memory processor trait
#[async_trait]
pub trait MemoryProcessor: Send + Sync {
    /// Process memory entries
    async fn process(&self, entries: &mut Vec<MemoryEntry>) -> Result<()>;
    
    /// Get processor name
    fn name(&self) -> &str;
    
    /// Get processor description
    fn description(&self) -> &str;
}

/// Importance scoring processor
pub struct ImportanceProcessor {
    llm: Arc<dyn LlmProvider>,
}

impl ImportanceProcessor {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl MemoryProcessor for ImportanceProcessor {
    async fn process(&self, entries: &mut Vec<MemoryEntry>) -> Result<()> {
        for entry in entries.iter_mut() {
            // Use LLM to evaluate importance
            let prompt = format!(
                "Please evaluate the importance of the following content, return a value between 0.0 and 1.0:\n\n{}",
                entry.content
            );
            
            let options = crate::llm::LlmOptions::default();
            match self.llm.generate(&prompt, &options).await {
                Ok(response) => {
                    if let Ok(importance) = response.trim().parse::<f32>() {
                        entry.importance = importance.clamp(0.0, 1.0);
                    }
                }
                Err(_) => {
                    // If LLM evaluation fails, use heuristic method
                    entry.importance = self.calculate_heuristic_importance(entry);
                }
            }
        }
        Ok(())
    }
    
    fn name(&self) -> &str {
        "ImportanceProcessor"
    }
    
    fn description(&self) -> &str {
        "Use LLM to evaluate the importance of memory entries"
    }
}

impl ImportanceProcessor {
    fn calculate_heuristic_importance(&self, entry: &MemoryEntry) -> f32 {
        let mut importance = 0.5; // Base importance
        
        // Adjust based on content length
        if entry.content.len() > 100 {
            importance += 0.1;
        }
        
        // Adjust based on access count
        importance += (entry.access_count as f32 * 0.01).min(0.3);
        
        // Adjust based on entry type
        match entry.entry_type {
            MemoryEntryType::Fact => importance += 0.2,
            MemoryEntryType::ToolCall | MemoryEntryType::ToolResult => importance += 0.1,
            _ => {}
        }
        
        importance.clamp(0.0, 1.0)
    }
}

/// Enhanced memory manager
pub struct EnhancedMemory {
    /// Base component
    base: BaseComponent,
    /// Vector storage
    vector_storage: Arc<dyn VectorStorage>,
    /// LLM provider
    llm: Arc<dyn LlmProvider>,
    /// Memory processors
    processors: Vec<Arc<dyn MemoryProcessor>>,
    /// Configuration
    config: MemoryConfig,
}

impl EnhancedMemory {
    /// Create new enhanced memory manager
    pub fn new(
        vector_storage: Arc<dyn VectorStorage>,
        llm: Arc<dyn LlmProvider>,
        config: MemoryConfig,
    ) -> Self {
        let component_config = ComponentConfig {
            name: Some("EnhancedMemory".to_string()),
            component: Component::Memory,
            log_level: None,
        };

        let mut memory = Self {
            base: BaseComponent::new(component_config),
            vector_storage,
            llm: llm.clone(),
            processors: Vec::new(),
            config,
        };

        // Add default processor
        memory.add_processor(Arc::new(ImportanceProcessor::new(llm)));
        
        memory
    }

    /// Add memory processor
    pub fn add_processor(&mut self, processor: Arc<dyn MemoryProcessor>) {
        self.processors.push(processor);
    }

    /// Store memory entry
    pub async fn store_entry(&self, mut entry: MemoryEntry) -> Result<()> {
        // Apply processors
        let mut entries = vec![entry.clone()];
        for processor in &self.processors {
            processor.process(&mut entries).await?;
        }
        entry = entries.into_iter().next().unwrap();

        // Generate embedding vector
        let embedding = self.llm.get_embedding(&entry.content).await?;
        
        // Store to vector database
        let mut metadata = entry.metadata.clone();
        metadata.insert("id".to_string(), Value::String(entry.id.clone()));
        metadata.insert("entry_type".to_string(), Value::String(format!("{:?}", entry.entry_type)));
        metadata.insert("created_at".to_string(), Value::Number(entry.created_at.into()));
        metadata.insert("importance".to_string(), Value::Number(serde_json::Number::from_f64(entry.importance as f64).unwrap_or_else(|| serde_json::Number::from(0))));
        
        if let Some(thread_id) = &entry.thread_id {
            metadata.insert("thread_id".to_string(), Value::String(thread_id.clone()));
        }
        
        if let Some(resource_id) = &entry.resource_id {
            metadata.insert("resource_id".to_string(), Value::String(resource_id.clone()));
        }

        let entry_id = entry.id.clone();
        self.vector_storage.upsert(
            "default",
            vec![embedding],
            Some(vec![entry.id]),
            Some(vec![metadata])
        ).await?;

        self.base.logger().debug(&format!("Stored memory entry: {}", entry_id), None);
        Ok(())
    }

    /// Query memory
    pub async fn query(&self, options: &MemoryQueryOptions) -> Result<Vec<MemoryEntry>> {
        // Generate query vector
        let query_embedding = self.llm.get_embedding(&options.query).await?;
        
        // Build filter conditions
        let mut filters = options.filters.clone().unwrap_or_default();
        
        if let Some(thread_id) = &options.thread_id {
            filters.insert("thread_id".to_string(), Value::String(thread_id.clone()));
        }
        
        if let Some(resource_id) = &options.resource_id {
            filters.insert("resource_id".to_string(), Value::String(resource_id.clone()));
        }

        // Execute vector search
        let limit = options.limit.unwrap_or(10);
        let results = self.vector_storage.query(
            "default",
            query_embedding,
            limit,
            None, // filters not supported in this simple implementation
            false, // include_vectors
        ).await?;

        // Convert results to memory entries
        let mut entries = Vec::new();
        for result in results {
            if let Some(threshold) = options.similarity_threshold {
                if result.score < threshold {
                    continue;
                }
            }

            // Reconstruct memory entry from metadata
            if let Some(metadata) = &result.metadata {
                if let Some(entry) = self.reconstruct_entry_from_metadata(&result.id, metadata).await? {
                    entries.push(entry);
                }
            }
        }

        // Apply additional filters
        if let Some(entry_types) = &options.entry_types {
            entries.retain(|entry| entry_types.contains(&entry.entry_type));
        }

        if let Some((start_time, end_time)) = options.time_range {
            entries.retain(|entry| entry.created_at >= start_time && entry.created_at <= end_time);
        }

        if let Some(importance_threshold) = options.importance_threshold {
            entries.retain(|entry| entry.importance >= importance_threshold);
        }

        Ok(entries)
    }

    /// Reconstruct memory entry from metadata
    async fn reconstruct_entry_from_metadata(
        &self,
        id: &str,
        metadata: &HashMap<String, Value>,
    ) -> Result<Option<MemoryEntry>> {
        // This should get entry details from complete storage
        // For simplification, we reconstruct basic info from metadata
        let entry_type = metadata.get("entry_type")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "UserMessage" => MemoryEntryType::UserMessage,
                "AssistantMessage" => MemoryEntryType::AssistantMessage,
                "SystemMessage" => MemoryEntryType::SystemMessage,
                "ToolCall" => MemoryEntryType::ToolCall,
                "ToolResult" => MemoryEntryType::ToolResult,
                "Fact" => MemoryEntryType::Fact,
                "Context" => MemoryEntryType::Context,
                _ => MemoryEntryType::Custom(s.to_string()),
            })
            .unwrap_or(MemoryEntryType::Context);

        let created_at = metadata.get("created_at")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let importance = metadata.get("importance")
            .and_then(|v| v.as_f64())
            .map(|f| f as f32)
            .unwrap_or(0.5);

        let thread_id = metadata.get("thread_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let resource_id = metadata.get("resource_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(Some(MemoryEntry {
            id: id.to_string(),
            entry_type,
            content: "".to_string(), // Need to get from complete storage
            metadata: metadata.clone(),
            created_at,
            updated_at: created_at,
            importance,
            access_count: 0,
            last_accessed: created_at,
            thread_id,
            resource_id,
        }))
    }
}

#[async_trait]
impl Memory for EnhancedMemory {
    async fn store(&self, message: &Message) -> Result<()> {
        let entry_type = match message.role {
            crate::llm::Role::User => MemoryEntryType::UserMessage,
            crate::llm::Role::Assistant => MemoryEntryType::AssistantMessage,
            crate::llm::Role::System => MemoryEntryType::SystemMessage,
            crate::llm::Role::Tool => MemoryEntryType::ToolResult,
            crate::llm::Role::Function => MemoryEntryType::ToolCall,
            crate::llm::Role::Custom(_) => MemoryEntryType::Context,
        };

        let entry = MemoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            entry_type,
            content: message.content.clone(),
            metadata: message.metadata.clone().unwrap_or_default(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            importance: 0.5,
            access_count: 0,
            last_accessed: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            thread_id: None,
            resource_id: None,
        };

        self.store_entry(entry).await
    }

    async fn retrieve(&self, config: &MemoryConfig) -> Result<Vec<Message>> {
        // Basic implementation, can be extended as needed
        Ok(Vec::new())
    }
}
