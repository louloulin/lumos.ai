use lumosai_core::Result;
use lumosai_core::llm::{LlmOptions, LlmProvider, Message, Role, MockLlmProvider};
use lumosai_core::agent::{Agent, BasicAgent, AgentConfig, AgentGenerateOptions, create_basic_agent};
use lumosai_core::tool::{Tool, ToolExecuteOptions};
use serde_json::{json, Value};
use std::collections::HashMap;
use async_trait::async_trait;
use std::sync::Arc;
use futures::stream::BoxStream;

use lumosai_core::Result;
use lumosai_core::llm::{LlmOptions, MockLlmProvider, Message, Role};
use lumosai_core::agent::{Agent, BasicAgent, AgentConfig, AgentGenerateOptions, create_basic_agent};
use lumosai_core::tool::{Tool, ToolExecuteOptions};
use serde_json::{json, Value};
use async_trait::async_trait;
use std::sync::Arc;

// 简单的计算器工具实现
#[derive(Debug, Clone)]
pub struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "执行基本的数学运算"
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"],
                    "description": "要执行的数学运算"
                },
                "a": {
                    "type": "number",
                    "description": "第一个数字"
                },
                "b": {
                    "type": "number",
                    "description": "第二个数字"
                }
            },
            "required": ["operation", "a", "b"]
        })
    }

    async fn execute(&self, params: Value, _options: &ToolExecuteOptions) -> Result<Value> {
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
}

// 简单的天气工具实现
#[derive(Debug, Clone)]
pub struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "weather"
    }

    fn description(&self) -> &str {
        "获取指定城市的天气信息"
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "要查询天气的城市名称"
                }
            },
            "required": ["city"]
        })
    }

    async fn execute(&self, params: Value, _options: &ToolExecuteOptions) -> Result<Value> {
        let city = params["city"].as_str().unwrap_or("未知");
        
        // 简化的天气数据模拟
        let weather_data = match city {
            "北京" => json!({
                "city": "北京",
                "condition": "晴朗",
                "temperature": { "min": 20, "max": 25 },
                "humidity": 65
            }),
            "上海" => json!({
                "city": "上海",
                "condition": "多云",
                "temperature": { "min": 22, "max": 28 },
                "humidity": 72
            }),
            _ => json!({
                "city": city,
                "condition": "未知",
                "error": "没有该城市的天气数据"
            })
        };
        
        Ok(weather_data)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Agent和Tools示例");
    
    // 创建LLM提供者（使用MockLlmProvider进行测试）
    let llm_provider = Arc::new(MockLlmProvider::new());
    
    // 创建Agent配置
    let agent_config = AgentConfig {
        name: "assistant".to_string(),
        instructions: "你是一个通用助手，可以进行数学计算和查询天气信息。".to_string(),
        memory_config: None,
        model_id: Some("mock-model".to_string()),
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(true),
    };
    
    // 创建代理
    let mut agent = create_basic_agent(
        "assistant", 
        "你是一个通用助手，可以进行数学计算和查询天气信息。", 
        llm_provider
    );
    
    // 添加工具到代理
    agent.add_tool(Box::new(CalculatorTool));
    agent.add_tool(Box::new(WeatherTool));
    
    // 测试计算器工具直接调用
    println!("\n直接调用计算器工具:");
    let calc_tool = CalculatorTool;
    let calc_params = json!({
        "operation": "multiply",
        "a": 6.5,
        "b": 7.2
    });
    
    let tool_result = calc_tool.execute(calc_params, &ToolExecuteOptions::default()).await?;
    println!("计算结果: {}", tool_result["result"]);
    
    // 测试天气工具直接调用
    println!("\n直接调用天气工具:");
    let weather_tool = WeatherTool;
    let weather_params = json!({
        "city": "北京"
    });
    
    let weather_result = weather_tool.execute(weather_params, &ToolExecuteOptions::default()).await?;
    println!("天气结果: {}", weather_result);
    
    // 使用代理处理查询（这里Agent需要实现工具调用功能）
    println!("\n处理数学计算查询:");
    let user_message = Message {
        role: Role::User,
        content: "计算 15.2 + 15.3".to_string(),
        metadata: None,
        name: None,
    };
    
    let calc_result = agent.generate(&[user_message], &AgentGenerateOptions::default()).await?;
    println!("代理回答: {}", calc_result.response);
    
    println!("\n处理天气查询:");
    let user_message = Message {
        role: Role::User,
        content: "查询北京的天气".to_string(),
        metadata: None,
        name: None,
    };
    
    let weather_result = agent.generate(&[user_message], &AgentGenerateOptions::default()).await?;
    println!("代理回答: {}", weather_result.response);
    
    Ok(())
} 