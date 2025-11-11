# 🧠 RAG系统入门

**45分钟构建智能问答系统，掌握检索增强生成的核心概念**

## 🎯 学习目标

完成本教程后，你将能够：
- ✅ 理解RAG（检索增强生成）的原理和价值
- ✅ 创建和管理向量知识库
- ✅ 实现智能文档问答系统
- ✅ 掌握RAG系统的优化技巧

## ⏱️ 预计时间: 45分钟
## 📋 前置要求: Agent基础概念

---

## 🤔 什么是RAG？

### RAG定义

**RAG**（Retrieval-Augmented Generation，检索增强生成）是一种结合了信息检索和文本生成的AI系统架构。

```rust
// RAG系统的工作流程
用户提问 → 向量检索 → 相关文档 → 上下文增强 → 生成回答
    ↓           ↓          ↓          ↓           ↓
  "什么是AI?" → 语义搜索  → AI文档   → "基于以下信息..." → "AI是..."
```

### 为什么需要RAG？

**传统LLM的局限**:
- ❌ 知识截止日期（无法获取最新信息）
- ❌ 幻觉问题（生成不准确的内容）
- ❌ 缺乏具体领域知识
- ❌ 无法引用信息来源

**RAG的优势**:
- ✅ 实时知识更新
- ✅ 减少幻觉，提高准确性
- ✅ 可追溯的信息来源
- ✅ 领域专业知识整合

### RAG vs 传统搜索

| 特性 | 传统搜索 | RAG系统 |
|------|----------|---------|
| 回答方式 | 文档列表 | 自然语言答案 |
| 信息整合 | 手动 | 自动 |
| 上下文理解 | 无 | 有 |
| 个性化 | 有限 | 高 |
| 准确性 | 取决于用户 | 高 |

---

## 🚀 第一个RAG系统

### 步骤1: 环境准备

```rust
use lumosai::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧠 开始创建RAG系统");
    
    // 1. 创建向量存储（内存版本，适合快速开始）
    let storage = lumosai::vector::memory().await?;
    println!("✅ 向量存储创建成功");
    
    // 2. 创建RAG系统
    let rag = lumosai::rag::builder()
        .storage(storage)
        .embedding_provider("openai")  // 使用OpenAI的嵌入模型
        .chunking_strategy("recursive")  // 递归分块策略
        .build()
        .await?;
    
    println!("✅ RAG系统创建成功");
    Ok(())
}
```

### 步骤2: 添加知识文档

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system().await?;
    
    // 添加产品知识
    let product_docs = vec![
        "LumosAI是一个基于Rust的企业级AI应用开发框架。",
        "它支持Agent创建、RAG系统、工具集成和工作流编排。",
        "LumosAI采用模块化设计，具有高性能和高安全性。",
        "框架支持多种大语言模型，包括OpenAI、Anthropic Claude等。",
        "LumosAI提供了完整的API和丰富的示例代码。"
    ];
    
    for doc in product_docs {
        rag.add_document(doc).await?;
        println!("📄 已添加文档: {}", &doc[..30.min(doc.len())]);
    }
    
    println!("✅ 所有文档添加完成");
    Ok(())
}
```

### 步骤3: 智能问答

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system().await?;
    
    // 添加一些示例文档
    add_sample_documents(&rag).await?;
    
    let questions = vec![
        "LumosAI是什么？",
        "LumosAI支持哪些功能？",
        "LumosAI有什么优势？",
        "如何开始使用LumosAI？"
    ];
    
    for question in questions {
        println!("👤 问题: {}", question);
        
        // 搜索相关文档
        let search_results = rag.search(question, 3).await?;
        println!("📚 找到 {} 个相关文档", search_results.len());
        
        // 生成基于文档的回答
        let answer = generate_answer_with_rag(&rag, question).await?;
        println!("🤖 回答: {}\n", answer);
    }
    
    Ok(())
}

async fn generate_answer_with_rag(rag: &RAGEngine, question: &str) -> Result<String> {
    // 获取相关文档
    let context_docs = rag.search(question, 3).await?;
    
    // 构建上下文
    let context = context_docs
        .iter()
        .enumerate()
        .map(|(i, doc)| format!("{}. {}", i + 1, doc.content))
        .collect::<Vec<_>>()
        .join("\n");
    
    // 创建专门用于回答的Agent
    let agent = lumosai::agent::simple(
        "gpt-3.5-turbo",
        &format!(
            "你是一个专业的问答助手。基于以下提供的文档信息回答用户问题。\n\n文档信息：\n{}\n\n请基于上述信息回答问题，如果文档中没有相关信息，请诚实地说明。",
            context
        )
    ).await?;
    
    agent.chat(question).await
}
```

