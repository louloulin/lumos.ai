//! 工具集成演示
//!
//! 展示如何创建和使用工具系统，包括：
//! - 自定义工具创建
//! - 工具与 Agent 集成
//! - 工具调用和结果处理
//! - 内置工具使用

use async_trait::async_trait;
use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::base::{Base, BaseComponent};
use lumosai_core::compat::Component;
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::telemetry::TelemetrySink;
use lumosai_core::tool::{
    ParameterSchema, SchemaFormat, Tool, ToolExecutionContext, ToolExecutionOptions, ToolSchema,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🛠️ 工具集成演示");
    println!("================");

    // 演示1: 创建自定义工具
    demo_custom_tools().await?;

    // 演示2: Agent 与工具集成
    demo_agent_with_tools().await?;

    // 演示3: 复杂工具链
    demo_tool_chain().await?;

    // 演示4: 内置工具使用
    demo_builtin_tools().await?;

    Ok(())
}

/// 演示自定义工具创建
async fn demo_custom_tools() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示1: 自定义工具创建 ===");

    // 创建计算器工具
    let calculator = CalculatorTool::new();

    // 测试工具
    let context = ToolExecutionContext::new();
    let params = json!({
        "expression": "15 + 27 * 3",
        "precision": 2
    });

    let result = calculator
        .execute(params, context.clone(), &ToolExecutionOptions::default())
        .await?;
    println!("计算器工具测试:");
    println!("表达式: 15 + 27 * 3");
    println!("结果: {}", result);

    // 创建天气工具
    let weather_tool = WeatherTool::new();
    let weather_params = json!({
        "city": "北京",
        "units": "metric"
    });

    let weather_result = weather_tool
        .execute(
            weather_params,
            context.clone(),
            &ToolExecutionOptions::default(),
        )
        .await?;
    println!("\n天气工具测试:");
    println!("城市: 北京");
    println!("结果: {}", weather_result);

    Ok(())
}

/// 演示 Agent 与工具集成
async fn demo_agent_with_tools() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示2: Agent 与工具集成 ===");

    // 创建工具
    let calculator = Arc::new(CalculatorTool::new());
    let weather_tool = Arc::new(WeatherTool::new());

    // 创建模拟响应，包含工具调用
    let mock_responses = vec![
        "我来帮您计算 (15 + 27) * 3 的结果。让我使用计算器工具来计算这个表达式。".to_string(),
        "根据计算结果，(15 + 27) * 3 = 126。这个计算过程是：首先计算括号内的 15 + 27 = 42，然后 42 * 3 = 126。".to_string(),
        "我来为您查询北京的天气情况。".to_string(),
        "根据查询结果，北京今天的天气是晴朗，温度22°C，湿度65%，微风。适合外出活动。".to_string(),
    ];
    let llm_provider = Arc::new(MockLlmProvider::new(mock_responses));

    // 创建带工具的 Agent
    let agent = AgentBuilder::new()
        .name("tool_agent")
        .instructions("你是一个助手，可以使用计算器和天气查询工具来帮助用户。当用户需要计算时使用计算器工具，需要天气信息时使用天气工具。")
        .model(llm_provider)
        .tools(vec![
            calculator.clone_box(),
            weather_tool.clone_box()
        ])
        .build()?;

    // 测试计算功能
    println!("测试计算功能:");
    let calc_response = agent.generate_simple("请计算 (15 + 27) * 3 的结果").await?;
    println!("用户: 请计算 (15 + 27) * 3 的结果");
    println!("AI: {}", calc_response);

    // 测试天气查询
    println!("\n测试天气查询:");
    let weather_response = agent.generate_simple("请查询北京的天气情况").await?;
    println!("用户: 请查询北京的天气情况");
    println!("AI: {}", weather_response);

    Ok(())
}

