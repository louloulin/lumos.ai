use lumosai_core::Result;
use lumosai_core::llm::{LlmOptions, LlmProvider, Message, Role};
use serde_json::{json, Value};
use std::collections::HashMap;
use lumos_macro::tool;
use async_trait::async_trait;
use std::sync::Arc;

// 定义一个简单的LLM适配器用于示例
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
}

#[async_trait]
impl LlmProvider for MockLlmAdapter {
    async fn generate(&self, prompt: &str, _options: &LlmOptions) -> Result<String> {
        // 简单的模拟实现
        for (keyword, response) in &self.responses {
            if prompt.contains(keyword) {
                return Ok(response.clone());
            }
        }
        Ok("我不知道如何回答这个问题".to_string())
    }
    
    async fn generate_with_messages(&self, messages: &[Message], _options: &LlmOptions) -> Result<String> {
        // 简单的模拟实现
        for message in messages {
            if message.role == Role::User {
                // 修复Option<String>处理
                if let Some(content) = &message.content {
                    for (keyword, response) in &self.responses {
                        if content.contains(keyword) {
                            return Ok(response.clone());
                        }
                    }
                }
            }
        }
        Ok("我不知道如何回答这个问题".to_string())
    }
    
    // 修复生命周期参数
    async fn generate_stream(&self, prompt: &str, _options: &LlmOptions) -> Result<Box<dyn Iterator<Item = Result<String>> + Send>> {
        let response = self.generate(prompt, _options).await?;
        Ok(Box::new(std::iter::once(Ok(response))))
    }
    
    async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.1, 0.2, 0.3])
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Agent和Tools示例");
    
    // 创建一个简单的计算器工具
    #[tool(name = "calculator", description = "执行基本的数学运算")]
    async fn calculator(params: Value) -> Result<Value> {
        let operation = params["operation"].as_str().unwrap_or("");
        let a = params["a"].as_f64().unwrap_or(0.0);
        let b = params["b"].as_f64().unwrap_or(0.0);
        
        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => if b == 0.0 {
                return Err(lumosai_core::Error::InvalidInput("Cannot divide by zero".into()))
            } else {
                a / b
            },
            _ => return Err(lumosai_core::Error::InvalidInput("Unknown operation".into()))
        };
        
        Ok(json!({ "result": result }))
    }
    
    // 创建一个简单的天气工具
    #[tool(name = "weather", description = "获取指定城市的天气信息")]
    async fn weather(params: Value) -> Result<Value> {
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
    
    // 使用创建的工具
    let llm_provider = Arc::new(MockLlmAdapter::new());
    
    // 创建一个简单的代理
    let agent_config = lumosai_core::agent::AgentConfig {
        name: "assistant".to_string(),
        instructions: "你是一个通用助手，可以进行数学计算和查询天气信息。".to_string(),
        memory_config: None,
        model_id: Some("mock-model".to_string()),
        voice_config: None,
        telemetry: None,
        working_memory: None,
    };
    
    let mut agent = lumosai_core::agent::create_basic_agent(
        "assistant", 
        "你是一个通用助手，可以进行数学计算和查询天气信息。", 
        llm_provider
    );
    
    // 添加工具到代理
    // 获取tool实例
    let calc_tool = calculator();
    let weather_tool = weather();
    
    agent.add_tool(Box::new(calc_tool.clone())).expect("无法添加计算器工具");
    agent.add_tool(Box::new(weather_tool.clone())).expect("无法添加天气工具");
    
    // 使用代理处理查询
    println!("\n处理数学计算查询:");
    let user_message = Message {
        role: Role::User,
        content: Some("计算 15.2 + 15.3".to_string()),
        metadata: None,
        name: None,
    };
    
    let calc_result = agent.generate(&[user_message], &lumosai_core::agent::AgentGenerateOptions::default()).await?;
    println!("代理回答: {}", calc_result.response);
    
    println!("\n处理天气查询:");
    let user_message = Message {
        role: Role::User,
        content: Some("查询北京的天气".to_string()),
        metadata: None,
        name: None,
    };
    
    let weather_result = agent.generate(&[user_message], &lumosai_core::agent::AgentGenerateOptions::default()).await?;
    println!("代理回答: {}", weather_result.response);
    
    // 执行工具直接调用
    println!("\n直接调用计算器工具:");
    let calc_params = json!({
        "operation": "multiply",
        "a": 6.5,
        "b": 7.2
    });
    
    let tool_result = calc_tool.execute(calc_params, &Default::default()).await?;
    println!("计算结果: {}", tool_result["result"]);
    
    Ok(())
} 