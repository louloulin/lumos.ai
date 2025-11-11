> ℹ️ 统一入口迁移提示：本页面为历史/补充版。请前往规范入口 [quick-start/README.md](./quick-start/README.md) 获取最新的快速开始指南。更多映射与说明见 [CONTENT_MAP.md](./CONTENT_MAP.md)。

 # LumosAI 快速开始指南

## 概述

LumosAI 是一个高性能的 Rust AI 框架，专为企业级应用设计。它提供了完整的 RAG（检索增强生成）系统、Agent 框架和向量数据库集成。

## 特性

- 🚀 **高性能**: Rust 原生性能，>1M ops/sec 向量操作
- 🔒 **类型安全**: 编译时保证，零运行时错误
- 🏢 **企业就绪**: 连接池、事务、监控、配置管理
- 🔧 **模块化**: 独立 crate，按需引入
- 📚 **完整生态**: RAG、Agent、向量存储一体化解决方案

## 安装

将以下依赖添加到您的 `Cargo.toml`:

```toml
[dependencies]
lumosai = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## 快速开始

### 1. 简单的 Agent

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建一个简单的 Agent
    let agent = lumos::agent::simple("gpt-4", "You are a helpful assistant").await?;
    
    // 与 Agent 对话
    let response = agent.chat("Hello, how are you?").await?;
    println!("Agent: {}", response);
    
    Ok(())
}
```

### 2. 向量存储

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建内存向量存储
    let storage = lumos::vector::memory().await?;
    
    // 创建索引
    let config = IndexConfig::new("documents", 384);
    storage.create_index(config).await?;
    
    // 插入文档
    let doc = Document::new("doc1", "人工智能的发展历程")
        .with_embedding(vec![0.1; 384])
        .with_metadata("category", "technology");
    
    storage.upsert_documents("documents", vec![doc]).await?;
    
    // 搜索
    let query = vec![0.1; 384];
    let request = SearchRequest {
        index_name: "documents".to_string(),
        query: SearchQuery::Vector(query),
        top_k: 5,
        filter: None,
        include_metadata: true,
        include_vectors: false,
        options: HashMap::new(),
    };
    
    let results = storage.search(request).await?;
    println!("找到 {} 个相关文档", results.results.len());
    
    Ok(())
}
```

### 3. RAG 系统

```rust
use lumosai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建 RAG 系统
    let storage = lumos::vector::memory().await?;
    let rag = lumos::rag::simple(storage, "openai").await?;
    
    // 处理文档
    let documents = vec![
        "人工智能是计算机科学的一个分支。",
        "机器学习是人工智能的一个子领域。",
        "深度学习是机器学习的一种方法。",
    ];
    
    rag.process_documents(documents).await?;
    
    // 查询
    let answer = rag.query("什么是人工智能？").await?;
    println!("答案: {}", answer);
    
    Ok(())
}
```

## 核心概念

### Agent

Agent 是 LumosAI 的核心抽象，它封装了 LLM 模型、系统提示、工具和上下文管理。

```rust
let agent = Agent::builder()
    .name("assistant")
    .model(OpenAI::gpt4())
    .system_prompt("You are a helpful assistant")
    .tools(vec![calculator_tool, weather_tool])
    .memory(conversation_memory)
    .build();
```

### 向量存储

LumosAI 提供统一的向量存储抽象，支持多种后端：

- **内存存储**: 高性能，适合开发和测试
- **PostgreSQL**: 企业级，支持事务和 ACID
- **Qdrant**: 专业向量数据库
- **Weaviate**: 云原生向量数据库

### RAG 系统

RAG（检索增强生成）系统结合了向量检索和生成模型：

```rust
let rag = RagPipeline::builder()
    .chunking_strategy(ChunkingStrategy::Recursive { size: 1000, overlap: 200 })
    .embedding_model(EmbeddingModel::OpenAI("text-embedding-3-small"))
    .vector_store(postgres_storage)
    .build();
```

## 配置

### 环境变量

LumosAI 支持通过环境变量进行配置：

```bash
# OpenAI API 密钥
export OPENAI_API_KEY="your-api-key"

# 数据库连接
export DATABASE_URL="postgresql://user:password@localhost/db"

# 向量数据库
export QDRANT_URL="http://localhost:6333"
export WEAVIATE_URL="http://localhost:8080"
```

### 配置文件

您也可以使用配置文件：

```toml
# lumos.toml
[llm]
provider = "openai"
model = "gpt-4"
api_key = "your-api-key"

[vector]
provider = "postgres"
url = "postgresql://user:password@localhost/db"
pool_size = 10

[rag]
chunk_size = 1000
chunk_overlap = 200
top_k = 5
```

## 最佳实践

### 1. 错误处理

始终处理可能的错误：

```rust
match agent.chat("Hello").await {
    Ok(response) => println!("Response: {}", response),
    Err(e) => eprintln!("Error: {}", e),
}
```

### 2. 资源管理

使用连接池管理数据库连接：

```rust
let config = PostgresConfig::new(database_url)
    .with_pool(PoolConfig { 
        max_connections: 20,
        min_connections: 5,
        connection_timeout: Duration::from_secs(30),
    });
```

### 3. 性能优化

启用缓存以提高性能：

```rust
let cache_config = CacheConfig {
    max_entries: 1000,
    ttl: Duration::from_secs(3600),
    enable_lru: true,
};
```

### 4. 监控

使用内置的性能监控：

```rust
let metrics = storage.get_performance_metrics().await;
println!("平均响应时间: {:?}", metrics.average_response_time);

let cache_stats = storage.get_cache_stats().await;
println!("缓存命中率: {:.2}%", cache_stats.hit_rate * 100.0);
```

## 下一步

- 查看 [API 参考文档](./vector_api_reference.md)
- 浏览 [示例项目](../examples/)
- 了解 [架构设计](./architecture.md)
- 参与 [社区讨论](https://github.com/louloulin/lumos.ai/discussions)

## 获取帮助

- 📖 [文档](https://docs.lumosai.dev)
- 💬 [Discord 社区](https://discord.gg/lumosai)
- 🐛 [问题反馈](https://github.com/louloulin/lumos.ai/issues)
- 📧 [邮件支持](mailto:support@lumosai.dev)
