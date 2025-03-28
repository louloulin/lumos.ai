use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::Result;
use super::schema::{ToolSchema, ToolExecutionOptions};

/// Trait representing a tool that can be used by an agent
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the name of the tool
    fn name(&self) -> &str;
    
    /// Get the description of the tool
    fn description(&self) -> &str;
    
    /// Get the schema of the tool
    fn schema(&self) -> &ToolSchema;
    
    /// Execute the tool with the given parameters
    async fn execute(&self, params: HashMap<String, Value>, options: &ToolExecutionOptions) -> Result<Value>;
} 