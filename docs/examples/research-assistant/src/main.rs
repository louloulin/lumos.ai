use anyhow::Result;
use clap::{Parser, Subcommand};
use lumosai::prelude::*;
use reqwest;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;
use url::Url;

#[derive(Parser)]
#[command(name = "research-assistant")]
#[command(about = "LumosAI 研究助手示例 - 带工具的智能研究助手")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 研究特定主题
    Research {
        /// 研究主题
        topic: String,
        /// 使用的模型
        #[arg(long, default_value = "gpt-3.5-turbo")]
        model: String,
        /// 搜索深度 (1-5)
        #[arg(long, default_value = "3")]
        depth: u8,
    },
    /// 分析网页内容
    Analyze {
        /// 网页URL
        url: String,
        /// 分析问题
        question: String,
        /// 使用的模型
        #[arg(long, default_value = "gpt-3.5-turbo")]
        model: String,
    },
    /// 交互式研究模式
    Interactive {
        /// 使用的模型
        #[arg(long, default_value = "gpt-3.5-turbo")]
        model: String,
    },
}

// 网页搜索工具
#[derive(Debug)]
struct WebSearchTool {
    client: reqwest::Client,
}

impl WebSearchTool {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    async fn search(&self, query: &str, num_results: usize) -> Result<Vec<SearchResult>> {
        // 模拟搜索结果（实际应用中可以集成真实的搜索API）
        let mock_results = vec![
            SearchResult {
                title: format!("关于 {} 的详细介绍", query),
                url: "https://example.com/article1".to_string(),
                snippet: format!("{} 是一个重要的概念，具有广泛的应用...", query),
            },
            SearchResult {
                title: format!("{} 的最新研究进展", query),
                url: "https://example.com/research".to_string(),
                snippet: format!("最新研究表明，{} 在多个领域都有突破性进展...", query),
            },
            SearchResult {
                title: format!("{} 实践指南", query),
                url: "https://example.com/guide".to_string(),
                snippet: format!("本指南详细介绍了如何在实际项目中应用 {}...", query),
            },
        ];

        Ok(mock_results.into_iter().take(num_results).collect())
    }
}

// 网页内容提取工具
#[derive(Debug)]
struct WebScrapingTool {
    client: reqwest::Client,
}

impl WebScrapingTool {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    async fn extract_content(&self, url: &str) -> Result<WebContent> {
        // 模拟网页内容提取（实际应用中会真实抓取网页）
        let mock_content = WebContent {
            url: url.to_string(),
            title: "示例网页标题".to_string(),
            content: format!(
                "这是从 {} 提取的示例内容。\n\n在实际应用中，这里会包含真实的网页文本内容，\
                包括文章正文、重要段落和关键信息。\n\n内容会经过清理和格式化，\
                去除HTML标签和无关元素，保留有价值的文本信息。",
                url
            ),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("word_count".to_string(), "150".to_string());
                meta.insert("language".to_string(), "zh-CN".to_string());
                meta.insert("extracted_at".to_string(), chrono::Utc::now().to_rfc3339());
                meta
            },
        };

        Ok(mock_content)
    }
}

// 数据结构
#[derive(Debug, Clone)]
struct SearchResult {
    title: String,
    url: String,
    snippet: String,
}

