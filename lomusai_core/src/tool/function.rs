use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use crate::error::Result;
use super::tool::Tool;
use super::schema::{ToolSchema, ToolExecutionOptions};

/// A simple tool that executes a function
pub struct FunctionTool {
    /// The name of the tool
    name: String,
    /// The description of the tool
    description: String,
    /// The schema of the tool
    schema: ToolSchema,
    /// The function to execute
    function: Arc<dyn Fn(HashMap<String, Value>) -> Result<Value> + Send + Sync>,
}

// Implement Debug for FunctionTool
impl Debug for FunctionTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionTool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("schema", &self.schema)
            .finish_non_exhaustive() // Skip function field which can't be debugged
    }
}

impl FunctionTool {
    /// Create a new function tool
    pub fn new(
        name: String,
        description: String,
        schema: ToolSchema,
        function: impl Fn(HashMap<String, Value>) -> Result<Value> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name,
            description,
            schema,
            function: Arc::new(function),
        }
    }
}

#[async_trait]
impl Tool for FunctionTool {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn schema(&self) -> ToolSchema {
        self.schema.clone()
    }
    
    async fn execute(&self, params: HashMap<String, Value>, _options: &ToolExecutionOptions) -> Result<Value> {
        (self.function)(params)
    }
    
    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(Self {
            name: self.name.clone(),
            description: self.description.clone(),
            schema: self.schema.clone(),
            function: self.function.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::schema::ParameterSchema;
    
    #[tokio::test]
    async fn test_function_tool() {
        let schema = ToolSchema {
            parameters: vec![
                ParameterSchema {
                    name: "message".to_string(),
                    description: "The message to echo".to_string(),
                    r#type: "string".to_string(),
                    required: true,
                    properties: None,
                    default: None,
                },
            ],
        };
        
        let echo_tool = FunctionTool::new(
            "echo".to_string(),
            "Echoes the input message".to_string(),
            schema,
            |params| {
                let message = params.get("message").and_then(|v| v.as_str()).unwrap_or("No message");
                Ok(Value::String(format!("Echo: {}", message)))
            },
        );
        
        let mut params = HashMap::new();
        params.insert("message".to_string(), Value::String("Hello, world!".to_string()));
        
        let result = echo_tool.execute(params, &ToolExecutionOptions::default()).await.unwrap();
        
        assert_eq!(result, Value::String("Echo: Hello, world!".to_string()));
    }
} 