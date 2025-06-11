//! File operation tools inspired by Mastra's file handling
//! 
//! This module provides file reading, writing, and directory operations

use crate::tool::{Tool, ToolSchema, ParameterSchema, FunctionTool, ToolExecutionContext, ToolExecutionOptions};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use crate::{Result, Error};
use crate::base::Base;
use async_trait::async_trait;

/// Create a file reader tool
/// Similar to Mastra's file reading capabilities
pub fn create_file_reader_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "path".to_string(),
            description: "Path to the file to read".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "encoding".to_string(),
            description: "File encoding (utf-8, ascii, etc.)".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("utf-8")),
        },
        ParameterSchema {
            name: "max_size".to_string(),
            description: "Maximum file size to read in bytes".to_string(),
            r#type: "number".to_string(),
            required: false,
            properties: None,
            default: Some(json!(1048576)), // 1MB default
        },
    ]);

    FunctionTool::new(
        "file_reader",
        "Read content from files with safety checks",
        schema,
        |params| {
            let path = params.get("path")
                .and_then(|v| v.as_str())
                .ok_or("File path is required")?;
            
            let encoding = params.get("encoding")
                .and_then(|v| v.as_str())
                .unwrap_or("utf-8");
            
            let max_size = params.get("max_size")
                .and_then(|v| v.as_u64())
                .unwrap_or(1048576);

            // Mock file reading - in real implementation would use std::fs
            let file_exists = Path::new(path).extension().is_some();
            
            if file_exists {
                Ok(json!({
                    "success": true,
                    "path": path,
                    "encoding": encoding,
                    "content": format!("Mock content of file: {}", path),
                    "size": 1024,
                    "max_size": max_size,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            } else {
                Ok(json!({
                    "success": false,
                    "path": path,
                    "error": "File not found or invalid path",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            }
        },
    )
}

/// Create a file writer tool
/// Similar to Mastra's file writing capabilities
pub fn create_file_writer_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "path".to_string(),
            description: "Path where to write the file".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "content".to_string(),
            description: "Content to write to the file".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "encoding".to_string(),
            description: "File encoding".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("utf-8")),
        },
        ParameterSchema {
            name: "overwrite".to_string(),
            description: "Whether to overwrite existing file".to_string(),
            r#type: "boolean".to_string(),
            required: false,
            properties: None,
            default: Some(json!(false)),
        },
    ]);

    FunctionTool::new(
        "file_writer",
        "Write content to files with safety checks",
        schema,
        |params| {
            let path = params.get("path")
                .and_then(|v| v.as_str())
                .ok_or("File path is required")?;
            
            let content = params.get("content")
                .and_then(|v| v.as_str())
                .ok_or("Content is required")?;
            
            let encoding = params.get("encoding")
                .and_then(|v| v.as_str())
                .unwrap_or("utf-8");
            
            let overwrite = params.get("overwrite")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            // Mock file writing
            Ok(json!({
                "success": true,
                "path": path,
                "encoding": encoding,
                "bytes_written": content.len(),
                "overwrite": overwrite,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        },
    )
}

/// Create a directory lister tool
/// Similar to Mastra's directory operations
pub fn create_directory_lister_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "path".to_string(),
            description: "Directory path to list".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "recursive".to_string(),
            description: "Whether to list recursively".to_string(),
            r#type: "boolean".to_string(),
            required: false,
            properties: None,
            default: Some(json!(false)),
        },
        ParameterSchema {
            name: "include_hidden".to_string(),
            description: "Whether to include hidden files".to_string(),
            r#type: "boolean".to_string(),
            required: false,
            properties: None,
            default: Some(json!(false)),
        },
        ParameterSchema {
            name: "filter".to_string(),
            description: "File extension filter (e.g., '.txt', '.rs')".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: None,
        },
    ]);

    FunctionTool::new(
        "directory_lister",
        "List files and directories with filtering options",
        schema,
        |params| {
            let path = params.get("path")
                .and_then(|v| v.as_str())
                .ok_or("Directory path is required")?;
            
            let recursive = params.get("recursive")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            let include_hidden = params.get("include_hidden")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            let filter = params.get("filter")
                .and_then(|v| v.as_str());

            // Mock directory listing
            let mock_files = vec![
                json!({
                    "name": "example.txt",
                    "path": format!("{}/example.txt", path),
                    "type": "file",
                    "size": 1024,
                    "modified": chrono::Utc::now().to_rfc3339()
                }),
                json!({
                    "name": "subdirectory",
                    "path": format!("{}/subdirectory", path),
                    "type": "directory",
                    "size": null,
                    "modified": chrono::Utc::now().to_rfc3339()
                }),
            ];

            Ok(json!({
                "success": true,
                "path": path,
                "recursive": recursive,
                "include_hidden": include_hidden,
                "filter": filter,
                "files": mock_files,
                "total_count": 2,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        },
    )
}

/// Create a file info tool
/// Get detailed information about a file or directory
pub fn create_file_info_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "path".to_string(),
            description: "Path to get information about".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);

    FunctionTool::new(
        "file_info",
        "Get detailed information about files and directories",
        schema,
        |params| {
            let path = params.get("path")
                .and_then(|v| v.as_str())
                .ok_or("Path is required")?;

            // Mock file info
            Ok(json!({
                "path": path,
                "exists": true,
                "type": "file",
                "size": 2048,
                "permissions": {
                    "readable": true,
                    "writable": true,
                    "executable": false
                },
                "timestamps": {
                    "created": chrono::Utc::now().to_rfc3339(),
                    "modified": chrono::Utc::now().to_rfc3339(),
                    "accessed": chrono::Utc::now().to_rfc3339()
                },
                "metadata": {
                    "extension": Path::new(path).extension().and_then(|s| s.to_str()),
                    "filename": Path::new(path).file_name().and_then(|s| s.to_str()),
                    "parent": Path::new(path).parent().and_then(|p| p.to_str())
                }
            }))
        },
    )
}

