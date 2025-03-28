use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::Result;
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
    
    fn schema(&self) -> &ToolSchema {
        &self.schema
    }
    
    async fn execute(&self, params: HashMap<String, Value>, _options: &ToolExecutionOptions) -> Result<Value> {
        (self.function)(params)
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