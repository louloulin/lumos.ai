//! 宏驱动工具的测试
//! 
//! 验证基于 #[tool] 宏实现的工具功能

use crate::tool::builtin::macro_tools::*;
use crate::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;
use std::collections::HashMap;
use tokio::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_macro_file_reader_tool() {
    // 创建临时文件
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let test_content = "Hello, World! 这是测试内容。";
    
    fs::write(&file_path, test_content).await.unwrap();

    // 创建工具实例
    let tool = read_file_tool();
    
    // 验证工具基本信息
    assert_eq!(tool.id(), "file_reader_v2");
    assert!(tool.description().contains("读取文件内容"));
    
    // 验证工具 schema
    let schema = tool.schema();
    assert_eq!(schema.parameters.len(), 3); // path, encoding, max_size
    
    // 找到 path 参数
    let path_param = schema.parameters.iter()
        .find(|p| p.name == "path")
        .expect("Should have path parameter");
    assert!(path_param.required);
    assert_eq!(path_param.r#type, "string");

    // 测试成功读取文件
    let params = json!({
        "path": file_path.to_string_lossy(),
        "encoding": "utf-8",
        "max_size": 1048576
    });

    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();
    
    let result = tool.execute(params, context, &options).await.unwrap();
    
    // 验证结果
    assert_eq!(result["success"], true);
    assert_eq!(result["content"], test_content);
    assert_eq!(result["encoding"], "utf-8");
    assert!(result["size"].as_u64().unwrap() > 0);
    assert!(result["timestamp"].is_string());

    // 测试文件不存在的情况
    let params = json!({
        "path": "/nonexistent/file.txt"
    });

    let result = tool.execute(params, ToolExecutionContext::default(), &options).await.unwrap();
    assert_eq!(result["success"], false);
    assert!(result["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_macro_file_writer_tool() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("output.txt");
    let test_content = "写入测试内容";

    let tool = write_file_tool();
    
    // 验证工具基本信息
    assert_eq!(tool.id(), "file_writer_v2");
    assert!(tool.description().contains("写入内容到文件"));

    // 测试写入文件
    let params = json!({
        "path": file_path.to_string_lossy(),
        "content": test_content,
        "encoding": "utf-8",
        "create_backup": false,
        "create_dirs": true
    });

    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();
    
    let result = tool.execute(params, context, &options).await.unwrap();
    
    // 验证结果
    assert_eq!(result["success"], true);
    assert_eq!(result["bytes_written"], test_content.len());
    assert_eq!(result["encoding"], "utf-8");
    assert_eq!(result["backup_created"], false);

    // 验证文件确实被写入
    let written_content = fs::read_to_string(&file_path).await.unwrap();
    assert_eq!(written_content, test_content);
}

#[tokio::test]
async fn test_macro_directory_lister_tool() {
    let temp_dir = TempDir::new().unwrap();
    
    // 创建一些测试文件和目录
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.json");
    let subdir = temp_dir.path().join("subdir");
    
    fs::write(&file1, "content1").await.unwrap();
    fs::write(&file2, "{}").await.unwrap();
    fs::create_dir(&subdir).await.unwrap();

    let tool = list_directory_tool();
    
    // 验证工具基本信息
    assert_eq!(tool.id(), "directory_lister_v2");
    assert!(tool.description().contains("列出目录内容"));

    // 测试列出目录
    let params = json!({
        "path": temp_dir.path().to_string_lossy(),
        "recursive": false,
        "show_hidden": false
    });

    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();
    
    let result = tool.execute(params, context, &options).await.unwrap();
    
    // 验证结果
    assert_eq!(result["success"], true);
    assert!(result["total_files"].as_u64().unwrap() >= 2);
    assert!(result["total_directories"].as_u64().unwrap() >= 1);
    
    let entries = result["entries"].as_array().unwrap();
    assert!(entries.len() >= 3); // 至少有 2 个文件和 1 个目录

    // 验证条目包含必要字段
    for entry in entries {
        assert!(entry["name"].is_string());
        assert!(entry["path"].is_string());
        assert!(entry["is_directory"].is_boolean());
        assert!(entry["size"].is_number());
        assert!(entry["modified"].is_string());
    }

    // 测试扩展名过滤
    let params = json!({
        "path": temp_dir.path().to_string_lossy(),
        "extension_filter": "txt"
    });

    let result = tool.execute(params, ToolExecutionContext::default(), &options).await.unwrap();
    assert_eq!(result["success"], true);
    
    let entries = result["entries"].as_array().unwrap();
    // 应该只有 .txt 文件
    for entry in entries {
        if !entry["is_directory"].as_bool().unwrap() {
            let name = entry["name"].as_str().unwrap();
            assert!(name.ends_with(".txt"));
        }
    }
}

#[tokio::test]
async fn test_macro_file_info_tool() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("info_test.txt");
    let test_content = "文件信息测试内容";
    
    fs::write(&file_path, test_content).await.unwrap();

    let tool = get_file_info_tool();
    
    // 验证工具基本信息
    assert_eq!(tool.id(), "file_info_v2");
    assert!(tool.description().contains("获取文件或目录的详细信息"));

    // 测试获取文件信息
    let params = json!({
        "path": file_path.to_string_lossy(),
        "check_permissions": true,
        "calculate_hash": true
    });

    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();
    
    let result = tool.execute(params, context, &options).await.unwrap();
    
    // 验证结果
    assert_eq!(result["success"], true);
    assert_eq!(result["exists"], true);
    assert_eq!(result["is_file"], true);
    assert_eq!(result["is_directory"], false);
    assert_eq!(result["size"], test_content.len());
    assert_eq!(result["extension"], "txt");
    assert_eq!(result["filename"], "info_test.txt");
    
    // 验证权限信息
    assert!(result["permissions"].is_object());
    let permissions = &result["permissions"];
    assert!(permissions["readable"].is_boolean());
    assert!(permissions["writable"].is_boolean());
    assert!(permissions["executable"].is_boolean());
    
    // 验证哈希信息
    assert!(result["hash"].is_object());
    let hash = &result["hash"];
    assert_eq!(hash["algorithm"], "sha256");
    assert!(hash["value"].is_string());

    // 测试目录信息
    let params = json!({
        "path": temp_dir.path().to_string_lossy(),
        "check_permissions": false,
        "calculate_hash": false
    });

    let result = tool.execute(params, ToolExecutionContext::default(), &options).await.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["is_directory"], true);
    assert_eq!(result["is_file"], false);

    // 测试不存在的文件
    let params = json!({
        "path": "/nonexistent/file.txt"
    });

    let result = tool.execute(params, ToolExecutionContext::default(), &options).await.unwrap();
    assert_eq!(result["success"], false);
    assert!(result["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_get_macro_file_tools() {
    let tools = get_macro_file_tools();
    
    // 验证返回了正确数量的工具
    assert_eq!(tools.len(), 4);
    
    // 验证每个工具的 ID
    let tool_ids: Vec<&str> = tools.iter().map(|t| t.id()).collect();
    assert!(tool_ids.contains(&"file_reader_v2"));
    assert!(tool_ids.contains(&"file_writer_v2"));
    assert!(tool_ids.contains(&"directory_lister_v2"));
    assert!(tool_ids.contains(&"file_info_v2"));
    
    // 验证每个工具都有描述
    for tool in &tools {
        assert!(!tool.description().is_empty());
        assert!(tool.schema().parameters.len() > 0);
    }
}

#[test]
fn test_macro_tools_schema_validation() {
    let tools = get_macro_file_tools();
    
    for tool in tools {
        let schema = tool.schema();
        
        // 验证每个工具都有参数
        assert!(!schema.parameters.is_empty(), "Tool {} should have parameters", tool.id());
        
        // 验证必需参数
        let required_params: Vec<_> = schema.parameters.iter()
            .filter(|p| p.required)
            .collect();
        assert!(!required_params.is_empty(), "Tool {} should have at least one required parameter", tool.id());
        
        // 验证参数类型
        for param in &schema.parameters {
            assert!(!param.name.is_empty());
            assert!(!param.description.is_empty());
            assert!(!param.r#type.is_empty());
            assert!(["string", "number", "boolean", "array", "object"].contains(&param.r#type.as_str()));
        }
    }
}
