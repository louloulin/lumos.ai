//! API 测试工具演示程序
//!
//! 演示如何使用 API 测试工具进行端点测试、性能测试和负载测试
//!
//! 运行方式:
//! ```bash
//! cargo run --example api_testing_demo
//! ```

use lumosai_core::tool::builtin::api_testing::*;
use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 API 测试工具演示\n");
    println!("{}", "=".repeat(80));

    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    // ========================================================================
    // 测试 1: API 端点测试
    // ========================================================================
    println!("\n📍 测试 1: API 端点测试");
    println!("{}", "-".repeat(80));

    let endpoint_tool = endpoint_test_tool();
    println!("工具名称: {}", endpoint_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", endpoint_tool.description());

    let params = json!({
        "url": "https://api.github.com/users/octocat",
        "method": "GET",
        "expected_status": 200,
        "timeout_seconds": 30
    });

    println!("\n测试参数:");
    println!("  URL: https://api.github.com/users/octocat");
    println!("  方法: GET");
    println!("  期望状态码: 200");

    match endpoint_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            println!("\n✅ 测试结果:");
            println!("  成功: {}", result["success"]);
            println!("  状态码: {}", result["status_code"]);
            println!("  响应时间: {} ms", result["response_time_ms"]);
            println!("  响应体: {}", result["response_body"]);
        }
        Err(e) => {
            println!("\n❌ 测试失败: {}", e);
        }
    }

    // ========================================================================
    // 测试 2: API 性能测试
    // ========================================================================
    println!("\n\n⚡ 测试 2: API 性能测试");
    println!("{}", "-".repeat(80));

    let performance_tool = performance_test_tool();
    println!("工具名称: {}", performance_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", performance_tool.description());

    let params = json!({
        "url": "https://api.example.com/test",
        "method": "GET",
        "requests_count": 100,
        "concurrent": 10,
        "timeout_seconds": 60
    });

    println!("\n测试参数:");
    println!("  URL: https://api.example.com/test");
    println!("  方法: GET");
    println!("  请求次数: 100");
    println!("  并发数: 10");

    match performance_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            println!("\n✅ 性能测试结果:");
            println!("  总请求数: {}", result["total_requests"]);
            println!("  成功请求: {}", result["successful_requests"]);
            println!("  失败请求: {}", result["failed_requests"]);
            println!("  平均响应时间: {} ms", result["avg_response_time_ms"]);
            println!("  最小响应时间: {} ms", result["min_response_time_ms"]);
            println!("  最大响应时间: {} ms", result["max_response_time_ms"]);
            println!("  吞吐量: {} req/s", result["requests_per_second"]);
            println!("  总测试时间: {} ms", result["total_duration_ms"]);
        }
        Err(e) => {
            println!("\n❌ 测试失败: {}", e);
        }
    }

    // ========================================================================
    // 测试 3: API 负载测试
    // ========================================================================
    println!("\n\n🔥 测试 3: API 负载测试");
    println!("{}", "-".repeat(80));

    let load_tool = load_test_tool();
    println!("工具名称: {}", load_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", load_tool.description());

    let params = json!({
        "url": "https://api.example.com/test",
        "method": "POST",
        "duration_seconds": 60,
        "concurrent_users": 50,
        "ramp_up_seconds": 10
    });

    println!("\n测试参数:");
    println!("  URL: https://api.example.com/test");
    println!("  方法: POST");
    println!("  测试时长: 60 秒");
    println!("  并发用户: 50");
    println!("  爬坡时间: 10 秒");

    match load_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            println!("\n✅ 负载测试结果:");
            println!("  总请求数: {}", result["total_requests"]);
            println!("  成功请求: {}", result["successful_requests"]);
            println!("  失败请求: {}", result["failed_requests"]);
            println!("  错误率: {:.2}%", result["error_rate"]);
            println!("  平均响应时间: {} ms", result["avg_response_time_ms"]);
            println!("  P50 响应时间: {} ms", result["p50_response_time_ms"]);
            println!("  P95 响应时间: {} ms", result["p95_response_time_ms"]);
            println!("  P99 响应时间: {} ms", result["p99_response_time_ms"]);
            println!("  最大并发用户: {}", result["max_concurrent_users"]);
            println!("  平均吞吐量: {} req/s", result["requests_per_second"]);
        }
        Err(e) => {
            println!("\n❌ 测试失败: {}", e);
        }
    }

    // ========================================================================
    // 测试 4: 批量获取所有工具
    // ========================================================================
    println!("\n\n📦 测试 4: 获取所有 API 测试工具");
    println!("{}", "-".repeat(80));

    let all_tools = get_all_api_testing_tools();
    println!("总共 {} 个 API 测试工具:\n", all_tools.len());

    for (i, tool) in all_tools.iter().enumerate() {
        println!("{}. {} - {}", i + 1, tool.name().unwrap_or("unknown"), tool.description());
    }

    // ========================================================================
    // 测试 5: 参数验证测试
    // ========================================================================
    println!("\n\n🔍 测试 5: 参数验证测试");
    println!("{}", "-".repeat(80));

    println!("\n测试缺少必需参数:");
    let invalid_params = json!({
        "method": "GET"
        // 缺少 url 参数
    });

    match endpoint_test_tool().execute(invalid_params, context.clone(), &options).await {
        Ok(_) => println!("  ⚠️  应该返回错误但没有"),
        Err(e) => println!("  ✅ 正确返回错误: {}", e),
    }

    println!("\n测试无效的 HTTP 方法:");
    let invalid_params = json!({
        "url": "https://api.example.com/test",
        "method": "INVALID_METHOD"
    });

    match endpoint_test_tool().execute(invalid_params, context.clone(), &options).await {
        Ok(result) => println!("  ✅ 工具接受了参数: {}", result["method"]),
        Err(e) => println!("  ❌ 返回错误: {}", e),
    }

    // ========================================================================
    // 测试 6: 边界值测试
    // ========================================================================
    println!("\n\n🎯 测试 6: 边界值测试");
    println!("{}", "-".repeat(80));

    println!("\n测试极小请求数:");
    let params = json!({
        "url": "https://api.example.com/test",
        "method": "GET",
        "requests_count": 1,
        "concurrent": 1
    });

    match performance_test_tool().execute(params, context.clone(), &options).await {
        Ok(result) => {
            println!("  ✅ 成功处理 1 个请求");
            println!("     总请求数: {}", result["total_requests"]);
        }
        Err(e) => println!("  ❌ 失败: {}", e),
    }

    println!("\n测试大量请求:");
    let params = json!({
        "url": "https://api.example.com/test",
        "method": "GET",
        "requests_count": 10000,
        "concurrent": 100
    });

    match performance_test_tool().execute(params, context.clone(), &options).await {
        Ok(result) => {
            println!("  ✅ 成功处理 10000 个请求");
            println!("     总请求数: {}", result["total_requests"]);
            println!("     吞吐量: {} req/s", result["requests_per_second"]);
        }
        Err(e) => println!("  ❌ 失败: {}", e),
    }

    // ========================================================================
    // 总结
    // ========================================================================
    println!("\n\n{}", "=".repeat(80));
    println!("✅ API 测试工具演示完成！");
    println!("{}", "=".repeat(80));
    println!("\n📊 工具统计:");
    println!("  - 端点测试工具: endpoint_test");
    println!("  - 性能测试工具: performance_test");
    println!("  - 负载测试工具: load_test");
    println!("\n💡 使用建议:");
    println!("  1. 端点测试: 用于验证 API 的基本可用性");
    println!("  2. 性能测试: 用于评估 API 的响应时间和吞吐量");
    println!("  3. 负载测试: 用于测试 API 在高负载下的稳定性");
    println!("\n🚀 下一步:");
    println!("  - 集成真实的 HTTP 客户端（reqwest）");
    println!("  - 添加更多测试指标（内存使用、CPU 使用等）");
    println!("  - 支持自定义断言和验证规则");

    Ok(())
}

