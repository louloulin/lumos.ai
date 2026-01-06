//! Tool builder for simplified tool creation
//!
//! This module provides a fluent builder API for creating tools,
//! inspired by Mastra's design but optimized for Rust.
//!
//! # Overview
//!
//! The `ToolBuilder` provides a convenient way to create tools without
//! implementing the `Tool` trait manually. It uses the builder pattern
//! to configure tool properties step by step.
//!
//! # Examples
//!
//! ## Basic Tool
//!
//! ```rust
//! use lumosai_core::tool::ToolBuilder;
//! use serde_json::json;
//!
//! # fn example() -> lumosai_core::Result<()> {
//! let tool = ToolBuilder::new()
//!     .name("echo")
//!     .description("Echoes the input")
//!     .parameter("message", "string", "Message to echo", true)
//!     .handler(|params| {
//!         let message = params.get("message")
//!             .and_then(|v| v.as_str())
//!             .unwrap_or("");
//!         Ok(json!({"echo": message}))
//!     })
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Calculator Tool
//!
//! ```rust
//! use lumosai_core::tool::ToolBuilder;
//! use serde_json::json;
//!
//! # fn example() -> lumosai_core::Result<()> {
//! let tool = ToolBuilder::new()
//!     .name("calculator")
//!     .description("Performs basic math operations")
//!     .parameter("a", "number", "First number", true)
//!     .parameter("b", "number", "Second number", true)
//!     .parameter("operation", "string", "Operation (+, -, *, /)", true)
//!     .handler(|params| {
//!         let a = params["a"].as_f64().unwrap_or(0.0);
//!         let b = params["b"].as_f64().unwrap_or(0.0);
//!         let op = params["operation"].as_str().unwrap_or("+");
//!
//!         let result = match op {
//!             "+" => a + b,
//!             "-" => a - b,
//!             "*" => a * b,
//!             "/" => if b != 0.0 { a / b } else { 0.0 },
//!             _ => 0.0,
//!         };
//!
//!         Ok(json!({"result": result}))
//!     })
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Tool with Default Values
//!
//! ```rust
//! use lumosai_core::tool::ToolBuilder;
//! use serde_json::json;
//!
//! # fn example() -> lumosai_core::Result<()> {
//! let tool = ToolBuilder::new()
//!     .name("greet")
//!     .description("Greets a person")
//!     .parameter("name", "string", "Person's name", true)
//!     .parameter_with_default(
//!         "greeting",
//!         "string",
//!         "Greeting message",
//!         false,
//!         json!("Hello")
//!     )
//!     .handler(|params| {
//!         let name = params["name"].as_str().unwrap_or("stranger");
//!         let greeting = params.get("greeting")
//!             .and_then(|v| v.as_str())
//!             .unwrap_or("Hello");
//!         Ok(json!({"message": format!("{}, {}!", greeting, name)}))
//!     })
//!     .build()?;
//! # Ok(())
//! # }
//! ```

use serde_json::Value;
use std::collections::HashMap;

use super::{FunctionTool, ParameterSchema, Tool, ToolSchema};
use crate::{Error, Result};

/// Builder for creating tools with a fluent API
///
/// `ToolBuilder` provides a convenient way to create tools using the builder
/// pattern. It handles parameter schema construction, validation, and tool
/// creation.
///
/// # Required Fields
///
/// - **name**: Unique tool identifier
/// - **description**: Human-readable description
/// - **handler**: Function that executes the tool
///
/// # Optional Fields
///
/// - **parameters**: Tool parameters (can be empty)
///
/// # Examples
///
/// ## Minimal Tool
///
/// ```rust
/// use lumosai_core::tool::ToolBuilder;
/// use serde_json::json;
///
/// # fn example() -> lumosai_core::Result<()> {
/// let tool = ToolBuilder::new()
///     .name("ping")
///     .description("Returns pong")
///     .handler(|_params| Ok(json!({"response": "pong"})))
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// ## Tool with Multiple Parameters
///
/// ```rust
/// use lumosai_core::tool::ToolBuilder;
/// use serde_json::json;
///
/// # fn example() -> lumosai_core::Result<()> {
/// let tool = ToolBuilder::new()
///     .name("user_info")
///     .description("Creates user information")
///     .parameter("name", "string", "User's name", true)
///     .parameter("age", "number", "User's age", true)
///     .parameter("email", "string", "User's email", false)
///     .handler(|params| {
///         Ok(json!({
///             "name": params["name"],
///             "age": params["age"],
///             "email": params.get("email").unwrap_or(&json!(null))
///         }))
///     })
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// # See Also
///
/// - [`Tool`] - Tool trait
/// - [`FunctionTool`] - Function-based tool implementation
/// - [`ToolSchema`] - Tool parameter schema
pub struct ToolBuilder {
    /// Tool name (required)
    name: Option<String>,
    /// Tool description (required)
    description: Option<String>,
    /// Tool parameters
    parameters: Vec<ParameterSchema>,
    /// Tool handler function (required)
    handler: Option<Box<dyn Fn(Value) -> Result<Value> + Send + Sync>>,
}

