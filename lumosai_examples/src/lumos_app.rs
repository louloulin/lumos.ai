use async_trait::async_trait;
use lumos_macro::{agent, tools};
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::llm::{DeepSeekProvider, LlmOptions, LlmProvider, Message, Role};
use lumosai_core::{Agent, Result};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

// 创建DeepSeek适配器，包装DeepSeekProvider以符合宏要求
// #[derive(LlmAdapter)] // 暂时禁用宏，使用手动实现
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

    fn with_model(self, _model: &str) -> Self {
        // DeepSeek已经在创建时指定了模型
        self
    }

    fn with_options(self, _options: &HashMap<String, String>) -> Self {
        // 可以在这里处理额外的选项
        self
    }
}

// 手动实现LlmProvider trait
#[async_trait]
impl LlmProvider for DeepSeekLlmAdapter {
    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        self.provider.generate(prompt, options).await
    }

    async fn generate_with_messages(
        &self,
        messages: &[Message],
        options: &LlmOptions,
    ) -> Result<String> {
        self.provider
            .generate_with_messages(messages, options)
            .await
    }

    async fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        options: &'a LlmOptions,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Result<String>> + Send + 'a>>> {
        self.provider.generate_stream(prompt, options).await
    }

    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        self.provider.get_embedding(text).await
    }
}

// 使用tools!宏定义工具
tools! {
    {
        name: "stock_price",
        description: "获取股票的实时价格信息，包括当前价格、涨跌幅等",
        parameters: {
            {
                name: "symbol",
                description: "股票代码（如AAPL、MSFT、TSLA、GOOGL等）",
                r#type: "string",
                required: true
            }
        },
        handler: |params| {
            let symbol = params.get("symbol").unwrap().as_str().unwrap();

            // 模拟真实的股票数据（在实际应用中，这里会调用真实的股票API）
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
        }
    },
    {
        name: "stock_news",
        description: "获取指定股票的最新新闻和市场动态",
        parameters: {
            {
                name: "symbol",
                description: "股票代码",
                r#type: "string",
                required: true
            },
            {
                name: "limit",
                description: "返回新闻条数（默认3条）",
                r#type: "number",
                required: false
            }
        },
        handler: |params| {
            let symbol = params.get("symbol").unwrap().as_str().unwrap();
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
                "MSFT" => json!([
                    {
                        "title": "微软Azure云服务增长强劲，市场份额持续扩大",
                        "summary": "微软Azure在Q1实现了30%的增长，在云计算市场的竞争中表现出色。",
                        "date": "2024-03-15",
                        "source": "云计算周刊",
                        "sentiment": "positive",
                        "impact": "high"
                    },
                    {
                        "title": "微软Copilot用户突破1亿，AI业务快速发展",
                        "summary": "微软AI助手Copilot用户数量突破1亿大关，为公司带来显著收入增长。",
                        "date": "2024-03-14",
                        "source": "AI科技评论",
                        "sentiment": "positive",
                        "impact": "high"
                    }
                ]),
                "TSLA" => json!([
                    {
                        "title": "特斯拉Q1交付量略低于预期，股价承压",
                        "summary": "特斯拉Q1全球交付量为38.6万辆，略低于市场预期的40万辆。",
                        "date": "2024-03-15",
                        "source": "汽车新闻",
                        "sentiment": "negative",
                        "impact": "medium"
                    },
                    {
                        "title": "特斯拉在中国市场推出新的充电网络计划",
                        "summary": "特斯拉宣布将在中国新建5000个超级充电桩，进一步完善充电基础设施。",
                        "date": "2024-03-14",
                        "source": "电动汽车时代",
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
                ])
            };

            // 根据limit参数限制返回的新闻数量
            if let Some(news_array) = news.as_array() {
                let limited_news: Vec<_> = news_array.iter().take(limit as usize).cloned().collect();
                Ok(json!(limited_news))
            } else {
                Ok(news)
            }
        }
    }
}

// 创建DeepSeek提供者的辅助函数
fn create_deepseek_provider() -> DeepSeekLlmAdapter {
    let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
    DeepSeekLlmAdapter::new(api_key)
}

// 使用优化后的agent!宏 - 新的简化语法
fn create_stock_agent() -> impl lumosai_core::Agent {
    agent! {
        name: "stock_agent",
        instructions: "你是一个专业的股票分析师和投资顾问，擅长分析股票价格、市场趋势和相关新闻。你可以使用专业工具来获取实时股票数据和新闻信息，为用户提供准确、及时的投资建议。请用中文回答，并在适当时候调用相应的工具。",
        provider: create_deepseek_provider(),
        tools: [stock_price, stock_news]
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Lumos股票助手应用 (基于DeepSeek AI)");
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

    // 暂时直接使用agent，不使用lumos!宏
    let app = create_stock_agent();

    println!("✅ 应用初始化完成！");
    println!("📱 应用名称: Lumos股票助手");
    println!("📝 应用描述: 基于DeepSeek AI的专业股票分析助手");

    println!("\n📈 支持的股票:");
    println!("  1. AAPL (苹果)");
    println!("  2. MSFT (微软)");
    println!("  3. TSLA (特斯拉)");
    println!("  4. GOOGL (谷歌)");

    // 演示各种股票查询功能
    let demo_queries = [
        (
            "📊 股票价格查询",
            "请查询苹果公司(AAPL)的当前股票价格和基本信息",
        ),
        (
            "📰 股票新闻分析",
            "请获取苹果公司的最新新闻，并分析对股价的影响",
        ),
        ("💹 多股票对比", "请对比苹果(AAPL)和微软(MSFT)的股票表现"),
        (
            "🔍 投资建议",
            "基于特斯拉(TSLA)的最新数据，给我一些投资建议",
        ),
    ];

    for (title, query) in demo_queries.iter() {
        println!("\n{}", "=".repeat(60));
        println!("{}", title);
        println!("{}", "=".repeat(60));
        println!("👤 用户: {}", query);
        println!("\n🤖 Lumos正在分析...");

        // 使用代理处理请求
        let user_message = Message {
            role: Role::User,
            content: query.to_string(),
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

        // 添加延迟避免API限制
        println!("\n⏳ 等待3秒后继续下一个查询...");
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }

    println!("\n{}", "=".repeat(60));
    println!("🎉 Lumos股票助手演示完成！");
    println!("{}", "=".repeat(60));
    println!("✨ 应用特性总结:");
    println!("  • 🧠 DeepSeek AI驱动的智能分析");
    println!("  • 📊 实时股票价格查询");
    println!("  • 📰 最新股票新闻获取");
    println!("  • 💡 专业投资建议");
    println!("  • 🌐 中文原生支持");
    println!("  • ⚡ 使用宏简化开发");

    Ok(())
}
