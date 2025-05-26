use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use serde::{Serialize, Deserialize};

use crate::error::Result;
use crate::base::Base;
use super::schema::{ToolSchema, ToolExecutionOptions};
use super::context::ToolExecutionContext;

/// Trait representing a tool that can be used by an agent
#[async_trait]
pub trait Tool: Base + Send + Sync + Debug {
    /// Get the unique identifier of the tool
    fn id(&self) -> &str;
    
    /// Get the description of the tool
    fn description(&self) -> &str;
    
    /// Get the tool schema with parameter definitions
    fn schema(&self) -> ToolSchema;
    
    /// Get the output schema for validation (optional)
    fn output_schema(&self) -> Option<Value> {
        None
    }
    
    /// Get the category of the tool (optional)
    fn category(&self) -> Option<String> {
        None
    }
    
    /// Get examples of using the tool (optional)
    fn examples(&self) -> Option<Vec<String>> {
        None
    }
    
    /// Execute the tool with the given parameters
    async fn execute(
        &self, 
        params: Value, 
        context: ToolExecutionContext, 
        options: &ToolExecutionOptions
    ) -> Result<Value>;
    
    /// Clone the tool (needed since trait objects can't use derive Clone)
    fn clone_box(&self) -> Box<dyn Tool>;
}

// Add ability to clone Box<dyn Tool>
impl Clone for Box<dyn Tool> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Generic tool implementation
#[derive(Clone)]
pub struct GenericTool<F>
where
    F: Fn(Value, ToolExecutionContext) -> Result<Value> + Send + Sync + Clone + 'static,
{
    /// Base component
    base: crate::base::BaseComponent,
    /// Tool ID
    id: String,
    /// Tool description
    description: String,
    /// Tool schema
    schema: ToolSchema,
    /// Tool execution function
    execute_fn: F,
    /// Output schema
    output_schema: Option<Value>,
}

impl<F> GenericTool<F>
where
    F: Fn(Value, ToolExecutionContext) -> Result<Value> + Send + Sync + Clone + 'static,
{
    /// Create a new generic tool
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        schema: ToolSchema,
        execute_fn: F
    ) -> Self {
        let id_str = id.into();
        Self {
            base: crate::base::BaseComponent::new_with_name(
                id_str.clone(), 
                crate::logger::Component::Tool
            ),
            id: id_str,
            description: description.into(),
            schema,
            execute_fn,
            output_schema: None,
        }
    }
    
    /// Set the output schema
    pub fn with_output_schema(mut self, output_schema: Value) -> Self {
        self.output_schema = Some(output_schema);
        self
    }
}

// Debug implementation for GenericTool that skips execute_fn
impl<F> Debug for GenericTool<F>
where
    F: Fn(Value, ToolExecutionContext) -> Result<Value> + Send + Sync + Clone + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericTool")
            .field("id", &self.id)
            .field("description", &self.description)
            .field("schema", &self.schema)
            .field("output_schema", &self.output_schema)
            .finish_non_exhaustive()
    }
}

impl<F> Base for GenericTool<F>
where
    F: Fn(Value, ToolExecutionContext) -> Result<Value> + Send + Sync + Clone + 'static,
{
    fn name(&self) -> Option<&str> {
        self.base.name()
    }
    
    fn component(&self) -> crate::logger::Component {
        self.base.component()
    }
    
    fn logger(&self) -> std::sync::Arc<dyn crate::logger::Logger> {
        self.base.logger()
    }
    
    fn set_logger(&mut self, logger: std::sync::Arc<dyn crate::logger::Logger>) {
        self.base.set_logger(logger);
    }
    
    fn telemetry(&self) -> Option<std::sync::Arc<dyn crate::telemetry::TelemetrySink>> {
        self.base.telemetry()
    }
    
    fn set_telemetry(&mut self, telemetry: std::sync::Arc<dyn crate::telemetry::TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
}

#[async_trait]
impl<F> Tool for GenericTool<F>
where
    F: Fn(Value, ToolExecutionContext) -> Result<Value> + Send + Sync + Clone + 'static,
{
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
            "Executing tool [id={}] [thread_id={:?}]", 
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
        
        // Execute the tool
        let result = (self.execute_fn)(params, context)?;
        
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
        Box::new(self.clone())
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