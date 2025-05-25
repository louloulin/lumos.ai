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

/// Trait for tools that can be automatically converted to OpenAI function definitions
/// 
/// This trait is typically implemented automatically using the `FunctionSchema` derive macro.
/// It provides the bridge between Rust type definitions and OpenAI function calling schema.
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_derive::FunctionSchema;
/// use serde::{Serialize, Deserialize};
/// 
/// #[derive(Serialize, Deserialize, FunctionSchema)]
/// #[function(name = "calculate", description = "Performs calculations")]
/// pub struct CalculatorParams {
///     pub expression: String,
///     pub precision: Option<u32>,
/// }
/// 
/// // The derive macro automatically implements FunctionSchema
/// let definition = CalculatorParams::function_definition();
/// assert_eq!(definition.name, "calculate");
/// ```
pub trait FunctionSchema {
    /// Generate the OpenAI function definition for this tool
    /// 
    /// Returns a `FunctionDefinition` that can be sent to OpenAI's function calling API.
    /// The definition includes the function name, description, and parameter schema.
    fn function_definition() -> crate::llm::function_calling::FunctionDefinition;
    
    /// Validate that the provided arguments match this function's schema
    /// 
    /// This is an optional validation step that can be used to ensure
    /// arguments received from OpenAI function calling are valid.
    fn validate_arguments(arguments: &Value) -> Result<()> {
        // Default implementation: basic JSON validation
        if !arguments.is_object() {
            return Err(crate::Error::InvalidInput(
                "Function arguments must be a JSON object".to_string()
            ));
        }
        Ok(())
    }
    
    /// Get the function name (convenience method)
    fn function_name() -> String {
        Self::function_definition().name
    }
    
    /// Get the function description (convenience method)
    fn function_description() -> Option<String> {
        Self::function_definition().description
    }
}

/// Enhanced function calling utilities
pub mod utils {
    use super::*;
    use crate::llm::function_calling::FunctionDefinition;
    use std::collections::HashMap;
    
    /// Convert a collection of tools to OpenAI function definitions
    pub fn tools_to_function_definitions(tools: &HashMap<String, Box<dyn Tool>>) -> Vec<FunctionDefinition> {
        let mut functions = Vec::new();
        
        for tool in tools.values() {
            let schema = tool.schema();
            
            // Build parameters schema for OpenAI format
            let mut properties = serde_json::Map::new();
            let mut required = Vec::new();
            
            for param in &schema.parameters {
                let mut param_schema = serde_json::Map::new();
                param_schema.insert("type".to_string(), serde_json::Value::String(param.r#type.clone()));
                param_schema.insert("description".to_string(), serde_json::Value::String(param.description.clone()));
                
                if let Some(default) = &param.default {
                    param_schema.insert("default".to_string(), default.clone());
                }
                
                properties.insert(param.name.clone(), serde_json::Value::Object(param_schema));
                
                if param.required {
                    required.push(param.name.clone());
                }
            }
            
            let parameters = serde_json::json!({
                "type": "object",
                "properties": properties,
                "required": required
            });
            
            functions.push(FunctionDefinition::new(
                tool.id().to_string(),
                Some(tool.description().to_string()),
                parameters,
            ));
        }
        
        functions
    }
    
    /// Validate function call arguments against a schema
    pub fn validate_function_arguments(
        function_name: &str,
        arguments: &Value,
        functions: &[FunctionDefinition],
    ) -> Result<()> {
        let function_def = functions
            .iter()
            .find(|f| f.name == function_name)
            .ok_or_else(|| crate::Error::InvalidInput(
                format!("Unknown function: {}", function_name)
            ))?;
        
        // Basic validation - could be enhanced with full JSON schema validation
        if !arguments.is_object() {
            return Err(crate::Error::InvalidInput(
                "Function arguments must be a JSON object".to_string()
            ));
        }
        
        // Check required fields
        if let Some(required) = function_def.parameters.get("required") {
            if let Some(required_array) = required.as_array() {
                let args_obj = arguments.as_object().unwrap();
                
                for required_field in required_array {
                    if let Some(field_name) = required_field.as_str() {
                        if !args_obj.contains_key(field_name) {
                            return Err(crate::Error::InvalidInput(
                                format!("Missing required field: {}", field_name)
                            ));
                        }
                    }
                }
            }
        }
        
        Ok(())
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