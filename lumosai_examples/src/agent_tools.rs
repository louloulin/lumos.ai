use lumosai_core::{Result, Base, BaseComponent, LogComponent};
use lumosai_core::llm::{Message, Role, MockLlmProvider};
use lumosai_core::agent::{Agent, AgentGenerateOptions, create_basic_agent};
use lumosai_core::tool::{Tool, ToolExecutionOptions, ToolExecutionContext, ToolSchema, ParameterSchema, SchemaFormat};
use lumosai_core::logger::Logger;
use lumosai_core::telemetry::TelemetrySink;
use serde_json::{json, Value};
use async_trait::async_trait;
use std::sync::Arc;

// 简单的计算器工具实现
#[derive(Clone)]
pub struct CalculatorTool {
    id: String,
    base: BaseComponent,
}

impl CalculatorTool {
    pub fn new() -> Self {
        let id = "calculator".to_string();
        Self {
            id: id.clone(),
            base: BaseComponent::new_with_name(id, LogComponent::Tool),
        }
    }
}

impl std::fmt::Debug for CalculatorTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CalculatorTool")
            .field("id", &self.id)
            .finish()
    }
}

impl Base for CalculatorTool {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }

    fn component(&self) -> LogComponent {
        self.base.component()
    }

    fn logger(&self) -> Arc<dyn Logger> {
        self.base.logger()
    }

    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.base.set_logger(logger);
    }

    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        self.base.telemetry()
    }

    fn set_telemetry(&mut self, telemetry: Arc<dyn TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        "执行基本的数学运算"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            parameters: vec![
                ParameterSchema {
                    name: "operation".to_string(),
                    description: "要执行的数学运算".to_string(),
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
            json_schema: None,
            format: SchemaFormat::Parameters,
            output_schema: None,
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

// 简单的天气工具实现
#[derive(Clone)]
pub struct WeatherTool {
    id: String,
    base: BaseComponent,
}

impl WeatherTool {
    pub fn new() -> Self {
        let id = "weather".to_string();
        Self {
            id: id.clone(),
            base: BaseComponent::new_with_name(id, LogComponent::Tool),
        }
    }
}

impl std::fmt::Debug for WeatherTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeatherTool")
            .field("id", &self.id)
            .finish()
    }
}

impl Base for WeatherTool {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }

    fn component(&self) -> LogComponent {
        self.base.component()
    }

    fn logger(&self) -> Arc<dyn Logger> {
        self.base.logger()
    }

    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.base.set_logger(logger);
    }

    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        self.base.telemetry()
    }

    fn set_telemetry(&mut self, telemetry: Arc<dyn TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
}

#[async_trait]
impl Tool for WeatherTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        "获取指定城市的天气信息"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            parameters: vec![
                ParameterSchema {
                    name: "city".to_string(),
                    description: "要查询天气的城市名称".to_string(),
                    r#type: "string".to_string(),
                    required: true,
                    properties: None,
                    default: None,
                },
            ],
            json_schema: None,
            format: SchemaFormat::Parameters,
            output_schema: None,
        }
    }

    async fn execute(&self, params: Value, _context: ToolExecutionContext, _options: &ToolExecutionOptions) -> Result<Value> {
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

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Agent和Tools示例");

    // 创建LLM提供者（使用MockLlmProvider进行测试）
    let mock_responses = vec![
        "我将帮您进行数学计算。".to_string(),
        "我将为您查询天气信息。".to_string(),
    ];
    let llm_provider = Arc::new(MockLlmProvider::new(mock_responses));

    // 创建代理
    let mut agent = create_basic_agent(
        "assistant".to_string(),
        "你是一个通用助手，可以进行数学计算和查询天气信息。".to_string(),
        llm_provider
    );

    // 添加工具到代理
    agent.add_tool(Box::new(CalculatorTool::new()));
    agent.add_tool(Box::new(WeatherTool::new()));

    // 测试计算器工具直接调用
    println!("\n直接调用计算器工具:");
    let calc_tool = CalculatorTool::new();
    let calc_params = json!({
        "operation": "multiply",
        "a": 6.5,
        "b": 7.2
    });

    let context = ToolExecutionContext::default();
    let tool_result = calc_tool.execute(calc_params, context, &ToolExecutionOptions::default()).await?;
    println!("计算结果: {}", tool_result["result"]);

    // 测试天气工具直接调用
    println!("\n直接调用天气工具:");
    let weather_tool = WeatherTool::new();
    let weather_params = json!({
        "city": "北京"
    });

    let context = ToolExecutionContext::default();
    let weather_result = weather_tool.execute(weather_params, context, &ToolExecutionOptions::default()).await?;
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