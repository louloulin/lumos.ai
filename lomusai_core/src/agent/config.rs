use serde::{Deserialize, Serialize};
use crate::memory::MemoryConfig;
use crate::llm::{LlmOptions, Message};

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
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "Agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            memory_config: None,
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
    /// Unique ID for this generation run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    /// Maximum number of steps allowed for generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_steps: Option<u32>,
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
            run_id: None,
            max_steps: None,
            llm_options: LlmOptions::default(),
        }
    }
} 