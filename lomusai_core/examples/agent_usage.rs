use std::collections::HashMap;
use std::sync::Arc;

use lomusai_core::{
    AgentConfig, 
    BasicAgent, 
    AgentGenerateOptions, 
    Message, 
    Role, 
    Tool, 
    create_basic_agent,
};
use lomusai_core::llm::{MockLlmProvider, LlmOptions};
use lomusai_core::tool::{FunctionTool, ParameterSchema, ToolSchema, ToolExecutionOptions};

// 创建一个简单的计算器工具
fn create_calculator_tool() -> Box<dyn Tool> {
    let schema = ToolSchema {
        parameters: vec![
            ParameterSchema {
                name: "operation".to_string(),
                description: "要执行的数学运算，如add（加）、subtract（减）、multiply（乘）或divide（除）".to_string(),
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
    };
    
    Box::new(FunctionTool::new(
        "calculator".to_string(),
        "执行简单的数学运算".to_string(),
        schema,
        |params| {
            let operation = params.get("operation").and_then(|v| v.as_str()).unwrap_or("");
            let a = params.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b = params.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
            
            let result = match operation {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => {
                    if b == 0.0 {
                        return Err(lomusai_core::Error::InvalidInput("除数不能为零".to_string()));
                    }
                    a / b
                },
                _ => return Err(lomusai_core::Error::InvalidInput(format!("未知操作: {}", operation))),
            };
            
            Ok(serde_json::json!({ "result": result }))
        },
    ))
}

// 创建一个天气工具
fn create_weather_tool() -> Box<dyn Tool> {
    let schema = ToolSchema {
        parameters: vec![
            ParameterSchema {
                name: "location".to_string(),
                description: "要查询天气的位置".to_string(),
                r#type: "string".to_string(),
                required: true,
                properties: None,
                default: None,
            },
        ],
    };
    
    Box::new(FunctionTool::new(
        "weather".to_string(),
        "获取指定位置的天气信息".to_string(),
        schema,
        |params| {
            let location = params.get("location").and_then(|v| v.as_str()).unwrap_or("");
            
            // 这只是一个模拟，实际应用中可能会调用真实的天气API
            let weather_data = match location {
                "北京" => serde_json::json!({
                    "temperature": 22,
                    "condition": "晴朗",
                    "humidity": 45
                }),
                "上海" => serde_json::json!({
                    "temperature": 26,
                    "condition": "多云",
                    "humidity": 70
                }),
                "广州" => serde_json::json!({
                    "temperature": 30,
                    "condition": "雨",
                    "humidity": 80
                }),
                _ => serde_json::json!({
                    "temperature": 25,
                    "condition": "未知",
                    "humidity": 50
                }),
            };
            
            Ok(serde_json::json!({
                "location": location,
                "weather": weather_data
            }))
        },
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("创建一个具有多个工具的智能体示例");
    
    // 创建一个模拟的LLM提供者，用于测试
    let mock_responses = vec![
        // 第一个响应调用计算器工具
        "我需要计算一下。\nUsing the tool 'calculator' with parameters: { \"operation\": \"multiply\", \"a\": 12, \"b\": 5 }".to_string(),
        
        // 第二个响应调用天气工具
        "让我为您查询天气。\nUsing the tool 'weather' with parameters: { \"location\": \"北京\" }".to_string(),
        
        // 最终响应
        "根据我的计算，12 乘以 5 等于 60。另外，北京今天的天气是晴朗，温度为22°C，湿度为45%。".to_string(),
    ];
    
    let llm = Arc::new(MockLlmProvider::new(mock_responses));
    
    // 创建智能体
    let config = AgentConfig {
        name: "助手".to_string(),
        instructions: "你是一个有用的助手，可以进行计算和查询天气。".to_string(),
        memory_config: None,
    };
    
    let mut agent = create_basic_agent(config, llm);
    
    // 添加工具
    agent.add_tool(create_calculator_tool())?;
    agent.add_tool(create_weather_tool())?;
    
    // 创建用户消息
    let user_message = Message {
        role: Role::User,
        content: "请计算12乘以5，然后告诉我北京的天气。".to_string(),
    };
    
    // 生成响应
    let options = AgentGenerateOptions::default();
    let result = agent.generate(&[user_message], &options).await?;
    
    // 打印结果
    println!("\n智能体响应：\n{}", result.response);
    println!("\n步骤数量：{}", result.steps.len());
    
    // 分析步骤
    for (i, step) in result.steps.iter().enumerate() {
        println!("\n步骤 {}:", i + 1);
        println!("类型: {:?}", step.step_type);
        
        if !step.tool_calls.is_empty() {
            println!("工具调用:");
            for call in &step.tool_calls {
                println!("  - {}: {:?}", call.name, call.arguments);
            }
        }
        
        if !step.tool_results.is_empty() {
            println!("工具结果:");
            for result in &step.tool_results {
                println!("  - {}: {:?}", result.name, result.result);
            }
        }
    }
    
    Ok(())
}

// 创建一个简单的MockLlmProvider实现，用于测试
mod lomusai_core {
    pub mod llm {
        use std::sync::Arc;
        use async_trait::async_trait;
        use futures::stream::BoxStream;
        
        use lomusai_core::{Result, Error, Message, LlmProvider, LlmOptions};
        
        pub struct MockLlmProvider {
            responses: Vec<String>,
            current_response: std::sync::Mutex<usize>,
        }
        
        impl MockLlmProvider {
            pub fn new(responses: Vec<String>) -> Self {
                Self {
                    responses,
                    current_response: std::sync::Mutex::new(0),
                }
            }
        }
        
        #[async_trait]
        impl LlmProvider for MockLlmProvider {
            async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> Result<String> {
                let mut current = self.current_response.lock().map_err(|_| Error::Lock("Could not lock current_response".to_string()))?;
                
                if *current >= self.responses.len() {
                    return Err(Error::InvalidInput("No more mock responses".to_string()));
                }
                
                let response = self.responses[*current].clone();
                *current += 1;
                
                Ok(response)
            }
            
            async fn generate_with_messages(&self, _messages: &[Message], _options: &LlmOptions) -> Result<String> {
                self.generate("", _options).await
            }
            
            async fn generate_stream<'a>(
                &'a self,
                _prompt: &'a str,
                _options: &'a LlmOptions,
            ) -> Result<BoxStream<'a, Result<String>>> {
                unimplemented!("Streaming not implemented for mock provider")
            }
            
            async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>> {
                unimplemented!("Embeddings not implemented for mock provider")
            }
        }
    }
} 