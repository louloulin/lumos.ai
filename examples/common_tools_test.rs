//! # 常用工具集成测试
//!
//! 演示如何使用常用的内置工具

use lumosai_core::tools::common::{CalculatorTool, DateTimeTool, SearchTool, TextTool};
use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};

#[tokio::test]
async fn test_calculator_tool() {
    let tool = CalculatorTool;
    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    // 测试加法
    let result = tool
        .execute(serde_json::json!({"expression": "2+2"}), context, &options)
        .await
        .unwrap();

    assert_eq!(result["result"], 4.0);
    println!("✅ CalculatorTool: 2+2 = {}", result["result"]);

    // 测试乘法
    let result = tool
        .execute(serde_json::json!({"expression": "2*2"}), context, &options)
        .await
        .unwrap();

    assert_eq!(result["result"], 4.0);
    println!("✅ CalculatorTool: 2*2 = {}", result["result"]);

    // 测试减法
    let result = tool
        .execute(serde_json::json!({"expression": "10-5"}), context, &options)
        .await
        .unwrap();

    assert_eq!(result["result"], 5.0);
    println!("✅ CalculatorTool: 10-5 = {}", result["result"]);

    // 测试除法
    let result = tool
        .execute(serde_json::json!({"expression": "100/10"}), context, &options)
        .await
        .unwrap();

    assert_eq!(result["result"], 10.0);
    println!("✅ CalculatorTool: 100/10 = {}", result["result"]);
}

#[tokio::test]
async fn test_datetime_tool() {
    let tool = DateTimeTool;
    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    // 测试获取当前时间
    let result = tool
        .execute(serde_json::json!({"operation": "now"}), context, &options)
        .await
        .unwrap();

    assert!(result["result"].is_string());
    assert!(result["timestamp"].is_number());
    println!("✅ DateTimeTool: {}", result["result"]);

    // 测试获取日期
    let result = tool
        .execute(serde_json::json!({"operation": "date"}), context, &options)
        .await
        .unwrap();

    assert!(result["result"].is_string());
    println!("✅ DateTimeTool (date): {}", result["result"]);

    // 测试获取时间
    let result = tool
        .execute(serde_json::json!({"operation": "time"}), context, &options)
        .await
        .unwrap();

    assert!(result["result"].is_string());
    println!("✅ DateTimeTool (time): {}", result["result"]);
}

#[tokio::test]
async fn test_search_tool() {
    let tool = SearchTool::new();
    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    let result = tool
        .execute(serde_json::json!({"query": "Rust programming"}), context, &options)
        .await
        .unwrap();

    assert_eq!(result["query"], "Rust programming");
    assert_eq!(result["count"], 2);
    println!("✅ SearchTool: Found {} results for '{}'", result["count"], result["query"]);
}

#[tokio::test]
async fn test_text_tool() {
    let tool = TextTool;
    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    let input = "Hello World";
    let result = tool
        .execute(serde_json::json!({"text": input}), context, &options)
        .await
        .unwrap();

    assert_eq!(result["words"], 2);
    assert_eq!(result["characters"], 11);
    assert_eq!(result["lines"], 1);
    assert_eq!(result["uppercase"], "HELLO WORLD");
    assert_eq!(result["lowercase"], "hello world");

    println!("✅ TextTool:");
    println!("   Words: {}", result["words"]);
    println!("   Characters: {}", result["characters"]);
    println!("   Lines: {}", result["lines"]);
    println!("   Uppercase: {}", result["uppercase"]);
    println!("   Lowercase: {}", result["lowercase"]);
}

#[tokio::test]
async fn test_all_tools() {
    println!("\n═══════════════════════════════════════");
    println!("   常用工具集成测试");
    println!("═══════════════════════════════════════\n");

    // 测试计算器
    let calc = CalculatorTool;
    let ctx = ToolExecutionContext::default();
    let opts = ToolExecutionOptions::default();

    let result = calc
        .execute(serde_json::json!({"expression": "2+2"}), ctx, &opts)
        .await
        .unwrap();
    println!("🔢 Calculator: 2+2 = {}", result["result"]);

    // 测试日期时间
    let dt = DateTimeTool;
    let result = dt
        .execute(serde_json::json!({"operation": "now"}), ctx, &opts)
        .await
        .unwrap();
    println!("🕐 DateTime: {}", result["result"]);

    // 测试搜索
    let search = SearchTool::new();
    let result = search
        .execute(serde_json::json!({"query": "test"}), ctx, &opts)
        .await
        .unwrap();
    println!("🔍 Search: {} results", result["count"]);

    // 测试文本处理
    let txt = TextTool;
    let result = txt
        .execute(serde_json::json!({"text": "Hello"}), ctx, &opts)
        .await
        .unwrap();
    println!("📝 Text: {} words, {} chars", result["words"], result["characters"]);

    println!("\n✅ 所有工具测试通过!");
    println!("═══════════════════════════════════════\n");
}

fn main() {
    println!("═══════════════════════════════════════");
    println!("   LumosAI 常用工具测试");
    println!("═══════════════════════════════════════");
    println!();
    println!("测试项目:");
    println!("  ✅ CalculatorTool - 数学计算工具");
    println!("  ✅ DateTimeTool - 日期时间工具");
    println!("  ✅ SearchTool - 信息搜索工具");
    println!("  ✅ TextTool - 文本处理工具");
    println!();
    println!("运行测试:");
    println!("  cargo test --example common_tools_test");
    println!();
    println!("═══════════════════════════════════════");
}
