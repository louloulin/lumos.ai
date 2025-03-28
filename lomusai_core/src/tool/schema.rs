use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Schema for a tool parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    /// The name of the parameter
    pub name: String,
    /// The description of the parameter
    pub description: String,
    /// The type of the parameter (string, number, boolean, object, array)
    pub r#type: String,
    /// Whether the parameter is required
    #[serde(default)]
    pub required: bool,
    /// Additional properties for the parameter (for complex types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, ParameterSchema>>,
    /// Default value for the parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}

/// Schema for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// Parameters for the tool
    pub parameters: Vec<ParameterSchema>,
}

/// Execution options for a tool
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolExecutionOptions {
    /// Additional context for the tool execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, Value>>,
} 