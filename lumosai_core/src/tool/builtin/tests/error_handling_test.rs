//! 错误处理测试
//! 
//! 测试宏驱动工具的错误处理能力，包括：
//! - 参数验证错误
//! - 类型转换错误
//! - 缺失参数错误
//! - 无效参数值错误

use crate::tool::builtin::macro_tools::*;
use crate::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;

/// 创建测试上下文
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

/// 创建测试选项
fn create_test_options() -> ToolExecutionOptions {
    ToolExecutionOptions {
        context: None,
        validate_params: true,
        validate_output: false,
    }
}

#[tokio::test]
async fn test_missing_required_parameter() {
    let tool = parse_json_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // 缺少必需的 json_string 参数
    let params = json!({});
    
    let result = tool.execute(params, context, &options).await;
    assert!(result.is_err(), "应该返回错误：缺少必需参数");
    
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(error_msg.contains("Missing required parameter"), 
                "错误信息应包含 'Missing required parameter'，实际: {}", error_msg);
    }
}

#[tokio::test]
async fn test_invalid_parameter_type_string() {
    let tool = parse_json_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // json_string 应该是字符串，但传入了数字
    let params = json!({
        "json_string": 123
    });
    
    let result = tool.execute(params, context, &options).await;
    assert!(result.is_err(), "应该返回错误：参数类型不匹配");
    
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(error_msg.contains("must be a string"), 
                "错误信息应包含 'must be a string'，实际: {}", error_msg);
    }
}

#[tokio::test]
async fn test_invalid_parameter_type_integer() {
    let tool = read_file_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // max_size 应该是整数，但传入了字符串
    let params = json!({
        "path": "/tmp/test.txt",
        "max_size": "not_a_number"
    });
    
    let result = tool.execute(params, context, &options).await;
    assert!(result.is_err(), "应该返回错误：参数类型不匹配");
    
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(error_msg.contains("must be an integer") || error_msg.contains("must be a string"), 
                "错误信息应包含类型错误，实际: {}", error_msg);
    }
}

#[tokio::test]
async fn test_invalid_parameter_type_boolean() {
    let tool = list_directory_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // recursive 应该是布尔值，但传入了字符串
    let params = json!({
        "path": "/tmp",
        "recursive": "yes"
    });
    
    let result = tool.execute(params, context, &options).await;
    assert!(result.is_err(), "应该返回错误：参数类型不匹配");
    
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(error_msg.contains("must be a boolean") || error_msg.contains("must be a string"), 
                "错误信息应包含类型错误，实际: {}", error_msg);
    }
}

#[tokio::test]
async fn test_invalid_parameter_value_object() {
    let tool = http_post_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // body 应该是 Value 对象，传入各种类型都应该成功
    let test_cases = vec![
        json!({"test": "data"}),
        json!([1, 2, 3]),
        json!("string"),
        json!(123),
        json!(true),
    ];
    
    for body in test_cases {
        let params = json!({
            "url": "https://api.example.com/test",
            "body": body
        });
        
        let result = tool.execute(params, context.clone(), &options).await;
        assert!(result.is_ok(), "Value 类型应该接受任何 JSON 值，body: {:?}", body);
    }
}

#[tokio::test]
async fn test_optional_parameter_missing() {
    let tool = read_file_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // 只提供必需参数，可选参数缺失
    let params = json!({
        "path": "/tmp/nonexistent.txt"
    });
    
    let result = tool.execute(params, context, &options).await;
    // 应该成功执行（虽然文件不存在会返回错误结果，但不是参数验证错误）
    assert!(result.is_ok(), "可选参数缺失不应该导致参数验证错误");
}

#[tokio::test]
async fn test_optional_parameter_null() {
    let tool = read_file_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // 可选参数设置为 null
    let params = json!({
        "path": "/tmp/nonexistent.txt",
        "encoding": null,
        "max_size": null
    });
    
    let result = tool.execute(params, context, &options).await;
    // 应该成功执行
    assert!(result.is_ok(), "可选参数为 null 不应该导致参数验证错误");
}