/// 演示复杂工具链
async fn demo_tool_chain() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示3: 复杂工具链 ===");

    // 创建数据处理工具链
    let data_fetcher = Arc::new(DataFetcherTool::new());
    let data_processor = Arc::new(DataProcessorTool::new());
    let report_generator = Arc::new(ReportGeneratorTool::new());

    let mock_responses = vec![
        "我将为您执行完整的数据分析流程：首先获取数据，然后处理分析，最后生成报告。".to_string(),
        "数据分析完成！我已经获取了销售数据，进行了趋势分析，并生成了详细报告。报告显示销售呈上升趋势，建议继续当前策略。".to_string(),
    ];
    let llm_provider = Arc::new(MockLlmProvider::new(mock_responses));

    let pipeline_agent = AgentBuilder::new()
        .name("data_analyst")
        .instructions("你是一个数据分析专家，可以获取数据、处理分析并生成报告。请按顺序使用工具完成完整的分析流程。")
        .model(llm_provider)
        .tools(vec![
            data_fetcher.clone_box(),
            data_processor.clone_box(),
            report_generator.clone_box()
        ])
        .build()?;

    let response = pipeline_agent
        .generate_simple("请执行完整的数据分析流程：获取最新销售数据，进行趋势分析，并生成报告")
        .await?;

    println!("数据分析流程:");
    println!("用户: 请执行完整的数据分析流程：获取最新销售数据，进行趋势分析，并生成报告");
    println!("AI: {}", response);

    Ok(())
}

/// 演示内置工具使用
async fn demo_builtin_tools() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 演示4: 内置工具使用 ===");

    // 这里演示如何使用内置工具（如果有的话）
    println!("内置工具演示:");
    println!("- 文件操作工具");
    println!("- HTTP 客户端工具");
    println!("- 数据处理工具");
    println!("- 系统信息工具");

    // 注意：实际的内置工具需要在 lumosai_core 中实现
    println!("\n注意: 内置工具的具体实现需要在 lumosai_core::tool::builtin 模块中完成");

    Ok(())
}

// ============================================================================
// 自定义工具实现
// ============================================================================

/// 计算器工具
#[derive(Clone)]
pub struct CalculatorTool {
    name: String,
    base: BaseComponent,
}

impl std::fmt::Debug for CalculatorTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CalculatorTool")
            .field("name", &self.name)
            .finish()
    }
}

impl CalculatorTool {
    pub fn new() -> Self {
        Self {
            name: "calculator".to_string(),
            base: BaseComponent::new_with_name("calculator".to_string(), Component::Tool),
        }
    }
}

impl Base for CalculatorTool {
    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn component(&self) -> Component {
        Component::Tool
    }

    fn logger(&self) -> Arc<dyn lumosai_core::Logger> {
        self.base.logger()
    }

    fn set_logger(&mut self, logger: Arc<dyn lumosai_core::Logger>) {
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
        &self.name
    }

    fn description(&self) -> &str {
        "执行基础数学计算，支持加减乘除和括号运算"
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(CalculatorTool {
            name: self.name.clone(),
            base: self.base.clone(),
        })
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            parameters: vec![
                ParameterSchema {
                    name: "expression".to_string(),
                    description: "要计算的数学表达式".to_string(),
                    r#type: "string".to_string(),
                    required: true,
                    properties: None,
                    default: None,
                },
                ParameterSchema {
                    name: "precision".to_string(),
                    description: "计算精度（小数位数）".to_string(),
                    r#type: "integer".to_string(),
                    required: false,
                    properties: None,
                    default: Some(json!(2)),
                },
            ],
            json_schema: Some(json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "要计算的数学表达式"
                    },
                    "precision": {
                        "type": "integer",
                        "description": "计算精度（小数位数）",
                        "default": 2
                    }
                },
                "required": ["expression"]
            })),
            format: SchemaFormat::JsonSchema,
            output_schema: None,
        }
    }

    async fn execute(
        &self,
        params: Value,
        _ctx: ToolExecutionContext,
        _opts: &ToolExecutionOptions,
    ) -> lumosai_core::Result<Value> {
        let expression = params["expression"]
            .as_str()
            .ok_or_else(|| lumosai_core::Error::Tool("缺少表达式参数".to_string()))?;

        let precision = params["precision"].as_u64().unwrap_or(2) as usize;

        // 简单的表达式计算（实际项目中应使用 evalexpr 等库）
        let result = match expression {
            "15 + 27 * 3" => 96.0,
            "(15 + 27) * 3" => 126.0,
            "15 + 27" => 42.0,
            _ => 42.0, // 默认值
        };

        Ok(json!({
            "result": format!("{:.precision$}", result, precision = precision),
            "expression": expression,
            "precision": precision
        }))
    }
}

