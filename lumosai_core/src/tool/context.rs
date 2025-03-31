use serde::{Serialize, Deserialize};
use tokio::sync::watch;
use crate::llm::Message;

/// Context for tool execution
///
/// Provides additional context for a tool execution, such as thread ID, resource ID,
/// and other metadata that might be useful for the tool to know about.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolExecutionContext {
    /// The ID of the thread where the tool is being executed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    
    /// The ID of the resource associated with the tool execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    
    /// The ID of the run associated with the tool execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    
    /// The ID of the specific tool call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    
    /// The messages that led to this tool execution (for context)
    #[serde(skip)]
    pub messages: Option<Vec<Message>>,
    
    /// Signal that can be used to abort the tool execution
    #[serde(skip)]
    pub abort_signal: Option<watch::Receiver<bool>>,
}

impl ToolExecutionContext {
    /// Create a new empty tool execution context
    pub fn new() -> Self {
        Default::default()
    }
    
    /// Create a new tool execution context with thread ID
    pub fn with_thread_id(thread_id: impl Into<String>) -> Self {
        Self {
            thread_id: Some(thread_id.into()),
            ..Default::default()
        }
    }
    
    /// Add a resource ID to the context
    pub fn with_resource_id(mut self, resource_id: impl Into<String>) -> Self {
        self.resource_id = Some(resource_id.into());
        self
    }
    
    /// Add a run ID to the context
    pub fn with_run_id(mut self, run_id: impl Into<String>) -> Self {
        self.run_id = Some(run_id.into());
        self
    }
    
    /// Add a tool call ID to the context
    pub fn with_tool_call_id(mut self, tool_call_id: impl Into<String>) -> Self {
        self.tool_call_id = Some(tool_call_id.into());
        self
    }
    
    /// Add messages to the context
    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = Some(messages);
        self
    }
    
    /// Add an abort signal to the context
    pub fn with_abort_signal(mut self, abort_signal: watch::Receiver<bool>) -> Self {
        self.abort_signal = Some(abort_signal);
        self
    }
    
    /// Check if an abort has been requested
    pub fn is_abort_requested(&self) -> bool {
        self.abort_signal.as_ref()
            .map(|signal| *signal.borrow())
            .unwrap_or(false)
    }
} 