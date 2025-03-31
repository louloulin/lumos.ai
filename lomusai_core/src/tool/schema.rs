use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use crate::error::Result;

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

/// Schema format for tool schemas
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SchemaFormat {
    /// Simple parameter list format
    #[serde(rename = "parameters")]
    Parameters,
    /// Full JSON Schema format
    #[serde(rename = "jsonschema")]
    JsonSchema,
    /// OpenAPI schema format
    #[serde(rename = "openapi")]
    OpenAPI,
}

impl Default for SchemaFormat {
    fn default() -> Self {
        Self::Parameters
    }
}

/// Schema for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// Parameters for the tool
    pub parameters: Vec<ParameterSchema>,
    
    /// JSON Schema representation (alternative to parameters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<Value>,
    
    /// The format of the schema
    #[serde(default)]
    pub format: SchemaFormat,
    
    /// JSON Schema for the output (for validation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
}

impl ToolSchema {
    /// Create a new tool schema with parameters
    pub fn new(parameters: Vec<ParameterSchema>) -> Self {
        Self {
            parameters,
            json_schema: None,
            format: SchemaFormat::Parameters,
            output_schema: None,
        }
    }
    
    /// Create a new tool schema with JSON Schema
    pub fn with_json_schema(json_schema: Value) -> Self {
        Self {
            parameters: Vec::new(),
            json_schema: Some(json_schema),
            format: SchemaFormat::JsonSchema,
            output_schema: None,
        }
    }
    
    /// Add output schema for validation
    pub fn with_output_schema(mut self, output_schema: Value) -> Self {
        self.output_schema = Some(output_schema);
        self
    }
    
    /// Validate parameters against the schema
    pub fn validate_params(&self, params: &Value) -> Result<()> {
        // If we have a JSON Schema, use it for validation
        if let Some(schema) = &self.json_schema {
            validate_with_json_schema(schema, params)?;
            return Ok(());
        }
        
        // Otherwise, do basic validation with our parameter schema
        validate_with_parameter_schema(&self.parameters, params)?;
        Ok(())
    }
    
    /// Validate the output against the output schema
    pub fn validate_output(&self, output: &Value) -> Result<()> {
        if let Some(output_schema) = &self.output_schema {
            validate_with_json_schema(output_schema, output)?;
        }
        Ok(())
    }
}

/// Execution options for a tool
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolExecutionOptions {
    /// Additional context for the tool execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, Value>>,
    
    /// Whether parameter validation should be performed
    #[serde(default)]
    pub validate_params: bool,
    
    /// Whether output validation should be performed
    #[serde(default)]
    pub validate_output: bool,
}

impl ToolExecutionOptions {
    /// Create new default options
    pub fn new() -> Self {
        Default::default()
    }
    
    /// Create options with validation enabled
    pub fn with_validation() -> Self {
        Self {
            validate_params: true,
            validate_output: true,
            ..Default::default()
        }
    }
    
    /// Add a context value
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        let context = self.context.get_or_insert_with(HashMap::new);
        context.insert(key.into(), value.into());
        self
    }
}

// Helper function to validate params against our parameter schema
fn validate_with_parameter_schema(parameters: &[ParameterSchema], params: &Value) -> Result<()> {
    // For now, just a simple implementation
    // Could be enhanced with more detailed validation
    
    if !params.is_object() {
        return Err(crate::error::Error::InvalidParams("Parameters must be an object".to_string()));
    }
    
    let params_obj = params.as_object().unwrap();
    
    // Check required parameters
    for param in parameters {
        if param.required && !params_obj.contains_key(&param.name) {
            return Err(crate::error::Error::InvalidParams(
                format!("Required parameter '{}' is missing", param.name)
            ));
        }
    }
    
    Ok(())
}

// Helper function to validate with JSON Schema
fn validate_with_json_schema(schema: &Value, instance: &Value) -> Result<()> {
    // Basic validation for required fields
    if let Some(schema_obj) = schema.as_object() {
        // Check for required properties
        if let Some(required) = schema_obj.get("required") {
            if let Some(required_fields) = required.as_array() {
                if let Some(instance_obj) = instance.as_object() {
                    for field in required_fields {
                        if let Some(field_str) = field.as_str() {
                            if !instance_obj.contains_key(field_str) {
                                return Err(crate::error::Error::ValidationError(
                                    format!("Required field '{}' is missing in output", field_str)
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    
    // For now, we just do basic required field validation
    // A complete implementation would use a full JSON Schema validator library
    Ok(())
} 