# 🧠 RAG API

**检索增强生成系统API - 智能文档问答和知识管理**

## 🚀 快速开始

### 简单RAG系统创建

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建向量存储
    let storage = lumosai::vector::memory().await?;
    
    // 创建RAG系统
    let rag = lumosai::rag::simple(storage).await?;
    
    // 添加文档
    rag.add_document("LumosAI是一个AI框架").await?;
    
    // 搜索文档
    let results = rag.search("什么是LumosAI", 3).await?;
    println!("找到 {} 个相关文档", results.len());
    
    Ok(())
}
```

---

## 🔧 RAG系统构建

### `lumosai::rag::simple()`

最简单的RAG系统创建方式。

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let storage = lumosai::vector::memory().await?;
    let rag = lumosai::rag::simple(storage).await?;
    Ok(())
}
```

**参数:**
- `storage: impl VectorStore` - 向量存储实例

**返回:** `Result<RAGEngine>` - RAG系统实例

### `RAGBuilder`

使用构建器模式创建高级RAG系统。

```rust
use lumosai::prelude::*;
use lumosai::rag::{RAGBuilder, EmbeddingProvider, ChunkingStrategy};

#[tokio::main]
async fn main() -> Result<()> {
    let rag = RAGBuilder::new()
        .storage(lumosai::vector::memory().await?)
        .embedding_provider(EmbeddingProvider::OpenAI)
        .chunking_strategy(ChunkingStrategy::Recursive)
        .max_context_length(4000)
        .top_k(5)
        .similarity_threshold(0.7)
        .build()
        .await?;
    
    Ok(())
}
```

### 构建器配置选项

| 方法 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `storage(storage)` | 必需 | - | 向量存储后端 |
| `embedding_provider(provider)` | 可选 | OpenAI | 嵌入模型提供商 |
| `chunking_strategy(strategy)` | 可选 | Recursive | 文档分块策略 |
| `max_context_length(length)` | 可选 | 4000 | 最大上下文长度 |
| `top_k(k)` | 可选 | 5 | 检索文档数量 |
| `similarity_threshold(threshold)` | 可选 | 0.7 | 相似度阈值 |
| `rerank(enabled)` | 可选 | false | 是否启用重排序 |

---

## 📚 文档管理

### 添加文档

#### `add_document(content: &str) -> Result<String>`

添加单个文档到RAG系统。

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system().await?;
    
    // 添加文档
    let doc_id = rag.add_document("LumosAI是一个基于Rust的AI框架。").await?;
    println!("文档添加成功，ID: {}", doc_id);
    
    // 添加带元数据的文档
    let doc_with_metadata = format!(
        "文件: manual.pdf\n页码: 15\n内容: LumosAI支持多种向量存储后端"
    );
    let doc_id2 = rag.add_document(doc_with_metadata).await?;
    
    Ok(())
}
```

**参数:**
- `content: &str` - 文档内容（可包含元数据）

**返回:** `Result<String>` - 文档ID

#### `add_documents(documents: Vec<&str>) -> Result<Vec<String>>`

批量添加文档。

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system().await?;
    
    let documents = vec![
        "LumosAI支持Agent创建",
        "LumosAI支持RAG系统", 
        "LumosAI支持工具集成",
        "LumosAI支持多Agent协作"
    ];
    
    let doc_ids = rag.add_documents(documents).await?;
    println!("批量添加了 {} 个文档", doc_ids.len());
    
    Ok(())
}
```

### 文档处理

#### `process_file(file_path: &str) -> Result<Vec<String>>`

处理文件并自动添加到RAG系统。

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system().await?;
    
    // 处理Markdown文件
    let md_doc_ids = rag.process_file("docs/manual.md").await?;
    println!("处理Markdown文件: {} 个文档块", md_doc_ids.len());
    
    // 处理PDF文件
    let pdf_doc_ids = rag.process_file("data/specification.pdf").await?;
    println!("处理PDF文件: {} 个文档块", pdf_doc_ids.len());
    
    // 处理网页
    let web_doc_ids = rag.process_file("https://example.com/article").await?;
    println!("处理网页: {} 个文档块", web_doc_ids.len());
    
    Ok(())
}
```

#### `update_document(doc_id: &str, content: &str) -> Result<()>`

更新现有文档。

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system().await?;
    
    let doc_id = rag.add_document("初始内容").await?;
    
    // 更新文档
    rag.update_document(&doc_id, "更新后的内容").await?;
    println!("文档 {} 更新成功", doc_id);
    
    Ok(())
}
```

