//! 工具市场完整演示
//! 
//! 这个示例展示了Lumos.ai工具市场的完整功能，包括：
//! - 工具包注册和发布
//! - 搜索和发现
//! - 评分和下载
//! - 安全扫描和验证
//! - 分析和统计

use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;
use semver::Version;

use lumosai_marketplace::{
    ToolMarketplace, MarketplaceBuilder,
    models::*,
    search::SearchQuery,
    discovery::UserContext,
    analytics::{UserInfo, UserType, UsageInfo, TimeRange},
    publisher::{PublishRequest, PublisherInfo},
    validator::ValidationResult,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::init();
    
    println!("🚀 Lumos.ai 工具市场完整演示");
    println!("=====================================");
    
    // 1. 创建工具市场
    println!("\n📦 1. 初始化工具市场...");
    let marketplace = MarketplaceBuilder::new()
        .database_url("sqlite://marketplace_demo.db")
        .search_index_path("./demo_search_index")
        .enable_security_scanning(true)
        .enable_analytics(true)
        .build()
        .await?;
    
    println!("   ✅ 工具市场初始化完成");
    
    // 2. 创建示例工具包
    println!("\n🔧 2. 创建示例工具包...");
    let demo_packages = create_demo_packages();
    
    for (i, package) in demo_packages.iter().enumerate() {
        println!("   创建工具包 {}: {}", i + 1, package.name);
        
        // 验证工具包
        let validation_result = marketplace.validate_package(package).await?;
        println!("     验证结果: {} (分数: {})", 
            if validation_result.passed { "✅ 通过" } else { "❌ 失败" },
            validation_result.score
        );
        
        // 安全扫描
        let security_result = marketplace.scan_package_security(package).await?;
        println!("     安全扫描: {:?} (分数: {})", 
            security_result.security_level,
            security_result.score
        );
        
        // 注册工具包
        marketplace.registry().register_package(package.clone()).await?;
        println!("     ✅ 工具包注册成功");
    }
    
    // 3. 搜索和发现功能演示
    println!("\n🔍 3. 搜索和发现功能演示...");
    
    // 基础搜索
    let search_results = marketplace.search("web").await?;
    println!("   搜索 'web' 找到 {} 个结果", search_results.len());
    for result in &search_results {
        println!("     - {}: {} (分数: {:.2})", 
            result.package.name, 
            result.match_reason,
            result.relevance_score
        );
    }
    
    // 高级搜索
    let advanced_query = SearchQuery {
        text: "data".to_string(),
        categories: vec![ToolCategory::Data],
        published_only: true,
        limit: 5,
        ..Default::default()
    };
    
    let advanced_results = marketplace.advanced_search(&advanced_query).await?;
    println!("   高级搜索找到 {} 个数据处理工具", advanced_results.len());
    
    // 按分类搜索
    let web_tools = marketplace.get_by_category(&ToolCategory::Web).await?;
    println!("   网络工具分类包含 {} 个工具", web_tools.len());
    
    // 4. 热门和推荐功能
    println!("\n⭐ 4. 热门和推荐功能演示...");
    
    // 获取热门工具
    let trending = marketplace.get_trending(None, 5).await?;
    println!("   热门工具 ({} 个):", trending.len());
    for tool in &trending {
        println!("     - {}: 下载 {} 次, 评分 {:.1}", 
            tool.package.name,
            tool.package.download_count,
            tool.package.rating
        );
    }
    
    // 获取最新工具
    let recent = marketplace.get_recent(3).await?;
    println!("   最新工具 ({} 个):", recent.len());
    for tool in &recent {
        println!("     - {}: 发布于 {}", 
            tool.package.name,
            tool.package.created_at.format("%Y-%m-%d")
        );
    }
    
    // 个性化推荐
    let user_context = UserContext {
        user_id: Some("demo_user".to_string()),
        preferred_categories: vec![ToolCategory::Web, ToolCategory::Data],
        used_tools: vec!["web_scraper".to_string()],
        search_history: vec!["api".to_string(), "json".to_string()],
        rating_history: HashMap::new(),
    };
    
    let recommendations = marketplace.get_recommendations(&user_context).await?;
    println!("   个性化推荐 ({} 个):", recommendations.len());
    for rec in &recommendations {
        println!("     - {}: {} (分数: {:.2})", 
            rec.package.name,
            rec.recommendation_reason.as_deref().unwrap_or("推荐"),
            rec.relevance_score
        );
    }
    
    // 5. 下载和评分演示
    println!("\n📥 5. 下载和评分演示...");
    
    if let Some(first_package) = demo_packages.first() {
        let user_info = UserInfo {
            user_id: Some("demo_user".to_string()),
            user_type: UserType::Individual,
            location: Some("CN".to_string()),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Demo Client 1.0".to_string()),
        };
        
        // 模拟下载
        println!("   模拟下载工具包: {}", first_package.name);
        let download_result = marketplace.download_package(first_package.id, &user_info).await;
        match download_result {
            Ok(download_path) => println!("     ✅ 下载成功: {}", download_path),
            Err(e) => println!("     ❌ 下载失败: {}", e),
        }
        
        // 模拟评分
        println!("   模拟评分工具包: {}", first_package.name);
        let rating_result = marketplace.rate_package(first_package.id, 4.5, &user_info).await;
        match rating_result {
            Ok(()) => println!("     ✅ 评分成功: 4.5 星"),
            Err(e) => println!("     ❌ 评分失败: {}", e),
        }
        
        // 记录使用情况
        let usage_info = UsageInfo {
            user_info: user_info.clone(),
            duration_seconds: 300, // 5分钟
            start_time: Utc::now() - chrono::Duration::minutes(5),
            end_time: Utc::now(),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("session_id".to_string(), "demo_session_123".to_string());
                ctx.insert("feature_used".to_string(), "web_scraping".to_string());
                ctx
            },
        };
        
        marketplace.record_usage(first_package.id, &usage_info).await?;
        println!("     ✅ 使用记录已保存");
    }
    
    // 6. 统计和分析
    println!("\n📊 6. 统计和分析演示...");
    
    // 市场总体统计
    let marketplace_stats = marketplace.get_marketplace_statistics().await?;
    println!("   市场统计:");
    println!("     总工具包数: {}", marketplace_stats.total_packages);
    println!("     已发布工具包: {}", marketplace_stats.published_packages);
    println!("     总下载次数: {}", marketplace_stats.total_downloads);
    println!("     平均评分: {:.2}", marketplace_stats.average_rating);
    
    // 工具包详细统计
    if let Some(first_package) = demo_packages.first() {
        let tool_stats = marketplace.get_tool_statistics(first_package.id).await?;
        println!("   工具包 '{}' 统计:", first_package.name);
        println!("     总下载: {}", tool_stats.total_downloads);
        println!("     日下载: {}", tool_stats.daily_downloads);
        println!("     平均评分: {:.2} ({} 个评分)", 
            tool_stats.average_rating, 
            tool_stats.rating_count
        );
        println!("     平均使用时长: {:.1} 秒", 
            tool_stats.usage_duration_stats.average_duration_seconds
        );
    }
    
    // 生成分析报告
    let time_range = TimeRange {
        start: Utc::now() - chrono::Duration::days(30),
        end: Utc::now(),
    };
    
    let analytics_report = marketplace.generate_analytics_report(time_range).await?;
    println!("   分析报告 (最近30天):");
    println!("     生成时间: {}", analytics_report.generated_at.format("%Y-%m-%d %H:%M:%S"));
    println!("     总工具数: {}", analytics_report.overall_stats.total_tools);
    println!("     活跃用户: {}", analytics_report.overall_stats.active_users);
    
    // 7. 相似工具推荐
    println!("\n🔗 7. 相似工具推荐演示...");
    
    if let Some(first_package) = demo_packages.first() {
        let similar_tools = marketplace.get_similar(first_package.id, 3).await?;
        println!("   与 '{}' 相似的工具 ({} 个):", first_package.name, similar_tools.len());
        for similar in &similar_tools {
            println!("     - {}: {}", 
                similar.package.name,
                similar.recommendation_reason.as_deref().unwrap_or("相似工具")
            );
        }
    }
    
    // 8. 工具包管理演示
    println!("\n🛠️ 8. 工具包管理演示...");
    
    // 列出所有工具包
    let all_packages = marketplace.list_packages(0, 10).await?;
    println!("   所有工具包 ({} 个):", all_packages.len());
    for package in &all_packages {
        println!("     - {}: {} (版本 {})", 
            package.name,
            package.description,
            package.version
        );
    }
    
    // 按名称和版本获取工具包
    if let Some(first_package) = demo_packages.first() {
        let found_package = marketplace.get_package_by_name_version(
            &first_package.name, 
            &first_package.version.to_string()
        ).await?;
        
        match found_package {
            Some(pkg) => println!("   ✅ 按名称版本找到工具包: {}", pkg.name),
            None => println!("   ❌ 未找到指定的工具包"),
        }
    }
    
    println!("\n✅ 工具市场演示完成!");
    println!("=====================================");
    println!("主要功能演示:");
    println!("✓ 工具包注册和验证");
    println!("✓ 安全扫描和评估");
    println!("✓ 搜索和发现引擎");
    println!("✓ 个性化推荐系统");
    println!("✓ 下载和评分功能");
    println!("✓ 使用分析和统计");
    println!("✓ 相似工具推荐");
    println!("✓ 工具包管理功能");
    
    Ok(())
}

