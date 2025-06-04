//! Tool builder for simplified tool creation
//! 
//! This module provides a fluent builder API for creating tools,
//! inspired by Mastra's design but optimized for Rust.

use std::collections::HashMap;
use serde_json::Value;

use crate::{Result, Error};
use super::{Tool, FunctionTool, ToolSchema, ParameterSchema};

/// Builder for creating tools with a fluent API
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::tool::ToolBuilder;
/// 
/// let tool = ToolBuilder::new()
///     .name("calculator")
///     .description("Performs basic math operations")
///     .parameter("a", "number", "First number", true)
///     .parameter("b", "number", "Second number", true)
///     .parameter("operation", "string", "Operation (+, -, *, /)", true)
///     .handler(|params| {
///         let a = params.get("a")?.as_f64().ok_or("Invalid number a")?;
///         let b = params.get("b")?.as_f64().ok_or("Invalid number b")?;
///         let op = params.get("operation")?.as_str().ok_or("Invalid operation")?;
///         
///         let result = match op {
///             "+" => a + b,
///             "-" => a - b,
///             "*" => a * b,
///             "/" => a / b,
///             _ => return Err("Unknown operation".into()),
///         };
///         
///         Ok(serde_json::json!({"result": result}))
///     })
///     .build()
///     .expect("Failed to build tool");
/// ```
pub struct ToolBuilder {
    name: Option<String>,
    description: Option<String>,
    parameters: Vec<ParameterSchema>,
    handler: Option<Box<dyn Fn(Value) -> Result<Value> + Send + Sync>>,
}

impl ToolBuilder {
    /// Create a new tool builder
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            parameters: Vec::new(),
            handler: None,
        }
    }

    /// Set the tool name
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the tool description
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a parameter to the tool
    pub fn parameter<S: Into<String>>(
        mut self, 
        name: S, 
        param_type: S, 
        description: S, 
        required: bool
    ) -> Self {
        let param = ParameterSchema {
            name: name.into(),
            description: description.into(),
            r#type: param_type.into(),
            required,
            properties: None,
            default: None,
        };
        self.parameters.push(param);
        self
    }

    /// Add a parameter with default value
    pub fn parameter_with_default<S: Into<String>>(
        mut self, 
        name: S, 
        param_type: S, 
        description: S, 
        required: bool,
        default: Value
    ) -> Self {
        let param = ParameterSchema {
            name: name.into(),
            description: description.into(),
            r#type: param_type.into(),
            required,
            properties: None,
            default: Some(default),
        };
        self.parameters.push(param);
        self
    }

    /// Add a complex parameter with properties (for object types)
    pub fn parameter_with_properties<S: Into<String>>(
        mut self,
        name: S,
        param_type: S,
        description: S,
        required: bool,
        properties: HashMap<String, ParameterSchema>
    ) -> Self {
        let param = ParameterSchema {
            name: name.into(),
            description: description.into(),
            r#type: param_type.into(),
            required,
            properties: Some(properties),
            default: None,
        };
        self.parameters.push(param);
        self
    }

    /// Set the tool handler function
    pub fn handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(Value) -> Result<Value> + Send + Sync + 'static,
    {
        self.handler = Some(Box::new(handler));
        self
    }

    /// Build the tool
    pub fn build(self) -> Result<FunctionTool> {
        // Validate required fields
        let name = self.name.ok_or_else(|| Error::Configuration("Tool name is required".to_string()))?;
        let description = self.description.ok_or_else(|| Error::Configuration("Tool description is required".to_string()))?;
        let handler = self.handler.ok_or_else(|| Error::Configuration("Tool handler is required".to_string()))?;

        // Create schema
        let schema = ToolSchema::new(self.parameters);

        // Create tool
        Ok(FunctionTool::new(name, description, schema, handler))
    }
}

impl Default for ToolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a simple tool
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::tool::create_tool;
/// 
/// let tool = create_tool(
///     "echo",
///     "Echo a message",
///     vec![("message", "string", "Message to echo", true)],
///     |params| {
///         let message = params.get("message")?.as_str().ok_or("Invalid message")?;
///         Ok(serde_json::json!({"echo": message}))
///     }
/// ).expect("Failed to create tool");
/// ```
pub fn create_tool<F>(
    name: &str,
    description: &str,
    parameters: Vec<(&str, &str, &str, bool)>,
    handler: F,
) -> Result<FunctionTool>
where
    F: Fn(Value) -> Result<Value> + Send + Sync + 'static,
{
    let mut builder = ToolBuilder::new()
        .name(name)
        .description(description);

    for (param_name, param_type, param_desc, required) in parameters {
        builder = builder.parameter(param_name, param_type, param_desc, required);
    }

    builder.handler(handler).build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_builder_basic() {
        let tool = ToolBuilder::new()
            .name("test_tool")
            .description("A test tool")
            .parameter("input", "string", "Test input", true)
            .handler(|params| {
                let input = params.get("input").and_then(|v| v.as_str()).unwrap_or("default");
                Ok(json!({"output": input}))
            })
            .build()
            .expect("Failed to build tool");

        assert_eq!(tool.id(), "test_tool");
        assert_eq!(tool.description(), "A test tool");
    }

    #[test]
    fn test_tool_builder_with_default() {
        let tool = ToolBuilder::new()
            .name("test_tool")
            .description("A test tool")
            .parameter_with_default("count", "number", "Count value", false, json!(10))
            .handler(|params| {
                let count = params.get("count").and_then(|v| v.as_i64()).unwrap_or(10);
                Ok(json!({"count": count}))
            })
            .build()
            .expect("Failed to build tool");

        assert_eq!(tool.id(), "test_tool");
    }

    #[test]
    fn test_create_tool_convenience() {
        let tool = create_tool(
            "echo",
            "Echo a message",
            vec![("message", "string", "Message to echo", true)],
            |params| {
                let message = params.get("message").and_then(|v| v.as_str()).unwrap_or("No message");
                Ok(json!({"echo": message}))
            }
        ).expect("Failed to create tool");

        assert_eq!(tool.id(), "echo");
        assert_eq!(tool.description(), "Echo a message");
    }

    #[test]
    fn test_tool_builder_validation() {
        // Test missing name
        let result = ToolBuilder::new()
            .description("Test description")
            .handler(|_| Ok(json!({})))
            .build();
        assert!(result.is_err());

        // Test missing description
        let result = ToolBuilder::new()
            .name("test")
            .handler(|_| Ok(json!({})))
            .build();
        assert!(result.is_err());

        // Test missing handler
        let result = ToolBuilder::new()
            .name("test")
            .description("Test description")
            .build();
        assert!(result.is_err());
    }
}
