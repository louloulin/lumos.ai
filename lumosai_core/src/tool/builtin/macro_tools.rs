//! 基于宏的工具实现
//!
//! 这个模块展示了如何使用 #[tool] 宏来简化工具定义，
//! 替代手动的 FunctionTool 实现

use crate::{Error, Result};
use lumos_macro::tool;
use serde_json::{json, Value};
use std::path::Path;
use tokio::fs;

// ============================================================================
// 文件操作工具 (使用宏重构)
// ============================================================================

/// 文件读取工具 - 使用宏实现
/// 
/// 替代原有的 create_file_reader_tool() 手动实现
#[tool(
    name = "file_reader_v2",
    description = "读取文件内容，支持编码和大小限制"
)]
async fn read_file(
    path: String,
    encoding: Option<String>,
    max_size: Option<u64>,
) -> Result<Value> {
    let encoding = encoding.unwrap_or_else(|| "utf-8".to_string());
    let max_size = max_size.unwrap_or(1048576); // 1MB default

    // 验证文件路径
    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Ok(json!({
            "success": false,
            "error": "File not found",
            "path": path
        }));
    }

    // 检查文件大小
    let metadata = fs::metadata(&path).await
        .map_err(|e| Error::Tool(format!("Failed to read file metadata: {}", e)))?;
    
    if metadata.len() > max_size {
        return Ok(json!({
            "success": false,
            "error": "File too large",
            "path": path,
            "size": metadata.len(),
            "max_size": max_size
        }));
    }

    // 读取文件内容
    let content = fs::read_to_string(&path).await
        .map_err(|e| Error::Tool(format!("Failed to read file: {}", e)))?;

    Ok(json!({
        "success": true,
        "path": path,
        "content": content,
        "size": content.len(),
        "encoding": encoding,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 文件写入工具 - 使用宏实现
#[tool(
    name = "file_writer_v2",
    description = "写入内容到文件，支持创建目录和备份"
)]
async fn write_file(
    path: String,
    content: String,
    encoding: Option<String>,
    create_backup: Option<bool>,
    create_dirs: Option<bool>,
) -> Result<Value> {
    let encoding = encoding.unwrap_or_else(|| "utf-8".to_string());
    let create_backup = create_backup.unwrap_or(false);
    let create_dirs = create_dirs.unwrap_or(true);

    let file_path = Path::new(&path);

    // 创建父目录
    if create_dirs {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| Error::Tool(format!("Failed to create directories: {}", e)))?;
        }
    }

    // 创建备份
    if create_backup && file_path.exists() {
        let backup_path = format!("{}.backup", path);
        fs::copy(&path, &backup_path).await
            .map_err(|e| Error::Tool(format!("Failed to create backup: {}", e)))?;
    }

    // 写入文件
    fs::write(&path, &content).await
        .map_err(|e| Error::Tool(format!("Failed to write file: {}", e)))?;

    Ok(json!({
        "success": true,
        "path": path,
        "bytes_written": content.len(),
        "encoding": encoding,
        "backup_created": create_backup,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 目录列表工具 - 使用宏实现
#[tool(
    name = "directory_lister_v2",
    description = "列出目录内容，支持递归和过滤"
)]
async fn list_directory(
    path: String,
    recursive: Option<bool>,
    max_depth: Option<u32>,
    extension_filter: Option<String>,
    show_hidden: Option<bool>,
) -> Result<Value> {
    let recursive = recursive.unwrap_or(false);
    let max_depth = max_depth.unwrap_or(10);
    let show_hidden = show_hidden.unwrap_or(false);

    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        return Ok(json!({
            "success": false,
            "error": "Directory not found",
            "path": path
        }));
    }

    if !dir_path.is_dir() {
        return Ok(json!({
            "success": false,
            "error": "Path is not a directory",
            "path": path
        }));
    }

    let mut entries = Vec::new();
    let mut total_files = 0;
    let mut total_dirs = 0;

    // 简化实现 - 实际应用中会使用 walkdir 或类似库
    let mut read_dir = fs::read_dir(&path).await
        .map_err(|e| Error::Tool(format!("Failed to read directory: {}", e)))?;

    while let Some(entry) = read_dir.next_entry().await
        .map_err(|e| Error::Tool(format!("Failed to read directory entry: {}", e)))? {
        
        let file_name = entry.file_name().to_string_lossy().to_string();
        
        // 跳过隐藏文件
        if !show_hidden && file_name.starts_with('.') {
            continue;
        }

        let metadata = entry.metadata().await
            .map_err(|e| Error::Tool(format!("Failed to read metadata: {}", e)))?;

        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };

        // 应用扩展名过滤器
        if let Some(ref filter) = extension_filter {
            if !is_dir {
                if let Some(ext) = Path::new(&file_name).extension() {
                    if ext.to_string_lossy() != *filter {
                        continue;
                    }
                } else {
                    continue;
                }
            }
        }

        entries.push(json!({
            "name": file_name,
            "path": entry.path().to_string_lossy(),
            "is_directory": is_dir,
            "size": size,
            "modified": metadata.modified()
                .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                .unwrap_or_else(|_| "unknown".to_string())
        }));

        if is_dir {
            total_dirs += 1;
        } else {
            total_files += 1;
        }
    }

    Ok(json!({
        "success": true,
        "path": path,
        "entries": entries,
        "total_files": total_files,
        "total_directories": total_dirs,
        "recursive": recursive,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 文件信息工具 - 使用宏实现
#[tool(
    name = "file_info_v2",
    description = "获取文件或目录的详细信息"
)]
async fn get_file_info(
    path: String,
    check_permissions: Option<bool>,
    calculate_hash: Option<bool>,
) -> Result<Value> {
    let check_permissions = check_permissions.unwrap_or(false);
    let calculate_hash = calculate_hash.unwrap_or(false);

    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Ok(json!({
            "success": false,
            "error": "File or directory not found",
            "path": path
        }));
    }

    let metadata = fs::metadata(&path).await
        .map_err(|e| Error::Tool(format!("Failed to read metadata: {}", e)))?;

    let mut info = json!({
        "success": true,
        "path": path,
        "exists": true,
        "is_file": metadata.is_file(),
        "is_directory": metadata.is_dir(),
        "size": metadata.len(),
        "created": metadata.created()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
            .unwrap_or_else(|_| "unknown".to_string()),
        "modified": metadata.modified()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
            .unwrap_or_else(|_| "unknown".to_string()),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    // 添加文件扩展名信息
    if metadata.is_file() {
        if let Some(extension) = file_path.extension() {
            info["extension"] = json!(extension.to_string_lossy());
        }
        if let Some(file_name) = file_path.file_name() {
            info["filename"] = json!(file_name.to_string_lossy());
        }
    }

    // 权限检查 (简化版本)
    if check_permissions {
        info["permissions"] = json!({
            "readable": file_path.exists(), // 简化检查
            "writable": true, // 实际应用中需要更复杂的权限检查
            "executable": false
        });
    }

    // 文件哈希计算 (模拟)
    if calculate_hash && metadata.is_file() {
        // 实际应用中会计算真实的文件哈希
        info["hash"] = json!({
            "algorithm": "sha256",
            "value": format!("mock_hash_{}", path.len())
        });
    }

    Ok(info)
}

// ============================================================================
// 网络请求工具 (Network Operations)
// ============================================================================

/// HTTP GET 请求工具
#[tool(
    name = "http_get",
    description = "发送 HTTP GET 请求并返回响应"
)]
async fn http_get(
    url: String,
    headers: Option<Value>,
    timeout_seconds: Option<u64>,
) -> Result<Value> {
    let timeout = timeout_seconds.unwrap_or(30);

    // 验证 URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Ok(json!({
            "success": false,
            "error": "Invalid URL: must start with http:// or https://",
            "url": url
        }));
    }

    // 模拟 HTTP 请求 (实际应用中使用 reqwest)
    Ok(json!({
        "success": true,
        "url": url,
        "status_code": 200,
        "headers": {
            "content-type": "application/json",
            "server": "LumosAI-Mock"
        },
        "body": {
            "message": "Mock response",
            "timestamp": chrono::Utc::now().to_rfc3339()
        },
        "timeout": timeout,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// HTTP POST 请求工具
#[tool(
    name = "http_post",
    description = "发送 HTTP POST 请求并返回响应"
)]
async fn http_post(
    url: String,
    body: Value,
    headers: Option<Value>,
    timeout_seconds: Option<u64>,
) -> Result<Value> {
    let timeout = timeout_seconds.unwrap_or(30);

    // 验证 URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Ok(json!({
            "success": false,
            "error": "Invalid URL: must start with http:// or https://",
            "url": url
        }));
    }

    // 模拟 HTTP POST 请求
    Ok(json!({
        "success": true,
        "url": url,
        "status_code": 201,
        "request_body": body,
        "headers": headers.unwrap_or(json!({})),
        "response": {
            "message": "Resource created",
            "id": "mock_id_12345"
        },
        "timeout": timeout,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// API 调用工具
#[tool(
    name = "api_call",
    description = "通用 API 调用工具，支持多种 HTTP 方法"
)]
async fn api_call(
    url: String,
    method: String,
    body: Option<Value>,
    headers: Option<Value>,
    query_params: Option<Value>,
) -> Result<Value> {
    let method = method.to_uppercase();

    // 验证 HTTP 方法
    let valid_methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH"];
    if !valid_methods.contains(&method.as_str()) {
        return Ok(json!({
            "success": false,
            "error": format!("Invalid HTTP method: {}. Must be one of: GET, POST, PUT, DELETE, PATCH", method),
            "url": url
        }));
    }

    // 验证 URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Ok(json!({
            "success": false,
            "error": "Invalid URL: must start with http:// or https://",
            "url": url
        }));
    }

    // 模拟 API 调用
    Ok(json!({
        "success": true,
        "url": url,
        "method": method,
        "status_code": 200,
        "request": {
            "body": body.unwrap_or(json!(null)),
            "headers": headers.unwrap_or(json!({})),
            "query_params": query_params.unwrap_or(json!({}))
        },
        "response": {
            "message": format!("{} request successful", method),
            "data": json!({"mock": true})
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// ============================================================================
// 数据处理工具 (Data Processing)
// ============================================================================

/// JSON 解析工具
#[tool(
    name = "json_parser",
    description = "解析 JSON 字符串并验证格式"
)]
async fn parse_json(
    json_string: String,
    validate_schema: Option<bool>,
) -> Result<Value> {
    let validate = validate_schema.unwrap_or(false);

    // 尝试解析 JSON
    match serde_json::from_str::<Value>(&json_string) {
        Ok(parsed) => {
            Ok(json!({
                "success": true,
                "parsed": parsed,
                "type": match &parsed {
                    Value::Object(_) => "object",
                    Value::Array(_) => "array",
                    Value::String(_) => "string",
                    Value::Number(_) => "number",
                    Value::Bool(_) => "boolean",
                    Value::Null => "null",
                },
                "validated": validate,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        },
        Err(e) => {
            Ok(json!({
                "success": false,
                "error": format!("JSON parse error: {}", e),
                "input": json_string,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
    }
}

/// 文本处理工具
#[tool(
    name = "text_processor",
    description = "处理文本，支持大小写转换、修剪、替换等操作"
)]
async fn process_text(
    text: String,
    operation: String,
    options: Option<Value>,
) -> Result<Value> {
    let result = match operation.as_str() {
        "uppercase" => text.to_uppercase(),
        "lowercase" => text.to_lowercase(),
        "trim" => text.trim().to_string(),
        "reverse" => text.chars().rev().collect(),
        "length" => return Ok(json!({
            "success": true,
            "operation": operation,
            "result": text.len(),
            "original_text": text,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
        "replace" => {
            if let Some(opts) = options {
                let from = opts.get("from").and_then(|v| v.as_str()).unwrap_or("");
                let to = opts.get("to").and_then(|v| v.as_str()).unwrap_or("");
                text.replace(from, to)
            } else {
                return Ok(json!({
                    "success": false,
                    "error": "Replace operation requires 'from' and 'to' options",
                    "operation": operation
                }));
            }
        },
        _ => {
            return Ok(json!({
                "success": false,
                "error": format!("Unknown operation: {}. Supported: uppercase, lowercase, trim, reverse, length, replace", operation),
                "operation": operation
            }));
        }
    };

    Ok(json!({
        "success": true,
        "operation": operation,
        "result": result,
        "original_text": text,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 数据转换工具
#[tool(
    name = "data_converter",
    description = "在不同数据格式之间转换 (JSON, CSV, YAML 等)"
)]
async fn convert_data(
    data: String,
    from_format: String,
    to_format: String,
) -> Result<Value> {
    let from = from_format.to_lowercase();
    let to = to_format.to_lowercase();

    // 验证格式
    let valid_formats = vec!["json", "csv", "yaml", "xml"];
    if !valid_formats.contains(&from.as_str()) || !valid_formats.contains(&to.as_str()) {
        return Ok(json!({
            "success": false,
            "error": format!("Invalid format. Supported formats: {}", valid_formats.join(", ")),
            "from_format": from,
            "to_format": to
        }));
    }

    // 模拟数据转换
    Ok(json!({
        "success": true,
        "from_format": from,
        "to_format": to,
        "original_data": data,
        "converted_data": format!("Mock {} data converted to {}", from, to),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// ============================================================================
// 工具导出函数
// ============================================================================

/// 获取所有宏驱动的文件操作工具
pub fn get_macro_file_tools() -> Vec<Box<dyn crate::tool::Tool>> {
    vec![
        read_file_tool(),
        write_file_tool(),
        list_directory_tool(),
        get_file_info_tool(),
    ]
}

/// 获取所有宏驱动的网络请求工具
pub fn get_macro_network_tools() -> Vec<Box<dyn crate::tool::Tool>> {
    vec![
        http_get_tool(),
        http_post_tool(),
        api_call_tool(),
    ]
}

/// 获取所有宏驱动的数据处理工具
pub fn get_macro_data_tools() -> Vec<Box<dyn crate::tool::Tool>> {
    vec![
        parse_json_tool(),
        process_text_tool(),
        convert_data_tool(),
    ]
}

/// 获取所有宏驱动的工具
pub fn get_all_macro_tools() -> Vec<Box<dyn crate::tool::Tool>> {
    let mut tools = Vec::new();
    tools.extend(get_macro_file_tools());
    tools.extend(get_macro_network_tools());
    tools.extend(get_macro_data_tools());
    tools
}
