use lumosai_core::Result;
use lumosai_core::llm::{MockLlmProvider, Message, Role};
use lumosai_core::agent::{Agent, AgentConfig, AgentGenerateOptions, create_basic_agent};
use lumosai_core::tool::{Tool, ToolExecutionOptions, ToolSchema, ParameterSchema, ToolExecutionContext};
use lumosai_core::base::Base;
use serde_json::{json, Value};
use async_trait::async_trait;
use std::sync::Arc;

// 简单的计算器工具实现
#[derive(Debug, Clone)]
pub struct CalculatorTool;

impl Base for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    fn id(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "执行基本的数学运算"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "calculator".to_string(),
            description: "执行基本的数学运算".to_string(),
            parameters: vec![
                ParameterSchema {
                    name: "operation".to_string(),
                    parameter_type: "string".to_string(),
                    description: "要执行的数学运算 (add, subtract, multiply, divide)".to_string(),
                    required: true,
                    default: None,
                    format: None,
                },
                ParameterSchema {
                    name: "a".to_string(),
                    parameter_type: "number".to_string(),
                    description: "第一个数字".to_string(),
                    required: true,
                    default: None,
                    format: None,
                },
                ParameterSchema {
                    name: "b".to_string(),
                    parameter_type: "number".to_string(),
                    description: "第二个数字".to_string(),
                    required: true,
                    default: None,
                    format: None,
                },
            ],
        }
    }

    async fn execute(&self, params: Value, _context: ToolExecutionContext, _options: &ToolExecutionOptions) -> Result<Value> {
        let operation = params["operation"].as_str().unwrap_or("");
        let a = params["a"].as_f64().unwrap_or(0.0);
        let b = params["b"].as_f64().unwrap_or(0.0);
        
        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err(lumosai_core::Error::InvalidInput("Cannot divide by zero".into()));
                }
                a / b
            },
            _ => return Err(lumosai_core::Error::InvalidInput("Unknown operation".into()))
        };
        
        Ok(json!({ "result": result }))
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("简单的Agent和Tools示例");
    
    // 创建LLM提供者（使用MockLlmProvider进行测试）
    let llm_provider = Arc::new(MockLlmProvider::new());
    
    // 创建代理
    let mut agent = create_basic_agent(
        "assistant", 
        "你是一个数学助手，可以进行基本计算。", 
        llm_provider
    );
    
    // 添加工具到代理
    agent.add_tool(Box::new(CalculatorTool));
    
    // 测试计算器工具直接调用
    println!("\n直接调用计算器工具:");
    let calc_tool = CalculatorTool;
    let calc_params = json!({
        "operation": "multiply",
        "a": 6.5,
        "b": 7.2
    });
    
    let tool_result = calc_tool.execute(calc_params, ToolExecutionContext::default(), &ToolExecutionOptions::default()).await?;
    println!("计算结果: {}", tool_result["result"]);
    
    // 使用代理处理查询
    println!("\n处理数学计算查询:");
    let user_message = Message {
        role: Role::User,
        content: "计算 15.2 + 15.3".to_string(),
        metadata: None,
        name: None,
    };
    
    let calc_result = agent.generate(&[user_message], &AgentGenerateOptions::default()).await?;
    println!("代理回答: {}", calc_result.response);
    
    Ok(())
}
