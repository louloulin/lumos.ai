use lumosai_core::{Agent, Error, Tool};
use lumosai_core::llm::{DeepSeekProvider, LlmOptions, LlmProvider, Message, Role};
use lumosai_core::agent::{AgentBuilder};
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::tool::{ToolBuilder, create_tool};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

// 创建DeepSeek适配器，包装DeepSeekProvider以符合宏要求
struct DeepSeekLlmAdapter {
    provider: Arc<DeepSeekProvider>,
}

impl DeepSeekLlmAdapter {
    fn new(api_key: String) -> Self {
        let provider = Arc::new(DeepSeekProvider::new(
            api_key,
            Some("deepseek-chat".to_string()),
        ));
        Self { provider }
    }
}

// 手动实现LlmProvider trait
#[async_trait]
impl LlmProvider for DeepSeekLlmAdapter {
    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String, Error> {
        self.provider.generate(prompt, options).await
    }

    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String, Error> {
        self.provider.generate_with_messages(messages, options).await
    }

    async fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        options: &'a LlmOptions,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Result<String, Error>> + Send + 'a>>, Error> {
        self.provider.generate_stream(prompt, options).await
    }

    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>, Error> {
        self.provider.get_embedding(text).await
    }
}

// 使用新的简化API创建工具
fn create_stock_price_tool() -> Result<Box<dyn lumosai_core::tool::Tool>, Error> {
    let tool = ToolBuilder::new()
        .name("stock_price")
        .description("获取股票的实时价格信息，包括当前价格、涨跌幅等")
        .parameter("symbol", "string", "股票代码（如AAPL、MSFT、TSLA、GOOGL等）", true)
        .handler(|params| {
            let symbol = params.get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Configuration("Missing or invalid symbol parameter".to_string()))?;

            // 模拟真实的股票数据
            let price_data = match symbol.to_uppercase().as_str() {
                "AAPL" => json!({
                    "symbol": "AAPL",
                    "name": "Apple Inc.",
                    "price": 175.25,
                    "change": 2.35,
                    "percent": 1.36,
                    "volume": 45678900,
                    "market_cap": "2.8T",
                    "pe_ratio": 28.5,
                    "last_updated": "2024-03-15 16:00:00"
                }),
                _ => json!({
                    "symbol": symbol,
                    "error": "找不到该股票信息",
                    "suggestion": "请检查股票代码是否正确，支持的股票包括：AAPL、MSFT、TSLA、GOOGL等"
                })
            };

            Ok(price_data)
        })
        .build()?;

    Ok(Box::new(tool))
}

// 创建DeepSeek提供者的辅助函数
fn create_deepseek_provider() -> Arc<DeepSeekLlmAdapter> {
    let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| "test-key".to_string());
    Arc::new(DeepSeekLlmAdapter::new(api_key))
}

// 使用新的简化API创建Agent
fn create_stock_agent() -> Result<impl lumosai_core::Agent, Error> {
    let llm = create_deepseek_provider();
    
    // 创建工具
    let stock_price_tool = create_stock_price_tool()?;

    // 使用构建器模式创建Agent
    let agent = AgentBuilder::new()
        .name("stock_agent")
        .instructions("你是一个专业的股票分析师和投资顾问。")
        .model(llm)
        .tool(stock_price_tool)
        .max_tool_calls(5)
        .tool_timeout(30)
        .enable_function_calling(true)
        .add_metadata("version", "1.0")
        .add_metadata("category", "finance")
        .build()?;

    Ok(agent)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("🚀 Lumos简化API演示 - 股票助手应用");
    println!("=====================================");

    println!("✅ 正在初始化Lumos股票助手...");

    // 使用新的简化API创建agent
    let app = create_stock_agent()?;

    println!("✅ 应用初始化完成！");
    println!("📱 应用名称: {}", app.get_name());
    println!("🔧 可用工具数量: {}", app.get_tools().len());

    println!("\n🎉 简化API演示完成！");
    println!("✨ 新API特性总结:");
    println!("  • 🏗️  构建器模式 - 更直观的Agent创建");
    println!("  • 🔧 简化工具定义 - 减少样板代码");
    println!("  • 🎯 类型安全 - 编译时错误检查");

    Ok(())
}
