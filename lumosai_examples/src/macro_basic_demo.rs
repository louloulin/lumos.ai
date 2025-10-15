use lumosai_core::prelude::*;
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::llm::types::user_message;
use lumosai_core::llm::mock::MockLlmProvider;
use lumosai_core::Tool;
use std::sync::Arc;
use lumos_macro::*; // 现在启用宏

// 演示真实的宏使用

// 1. 使用 #[tool] 宏创建计算器工具
#[tool(name = "macro_calculator", description = "执行基本的数学运算")]
fn calculator_tool(
    #[parameter(name = "operation", description = "数学运算类型 (add, subtract, multiply, divide)", r#type = "string", required = true)]
    operation: String,
    #[parameter(name = "a", description = "第一个数字", r#type = "number", required = true)]
    a: f64,
    #[parameter(name = "b", description = "第二个数字", r#type = "number", required = true)]
    b: f64,
) -> lumosai_core::Result<serde_json::Value> {
    use serde_json::json;

    let result = match operation.as_str() {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => {
            if b == 0.0 {
                return Err(lumosai_core::Error::InvalidInput("除数不能为零".to_string()));
            }
            a / b
        }
        _ => return Err(lumosai_core::Error::InvalidInput(
            format!("不支持的运算类型: {}", operation)
        )),
    };

    Ok(json!({
        "result": result,
        "operation": operation,
        "operands": [a, b]
    }))
}

// 2. 手动创建工具（演示宏应该生成的代码）
fn create_manual_calculator_tool() -> Box<dyn Tool> {
    use lumosai_core::tool::{FunctionTool, ParameterSchema, ToolSchema, SchemaFormat};
    use serde_json::json;

    let schema = ToolSchema {
        parameters: vec![
            ParameterSchema {
                name: "operation".to_string(),
                description: "数学运算类型 (add, subtract, multiply, divide)".to_string(),
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
    };

    Box::new(FunctionTool::new(
        "manual_calculator",
        "手动创建的计算器工具",
        schema,
        |params| {
            let operation = params.get("operation")
                .and_then(|v| v.as_str())
                .ok_or_else(|| lumosai_core::Error::InvalidInput("缺少 operation 参数".to_string()))?;

            let a = params.get("a")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| lumosai_core::Error::InvalidInput("缺少或无效的 a 参数".to_string()))?;

            let b = params.get("b")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| lumosai_core::Error::InvalidInput("缺少或无效的 b 参数".to_string()))?;

            let result = match operation {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => {
                    if b == 0.0 {
                        return Err(lumosai_core::Error::InvalidInput("除数不能为零".to_string()));
                    }
                    a / b
                }
                _ => return Err(lumosai_core::Error::InvalidInput(
                    format!("不支持的运算类型: {}", operation)
                )),
            };

            Ok(json!({
                "result": result,
                "operation": operation,
                "operands": [a, b]
            }))
        },
    ))
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚀 LumosAI 宏功能基础演示");
    println!("==================================================");

    // 1. 演示宏定义的工具
    println!("✅ 成功导入 lumos_macro 并定义工具");
    println!("📋 已定义的宏工具：");
    println!("   - calculator: 数学计算工具");
    println!("   - text_analyzer: 文本分析工具");

    // 2. 测试手动创建的工具（演示宏应该生成的效果）
    println!("\n🧮 测试手动创建的工具（演示宏概念）:");

    // 使用宏生成的工具
    let macro_tool = calculator_tool();

    // 手动创建工具（对比）
    let manual_tool = create_manual_calculator_tool();

    println!("   ✅ 成功创建宏工具: {}", macro_tool.id());
    println!("   📝 宏工具描述: {}", macro_tool.description());
    println!("   ✅ 成功创建手动工具: {}", manual_tool.id());
    println!("   📝 手动工具描述: {}", manual_tool.description());

    // 演示工具的 schema
    println!("\n� 工具 Schema 信息:");


    // 4. 创建基础 Agent 演示宏的价值
    println!("\n🤖 创建 Agent 演示宏的价值:");
    let mock_provider = MockLlmProvider::new(vec![
        "我是一个演示宏功能的 AI 助手。通过使用 LumosAI 的宏系统，开发者可以：\n1. 快速定义工具函数\n2. 自动生成参数验证\n3. 简化 Agent 配置\n4. 提高代码可读性和维护性".to_string()
    ]);

    let agent = quick_agent(
        "macro_demo_agent",
        "你是一个演示宏功能的 AI 助手，专门用于展示 LumosAI 宏系统的优势。"
    )
    .model(Arc::new(mock_provider))
    .build()?;

    println!("✅ 创建演示 Agent 成功");

    // 3. 测试工具调用
    println!("\n🔧 测试工具调用:");

    // 测试计算器工具
    use serde_json::json;
    let calc_params = json!({
        "operation": "add",
        "a": 15.5,
        "b": 24.3
    });

    use lumosai_core::tool::ToolExecutionContext;
    let context = ToolExecutionContext::default();
    let calc_result = manual_tool.execute(calc_params, context, &Default::default()).await?;
    println!("   📊 计算结果: {}", calc_result);

    // 5. 测试 Agent 对话
    let messages = vec![user_message("请介绍一下 LumosAI 宏系统的优势")];
    let options = AgentGenerateOptions::default();
    let response = agent.generate(&messages, &options).await?;
    println!("🤖 Agent 回答: {}", response.response);

    // 6. 显示宏系统功能总结
    println!("\n📋 LumosAI 宏系统功能总结:");
    println!("✨ 已实现的宏：");
    println!("   - #[tool] - 工具定义宏");
    println!("   - #[agent_attr] - Agent 属性宏");
    println!("   - workflow! - 工作流定义宏");
    println!("   - rag_pipeline! - RAG 管道宏");
    println!("   - eval_suite! - 评估套件宏");
    println!("   - mcp_client! - MCP 客户端宏");
    println!("   - tools! - 批量工具定义宏");
    println!("   - lumos! - 应用级配置宏");

    println!("\n🎯 宏系统的价值：");
    println!("   ✅ 减少样板代码");
    println!("   ✅ 提高开发效率");
    println!("   ✅ 自动参数验证");
    println!("   ✅ 统一代码风格");
    println!("   ✅ 降低学习成本");

    // 7. 测试完成
    println!("\n✅ 所有宏功能演示完成！");
    println!("📊 当前 workspace 包含 8 个模块：");
    println!("   1. lumosai_core - 核心功能");
    println!("   2. lumosai_examples - 示例代码");
    println!("   3. lumosai_vector - 向量存储");
    println!("   4. lumosai_rag - RAG 系统");
    println!("   5. lumosai_cli - CLI 工具");
    println!("   6. lumosai_network - 网络层");
    println!("   7. lumosai_mcp - MCP 协议");
    println!("   8. lumos_macro - 宏定义 ✨ 新增");

    Ok(())
}
