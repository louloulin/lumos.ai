use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;

use super::context::ToolExecutionContext;
use super::schema::{ToolExecutionOptions, ToolSchema};
use crate::base::Base;
use crate::error::Result;

/// Core trait for tools that extend agent capabilities
///
/// The `Tool` trait defines the interface for all tools in LumosAI. Tools are
/// reusable components that agents can invoke to perform specific actions like
/// web searches, file operations, calculations, API calls, etc.
///
/// # Overview
///
/// A tool consists of:
/// - **ID**: Unique identifier for the tool
/// - **Description**: Human-readable description of what the tool does
/// - **Schema**: Parameter definitions (types, descriptions, required/optional)
/// - **Execute**: Async function that performs the tool's action
///
/// # Tool Categories
///
/// Tools can be categorized into:
/// - **Web Tools**: HTTP requests, web scraping, API calls
/// - **File Tools**: File I/O, directory operations
/// - **Data Tools**: JSON/CSV processing, data transformation
/// - **Search Tools**: Web search, database queries
/// - **Communication Tools**: Email, messaging, notifications
/// - **System Tools**: Shell commands, system information
/// - **Custom Tools**: User-defined tools
///
/// # Creating Tools
///
/// There are three ways to create tools:
///
/// ## 1. Using the `#[tool]` Macro (Recommended)
///
/// ```rust
/// use lumosai_core::prelude::*;
/// use lumos_macro::tool;
/// use serde_json::{json, Value};
///
/// /// Calculator tool - performs basic math operations
/// ///
/// /// Parameters:
/// /// - operation: Operation type (add, subtract, multiply, divide)
/// /// - a: First number
/// /// - b: Second number
/// #[tool(name = "calculator", description = "Performs basic math operations")]
/// async fn calculator(operation: String, a: f64, b: f64) -> Result<Value> {
///     let result = match operation.as_str() {
///         "add" => a + b,
///         "subtract" => a - b,
///         "multiply" => a * b,
///         "divide" => a / b,
///         _ => return Err(Error::tool_execution("Invalid operation")),
///     };
///     Ok(json!(result))
/// }
/// ```
///
/// ## 2. Using ToolBuilder
///
/// ```rust
/// use lumosai_core::tool::{ToolBuilder, ToolSchema};
/// use serde_json::{json, Value};
///
/// # fn example() -> lumosai_core::Result<()> {
/// let tool = ToolBuilder::new()
///     .id("calculator")
///     .description("Performs basic math operations")
///     .parameter("operation", "string", "Operation type", true)
///     .parameter("a", "number", "First number", true)
///     .parameter("b", "number", "Second number", true)
///     .handler(|params, _context| {
///         // Implementation
///         Ok(json!({"result": 42}))
///     })
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// ## 3. Implementing the Trait Directly
///
/// ```rust
/// use lumosai_core::tool::{Tool, ToolSchema, ToolExecutionContext, ToolExecutionOptions};
/// use lumosai_core::base::{Base, BaseComponent};
/// use lumosai_core::compat::Component;
/// use lumosai_core::Result;
/// use serde_json::Value;
/// use async_trait::async_trait;
///
/// #[derive(Debug, Clone)]
/// struct MyTool {
///     base: BaseComponent,
/// }
///
/// #[async_trait]
/// impl Tool for MyTool {
///     fn id(&self) -> &str { "my_tool" }
///     fn description(&self) -> &str { "My custom tool" }
///     fn schema(&self) -> ToolSchema { ToolSchema::default() }
///
///     async fn execute(
///         &self,
///         params: Value,
///         context: ToolExecutionContext,
///         options: &ToolExecutionOptions,
///     ) -> Result<Value> {
///         // Implementation
///         Ok(Value::Null)
///     }
///
///     fn clone_box(&self) -> Box<dyn Tool> {
///         Box::new(self.clone())
///     }
/// }
///
/// impl Base for MyTool {
///     fn base(&self) -> &BaseComponent { &self.base }
///     fn base_mut(&mut self) -> &mut BaseComponent { &mut self.base }
/// }
/// ```
///
/// # Thread Safety
///
/// All tools must be `Send + Sync`, allowing them to be safely used across
/// threads and in async contexts.
///
/// # See Also
///
/// - [`ToolBuilder`](crate::tool::ToolBuilder) - Builder for creating tools
/// - [`ToolSchema`](crate::tool::ToolSchema) - Tool parameter schema
/// - [`ToolExecutionContext`](crate::tool::ToolExecutionContext) - Execution context
#[async_trait]
pub trait Tool: Base + Send + Sync + Debug {
    /// Returns the tool's unique identifier
    ///
    /// The ID is used to reference the tool in agent configurations and
    /// tool calls. It should be unique within an agent's tool set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::Tool;
    ///
    /// # fn example(tool: &dyn Tool) {
    /// let id = tool.id();
    /// println!("Tool ID: {}", id);
    /// # }
    /// ```
    fn id(&self) -> &str;

