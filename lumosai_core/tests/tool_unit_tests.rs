//! Tool 单元测试
//!
//! P0-3 任务：增加 Tool 模块的单元测试覆盖率
//! 测试范围：
//! - Tool 创建和配置
//! - Tool 参数验证
//! - Tool 执行
//! - Tool Schema 管理
//! - Tool Builder

use lumosai_core::tool::{
    GenericTool, ParameterSchema, Tool, ToolBuilder, ToolExecutionContext, ToolExecutionOptions,
    ToolSchema,
};
use serde_json::{json, Value};

/// 测试 1: GenericTool 基本创建
#[tokio::test]
async fn test_generic_tool_creation() {
    let schema = ToolSchema::new(vec![ParameterSchema {
        name: "input".to_string(),
        description: "Test input".to_string(),
        r#type: "string".to_string(),
        required: true,
        properties: None,
        default: None,
    }]);

    let tool = GenericTool::new("test_tool", "A test tool", schema, |params, _ctx| {
        Ok(params)
    });

    assert_eq!(tool.id(), "test_tool");
    assert_eq!(tool.description(), "A test tool");
}

/// 测试 2: Tool 执行成功
#[tokio::test]
async fn test_tool_execution_success() {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "a".to_string(),
            description: "First number".to_string(),
            r#type: "number".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "b".to_string(),
            description: "Second number".to_string(),
            r#type: "number".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);

    let tool = GenericTool::new("add", "Add two numbers", schema, |params, _ctx| {
        let a = params["a"].as_f64().unwrap_or(0.0);
        let b = params["b"].as_f64().unwrap_or(0.0);
        Ok(json!(a + b))
    });

    let context = ToolExecutionContext::new();
    let options = ToolExecutionOptions::default();
    let params = json!({"a": 5, "b": 3});

    let result = tool.execute(params, context, &options).await.unwrap();
    assert_eq!(result, json!(8.0));
}

/// 测试 3: Tool 执行错误处理
#[tokio::test]
async fn test_tool_execution_error() {
    let schema = ToolSchema::new(vec![ParameterSchema {
        name: "divisor".to_string(),
        description: "Divisor".to_string(),
        r#type: "number".to_string(),
        required: true,
        properties: None,
        default: None,
    }]);

    let tool = GenericTool::new("divide", "Divide by number", schema, |params, _ctx| {
        let divisor = params["divisor"].as_f64().unwrap_or(0.0);
        if divisor == 0.0 {
            return Err(lumosai_core::error::Error::Tool(
                "Cannot divide by zero".to_string(),
            ));
        }
        Ok(json!(10.0 / divisor))
    });

    let context = ToolExecutionContext::new();
    let options = ToolExecutionOptions::default();
    let params = json!({"divisor": 0});

    let result = tool.execute(params, context, &options).await;
    assert!(result.is_err());
}

