# 向量数据库集成教程

本教程演示如何在 LumosAI 中集成与使用向量数据库，覆盖快速自动选择、手动指定后端、构建器模式以及常见环境变量设置。更多细节请参考：

- 参考文档：`docs/VECTOR_DATABASES.md`
- API 参考：`docs/vector_api_reference.md`
- 便捷函数源码：`src/vector.rs`

---

## 自动选择最佳后端（推荐）

```rust
use lumosai::prelude::*;

/// 自动检测并创建最佳向量存储（Qdrant > Weaviate > Postgres > Memory）
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = lumosai::vector::auto().await?;
    println!("后端: {}", storage.backend_info().name);
    Ok(())
}
```

> 提示：`auto()` 会读取环境变量 `QDRANT_URL`、`WEAVIATE_URL` 与 `DATABASE_URL`，自动选择可用的后端。

---

## 手动指定后端

```rust
use lumosai::prelude::*;

/// 手动创建各类向量存储后端示例
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Qdrant（需启用 feature: vector-qdrant）
    let qdrant = lumosai::vector::qdrant("http://localhost:6334").await?;

    // Weaviate（需启用 feature: vector-weaviate）
    let weaviate = lumosai::vector::weaviate("http://localhost:8080").await?;

    // PostgreSQL（需设置 DATABASE_URL 或使用 postgres_with_url）
    let postgres = lumosai::vector::postgres().await?;
    // 或指定连接串：
    // let postgres = lumosai::vector::postgres_with_url("postgresql://user:pass@localhost/db").await?;

    // 内存存储（需启用 feature: vector-memory）
    let memory = lumosai::vector::memory().await?;

    println!("创建成功");
    Ok(())
}
```

---

## 构建器模式

```rust
use lumosai::prelude::*;

/// 使用构建器选择后端与连接参数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = lumosai::vector::builder()
        .backend("qdrant")
        .url("http://localhost:6334")
        .batch_size(1000)
        .build()
        .await?;

    println!("已创建向量存储: {}", storage.backend_info().name);
    Ok(())
}
```

---

## 常见操作：创建索引、插入与搜索

以下演示统一接口（`VectorStorage` trait）的典型用法：

```rust
use std::collections::HashMap;
use lumosai::prelude::*;
use lumosai_vector_core::{IndexConfig, Document, SearchRequest, SearchQuery};

/// 演示索引创建、文档插入与向量搜索的完整流程
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 选择后端（示例用内存存储）
    let storage = lumosai::vector::memory().await?;

    // 2. 创建索引
    let index = IndexConfig::new("articles", 384);
    storage.create_index(index).await?;

    // 3. 准备文档（假设已生成嵌入）
    let docs = vec![
        Document::new("doc1", "人工智能的发展历程").with_embedding(vec![0.1; 384]).with_metadata("category", "tech"),
        Document::new("doc2", "机器学习算法介绍").with_embedding(vec![0.2; 384]).with_metadata("category", "tech"),
    ];
    let _ids = storage.upsert_documents("articles", docs).await?;

    // 4. 构造搜索请求
    let request = SearchRequest {
        index_name: "articles".to_string(),
        query: SearchQuery::Vector(vec![0.15; 384]),
        top_k: 5,
        filter: None,
        include_metadata: true,
        include_vectors: false,
        options: HashMap::new(),
    };

    // 5. 执行搜索
    let resp = storage.search(request).await?;
    println!("top_k={} 命中数={}", resp.top_k, resp.hits.len());
    Ok(())
}
```

---

## 在 macOS 设置环境变量

```bash
# Qdrant（gRPC）
export QDRANT_URL=http://localhost:6334

# Weaviate（REST）
export WEAVIATE_URL=http://localhost:8080

# PostgreSQL（pgvector）
export DATABASE_URL=postgresql://postgres:password@localhost:5432/lumos
```

> 在 `Cargo.toml` 中启用相应 feature，例如：`vector-qdrant`、`vector-weaviate`、`vector-postgres`、`vector-memory`。也可使用聚合特性 `vector-all`。

---

## 迁移到生产后端

- Qdrant：详见 `lumosai_vector/qdrant/` 与示例 `examples/basic_usage.rs`
- Weaviate：参见 `lumosai_vector/weaviate/` 配置示例
- LanceDB/Milvus/Postgres：参考各自 `README.md` 与示例目录

> 生产后端建议：评估索引类型（如 HNSW/IVF）、批量写入、检索过滤与资源限制，配合 RAG 管线实现高质检索。

---

## 参考

- `docs/VECTOR_DATABASES.md`（详细集成指南）
- `docs/vector_api_reference.md`（完整 API 类型与方法）
- `src/vector.rs`（便捷连接函数与构建器）