//! 工具模块提供了可由Agent或工作流执行的工具系统
#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
#![allow(non_camel_case_types, ambiguous_glob_reexports, hidden_glob_reexports)]
#![allow(unexpected_cfgs, unused_assignments)]

pub mod builder;
pub mod builtin;
mod context;
pub mod enhanced;
pub mod function;
pub mod registry;
mod schema;
mod tool;
pub mod toolset;

#[cfg(test)]
mod tests;

pub use builder::{create_tool, ToolBuilder};
pub use builtin::{
    create_all_builtin_tools, create_dev_builtin_tools, create_safe_builtin_tools,
    get_tool_categories, get_tool_info, BuiltinToolsConfig, DataProcessingConfig, FileOpsConfig,
    HttpClientConfig, ToolInfo,
};
pub use context::ToolExecutionContext;
pub use enhanced::{EnhancedTool, ToolCapability, ToolCategory as EnhancedToolCategory};
pub use function::{FunctionSchema, FunctionTool};
pub use registry::{ToolCategory, ToolMetadata, ToolRegistry, ToolRegistryStats};
pub use schema::{ParameterSchema, SchemaFormat, ToolExecutionOptions, ToolSchema};
pub use tool::{GenericTool, Tool};
pub use toolset::{ToolSet, ToolSetBuilder, ToolSetError};

// Export built-in tools from builtin module
pub use builtin::{CalculatorTool, CodeExecutorTool, FileManagerTool, WebSearchTool};