    /// Returns the tool's description
    ///
    /// The description explains what the tool does and is used by the LLM
    /// to decide when to use the tool.
    ///
    /// # Best Practices
    ///
    /// - Be clear and concise
    /// - Describe what the tool does, not how it works
    /// - Include key capabilities and limitations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::Tool;
    ///
    /// # fn example(tool: &dyn Tool) {
    /// let description = tool.description();
    /// println!("Tool description: {}", description);
    /// # }
    /// ```
    fn description(&self) -> &str;

    /// Returns the tool's parameter schema
    ///
    /// The schema defines the tool's parameters, including their types,
    /// descriptions, and whether they are required or optional.
    ///
    /// # Returns
    ///
    /// A `ToolSchema` containing parameter definitions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::Tool;
    ///
    /// # fn example(tool: &dyn Tool) {
    /// let schema = tool.schema();
    /// println!("Tool parameters: {:?}", schema.parameters);
    /// # }
    /// ```
    fn schema(&self) -> ToolSchema;

    /// Returns the tool's output schema for validation (optional)
    ///
    /// If provided, the output schema can be used to validate the tool's
    /// return value.
    ///
    /// # Returns
    ///
    /// - `Some(schema)` if the tool has an output schema
    /// - `None` if no output validation is needed (default)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::Tool;
    ///
    /// # fn example(tool: &dyn Tool) {
    /// if let Some(schema) = tool.output_schema() {
    ///     println!("Output schema: {:?}", schema);
    /// }
    /// # }
    /// ```
    fn output_schema(&self) -> Option<Value> {
        None
    }

    /// Returns the tool's category (optional)
    ///
    /// Categories help organize tools and can be used for filtering or
    /// grouping in UIs.
    ///
    /// # Common Categories
    ///
    /// - "web" - Web-related tools
    /// - "file" - File operation tools
    /// - "data" - Data processing tools
    /// - "search" - Search tools
    /// - "communication" - Communication tools
    /// - "system" - System tools
    ///
    /// # Returns
    ///
    /// - `Some(category)` if the tool has a category
    /// - `None` if no category is assigned (default)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::Tool;
    ///
    /// # fn example(tool: &dyn Tool) {
    /// if let Some(category) = tool.category() {
    ///     println!("Tool category: {}", category);
    /// }
    /// # }
    /// ```
    fn category(&self) -> Option<String> {
        None
    }

    /// Returns usage examples for the tool (optional)
    ///
    /// Examples help users and LLMs understand how to use the tool correctly.
    ///
    /// # Returns
    ///
    /// - `Some(examples)` if the tool has usage examples
    /// - `None` if no examples are provided (default)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::Tool;
    ///
    /// # fn example(tool: &dyn Tool) {
    /// if let Some(examples) = tool.examples() {
    ///     for (i, example) in examples.iter().enumerate() {
    ///         println!("Example {}: {}", i + 1, example);
    ///     }
    /// }
    /// # }
    /// ```
    fn examples(&self) -> Option<Vec<String>> {
        None
    }

    /// Executes the tool with the given parameters
    ///
    /// This is the core method that performs the tool's action. It receives
    /// parameters as JSON, executes the tool's logic, and returns the result
    /// as JSON.
    ///
    /// # Arguments
    ///
    /// * `params` - Tool parameters as a JSON value
    /// * `context` - Execution context (thread ID, user ID, etc.)
    /// * `options` - Execution options (timeout, retry, etc.)
    ///
    /// # Returns
    ///
    /// The tool's output as a JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Parameters are invalid or missing
    /// - The tool execution fails
    /// - The tool times out
    /// - External dependencies are unavailable
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
    /// use serde_json::json;
    ///
    /// # async fn example(tool: &dyn Tool) -> lumosai_core::Result<()> {
    /// let params = json!({
    ///     "operation": "add",
    ///     "a": 5,
    ///     "b": 3
    /// });
    ///
    /// let context = ToolExecutionContext::default();
    /// let options = ToolExecutionOptions::default();
    ///
    /// let result = tool.execute(params, context, &options).await?;
    /// println!("Result: {:?}", result);
    /// # Ok(())
    /// # }
    /// ```
    async fn execute(
        &self,
        params: Value,
        context: ToolExecutionContext,
        options: &ToolExecutionOptions,
    ) -> Result<Value>;

