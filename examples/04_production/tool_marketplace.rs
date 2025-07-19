//! 工具市场平台示例 - 展示工具发现、分享和管理
//! 
//! 这个示例展示了LumosAI的工具市场功能，包括工具发布、发现、评级和管理。
//! 
//! 运行方式:
//! ```bash
//! cargo run --example tool_marketplace
//! ```

use lumosai_core::prelude::*;
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::tool::{Tool, ToolMetadata, ToolCategory};
use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use tracing::{info, error};
use serde::{Serialize, Deserialize};

/// 工具市场条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceTool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: ToolCategory,
    pub author: String,
    pub version: String,
    pub rating: f64,
    pub download_count: u64,
    pub tags: Vec<String>,
    pub metadata: ToolMetadata,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 工具市场
#[derive(Debug)]
pub struct ToolMarketplace {
    tools: HashMap<String, MarketplaceTool>,
    categories: HashMap<ToolCategory, Vec<String>>,
    featured_tools: Vec<String>,
}

impl ToolMarketplace {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            categories: HashMap::new(),
            featured_tools: Vec::new(),
        }
    }

    /// 发布工具到市场
    pub fn publish_tool(&mut self, tool: MarketplaceTool) -> Result<()> {
        let tool_id = tool.id.clone();
        let category = tool.category.clone();
        
        // 添加到工具列表
        self.tools.insert(tool_id.clone(), tool);
        
        // 添加到分类
        self.categories.entry(category).or_insert_with(Vec::new).push(tool_id);
        
        Ok(())
    }

    /// 搜索工具
    pub fn search_tools(&self, query: &str, category: Option<ToolCategory>) -> Vec<&MarketplaceTool> {
        self.tools.values()
            .filter(|tool| {
                let matches_query = tool.name.to_lowercase().contains(&query.to_lowercase()) ||
                                  tool.description.to_lowercase().contains(&query.to_lowercase()) ||
                                  tool.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase()));
                
                let matches_category = category.map_or(true, |cat| tool.category == cat);
                
                matches_query && matches_category
            })
            .collect()
    }

    /// 获取热门工具
    pub fn get_popular_tools(&self, limit: usize) -> Vec<&MarketplaceTool> {
        let mut tools: Vec<_> = self.tools.values().collect();
        tools.sort_by(|a, b| b.download_count.cmp(&a.download_count));
        tools.into_iter().take(limit).collect()
    }

    /// 获取推荐工具
    pub fn get_featured_tools(&self) -> Vec<&MarketplaceTool> {
        self.featured_tools.iter()
            .filter_map(|id| self.tools.get(id))
            .collect()
    }

    /// 按分类获取工具
    pub fn get_tools_by_category(&self, category: ToolCategory) -> Vec<&MarketplaceTool> {
        self.categories.get(&category)
            .map(|tool_ids| {
                tool_ids.iter()
                    .filter_map(|id| self.tools.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🛒 LumosAI 工具市场示例");
    println!("========================");

    // 创建LLM提供者
    let llm = Arc::new(MockLlmProvider::new(vec![
        "我已经从工具市场安装了新工具。".to_string(),
        "工具市场搜索完成。".to_string(),
        "推荐工具已加载。".to_string(),
        "工具评级已更新。".to_string(),
    ]));

    // 1. 创建工具市场
    println!("\n1️⃣ 创建工具市场");
    println!("----------------");

    let mut marketplace = ToolMarketplace::new();
    
    // 添加示例工具
    populate_marketplace(&mut marketplace)?;
    
    info!("✅ 工具市场创建成功");
    info!("📊 市场统计:");
    info!("   - 总工具数: {}", marketplace.tools.len());
    info!("   - 分类数: {}", marketplace.categories.len());

    // 2. 工具搜索和发现
    println!("\n2️⃣ 工具搜索和发现");
    println!("------------------");

    // 搜索天气相关工具
    let weather_tools = marketplace.search_tools("weather", None);
    info!("🔍 搜索'weather'找到{}个工具:", weather_tools.len());
    for tool in &weather_tools {
        info!("   - {}: {} (⭐ {:.1})", tool.name, tool.description, tool.rating);
    }

    // 按分类搜索
    let web_tools = marketplace.search_tools("", Some(ToolCategory::Web));
    info!("🌐 Web分类工具{}个:", web_tools.len());
    for tool in &web_tools {
        info!("   - {}: {} (📥 {})", tool.name, tool.description, tool.download_count);
    }

    // 3. 热门工具展示
    println!("\n3️⃣ 热门工具");
    println!("------------");

    let popular_tools = marketplace.get_popular_tools(5);
    info!("🔥 热门工具TOP 5:");
    for (i, tool) in popular_tools.iter().enumerate() {
        info!("   {}. {} - {} 次下载 (⭐ {:.1})", 
              i + 1, tool.name, tool.download_count, tool.rating);
    }

    // 4. 创建使用市场工具的Agent
    println!("\n4️⃣ 使用市场工具的Agent");
    println!("----------------------");

    // 从市场选择工具
    let selected_tools = select_tools_from_marketplace(&marketplace, vec![
        "weather_pro",
        "calculator_advanced", 
        "web_scraper_pro",
        "file_manager_plus"
    ])?;

    let marketplace_agent = quick_agent("marketplace_user", "使用工具市场工具的助手")
        .model(llm.clone())
        .tools(selected_tools)
        .build()?;

    info!("✅ 市场工具Agent创建成功，工具数量: {}", marketplace_agent.get_tools().len());

    // 测试Agent
    let response = marketplace_agent.generate("请使用高级计算器计算复杂表达式").await?;
    println!("🤖 市场助手: {}", response.content);

    // 5. 工具推荐系统
    println!("\n5️⃣ 工具推荐系统");
    println!("----------------");

    let recommendations = get_tool_recommendations(&marketplace, &["数据分析", "机器学习"])?;
    info!("💡 基于兴趣推荐{}个工具:", recommendations.len());
    for tool in &recommendations {
        info!("   - {}: {} (匹配度: {:.1})", 
              tool.name, tool.description, calculate_match_score(tool, &["数据分析", "机器学习"]));
    }

    // 6. 工具评级和反馈
    println!("\n6️⃣ 工具评级系统");
    println!("----------------");

    // 模拟用户评级
    let ratings = vec![
        ("weather_pro", 4.8),
        ("calculator_advanced", 4.6),
        ("web_scraper_pro", 4.2),
        ("file_manager_plus", 4.5),
    ];

    for (tool_id, rating) in ratings {
        if let Some(tool) = marketplace.tools.get_mut(tool_id) {
            let old_rating = tool.rating;
            tool.rating = (tool.rating + rating) / 2.0; // 简化的评级更新
            info!("📊 {} 评级更新: {:.1} -> {:.1}", tool.name, old_rating, tool.rating);
        }
    }

    // 7. 工具安装和管理
    println!("\n7️⃣ 工具安装管理");
    println!("----------------");

    let tool_manager = ToolManager::new();
    
    // 安装工具
    let tools_to_install = vec!["weather_pro", "calculator_advanced"];
    for tool_id in tools_to_install {
        match tool_manager.install_tool(&marketplace, tool_id).await {
            Ok(_) => info!("✅ 工具 {} 安装成功", tool_id),
            Err(e) => error!("❌ 工具 {} 安装失败: {}", tool_id, e),
        }
    }

    // 列出已安装工具
    let installed_tools = tool_manager.list_installed_tools();
    info!("📦 已安装工具: {:?}", installed_tools);

    // 8. 工具市场统计
    println!("\n8️⃣ 市场统计");
    println!("------------");

    let stats = generate_marketplace_stats(&marketplace);
    info!("📈 工具市场统计:");
    info!("   - 总工具数: {}", stats.total_tools);
    info!("   - 总下载量: {}", stats.total_downloads);
    info!("   - 平均评分: {:.2}", stats.average_rating);
    info!("   - 活跃开发者: {}", stats.active_developers);

    // 按分类统计
    info!("📊 分类统计:");
    for (category, count) in stats.category_counts {
        info!("   - {:?}: {} 个工具", category, count);
    }

    println!("\n🎉 工具市场示例完成!");
    println!("\n📚 下一步学习:");
    println!("   - examples/04_production/monitoring.rs - 性能监控");
    println!("   - examples/04_production/deployment.rs - 部署指南");
    println!("   - docs/best-practices/tool-marketplace.md - 工具市场最佳实践");

    Ok(())
}

/// 填充市场示例数据
fn populate_marketplace(marketplace: &mut ToolMarketplace) -> Result<()> {
    let tools = vec![
        MarketplaceTool {
            id: "weather_pro".to_string(),
            name: "Weather Pro".to_string(),
            description: "专业天气查询工具，支持全球城市".to_string(),
            category: ToolCategory::Web,
            author: "WeatherCorp".to_string(),
            version: "2.1.0".to_string(),
            rating: 4.8,
            download_count: 15420,
            tags: vec!["weather".to_string(), "api".to_string(), "global".to_string()],
            metadata: ToolMetadata::default(),
            created_at: chrono::Utc::now() - chrono::Duration::days(90),
            updated_at: chrono::Utc::now() - chrono::Duration::days(7),
        },
        MarketplaceTool {
            id: "calculator_advanced".to_string(),
            name: "Advanced Calculator".to_string(),
            description: "高级数学计算器，支持复杂表达式".to_string(),
            category: ToolCategory::Math,
            author: "MathTools Inc".to_string(),
            version: "1.5.2".to_string(),
            rating: 4.6,
            download_count: 8930,
            tags: vec!["math".to_string(), "calculator".to_string(), "advanced".to_string()],
            metadata: ToolMetadata::default(),
            created_at: chrono::Utc::now() - chrono::Duration::days(120),
            updated_at: chrono::Utc::now() - chrono::Duration::days(14),
        },
        MarketplaceTool {
            id: "web_scraper_pro".to_string(),
            name: "Web Scraper Pro".to_string(),
            description: "专业网页抓取工具".to_string(),
            category: ToolCategory::Web,
            author: "DataHarvest".to_string(),
            version: "3.0.1".to_string(),
            rating: 4.2,
            download_count: 12100,
            tags: vec!["scraping".to_string(), "web".to_string(), "data".to_string()],
            metadata: ToolMetadata::default(),
            created_at: chrono::Utc::now() - chrono::Duration::days(60),
            updated_at: chrono::Utc::now() - chrono::Duration::days(3),
        },
        MarketplaceTool {
            id: "file_manager_plus".to_string(),
            name: "File Manager Plus".to_string(),
            description: "增强文件管理工具".to_string(),
            category: ToolCategory::System,
            author: "FileTools".to_string(),
            version: "1.8.0".to_string(),
            rating: 4.5,
            download_count: 6750,
            tags: vec!["file".to_string(), "management".to_string(), "system".to_string()],
            metadata: ToolMetadata::default(),
            created_at: chrono::Utc::now() - chrono::Duration::days(45),
            updated_at: chrono::Utc::now() - chrono::Duration::days(10),
        },
    ];

    for tool in tools {
        marketplace.publish_tool(tool)?;
    }

    // 设置推荐工具
    marketplace.featured_tools = vec![
        "weather_pro".to_string(),
        "calculator_advanced".to_string(),
    ];

    Ok(())
}

/// 从市场选择工具
fn select_tools_from_marketplace(marketplace: &ToolMarketplace, tool_ids: Vec<&str>) -> Result<Vec<Box<dyn Tool>>> {
    let mut tools = Vec::new();
    
    for tool_id in tool_ids {
        if marketplace.tools.contains_key(tool_id) {
            // 这里应该实际加载工具，现在使用模拟工具
            let tool: Box<dyn Tool> = match tool_id {
                "weather_pro" => weather_tool(),
                "calculator_advanced" => calculator(),
                "web_scraper_pro" => web_scraper(),
                "file_manager_plus" => file_reader(),
                _ => continue,
            };
            tools.push(tool);
        }
    }
    
    Ok(tools)
}

/// 获取工具推荐
fn get_tool_recommendations(marketplace: &ToolMarketplace, interests: &[&str]) -> Result<Vec<&MarketplaceTool>> {
    let mut recommendations: Vec<_> = marketplace.tools.values()
        .filter(|tool| {
            interests.iter().any(|interest| {
                tool.tags.iter().any(|tag| tag.contains(interest)) ||
                tool.description.contains(interest)
            })
        })
        .collect();
    
    // 按评分排序
    recommendations.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());
    
    Ok(recommendations)
}

/// 计算匹配分数
fn calculate_match_score(tool: &MarketplaceTool, interests: &[&str]) -> f64 {
    let mut score = 0.0;
    
    for interest in interests {
        if tool.tags.iter().any(|tag| tag.contains(interest)) {
            score += 1.0;
        }
        if tool.description.contains(interest) {
            score += 0.5;
        }
    }
    
    score
}

/// 工具管理器
#[derive(Debug)]
struct ToolManager {
    installed_tools: Vec<String>,
}

impl ToolManager {
    fn new() -> Self {
        Self {
            installed_tools: Vec::new(),
        }
    }
    
    async fn install_tool(&mut self, marketplace: &ToolMarketplace, tool_id: &str) -> Result<()> {
        if let Some(tool) = marketplace.tools.get(tool_id) {
            // 模拟安装过程
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            self.installed_tools.push(tool_id.to_string());
            info!("安装工具: {} v{}", tool.name, tool.version);
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("工具不存在: {}", tool_id))
        }
    }
    
    fn list_installed_tools(&self) -> &[String] {
        &self.installed_tools
    }
}

/// 市场统计
#[derive(Debug)]
struct MarketplaceStats {
    total_tools: usize,
    total_downloads: u64,
    average_rating: f64,
    active_developers: usize,
    category_counts: HashMap<ToolCategory, usize>,
}

/// 生成市场统计
fn generate_marketplace_stats(marketplace: &ToolMarketplace) -> MarketplaceStats {
    let total_tools = marketplace.tools.len();
    let total_downloads = marketplace.tools.values().map(|t| t.download_count).sum();
    let average_rating = marketplace.tools.values().map(|t| t.rating).sum::<f64>() / total_tools as f64;
    
    let mut developers = std::collections::HashSet::new();
    let mut category_counts = HashMap::new();
    
    for tool in marketplace.tools.values() {
        developers.insert(&tool.author);
        *category_counts.entry(tool.category.clone()).or_insert(0) += 1;
    }
    
    MarketplaceStats {
        total_tools,
        total_downloads,
        average_rating,
        active_developers: developers.len(),
        category_counts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_marketplace_creation() {
        let mut marketplace = ToolMarketplace::new();
        populate_marketplace(&mut marketplace).unwrap();
        
        assert_eq!(marketplace.tools.len(), 4);
        assert!(marketplace.tools.contains_key("weather_pro"));
    }

    #[test]
    fn test_tool_search() {
        let mut marketplace = ToolMarketplace::new();
        populate_marketplace(&mut marketplace).unwrap();
        
        let results = marketplace.search_tools("weather", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Weather Pro");
    }

    #[test]
    fn test_popular_tools() {
        let mut marketplace = ToolMarketplace::new();
        populate_marketplace(&mut marketplace).unwrap();
        
        let popular = marketplace.get_popular_tools(2);
        assert_eq!(popular.len(), 2);
        assert!(popular[0].download_count >= popular[1].download_count);
    }
}
