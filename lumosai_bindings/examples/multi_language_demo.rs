//! 多语言绑定演示
//!
//! 展示Lumos.ai多语言绑定的完整功能

use lumosai_bindings::core::*;
use lumosai_bindings::error::*;
use lumosai_bindings::types::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌍 Lumos.ai 多语言绑定演示");
    println!("=".repeat(50));

    // 演示核心绑定功能
    demo_core_bindings().await?;

    // 演示工具系统
    demo_tool_system().await?;

    // 演示错误处理
    demo_error_handling().await?;

    // 演示类型转换
    demo_type_conversion().await?;

    // 演示配置管理
    demo_configuration().await?;

    println!("\n🎉 多语言绑定演示完成！");
    println!("\n📚 支持的语言:");
    println!("  🐍 Python: pip install lumosai");
    println!("  📦 Node.js: npm install @lumosai/core");
    println!("  🌐 WebAssembly: 浏览器直接使用");
    println!("  🔧 C/Go: 通过C绑定集成");

    Ok(())
}

/// 演示核心绑定功能
async fn demo_core_bindings() -> Result<()> {
    println!("\n🔧 演示：核心绑定功能");
    println!("-".repeat(30));

    // 创建Agent构建器
    println!("📝 创建Agent构建器...");
    let builder = CrossLangAgentBuilder::new()
        .name("multi_lang_demo")
        .instructions("你是一个多语言绑定演示助手，可以帮助用户了解Lumos.ai的各种功能")
        .model("demo_model");

    // 构建Agent
    println!("🏗️  构建Agent...");
    let agent = builder.build()?;

    // 获取配置
    let config = agent.get_config();
    println!("⚙️  Agent配置:");
    println!("   模型: {}", config.model.name);
    println!("   超时: {}秒", config.runtime.timeout_seconds);
    println!("   重试: {}次", config.runtime.max_retries);
    println!("   并发: {}", config.runtime.concurrency_limit);

    // 生成响应
    println!("\n💬 生成响应...");
    let test_inputs = vec![
        "Hello, world!",
        "你好，世界！",
        "Hola, mundo!",
        "Bonjour, le monde!",
    ];

    for input in test_inputs {
        println!("   输入: {}", input);
        match agent.generate(input) {
            Ok(response) => {
                println!("   输出: {}", response.content);
                println!("   类型: {:?}", response.response_type);
                if let Some(error) = &response.error {
                    println!("   错误: {}", error);
                }
            }
            Err(e) => {
                println!("   错误: {}", e);
            }
        }
        println!();
    }

    // 异步生成
    println!("🔄 异步生成响应...");
    let async_response = agent.generate_async("这是一个异步请求").await?;
    println!("   异步响应: {}", async_response.content);

    Ok(())
}

/// 演示工具系统
async fn demo_tool_system() -> Result<()> {
    println!("\n🛠️  演示：工具系统");
    println!("-".repeat(30));

    // 创建各种工具
    let tools = vec![
        ("计算器", create_calculator_tool()),
        ("文本处理", create_text_processor_tool()),
        ("数据转换", create_data_converter_tool()),
    ];

    for (name, tool) in tools {
        println!("🔧 工具: {}", name);

        // 获取工具元数据
        let metadata = tool.metadata();
        println!("   名称: {}", metadata.name);
        println!("   描述: {}", metadata.description);
        println!("   类型: {}", metadata.tool_type);
        println!("   异步: {}", metadata.is_async);

        // 执行工具
        let test_params = match metadata.name.as_str() {
            "calculator" => serde_json::json!({"expression": "2 + 2 * 3"}),
            "text_processor" => {
                serde_json::json!({"text": "Hello World", "operation": "uppercase"})
            }
            "data_converter" => serde_json::json!({"data": "[1,2,3]", "format": "csv"}),
            _ => serde_json::json!({}),
        };

        match tool.execute(test_params) {
            Ok(result) => {
                println!("   执行结果:");
                println!("     成功: {}", result.success);
                println!("     耗时: {}ms", result.execution_time_ms);
                println!("     结果: {}", result.result);
                if let Some(error) = &result.error {
                    println!("     错误: {}", error);
                }
            }
            Err(e) => {
                println!("   执行失败: {}", e);
            }
        }
        println!();
    }

    Ok(())
}