#### `delete_document(doc_id: &str) -> Result<()>`

删除文档。

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system().await?;
    
    let doc_id = rag.add_document("临时文档").await?;
    
    // 删除文档
    rag.delete_document(&doc_id).await?;
    println!("文档 {} 删除成功", doc_id);
    
    Ok(())
}
```

---

## 🔍 文档检索

### 基础搜索

#### `search(query: &str, top_k: usize) -> Result<Vec<RetrievedDocument>>`

基于查询检索相关文档。

```rust
use lumosai::rag::RetrievedDocument;

#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system_with_documents().await?;
    
    let query = "LumosAI有什么特性？";
    let results = rag.search(query, 5).await?;
    
    println!("查询: {}", query);
    println!("找到 {} 个相关文档:\n", results.len());
    
    for (i, doc) in results.iter().enumerate() {
        println!("{}. [相似度: {:.3}] {}", i + 1, doc.score, doc.content);
    }
    
    Ok(())
}
```

**参数:**
- `query: &str` - 搜索查询
- `top_k: usize` - 返回文档数量

**返回:** `Result<Vec<RetrievedDocument>>` - 检索到的文档列表

### 高级搜索

#### `search_with_filters(query: &str, filters: SearchFilters) -> Result<Vec<RetrievedDocument>>`

带过滤条件的搜索。

```rust
use lumosai::rag::{SearchFilters, DateFilter, MetadataFilter};

#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system().await?;
    
    let filters = SearchFilters::new()
        .date_filter(DateFilter::after(
            chrono::Utc::now() - chrono::Duration::days(30)
        ))
        .metadata_filter("department", "技术部")
        .metadata_filter("tag", "重要");
    
    let results = rag.search_with_filters("LumosAI", filters, 10).await?;
    
    println!("过滤搜索找到 {} 个文档", results.len());
    
    Ok(())
}
```

#### `hybrid_search(query: &str, keyword_weight: f64, semantic_weight: f64) -> Result<Vec<RetrievedDocument>>`

混合搜索（关键词+语义）。

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system().await?;
    
    let query = "企业级AI应用开发";
    
    // 混合搜索：30%关键词 + 70%语义
    let results = rag.hybrid_search(query, 0.3, 0.7, 8).await?;
    
    println!("混合搜索结果:");
    for doc in &results {
        println!("相关度: {:.3} | {}", doc.score, doc.content);
    }
    
    Ok(())
}
```

### 多查询检索

#### `multi_query_search(query: &str, num_queries: usize) -> Result<Vec<RetrievedDocument>>`

生成多个查询变体进行检索。

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system().await?;
    
    let original_query = "如何提升AI应用性能？";
    
    // 生成3个查询变体
    let results = rag.multi_query_search(original_query, 3, 10).await?;
    
    println!("原始查询: {}", original_query);
    println!("多查询检索找到 {} 个相关文档", results.len());
    
    Ok(())
}
```

---

## ⚙️ 配置管理

### 分块策略

#### `ChunkingStrategy` 枚举

```rust
use lumosai::rag::{ChunkingStrategy, ChunkConfig};

// 递归分块（推荐）
let recursive_config = ChunkConfig::recursive()
    .chunk_size(1000)
    .chunk_overlap(200)
    .separators(vec!["\n\n", "\n", " ", ""]);

// 固定大小分块
let fixed_config = ChunkConfig::fixed()
    .chunk_size(800)
    .chunk_overlap(100);

// 语义分块
let semantic_config = ChunkConfig::semantic()
    .max_chunk_size(1500)
    .min_chunk_size(200)
    .embedding_model("text-embedding-3-small");

let rag = RAGBuilder::new()
    .chunking_strategy(ChunkingStrategy::Custom(recursive_config))
    .build()
    .await?;
```

### 嵌入模型配置

#### `EmbeddingProvider` 配置

```rust
use lumosai::rag::{EmbeddingProvider, EmbeddingConfig};

// OpenAI嵌入模型
let openai_config = EmbeddingConfig::openai()
    .model("text-embedding-3-large")
    .batch_size(100)
    .max_retries(3);

// 本地嵌入模型
let local_config = EmbeddingConfig::local()
    .model_path("/path/to/model")
    .device("cuda")  // or "cpu"

