//! Agent types and configurations

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::llm::{LlmOptions, Message, Role};
use crate::memory::MemoryConfig;
use crate::tool::Tool;

/// Voice configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    /// Voice provider to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Voice ID or name to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<String>,
    /// Voice settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<Value>,
}

/// Telemetry settings for monitoring agent behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySettings {
    /// Whether telemetry is enabled
    #[serde(default)]
    pub is_enabled: bool,
    /// Whether to record inputs
    #[serde(default = "default_true")]
    pub record_inputs: bool,
    /// Whether to record outputs
    #[serde(default = "default_true")]
    pub record_outputs: bool,
    /// Function ID for telemetry grouping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_id: Option<String>,
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

fn default_true() -> bool {
    true
}

/// Types of agent tool choices
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoice {
    /// Let the model decide
    Auto,
    /// Don't use tools
    None,
    /// Require the model to use a tool
    Required,
    /// Use a specific tool
    Tool {
        /// Name of the tool to use
        tool_name: String,
    },
}

impl Default for ToolChoice {
    fn default() -> Self {
        Self::Auto
    }
}

/// Tool data structure for agent tools
#[derive(Debug, Clone)]
pub struct ToolData {
    /// Tool implementation
    pub tool: Box<dyn Tool>,
    /// Whether the tool is enabled
    pub enabled: bool,
}

/// Options for generating responses with an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGenerateOptions {
    /// Optional instructions to override the agent's default instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    
    /// Additional context messages to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<Message>>,
    
    /// Memory configuration options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_options: Option<MemoryConfig>,
    
    /// Thread ID for conversation tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    
    /// Resource ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    
    /// Unique ID for this generation run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    
    /// Maximum number of steps allowed for generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_steps: Option<u32>,
    
    /// Controls how tools are selected during generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    
    /// LLM options
    #[serde(flatten)]
    pub llm_options: LlmOptions,
}

impl Default for AgentGenerateOptions {
    fn default() -> Self {
        Self {
            instructions: None,
            context: None,
            memory_options: None,
            thread_id: None,
            resource_id: None,
            run_id: Some(Uuid::new_v4().to_string()),
            max_steps: Some(5),
            tool_choice: Some(ToolChoice::Auto),
            llm_options: LlmOptions::default(),
        }
    }
}

/// Options for streaming responses with an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStreamOptions {
    /// Optional instructions to override the agent's default instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    
    /// Additional context messages to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<Message>>,
    
    /// Memory configuration options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_options: Option<MemoryConfig>,
    
    /// Thread ID for conversation tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    
    /// Resource ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    
    /// Unique ID for this generation run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    
    /// Maximum number of steps allowed for generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_steps: Option<u32>,
    
    /// Controls how tools are selected during generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    
    /// LLM options
    #[serde(flatten)]
    pub llm_options: LlmOptions,
}

impl Default for AgentStreamOptions {
    fn default() -> Self {
        Self {
            instructions: None,
            context: None,
            memory_options: None,
            thread_id: None,
            resource_id: None,
            run_id: Some(Uuid::new_v4().to_string()),
            max_steps: Some(5),
            tool_choice: Some(ToolChoice::Auto),
            llm_options: LlmOptions::default(),
        }
    }
}

/// Agent step result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    /// Step ID
    pub id: String,
    /// Type of step
    pub step_type: StepType,
    /// Input messages for this step
    pub input: Vec<Message>,
    /// Output messages from this step
    pub output: Option<Message>,
    /// Tool calls from this step
    pub tool_calls: Vec<ToolCall>,
    /// Tool results from this step
    pub tool_results: Vec<ToolResult>,
    /// Step metadata
    pub metadata: HashMap<String, Value>,
}

/// Type of agent step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepType {
    /// Initial step 
    Initial,
    /// Tool execution step
    Tool,
    /// Final response step
    Final,
}

/// Tool call structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    /// Name of the tool
    pub name: String,
    /// Parameters for the tool call
    pub arguments: HashMap<String, Value>,
}

/// Tool result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// ID of the tool call this result belongs to
    pub call_id: String,
    /// Name of the tool
    pub name: String,
    /// Result of the tool execution
    pub result: Value,
    /// Status of the execution
    pub status: ToolResultStatus,
}

/// Status of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolResultStatus {
    /// Tool executed successfully
    Success,
    /// Tool execution failed
    Error,
}

/// Agent generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGenerateResult {
    /// The final response message
    pub response: String,
    /// All steps taken by the agent
    pub steps: Vec<AgentStep>,
    /// Total number of tokens used
    pub usage: TokenUsage,
    /// Agent metadata
    pub metadata: HashMap<String, Value>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of prompt tokens
    pub prompt_tokens: usize,
    /// Number of completion tokens
    pub completion_tokens: usize,
    /// Total number of tokens
    pub total_tokens: usize,
}

/// Helper function to create a system message
pub fn system_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::System,
        content: content.into(),
        metadata: None,
        name: None,
    }
}

/// Helper function to create a user message
pub fn user_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::User,
        content: content.into(),
        metadata: None,
        name: None,
    }
}

/// Helper function to create an assistant message
pub fn assistant_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::Assistant,
        content: content.into(),
        metadata: None,
        name: None,
    }
}

/// Helper function to create a tool message
pub fn tool_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::Tool,
        content: content.into(),
        metadata: None,
        name: None,
    }
} 