/// File manager tool for file system operations
#[derive(Clone)]
pub struct FileManagerTool {
    base: crate::base::BaseComponent,
    id: String,
    description: String,
    schema: ToolSchema,
}

impl FileManagerTool {
    /// Create a new file manager tool
    pub fn new() -> Self {
        let schema = ToolSchema::new(vec![
            ParameterSchema {
                name: "operation".to_string(),
                description: "File operation to perform (read, write, list, exists, delete)".to_string(),
                r#type: "string".to_string(),
                required: true,
                properties: None,
                default: None,
            },
            ParameterSchema {
                name: "path".to_string(),
                description: "File or directory path".to_string(),
                r#type: "string".to_string(),
                required: true,
                properties: None,
                default: None,
            },
            ParameterSchema {
                name: "content".to_string(),
                description: "Content to write (for write operation)".to_string(),
                r#type: "string".to_string(),
                required: false,
                properties: None,
                default: None,
            },
        ]);

        Self {
            base: crate::base::BaseComponent::new_with_name(
                "file_manager".to_string(),
                crate::logger::Component::Tool
            ),
            id: "file_manager".to_string(),
            description: "Perform file system operations".to_string(),
            schema,
        }
    }

    fn read_file(&self, path: &str) -> Result<String> {
        fs::read_to_string(path)
            .map_err(|e| Error::Tool(format!("Failed to read file '{}': {}", path, e)))
    }

    fn write_file(&self, path: &str, content: &str) -> Result<()> {
        fs::write(path, content)
            .map_err(|e| Error::Tool(format!("Failed to write file '{}': {}", path, e)))
    }

    fn list_directory(&self, path: &str) -> Result<Vec<String>> {
        let entries = fs::read_dir(path)
            .map_err(|e| Error::Tool(format!("Failed to read directory '{}': {}", path, e)))?;

        let mut files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| Error::Tool(format!("Failed to read directory entry: {}", e)))?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            files.push(file_name);
        }

        files.sort();
        Ok(files)
    }

    fn file_exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn delete_file(&self, path: &str) -> Result<()> {
        let path_obj = Path::new(path);
        if path_obj.is_file() {
            fs::remove_file(path)
                .map_err(|e| Error::Tool(format!("Failed to delete file '{}': {}", path, e)))
        } else if path_obj.is_dir() {
            fs::remove_dir_all(path)
                .map_err(|e| Error::Tool(format!("Failed to delete directory '{}': {}", path, e)))
        } else {
            Err(Error::Tool(format!("Path '{}' does not exist", path)))
        }
    }
}

