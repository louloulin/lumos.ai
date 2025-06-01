//! 工具模块提供了可由Agent或工作流执行的工具系统

mod schema;
mod tool;
pub mod function;
mod context;
pub mod registry;
pub mod builtin;

#[cfg(test)]
mod tests;

pub use schema::{ParameterSchema, ToolSchema, ToolExecutionOptions, SchemaFormat};
pub use tool::{Tool, GenericTool};
pub use function::{FunctionTool, FunctionSchema};
pub use context::ToolExecutionContext;
pub use registry::{ToolRegistry, ToolMetadata, ToolCategory, ToolRegistryStats};
pub use builtin::{
    create_all_builtin_tools, create_safe_builtin_tools, create_dev_builtin_tools,
    BuiltinToolsConfig, FileOpsConfig, HttpClientConfig, DataProcessingConfig,
    get_tool_categories, get_tool_info, ToolInfo
};