use async_trait::async_trait;
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::agent::AgentBuilder;
use lumosai_core::llm::{DeepSeekProvider, LlmOptions, LlmProvider, Message, Role};
use lumosai_core::tool::{create_tool, ToolBuilder};
use lumosai_core::{Agent, Error, Tool};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

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

    async fn generate_with_messages(
        &self,
        messages: &[Message],
        options: &LlmOptions,
    ) -> Result<String, Error> {
        self.provider
            .generate_with_messages(messages, options)
            .await
    }

    async fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        options: &'a LlmOptions,
    ) -> Result<
        std::pin::Pin<Box<dyn futures::Stream<Item = Result<String, Error>> + Send + 'a>>,
        Error,
    > {
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
                "MSFT" => json!({
                    "symbol": "MSFT",
                    "name": "Microsoft Corporation",
                    "price": 412.50,
                    "change": 5.70,
                    "percent": 1.40,
                    "volume": 23456789,
                    "market_cap": "3.1T",
                    "pe_ratio": 32.1,
                    "last_updated": "2024-03-15 16:00:00"
                }),
                "TSLA" => json!({
                    "symbol": "TSLA",
                    "name": "Tesla Inc.",
                    "price": 185.60,
                    "change": -3.20,
                    "percent": -1.69,
                    "volume": 67890123,
                    "market_cap": "590B",
                    "pe_ratio": 45.2,
                    "last_updated": "2024-03-15 16:00:00"
                }),
                "GOOGL" => json!({
                    "symbol": "GOOGL",
                    "name": "Alphabet Inc.",
                    "price": 138.75,
                    "change": 1.85,
                    "percent": 1.35,
                    "volume": 34567890,
                    "market_cap": "1.7T",
                    "pe_ratio": 25.8,
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

// 使用便利函数创建工具
fn create_stock_news_tool() -> Result<Box<dyn lumosai_core::tool::Tool>, Error> {
    let tool = create_tool(
        "stock_news",
        "获取指定股票的最新新闻和市场动态",
        vec![
            ("symbol", "string", "股票代码", true),
            ("limit", "number", "返回新闻条数（默认3条）", false),
        ],
        |params| {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    Error::Configuration("Missing or invalid symbol parameter".to_string())
                })?;
            let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(3);

            // 模拟真实的新闻数据
            let news = match symbol.to_uppercase().as_str() {
                "AAPL" => json!([
                    {
                        "title": "苹果公司发布iPhone 15系列，销量超预期",
                        "summary": "苹果最新发布的iPhone 15系列在全球市场表现强劲，预订量创历史新高。",
                        "date": "2024-03-15",
                        "source": "科技日报",
                        "sentiment": "positive",
                        "impact": "high"
                    },
                    {
                        "title": "苹果在AI领域加大投资，与OpenAI达成合作",
                        "summary": "苹果宣布将在人工智能领域投资100亿美元，并与OpenAI建立战略合作关系。",
                        "date": "2024-03-14",
                        "source": "财经网",
                        "sentiment": "positive",
                        "impact": "high"
                    },
                    {
                        "title": "苹果服务业务收入创新高，占总收入25%",
                        "summary": "苹果Q1财报显示，服务业务收入达到230亿美元，同比增长11.3%。",
                        "date": "2024-03-13",
                        "source": "华尔街日报",
                        "sentiment": "positive",
                        "impact": "medium"
                    }
                ]),
                _ => json!([
                    {
                        "title": "暂无相关新闻",
                        "summary": "未找到该股票的相关新闻信息",
                        "date": "2024-03-15",
                        "source": "系统提示",
                        "sentiment": "neutral",
                        "impact": "none"
                    }
                ]),
            };

            // 根据limit参数限制返回的新闻数量
            if let Some(news_array) = news.as_array() {
                let limited_news: Vec<_> =
                    news_array.iter().take(limit as usize).cloned().collect();
                Ok(json!(limited_news))
            } else {
                Ok(news)
            }
        },
    )?;

    Ok(Box::new(tool))
}

// 创建DeepSeek提供者的辅助函数
fn create_deepseek_provider() -> Arc<DeepSeekLlmAdapter> {
    let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
    Arc::new(DeepSeekLlmAdapter::new(api_key))
}

// 使用新的简化API创建Agent
fn create_stock_agent() -> Result<impl lumosai_core::Agent, Error> {
    let llm = create_deepseek_provider();

    // 创建工具
    let stock_price_tool = create_stock_price_tool()?;
    let stock_news_tool = create_stock_news_tool()?;

    // 使用构建器模式创建Agent
    let agent = AgentBuilder::new()
        .name("stock_agent")
        .instructions("你是一个专业的股票分析师和投资顾问，擅长分析股票价格、市场趋势和相关新闻。你可以使用专业工具来获取实时股票数据和新闻信息，为用户提供准确、及时的投资建议。请用中文回答，并在适当时候调用相应的工具。")
        .model(llm)
        .tool(stock_price_tool)
        .tool(stock_news_tool)
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

    // 检查API密钥
    let _api_key = match std::env::var("DEEPSEEK_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("❌ 错误：未设置DEEPSEEK_API_KEY环境变量");
            println!("请设置您的DeepSeek API密钥：");
            println!("Windows: $env:DEEPSEEK_API_KEY=\"your-api-key\"");
            println!("Linux/macOS: export DEEPSEEK_API_KEY=\"your-api-key\"");
            return Ok(());
        }
    };

    println!("✅ 正在初始化Lumos股票助手...");

    // 使用新的简化API创建agent
    let app = create_stock_agent()?;

    println!("✅ 应用初始化完成！");
    println!("📱 应用名称: {}", app.get_name());
    println!("📝 应用描述: 基于DeepSeek AI的专业股票分析助手");
    println!("🔧 可用工具数量: {}", app.get_tools().len());

    println!("\n📈 支持的股票:");
    println!("  1. AAPL (苹果)");
    println!("  2. MSFT (微软)");
    println!("  3. TSLA (特斯拉)");
    println!("  4. GOOGL (谷歌)");

    // 演示查询
    let demo_query = "请查询苹果公司(AAPL)的当前股票价格和基本信息";

    println!("\n{}", "=".repeat(60));
    println!("📊 简化API演示");
    println!("{}", "=".repeat(60));
    println!("👤 用户: {}", demo_query);
    println!("\n🤖 Lumos正在分析...");

    // 使用代理处理请求
    let user_message = Message {
        role: Role::User,
        content: demo_query.to_string(),
        metadata: None,
        name: None,
    };

    match app
        .generate(&[user_message], &AgentGenerateOptions::default())
        .await
    {
        Ok(result) => {
            println!("\n💬 Lumos股票助手: {}", result.response);
        }
        Err(e) => {
            println!("❌ 错误: {}", e);
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("🎉 简化API演示完成！");
    println!("{}", "=".repeat(60));
    println!("✨ 新API特性总结:");
    println!("  • 🏗️  构建器模式 - 更直观的Agent创建");
    println!("  • 🔧 简化工具定义 - 减少样板代码");
    println!("  • 🎯 类型安全 - 编译时错误检查");
    println!("  • 📚 更好的文档 - 内置示例和说明");
    println!("  • ⚡ 保持性能 - Rust核心优势不变");

    Ok(())
}
