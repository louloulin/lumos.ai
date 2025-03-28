use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

use crate::error::Result;
use super::schema::{ToolSchema, ToolExecutionOptions};

/// Trait representing a tool that can be used by an agent
#[async_trait]
pub trait Tool: Send + Sync + Debug {
    /// Get the name of the tool
    fn name(&self) -> &str;
    
    /// Get the description of the tool
    fn description(&self) -> &str;
    
    /// Get the tool schema with parameter definitions
    fn schema(&self) -> ToolSchema;
    
    /// Execute the tool with the given parameters
    async fn execute(&self, params: HashMap<String, Value>, options: &ToolExecutionOptions) -> Result<Value>;
    
    /// Clone the tool (needed since trait objects can't use derive Clone)
    fn clone_box(&self) -> Box<dyn Tool>;
}

// Add ability to clone Box<dyn Tool>
impl Clone for Box<dyn Tool> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Parameter schema for tool parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    /// Parameter name
    pub name: String,
    /// Parameter description
    pub description: String,
    /// Parameter type (string, number, boolean, etc.)
    pub r#type: String,
    /// Whether the parameter is required
    pub required: bool,
    /// Nested properties for object type parameters
    pub properties: Option<Vec<ParameterSchema>>,
    /// Default value for the parameter
    pub default: Option<Value>,
} 