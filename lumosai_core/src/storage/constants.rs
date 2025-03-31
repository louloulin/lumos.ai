//! Constants for the storage module

/// Table name for workflow snapshots
pub const TABLE_WORKFLOW_SNAPSHOT: &str = "lumosai_workflow_snapshot";

/// Table name for evaluation results
pub const TABLE_EVALS: &str = "lumosai_evals";

/// Table name for messages
pub const TABLE_MESSAGES: &str = "lumosai_messages";

/// Table name for threads
pub const TABLE_THREADS: &str = "lumosai_threads";

/// Table name for trace data
pub const TABLE_TRACES: &str = "lumosai_traces";

/// Enum of table names used in the storage system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TableName {
    /// Workflow snapshot table
    WorkflowSnapshot,
    /// Evaluations table
    Evals,
    /// Messages table
    Messages,
    /// Threads table
    Threads,
    /// Traces table
    Traces,
}

impl TableName {
    /// Returns the string representation of the table name
    pub fn as_str(&self) -> &'static str {
        match self {
            TableName::WorkflowSnapshot => TABLE_WORKFLOW_SNAPSHOT,
            TableName::Evals => TABLE_EVALS,
            TableName::Messages => TABLE_MESSAGES,
            TableName::Threads => TABLE_THREADS,
            TableName::Traces => TABLE_TRACES,
        }
    }
} 