---

## 🔧 RAG系统高级配置

### 向量存储选择

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 内存向量存储（适合开发和小规模数据）
    let memory_storage = lumosai::vector::memory().await?;
    
    // PostgreSQL向量存储（适合中等规模，持久化）
    let pg_storage = lumosai::vector::postgres("postgresql://user:pass@localhost/db").await?;
    
    // Qdrant向量存储（适合大规模，高性能）
    let qdrant_storage = lumosai::vector::qdrant("http://localhost:6333").await?;
    
    // 选择存储并创建RAG
    let rag = lumosai::rag::builder()
        .storage(qdrant_storage)  // 选择Qdrant
        .embedding_provider("openai")
        .build()
        .await?;
    
    Ok(())
}
```

### 分块策略配置

```rust
use lumosai::rag::{ChunkingStrategy, ChunkConfig};

// 递归分块（推荐）
let recursive_config = ChunkConfig::recursive()
    .chunk_size(1000)        // 每块1000字符
    .chunk_overlap(200)      // 重叠200字符
    .separators(vec!["\n\n", "\n", " ", ""]);  // 分隔符优先级

// 语义分块（基于语义相似性）
let semantic_config = ChunkConfig::semantic()
    .max_chunk_size(1500)    // 最大块大小
    .min_chunk_size(200)     // 最小块大小
    .embedding_model("text-embedding-3-small");

// 固定大小分块
let fixed_config = ChunkConfig::fixed()
    .chunk_size(800)         // 固定800字符
    .chunk_overlap(100);     // 重叠100字符

let rag = lumosai::rag::builder()
    .storage(storage)
    .chunking_strategy(recursive_config)
    .build()
    .await?;
```

### 检索参数优化

```rust
use lumosai::rag::{RetrievalConfig, SimilarityMetric};

let retrieval_config = RetrievalConfig::new()
    .top_k(5)                           // 检索前5个最相关文档
    .similarity_threshold(0.7)           // 相似度阈值
    .rerank(true)                       // 启用重排序
    .diversification(true)              // 启用多样性
    .similarity_metric(SimilarityMetric::Cosine)  // 余弦相似度
    .max_context_length(4000);           // 最大上下文长度

let rag = lumosai::rag::builder()
    .storage(storage)
    .retrieval_config(retrieval_config)
    .build()
    .await?;
```

---

## 📚 文档处理实战

### 处理Markdown文档

```rust
use std::path::Path;
use tokio::fs;

async fn process_markdown_files(rag: &RAGEngine, docs_dir: &str) -> Result<()> {
    let mut entries = fs::read_dir(docs_dir).await?;
    let mut processed_count = 0;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        // 只处理.md文件
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path).await?;
            
            // 添加文件路径作为元数据
            let file_path = path.to_string_lossy().to_string();
            let doc_with_metadata = format!("文件: {}\n\n{}", file_path, content);
            
            rag.add_document(doc_with_metadata).await?;
            processed_count += 1;
            
            println!("📄 处理文件: {} (第{}个)", path.display(), processed_count);
        }
    }
    
    println!("✅ 共处理了 {} 个Markdown文件", processed_count);
    Ok(())
}
```

### 处理PDF文档

```rust
use lumosai::document::{DocumentProcessor, PdfProcessor};

