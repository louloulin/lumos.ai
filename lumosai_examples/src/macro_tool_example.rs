use lumosai_core::{Error, Result};
use lumosai_core::tool::{ToolExecutionOptions};
use serde_json::{json, Value};

#[derive(Clone)]
struct Calculator {
    precision: u8,
}

impl Calculator {
    fn new(precision: u8) -> Self {
        Self { precision }
    }
    
    async fn execute(&self, params: Value, _options: &ToolExecutionOptions) -> Result<Value> {
        let operation = params["operation"].as_str().unwrap_or("");
        let a = params["a"].as_f64().unwrap_or(0.0);
        let b = params["b"].as_f64().unwrap_or(0.0);
        
        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err(Error::InvalidInput("Cannot divide by zero".into()));
                }
                a / b
            },
            _ => return Err(Error::InvalidInput("Unknown operation".into()))
        };
        
        // 应用精度设置
        let result = (result * 10_f64.powi(self.precision as i32)).round() / 10_f64.powi(self.precision as i32);
        
        Ok(json!({ "result": result }))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("宏工具示例程序");
    
    // 创建计算器实例
    let calculator = Calculator::new(2); // 精度设置为小数点后2位
    
    // 测试计算器
    let add_params = json!({
        "operation": "add",
        "a": 10.123,
        "b": 5.789
    });
    
    let multiply_params = json!({
        "operation": "multiply",
        "a": 6.5,
        "b": 7.2
    });
    
    let add_result = calculator.execute(add_params, &ToolExecutionOptions::default()).await?;
    let multiply_result = calculator.execute(multiply_params, &ToolExecutionOptions::default()).await?;
    
    println!("加法结果: {}", add_result["result"]);
    println!("乘法结果: {}", multiply_result["result"]);
    
    Ok(())
} 