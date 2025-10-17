//! 内置工具演示
//! 
//! 展示如何使用 LumosAI 的内置工具集

use lumosai_core::{
    error::Result,
    tool::{
        builtin::{create_calculator_tool, create_datetime_tool, create_json_parser_tool, create_data_validator_tool},
        Tool, ToolExecutionContext, ToolExecutionOptions,
    },
};
use serde_json::{json, Value};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== LumosAI 内置工具演示 ===\n");

    // 创建工具执行上下文
    let context = ToolExecutionContext {
        thread_id: Some("demo_thread".to_string()),
        resource_id: Some("demo_resource".to_string()),
        run_id: Some("demo_run".to_string()),
        tool_call_id: Some("demo_call".to_string()),
        messages: None,
        abort_signal: None,
    };

    // 创建工具执行选项
    let options = ToolExecutionOptions {
        context: None,
        validate_params: true,
        validate_output: false,
    };

    // 1. 演示计算器工具
    println!("1. 计算器工具演示");
    println!("==================");
    
    let calculator = create_calculator_tool();
    println!("工具ID: {}", calculator.id());
    println!("工具描述: {}", calculator.description());
    
    // 基本数学计算
    let params = json!({
        "expression": "2 + 3 * 4",
        "precision": 2
    });
    
    let result = calculator.execute(params, context.clone(), &options).await?;
    println!("计算结果: {}", result);
    
    // 复杂数学表达式
    let params = json!({
        "expression": "sqrt(16) + pow(2, 3)",
        "precision": 4,
        "format": "decimal"
    });
    
    let result = calculator.execute(params, context.clone(), &options).await?;
    println!("复杂计算结果: {}\n", result);

    // 2. 演示时间工具
    println!("2. 时间工具演示");
    println!("================");
    
    let datetime_tool = create_datetime_tool();
    println!("工具ID: {}", datetime_tool.id());
    println!("工具描述: {}", datetime_tool.description());
    
    // 获取当前时间
    let params = json!({
        "operation": "now"
    });

    let result = datetime_tool.execute(params, context.clone(), &options).await?;
    println!("当前时间: {}", result);

    // 时间格式化
    let params = json!({
        "operation": "format",
        "input": "2024-01-15T10:30:00Z",
        "format": "%Y年%m月%d日 %H:%M:%S"
    });

    let result = datetime_tool.execute(params, context.clone(), &options).await?;
    println!("格式化时间: {}\n", result);

    // 3. 演示JSON解析工具
    println!("3. JSON解析工具演示");
    println!("===================");
    
    let json_parser = create_json_parser_tool();
    println!("工具ID: {}", json_parser.id());
    println!("工具描述: {}", json_parser.description());
    
    // JSON解析
    let sample_json = r#"{"name": "张三", "age": 30, "skills": ["Rust", "Python", "JavaScript"]}"#;
    let params = json!({
        "json_string": sample_json
    });

    let result = json_parser.execute(params, context.clone(), &options).await?;
    println!("JSON解析结果: {}", result);

    // JSON路径提取
    let params = json!({
        "json_string": sample_json,
        "path": "$.skills"
    });
    
    let result = json_parser.execute(params, context.clone(), &options).await?;
    println!("JSON查询结果: {}\n", result);

    // 4. 演示数据验证工具
    println!("4. 数据验证工具演示");
    println!("===================");
    
    let data_validator = create_data_validator_tool();
    println!("工具ID: {}", data_validator.id());
    println!("工具描述: {}", data_validator.description());
    
    // 邮箱验证
    let params = json!({
        "data": "user@example.com",
        "validation_type": "email"
    });
    
    let result = data_validator.execute(params, context.clone(), &options).await?;
    println!("邮箱验证结果: {}", result);
    
    // URL验证
    let params = json!({
        "data": "https://www.example.com/path?param=value",
        "validation_type": "url"
    });
    
    let result = data_validator.execute(params, context.clone(), &options).await?;
    println!("URL验证结果: {}", result);
    
    // JSON Schema验证
    let params = json!({
        "data": r#"{"name": "张三", "age": 30}"#,
        "validation_type": "json_schema",
        "schema": {
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number", "minimum": 0}
            },
            "required": ["name", "age"]
        }
    });
    
    let result = data_validator.execute(params, context.clone(), &options).await?;
    println!("JSON Schema验证结果: {}\n", result);

    // 5. 演示工具组合使用
    println!("5. 工具组合使用演示");
    println!("===================");
    
    // 先用计算器计算一个值
    let calc_params = json!({
        "expression": "100 * 1.08",
        "precision": 2
    });
    
    let calc_result = calculator.execute(calc_params, context.clone(), &options).await?;
    println!("计算结果: {}", calc_result);
    
    // 然后用JSON工具构造数据
    let json_params = json!({
        "operation": "create",
        "data": {
            "product": "商品A",
            "price": calc_result,
            "timestamp": "2024-01-15T10:30:00Z"
        }
    });
    
    let json_result = json_parser.execute(json_params, context.clone(), &options).await?;
    println!("JSON构造结果: {}", json_result);
    
    // 最后验证构造的数据
    let validation_params = json!({
        "data": json_result.to_string(),
        "validation_type": "json"
    });
    
    let validation_result = data_validator.execute(validation_params, context.clone(), &options).await?;
    println!("数据验证结果: {}", validation_result);

    println!("\n=== 演示完成 ===");
    println!("所有内置工具都运行正常！");
    
    Ok(())
}
