//! LumosAI 工具生态演示 - 对标 LangChain
//!
//! 展示 LumosAI 的丰富工具生态系统，包括：
//! - 35+ 个内置工具
//! - 11 个核心分类
//! - 宏驱动的工具创建
//! - 工具注册和发现机制

use lumosai_core::prelude::*;
use lumosai_core::tool::builtin::*;
use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║     LumosAI 工具生态演示 (对标 LangChain)               ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // 场景 1: 展示所有工具分类
    demo_tool_categories().await?;

    // 场景 2: 演示数学工具
    demo_math_tools().await?;

    // 场景 3: 演示数据处理工具
    demo_data_tools().await?;

    // 场景 4: 演示系统工具
    demo_system_tools().await?;

    // 场景 5: 工具注册和发现
    demo_tool_registry().await?;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║     ✅ 所有工具生态功能验证通过！                       ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    Ok(())
}

/// 场景 1: 展示所有工具分类
async fn demo_tool_categories() -> Result<()> {
    println!("=== 场景 1: 工具分类总览 ===\n");

    let categories = get_tool_categories();

    println!("✅ LumosAI 工具生态包含 {} 个核心分类:\n", categories.len());

    for (category, tools) in categories.iter() {
        println!("📁 {}: {} 个工具", category, tools.len());
        for tool in tools.iter() {
            println!("   - {}", tool);
        }
        println!();
    }

    Ok(())
}

/// 场景 2: 演示数学工具
async fn demo_math_tools() -> Result<()> {
    println!("=== 场景 2: 数学工具演示 ===\n");

    // 创建计算器工具
    let calculator = create_calculator_tool();
    let ctx = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    // 测试基础运算
    let result = calculator
        .execute(json!({"expression": "2 + 3 * 4"}), ctx.clone(), &options)
        .await?;
    println!("✅ 计算器工具:");
    println!("   表达式: 2 + 3 * 4");
    println!("   结果: {}\n", result);

    // 创建统计工具
    let statistics = create_statistics_tool();
    let result = statistics
        .execute(
            json!({
                "numbers": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                "operation": "mean"
            }),
            ctx,
            &options,
        )
        .await?;
    println!("✅ 统计工具:");
    println!("   数据: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]");
    println!("   操作: 计算平均值");
    println!("   结果: {}\n", result);

    Ok(())
}

/// 场景 3: 演示数据处理工具
async fn demo_data_tools() -> Result<()> {
    println!("=== 场景 3: 数据处理工具演示 ===\n");

    let ctx = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    // JSON 解析工具
    let json_parser = create_json_parser_tool();
    let test_json = r#"{"name": "LumosAI", "version": "0.2.0", "tools": 35}"#;

    let result = json_parser
        .execute(json!({"json_string": test_json}), ctx.clone(), &options)
        .await?;
    println!("✅ JSON 解析工具:");
    println!("   输入: {}", test_json);
    println!("   解析结果: {}\n", result);

    // 数据验证工具
    let validator = create_data_validator_tool();
    let result = validator
        .execute(
            json!({
                "data": {"email": "user@example.com", "age": 25},
                "schema": {
                    "email": "email",
                    "age": "number"
                }
            }),
            ctx.clone(),
            &options,
        )
        .await?;
    println!("✅ 数据验证工具:");
    println!("   数据: {{email: user@example.com, age: 25}}");
    println!("   验证结果: {}\n", result);

    // 数据清洗工具
    let cleaner = create_data_cleaner_tool();
    let result = cleaner
        .execute(
            json!({
                "data": ["  hello  ", "world", "", "  ", "test"],
                "operations": ["trim", "remove_empty"]
            }),
            ctx,
            &options,
        )
        .await?;
    println!("✅ 数据清洗工具:");
    println!("   原始数据: [\"  hello  \", \"world\", \"\", \"  \", \"test\"]");
    println!("   清洗操作: trim, remove_empty");
    println!("   清洗结果: {}\n", result);

    Ok(())
}

/// 场景 4: 演示系统工具
async fn demo_system_tools() -> Result<()> {
    println!("=== 场景 4: 系统工具演示 ===\n");

    let ctx = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    // 日期时间工具
    let datetime = create_datetime_tool();
    let result = datetime
        .execute(json!({"operation": "now"}), ctx.clone(), &options)
        .await?;
    println!("✅ 日期时间工具:");
    println!("   操作: 获取当前时间");
    println!("   结果: {}\n", result);

    // UUID 生成器
    let uuid_gen = create_uuid_generator_tool();
    let result = uuid_gen
        .execute(json!({"version": "v4"}), ctx.clone(), &options)
        .await?;
    println!("✅ UUID 生成器:");
    println!("   版本: v4");
    println!("   生成的 UUID: {}\n", result);

    // 哈希生成器
    let hash_gen = create_hash_generator_tool();
    let result = hash_gen
        .execute(
            json!({
                "data": "LumosAI",
                "algorithm": "sha256"
            }),
            ctx,
            &options,
        )
        .await?;
    println!("✅ 哈希生成器:");
    println!("   数据: LumosAI");
    println!("   算法: SHA-256");
    println!("   哈希值: {}\n", result);

    Ok(())
}

/// 场景 5: 工具注册和发现
async fn demo_tool_registry() -> Result<()> {
    println!("=== 场景 5: 工具注册和发现机制 ===\n");

    // 创建所有内置工具
    let config = BuiltinToolsConfig::default();
    let all_tools = create_all_builtin_tools(&config);

    println!("✅ 工具注册统计:");
    println!("   总工具数: {} 个", all_tools.len());
    println!("   工具列表:");

    for (idx, tool) in all_tools.iter().enumerate() {
        println!("   {}. {} - {}", idx + 1, tool.id(), tool.description());
    }
    println!();

    // 创建安全工具集
    let safe_tools = create_safe_builtin_tools(std::env::current_dir()?);
    println!("✅ 安全工具集（生产环境）:");
    println!("   工具数: {} 个", safe_tools.len());
    println!("   说明: 排除了文件和网络工具，仅包含数据处理和计算工具\n");

    // 创建开发工具集
    let dev_tools = create_dev_builtin_tools();
    println!("✅ 开发工具集（开发环境）:");
    println!("   工具数: {} 个", dev_tools.len());
    println!("   说明: 包含所有工具，权限较为宽松\n");

    // 工具发现示例
    println!("✅ 工具发现机制:");
    println!("   按名称查找: 可通过工具名称快速定位");
    println!("   按分类查找: 可按 11 个分类浏览工具");
    println!("   按功能查找: 可通过描述关键词搜索工具\n");

    Ok(())
}