/// 演示错误处理
async fn demo_error_handling() -> Result<()> {
    println!("\n❌ 演示：错误处理");
    println!("-".repeat(30));

    // 测试各种错误类型
    let errors = vec![
        BindingError::core("核心模块错误"),
        BindingError::network("网络连接失败"),
        BindingError::timeout(30),
        BindingError::configuration("model", "无效的模型名称"),
        BindingError::tool("calculator", "除零错误"),
        BindingError::serialization("JSON解析失败"),
    ];

    for error in errors {
        println!("🚨 错误类型: {}", error.error_code());
        println!("   消息: {}", error);
        println!("   可重试: {}", error.is_retryable());

        let context = error.context();
        println!("   分类: {:?}", context.category);
        println!("   严重程度: {:?}", context.severity);
        println!(
            "   建议: {:?}",
            context.suggestions.get(0).unwrap_or(&"无".to_string())
        );

        if let Some(doc_url) = &context.documentation_url {
            println!("   文档: {}", doc_url);
        }
        println!();
    }

    Ok(())
}

/// 演示类型转换
async fn demo_type_conversion() -> Result<()> {
    println!("\n🔄 演示：类型转换");
    println!("-".repeat(30));

    use lumosai_bindings::types::conversion::*;

    // 测试各种数据类型
    let test_values = vec![
        ("字符串", serde_json::json!("Hello, World!")),
        ("数字", serde_json::json!(42.5)),
        ("布尔值", serde_json::json!(true)),
        ("数组", serde_json::json!([1, 2, 3, 4, 5])),
        (
            "对象",
            serde_json::json!({
                "name": "Lumos.ai",
                "version": "0.1.0",
                "features": ["multi-language", "high-performance"]
            }),
        ),
    ];

    for (type_name, value) in test_values {
        println!("📊 类型: {}", type_name);
        println!("   原始值: {}", value);

        // 转换为跨语言值
        let cross_lang_value = to_cross_lang_value(&value);
        println!("   跨语言值: {}", cross_lang_value);

        // 转换回原始值
        let back_value = from_cross_lang_value(&cross_lang_value);
        println!("   转换回值: {}", back_value);

        // 验证一致性
        let is_consistent = value == back_value;
        println!("   一致性: {}", if is_consistent { "✅" } else { "❌" });
        println!();
    }

    Ok(())
}

/// 演示配置管理
async fn demo_configuration() -> Result<()> {
    println!("\n⚙️  演示：配置管理");
    println!("-".repeat(30));

    // 创建各种配置
    let configs = vec![
        ("性能配置", create_performance_config()),
        ("安全配置", create_security_config()),
        ("日志配置", create_logging_config()),
    ];

    for (name, config) in configs {
        println!("📋 配置类型: {}", name);

        match name {
            "性能配置" => {
                if let ConfigOptions { performance, .. } = config {
                    println!("   超时时间: {}秒", performance.timeout_seconds);
                    println!("   启用缓存: {}", performance.enable_cache);
                    println!("   缓存大小: {:?}", performance.cache_size);
                    println!(
                        "   内存限制: {:?}MB",
                        performance.memory_limit_bytes.map(|b| b / 1024 / 1024)
                    );
                }
            }
            "安全配置" => {
                if let ConfigOptions { security, .. } = config {
                    println!("   启用沙箱: {}", security.enable_sandbox);
                    println!("   需要API密钥: {}", security.require_api_key);
                    println!("   允许域名数: {}", security.allowed_domains.len());
                    println!("   禁止操作数: {}", security.forbidden_operations.len());
                }
            }
            "日志配置" => {
                if let ConfigOptions { logging, .. } = config {
                    println!("   日志级别: {:?}", logging.level);
                    println!("   日志格式: {:?}", logging.format);
                    println!("   输出目标: {:?}", logging.output);
                    println!("   结构化: {}", logging.structured);
                }
            }
            _ => {}
        }
        println!();
    }

    Ok(())
}

