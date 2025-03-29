use lomusai_core::Result;
use lomusai_core::llm::{LlmAdapter, LlmOptions, LlmProvider};
use lomusai_core::Message;
use lumos_macro::{tools, agent, LlmAdapter};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

// 定义一个简单的LLM适配器用于示例
#[derive(LlmAdapter)]
struct MockLlmAdapter {
    responses: HashMap<String, String>,
}

impl MockLlmAdapter {
    fn new() -> Self {
        let mut responses = HashMap::new();
        responses.insert("计算".to_string(), "根据计算，结果是30.5".to_string());
        responses.insert("查询天气".to_string(), "北京今天晴朗，温度20-25摄氏度".to_string());
        Self { responses }
    }
    
    fn with_model(self, _model: &str) -> Self {
        self
    }
    
    fn with_options(self, _options: &HashMap<String, String>) -> Self {
        self
    }
}

#[async_trait]
impl LlmProvider for MockLlmAdapter {
    async fn generate_with_messages(&self, messages: &[Message], _options: &LlmOptions) -> Result<String> {
        // 简单的模拟实现
        for message in messages {
            if let Some(content) = &message.content {
                for (keyword, response) in &self.responses {
                    if content.contains(keyword) {
                        return Ok(response.clone());
                    }
                }
            }
        }
        Ok("我不知道如何回答这个问题".to_string())
    }
    
    async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> Result<String> {
        Ok("模拟响应".to_string())
    }
    
    async fn generate_stream(&self, _prompt: &str, _options: &LlmOptions) -> Result<Box<dyn Iterator<Item = Result<String>> + Send>> {
        Ok(Box::new(std::iter::once(Ok("模拟流式响应".to_string()))))
    }
    
    async fn get_embedding(&self, _text: &str, _options: &LlmOptions) -> Result<Vec<f32>> {
        Ok(vec![0.1, 0.2, 0.3])
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Agent和Tools示例");
    
    // 使用tools!宏定义工具
    tools! {
        {
            name: "calculator",
            description: "执行基本的数学运算",
            parameters: {
                {
                    name: "operation",
                    description: "要执行的操作: add, subtract, multiply, divide",
                    type: "string",
                    required: true
                },
                {
                    name: "a",
                    description: "第一个数字",
                    type: "number",
                    required: true
                },
                {
                    name: "b",
                    description: "第二个数字",
                    type: "number",
                    required: true
                }
            },
            handler: |params| async move {
                let operation = params.get("operation").unwrap().as_str().unwrap();
                let a = params.get("a").unwrap().as_f64().unwrap();
                let b = params.get("b").unwrap().as_f64().unwrap();
                
                let result = match operation {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    "divide" => if b == 0.0 {
                        return Err(lomusai_core::Error::InvalidInput("Cannot divide by zero".into()))
                    } else {
                        a / b
                    },
                    _ => return Err(lomusai_core::Error::InvalidInput("Unknown operation".into()))
                };
                
                Ok(json!({ "result": result }))
            }
        },
        {
            name: "weather",
            description: "获取指定城市的天气信息",
            parameters: {
                {
                    name: "city",
                    description: "城市名称",
                    type: "string",
                    required: true
                }
            },
            handler: |params| async move {
                let city = params.get("city").unwrap().as_str().unwrap();
                
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
    }
    
    // 使用agent!宏定义代理
    let agent = agent! {
        name: "assistant",
        instructions: "你是一个通用助手，可以进行数学计算和查询天气信息。",
        
        llm: {
            provider: MockLlmAdapter::new(),
            model: "mock-model"
        },
        
        memory: {
            store_type: "buffer",
            capacity: 5
        },
        
        tools: {
            calculator,
            weather
        }
    };
    
    // 使用代理处理查询
    println!("\n处理数学计算查询:");
    let calc_result = agent.run("计算 15.2 + 15.3").await?;
    println!("代理回答: {}", calc_result);
    
    println!("\n处理天气查询:");
    let weather_result = agent.run("查询北京的天气").await?;
    println!("代理回答: {}", weather_result);
    
    // 执行工具直接调用
    println!("\n直接调用计算器工具:");
    let calc_params = json!({
        "operation": "multiply",
        "a": 6.5,
        "b": 7.2
    });
    
    let tool_result: Value = calculator().execute(calc_params, &Default::default()).await?;
    println!("计算结果: {}", tool_result["result"]);
    
    Ok(())
} 