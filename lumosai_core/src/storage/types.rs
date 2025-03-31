//! Type definitions for the storage module

use std::collections::HashMap;
use std::fmt;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::error::Result;
use crate::workflow::WorkflowState;

/// Column type for schema definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    /// Text type
    Text,
    /// Timestamp type
    Timestamp,
    /// UUID type
    Uuid,
    /// JSON or JSONB type
    Json,
    /// Integer type
    Integer,
    /// BigInt type
    BigInt,
}

impl fmt::Display for ColumnType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColumnType::Text => write!(f, "TEXT"),
            ColumnType::Timestamp => write!(f, "TIMESTAMP"),
            ColumnType::Uuid => write!(f, "UUID"),
            ColumnType::Json => write!(f, "JSONB"),
            ColumnType::Integer => write!(f, "INTEGER"),
            ColumnType::BigInt => write!(f, "BIGINT"),
        }
    }
}

/// Column definition for schema creation
#[derive(Debug, Clone)]
pub struct ColumnDefinition {
    /// Column type
    pub column_type: ColumnType,
    /// Whether this column is a primary key
    pub primary_key: bool,
    /// Whether this column can be null
    pub nullable: bool,
    /// Reference to another table and column
    pub references: Option<(String, String)>,
}

/// Thread data structure for memory storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    /// Thread ID
    pub id: String,
    /// Associated resource ID
    pub resource_id: String,
    /// Thread title
    pub title: String,
    /// Additional metadata (JSON)
    pub metadata: Option<serde_json::Value>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Message data structure for memory storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message ID
    pub id: String,
    /// Thread ID this message belongs to
    pub thread_id: String,
    /// Message content
    pub content: String,
    /// Message role (system, user, assistant)
    pub role: String,
    /// Message type
    pub message_type: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Evaluation result row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRow {
    /// Input text
    pub input: String,
    /// Output text
    pub output: String,
    /// Evaluation result
    pub result: serde_json::Value,
    /// Agent name
    pub agent_name: String,
    /// Creation timestamp
    pub created_at: String,
    /// Metric name used for evaluation
    pub metric_name: String,
    /// Evaluation instructions
    pub instructions: String,
    /// Run ID
    pub run_id: String,
    /// Global run ID
    pub global_run_id: String,
    /// Test information (optional)
    pub test_info: Option<serde_json::Value>,
}

/// Workflow snapshot row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRow {
    /// Workflow name
    pub workflow_name: String,
    /// Run ID
    pub run_id: String,
    /// Workflow state snapshot
    pub snapshot: WorkflowState,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Arguments for retrieving messages
#[derive(Debug, Clone)]
pub struct GetMessagesArgs {
    /// Thread ID
    pub thread_id: String,
    /// Resource ID (optional)
    pub resource_id: Option<String>,
    /// Selection criteria
    pub select_by: Option<MessageSelection>,
    /// Thread configuration
    pub thread_config: Option<serde_json::Value>,
}

/// Message selection criteria
#[derive(Debug, Clone)]
pub struct MessageSelection {
    /// Vector search string
    pub vector_search_string: Option<String>,
    /// Get last N messages (or false to get all)
    pub last: Option<usize>,
    /// Include specific messages with context
    pub include: Option<Vec<MessageInclude>>,
}

/// Message include specification
#[derive(Debug, Clone)]
pub struct MessageInclude {
    /// Message ID to include
    pub id: String,
    /// Number of previous messages to include
    pub with_previous_messages: Option<usize>,
    /// Number of next messages to include
    pub with_next_messages: Option<usize>,
}

/// Storage trait defining the interface for all storage providers
#[async_trait]
pub trait Storage: Send + Sync {
    /// Get the name of this storage provider
    fn name(&self) -> &str;
    
    /// Initialize the storage
    async fn init(&self) -> Result<()>;
    
    /// Create a table with the given schema
    async fn create_table(&self, table_name: &str, schema: HashMap<String, ColumnDefinition>) -> Result<()>;
    
    /// Clear all data from a table
    async fn clear_table(&self, table_name: &str) -> Result<()>;
    
    /// Insert a record into a table
    async fn insert(&self, table_name: &str, record: serde_json::Value) -> Result<()>;
    
    /// Batch insert multiple records into a table
    async fn batch_insert(&self, table_name: &str, records: Vec<serde_json::Value>) -> Result<()>;
    
    /// Load a record from a table by keys
    async fn load(&self, table_name: &str, keys: HashMap<String, String>) -> Result<Option<serde_json::Value>>;
    
    /// Get a thread by ID
    async fn get_thread_by_id(&self, thread_id: &str) -> Result<Option<Thread>>;
    
    /// Get threads by resource ID
    async fn get_threads_by_resource_id(&self, resource_id: &str) -> Result<Vec<Thread>>;
    
    /// Save a thread
    async fn save_thread(&self, thread: Thread) -> Result<Thread>;
    
    /// Update a thread
    async fn update_thread(&self, id: &str, title: &str, metadata: serde_json::Value) -> Result<Thread>;
    
    /// Delete a thread
    async fn delete_thread(&self, thread_id: &str) -> Result<()>;
    
    /// Get messages
    async fn get_messages(&self, args: GetMessagesArgs) -> Result<Vec<Message>>;
    
    /// Save messages
    async fn save_messages(&self, messages: Vec<Message>) -> Result<Vec<Message>>;
    
    /// Get evaluation results by agent name
    async fn get_evals_by_agent_name(&self, agent_name: &str, eval_type: Option<&str>) -> Result<Vec<EvalRow>>;
    
    /// Get traces
    async fn get_traces(&self, 
        name: Option<&str>,
        scope: Option<&str>,
        page: usize,
        per_page: usize,
        attributes: Option<HashMap<String, String>>
    ) -> Result<Vec<serde_json::Value>>;
    
    /// Persist a workflow snapshot
    async fn persist_workflow_snapshot(&self, 
        workflow_name: &str, 
        run_id: &str, 
        snapshot: &WorkflowState
    ) -> Result<()>;
    
    /// Load a workflow snapshot
    async fn load_workflow_snapshot(&self, 
        workflow_name: &str,
        run_id: &str
    ) -> Result<Option<WorkflowState>>;
} 