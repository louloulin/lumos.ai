//! 代码分析工具演示
//!
//! 展示如何使用代码质量分析、复杂度分析和安全扫描工具

use lumosai_core::tool::builtin::code_analysis::*;
use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 代码分析工具演示\n");
    println!("{}", "=".repeat(80));

    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();

    // ========================================================================
    // 测试 1: 代码质量分析
    // ========================================================================
    println!("\n📊 测试 1: 代码质量分析");
    println!("{}", "-".repeat(80));

    let quality_tool = code_quality_tool();
    println!("工具名称: {}", quality_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", quality_tool.description());

    // 测试代码示例 (低质量代码)
    let low_quality_code = r#"
fn camelCaseFunction() {
	let x=1;
    let y = 2;
	if x > 0 {
		if y > 0 {
			println!("test");
		}
	}
}
"#;

    println!("\n测试代码:");
    println!("{}", "-".repeat(40));
    println!("{}", low_quality_code);
    println!("{}", "-".repeat(40));

    let params = json!({
        "code": low_quality_code,
        "language": "rust",
        "check_style": true,
        "check_naming": true,
        "check_comments": true
    });

    match quality_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            println!("\n✅ 质量分析结果:");
            println!("  质量评分: {}", result["quality_score"]);
            println!("  代码行数: {}", result["metrics"]["code_lines"]);
            println!("  注释行数: {}", result["metrics"]["comment_lines"]);
            println!("  注释率: {}", result["metrics"]["comment_ratio"]);
            
            if let Some(issues) = result["issues"].as_array() {
                if !issues.is_empty() {
                    println!("\n  发现的问题:");
                    for (i, issue) in issues.iter().enumerate() {
                        println!("    {}. [{}] {}", 
                            i + 1, 
                            issue["severity"].as_str().unwrap_or("unknown"),
                            issue["message"].as_str().unwrap_or("unknown")
                        );
                    }
                }
            }

            if let Some(suggestions) = result["suggestions"].as_array() {
                if !suggestions.is_empty() {
                    println!("\n  改进建议:");
                    for (i, suggestion) in suggestions.iter().enumerate() {
                        println!("    {}. {}", i + 1, suggestion.as_str().unwrap_or("unknown"));
                    }
                }
            }
        }
        Err(e) => println!("❌ 分析失败: {}", e),
    }

    // ========================================================================
    // 测试 2: 代码复杂度分析
    // ========================================================================
    println!("\n\n🔢 测试 2: 代码复杂度分析");
    println!("{}", "-".repeat(80));

    let complexity_tool = code_complexity_tool();
    println!("工具名称: {}", complexity_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", complexity_tool.description());

    // 测试代码示例 (高复杂度代码)
    let complex_code = r#"
fn complex_function(x: i32, y: i32, z: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            if z > 0 {
                for i in 0..10 {
                    if i % 2 == 0 {
                        match i {
                            0 => println!("zero"),
                            1 => println!("one"),
                            _ => println!("other"),
                        }
                    }
                }
            } else {
                while z < 10 {
                    z += 1;
                }
            }
        } else if y < 0 {
            for j in 0..5 {
                println!("{}", j);
            }
        }
    } else {
        return 0;
    }
    x + y + z
}
"#;

    println!("\n测试代码:");
    println!("{}", "-".repeat(40));
    println!("{}", complex_code);
    println!("{}", "-".repeat(40));

    let params = json!({
        "code": complex_code,
        "language": "rust",
        "threshold": 10
    });

    match complexity_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            println!("\n✅ 复杂度分析结果:");
            println!("  圈复杂度: {}", result["cyclomatic_complexity"]);
            println!("  认知复杂度: {}", result["cognitive_complexity"]);
            println!("  最大嵌套深度: {}", result["max_nesting_depth"]);
            println!("  函数数量: {}", result["function_count"]);
            println!("  状态: {}", result["status"]);

            if let Some(warnings) = result["warnings"].as_array() {
                if !warnings.is_empty() {
                    println!("\n  警告:");
                    for (i, warning) in warnings.iter().enumerate() {
                        println!("    {}. [{}] {}", 
                            i + 1,
                            warning["severity"].as_str().unwrap_or("unknown"),
                            warning["message"].as_str().unwrap_or("unknown")
                        );
                    }
                }
            }

            if let Some(functions) = result["functions"].as_array() {
                if !functions.is_empty() {
                    println!("\n  函数复杂度:");
                    for func in functions {
                        println!("    - {}: {} ({})", 
                            func["name"].as_str().unwrap_or("unknown"),
                            func["complexity"],
                            func["status"].as_str().unwrap_or("unknown")
                        );
                    }
                }
            }
        }
        Err(e) => println!("❌ 分析失败: {}", e),
    }

    // ========================================================================
    // 测试 3: 安全扫描
    // ========================================================================
    println!("\n\n🔒 测试 3: 安全扫描");
    println!("{}", "-".repeat(80));

    let security_tool = security_scan_tool();
    println!("工具名称: {}", security_tool.name().unwrap_or("unknown"));
    println!("工具描述: {}", security_tool.description());

    // 测试代码示例 (不安全代码)
    let unsafe_code = r#"