    /// Clones the tool into a boxed trait object
    ///
    /// This method is needed because trait objects cannot use `derive(Clone)`.
    /// It allows tools to be cloned when stored in `Box<dyn Tool>`.
    ///
    /// # Returns
    ///
    /// A new boxed instance of the tool.
    ///
    /// # Implementation
    ///
    /// For types that implement `Clone`, this is typically:
    ///
    /// ```rust
    /// # use lumosai_core::tool::Tool;
    /// # #[derive(Clone)]
    /// # struct MyTool;
    /// # impl MyTool {
    /// fn clone_box(&self) -> Box<dyn Tool> {
    ///     Box::new(self.clone())
    /// }
    /// # }
    /// ```
    fn clone_box(&self) -> Box<dyn Tool>;
}

// Add ability to clone Box<dyn Tool>
impl Clone for Box<dyn Tool> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Generic tool implementation using a closure
///
/// `GenericTool` provides a simple way to create tools using closures or
/// function pointers, without needing to implement the `Tool` trait manually.
///
/// # Type Parameters
///
/// * `F` - A function that takes `(Value, ToolExecutionContext)` and returns `Result<Value>`
///
/// # Examples
///
/// ## Creating a Simple Tool
///
/// ```rust
/// use lumosai_core::tool::{GenericTool, ToolSchema, ToolExecutionContext};
/// use serde_json::{json, Value};
/// use lumosai_core::Result;
///
/// let calculator = GenericTool::new(
///     "calculator",
///     "Performs basic math operations",
///     ToolSchema::default(),
///     |params: Value, _context: ToolExecutionContext| -> Result<Value> {
///         let a = params["a"].as_f64().unwrap_or(0.0);
///         let b = params["b"].as_f64().unwrap_or(0.0);
///         Ok(json!(a + b))
///     }
/// );
/// ```
///
/// ## With Captured Variables
///
/// ```rust
/// use lumosai_core::tool::{GenericTool, ToolSchema, ToolExecutionContext};
/// use serde_json::{json, Value};
/// use lumosai_core::Result;
///
/// let multiplier = 2.0;
/// let tool = GenericTool::new(
///     "multiply",
///     "Multiplies input by a constant",
///     ToolSchema::default(),
///     move |params: Value, _context: ToolExecutionContext| -> Result<Value> {
///         let x = params["x"].as_f64().unwrap_or(0.0);
///         Ok(json!(x * multiplier))
///     }
/// );
/// ```
///
/// # Thread Safety
///
/// The closure `F` must be `Send + Sync + Clone`, ensuring the tool can be
/// safely used across threads.
#[derive(Clone)]
pub struct GenericTool<F>
where
    F: Fn(Value, ToolExecutionContext) -> Result<Value> + Send + Sync + Clone + 'static,
{
    /// Base component for common functionality
    base: crate::base::BaseComponent,
    /// Unique tool identifier
    id: String,
    /// Human-readable tool description
    description: String,
    /// Parameter schema defining inputs
    schema: ToolSchema,
    /// Execution function that performs the tool's action
    execute_fn: F,
    /// Optional output schema for validation
    output_schema: Option<Value>,
}

impl<F> GenericTool<F>
where
    F: Fn(Value, ToolExecutionContext) -> Result<Value> + Send + Sync + Clone + 'static,
{
    /// Creates a new generic tool
    ///
    /// # Arguments
    ///
    /// * `id` - Unique tool identifier
    /// * `description` - Human-readable description
    /// * `schema` - Parameter schema
    /// * `execute_fn` - Function that executes the tool
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::tool::{GenericTool, ToolSchema, ToolExecutionContext};
    /// use serde_json::{json, Value};
    /// use lumosai_core::Result;
    ///
    /// let tool = GenericTool::new(
    ///     "echo",
    ///     "Echoes the input",
    ///     ToolSchema::default(),
    ///     |params: Value, _context: ToolExecutionContext| -> Result<Value> {
    ///         Ok(params)
    ///     }
    /// );
    /// ```
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        schema: ToolSchema,
        execute_fn: F,
    ) -> Self {
        let id_str = id.into();
        Self {
            base: crate::base::BaseComponent::new_with_name(
                id_str.clone(),
                crate::compat::Component::Tool,
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

    fn component(&self) -> crate::compat::Component {
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
        options: &ToolExecutionOptions,
    ) -> Result<Value> {
        // Log the tool execution
        self.logger().debug(&format!(
            "Executing tool [id={}] [thread_id={:?}]",
            self.id, context.thread_id
        ));

        // Check if abort is requested
        if context.is_abort_requested() {
            return Err(crate::error::Error::Tool(
                "Tool execution aborted".to_string(),
            ));
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
                self.schema()
                    .with_output_schema(output_schema)
                    .validate_output(&result)?;
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
