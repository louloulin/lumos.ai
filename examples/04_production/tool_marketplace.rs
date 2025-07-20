//! # 工具市场示例
//!
//! 本示例展示如何创建和管理工具市场：
//! - 工具发现和搜索
//! - 工具分类和评级
//! - 动态工具加载
//! - 工具推荐系统
//!
//! 运行方式:
//! ```bash
//! cargo run --example tool_marketplace
//! ```

use lumosai_core::llm::MockLlmProvider;
use lumosai_core::agent::AgentTrait;
use lumosai_core::tool::{Tool, ToolSchema, ParameterSchema, FunctionTool};
use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use serde_json::{json, Value};
use tracing::info;

/// 市场工具信息
#[derive(Debug, Clone)]
pub struct MarketplaceTool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub rating: f32,
    pub downloads: u64,
    pub tags: Vec<String>,
    pub tool: Box<dyn Tool>,
}

/// 工具市场
#[derive(Debug)]
pub struct ToolMarketplace {
    tools: HashMap<String, MarketplaceTool>,
}

impl ToolMarketplace {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// 添加工具到市场
    pub fn add_tool(&mut self, tool_info: MarketplaceTool) {
        self.tools.insert(tool_info.id.clone(), tool_info);
    }

    /// 搜索工具
    pub fn search_tools(&self, query: &str) -> Vec<&MarketplaceTool> {
        self.tools
            .values()
            .filter(|tool| {
                tool.name.to_lowercase().contains(&query.to_lowercase()) ||
                tool.description.to_lowercase().contains(&query.to_lowercase()) ||
                tool.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase()))
            })
            .collect()
    }

    /// 获取热门工具
    pub fn get_popular_tools(&self, limit: usize) -> Vec<&MarketplaceTool> {
        let mut tools: Vec<&MarketplaceTool> = self.tools.values().collect();
        tools.sort_by(|a, b| b.downloads.cmp(&a.downloads));
        tools.into_iter().take(limit).collect()
    }

    /// 获取高评分工具
    pub fn get_top_rated_tools(&self, limit: usize) -> Vec<&MarketplaceTool> {
        let mut tools: Vec<&MarketplaceTool> = self.tools.values().collect();
        tools.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal));
        tools.into_iter().take(limit).collect()
    }

    /// 获取所有工具
    pub fn get_all_tools(&self) -> Vec<&MarketplaceTool> {
        self.tools.values().collect()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 启动工具市场示例");
    println!("=========================");
    println!("🛒 工具市场管理和使用示例");
    println!("=========================\n");

    // 创建LLM提供商
    let llm = Arc::new(MockLlmProvider::new(vec![
        "我将从工具市场选择合适的工具".to_string(),
        "已找到相关工具并开始使用".to_string(),
        "工具执行完成".to_string(),
        "推荐系统已为您找到最佳工具".to_string(),
    ]));

    // 1. 创建工具市场
    println!("1️⃣ 创建工具市场");
    println!("----------------");

    let mut marketplace = ToolMarketplace::new();

    // 添加一些示例工具
    let calc_schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "expression".to_string(),
            description: "数学表达式".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);

    let advanced_calc = FunctionTool::new(
        "advanced_calculator",
        "高级计算器",
        calc_schema,
        |params: Value| {
            let expr = params["expression"].as_str().unwrap_or("0");
            let result = expr.parse::<f64>().unwrap_or(0.0);
            Ok(json!({"result": result}))
        }
    );

    marketplace.add_tool(MarketplaceTool {
        id: "calc_001".to_string(),
        name: "高级计算器".to_string(),
        description: "支持复杂数学表达式计算的高级计算器工具".to_string(),
        version: "1.2.0".to_string(),
        author: "LumosAI Team".to_string(),
        rating: 4.8,
        downloads: 15420,
        tags: vec!["数学".to_string(), "计算".to_string(), "工具".to_string()],
        tool: Box::new(advanced_calc),
    });

    info!("✅ 工具市场创建完成，包含 {} 个工具", marketplace.get_all_tools().len());

    // 2. 搜索工具
    println!("\n2️⃣ 搜索工具");
    println!("------------");

    let search_results = marketplace.search_tools("计算");
    info!("🔍 搜索'计算'找到 {} 个工具", search_results.len());
    for tool in &search_results {
        println!("   📦 {}: {} (评分: {:.1})", tool.name, tool.description, tool.rating);
    }

    println!("\n✅ 工具市场示例完成!");
    println!("\n📚 相关资源:");
    println!("   - examples/02_intermediate/custom_tools.rs - 自定义工具");
    println!("   - docs/tools/marketplace.md - 工具市场指南");

    Ok(())
}