use serde::{Deserialize, Serialize};

use crate::memory::MemoryConfig;
use crate::memory::WorkingMemoryConfig;
use crate::llm::{LlmOptions, Message};
use crate::agent::types::{VoiceConfig, TelemetrySettings};

/// Configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Name of the agent
    pub name: String,
    /// Instructions for the agent
    pub instructions: String,
    /// Memory configuration for the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_config: Option<MemoryConfig>,
    /// Model configuration for the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    /// Voice configuration for the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_config: Option<VoiceConfig>,
    /// Telemetry settings for the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<TelemetrySettings>,
    /// Working memory configuration for the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_memory: Option<WorkingMemoryConfig>,
    /// Enable function calling (if provider supports it)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_function_calling: Option<bool>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "Agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            memory_config: None,
            model_id: None,
            voice_config: None,
            telemetry: None,
            working_memory: None,
            enable_function_calling: Some(true), // Default to enabled
        }
    }
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
    /// Temperature for generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Abort signal to cancel ongoing operations
    #[serde(skip)]
    pub abort_signal: Option<tokio::sync::watch::Receiver<bool>>,
    /// Structured output schema (either JSON schema or serialized Zod schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
    /// Experimental structured output alongside text generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_output: Option<serde_json::Value>,
    /// Telemetry settings for this run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<TelemetrySettings>,
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
            run_id: None,
            max_steps: Some(5),
            temperature: None,
            abort_signal: None,
            output_schema: None,
            experimental_output: None,
            telemetry: None,
            llm_options: LlmOptions::default(),
        }
    }
} 