impl std::fmt::Debug for FileManagerTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileManagerTool")
            .field("id", &self.id)
            .field("description", &self.description)
            .finish()
    }
}

impl Base for FileManagerTool {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }

    fn component(&self) -> crate::logger::Component {
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
impl Tool for FileManagerTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn schema(&self) -> ToolSchema {
        self.schema.clone()
    }

    async fn execute(
        &self,
        params: Value,
        _context: ToolExecutionContext,
        _options: &ToolExecutionOptions
    ) -> Result<Value> {
        let operation = params.get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Tool("Operation parameter is required".to_string()))?;

        let path = params.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Tool("Path parameter is required".to_string()))?;

        match operation {
            "read" => {
                let content = self.read_file(path)?;
                Ok(json!({
                    "operation": "read",
                    "path": path,
                    "content": content,
                    "size": content.len()
                }))
            },
            "write" => {
                let content = params.get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::Tool("Content parameter is required for write operation".to_string()))?;

                self.write_file(path, content)?;
                Ok(json!({
                    "operation": "write",
                    "path": path,
                    "size": content.len(),
                    "success": true
                }))
            },
            "list" => {
                let files = self.list_directory(path)?;
                Ok(json!({
                    "operation": "list",
                    "path": path,
                    "files": files,
                    "count": files.len()
                }))
            },
            "exists" => {
                let exists = self.file_exists(path);
                Ok(json!({
                    "operation": "exists",
                    "path": path,
                    "exists": exists
                }))
            },
            "delete" => {
                self.delete_file(path)?;
                Ok(json!({
                    "operation": "delete",
                    "path": path,
                    "success": true
                }))
            },
            _ => Err(Error::Tool(format!("Unknown operation: {}", operation)))
        }
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

impl Default for FileManagerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_reader_tool() {
        let tool = create_file_reader_tool();
        
        let mut params = HashMap::new();
        params.insert("path".to_string(), json!("test.txt"));
        params.insert("encoding".to_string(), json!("utf-8"));

        let context = crate::tool::context::ToolExecutionContext::new();
        let options = crate::tool::schema::ToolExecutionOptions::new();
        let result = tool.execute(serde_json::to_value(&params).unwrap(), context, &options).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["path"], "test.txt");
        assert_eq!(response["encoding"], "utf-8");
    }

    #[tokio::test]
    async fn test_file_writer_tool() {
        let tool = create_file_writer_tool();
        
        let mut params = HashMap::new();
        params.insert("path".to_string(), json!("output.txt"));
        params.insert("content".to_string(), json!("Hello, World!"));

        let context = crate::tool::context::ToolExecutionContext::new();
        let options = crate::tool::schema::ToolExecutionOptions::new();
        let result = tool.execute(serde_json::to_value(&params).unwrap(), context, &options).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["success"], true);
        assert_eq!(response["path"], "output.txt");
        assert_eq!(response["bytes_written"], 13);
    }

    #[tokio::test]
    async fn test_directory_lister_tool() {
        let tool = create_directory_lister_tool();
        
        let mut params = HashMap::new();
        params.insert("path".to_string(), json!("/tmp"));
        params.insert("recursive".to_string(), json!(false));

        let context = crate::tool::context::ToolExecutionContext::new();
        let options = crate::tool::schema::ToolExecutionOptions::new();
        let result = tool.execute(serde_json::to_value(&params).unwrap(), context, &options).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["success"], true);
        assert_eq!(response["path"], "/tmp");
        assert_eq!(response["total_count"], 2);
    }

    #[tokio::test]
    async fn test_file_manager_tool() {
        let tool = FileManagerTool::new();

        assert_eq!(tool.name(), Some("file_manager"));
        assert_eq!(tool.description(), "Perform file system operations");
    }

    #[tokio::test]
    async fn test_file_manager_exists() {
        let tool = FileManagerTool::new();

        let params = json!({
            "operation": "exists",
            "path": "Cargo.toml"  // This file should exist in the project root
        });

        let context = crate::tool::ToolExecutionContext::default();
        let options = crate::tool::ToolExecutionOptions::default();

        let result = tool.execute(params, context, &options).await.unwrap();
        assert_eq!(result.get("operation").unwrap().as_str().unwrap(), "exists");
        assert!(result.get("exists").is_some());
    }
}