impl ToolBuilder {
    /// Creates a new tool builder
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::ToolBuilder;
    ///
    /// let builder = ToolBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            parameters: Vec::new(),
            handler: None,
        }
    }

    /// Sets the tool name (required)
    ///
    /// The name is used as the tool's unique identifier and should be
    /// descriptive and follow naming conventions (lowercase, underscores).
    ///
    /// # Arguments
    ///
    /// * `name` - Tool name (e.g., "calculator", "web_search")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::ToolBuilder;
    ///
    /// let builder = ToolBuilder::new()
    ///     .name("calculator");
    /// ```
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the tool description (required)
    ///
    /// The description explains what the tool does and is used by the LLM
    /// to decide when to use the tool. Be clear and concise.
    ///
    /// # Arguments
    ///
    /// * `description` - Human-readable description
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::ToolBuilder;
    ///
    /// let builder = ToolBuilder::new()
    ///     .name("calculator")
    ///     .description("Performs basic arithmetic operations");
    /// ```
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a parameter to the tool
    ///
    /// Parameters define the inputs the tool accepts. Each parameter has a
    /// name, type, description, and required flag.
    ///
    /// # Arguments
    ///
    /// * `name` - Parameter name
    /// * `param_type` - Parameter type ("string", "number", "boolean", "object", "array")
    /// * `description` - Parameter description
    /// * `required` - Whether the parameter is required
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::ToolBuilder;
    ///
    /// let builder = ToolBuilder::new()
    ///     .name("calculator")
    ///     .description("Performs math operations")
    ///     .parameter("a", "number", "First number", true)
    ///     .parameter("b", "number", "Second number", true)
    ///     .parameter("operation", "string", "Operation type", true);
    /// ```
    pub fn parameter<S: Into<String>>(
        mut self,
        name: S,
        param_type: S,
        description: S,
        required: bool,
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

    /// Adds a parameter with a default value
    ///
    /// Use this for optional parameters that have a sensible default value.
    ///
    /// # Arguments
    ///
    /// * `name` - Parameter name
    /// * `param_type` - Parameter type
    /// * `description` - Parameter description
    /// * `required` - Whether the parameter is required
    /// * `default` - Default value as JSON
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::ToolBuilder;
    /// use serde_json::json;
    ///
    /// let builder = ToolBuilder::new()
    ///     .name("greet")
    ///     .description("Greets a person")
    ///     .parameter("name", "string", "Person's name", true)
    ///     .parameter_with_default(
    ///         "greeting",
    ///         "string",
    ///         "Greeting message",
    ///         false,
    ///         json!("Hello")
    ///     );
    /// ```
    pub fn parameter_with_default<S: Into<String>>(
        mut self,
        name: S,
        param_type: S,
        description: S,
        required: bool,
        default: Value,
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

    /// Adds a complex parameter with nested properties
    ///
    /// Use this for object-type parameters that have nested fields.
    ///
    /// # Arguments
    ///
    /// * `name` - Parameter name
    /// * `param_type` - Parameter type (typically "object")
    /// * `description` - Parameter description
    /// * `required` - Whether the parameter is required
    /// * `properties` - Nested parameter schemas
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::{ToolBuilder, ParameterSchema};
    /// use std::collections::HashMap;
    ///
    /// let mut properties = HashMap::new();
    /// properties.insert("street".to_string(), ParameterSchema {
    ///     name: "street".to_string(),
    ///     description: "Street address".to_string(),
    ///     r#type: "string".to_string(),
    ///     required: true,
    ///     properties: None,
    ///     default: None,
    /// });
    ///
    /// let builder = ToolBuilder::new()
    ///     .name("create_user")
    ///     .description("Creates a user")
    ///     .parameter_with_properties(
    ///         "address",
    ///         "object",
    ///         "User's address",
    ///         false,
    ///         properties
    ///     );
    /// ```
    pub fn parameter_with_properties<S: Into<String>>(
        mut self,
        name: S,
        param_type: S,
        description: S,
        required: bool,
        properties: HashMap<String, ParameterSchema>,
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

    /// Sets the tool handler function (required)
    ///
    /// The handler is the function that executes when the tool is called.
    /// It receives parameters as a JSON value and returns a result.
    ///
    /// # Arguments
    ///
    /// * `handler` - Function that takes `Value` and returns `Result<Value>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::ToolBuilder;
    /// use serde_json::json;
    ///
    /// # fn example() -> lumosai_core::Result<()> {
    /// let tool = ToolBuilder::new()
    ///     .name("echo")
    ///     .description("Echoes input")
    ///     .parameter("message", "string", "Message to echo", true)
    ///     .handler(|params| {
    ///         let message = params["message"].as_str().unwrap_or("");
    ///         Ok(json!({"echo": message}))
    ///     })
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(Value) -> Result<Value> + Send + Sync + 'static,
    {
        self.handler = Some(Box::new(handler));
        self
    }

    /// Builds the tool
    ///
    /// Validates that all required fields are set and creates a `FunctionTool`.
    ///
    /// # Returns
    ///
    /// A `FunctionTool` instance ready to be used by agents.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Tool name is not set
    /// - Tool description is not set
    /// - Tool handler is not set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::ToolBuilder;
    /// use serde_json::json;
    ///
    /// # fn example() -> lumosai_core::Result<()> {
    /// let tool = ToolBuilder::new()
    ///     .name("calculator")
    ///     .description("Performs math operations")
    ///     .parameter("a", "number", "First number", true)
    ///     .parameter("b", "number", "Second number", true)
    ///     .handler(|params| {
    ///         let a = params["a"].as_f64().unwrap_or(0.0);
    ///         let b = params["b"].as_f64().unwrap_or(0.0);
    ///         Ok(json!(a + b))
    ///     })
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> Result<FunctionTool> {
        // Validate required fields
        let name = self
            .name
            .ok_or_else(|| Error::Configuration("Tool name is required".to_string()))?;
        let description = self
            .description
            .ok_or_else(|| Error::Configuration("Tool description is required".to_string()))?;
        let handler = self
            .handler
            .ok_or_else(|| Error::Configuration("Tool handler is required".to_string()))?;

        // Create schema
        let schema = ToolSchema::new(self.parameters);

        // Create tool
        Ok(FunctionTool::new(name, description, schema, handler))
    }
}

impl Default for ToolBuilder {
    /// Creates a new tool builder (same as `new()`)
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a simple tool
///
/// This function provides a quick way to create tools without using the
/// builder pattern. It's useful for simple tools with basic parameters.
///
/// # Arguments
///
/// * `name` - Tool name
/// * `description` - Tool description
/// * `parameters` - Vector of (name, type, description, required) tuples
/// * `handler` - Function that executes the tool
///
/// # Returns
///
/// A `FunctionTool` instance.
///
/// # Errors
///
/// Returns an error if the tool cannot be created.
///
/// # Examples
///
/// ```rust
/// use lumosai_core::tool::create_tool;
/// use serde_json::json;
///
/// # fn example() -> lumosai_core::Result<()> {
/// let tool = create_tool(
///     "echo",
///     "Echoes a message",
///     vec![("message", "string", "Message to echo", true)],
///     |params| {
///         let message = params["message"].as_str().unwrap_or("");
///         Ok(json!({"echo": message}))
///     }
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// # See Also
///
/// - [`ToolBuilder`] - For more complex tool creation
pub fn create_tool<F>(
    name: &str,
    description: &str,
    parameters: Vec<(&str, &str, &str, bool)>,
    handler: F,
) -> Result<FunctionTool>
where
    F: Fn(Value) -> Result<Value> + Send + Sync + 'static,
{
    let mut builder = ToolBuilder::new().name(name).description(description);

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
                let input = params
                    .get("input")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
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
                let message = params
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No message");
                Ok(json!({"echo": message}))
            },
        )
        .expect("Failed to create tool");

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