async fn process_pdf_documents(rag: &RAGEngine, pdf_paths: Vec<&str>) -> Result<()> {
    let pdf_processor = PdfProcessor::new();
    
    for pdf_path in pdf_paths {
        println!("📄 处理PDF: {}", pdf_path);
        
        // 提取PDF文本
        let pages = pdf_processor.extract_text(pdf_path).await?;
        
        // 按页添加文档（带页码信息）
        for (page_num, page_content) in pages.iter().enumerate() {
            let doc_with_metadata = format!(
                "PDF: {}\n页码: {}\n\n{}",
                pdf_path, page_num + 1, page_content
            );
            
            rag.add_document(doc_with_metadata).await?;
        }
        
        println!("✅ PDF {} 处理完成，共 {} 页", pdf_path, pages.len());
    }
    
    Ok(())
}
```

### 处理网页内容

```rust
use lumosai::document::{WebScraper, WebContent};

async fn process_web_pages(rag: &RAGEngine, urls: Vec<&str>) -> Result<()> {
    let scraper = WebScraper::new();
    
    for url in urls {
        println!("🌐 抓取网页: {}", url);
        
        // 抓取网页内容
        let web_content = scraper.scrape(url).await?;
        
        // 清理HTML标签
        let clean_content = scraper.clean_html(&web_content.html)?;
        
        // 添加文档（带URL和标题信息）
        let doc_with_metadata = format!(
            "网页: {}\n标题: {}\n抓取时间: {}\n\n{}",
            url,
            web_content.title,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            clean_content
        );
        
        rag.add_document(doc_with_metadata).await?;
        println!("✅ 网页 {} 处理完成", url);
    }
    
    Ok(())
}
```

---

## 🔍 高级检索功能

### 混合检索（语义+关键词）

```rust
use lumosai::rag::{HybridRetriever, KeywordSearchConfig};

let keyword_config = KeywordSearchConfig::new()
    .boost_factor(0.3)          // 关键词检索权重
    .min_term_frequency(2)      // 最小词频
    .max_terms_per_query(10);   // 最大查询词数

let hybrid_retriever = HybridRetriever::new()
    .semantic_weight(0.7)        // 语义检索权重
    .keyword_weight(0.3)         // 关键词检索权重
    .keyword_config(keyword_config);

let rag = lumosai::rag::builder()
    .storage(storage)
    .retriever(hybrid_retriever)
    .build()
    .await?;
```

### 多查询检索

```rust
use lumosai::rag::{MultiQueryRetriever, QueryGenerator};

let query_generator = QueryGenerator::new()
    .num_queries(3)              // 生成3个查询变体
    .diversity(0.8)              // 查询多样性
    .llm_provider("openai");     // 使用LLM生成查询

let multi_query_retriever = MultiQueryRetriever::new(query_generator);

// 执行多查询检索
let question = "如何在生产环境中部署LumosAI？";
let results = rag
    .search_with_retriever(&multi_query_retriever, question, 5)
    .await?;

println!("多查询检索找到 {} 个相关文档", results.len());
```

### 时间加权检索

```rust
use lumosai::rag::{TimeWeightedRetriever, DecayFunction};

let time_config = TimeWeightedRetriever::new()
    .decay_function(DecayFunction::Exponential)  // 指数衰减
    .half_life_days(30)                          // 30天半衰期
    .boost_recent(true)                          // 提升最新文档
    .min_date(Some(chrono::Utc::now() - chrono::Duration::days(365))); // 只考虑一年内的文档

let rag = lumosai::rag::builder()
    .storage(storage)
    .retriever(time_config)
    .build()
    .await?;