/// 测试 4: ParameterSchema 创建
#[test]
fn test_parameter_schema_creation() {
    let param = ParameterSchema {
        name: "test_param".to_string(),
        description: "A test parameter".to_string(),
        r#type: "string".to_string(),
        required: true,
        properties: None,
        default: None,
    };

    assert_eq!(param.name, "test_param");
    assert_eq!(param.description, "A test parameter");
    assert_eq!(param.r#type, "string");
    assert!(param.required);
}

/// 测试 5: ParameterSchema 带默认值
#[test]
fn test_parameter_schema_with_default() {
    let param = ParameterSchema {
        name: "optional_param".to_string(),
        description: "An optional parameter".to_string(),
        r#type: "number".to_string(),
        required: false,
        properties: None,
        default: Some(json!(42)),
    };

    assert!(!param.required);
    assert_eq!(param.default, Some(json!(42)));
}

/// 测试 6: ToolSchema 创建
#[test]
fn test_tool_schema_creation() {
    let params = vec![
        ParameterSchema {
            name: "param1".to_string(),
            description: "First param".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "param2".to_string(),
            description: "Second param".to_string(),
            r#type: "number".to_string(),
            required: false,
            properties: None,
            default: Some(json!(0)),
        },
    ];

    let schema = ToolSchema::new(params);
    // Schema 创建成功即可
    let _ = schema;
}

/// 测试 7: ToolExecutionContext 创建
#[test]
fn test_tool_execution_context_creation() {
    let context = ToolExecutionContext::new();
    // Context 创建成功即可
    let _ = context;
}

/// 测试 8: ToolExecutionOptions 默认值
#[test]
fn test_tool_execution_options_default() {
    let options = ToolExecutionOptions::default();
    // Options 创建成功即可
    let _ = options;
}

/// 测试 9: ToolExecutionOptions 带验证
#[test]
fn test_tool_execution_options_with_validation() {
    let options = ToolExecutionOptions::with_validation();
    assert!(options.validate_params);
    assert!(options.validate_output);
}

/// 测试 10: Tool 克隆
#[test]
fn test_tool_clone() {
    let schema = ToolSchema::new(vec![]);
    let tool = GenericTool::new("clone_test", "Test cloning", schema, |params, _ctx| {
        Ok(params)
    });

    let cloned = tool.clone();
    assert_eq!(cloned.id(), tool.id());
    assert_eq!(cloned.description(), tool.description());
}

/// 测试 11: Tool 带输出 Schema
#[tokio::test]
async fn test_tool_with_output_schema() {
    let schema = ToolSchema::new(vec![]);
    let output_schema = json!({
        "type": "object",
        "properties": {
            "result": {"type": "string"}
        }
    });

    let tool = GenericTool::new(
        "output_test",
        "Test output schema",
        schema,
        |_params, _ctx| Ok(json!({"result": "success"})),
    )
    .with_output_schema(output_schema.clone());

    assert_eq!(tool.output_schema(), Some(output_schema));
}

/// 测试 12: ToolBuilder 基本使用
#[test]
fn test_tool_builder_basic() {
    let tool = ToolBuilder::new()
        .name("builder_test")
        .description("Test tool builder")
        .parameter("input", "string", "Test input", true)
        .handler(|params| {
            let input = params.get("input").and_then(|v| v.as_str()).unwrap_or("");
            Ok(json!({"output": input}))
        })
        .build()
        .expect("Failed to build tool");

    assert_eq!(tool.id(), "builder_test");
    assert_eq!(tool.description(), "Test tool builder");
}

/// 测试 13: ToolBuilder 多个参数
#[test]
fn test_tool_builder_multiple_parameters() {
    let tool = ToolBuilder::new()
        .name("multi_param")
        .description("Tool with multiple parameters")
        .parameter("param1", "string", "First parameter", true)
        .parameter("param2", "number", "Second parameter", false)
        .parameter("param3", "boolean", "Third parameter", false)
        .handler(|_params| Ok(json!({"status": "ok"})))
        .build()
        .expect("Failed to build tool");

    assert_eq!(tool.id(), "multi_param");
}

/// 测试 14: Tool 不同数据类型
#[tokio::test]
async fn test_tool_different_data_types() {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "string_val".to_string(),
            description: "String value".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "number_val".to_string(),
            description: "Number value".to_string(),
            r#type: "number".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "bool_val".to_string(),
            description: "Boolean value".to_string(),
            r#type: "boolean".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);

    let tool = GenericTool::new(
        "type_test",
        "Test different types",
        schema,
        |params, _ctx| {
            Ok(json!({
                "string": params["string_val"],
                "number": params["number_val"],
                "boolean": params["bool_val"]
            }))
        },
    );

    let context = ToolExecutionContext::new();
    let options = ToolExecutionOptions::default();
    let params = json!({
        "string_val": "test",
        "number_val": 42,
        "bool_val": true
    });

    let result = tool.execute(params, context, &options).await.unwrap();
    assert_eq!(result["string"], json!("test"));
    assert_eq!(result["number"], json!(42));
    assert_eq!(result["boolean"], json!(true));
}

/// 测试 15: Tool 空参数
#[tokio::test]
async fn test_tool_no_parameters() {
    let schema = ToolSchema::new(vec![]);
    let tool = GenericTool::new(
        "no_params",
        "Tool with no parameters",
        schema,
        |_params, _ctx| Ok(json!({"message": "Hello, World!"})),
    );

    let context = ToolExecutionContext::new();
    let options = ToolExecutionOptions::default();
    let params = json!({});

    let result = tool.execute(params, context, &options).await.unwrap();
    assert_eq!(result["message"], json!("Hello, World!"));
}