/// 创建演示用的工具包
fn create_demo_packages() -> Vec<ToolPackage> {
    vec![
        create_web_scraper_package(),
        create_json_parser_package(),
        create_file_manager_package(),
        create_calculator_package(),
        create_data_analyzer_package(),
    ]
}

/// 创建网络爬虫工具包
fn create_web_scraper_package() -> ToolPackage {
    ToolPackage {
        id: Uuid::new_v4(),
        name: "web_scraper".to_string(),
        version: Version::new(1, 2, 0),
        description: "强大的网络爬虫工具，支持多种网站抓取和数据提取功能".to_string(),
        author: "WebTools Team".to_string(),
        author_email: Some("team@webtools.com".to_string()),
        license: "MIT".to_string(),
        homepage: Some("https://github.com/webtools/scraper".to_string()),
        repository: Some("https://github.com/webtools/scraper.git".to_string()),
        keywords: vec!["web".to_string(), "scraping".to_string(), "crawler".to_string(), "html".to_string()],
        categories: vec![ToolCategory::Web, ToolCategory::Data],
        dependencies: {
            let mut deps = HashMap::new();
            deps.insert("reqwest".to_string(), "0.11.0".to_string());
            deps.insert("scraper".to_string(), "0.17.0".to_string());
            deps.insert("tokio".to_string(), "1.0.0".to_string());
            deps
        },
        lumos_version: "0.1.0".to_string(),
        manifest: ToolManifest {
            tools: vec![
                ToolDefinition {
                    name: "scrape_url".to_string(),
                    description: "抓取指定URL的内容".to_string(),
                    parameters: vec![
                        ParameterDefinition {
                            name: "url".to_string(),
                            description: "要抓取的URL".to_string(),
                            r#type: "string".to_string(),
                            required: true,
                            default: None,
                            validation: Some(ValidationRule {
                                pattern: Some(r"^https?://".to_string()),
                                ..Default::default()
                            }),
                            examples: vec![serde_json::json!("https://example.com")],
                        },
                        ParameterDefinition {
                            name: "selector".to_string(),
                            description: "CSS选择器".to_string(),
                            r#type: "string".to_string(),
                            required: false,
                            default: Some(serde_json::json!("body")),
                            validation: None,
                            examples: vec![serde_json::json!("div.content")],
                        },
                    ],
                    returns: ReturnDefinition {
                        r#type: "object".to_string(),
                        description: "抓取结果".to_string(),
                        schema: Some(serde_json::json!({
                            "type": "object",
                            "properties": {
                                "content": {"type": "string"},
                                "status": {"type": "integer"},
                                "headers": {"type": "object"}
                            }
                        })),
                        examples: vec![serde_json::json!({
                            "content": "<html>...</html>",
                            "status": 200,
                            "headers": {"content-type": "text/html"}
                        })],
                    },
                    examples: vec![
                        ToolExample {
                            title: "抓取网页标题".to_string(),
                            description: "抓取网页的标题内容".to_string(),
                            input: serde_json::json!({
                                "url": "https://example.com",
                                "selector": "title"
                            }),
                            output: serde_json::json!({
                                "content": "Example Domain",
                                "status": 200
                            }),
                            code: Some("scraper.scrape_url(url=\"https://example.com\", selector=\"title\")".to_string()),
                        }
                    ],
                    tags: vec!["web".to_string(), "http".to_string()],
                    async_tool: true,
                    requires_auth: false,
                    permissions: vec![Permission::Network],
                }
            ],
            entry_point: "src/lib.rs".to_string(),
            exports: vec!["scrape_url".to_string()],
            permissions: vec![Permission::Network],
            config_schema: None,
            rust_version: Some("1.70.0".to_string()),
            build_script: None,
        },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("performance_tier".to_string(), serde_json::json!("high"));
            meta.insert("memory_usage".to_string(), serde_json::json!("medium"));
            meta.insert("cpu_intensive".to_string(), serde_json::json!(false));
            meta
        },
        created_at: Utc::now() - chrono::Duration::days(30),
        updated_at: Utc::now() - chrono::Duration::days(5),
        published_at: Some(Utc::now() - chrono::Duration::days(25)),
        download_count: 1250,
        rating: 4.6,
        rating_count: 89,
        published: true,
        verified: true,
        security_audit: None,
        performance_benchmark: None,
    }
}

