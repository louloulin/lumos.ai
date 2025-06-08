//! RAG 系统示例
//! 
//! 这个示例展示了如何使用 LumosAI 构建一个完整的 RAG（检索增强生成）系统。
//! 
//! 功能包括：
//! - 文档处理和分块
//! - 向量嵌入和存储
//! - 智能检索和重排序
//! - 上下文增强的生成
//! 
//! 运行方式:
//! ```bash
//! cargo run --example rag_system -- --help
//! ```

use anyhow::Result;
use clap::{Parser, Subcommand};
use lumosai_rag::prelude::*;
use lumosai_vector::memory::MemoryVectorStorage;
use lumosai_vector_core::{VectorStorage, IndexConfig, Document, SearchRequest, SearchQuery};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "rag_system")]
#[command(about = "LumosAI RAG 系统演示")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化 RAG 系统
    Init {
        /// 索引名称
        #[arg(short, long, default_value = "documents")]
        index: String,
        /// 向量维度
        #[arg(short, long, default_value = "384")]
        dimension: usize,
    },
    /// 添加文档到 RAG 系统
    Add {
        /// 文档文件路径
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// 直接输入文档内容
        #[arg(short, long)]
        text: Option<String>,
        /// 文档标题
        #[arg(short = 't', long)]
        title: Option<String>,
        /// 索引名称
        #[arg(short, long, default_value = "documents")]
        index: String,
    },
    /// 查询 RAG 系统
    Query {
        /// 查询问题
        question: String,
        /// 返回的文档数量
        #[arg(short, long, default_value = "5")]
        top_k: usize,
        /// 索引名称
        #[arg(short, long, default_value = "documents")]
        index: String,
        /// 是否显示检索到的文档
        #[arg(long)]
        show_docs: bool,
    },
    /// 显示系统统计信息
    Stats {
        /// 索引名称
        #[arg(short, long, default_value = "documents")]
        index: String,
    },
    /// 运行交互式查询
    Interactive {
        /// 索引名称
        #[arg(short, long, default_value = "documents")]
        index: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Commands::Init { index, dimension } => {
            init_rag_system(&index, dimension).await?;
        }
        Commands::Add { file, text, title, index } => {
            add_document(file, text, title, &index).await?;
        }
        Commands::Query { question, top_k, index, show_docs } => {
            query_rag_system(&question, top_k, &index, show_docs).await?;
        }
        Commands::Stats { index } => {
            show_stats(&index).await?;
        }
        Commands::Interactive { index } => {
            interactive_mode(&index).await?;
        }
    }
    
    Ok(())
}

async fn init_rag_system(index_name: &str, dimension: usize) -> Result<()> {
    println!("🚀 初始化 RAG 系统...");
    
    // 创建向量存储
    let storage = MemoryVectorStorage::new().await?;
    
    // 创建索引
    let config = IndexConfig::new(index_name, dimension);
    storage.create_index(config).await?;
    
    println!("✅ RAG 系统初始化完成");
    println!("   索引名称: {}", index_name);
    println!("   向量维度: {}", dimension);
    
    Ok(())
}

async fn add_document(
    file: Option<PathBuf>,
    text: Option<String>,
    title: Option<String>,
    index_name: &str,
) -> Result<()> {
    println!("📄 添加文档到 RAG 系统...");
    
    // 获取文档内容
    let content = if let Some(file_path) = file {
        std::fs::read_to_string(file_path)?
    } else if let Some(text_content) = text {
        text_content
    } else {
        anyhow::bail!("必须提供文件路径或文本内容");
    };
    
    // 创建向量存储
    let storage = MemoryVectorStorage::new().await?;
    
    // 创建文档处理器
    let chunker = TextChunker::new(ChunkingConfig {
        strategy: ChunkingStrategy::Recursive,
        chunk_size: 1000,
        chunk_overlap: 200,
        separators: vec!["\n\n".to_string(), "\n".to_string(), " ".to_string()],
        keep_separator: false,
        strip_whitespace: true,
        language: Some("chinese".to_string()),
    });
    
    // 分块处理
    let chunks = chunker.chunk_text(&content)?;
    println!("📝 文档分块完成，共 {} 个块", chunks.len());
    
    // 创建嵌入提供者（模拟）
    let embedding_provider = MockEmbeddingProvider::new(384);
    
    // 处理每个块
    let mut documents = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let doc_id = format!("{}_{}", title.as_deref().unwrap_or("doc"), i);
        let embedding = embedding_provider.embed_text(chunk).await?;
        
        let mut metadata = HashMap::new();
        metadata.insert("chunk_index".to_string(), lumosai_vector_core::MetadataValue::Integer(i as i64));
        metadata.insert("source".to_string(), lumosai_vector_core::MetadataValue::String(
            title.clone().unwrap_or_else(|| "unknown".to_string())
        ));
        metadata.insert("content_length".to_string(), lumosai_vector_core::MetadataValue::Integer(chunk.len() as i64));
        
        let document = Document {
            id: lumosai_vector_core::DocumentId::from(doc_id),
            content: chunk.clone(),
            embedding: Some(embedding),
            metadata,
        };
        
        documents.push(document);
    }
    
    // 存储文档
    let doc_ids = storage.upsert_documents(index_name, documents).await?;
    
    println!("✅ 文档添加完成");
    println!("   文档块数: {}", doc_ids.len());
    println!("   索引名称: {}", index_name);
    
    Ok(())
}

