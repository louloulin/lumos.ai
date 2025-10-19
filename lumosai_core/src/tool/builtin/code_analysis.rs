//! 代码分析工具
//!
//! 提供代码质量分析、复杂度分析和安全扫描功能。
//!
//! ## 工具列表
//!
//! - `code_quality_tool`: 代码质量分析
//! - `code_complexity_tool`: 代码复杂度分析
//! - `security_scan_tool`: 安全扫描
//!
//! ## 使用示例
//!
//! ```rust
//! use lumosai_core::tool::builtin::code_analysis::*;
//! use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() {
//!     let tool = code_quality_tool();
//!     let params = json!({
//!         "code": "fn main() { println!(\"Hello\"); }",
//!         "language": "rust"
//!     });
//!     let context = ToolExecutionContext::default();
//!     let options = ToolExecutionOptions::default();
//!     
//!     let result = tool.execute(params, context, &options).await.unwrap();
//!     println!("Quality score: {}", result["quality_score"]);
//! }
//! ```

use lumos_macro::tool;
use serde_json::{json, Value};
use crate::error::Result;

/// 代码质量分析工具
///
/// 分析代码的质量指标，包括可读性、可维护性、复杂度等。
///
/// # 参数
///
/// - `code` (必需): 要分析的代码字符串
/// - `language` (必需): 编程语言 (rust, python, javascript, typescript, go, java)
/// - `check_style` (可选): 是否检查代码风格，默认 true
/// - `check_naming` (可选): 是否检查命名规范，默认 true
/// - `check_comments` (可选): 是否检查注释覆盖率，默认 true
///
/// # 返回值
///
/// 返回一个 JSON 对象，包含以下字段：
/// - `success` (boolean): 分析是否成功
/// - `quality_score` (number): 质量评分 (0-100)
/// - `metrics` (object): 详细指标
///   - `lines_of_code` (number): 代码行数
///   - `comment_lines` (number): 注释行数
///   - `blank_lines` (number): 空白行数
///   - `comment_ratio` (number): 注释率 (%)
/// - `issues` (array): 发现的问题列表
/// - `suggestions` (array): 改进建议列表
///
/// # 示例
///
/// ```rust
/// let params = json!({
///     "code": "fn main() { println!(\"Hello\"); }",
///     "language": "rust",
///     "check_style": true
/// });
/// ```
#[tool(
    name = "code_quality",
    description = "分析代码质量指标（可读性、可维护性、复杂度）"
)]
async fn code_quality(
    code: String,
    language: String,
    check_style: Option<bool>,
    check_naming: Option<bool>,
    check_comments: Option<bool>,
) -> Result<Value> {
    let check_style = check_style.unwrap_or(true);
    let check_naming = check_naming.unwrap_or(true);
    let check_comments = check_comments.unwrap_or(true);

    // 基础代码分析
    let lines: Vec<&str> = code.lines().collect();
    let total_lines = lines.len();
    let blank_lines = lines.iter().filter(|l| l.trim().is_empty()).count();
    let comment_lines = lines.iter().filter(|l| {
        let trimmed = l.trim();
        trimmed.starts_with("//") || trimmed.starts_with("#") || trimmed.starts_with("/*")
    }).count();
    let code_lines = total_lines - blank_lines - comment_lines;
    let comment_ratio = if total_lines > 0 {
        (comment_lines as f64 / total_lines as f64) * 100.0
    } else {
        0.0
    };

    // 质量评分计算 (简化版)
    let mut quality_score: f64 = 100.0;
    let mut issues = Vec::new();
    let mut suggestions = Vec::new();

    // 检查注释率
    if check_comments && comment_ratio < 10.0 {
        quality_score -= 20.0;
        issues.push(json!({
            "type": "low_comment_ratio",
            "severity": "warning",
            "message": format!("注释率过低: {:.1}%", comment_ratio)
        }));
        suggestions.push("建议增加代码注释以提高可读性");
    }

    // 检查代码长度
    if code_lines > 500 {
        quality_score -= 10.0;
        issues.push(json!({
            "type": "long_code",
            "severity": "info",
            "message": format!("代码行数较多: {} 行", code_lines)
        }));
        suggestions.push("考虑将代码拆分为更小的模块");
    }

    // 检查命名规范 (简化版)
    if check_naming {
        if language == "rust" && code.contains("fn ") {
            // 检查是否有驼峰命名的函数 (Rust 应该使用 snake_case)
            if code.contains("fn camelCase") || code.contains("fn PascalCase") {
                quality_score -= 15.0;
                issues.push(json!({
                    "type": "naming_convention",
                    "severity": "warning",
                    "message": "Rust 函数应使用 snake_case 命名"
                }));
                suggestions.push("将函数名改为 snake_case 格式");
            }
        }
    }

    // 检查代码风格
    if check_style {
        // 检查缩进一致性
        let has_tabs = code.contains('\t');
        let has_spaces = code.contains("    ");
        if has_tabs && has_spaces {
            quality_score -= 10.0;
            issues.push(json!({
                "type": "mixed_indentation",
                "severity": "warning",
                "message": "混合使用 Tab 和空格缩进"
            }));
            suggestions.push("统一使用空格或 Tab 进行缩进");
        }
    }

    // 确保评分不低于 0
    quality_score = quality_score.max(0.0);

    Ok(json!({
        "success": true,
        "language": language,
        "quality_score": quality_score,
        "metrics": {
            "total_lines": total_lines,
            "code_lines": code_lines,
            "comment_lines": comment_lines,
            "blank_lines": blank_lines,
            "comment_ratio": format!("{:.1}%", comment_ratio)
        },
        "issues": issues,
        "suggestions": suggestions,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 代码复杂度分析工具
///
/// 分析代码的圈复杂度、认知复杂度等指标。
///
/// # 参数
///
/// - `code` (必需): 要分析的代码字符串
/// - `language` (必需): 编程语言
/// - `threshold` (可选): 复杂度阈值，默认 10
///
/// # 返回值
///
/// 返回一个 JSON 对象，包含以下字段：
/// - `success` (boolean): 分析是否成功
/// - `cyclomatic_complexity` (number): 圈复杂度
/// - `cognitive_complexity` (number): 认知复杂度
/// - `functions` (array): 函数复杂度列表
/// - `warnings` (array): 复杂度警告列表
///
/// # 示例
///
/// ```rust
/// let params = json!({
///     "code": "fn complex() { if x { if y { ... } } }",
///     "language": "rust",
///     "threshold": 10
/// });
/// ```
#[tool(
    name = "code_complexity",
    description = "分析代码复杂度（圈复杂度、认知复杂度）"
)]
async fn code_complexity(
    code: String,
    language: String,
    threshold: Option<i64>,
) -> Result<Value> {
    let threshold = threshold.unwrap_or(10);

    // 简化的复杂度计算
    let mut cyclomatic_complexity = 1; // 基础复杂度
    let mut cognitive_complexity = 0;
    let mut warnings = Vec::new();
    let mut functions = Vec::new();

    // 计算控制流语句数量
    let control_keywords = vec!["if", "else", "for", "while", "match", "case", "switch", "catch"];
    for keyword in control_keywords {
        let count = code.matches(keyword).count();
        cyclomatic_complexity += count;
        cognitive_complexity += count;
    }

    // 计算逻辑运算符数量
    let logical_ops = vec!["&&", "||"];
    for op in logical_ops {
        let count = code.matches(op).count();
        cyclomatic_complexity += count;
        cognitive_complexity += count;
    }

    // 检查嵌套深度 (简化版)
    let max_nesting = code.matches('{').count().min(code.matches('}').count());
    if max_nesting > 4 {
        cognitive_complexity += (max_nesting - 4) * 2;
        warnings.push(json!({
            "type": "deep_nesting",
            "severity": "warning",
            "message": format!("嵌套深度过深: {} 层", max_nesting)
        }));
    }

    // 分析函数 (简化版)
    let function_pattern = if language == "rust" { "fn " } else { "function " };
    let function_count = code.matches(function_pattern).count();
    
    for i in 0..function_count {
        let func_complexity = cyclomatic_complexity / function_count.max(1);
        functions.push(json!({
            "name": format!("function_{}", i + 1),
            "complexity": func_complexity,
            "status": if func_complexity > threshold as usize { "high" } else { "ok" }
        }));
    }

    // 复杂度警告
    if cyclomatic_complexity > threshold as usize {
        warnings.push(json!({
            "type": "high_complexity",
            "severity": "error",
            "message": format!("圈复杂度过高: {} (阈值: {})", cyclomatic_complexity, threshold)
        }));
    }

    Ok(json!({
        "success": true,
        "language": language,
        "cyclomatic_complexity": cyclomatic_complexity,
        "cognitive_complexity": cognitive_complexity,
        "max_nesting_depth": max_nesting,
        "function_count": function_count,
        "functions": functions,
        "warnings": warnings,
        "threshold": threshold,
        "status": if cyclomatic_complexity > threshold as usize { "high" } else { "ok" },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 安全扫描工具
///
/// 扫描代码中的安全漏洞和潜在风险。
///
/// # 参数
///
/// - `code` (必需): 要扫描的代码字符串
/// - `language` (必需): 编程语言
/// - `scan_level` (可选): 扫描级别 (basic, standard, strict)，默认 standard
///
/// # 返回值
///
/// 返回一个 JSON 对象，包含以下字段：
/// - `success` (boolean): 扫描是否成功
/// - `vulnerabilities` (array): 发现的漏洞列表
/// - `risk_score` (number): 风险评分 (0-100)
/// - `recommendations` (array): 安全建议列表
///
/// # 示例
///
/// ```rust
/// let params = json!({
///     "code": "let password = \"hardcoded\";",
///     "language": "rust",
///     "scan_level": "strict"
/// });
/// ```
#[tool(
    name = "security_scan",
    description = "扫描代码安全漏洞和潜在风险"
)]
async fn security_scan(
    code: String,
    language: String,
    scan_level: Option<String>,
) -> Result<Value> {
    let scan_level = scan_level.unwrap_or_else(|| "standard".to_string());

    let mut vulnerabilities = Vec::new();
    let mut risk_score = 0;
    let mut recommendations: Vec<String> = Vec::new();

    // 检查硬编码密码
    let password_patterns = vec!["password =", "passwd =", "pwd =", "secret =", "api_key ="];
    for pattern in password_patterns {
        if code.to_lowercase().contains(pattern) {
            risk_score += 30;
            vulnerabilities.push(json!({
                "type": "hardcoded_credentials",
                "severity": "high",
                "message": "发现硬编码的密码或密钥",
                "line": "N/A"
            }));
            recommendations.push("使用环境变量或密钥管理服务存储敏感信息".to_string());
            break;
        }
    }

    // 检查 SQL 注入风险
    if code.contains("SELECT") && (code.contains("+") || code.contains("format!")) {
        risk_score += 25;
        vulnerabilities.push(json!({
            "type": "sql_injection",
            "severity": "high",
            "message": "可能存在 SQL 注入风险",
            "line": "N/A"
        }));
        recommendations.push("使用参数化查询或 ORM 框架".to_string());
    }

    // 检查不安全的函数调用
    let unsafe_functions = vec!["eval", "exec", "system", "unsafe"];
    for func in unsafe_functions {
        if code.contains(func) {
            risk_score += 20;
            vulnerabilities.push(json!({
                "type": "unsafe_function",
                "severity": "medium",
                "message": format!("使用了不安全的函数: {}", func),
                "line": "N/A"
            }));
            let recommendation = format!("避免使用 {} 函数，寻找更安全的替代方案", func);
            recommendations.push(recommendation);
        }
    }

    // 检查未处理的错误
    if language == "rust" && code.contains("unwrap()") {
        risk_score += 10;
        vulnerabilities.push(json!({
            "type": "error_handling",
            "severity": "low",
            "message": "使用 unwrap() 可能导致 panic",
            "line": "N/A"
        }));
        recommendations.push("使用 ? 操作符或 match 进行错误处理".to_string());
    }

    // 确保风险评分不超过 100
    risk_score = risk_score.min(100);

    Ok(json!({
        "success": true,
        "language": language,
        "scan_level": scan_level,
        "risk_score": risk_score,
        "vulnerability_count": vulnerabilities.len(),
        "vulnerabilities": vulnerabilities,
        "recommendations": recommendations,
        "status": if risk_score > 50 { "high_risk" } else if risk_score > 20 { "medium_risk" } else { "low_risk" },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 获取所有代码分析工具
pub fn get_all_code_analysis_tools() -> Vec<Box<dyn crate::tool::Tool>> {
    vec![
        code_quality_tool(),
        code_complexity_tool(),
        security_scan_tool(),
    ]
}

