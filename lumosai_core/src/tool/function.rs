use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use crate::error::Result;
use crate::base::{Base, BaseComponent};
use crate::logger::Component;
use super::tool::Tool;
use super::schema::{ToolSchema, ToolExecutionOptions};
use super::context::ToolExecutionContext;

/// A simple tool that executes a function
pub struct FunctionTool {
    /// Base component
    base: BaseComponent,
    /// The unique identifier of the tool
    id: String,
    /// The description of the tool
    description: String,
    /// The schema of the tool
    schema: ToolSchema,
    /// The function to execute
    function: Arc<dyn Fn(Value) -> Result<Value> + Send + Sync>,
    /// Output schema for validation
    output_schema: Option<Value>,
}

// Implement Debug for FunctionTool
impl Debug for FunctionTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionTool")
            .field("id", &self.id)
            .field("description", &self.description)
            .field("schema", &self.schema)
            .field("output_schema", &self.output_schema)
            .finish_non_exhaustive() // Skip function field which can't be debugged
    }
}

impl FunctionTool {
    /// Create a new function tool
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        schema: ToolSchema,
        function: impl Fn(Value) -> Result<Value> + Send + Sync + 'static,
    ) -> Self {
        let id_str = id.into();
        Self {
            base: BaseComponent::new_with_name(id_str.clone(), Component::Tool),
            id: id_str,
            description: description.into(),
            schema,
            function: Arc::new(function),
            output_schema: None,
        }
    }
    
    /// Set the output schema
    pub fn with_output_schema(mut self, output_schema: Value) -> Self {
        self.output_schema = Some(output_schema);
        self
    }
}

impl Base for FunctionTool {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }
    
    fn component(&self) -> Component {
        self.base.component()
    }
    
    fn logger(&self) -> Arc<dyn crate::logger::Logger> {
        self.base.logger()
    }
    
    fn set_logger(&mut self, logger: Arc<dyn crate::logger::Logger>) {
        self.base.set_logger(logger);
    }
    
    fn telemetry(&self) -> Option<Arc<dyn crate::telemetry::TelemetrySink>> {
        self.base.telemetry()
    }
    
    fn set_telemetry(&mut self, telemetry: Arc<dyn crate::telemetry::TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
}

#[async_trait]
impl Tool for FunctionTool {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn schema(&self) -> ToolSchema {
        self.schema.clone()
    }
    
    fn output_schema(&self) -> Option<Value> {
        self.output_schema.clone()
    }
    
    async fn execute(
        &self, 
        params: Value, 
        context: ToolExecutionContext, 
        options: &ToolExecutionOptions
    ) -> Result<Value> {
        // Log the tool execution
        self.logger().debug(&format!(
            "Executing function tool [id={}] [thread_id={:?}]", 
            self.id, context.thread_id
        ), None);
        
        // Check if abort is requested
        if context.is_abort_requested() {
            return Err(crate::error::Error::Tool("Tool execution aborted".to_string()));
        }
        
        // Validate parameters if needed
        if options.validate_params {
            self.schema().validate_params(&params)?;
        }
        
        // Execute the function
        let result = (self.function)(params)?;
        
        // Validate output if needed
        if options.validate_output {
            if let Some(output_schema) = self.output_schema() {
                self.schema().with_output_schema(output_schema).validate_output(&result)?;
            }
        }
        
        // Record telemetry
        let mut metadata = HashMap::new();
        metadata.insert("tool_id".to_string(), Value::String(self.id.clone()));
        metadata.insert("success".to_string(), Value::Bool(true));
        self.record_event("tool_executed", metadata);
        
        Ok(result)
    }
    
    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(Self {
            base: self.base.clone(),
            id: self.id.clone(),
            description: self.description.clone(),
            schema: self.schema.clone(),
            function: self.function.clone(),
            output_schema: self.output_schema.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::schema::ParameterSchema;
    
    #[tokio::test]
    async fn test_function_tool() {
        let schema = ToolSchema::new(vec![
            ParameterSchema {
                name: "message".to_string(),
                description: "The message to echo".to_string(),
                r#type: "string".to_string(),
                required: true,
                properties: None,
                default: None,
            },
        ]);
        
        let echo_tool = FunctionTool::new(
            "echo",
            "Echoes the input message",
            schema,
            |params| {
                let message = params.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No message");
                Ok(Value::String(format!("Echo: {}", message)))
            },
        );
        
        let params = serde_json::json!({
            "message": "Hello, world!"
        });
        
        let context = ToolExecutionContext::new();
        let options = ToolExecutionOptions::default();
        
        let result = echo_tool.execute(params, context, &options).await.unwrap();
        
        assert_eq!(result, Value::String("Echo: Hello, world!".to_string()));
    }
} 