/// 创建JSON解析器工具包
fn create_json_parser_package() -> ToolPackage {
    ToolPackage {
        id: Uuid::new_v4(),
        name: "json_parser".to_string(),
        version: Version::new(2, 1, 3),
        description: "高性能JSON解析和处理工具，支持复杂数据结构操作".to_string(),
        author: "DataTools Inc".to_string(),
        author_email: Some("support@datatools.com".to_string()),
        license: "Apache-2.0".to_string(),
        homepage: Some("https://datatools.com/json-parser".to_string()),
        repository: Some("https://github.com/datatools/json-parser.git".to_string()),
        keywords: vec!["json".to_string(), "parser".to_string(), "data".to_string(), "api".to_string()],
        categories: vec![ToolCategory::Data, ToolCategory::API],
        dependencies: {
            let mut deps = HashMap::new();
            deps.insert("serde_json".to_string(), "1.0.0".to_string());
            deps.insert("serde".to_string(), "1.0.0".to_string());
            deps
        },
        lumos_version: "0.1.0".to_string(),
        manifest: ToolManifest {
            tools: vec![
                ToolDefinition {
                    name: "parse_json".to_string(),
                    description: "解析JSON字符串".to_string(),
                    parameters: vec![
                        ParameterDefinition {
                            name: "json_string".to_string(),
                            description: "要解析的JSON字符串".to_string(),
                            r#type: "string".to_string(),
                            required: true,
                            default: None,
                            validation: None,
                            examples: vec![serde_json::json!("{\"name\": \"test\"}")],
                        }
                    ],
                    returns: ReturnDefinition {
                        r#type: "object".to_string(),
                        description: "解析后的JSON对象".to_string(),
                        schema: None,
                        examples: vec![serde_json::json!({"name": "test"})],
                    },
                    examples: vec![],
                    tags: vec!["json".to_string(), "parse".to_string()],
                    async_tool: false,
                    requires_auth: false,
                    permissions: vec![],
                }
            ],
            entry_point: "src/lib.rs".to_string(),
            exports: vec!["parse_json".to_string()],
            permissions: vec![],
            config_schema: None,
            rust_version: Some("1.70.0".to_string()),
            build_script: None,
        },
        metadata: HashMap::new(),
        created_at: Utc::now() - chrono::Duration::days(60),
        updated_at: Utc::now() - chrono::Duration::days(10),
        published_at: Some(Utc::now() - chrono::Duration::days(55)),
        download_count: 2340,
        rating: 4.8,
        rating_count: 156,
        published: true,
        verified: true,
        security_audit: None,
        performance_benchmark: None,
    }
}