```

---

## 📊 RAG系统集成Agent

### 创建RAG增强的Agent

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 1. 创建RAG系统
    let rag = create_enterprise_rag().await?;
    
    // 2. 创建RAG增强的Agent
    let rag_agent = Agent::builder()
        .name("知识问答助手")
        .model("gpt-4")
        .system_prompt(r#"
你是一个专业的知识问答助手。请按照以下规则回答用户问题：

1. **基于文档回答**: 严格基于提供的文档信息回答问题
2. **引用来源**: 在回答中明确指出信息来源
3. **诚实回答**: 如果文档中没有相关信息，请明确说明
4. **结构化回答**: 使用清晰的格式组织答案
5. **保持准确**: 不要编造或推测信息

回答格式：
- 主要答案
- 详细解释
- 相关文档引用
        "#)
        .rag(rag)  // 关键：将RAG系统集成到Agent中
        .max_tool_calls(5)  // 限制检索次数
        .build()
        .await?;
    
    println!("✅ RAG增强Agent创建成功");
    
    // 测试问答
    let questions = vec![
        "LumosAI的主要特性有哪些？",
        "如何配置LumosAI的生产环境？",
        "LumosAI支持哪些向量数据库？"
    ];
    
    for question in questions {
        println!("\n👤 问题: {}", question);
        
        let start_time = std::time::Instant::now();
        let answer = rag_agent.chat(question).await?;
        let duration = start_time.elapsed();
        
        println!("🤖 回答: {}", answer);
        println!("⏱️ 响应时间: {:?}", duration);
    }
    
    Ok(())
}
```

### 实时知识更新

```rust
use tokio::time::{interval, Duration};

async fn setup_knowledge_updates(rag: &RAGEngine) -> Result<()> {
    let rag_clone = rag.clone();
    
    // 每小时检查一次更新
    let mut interval = interval(Duration::from_secs(3600));
    
    tokio::spawn(async move {
        loop {
            interval.tick().await;
            
            match check_for_updates().await {
                Ok(new_documents) => {
                    for doc in new_documents {
                        if let Err(e) = rag_clone.add_document(doc).await {
                            eprintln!("添加文档失败: {}", e);
                        }
                    }
                }
                Err(e) => eprintln!("检查更新失败: {}", e),
            }
        }
    });
    
    Ok(())
}

async fn check_for_updates() -> Result<Vec<String>> {
    // 这里实现检查新文档的逻辑
    // 可以是监控系统、文件系统、API等
    Ok(vec![])  // 返回新文档列表
}
```

---

## 🎯 实战项目：智能文档问答系统

### 项目概述

创建一个企业内部文档问答系统，能够：
- 支持多种文档格式上传
- 实时文档索引更新
- 智能问答和引用溯源
- 用户权限管理

### 核心实现

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct DocumentQASystem {
    rag: Arc<RAGEngine>,
    user_documents: Arc<Mutex<HashMap<String, Vec<String>>>>,  // 用户ID -> 文档ID列表
}

