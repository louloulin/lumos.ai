use lumosai_derive::FunctionSchema;
use serde_json::Value;

/// 演示 lumosai_derive 宏的基本功能
/// 
/// 这个示例展示了如何使用 #[derive(FunctionSchema)] 宏
/// 自动为函数生成 JSON Schema

#[derive(FunctionSchema)]
struct Calculator {
    precision: u8,
}

impl Calculator {
    fn new(precision: u8) -> Self {
        Self { precision }
    }

    /// 执行数学计算
    /// 
    /// # 参数
    /// - operation: 要执行的数学运算（add, subtract, multiply, divide）
    /// - a: 第一个数字
    /// - b: 第二个数字
    /// 
    /// # 返回值
    /// 计算结果
    fn calculate(&self, operation: String, a: f64, b: f64) -> Result<f64, String> {
        match operation.as_str() {
            "add" => Ok(a + b),
            "subtract" => Ok(a - b),
            "multiply" => Ok(a * b),
            "divide" => {
                if b == 0.0 {
                    Err("除数不能为零".to_string())
                } else {
                    Ok(a / b)
                }
            }
            _ => Err(format!("不支持的运算: {}", operation)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧮 LumosAI Derive 宏基础演示");
    println!("========================================");

    // 创建计算器实例
    let calculator = Calculator::new(2);

    // 演示基本计算功能
    println!("\n📊 基础计算演示:");
    
    let operations = vec![
        ("add", 10.0, 5.0),
        ("subtract", 10.0, 3.0),
        ("multiply", 4.0, 7.0),
        ("divide", 15.0, 3.0),
    ];

    for (op, a, b) in operations {
        match calculator.calculate(op.to_string(), a, b) {
            Ok(result) => {
                println!("  {} {} {} = {:.2}", a, op, b, result);
            }
            Err(e) => {
                println!("  错误: {}", e);
            }
        }
    }

    // 演示错误处理
    println!("\n⚠️  错误处理演示:");
    match calculator.calculate("divide".to_string(), 10.0, 0.0) {
        Ok(result) => println!("  结果: {}", result),
        Err(e) => println!("  ✅ 正确捕获错误: {}", e),
    }

    match calculator.calculate("invalid".to_string(), 1.0, 2.0) {
        Ok(result) => println!("  结果: {}", result),
        Err(e) => println!("  ✅ 正确捕获错误: {}", e),
    }

    // 演示 derive 宏生成的功能（如果可用）
    println!("\n🔧 Derive 宏功能演示:");
    println!("  Calculator 结构体已成功使用 #[derive(FunctionSchema)] 宏");
    println!("  宏会自动生成 JSON Schema 相关的代码");
    
    // 显示结构体信息
    println!("  - 精度设置: {} 位小数", calculator.precision);
    println!("  - 支持的运算: add, subtract, multiply, divide");

    println!("\n✅ Derive 宏演示完成!");
    println!("   lumosai_derive 包已成功集成到 workspace 中");

    Ok(())
}