let rag = RAGBuilder::new()
    .embedding_provider(EmbeddingProvider::Custom(openai_config))
    .build()
    .await?;
```

### 检索参数优化

```rust
use lumosai::rag::{RetrievalConfig, SimilarityMetric, RerankerConfig};

let retrieval_config = RetrievalConfig::new()
    .top_k(10)                          // 初始检索数量
    .final_top_k(5)                     // 最终返回数量
    .similarity_threshold(0.7)          // 相似度阈值
    .similarity_metric(SimilarityMetric::Cosine)
    .rerank_config(RerankerConfig::new()
        .enabled(true)
        .model("cross-encoder/ms-marco-MiniLM-L-6-v2")
        .top_k(20));

let rag = RAGBuilder::new()
    .retrieval_config(retrieval_config)
    .build()
    .await?;
```

---

## 🔧 向量存储配置

### 内存存储

```rust
use lumosai::vector::{MemoryConfig, IndexConfig};

let memory_config = MemoryConfig::new()
    .max_documents(10000)           // 最大文档数量
    .index_config(IndexConfig::flat());
    
let storage = lumosai::vector::memory_with_config(memory_config).await?;
```

### PostgreSQL存储

```rust
use lumosai::vector::{PostgresConfig, TableConfig};

let pg_config = PostgresConfig::new()
    .connection_string("postgresql://user:pass@localhost/rag_db")
    .table_config(TableConfig::new()
        .table_name("documents")
        .vector_dimension(1536)
        .index_type("hnsw"))
    .pool_size(10);

let storage = lumosai::vector::postgres_with_config(pg_config).await?;
```

### Qdrant存储

```rust
use lumosai::vector::{QdrantConfig, CollectionConfig};

let qdrant_config = QdrantConfig::new()
    .url("http://localhost:6333")
    .api_key("your-api-key")
    .collection_config(CollectionConfig::new()
        .name("documents")
        .vector_size(1536)
        .distance("Cosine"));

let storage = lumosai::vector::qdrant_with_config(qdrant_config).await?;
```

---

## 📊 监控和分析

### 统计信息

#### `get_statistics() -> RAGStatistics`

获取RAG系统的统计信息。

```rust
use lumosai::rag::RAGStatistics;

#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system().await?;
    
    let stats = rag.get_statistics().await?;
    
    println!("📊 RAG系统统计:");
    println!("  总文档数: {}", stats.total_documents);
    println!("  总向量数: {}", stats.total_vectors);
    println!("  平均文档长度: {:.1} 字符", stats.avg_document_length);
    println!("  存储使用: {:.2} MB", stats.storage_usage_mb);
    println!("  索引大小: {:.2} MB", stats.index_size_mb);
    println!("  查询次数: {}", stats.query_count);
    println!("  平均查询时间: {:.2} ms", stats.avg_query_time_ms);
    
    Ok(())
}
```

### 性能监控

```rust
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system().await?;
    
    let queries = vec![
        "LumosAI特性",
        "RAG系统原理",
        "向量数据库对比"
    ];
    
    for query in queries {
        let start = Instant::now();
        let results = rag.search(query, 5).await?;
        let duration = start.elapsed();
        
        println!("查询: {} | 耗时: {:?} | 结果数: {}", 
                query, duration, results.len());
    }
    
    Ok(())
}
```

---

## 🔄 RAG与Agent集成

### 创建RAG增强的Agent

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建RAG系统
    let rag = RAGBuilder::new()
        .storage(lumosai::vector::memory().await?)
        .embedding_provider(EmbeddingProvider::OpenAI)
        .build()
        .await?;
    
    // 添加文档
    rag.add_document("LumosAI支持多种大语言模型").await?;
    rag.add_document("LumosAI具有企业级安全性").await?;
    
    // 创建RAG增强Agent
    let rag_agent = Agent::builder()
        .name("知识问答助手")
        .model("gpt-4")
        .system_prompt(r#"
你是一个专业的知识问答助手。请严格基于提供的文档信息回答问题：
1. 如果文档中有相关信息，请准确引用并回答
2. 如果文档中没有相关信息，请诚实地说明
3. 提供清晰的来源引用
        "#)
        .rag(rag)
        .max_tool_calls(3)  // 限制RAG检索次数
        .build()
        .await?;
    
    // 测试问答
    let response = rag_agent.chat("LumosAI支持哪些模型？").await?;
    println!("回答: {}", response);
    
    Ok(())
}
```