fn connect_database() {
    let password = "hardcoded_password_123";
    let api_key = "sk-1234567890abcdef";
    
    let query = format!("SELECT * FROM users WHERE id = {}", user_input);
    
    let result = database.execute(&query).unwrap();
    
    unsafe {
        let ptr = &result as *const _;
        println!("{:?}", ptr);
    }
}
"#;

    println!("\n测试代码:");
    println!("{}", "-".repeat(40));
    println!("{}", unsafe_code);
    println!("{}", "-".repeat(40));

    let params = json!({
        "code": unsafe_code,
        "language": "rust",
        "scan_level": "strict"
    });

    match security_tool.execute(params, context.clone(), &options).await {
        Ok(result) => {
            println!("\n✅ 安全扫描结果:");
            println!("  风险评分: {}", result["risk_score"]);
            println!("  漏洞数量: {}", result["vulnerability_count"]);
            println!("  状态: {}", result["status"]);

            if let Some(vulnerabilities) = result["vulnerabilities"].as_array() {
                if !vulnerabilities.is_empty() {
                    println!("\n  发现的漏洞:");
                    for (i, vuln) in vulnerabilities.iter().enumerate() {
                        println!("    {}. [{}] {} - {}", 
                            i + 1,
                            vuln["severity"].as_str().unwrap_or("unknown"),
                            vuln["type"].as_str().unwrap_or("unknown"),
                            vuln["message"].as_str().unwrap_or("unknown")
                        );
                    }
                }
            }

            if let Some(recommendations) = result["recommendations"].as_array() {
                if !recommendations.is_empty() {
                    println!("\n  安全建议:");
                    for (i, rec) in recommendations.iter().enumerate() {
                        println!("    {}. {}", i + 1, rec.as_str().unwrap_or("unknown"));
                    }
                }
            }
        }
        Err(e) => println!("❌ 扫描失败: {}", e),
    }

    // ========================================================================
    // 测试 4: 批量获取所有工具
    // ========================================================================
    println!("\n\n📦 测试 4: 获取所有代码分析工具");
    println!("{}", "-".repeat(80));

    let all_tools = get_all_code_analysis_tools();
    println!("总共 {} 个代码分析工具:\n", all_tools.len());

    for (i, tool) in all_tools.iter().enumerate() {
        println!("{}. {} - {}", i + 1, tool.name().unwrap_or("unknown"), tool.description());
    }

    // ========================================================================
    // 测试 5: 高质量代码分析
    // ========================================================================
    println!("\n\n✨ 测试 5: 高质量代码分析");
    println!("{}", "-".repeat(80));

    let good_code = r#"
/// 计算两个数的和
///
/// # 参数
/// - `a`: 第一个数
/// - `b`: 第二个数
///
/// # 返回值
/// 返回两个数的和
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// 主函数
fn main() {
    // 调用 add 函数
    let result = add(1, 2);
    println!("Result: {}", result);
}
"#;

    println!("测试代码:");
    println!("{}", "-".repeat(40));
    println!("{}", good_code);
    println!("{}", "-".repeat(40));

    let params = json!({
        "code": good_code,
        "language": "rust",
        "check_style": true,
        "check_naming": true,
        "check_comments": true
    });

    match code_quality_tool().execute(params, context.clone(), &options).await {
        Ok(result) => {
            println!("\n✅ 质量分析结果:");
            println!("  质量评分: {}", result["quality_score"]);
            println!("  注释率: {}", result["metrics"]["comment_ratio"]);
            println!("  问题数量: {}", result["issues"].as_array().map(|a| a.len()).unwrap_or(0));
        }
        Err(e) => println!("❌ 分析失败: {}", e),
    }

    // ========================================================================
    // 总结
    // ========================================================================
    println!("\n\n{}", "=".repeat(80));
    println!("✅ 代码分析工具演示完成！");
    println!("{}", "=".repeat(80));

    println!("\n📊 工具统计:");
    println!("  - 代码质量分析工具: code_quality");
    println!("  - 代码复杂度分析工具: code_complexity");
    println!("  - 安全扫描工具: security_scan");

    println!("\n💡 使用建议:");
    println!("  1. 代码质量分析: 用于评估代码的可读性和可维护性");
    println!("  2. 复杂度分析: 用于识别过于复杂的代码并进行重构");
    println!("  3. 安全扫描: 用于发现潜在的安全漏洞和风险");

    println!("\n🚀 下一步:");
    println!("  - 集成真实的代码分析引擎（如 clippy, rustfmt）");
    println!("  - 添加更多编程语言支持");
    println!("  - 支持自定义规则和配置");
    println!("{}", "=".repeat(80));

    Ok(())
}

