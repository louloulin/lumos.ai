//! Tool name resolver for progressive API
//!
//! This module provides functionality to resolve tool names (strings) to tool instances,
//! enabling a more user-friendly API where users can specify tools by name instead of
//! manually creating tool instances.

use crate::error::{Error, Result};
use crate::tool::Tool;
use crate::tool::builtin::{
    CalculatorTool, FileManagerTool, WebSearchTool,
    create_calculator_tool, create_http_request_tool, create_web_scraper_tool,
    create_file_reader_tool, create_file_writer_tool,
    create_json_parser_tool, create_csv_parser_tool,
    create_datetime_tool, create_uuid_generator_tool,
};

/// Resolve a tool name to a tool instance
///
/// This function maps common tool names (strings) to their corresponding tool instances.
/// It supports both builtin tools and provides a way to extend with custom tools.
///
/// # Arguments
///
/// * `name` - The name of the tool (e.g., "web_search", "calculator", "file_manager")
///
/// # Returns
///
/// A `Box<dyn Tool>` if the tool name is recognized, or an error if not found.
///
/// # Examples
///
/// ```rust
/// use lumosai_core::agent::tool_resolver::resolve_tool_name;
///
/// let tool = resolve_tool_name("calculator")?;
/// let tool = resolve_tool_name("web_search")?;
/// ```
pub fn resolve_tool_name(name: &str) -> Result<Box<dyn Tool>> {
    match name.to_lowercase().as_str() {
        // Web tools
        "web_search" | "websearch" => Ok(Box::new(WebSearchTool::new())),
        "http_request" | "http" => Ok(Box::new(create_http_request_tool())),
        "web_scraper" | "scraper" => Ok(Box::new(create_web_scraper_tool())),
        
        // File tools
        "file_manager" | "filemanager" => Ok(Box::new(FileManagerTool::new())),
        "file_reader" | "read_file" => Ok(Box::new(create_file_reader_tool())),
        "file_writer" | "write_file" => Ok(Box::new(create_file_writer_tool())),
        
        // Math tools
        "calculator" | "calc" => Ok(Box::new(create_calculator_tool())),
        
        // Data tools
        "json_parser" | "json" => Ok(Box::new(create_json_parser_tool())),
        "csv_parser" | "csv" => Ok(Box::new(create_csv_parser_tool())),
        
        // System tools
        "datetime" | "date" => Ok(Box::new(create_datetime_tool())),
        "uuid" | "uuid_generator" => Ok(Box::new(create_uuid_generator_tool())),
        
        _ => Err(Error::NotFound(format!(
            "Tool '{}' not found. Available tools: web_search, calculator, file_manager, http_request, file_reader, file_writer, json_parser, csv_parser, datetime, uuid",
            name
        ))),
    }
}

/// Resolve multiple tool names to tool instances
///
/// # Arguments
///
/// * `names` - A slice of tool names
///
/// # Returns
///
/// A vector of tool instances, or an error if any tool name is not found.
///
/// # Examples
///
/// ```rust
/// use lumosai_core::agent::tool_resolver::resolve_tool_names;
///
/// let tools = resolve_tool_names(&["web_search", "calculator"])?;
/// ```
pub fn resolve_tool_names(names: &[&str]) -> Result<Vec<Box<dyn Tool>>> {
    names
        .iter()
        .map(|name| resolve_tool_name(name))
        .collect()
}

/// Get list of available tool names
///
/// Returns a list of all tool names that can be resolved by `resolve_tool_name`.
pub fn get_available_tool_names() -> Vec<&'static str> {
    vec![
        "web_search",
        "http_request",
        "web_scraper",
        "file_manager",
        "file_reader",
        "file_writer",
        "calculator",
        "json_parser",
        "csv_parser",
        "datetime",
        "uuid",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_tool_name() {
        // Test valid tool names
        assert!(resolve_tool_name("calculator").is_ok());
        assert!(resolve_tool_name("web_search").is_ok());
        assert!(resolve_tool_name("file_manager").is_ok());
        
        // Test case insensitive
        assert!(resolve_tool_name("Calculator").is_ok());
        assert!(resolve_tool_name("WEB_SEARCH").is_ok());
        
        // Test aliases
        assert!(resolve_tool_name("calc").is_ok());
        assert!(resolve_tool_name("http").is_ok());
        
        // Test invalid tool name
        assert!(resolve_tool_name("unknown_tool").is_err());
    }

    #[test]
    fn test_resolve_tool_names() {
        let tools = resolve_tool_names(&["calculator", "web_search"]).unwrap();
        assert_eq!(tools.len(), 2);
        
        // Test with invalid tool name
        assert!(resolve_tool_names(&["calculator", "unknown"]).is_err());
    }

    #[test]
    fn test_get_available_tool_names() {
        let names = get_available_tool_names();
        assert!(!names.is_empty());
        assert!(names.contains(&"calculator"));
        assert!(names.contains(&"web_search"));
    }
}

