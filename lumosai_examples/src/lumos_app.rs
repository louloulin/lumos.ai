use lumosai_core::Result;
use lumosai_core::llm::{LlmAdapter, LlmOptions, LlmProvider};
use lumosai_core::{Message, Role};
use lumos_macro::{tools, agent, lumos, LlmAdapter};
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
        responses.insert("股票".to_string(), "股票市场今天整体上涨".to_string());
        responses.insert("价格".to_string(), "该股票当前价格为100元".to_string());
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

// 使用tools!宏定义工具
tools! {
    {
        name: "stock_price",
        description: "获取股票价格信息",
        parameters: {
            {
                name: "symbol",
                description: "股票代码",
                type: "string",
                required: true
            }
        },
        handler: |params| async move {
            let symbol = params.get("symbol").unwrap().as_str().unwrap();
            
            // 模拟股票数据
            let price_data = match symbol {
                "AAPL" => json!({
                    "symbol": "AAPL",
                    "name": "Apple Inc.",
                    "price": 175.25,
                    "change": 2.35,
                    "percent": 1.36
                }),
                "MSFT" => json!({
                    "symbol": "MSFT",
                    "name": "Microsoft Corporation",
                    "price": 412.50,
                    "change": 5.70,
                    "percent": 1.40
                }),
                _ => json!({
                    "symbol": symbol,
                    "error": "找不到该股票信息"
                })
            };
            
            Ok(price_data)
        }
    },
    {
        name: "stock_news",
        description: "获取股票相关新闻",
        parameters: {
            {
                name: "symbol",
                description: "股票代码",
                type: "string",
                required: true
            },
            {
                name: "limit",
                description: "返回新闻条数",
                type: "number",
                required: false
            }
        },
        handler: |params| async move {
            let symbol = params.get("symbol").unwrap().as_str().unwrap();
            
            // 模拟新闻数据
            let news = match symbol {
                "AAPL" => json!([
                    {
                        "title": "苹果公司宣布新款iPhone将于下月发布",
                        "date": "2023-03-15",
                        "source": "科技日报"
                    },
                    {
                        "title": "苹果市值突破3万亿美元",
                        "date": "2023-03-10",
                        "source": "财经网"
                    }
                ]),
                _ => json!([])
            };
            
            Ok(news)
        }
    }
}

// 使用agent!宏定义一个股票助手代理
agent! {
    name: "stock_agent",
    instructions: "你是一个专业的股票分析师，能够提供股票价格和相关新闻信息。",
    
    llm: {
        provider: MockLlmAdapter::new(),
        model: "mock-model"
    },
    
    memory: {
        store_type: "buffer",
        capacity: 5
    },
    
    tools: {
        stock_price,
        stock_news
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Lumos应用示例 - 股票助手");
    
    // 使用lumos!宏一次性配置整个应用
    let app = lumos! {
        name: "stock_assistant",
        description: "一个能够提供股票信息的AI助手",
        
        agents: {
            stock_agent
        },
        
        tools: {
            stock_price,
            stock_news
        }
    };
    
    // 使用应用处理请求
    println!("\n查询股票价格:");
    let price_result = app.run("查询苹果公司的股票价格").await?;
    println!("应用回答: {}", price_result);
    
    println!("\n查询股票新闻:");
    let news_result = app.run("有关于苹果公司的最新新闻吗?").await?;
    println!("应用回答: {}", news_result);
    
    Ok(())
} 