impl DocumentQASystem {
    async fn new() -> Result<Self> {
        let rag = Arc::new(lumosai::rag::builder()
            .storage(lumosai::vector::memory().await?)
            .embedding_provider("openai")
            .build()
            .await?);
        
        Ok(Self {
            rag,
            user_documents: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    async fn add_document(&self, user_id: &str, content: &str) -> Result<String> {
        let doc_id = format!("{}_{}", user_id, uuid::Uuid::new_v4());
        let doc_with_metadata = format!(
            "文档ID: {}\n用户: {}\n上传时间: {}\n\n{}",
            doc_id,
            user_id,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            content
        );
        
        self.rag.add_document(doc_with_metadata).await?;
        
        // 记录用户的文档
        let mut user_docs = self.user_documents.lock().unwrap();
        user_docs.entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(doc_id.clone());
        
        Ok(doc_id)
    }
    
    async fn ask_question(&self, user_id: &str, question: &str) -> Result<String> {
        // 获取用户的文档限制
        let user_docs = self.user_documents.lock().unwrap();
        let user_doc_ids = user_docs.get(user_id).unwrap_or(&vec![]);
        
        // 创建用户专用的Agent
        let user_agent = Agent::builder()
            .name("文档问答助手")
            .model("gpt-3.5-turbo")
            .system_prompt(&format!(
                "你是{}的专属文档问答助手。只能基于用户上传的文档回答问题。",
                user_id
            ))
            .rag((*self.rag).clone())
            .build()
            .await?;
        
        user_agent.chat(question).await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 创建文档问答系统
    let qa_system = DocumentQASystem::new().await?;
    
    // 模拟用户使用
    let user_id = "user123";
    
    // 添加文档
    let doc1 = "LumosAI使用指南第1章：框架介绍...";
    let doc2 = "LumosAI使用指南第2章：快速开始...";
    
    let doc_id1 = qa_system.add_document(user_id, doc1).await?;
    let doc_id2 = qa_system.add_document(user_id, doc2).await?;
    
    println!("✅ 文档添加成功: {}, {}", doc_id1, doc_id2);
    
    // 问答测试
    let questions = vec![
        "LumosAI是什么？",
        "如何快速开始使用LumosAI？",
        "LumosAI支持哪些功能？"
    ];
    
    for question in questions {
        println!("\n👤 {}: {}", user_id, question);
        let answer = qa_system.ask_question(user_id, question).await?;
        println!("🤖 回答: {}", answer);
    }
    
    Ok(())
}
```

---

## 📈 性能优化

### 向量索引优化

```rust
use lumosai::vector::{IndexConfig, IndexType};

// 创建优化的向量索引
let index_config = IndexConfig::new()
    .index_type(IndexType::HNSW)       // HNSW索引，适合大规模数据
    .ef_construction(200)              // 构建时的搜索范围
    .ef_search(50)                     // 搜索时的候选数量
    .m(16)                            // HNSW的连接数
    .max_elements(100000);             // 最大元素数量

let storage = lumosai::vector::memory_with_index(index_config).await?;
```

### 缓存策略

```rust
use std::collections::LRU;
use tokio::time::Duration;

struct CachedRAG {
    rag: RAGEngine,
    cache: Arc<Mutex<LRU<String, Vec<String>>>>,  // 查询缓存
}

impl CachedRAG {
    async fn search_cached(&self, query: &str) -> Result<Vec<String>> {
        let cache_key = query.to_lowercase();
        
        // 检查缓存
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached_result) = cache.get(&cache_key) {
                println!("🎯 缓存命中: {}", query);
                return Ok(cached_result.clone());
            }
        }
        
        // 执行搜索
        let results = self.rag.search(query, 5).await?;
        let result_contents: Vec<String> = results.iter()
            .map(|doc| doc.content.clone())
            .collect();
        
        // 更新缓存
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(cache_key, result_contents.clone());
        }
        
        println!("🔍 搜索执行: {}", query);
        Ok(result_contents)
    }
}
```

---

## ✅ 学习检查

### 关键概念回顾

1. **RAG核心价值**:
   - 🧠 结合检索和生成
   - 📚 减少幻觉，提高准确性
   - 🔄 支持实时知识更新
   - 📖 可追溯的信息来源

2. **RAG系统组件**:
   - 向量存储（内存/PostgreSQL/Qdrant）
   - 文档处理器（PDF/Markdown/网页）
   - 分块策略（递归/语义/固定）
   - 检索配置（混合/多查询/时间加权）

3. **实际应用场景**:
   - 企业知识库问答
   - 产品技术支持
   - 学习辅助系统
   - 研究文献分析

### 实践验证

完成以下任务来巩固你的RAG知识：

- [ ] 创建支持多种文档格式的RAG系统
- [ ] 实现文档的实时更新机制
- [ ] 优化检索性能和准确性
- [ ] 集成RAG到Agent中实现智能问答

### 下一步学习

掌握了RAG基础后，你可以继续学习：
- [工具集成详解](../basics/tool-integration.md) - 扩展RAG系统能力
- [内存系统使用](../basics/memory-systems.md) - 管理对话上下文
- [多Agent协作](../advanced/multi-agent.md) - 构建复杂的AI系统

---

**🎉 恭喜！你已经掌握了RAG系统的核心概念和实现方法！**

*[← 返回教程导航](../README.md)*