#[derive(Debug)]
struct WebContent {
    url: String,
    title: String,
    content: String,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
struct ResearchReport {
    topic: String,
    summary: String,
    key_findings: Vec<String>,
    sources: Vec<SearchResult>,
    recommendations: Vec<String>,
}

async fn create_research_agent(model: &str) -> Result<SimpleAgent> {
    let system_prompt = r#"
你是一个专业的研究助手，具备以下能力：

1. **信息搜索**: 能够搜索和收集相关信息
2. **内容分析**: 深入分析网页内容和文档
3. **综合总结**: 将多个信息源整合成连贯的报告
4. **批判性思维**: 评估信息的可靠性和相关性

工作流程：
- 理解用户的研究需求
- 搜索相关信息源
- 分析和提取关键信息
- 综合形成结构化的研究报告

请用中文回答，保持专业和客观的语调。
"#;

    lumosai::agent::simple(model, system_prompt).await.map_err(|e| anyhow::anyhow!(e))
}

async fn research_topic(topic: &str, model: &str, depth: u8) -> Result<()> {
    println!("🔍 开始研究主题: {}", topic);
    println!("📊 搜索深度: {}", depth);
    println!("{}", "=".repeat(50));

    // 创建研究助手
    let agent = create_research_agent(model).await?;
    
    // 创建工具
    let search_tool = WebSearchTool::new();
    let scraping_tool = WebScrapingTool::new();

    // 第一步：搜索相关信息
    println!("📚 正在搜索相关信息...");
    let search_results = search_tool.search(topic, depth as usize * 2).await?;
    
    // 第二步：提取网页内容
    println!("📄 正在分析网页内容...");
    let mut web_contents = Vec::new();
    for result in &search_results {
        if let Ok(content) = scraping_tool.extract_content(&result.url).await {
            web_contents.push(content);
        }
    }

    // 第三步：生成研究报告
    println!("📝 正在生成研究报告...");
    
    let research_context = format!(
        "请基于以下信息为主题 '{}' 生成一份详细的研究报告：\n\n搜索结果：\n{}\n\n网页内容：\n{}",
        topic,
        search_results.iter()
            .map(|r| format!("- {}: {}", r.title, r.snippet))
            .collect::<Vec<_>>()
            .join("\n"),
        web_contents.iter()
            .map(|c| format!("来源 {}: {}", c.title, c.content))
            .collect::<Vec<_>>()
            .join("\n\n")
    );

    let report = agent.chat(&research_context).await?;

    // 显示结果
    println!("\n📋 研究报告");
    println!("{}", "=".repeat(50));
    println!("{}", report);
    
    println!("\n📚 信息来源");
    println!("{}", "-".repeat(30));
    for (i, result) in search_results.iter().enumerate() {
        println!("{}. {}", i + 1, result.title);
        println!("   URL: {}", result.url);
        println!("   摘要: {}", result.snippet);
        println!();
    }

    Ok(())
}

async fn analyze_webpage(url: &str, question: &str, model: &str) -> Result<()> {
    println!("🌐 分析网页: {}", url);
    println!("❓ 分析问题: {}", question);
    println!("{}", "=".repeat(50));

    // 创建研究助手
    let agent = create_research_agent(model).await?;
    
    // 创建工具
    let scraping_tool = WebScrapingTool::new();

    // 提取网页内容
    println!("📄 正在提取网页内容...");
    let content = scraping_tool.extract_content(url).await?;

    // 分析内容
    println!("🔍 正在分析内容...");
    let analysis_prompt = format!(
        "请分析以下网页内容，并回答问题：'{}'\n\n网页标题：{}\n网页URL：{}\n\n内容：\n{}",
        question, content.title, content.url, content.content
    );

    let analysis = agent.chat(&analysis_prompt).await?;

    // 显示结果
    println!("\n📊 分析结果");
    println!("{}", "=".repeat(50));
    println!("{}", analysis);

    println!("\n📋 网页信息");
    println!("{}", "-".repeat(30));
    println!("标题: {}", content.title);
    println!("URL: {}", content.url);
    println!("字数: {}", content.metadata.get("word_count").unwrap_or(&"未知".to_string()));
    println!("语言: {}", content.metadata.get("language").unwrap_or(&"未知".to_string()));

    Ok(())
}

async fn interactive_research(model: &str) -> Result<()> {
    println!("🤖 LumosAI 交互式研究助手");
    println!("输入 'quit' 或 'exit' 退出，'help' 查看帮助");
    println!("{}", "=".repeat(50));

    // 创建研究助手
    let agent = create_research_agent(model).await?;
    let search_tool = WebSearchTool::new();
    let scraping_tool = WebScrapingTool::new();

    loop {
        print!("\n🔍 请输入研究问题: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input.to_lowercase().as_str() {
            "quit" | "exit" => {
                println!("👋 再见！");
                break;
            }
            "help" => {
                println!("\n📖 帮助信息:");
                println!("- 直接输入研究问题进行研究");
                println!("- 输入 'search: 关键词' 进行搜索");
                println!("- 输入 'analyze: URL' 分析网页");
                println!("- 输入 'quit' 或 'exit' 退出");
                continue;
            }
            _ if input.starts_with("search:") => {
                let query = input.strip_prefix("search:").unwrap().trim();
                println!("🔍 搜索: {}", query);
                
                match search_tool.search(query, 5).await {
                    Ok(results) => {
                        println!("\n📚 搜索结果:");
                        for (i, result) in results.iter().enumerate() {
                            println!("{}. {}", i + 1, result.title);
                            println!("   {}", result.snippet);
                            println!("   URL: {}", result.url);
                            println!();
                        }
                    }
                    Err(e) => println!("❌ 搜索失败: {}", e),
                }
                continue;
            }
            _ if input.starts_with("analyze:") => {
                let url = input.strip_prefix("analyze:").unwrap().trim();
                println!("🌐 分析网页: {}", url);
                
                match scraping_tool.extract_content(url).await {
                    Ok(content) => {
                        println!("\n📄 网页内容:");
                        println!("标题: {}", content.title);
                        println!("内容预览: {}...", 
                            content.content.chars().take(200).collect::<String>());
                        
                        let summary_prompt = format!(
                            "请总结以下网页的主要内容：\n标题：{}\n内容：{}",
                            content.title, content.content
                        );
                        
                        match agent.chat(&summary_prompt).await {
                            Ok(summary) => {
                                println!("\n📊 内容总结:");
                                println!("{}", summary);
                            }
                            Err(e) => println!("❌ 分析失败: {}", e),
                        }
                    }
                    Err(e) => println!("❌ 网页提取失败: {}", e),
                }
                continue;
            }
            _ => {
                if input.is_empty() {
                    continue;
                }
                
                println!("🤔 正在思考...");
                
                // 先搜索相关信息
                match search_tool.search(input, 3).await {
                    Ok(search_results) => {
                        let context = format!(
                            "用户问题：{}\n\n相关搜索结果：\n{}",
                            input,
                            search_results.iter()
                                .map(|r| format!("- {}: {}", r.title, r.snippet))
                                .collect::<Vec<_>>()
                                .join("\n")
                        );
                        
                        match agent.chat(&context).await {
                            Ok(response) => {
                                println!("\n🤖 研究助手:");
                                println!("{}", response);
                                
                                if !search_results.is_empty() {
                                    println!("\n📚 参考来源:");
                                    for result in search_results.iter().take(3) {
                                        println!("- {}", result.title);
                                    }
                                }
                            }
                            Err(e) => println!("❌ 回答生成失败: {}", e),
                        }
                    }
                    Err(e) => {
                        // 如果搜索失败，直接回答
                        match agent.chat(input).await {
                            Ok(response) => {
                                println!("\n🤖 研究助手:");
                                println!("{}", response);
                            }
                            Err(e) => println!("❌ 回答生成失败: {}", e),
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Research { topic, model, depth }) => {
            research_topic(&topic, &model, depth).await?;
        }
        Some(Commands::Analyze { url, question, model }) => {
            analyze_webpage(&url, &question, &model).await?;
        }
        Some(Commands::Interactive { model }) => {
            interactive_research(&model).await?;
        }
        None => {
            // 默认进入交互模式
            interactive_research("gpt-3.5-turbo").await?;
        }
    }

    Ok(())
}