/// 天气查询工具
#[derive(Clone)]
pub struct WeatherTool {
    name: String,
    base: BaseComponent,
}

impl std::fmt::Debug for WeatherTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeatherTool")
            .field("name", &self.name)
            .finish()
    }
}

impl WeatherTool {
    pub fn new() -> Self {
        Self {
            name: "weather".to_string(),
            base: BaseComponent::new_with_name("weather".to_string(), Component::Tool),
        }
    }
}

impl Base for WeatherTool {
    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn component(&self) -> Component {
        Component::Tool
    }

    fn logger(&self) -> Arc<dyn lumosai_core::Logger> {
        self.base.logger()
    }

    fn set_logger(&mut self, logger: Arc<dyn lumosai_core::Logger>) {
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
        &self.name
    }

    fn description(&self) -> &str {
        "查询指定城市的天气信息"
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(WeatherTool {
            name: self.name.clone(),
            base: self.base.clone(),
        })
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            parameters: vec![
                ParameterSchema {
                    name: "city".to_string(),
                    description: "城市名称".to_string(),
                    r#type: "string".to_string(),
                    required: true,
                    properties: None,
                    default: None,
                },
                ParameterSchema {
                    name: "units".to_string(),
                    description: "温度单位 (metric/imperial)".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                    properties: None,
                    default: Some(json!("metric")),
                },
            ],
            json_schema: Some(json!({
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "城市名称"
                    },
                    "units": {
                        "type": "string",
                        "description": "温度单位 (metric/imperial)",
                        "default": "metric"
                    }
                },
                "required": ["city"]
            })),
            format: SchemaFormat::JsonSchema,
            output_schema: None,
        }
    }

    async fn execute(
        &self,
        params: Value,
        _ctx: ToolExecutionContext,
        _opts: &ToolExecutionOptions,
    ) -> lumosai_core::Result<Value> {
        let city = params["city"]
            .as_str()
            .ok_or_else(|| lumosai_core::Error::Tool("缺少城市参数".to_string()))?;

        let _units = params["units"].as_str().unwrap_or("metric");

        // 模拟天气查询（实际项目中调用真实 API）
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let weather_data = match city {
            "北京" => json!({
                "city": "北京",
                "temperature": "22°C",
                "condition": "晴朗",
                "humidity": "65%",
                "wind": "微风"
            }),
            "上海" => json!({
                "city": "上海",
                "temperature": "25°C",
                "condition": "多云",
                "humidity": "70%",
                "wind": "东南风"
            }),
            _ => json!({
                "city": city,
                "temperature": "20°C",
                "condition": "未知",
                "humidity": "60%",
                "wind": "无风"
            }),
        };

        Ok(weather_data)
    }
}

/// 数据获取工具
#[derive(Clone)]
pub struct DataFetcherTool {
    name: String,
    base: BaseComponent,
}

impl std::fmt::Debug for DataFetcherTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataFetcherTool")
            .field("name", &self.name)
            .finish()
    }
}

impl DataFetcherTool {
    pub fn new() -> Self {
        Self {
            name: "data_fetcher".to_string(),
            base: BaseComponent::new_with_name("data_fetcher".to_string(), Component::Tool),
        }
    }
}

impl Base for DataFetcherTool {
    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn component(&self) -> Component {
        Component::Tool
    }

    fn logger(&self) -> Arc<dyn lumosai_core::Logger> {
        self.base.logger()
    }

    fn set_logger(&mut self, logger: Arc<dyn lumosai_core::Logger>) {
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
impl Tool for DataFetcherTool {
    fn id(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "从数据源获取数据"
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(DataFetcherTool {
            name: self.name.clone(),
            base: self.base.clone(),
        })
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            parameters: vec![ParameterSchema {
                name: "source".to_string(),
                description: "数据源类型".to_string(),
                r#type: "string".to_string(),
                required: true,
                properties: None,
                default: None,
            }],
            json_schema: Some(json!({
                "type": "object",
                "properties": {
                    "source": {
                        "type": "string",
                        "description": "数据源类型"
                    }
                },
                "required": ["source"]
            })),
            format: SchemaFormat::JsonSchema,
            output_schema: None,
        }
    }

    async fn execute(
        &self,
        params: Value,
        _ctx: ToolExecutionContext,
        _opts: &ToolExecutionOptions,
    ) -> lumosai_core::Result<Value> {
        let _source = params["source"].as_str().unwrap_or("default");

        // 模拟数据获取
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        Ok(json!({
            "status": "success",
            "data": [
                {"date": "2024-01", "sales": 10000},
                {"date": "2024-02", "sales": 12000},
                {"date": "2024-03", "sales": 15000}
            ],
            "count": 3
        }))
    }
}

/// 数据处理工具
#[derive(Clone)]
pub struct DataProcessorTool {
    name: String,
    base: BaseComponent,
}

impl std::fmt::Debug for DataProcessorTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataProcessorTool")
            .field("name", &self.name)
            .finish()
    }
}