/// 创建计算器工具
fn create_calculator_tool() -> CrossLangTool {
    let tool = lumosai_core::tools::math::calculator();
    let metadata = ToolMetadata {
        name: "calculator".to_string(),
        description: "基础数学计算工具".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "数学表达式"
                }
            },
            "required": ["expression"]
        }),
        tool_type: "math".to_string(),
        is_async: false,
    };

    CrossLangTool::new(tool, metadata)
}

/// 创建文本处理工具
fn create_text_processor_tool() -> CrossLangTool {
    let tool = lumosai_core::tools::text::text_processor();
    let metadata = ToolMetadata {
        name: "text_processor".to_string(),
        description: "文本处理工具".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "输入文本"
                },
                "operation": {
                    "type": "string",
                    "description": "操作类型",
                    "enum": ["uppercase", "lowercase", "reverse", "length"]
                }
            },
            "required": ["text", "operation"]
        }),
        tool_type: "text".to_string(),
        is_async: false,
    };

    CrossLangTool::new(tool, metadata)
}

/// 创建数据转换工具
fn create_data_converter_tool() -> CrossLangTool {
    let tool = lumosai_core::tools::data::data_converter();
    let metadata = ToolMetadata {
        name: "data_converter".to_string(),
        description: "数据格式转换工具".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "data": {
                    "type": "string",
                    "description": "输入数据"
                },
                "format": {
                    "type": "string",
                    "description": "目标格式",
                    "enum": ["json", "csv", "xml", "yaml"]
                }
            },
            "required": ["data", "format"]
        }),
        tool_type: "data".to_string(),
        is_async: false,
    };

    CrossLangTool::new(tool, metadata)
}

/// 创建性能配置
fn create_performance_config() -> ConfigOptions {
    ConfigOptions {
        language_config: LanguageConfig {
            target_language: TargetLanguage::Python,
            language_version: "3.8+".to_string(),
            language_options: HashMap::new(),
        },
        performance: PerformanceConfig {
            thread_pool_size: Some(4),
            memory_limit_bytes: Some(512 * 1024 * 1024), // 512MB
            timeout_seconds: 60,
            enable_cache: true,
            cache_size: Some(2000),
        },
        security: SecurityConfig::default(),
        logging: LoggingConfig::default(),
    }
}

/// 创建安全配置
fn create_security_config() -> ConfigOptions {
    ConfigOptions {
        language_config: LanguageConfig {
            target_language: TargetLanguage::JavaScript,
            language_version: "ES2022".to_string(),
            language_options: HashMap::new(),
        },
        performance: PerformanceConfig::default(),
        security: SecurityConfig {
            enable_sandbox: true,
            allowed_domains: vec![
                "api.openai.com".to_string(),
                "api.anthropic.com".to_string(),
                "api.deepseek.com".to_string(),
            ],
            forbidden_operations: vec!["file_delete".to_string(), "system_shutdown".to_string()],
            require_api_key: true,
        },
        logging: LoggingConfig::default(),
    }
}

/// 创建日志配置
fn create_logging_config() -> ConfigOptions {
    ConfigOptions {
        language_config: LanguageConfig {
            target_language: TargetLanguage::Go,
            language_version: "1.21+".to_string(),
            language_options: HashMap::new(),
        },
        performance: PerformanceConfig::default(),
        security: SecurityConfig::default(),
        logging: LoggingConfig {
            level: LogLevel::Debug,
            format: LogFormat::Json,
            output: LogOutput::File {
                path: "/var/log/lumosai.log".to_string(),
            },
            structured: true,
        },
    }
}