// 简化其他工具包的创建函数
fn create_file_manager_package() -> ToolPackage {
    let mut package = create_json_parser_package();
    package.id = Uuid::new_v4();
    package.name = "file_manager".to_string();
    package.description = "文件管理工具，支持文件操作和目录管理".to_string();
    package.categories = vec![ToolCategory::File, ToolCategory::System];
    package.keywords = vec!["file".to_string(), "directory".to_string(), "management".to_string()];
    package.download_count = 890;
    package.rating = 4.3;
    package.rating_count = 67;
    package
}

fn create_calculator_package() -> ToolPackage {
    let mut package = create_json_parser_package();
    package.id = Uuid::new_v4();
    package.name = "calculator".to_string();
    package.description = "高精度数学计算工具，支持复杂数学运算".to_string();
    package.categories = vec![ToolCategory::Math, ToolCategory::Utility];
    package.keywords = vec!["math".to_string(), "calculation".to_string(), "arithmetic".to_string()];
    package.download_count = 1560;
    package.rating = 4.7;
    package.rating_count = 203;
    package
}

fn create_data_analyzer_package() -> ToolPackage {
    let mut package = create_json_parser_package();
    package.id = Uuid::new_v4();
    package.name = "data_analyzer".to_string();
    package.description = "数据分析工具，支持统计分析和数据可视化".to_string();
    package.categories = vec![ToolCategory::Data, ToolCategory::AI];
    package.keywords = vec!["data".to_string(), "analysis".to_string(), "statistics".to_string(), "visualization".to_string()];
    package.download_count = 720;
    package.rating = 4.4;
    package.rating_count = 45;
    package
}

impl Default for ValidationRule {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
            custom_validator: None,
        }
    }
}
