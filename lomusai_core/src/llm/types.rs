use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Role enum representing the role of a message sender
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// System message role
    System,
    /// User message role
    User,
    /// Assistant message role
    Assistant,
    /// Function message role
    Function,
    /// Custom role (for future extensions)
    Custom(String),
}

impl Role {
    /// Create a new role
    pub fn new(role: impl Into<String>) -> Self {
        let role_str = role.into();
        match role_str.to_lowercase().as_str() {
            "system" => Role::System,
            "user" => Role::User,
            "assistant" => Role::Assistant,
            "function" => Role::Function,
            _ => Role::Custom(role_str),
        }
    }
    
    /// Convert Role to its string representation
    pub fn as_str(&self) -> &str {
        match self {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Function => "function",
            Role::Custom(s) => s.as_str(),
        }
    }
}

impl From<Role> for String {
    fn from(role: Role) -> Self {
        match role {
            Role::System => "system".to_string(),
            Role::User => "user".to_string(),
            Role::Assistant => "assistant".to_string(),
            Role::Function => "function".to_string(),
            Role::Custom(s) => s,
        }
    }
}

impl From<&str> for Role {
    fn from(s: &str) -> Self {
        Role::new(s)
    }
}

impl From<String> for Role {
    fn from(s: String) -> Self {
        Role::new(s)
    }
}

impl PartialEq<&str> for Role {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents a message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender (e.g., "user", "assistant", "system")
    pub role: Role,
    /// The content of the message
    pub content: String,
    /// Additional metadata for the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Optional name for the message, used in function calls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Message {
    /// Create a new message
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            metadata: None,
            name: None,
        }
    }
    
    /// Create a new message with a name
    pub fn with_name(role: Role, content: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            metadata: None,
            name: Some(name.into()),
        }
    }
    
    /// Add metadata to the message
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        let metadata = self.metadata.get_or_insert_with(HashMap::new);
        metadata.insert(key.into(), value.into());
        self
    }
}

/// Options for LLM text generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmOptions {
    /// Temperature parameter for controlling randomness (0.0-1.0)
    pub temperature: Option<f32>,
    /// Maximum number of tokens to generate
    pub max_tokens: Option<u32>,
    /// Whether to stream the output
    pub stream: bool,
    /// Stop sequences that end generation
    pub stop: Option<Vec<String>>,
    /// Model name
    pub model: Option<String>,
    /// Additional model-specific parameters
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Default for LlmOptions {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stream: false,
            stop: None,
            model: None,
            extra: serde_json::Map::new(),
        }
    }
}

impl LlmOptions {
    /// Create a new LLM options
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
    
    /// Set maximum token count
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    
    /// Set whether to stream the output
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
    
    /// Set stop sequences
    pub fn with_stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }
    
    /// Set model name
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
    
    /// Add extra options
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
} 