impl DataProcessorTool {
    pub fn new() -> Self {
        Self {
            name: "data_processor".to_string(),
            base: BaseComponent::new_with_name("data_processor".to_string(), Component::Tool),
        }
    }
}

impl Base for DataProcessorTool {
    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn component(&self) -> Component {
        Component::Tool
    }

    fn logger(&self) -> Arc<dyn lumosai_core::Logger> {
        self.base.logger()
    }

    fn set_logger(&mut self, logger: Arc<dyn lumosai_core::Logger>) {
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
impl Tool for DataProcessorTool {
    fn id(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "处理和分析数据"
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(DataProcessorTool {
            name: self.name.clone(),
            base: self.base.clone(),
        })
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            parameters: vec![ParameterSchema {
                name: "data".to_string(),
                description: "要处理的数据".to_string(),
                r#type: "object".to_string(),
                required: true,
                properties: None,
                default: None,
            }],
            json_schema: Some(json!({
                "type": "object",
                "properties": {
                    "data": {
                        "type": "object",
                        "description": "要处理的数据"
                    }
                },
                "required": ["data"]
            })),
            format: SchemaFormat::JsonSchema,
            output_schema: None,
        }
    }

    async fn execute(
        &self,
        params: Value,
        _ctx: ToolExecutionContext,
        _opts: &ToolExecutionOptions,
    ) -> lumosai_core::Result<Value> {
        let _data = &params["data"];

        // 模拟数据处理
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

        Ok(json!({
            "status": "processed",
            "analysis": {
                "trend": "increasing",
                "growth_rate": "25%",
                "total": 37000
            }
        }))
    }
}

/// 报告生成工具
#[derive(Clone)]
pub struct ReportGeneratorTool {
    name: String,
    base: BaseComponent,
}

impl std::fmt::Debug for ReportGeneratorTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReportGeneratorTool")
            .field("name", &self.name)
            .finish()
    }
}

impl ReportGeneratorTool {
    pub fn new() -> Self {
        Self {
            name: "report_generator".to_string(),
            base: BaseComponent::new_with_name("report_generator".to_string(), Component::Tool),
        }
    }
}

impl Base for ReportGeneratorTool {
    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn component(&self) -> Component {
        Component::Tool
    }

    fn logger(&self) -> Arc<dyn lumosai_core::Logger> {
        self.base.logger()
    }

    fn set_logger(&mut self, logger: Arc<dyn lumosai_core::Logger>) {
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
impl Tool for ReportGeneratorTool {
    fn id(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "生成分析报告"
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(ReportGeneratorTool {
            name: self.name.clone(),
            base: self.base.clone(),
        })
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            parameters: vec![ParameterSchema {
                name: "analysis".to_string(),
                description: "分析结果".to_string(),
                r#type: "object".to_string(),
                required: true,
                properties: None,
                default: None,
            }],
            json_schema: Some(json!({
                "type": "object",
                "properties": {
                    "analysis": {
                        "type": "object",
                        "description": "分析结果"
                    }
                },
                "required": ["analysis"]
            })),
            format: SchemaFormat::JsonSchema,
            output_schema: None,
        }
    }

    async fn execute(
        &self,
        params: Value,
        _ctx: ToolExecutionContext,
        _opts: &ToolExecutionOptions,
    ) -> lumosai_core::Result<Value> {
        let _analysis = &params["analysis"];

        // 模拟报告生成
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

        Ok(json!({
            "status": "generated",
            "report": {
                "title": "销售趋势分析报告",
                "summary": "销售数据显示持续增长趋势",
                "recommendations": ["继续当前策略", "扩大市场投入"]
            }
        }))
    }
}
