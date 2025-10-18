//! 宏驱动工具的完整测试套件
//!
//! 测试所有使用 #[tool] 宏实现的工具

use crate::tool::builtin::macro_tools::*;
use crate::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;
use tempfile::TempDir;

/// 创建测试用的执行上下文
fn create_test_context() -> ToolExecutionContext {
    ToolExecutionContext {
        thread_id: Some("test-thread".to_string()),
        resource_id: Some("test-resource".to_string()),
        run_id: Some("test-run".to_string()),
        tool_call_id: Some("test-call".to_string()),
        messages: None,
        abort_signal: None,
    }
}

/// 创建测试用的执行选项
fn create_test_options() -> ToolExecutionOptions {
    ToolExecutionOptions {
        context: None,
        validate_params: true,
        validate_output: false,
    }
}

// ============================================================================
// 文件操作工具测试
// ============================================================================

#[tokio::test]
async fn test_file_reader_tool() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    tokio::fs::write(&test_file, "Hello, LumosAI!").await.unwrap();

    let tool = read_file_tool();
    let context = create_test_context();
    let options = create_test_options();

    let params = json!({
        "path": test_file.to_str().unwrap(),
        "encoding": "utf-8",
        "max_size": 1024
    });

    let result = tool.execute(params, context, &options).await.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["content"], "Hello, LumosAI!");
}

#[tokio::test]
async fn test_file_writer_tool() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("output.txt");

    let tool = write_file_tool();
    let context = create_test_context();
    let options = create_test_options();

    let params = json!({
        "path": test_file.to_str().unwrap(),
        "content": "Test content",
        "create_dirs": true
    });

    let result = tool.execute(params, context, &options).await.unwrap();
    assert_eq!(result["success"], true);
    
    // 验证文件已创建
    assert!(test_file.exists());
    let content = tokio::fs::read_to_string(&test_file).await.unwrap();
    assert_eq!(content, "Test content");
}

#[tokio::test]
async fn test_directory_lister_tool() {
    let temp_dir = TempDir::new().unwrap();
    
    // 创建测试文件
    tokio::fs::write(temp_dir.path().join("file1.txt"), "content1").await.unwrap();
    tokio::fs::write(temp_dir.path().join("file2.txt"), "content2").await.unwrap();

    let tool = list_directory_tool();
    let context = create_test_context();
    let options = create_test_options();

    let params = json!({
        "path": temp_dir.path().to_str().unwrap(),
        "recursive": false
    });

    let result = tool.execute(params, context, &options).await.unwrap();
    assert_eq!(result["success"], true);
    assert!(result["total_files"].as_u64().unwrap() >= 2);
}

#[tokio::test]
async fn test_file_info_tool() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("info_test.txt");
    tokio::fs::write(&test_file, "Test").await.unwrap();

    let tool = get_file_info_tool();
    let context = create_test_context();
    let options = create_test_options();

    let params = json!({
        "path": test_file.to_str().unwrap(),
        "check_permissions": true
    });

    let result = tool.execute(params, context, &options).await.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["is_file"], true);
    assert!(result["size"].as_u64().unwrap() > 0);
}

// ============================================================================
// 网络请求工具测试
// ============================================================================

#[tokio::test]
async fn test_http_get_tool() {
    let tool = http_get_tool();
    let context = create_test_context();
    let options = create_test_options();

    let params = json!({
        "url": "https://api.example.com/data",
        "timeout_seconds": 30
    });

    let result = tool.execute(params, context, &options).await.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["status_code"], 200);
}

#[tokio::test]
async fn test_http_post_tool() {
    let tool = http_post_tool();
    let context = create_test_context();
    let options = create_test_options();

    let params = json!({
        "url": "https://api.example.com/create",
        "body": {"name": "test", "value": 123}
    });

    let result = tool.execute(params, context, &options).await.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["status_code"], 201);
}

#[tokio::test]
async fn test_api_call_tool() {
    let tool = api_call_tool();
    let context = create_test_context();
    let options = create_test_options();

    let params = json!({
        "url": "https://api.example.com/resource",
        "method": "PUT",
        "body": {"updated": true}
    });

    let result = tool.execute(params, context, &options).await.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["method"], "PUT");
}

// ============================================================================
// 数据处理工具测试
// ============================================================================

#[tokio::test]
async fn test_json_parser_tool() {
    let tool = parse_json_tool();
    let context = create_test_context();
    let options = create_test_options();

    let params = json!({
        "json_string": r#"{"name": "LumosAI", "version": "0.2.0"}"#
    });

    let result = tool.execute(params, context, &options).await.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["type"], "object");
    assert_eq!(result["parsed"]["name"], "LumosAI");
}

#[tokio::test]
async fn test_text_processor_tool() {
    let tool = process_text_tool();
    let context = create_test_context();
    let options = create_test_options();

    // 测试大写转换
    let params = json!({
        "text": "hello world",
        "operation": "uppercase"
    });

    let result = tool.execute(params, context.clone(), &options).await.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["result"], "HELLO WORLD");

    // 测试长度计算
    let params = json!({
        "text": "LumosAI",
        "operation": "length"
    });

    let result = tool.execute(params, context, &options).await.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["result"], 7);
}

#[tokio::test]
async fn test_data_converter_tool() {
    let tool = convert_data_tool();
    let context = create_test_context();
    let options = create_test_options();

    let params = json!({
        "data": r#"{"key": "value"}"#,
        "from_format": "json",
        "to_format": "yaml"
    });

    let result = tool.execute(params, context, &options).await.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["from_format"], "json");
    assert_eq!(result["to_format"], "yaml");
}

// ============================================================================
// 工具集合测试
// ============================================================================

#[test]
fn test_get_all_macro_tools() {
    let all_tools = get_all_macro_tools();
    
    // 验证工具数量：4个文件 + 3个网络 + 3个数据 = 10个
    assert_eq!(all_tools.len(), 10);
    
    // 验证每个工具都有有效的 ID 和描述
    for tool in all_tools {
        assert!(!tool.id().is_empty());
        assert!(!tool.description().is_empty());
    }
}

#[test]
fn test_tool_categories() {
    let file_tools = get_macro_file_tools();
    let network_tools = get_macro_network_tools();
    let data_tools = get_macro_data_tools();
    
    assert_eq!(file_tools.len(), 4);
    assert_eq!(network_tools.len(), 3);
    assert_eq!(data_tools.len(), 3);
}

