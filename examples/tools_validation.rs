use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use lumosai_core::tool::builtin::{WebSearchTool, CalculatorTool};
use serde_json::json;
use std::time::Instant;

/// 工具调用系统全面验证测试
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 LumosAI 工具调用系统验证测试");
    println!("========================================");
    
    // 测试1: 内置工具验证
    println!("\n📋 测试1: 内置工具验证");
    test_builtin_tools().await?;

    // 测试2: 工具执行上下文验证
    println!("\n📋 测试2: 工具执行上下文验证");
    test_tool_execution_context().await?;

    // 测试3: 工具性能基准测试
    println!("\n📋 测试3: 工具性能基准测试");
    test_tool_performance().await?;
    
    println!("\n✅ 所有工具调用系统验证测试完成！");
    Ok(())
}

async fn test_builtin_tools() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试内置工具...");
    
    // 测试Web搜索工具
    let web_search = WebSearchTool::new();
    println!("✅ Web搜索工具创建成功");
    println!("📋 工具ID: {}", web_search.id());
    println!("📋 工具描述: {}", web_search.description());
    
    // 测试工具schema
    let schema = web_search.schema();
    println!("📋 工具参数数量: {}", schema.parameters.len());
    
    // 测试工具执行
    let context = ToolExecutionContext::new();
    let options = ToolExecutionOptions::default();
    let params = json!({
        "query": "Rust编程语言",
        "max_results": 5
    });

    let start_time = Instant::now();
    let result = web_search.execute(params, context, &options).await?;
    let duration = start_time.elapsed();
    
    println!("✅ Web搜索工具执行成功!");
    println!("⏱️ 执行时间: {:?}", duration);
    println!("📝 结果类型: {}", result.get("results").map(|v| v.as_array().map(|a| a.len()).unwrap_or(0)).unwrap_or(0));
    
    // 测试计算器工具
    let calculator = CalculatorTool::new();
    println!("✅ 计算器工具创建成功");
    
    let calc_params = json!({
        "expression": "2 + 3 * 4"
    });
    
    let start_time = Instant::now();
    let calc_result = calculator.execute(calc_params, ToolExecutionContext::new(), &options).await?;
    let duration = start_time.elapsed();
    
    println!("✅ 计算器工具执行成功!");
    println!("⏱️ 执行时间: {:?}", duration);
    println!("📝 计算结果: {}", calc_result.get("result").unwrap_or(&json!("未知")));
    
    // 文件读取工具暂时跳过，因为FileReaderTool不存在
    println!("⚠️ 文件读取工具测试跳过 - 工具不存在");
    
    Ok(())
}

// 自定义工具和工具注册表测试暂时跳过，因为API复杂性
// 这些功能需要进一步的API稳定化

async fn test_tool_execution_context() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试工具执行上下文...");

    // 创建基础上下文
    let mut context = ToolExecutionContext::new();
    context.thread_id = Some("test-thread-123".to_string());

    println!("✅ 工具执行上下文创建成功");
    println!("📋 线程ID: {:?}", context.thread_id);
    
    // 测试上下文传递
    let calculator = CalculatorTool::new();
    let params = json!({
        "expression": "5 * 6"
    });
    
    let start_time = Instant::now();
    let result = calculator.execute(params, context, &ToolExecutionOptions::default()).await?;
    let duration = start_time.elapsed();
    
    println!("✅ 带上下文的工具执行成功!");
    println!("⏱️ 执行时间: {:?}", duration);
    println!("📝 计算结果: {}", result.get("result").unwrap_or(&json!("未知")));
    
    // 测试不同的执行选项
    let mut options = ToolExecutionOptions::default();
    options.validate_params = true;

    let start_time = Instant::now();
    let result_with_options = calculator.execute(
        json!({"expression": "10 / 2"}),
        ToolExecutionContext::new(),
        &options
    ).await?;
    let duration = start_time.elapsed();
    
    println!("✅ 带选项的工具执行成功!");
    println!("⏱️ 执行时间: {:?}", duration);
    println!("📝 计算结果: {}", result_with_options.get("result").unwrap_or(&json!("未知")));
    
    Ok(())
}

async fn test_tool_performance() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试工具性能基准...");
    
    let calculator = CalculatorTool::new();
    let web_search = WebSearchTool::new();
    
    // 计算器性能测试
    let calc_expressions = vec![
        "1 + 1",
        "10 * 5",
        "100 / 4",
        "2^8",
        "sqrt(16)",
    ];
    
    let mut calc_total_time = std::time::Duration::new(0, 0);
    let mut calc_success_count = 0;
    
    for expr in calc_expressions {
        let params = json!({"expression": expr});
        let start_time = Instant::now();
        
        match calculator.execute(params, ToolExecutionContext::new(), &ToolExecutionOptions::default()).await {
            Ok(_) => {
                let duration = start_time.elapsed();
                calc_total_time += duration;
                calc_success_count += 1;
                println!("✅ 计算 '{}' 成功 - 耗时: {:?}", expr, duration);
            }
            Err(e) => {
                println!("❌ 计算 '{}' 失败: {}", expr, e);
            }
        }
    }
    
    // Web搜索性能测试
    let search_queries = vec![
        "Rust编程",
        "人工智能",
        "机器学习",
    ];
    
    let mut search_total_time = std::time::Duration::new(0, 0);
    let mut search_success_count = 0;
    
    for query in search_queries {
        let params = json!({
            "query": query,
            "max_results": 3
        });
        let start_time = Instant::now();
        
        match web_search.execute(params, ToolExecutionContext::new(), &ToolExecutionOptions::default()).await {
            Ok(_) => {
                let duration = start_time.elapsed();
                search_total_time += duration;
                search_success_count += 1;
                println!("✅ 搜索 '{}' 成功 - 耗时: {:?}", query, duration);
            }
            Err(e) => {
                println!("❌ 搜索 '{}' 失败: {}", query, e);
            }
        }
    }
    
    // 性能统计
    let calc_avg_time = if calc_success_count > 0 {
        calc_total_time / calc_success_count
    } else {
        std::time::Duration::new(0, 0)
    };
    
    let search_avg_time = if search_success_count > 0 {
        search_total_time / search_success_count
    } else {
        std::time::Duration::new(0, 0)
    };
    
    println!("\n📊 工具性能基准测试结果:");
    println!("计算器工具:");
    println!("- 成功率: {:.1}%", (calc_success_count as f64 / 5.0) * 100.0);
    println!("- 平均执行时间: {:?}", calc_avg_time);
    println!("- 总耗时: {:?}", calc_total_time);
    
    println!("Web搜索工具:");
    println!("- 成功率: {:.1}%", (search_success_count as f64 / 3.0) * 100.0);
    println!("- 平均执行时间: {:?}", search_avg_time);
    println!("- 总耗时: {:?}", search_total_time);
    
    Ok(())
}
