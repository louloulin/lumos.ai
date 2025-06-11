//! 工具模块提供了可由Agent或工作流执行的工具系统
#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
#![allow(non_camel_case_types, ambiguous_glob_reexports, hidden_glob_reexports)]
#![allow(unexpected_cfgs, unused_assignments)]

mod schema;
mod tool;
pub mod function;
mod context;
pub mod registry;
pub mod builtin;
pub mod builder;
pub mod enhanced;
pub mod toolset;

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
pub use builder::{ToolBuilder, create_tool};
pub use enhanced::{EnhancedTool, ToolCapability, ToolCategory as EnhancedToolCategory};
pub use toolset::{ToolSet, ToolSetBuilder, ToolSetError};

// Export built-in tools from builtin module
pub use builtin::{WebSearchTool, CalculatorTool, FileManagerTool, CodeExecutorTool};