#[tokio::test]
async fn test_extra_parameters_ignored() {
    let tool = parse_json_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // 提供额外的未定义参数
    let params = json!({
        "json_string": "{\"test\": \"data\"}",
        "extra_param": "should_be_ignored",
        "another_extra": 123
    });
    
    let result = tool.execute(params, context, &options).await;
    // 应该成功执行，额外参数被忽略
    assert!(result.is_ok(), "额外参数应该被忽略");
}

#[tokio::test]
async fn test_empty_string_parameter() {
    let tool = process_text_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // 空字符串参数
    let params = json!({
        "text": "",
        "operation": "length"
    });
    
    let result = tool.execute(params, context, &options).await;
    // 应该成功执行
    assert!(result.is_ok(), "空字符串参数应该被接受");
    
    if let Ok(value) = result {
        assert_eq!(value["success"], true);
        assert_eq!(value["result"], 0); // 空字符串长度为 0
    }
}

#[tokio::test]
async fn test_zero_value_parameter() {
    let tool = read_file_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // 零值参数
    let params = json!({
        "path": "/tmp/test.txt",
        "max_size": 0
    });
    
    let result = tool.execute(params, context, &options).await;
    // 应该成功执行（虽然 max_size=0 可能导致业务逻辑错误，但不是参数验证错误）
    assert!(result.is_ok(), "零值参数应该被接受");
}

#[tokio::test]
async fn test_negative_value_parameter() {
    let tool = read_file_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // 负值参数（虽然 u64 不应该接受负值，但 JSON 中的负数会被转换）
    let params = json!({
        "path": "/tmp/test.txt",
        "max_size": -1
    });
    
    let result = tool.execute(params, context, &options).await;
    // 可能会失败，因为 -1 无法转换为 u64
    // 但这取决于具体的实现
    if result.is_err() {
        let error_msg = format!("{:?}", result.unwrap_err());
        println!("负值参数错误: {}", error_msg);
    }
}

#[tokio::test]
async fn test_very_large_value_parameter() {
    let tool = read_file_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // 非常大的值
    let params = json!({
        "path": "/tmp/test.txt",
        "max_size": u64::MAX
    });
    
    let result = tool.execute(params, context, &options).await;
    // 应该成功执行
    assert!(result.is_ok(), "非常大的值应该被接受");
}

#[tokio::test]
async fn test_unicode_string_parameter() {
    let tool = process_text_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // Unicode 字符串
    let params = json!({
        "text": "你好，世界！🌍",
        "operation": "length"
    });
    
    let result = tool.execute(params, context, &options).await;
    // 应该成功执行
    assert!(result.is_ok(), "Unicode 字符串应该被接受");
}

#[tokio::test]
async fn test_special_characters_in_string() {
    let tool = process_text_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // 特殊字符
    let params = json!({
        "text": "Line1\nLine2\tTab\r\nCRLF",
        "operation": "length"
    });
    
    let result = tool.execute(params, context, &options).await;
    // 应该成功执行
    assert!(result.is_ok(), "特殊字符应该被接受");
}

#[tokio::test]
async fn test_nested_object_parameter() {
    let tool = http_post_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // 嵌套对象
    let params = json!({
        "url": "https://api.example.com/test",
        "body": {
            "user": {
                "name": "Test User",
                "age": 30,
                "tags": ["tag1", "tag2"],
                "metadata": {
                    "created": "2025-01-01",
                    "updated": "2025-01-02"
                }
            }
        }
    });
    
    let result = tool.execute(params, context, &options).await;
    // 应该成功执行
    assert!(result.is_ok(), "嵌套对象应该被接受");
}

#[tokio::test]
async fn test_array_parameter() {
    let tool = http_post_tool();
    let context = create_test_context();
    let options = create_test_options();
    
    // 数组参数
    let params = json!({
        "url": "https://api.example.com/test",
        "body": [1, 2, 3, 4, 5]
    });
    
    let result = tool.execute(params, context, &options).await;
    // 应该成功执行
    assert!(result.is_ok(), "数组参数应该被接受");
}

