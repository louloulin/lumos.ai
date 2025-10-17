//! 友好错误处理演示
//!
//! 展示 LumosAI 的友好错误处理系统，包括：
//! - 用户友好的错误消息
//! - 上下文信息和修复建议
//! - 错误分类和严重性级别
//! - 结构化错误输出

use lumosai_core::error::{Error, friendly::{FriendlyError, helpers}};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 LumosAI 友好错误处理演示\n");

    // 1. 配置错误演示
    println!("1️⃣ 配置错误演示:");
    let config_error = helpers::config_error(
        "缺少必需的 API 密钥 'openai_api_key'",
        Some("/path/to/config.toml")
    );
    println!("{}\n", config_error);

    // 2. 工具错误演示
    println!("2️⃣ 工具错误演示:");
    let tool_params = json!({
        "operation": "divide",
        "a": 10,
        "b": 0
    });
    let tool_error = helpers::tool_error(
        "calculator",
        "除零错误：不能除以零",
        Some(&tool_params)
    );
    println!("{}\n", tool_error);

    // 3. Agent 错误演示
    println!("3️⃣ Agent 错误演示:");
    let agent_error = helpers::agent_error(
        "data_analyst",
        "无法访问所需的数据源"
    );
    println!("{}\n", agent_error);

    // 4. 网络错误演示
    println!("4️⃣ 网络错误演示:");
    let network_error = helpers::network_error(
        "https://api.openai.com/v1/chat/completions",
        Some(429)
    );
    println!("{}\n", network_error);

    // 5. 自定义友好错误演示
    println!("5️⃣ 自定义友好错误演示:");
    let mut custom_error = FriendlyError::new(
        Error::Memory("向量数据库连接失败".to_string()),
        "内存系统无法初始化".to_string(),
    )
    .with_context("database_type", "Qdrant")
    .with_context("endpoint", "http://localhost:6333")
    .with_context("collection", "embeddings")
    .with_suggestion("检查 Qdrant 服务是否正在运行")
    .with_suggestion("验证连接配置是否正确")
    .with_suggestion("尝试使用 'docker run -p 6333:6333 qdrant/qdrant' 启动 Qdrant");

    custom_error.generate_suggestions();
    println!("{}\n", custom_error);

    // 6. 错误代码演示
    println!("6️⃣ 错误代码和分类演示:");
    let errors = vec![
        helpers::config_error("配置文件格式错误", None),
        helpers::tool_error("weather", "API 限额已用完", None),
        helpers::agent_error("assistant", "模型响应超时"),
        helpers::network_error("https://api.example.com", Some(500)),
    ];

    for (i, error) in errors.iter().enumerate() {
        println!("错误 {}: 代码={}, 分类={:?}, 严重性={:?}",
            i + 1,
            error.error_code(),
            error.category,
            error.severity
        );
    }

    println!("\n✅ 友好错误处理演示完成！");
    println!("💡 这些错误消息提供了清晰的问题描述、上下文信息和具体的修复建议。");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumosai_core::error::friendly::{ErrorCategory, ErrorSeverity};

    #[test]
    fn test_config_error_properties() {
        let error = helpers::config_error("测试错误", Some("/test/config"));
        assert_eq!(error.category, ErrorCategory::Configuration);
        assert_eq!(error.severity, ErrorSeverity::Medium);
        assert!(error.context.contains_key("config_file"));
        assert!(!error.suggestions.is_empty());
    }

    #[test]
    fn test_tool_error_with_params() {
        let params = json!({"param": "value"});
        let error = helpers::tool_error("test_tool", "测试错误", Some(&params));
        assert_eq!(error.category, ErrorCategory::Tool);
        assert!(error.context.contains_key("tool_name"));
        assert!(error.context.contains_key("parameters"));
    }

    #[test]
    fn test_network_error_with_status() {
        let error = helpers::network_error("http://test.com", Some(404));
        assert_eq!(error.category, ErrorCategory::Network);
        assert!(error.context.contains_key("endpoint"));
        assert!(error.context.contains_key("status_code"));
    }

    #[test]
    fn test_custom_error_building() {
        let error = FriendlyError::new(
            Error::Agent("测试".to_string()),
            "测试消息".to_string(),
        )
        .with_context("key1", "value1")
        .with_context("key2", 42)
        .with_suggestion("建议1")
        .with_suggestion("建议2");

        assert_eq!(error.context.len(), 2);
        assert_eq!(error.suggestions.len(), 2);
        assert_eq!(error.category, ErrorCategory::Agent);
    }

    #[test]
    fn test_error_code_generation() {
        let error = helpers::config_error("测试", None);
        assert_eq!(error.error_code(), "CONFIGURATION_MEDIUM");

        let error = helpers::tool_error("test", "测试", None);
        assert_eq!(error.error_code(), "TOOL_LOW");
    }

    #[test]
    fn test_error_display_formatting() {
        let error = FriendlyError::new(
            Error::Validation {
                field: "email".to_string(),
                message: "格式无效".to_string(),
            },
            "验证失败".to_string(),
        )
        .with_context("input", "invalid-email")
        .with_suggestion("请提供有效的邮箱地址");

        let formatted = error.format_for_display();
        assert!(formatted.contains("⚠️")); // Low severity emoji
        assert!(formatted.contains("Context:"));
        assert!(formatted.contains("Suggestions:"));
        assert!(formatted.contains("Technical Details:"));
    }
}