### 上下文增强

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system().await?;
    
    // 获取相关文档作为上下文
    let query = "如何在生产环境中部署LumosAI";
    let context_docs = rag.search(query, 5).await?;
    
    // 构建增强上下文
    let context = context_docs
        .iter()
        .enumerate()
        .map(|(i, doc)| format!("来源{}: {}", i + 1, doc.content))
        .collect::<Vec<_>>()
        .join("\n\n");
    
    // 创建基于上下文的Agent
    let contextual_agent = lumosai::agent::simple(
        "gpt-4",
        &format!(
            "基于以下文档信息回答用户问题：\n\n文档内容：\n{}\n\n请基于上述信息回答问题。",
            context
        )
    ).await?;
    
    let response = contextual_agent.chat(query).await?;
    println!("基于上下文的回答: {}", response);
    
    Ok(())
}
```

---

## ⚠️ 错误处理

### 常见错误类型

```rust
use lumosai::rag::{RAGError, Result};

async fn handle_rag_operations() -> Result<()> {
    let rag = create_rag_system().await?;
    
    match rag.search("查询内容", 5).await {
        Ok(results) => println!("找到 {} 个文档", results.len()),
        Err(RAGError::VectorStoreError(msg)) => {
            eprintln!("向量存储错误: {}", msg);
        }
        Err(RAGError::EmbeddingError(msg)) => {
            eprintln!("嵌入生成错误: {}", msg);
        }
        Err(RAGError::IndexingError(msg)) => {
            eprintln!("索引错误: {}", msg);
        }
        Err(RAGError::QueryError(msg)) => {
            eprintln!("查询错误: {}", msg);
        }
        Err(e) => {
            eprintln!("未知RAG错误: {}", e);
        }
    }
    
    Ok(())
}
```

### 重试机制

```rust
use tokio::time::{sleep, Duration};

async fn search_with_retry(rag: &RAGEngine, query: &str, max_retries: u32) -> Result<Vec<RetrievedDocument>> {
    for attempt in 1..=max_retries {
        match rag.search(query, 5).await {
            Ok(results) => return Ok(results),
            Err(_) if attempt < max_retries => {
                println!("RAG搜索失败，第{}次重试...", attempt);
                sleep(Duration::from_secs(2_u64.pow(attempt))).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(RAGError::QueryError("重试失败".to_string()))
}
```

---

## 🚀 性能优化

### 批量操作

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let rag = create_rag_system().await?;
    
    // 批量添加文档
    let documents: Vec<String> = (1..=1000)
        .map(|i| format!("这是第{}个测试文档", i))
        .collect();
    
    let start = std::time::Instant::now();
    let doc_ids = rag.add_documents(documents).await?;
    let duration = start.elapsed();
    
    println!("批量添加{}个文档，耗时: {:?}", doc_ids.len(), duration);
    
    Ok(())
}
```

### 缓存查询结果

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct CachedRAG {
    rag: RAGEngine,
    cache: Arc<Mutex<HashMap<String, Vec<RetrievedDocument>>>>,
}

impl CachedRAG {
    async fn search_cached(&self, query: &str) -> Result<Vec<RetrievedDocument>> {
        let cache_key = query.to_lowercase().trim().to_string();
        
        // 检查缓存
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached_results) = cache.get(&cache_key) {
                println!("缓存命中: {}", query);
                return Ok(cached_results.clone());
            }
        }
        
        // 执行搜索
        let results = self.rag.search(query, 5).await?;
        
        // 更新缓存
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(cache_key.clone(), results.clone());
        }
        
        println!("搜索执行: {}", query);
        Ok(results)
    }
}
```

---

## 🔗 相关API

- **[Agent API](agent.md)** - RAG与Agent的集成
- **[Vector Storage API](vector-storage.md)** - 向量存储操作
- **[Memory API](memory.md)** - 内存管理系统
- **[Tools API](tools.md)** - 工具系统

---

## 📖 更多示例

- [RAG基础教程](../learn/tutorials/basics/rag-basics.md)
- [智能问答系统](../learn/examples/README.md#智能问答系统-⭐⭐)
- [企业知识库示例](../learn/examples/README.md#企业知识库系统-⭐⭐⭐⭐⭐)
- [向量数据库优化指南](../guides/vector-optimization.md)

---

**开始构建智能文档问答系统吧！** 🧠

*[← 返回API参考](README.md)*