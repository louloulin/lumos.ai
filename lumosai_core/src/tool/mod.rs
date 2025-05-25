//! 工具模块提供了可由Agent或工作流执行的工具系统

mod schema;
mod tool;
mod function;
mod context;

#[cfg(test)]
mod tests;

pub use schema::{ParameterSchema, ToolSchema, ToolExecutionOptions, SchemaFormat};
pub use tool::{Tool, GenericTool};
pub use function::FunctionTool;
pub use context::ToolExecutionContext;

/// Trait for types that can provide OpenAI function definitions
///
/// This trait is typically implemented automatically via the `FunctionSchema` derive macro.
/// It allows Rust structs to generate their own OpenAI function calling schemas.
pub trait FunctionTool {
    /// Generate an OpenAI function definition for this type
    fn function_definition() -> crate::llm::function_calling::FunctionDefinition;
}