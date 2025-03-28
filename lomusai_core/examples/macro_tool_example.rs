use std::collections::HashMap;

use lomusai_core::{Error, Result, tool::Tool, tool::ToolSchema, tool::ParameterSchema, tool::ToolExecutionOptions};
use serde_json::{Value, json};
use async_trait::async_trait;

// 在macros特性启用时使用宏定义工具
#[cfg(feature = "macros")]
use lumos_macro::tool;

// 对于未启用宏的情况，提供传统实现
#[cfg(not(feature = "macros"))]
struct CalculatorTool;

#[cfg(not(feature = "macros"))]
#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }
    
    fn description(&self) -> &str {
        "执行基本的数学运算"
    }
    
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            parameters: vec![
                ParameterSchema {
                    name: "operation".to_string(),
                    description: "要执行的操作: add, subtract, multiply, divide".to_string(),
                    r#type: "string".to_string(),
                    required: true,
                    properties: None,
                    default: None,
                },
                ParameterSchema {
                    name: "a".to_string(),
                    description: "第一个数字".to_string(),
                    r#type: "number".to_string(),
                    required: true,
                    properties: None,
                    default: None,
                },
                ParameterSchema {
                    name: "b".to_string(),
                    description: "第二个数字".to_string(),
                    r#type: "number".to_string(),
                    required: true,
                    properties: None,
                    default: None,
                },
            ],
        }
    }
    
    async fn execute(&self, params: HashMap<String, Value>, _options: &ToolExecutionOptions) -> Result<Value> {
        let operation = params.get("operation").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::InvalidInput("operation参数缺失或类型错误".to_string())
        })?;
        
        let a = params.get("a").and_then(|v| v.as_f64()).ok_or_else(|| {
            Error::InvalidInput("a参数缺失或类型错误".to_string())
        })?;
        
        let b = params.get("b").and_then(|v| v.as_f64()).ok_or_else(|| {
            Error::InvalidInput("b参数缺失或类型错误".to_string())
        })?;
        
        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err(Error::InvalidInput("除数不能为零".to_string()));
                }
                a / b
            },
            _ => return Err(Error::InvalidInput(format!("未知操作: {}", operation))),
        };
        
        Ok(json!({ "result": result }))
    }
    
    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(Self)
    }
}

// 定义计算器函数
fn calculator_func(operation: String, a: f64, b: f64) -> Result<Value> {
    let result = match operation.as_str() {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => {
            if b == 0.0 {
                return Err(Error::InvalidInput("除数不能为零".to_string()));
            }
            a / b
        },
        _ => return Err(Error::InvalidInput(format!("未知操作: {}", operation))),
    };
    
    Ok(json!({ "result": result }))
}

// 定义一个天气函数
fn weather_func(city: String) -> Result<Value> {
    // 这只是模拟数据，实际应用中会调用天气API
    let weather_data = match city.to_lowercase().as_str() {
        "beijing" | "北京" => json!({
            "city": "北京",
            "temperature": 25,
            "condition": "晴朗",
            "humidity": 40
        }),
        "shanghai" | "上海" => json!({
            "city": "上海",
            "temperature": 28,
            "condition": "多云",
            "humidity": 65
        }),
        "guangzhou" | "广州" => json!({
            "city": "广州",
            "temperature": 30,
            "condition": "降雨",
            "humidity": 80
        }),
        _ => json!({
            "city": city,
            "temperature": 20,
            "condition": "未知",
            "humidity": 50
        }),
    };
    
    Ok(weather_data)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Lumos Macro 工具示例");
    
    let has_macros = cfg!(feature = "macros");
    println!("宏功能已{}", if has_macros { "启用" } else { "禁用" });
    
    // 测试基本功能，不依赖宏
    println!("\n测试基本功能:");
    
    // 调用计算器函数
    let calc_result = calculator_func("multiply".to_string(), 7.0, 8.0)?;
    println!("计算结果: 7 × 8 = {}", calc_result["result"]);
    
    // 调用天气函数
    let weather_result = weather_func("北京".to_string())?;
    println!("北京的天气: {}°C, {}", weather_result["temperature"], weather_result["condition"]);
    
    #[cfg(not(feature = "macros"))]
    {
        // 使用传统方式定义的工具
        println!("\n使用传统方式定义的工具:");
        
        let calculator = CalculatorTool {};
        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("multiply"));
        params.insert("a".to_string(), json!(7));
        params.insert("b".to_string(), json!(8));
        
        let result = calculator.execute(params, &ToolExecutionOptions::default()).await?;
        println!("计算结果: 7 × 8 = {}", result["result"]);
    }
    
    #[cfg(feature = "macros")]
    {
        println!("\n宏功能已启用");
        println!("在实际应用中，使用#[tool]宏可以自动生成工具定义");
        println!("示例: #[tool(name = \"calculator\", description = \"执行基本的数学运算\")]");
        println!("详情请参考lumos_macro库的文档");
        
        // 注释掉具体的宏用法，因为它需要完整的框架支持
        /* 
        // 使用宏定义一个计算器工具，会自动转换为Tool实现
        #[tool(
            name = "calculator",
            description = "执行基本的数学运算"
        )]
        fn calculator(
            #[parameter(
                name = "operation",
                description = "要执行的操作: add, subtract, multiply, divide",
                r#type = "string", 
                required = true
            )]
            operation: String,
            
            #[parameter(
                name = "a",
                description = "第一个数字",
                r#type = "number",
                required = true
            )]
            a: f64,
            
            #[parameter(
                name = "b",
                description = "第二个数字",
                r#type = "number",
                required = true
            )]
            b: f64,
        ) -> Result<Value> {
            calculator_func(operation, a, b)
        }
        */
    }
    
    #[cfg(not(feature = "macros"))]
    {
        println!("\n要启用宏功能，请在Cargo.toml中添加'macros'特性:");
        println!("lomusai_core = {{ path = \"path/to/lomusai_core\", features = [\"macros\"] }}");
    }
    
    Ok(())
} 