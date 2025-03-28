use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender (e.g., "user", "assistant", "system")
    pub role: String,
    /// The content of the message
    pub content: String,
    /// Additional metadata for the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Options for LLM text generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmOptions {
    /// Temperature parameter for controlling randomness (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Stop sequences that end generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// Message history for the conversation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<Message>>,
    /// Additional model-specific parameters
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub params: Option<HashMap<String, serde_json::Value>>,
}

impl Default for LlmOptions {
    fn default() -> Self {
        Self {
            temperature: None,
            max_tokens: None,
            stop: None,
            messages: None,
            params: None,
        }
    }
} 