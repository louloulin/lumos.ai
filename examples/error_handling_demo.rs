//! 错误处理演示
//!
//! 演示宏驱动工具的错误处理能力

use lumosai_core::tool::builtin::macro_tools::*;
use lumosai_core::tool::{ToolExecutionContext, ToolExecutionOptions};
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

#[tokio::main]
async fn main() {
    println!("🧪 LumosAI 工具系统错误处理演示\n");
    println!("{}", "=".repeat(80));

    let context = create_test_context();
    let options = create_test_options();

    // 测试 1: 缺少必需参数
    println!("\n📋 测试 1: 缺少必需参数");
    println!("{}", "-".repeat(80));
    let tool = parse_json_tool();
    let params = json!({});
    match tool.execute(params, context.clone(), &options).await {
        Ok(_) => println!("❌ 应该返回错误"),
        Err(e) => println!("✅ 正确返回错误: {:?}", e),
    }

    // 测试 2: 参数类型不匹配 (字符串)
    println!("\n📋 测试 2: 参数类型不匹配 (字符串)");
    println!("{}", "-".repeat(80));
    let tool = parse_json_tool();
    let params = json!({"json_string": 123});
    match tool.execute(params, context.clone(), &options).await {
        Ok(_) => println!("❌ 应该返回错误"),
        Err(e) => println!("✅ 正确返回错误: {:?}", e),
    }

    // 测试 3: 参数类型不匹配 (整数)
    println!("\n📋 测试 3: 参数类型不匹配 (整数)");
    println!("{}", "-".repeat(80));
    let tool = read_file_tool();
    let params = json!({
        "path": "/tmp/test.txt",
        "max_size": "not_a_number"
    });
    match tool.execute(params, context.clone(), &options).await {
        Ok(_) => println!("❌ 应该返回错误"),
        Err(e) => println!("✅ 正确返回错误: {:?}", e),
    }

    // 测试 4: 参数类型不匹配 (布尔值)
    println!("\n📋 测试 4: 参数类型不匹配 (布尔值)");
    println!("{}", "-".repeat(80));
    let tool = list_directory_tool();
    let params = json!({
        "path": "/tmp",
        "recursive": "yes"
    });
    match tool.execute(params, context.clone(), &options).await {
        Ok(_) => println!("❌ 应该返回错误"),
        Err(e) => println!("✅ 正确返回错误: {:?}", e),
    }

    // 测试 5: Value 类型参数 (应该接受任何 JSON 值)
    println!("\n📋 测试 5: Value 类型参数 (应该接受任何 JSON 值)");
    println!("{}", "-".repeat(80));
    let tool = http_post_tool();

    let test_cases = vec![
        ("对象", json!({"test": "data"})),
        ("数组", json!([1, 2, 3])),
        ("字符串", json!("string")),
        ("数字", json!(123)),
        ("布尔值", json!(true)),
    ];

    for (name, body) in test_cases {
        let params = json!({
            "url": "https://api.example.com/test",
            "body": body
        });

        match tool.execute(params, context.clone(), &options).await {
            Ok(_) => println!("✅ {} 类型正确接受", name),
            Err(e) => println!("❌ {} 类型错误拒绝: {:?}", name, e),
        }
    }

    // 测试 6: 可选参数缺失
    println!("\n📋 测试 6: 可选参数缺失");
    println!("{}", "-".repeat(80));
    let tool = read_file_tool();
    let params = json!({"path": "/tmp/nonexistent.txt"});
    match tool.execute(params, context.clone(), &options).await {
        Ok(_) => println!("✅ 可选参数缺失正确处理"),
        Err(e) => println!("⚠️  返回错误 (可能是文件不存在): {:?}", e),
    }

    // 测试 7: 可选参数为 null
    println!("\n📋 测试 7: 可选参数为 null");
    println!("{}", "-".repeat(80));
    let tool = read_file_tool();
    let params = json!({
        "path": "/tmp/nonexistent.txt",
        "encoding": null,
        "max_size": null
    });
    match tool.execute(params, context.clone(), &options).await {
        Ok(_) => println!("✅ 可选参数为 null 正确处理"),
        Err(e) => println!("⚠️  返回错误 (可能是文件不存在): {:?}", e),
    }

    // 测试 8: 额外参数被忽略
    println!("\n📋 测试 8: 额外参数被忽略");
    println!("{}", "-".repeat(80));
    let tool = parse_json_tool();
    let params = json!({
        "json_string": "{\"test\": \"data\"}",
        "extra_param": "should_be_ignored",
        "another_extra": 123
    });
    match tool.execute(params, context.clone(), &options).await {
        Ok(_) => println!("✅ 额外参数正确被忽略"),
        Err(e) => println!("❌ 不应该返回错误: {:?}", e),
    }

    // 测试 9: 空字符串参数
    println!("\n📋 测试 9: 空字符串参数");
    println!("{}", "-".repeat(80));
    let tool = process_text_tool();
    let params = json!({
        "text": "",
        "operation": "length"
    });
    match tool.execute(params, context.clone(), &options).await {
        Ok(value) => {
            println!("✅ 空字符串正确处理");
            println!("   结果: {}", value);
        }
        Err(e) => println!("❌ 不应该返回错误: {:?}", e),
    }

    // 测试 10: Unicode 字符串
    println!("\n📋 测试 10: Unicode 字符串");
    println!("{}", "-".repeat(80));
    let tool = process_text_tool();
    let params = json!({
        "text": "你好，世界！🌍",
        "operation": "length"
    });
    match tool.execute(params, context.clone(), &options).await {
        Ok(value) => {
            println!("✅ Unicode 字符串正确处理");
            println!("   结果: {}", value);
        }
        Err(e) => println!("❌ 不应该返回错误: {:?}", e),
    }

    // 测试 11: 嵌套对象参数
    println!("\n📋 测试 11: 嵌套对象参数");
    println!("{}", "-".repeat(80));
    let tool = http_post_tool();
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
    match tool.execute(params, context.clone(), &options).await {
        Ok(value) => {
            println!("✅ 嵌套对象正确处理");
            println!("   结果: {}", serde_json::to_string_pretty(&value).unwrap());
        }
        Err(e) => println!("❌ 不应该返回错误: {:?}", e),
    }

    // 测试 12: 数组参数
    println!("\n📋 测试 12: 数组参数");
    println!("{}", "-".repeat(80));
    let tool = http_post_tool();
    let params = json!({
        "url": "https://api.example.com/test",
        "body": [1, 2, 3, 4, 5]
    });
    match tool.execute(params, context.clone(), &options).await {
        Ok(value) => {
            println!("✅ 数组参数正确处理");
            println!("   结果: {}", serde_json::to_string_pretty(&value).unwrap());
        }
        Err(e) => println!("❌ 不应该返回错误: {:?}", e),
    }

    println!("\n{}", "=".repeat(80));
    println!("✅ 错误处理演示完成！");
}
