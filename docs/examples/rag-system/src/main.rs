use clap::{Parser, Subcommand};
use lumosai::prelude::SimpleAgent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "rag-system")]
#[command(about = "LumosAI RAG 系统示例 - 展示文档检索和知识问答")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 添加文档到知识库
    Add {
        /// 文档标题
        #[arg(short, long)]
        title: String,
        /// 文档内容
        #[arg(short, long)]
        content: String,
        /// 文档类型
        #[arg(long, default_value = "text")]
        doc_type: String,
    },
    /// 在知识库中搜索
    Search {
        /// 搜索查询
        query: String,
        /// 返回结果数量
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },
    /// 基于知识库回答问题
    Ask {
        /// 问题
        question: String,
        /// 使用的模型
        #[arg(short, long, default_value = "gpt-3.5-turbo")]
        model: String,
    },
    /// 交互式问答模式
    Interactive {
        /// 使用的模型
        #[arg(short, long, default_value = "gpt-3.5-turbo")]
        model: String,
    },
    /// 显示知识库统计信息
    Stats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Document {
    id: String,
    title: String,
    content: String,
    doc_type: String,
    created_at: DateTime<Utc>,
    embedding: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResult {
    document: Document,
    score: f32,
    relevance: String,
}

struct SimpleRAGSystem {
    documents: HashMap<String, Document>,
    agent: SimpleAgent,
}

impl SimpleRAGSystem {
    async fn new(model: &str) -> Result<Self> {
        let system_prompt = r#"
你是一个专业的知识问答助手，具备以下能力：
1. 基于提供的文档内容回答问题
2. 引用相关的文档片段
3. 承认知识边界，对不确定的信息说明
4. 提供结构化和准确的回答

当回答问题时，请：
- 优先使用提供的文档内容
- 明确标注信息来源
- 如果文档中没有相关信息，请诚实说明
- 提供简洁而全面的回答
"#;
        
        let agent = lumosai::agent::simple(model, system_prompt).await.map_err(|e| anyhow::anyhow!(e))?;
        
        Ok(Self {
            documents: HashMap::new(),
            agent,
        })
    }
    
    fn add_document(&mut self, title: String, content: String, doc_type: String) -> String {
        let id = Uuid::new_v4().to_string();
        let document = Document {
            id: id.clone(),
            title,
            content,
            doc_type,
            created_at: Utc::now(),
            embedding: None, // 在实际应用中，这里会生成真实的嵌入向量
        };
        
        self.documents.insert(id.clone(), document);
        println!("✅ 文档已添加到知识库，ID: {}", id);
        id
    }
    
    fn search_documents(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        // 简单的关键词匹配搜索（在实际应用中会使用向量相似度搜索）
        let query_lower = query.to_lowercase();
        let mut results: Vec<SearchResult> = self.documents
            .values()
            .filter_map(|doc| {
                let title_match = doc.title.to_lowercase().contains(&query_lower);
                let content_match = doc.content.to_lowercase().contains(&query_lower);
                
                if title_match || content_match {
                    let score = if title_match { 0.9 } else { 0.7 };
                    let relevance = if title_match { "高相关" } else { "中等相关" };
                    
                    Some(SearchResult {
                        document: doc.clone(),
                        score,
                        relevance: relevance.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();
        
        // 按相关性分数排序
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        results
    }
    
    async fn answer_question(&self, question: &str) -> Result<String> {
        // 搜索相关文档
        let search_results = self.search_documents(question, 3);
        
        if search_results.is_empty() {
            return Ok("抱歉，我在知识库中没有找到与您问题相关的信息。请尝试添加更多文档或重新表述您的问题。".to_string());
        }
        
        // 构建上下文
        let context = search_results
            .iter()
            .map(|result| {
                format!(
                    "文档标题：{}\n内容：{}\n相关性：{}\n",
                    result.document.title,
                    result.document.content,
                    result.relevance
                )
            })
            .collect::<Vec<_>>()
            .join("\n---\n");
        
        let prompt = format!(
            "基于以下文档内容回答问题：\n\n{}\n\n问题：{}\n\n请提供准确、有用的回答，并注明信息来源。",
            context, question
        );
        
        self.agent.chat(&prompt).await.map_err(|e: lumosai::Error| anyhow::anyhow!(e))
    }
    
    fn get_stats(&self) -> (usize, HashMap<String, usize>) {
        let total_docs = self.documents.len();
        let mut type_counts = HashMap::new();
        
        for doc in self.documents.values() {
            *type_counts.entry(doc.doc_type.clone()).or_insert(0) += 1;
        }
        
        (total_docs, type_counts)
    }
}

async fn add_document(rag: &mut SimpleRAGSystem, title: String, content: String, doc_type: String) -> Result<()> {
    rag.add_document(title, content, doc_type);
    Ok(())
}

async fn search_documents(rag: &SimpleRAGSystem, query: String, limit: usize) -> Result<()> {
    println!("🔍 搜索查询: {}", query);
    println!("📊 搜索结果:");
    
    let results = rag.search_documents(&query, limit);
    
    if results.is_empty() {
        println!("❌ 没有找到相关文档");
        return Ok(());
    }
    
    for (i, result) in results.iter().enumerate() {
        println!("\n{}. 📄 {}", i + 1, result.document.title);
        println!("   📅 创建时间: {}", result.document.created_at.format("%Y-%m-%d %H:%M:%S"));
        println!("   🏷️  文档类型: {}", result.document.doc_type);
        println!("   ⭐ 相关性: {} (分数: {:.2})", result.relevance, result.score);
        println!("   📝 内容预览: {}...", 
            if result.document.content.len() > 100 {
                &result.document.content[..100]
            } else {
                &result.document.content
            }
        );
    }
    
    Ok(())
}

async fn ask_question(rag: &SimpleRAGSystem, question: String) -> Result<()> {
    println!("❓ 问题: {}", question);
    println!("🤖 正在分析知识库并生成回答...\n");
    
    match rag.answer_question(&question).await {
        Ok(answer) => {
            println!("💡 回答:\n{}", answer);
        }
        Err(e) => {
            println!("❌ 回答生成失败: {}", e);
        }
    }
    
    Ok(())
}

async fn interactive_mode(rag: &mut SimpleRAGSystem) -> Result<()> {
    println!("🎯 进入交互式问答模式");
    println!("💡 提示：输入 'quit' 或 'exit' 退出，'help' 查看帮助");
    println!("📚 当前知识库包含 {} 个文档\n", rag.documents.len());
    
    loop {
        print!("❓ 请输入问题: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        match input.to_lowercase().as_str() {
            "quit" | "exit" => {
                println!("👋 再见！");
                break;
            }
            "help" => {
                println!("📖 可用命令:");
                println!("  - 直接输入问题进行问答");
                println!("  - 'stats' - 显示知识库统计");
                println!("  - 'help' - 显示此帮助");
                println!("  - 'quit' 或 'exit' - 退出程序");
                continue;
            }
            "stats" => {
                let (total, type_counts) = rag.get_stats();
                println!("📊 知识库统计:");
                println!("  总文档数: {}", total);
                for (doc_type, count) in type_counts {
                    println!("  {} 类型: {} 个", doc_type, count);
                }
                continue;
            }
            "" => continue,
            _ => {}
        }
        
        println!("🤖 正在分析...");
        match rag.answer_question(input).await {
            Ok(answer) => {
                println!("💡 回答:\n{}\n", answer);
            }
            Err(e) => {
                println!("❌ 回答生成失败: {}\n", e);
            }
        }
    }
    
    Ok(())
}

fn show_stats(rag: &SimpleRAGSystem) {
    let (total_docs, type_counts) = rag.get_stats();
    
    println!("📊 知识库统计信息:");
    println!("📚 总文档数: {}", total_docs);
    
    if !type_counts.is_empty() {
        println!("📋 按类型分布:");
        for (doc_type, count) in type_counts {
            println!("  🏷️  {}: {} 个文档", doc_type, count);
        }
    }
    
    if total_docs > 0 {
        println!("\n📄 文档列表:");
        for (i, doc) in rag.documents.values().enumerate() {
            println!("{}. {} ({})", i + 1, doc.title, doc.doc_type);
        }
    } else {
        println!("\n💡 提示: 知识库为空，请先添加一些文档");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    // 初始化 RAG 系统
    let mut rag = match &cli.command {
        Commands::Add { .. } | Commands::Search { .. } | Commands::Stats => {
            SimpleRAGSystem::new("gpt-3.5-turbo").await?
        }
        Commands::Ask { model, .. } | Commands::Interactive { model } => {
            SimpleRAGSystem::new(model).await?
        }
    };
    
    // 添加一些示例文档
    rag.add_document(
        "LumosAI 简介".to_string(),
        "LumosAI 是一个企业级AI应用开发框架，使用 Rust 作为核心，支持 RAG 系统、多 Agent 协作、工作流编排等功能。".to_string(),
        "介绍".to_string(),
    );
    
    rag.add_document(
        "RAG 系统原理".to_string(),
        "RAG（Retrieval-Augmented Generation）是一种结合信息检索和生成式AI的技术，通过检索相关文档来增强语言模型的回答质量。".to_string(),
        "技术".to_string(),
    );
    
    rag.add_document(
        "Rust 编程语言".to_string(),
        "Rust 是一种系统编程语言，注重安全、速度和并发。它通过所有权系统防止内存安全问题，同时提供零成本抽象。".to_string(),
        "编程".to_string(),
    );
    
    match cli.command {
        Commands::Add { title, content, doc_type } => {
            add_document(&mut rag, title, content, doc_type).await?;
        }
        Commands::Search { query, limit } => {
            search_documents(&rag, query, limit).await?;
        }
        Commands::Ask { question, .. } => {
            ask_question(&rag, question).await?;
        }
        Commands::Interactive { .. } => {
            interactive_mode(&mut rag).await?;
        }
        Commands::Stats => {
            show_stats(&rag);
        }
    }
    
    Ok(())
}
