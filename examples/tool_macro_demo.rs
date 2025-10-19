//! 工具宏演示
//!
//! 展示如何使用 #[tool] 宏来简化工具定义

use lumosai_core::{
    error::Result,
    tool::{Tool, ToolExecutionContext, ToolExecutionOptions},
};
use serde_json::{json, Value};

// 使用 lumos_macro 的 tool 宏
use lumos_macro::tool;

/// 计算器工具 - 基础示例
#[tool(name = "calculator", description = "执行基本的数学计算")]
async fn calculate(expression: String) -> Result<f64> {
    // 简单的表达式计算（实际应用中应该使用更安全的解析器）
    match expression.as_str() {
        "2+2" => Ok(4.0),
        "10-5" => Ok(5.0),
        "3*4" => Ok(12.0),
        "8/2" => Ok(4.0),
        _ => {
            // 尝试简单的解析
            if let Ok(result) = expression.parse::<f64>() {
                Ok(result)
            } else {
                Err(lumosai_core::error::Error::Tool(format!(
                    "无法计算表达式: {}",
                    expression
                )))
            }
        }
    }
}

/// 文本处理工具 - 带可选参数
#[tool(
    name = "text_processor",
    description = "处理文本，支持大小写转换和长度限制"
)]
async fn process_text(
    text: String,
    operation: String,
    max_length: Option<usize>,
) -> Result<String> {
    let mut result = match operation.as_str() {
        "uppercase" => text.to_uppercase(),
        "lowercase" => text.to_lowercase(),
        "reverse" => text.chars().rev().collect(),
        "trim" => text.trim().to_string(),
        _ => text,
    };

    // 应用长度限制
    if let Some(max_len) = max_length {
        if result.len() > max_len {
            result.truncate(max_len);
            result.push_str("...");
        }
    }

    Ok(result)
}

/// 时间工具 - 同步函数示例
#[tool(name = "current_time", description = "获取当前时间信息")]
fn get_current_time(format: Option<String>) -> Result<String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| lumosai_core::error::Error::Tool(format!("时间错误: {}", e)))?;

    let timestamp = now.as_secs();

    match format.as_deref() {
        Some("unix") => Ok(timestamp.to_string()),
        Some("iso") => {
            // 简化的 ISO 格式
            Ok(format!("2024-01-01T00:00:{}Z", timestamp % 60))
        }
        _ => Ok(format!("时间戳: {}", timestamp)),
    }
}

/// 数据验证工具 - 复杂参数示例
#[tool(name = "data_validator", description = "验证数据格式和内容")]
async fn validate_data(
    data: String,
    data_type: String,
    strict_mode: Option<bool>,
) -> Result<Value> {
    let strict = strict_mode.unwrap_or(false);

    let is_valid = match data_type.as_str() {
        "email" => {
            if strict {
                data.contains('@') && data.contains('.') && data.len() > 5
            } else {
                data.contains('@')
            }
        }
        "phone" => {
            if strict {
                data.chars()
                    .all(|c| c.is_ascii_digit() || c == '-' || c == ' ')
                    && data.len() >= 10
            } else {
                data.chars().any(|c| c.is_ascii_digit())
            }
        }
        "url" => {
            if strict {
                data.starts_with("http://") || data.starts_with("https://")
            } else {
                data.contains(".")
            }
        }
        _ => false,
    };

    Ok(serde_json::json!({
        "valid": is_valid,
        "data": data,
        "type": data_type,
        "strict_mode": strict
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 LumosAI 工具宏演示");
    println!("===================");

    // 测试计算器工具
    println!("\n📊 测试计算器工具:");
    let calc_tool = calculate_tool();
    println!("✅ 工具创建成功: {}", calc_tool.id());
    println!("📝 描述: {}", calc_tool.description());

    // 打印工具 schema
    let schema = calc_tool.schema();
    println!("🔧 参数数量: {}", schema.parameters.len());
    for param in &schema.parameters {
        println!(
            "  - {}: {} ({})",
            param.name, param.description, param.r#type
        );
    }

    // 测试工具执行
    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    let params = serde_json::json!({
        "expression": "2+2"
    });

    match calc_tool.execute(params, context, &options).await {
        Ok(result) => println!("🎯 计算结果: {}", result),
        Err(e) => println!("❌ 计算失败: {}", e),
    }

    // 测试文本处理工具
    println!("\n📝 测试文本处理工具:");
    let text_tool = process_text_tool();
    println!("✅ 工具创建成功: {}", text_tool.id());

    let params = serde_json::json!({
        "text": "Hello World",
        "operation": "uppercase",
        "max_length": 8
    });

    let context = ToolExecutionContext::default();
    match text_tool.execute(params, context, &options).await {
        Ok(result) => println!("🎯 处理结果: {}", result),
        Err(e) => println!("❌ 处理失败: {}", e),
    }

    // 测试时间工具
    println!("\n⏰ 测试时间工具:");
    let time_tool = get_current_time_tool();
    println!("✅ 工具创建成功: {}", time_tool.id());

    let params = serde_json::json!({
        "format": "unix"
    });

    let context = ToolExecutionContext::default();
    match time_tool.execute(params, context, &options).await {
        Ok(result) => println!("🎯 时间结果: {}", result),
        Err(e) => println!("❌ 时间获取失败: {}", e),
    }

    // 测试数据验证工具
    println!("\n🔍 测试数据验证工具:");
    let validator_tool = validate_data_tool();
    println!("✅ 工具创建成功: {}", validator_tool.id());

    let params = serde_json::json!({
        "data": "user@example.com",
        "data_type": "email",
        "strict_mode": true
    });

    let context = ToolExecutionContext::default();
    match validator_tool.execute(params, context, &options).await {
        Ok(result) => println!("🎯 验证结果: {}", result),
        Err(e) => println!("❌ 验证失败: {}", e),
    }

    println!("\n🎉 工具宏演示完成！");
    println!("\n📋 总结:");
    println!("  - ✅ 4个工具成功创建");
    println!("  - ✅ 自动生成 Tool trait 实现");
    println!("  - ✅ 参数类型自动推断");
    println!("  - ✅ 支持同步和异步函数");
    println!("  - ✅ 支持可选参数");
    println!("  - ✅ 自动参数验证和类型转换");

    Ok(())
}