async fn query_rag_system(
    question: &str,
    top_k: usize,
    index_name: &str,
    show_docs: bool,
) -> Result<()> {
    println!("🔍 查询 RAG 系统...");
    println!("问题: {}", question);
    
    // 创建向量存储
    let storage = MemoryVectorStorage::new().await?;
    
    // 创建嵌入提供者
    let embedding_provider = MockEmbeddingProvider::new(384);
    
    // 生成查询向量
    let query_embedding = embedding_provider.embed_text(question).await?;
    
    // 执行搜索
    let search_request = SearchRequest {
        index_name: index_name.to_string(),
        query: SearchQuery::Vector(query_embedding),
        top_k,
        filter: None,
        include_metadata: true,
        include_vectors: false,
        options: HashMap::new(),
    };
    
    let search_results = storage.search(search_request).await?;
    
    println!("\n📊 搜索结果:");
    println!("找到 {} 个相关文档", search_results.results.len());
    
    if show_docs {
        println!("\n📄 检索到的文档:");
        for (i, result) in search_results.results.iter().enumerate() {
            println!("\n--- 文档 {} (相似度: {:.4}) ---", i + 1, result.score);
            println!("{}", result.document.content);
            
            if let Some(metadata) = &result.metadata {
                println!("\n元数据:");
                for (key, value) in metadata {
                    println!("  {}: {:?}", key, value);
                }
            }
        }
    }
    
    // 生成答案（模拟）
    let context = search_results.results
        .iter()
        .map(|r| r.document.content.clone())
        .collect::<Vec<_>>()
        .join("\n\n");
    
    let answer = generate_answer(question, &context).await?;
    
    println!("\n🤖 AI 回答:");
    println!("{}", answer);
    
    Ok(())
}

async fn show_stats(index_name: &str) -> Result<()> {
    println!("📊 RAG 系统统计信息");
    
    // 创建向量存储
    let storage = MemoryVectorStorage::new().await?;
    
    // 获取性能指标
    let metrics = storage.get_performance_metrics().await;
    let cache_stats = storage.get_cache_stats().await;
    
    println!("\n🎯 性能指标:");
    println!("  总操作数: {}", metrics.total_operations);
    println!("  成功操作数: {}", metrics.successful_operations);
    println!("  平均响应时间: {:?}", metrics.average_response_time);
    println!("  最小响应时间: {:?}", metrics.min_response_time);
    println!("  最大响应时间: {:?}", metrics.max_response_time);
    
    println!("\n💾 缓存统计:");
    println!("  总请求数: {}", cache_stats.total_requests);
    println!("  缓存命中数: {}", cache_stats.cache_hits);
    println!("  缓存未命中数: {}", cache_stats.cache_misses);
    println!("  命中率: {:.2}%", cache_stats.hit_rate * 100.0);
    println!("  当前缓存大小: {}", cache_stats.current_size);
    
    // 获取索引信息
    match storage.describe_index(index_name).await {
        Ok(info) => {
            println!("\n📋 索引信息:");
            println!("  索引名称: {}", info.name);
            println!("  向量维度: {}", info.dimension);
            println!("  文档数量: {}", info.document_count.unwrap_or(0));
        }
        Err(e) => {
            println!("\n⚠️ 无法获取索引信息: {}", e);
        }
    }
    
    Ok(())
}

async fn interactive_mode(index_name: &str) -> Result<()> {
    println!("🎮 进入交互式查询模式");
    println!("输入问题进行查询，输入 'quit' 退出\n");
    
    loop {
        print!("❓ 请输入问题: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input == "quit" || input == "exit" {
            println!("👋 再见！");
            break;
        }
        
        if input.starts_with('/') {
            match input {
                "/help" => {
                    println!("📖 可用命令:");
                    println!("  /help  - 显示帮助");
                    println!("  /stats - 显示统计信息");
                    println!("  quit   - 退出程序");
                }
                "/stats" => {
                    show_stats(index_name).await?;
                }
                _ => {
                    println!("❓ 未知命令，输入 /help 查看帮助");
                }
            }
            continue;
        }
        
        // 执行查询
        match query_rag_system(input, 3, index_name, false).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("❌ 查询失败: {}", e);
            }
        }
        
        println!(); // 空行分隔
    }
    
    Ok(())
}

// 模拟的嵌入提供者
struct MockEmbeddingProvider {
    dimension: usize,
}

impl MockEmbeddingProvider {
    fn new(dimension: usize) -> Self {
        Self { dimension }
    }
    
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        // 模拟嵌入生成（基于文本哈希）
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();
        
        let embedding = (0..self.dimension)
            .map(|i| {
                let mut h = DefaultHasher::new();
                (hash + i as u64).hash(&mut h);
                (h.finish() % 1000) as f32 / 1000.0
            })
            .collect();
        
        Ok(embedding)
    }
}

// 模拟的答案生成
async fn generate_answer(question: &str, context: &str) -> Result<String> {
    // 模拟 LLM 生成答案
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    
    let answer = format!(
        "基于提供的上下文信息，关于「{}」的回答是：\n\n根据检索到的相关文档，{}。\n\n这个回答是基于 {} 个字符的上下文信息生成的。",
        question,
        "这是一个模拟的AI回答，展示了RAG系统如何结合检索到的文档内容来生成更准确的答案",
        context.len()
    );
    
    Ok(answer)
}
