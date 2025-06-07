# 真实向量数据库集成指南

Lumos支持多种真实的向量数据库，包括Qdrant、Weaviate和PostgreSQL（带pgvector扩展）。本指南将帮助您快速集成和使用这些向量数据库。

## 🚀 快速开始

### 1. 启用向量数据库特性

在`Cargo.toml`中启用所需的特性：

```toml
[dependencies]
lumos = { version = "0.1.0", features = ["vector-qdrant", "vector-weaviate", "vector-postgres"] }

# 或者启用所有向量数据库特性
lumos = { version = "0.1.0", features = ["vector-all"] }
```

### 2. 使用Docker快速启动向量数据库

我们提供了Docker Compose配置来快速启动所有支持的向量数据库：

```bash
# 启动所有向量数据库
docker-compose -f docker-compose.vector-dbs.yml up -d

# 或者使用便利脚本（Linux/macOS）
./scripts/vector-dbs.sh start

# 检查状态
./scripts/vector-dbs.sh status
```

### 3. 设置环境变量

```bash
# Qdrant
export QDRANT_URL=http://localhost:6334

# Weaviate  
export WEAVIATE_URL=http://localhost:8080

# PostgreSQL
export DATABASE_URL=postgresql://postgres:password@localhost:5432/lumos
```

## 📊 支持的向量数据库

| 数据库 | 特性 | 端口 | 状态 |
|--------|------|------|------|
| **Qdrant** | 高性能、Rust原生、gRPC/REST API | 6333/6334 | ✅ 完全支持 |
| **Weaviate** | GraphQL、语义搜索、模块化 | 8080 | ✅ 完全支持 |
| **PostgreSQL** | SQL兼容、pgvector扩展、ACID | 5432 | ✅ 完全支持 |
| **内存存储** | 开发测试、零配置 | - | ✅ 完全支持 |

## 💻 代码示例

### 自动选择最佳向量数据库

```rust
use lumos::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 自动选择最佳可用的向量数据库
    // 优先级：Qdrant > Weaviate > PostgreSQL > 内存存储
    let storage = lumos::vector::auto().await?;
    
    println!("使用后端: {}", storage.backend_info().name);
    Ok(())
}
```

### 手动指定向量数据库

```rust
use lumos::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Qdrant
    let qdrant = lumos::vector::qdrant("http://localhost:6334").await?;
    
    // Weaviate
    let weaviate = lumos::vector::weaviate("http://localhost:8080").await?;
    
    // PostgreSQL
    let postgres = lumos::vector::postgres().await?;
    
    // 内存存储
    let memory = lumos::vector::memory().await?;
    
    Ok(())
}
```

### 使用构建器模式

```rust
use lumos::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let storage = lumos::vector::builder()
        .backend("qdrant")
        .url("http://localhost:6334")
        .batch_size(1000)
        .build()
        .await?;
    
    Ok(())
}
```

### 完整的向量操作示例

```rust
use lumos::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建存储
    let storage = lumos::vector::auto().await?;
    
    // 创建索引
    let config = IndexConfig::new("documents", 384)
        .with_metric(SimilarityMetric::Cosine);
    storage.create_index(config).await?;
    
    // 插入文档
    let documents = vec![
        Document::new("doc1", "Hello world")
            .with_embedding(vec![0.1; 384])
            .with_metadata("category", "greeting"),
        Document::new("doc2", "Goodbye world")
            .with_embedding(vec![0.2; 384])
            .with_metadata("category", "farewell"),
    ];
    
    let ids = storage.upsert_documents("documents", documents).await?;
    println!("插入文档: {:?}", ids);
    
    // 向量搜索
    let query = vec![0.15; 384];
    let request = SearchRequest::new("documents", query)
        .with_top_k(5)
        .with_filter(FilterCondition::Eq("category".to_string(), "greeting".into()));
    
    let results = storage.search(request).await?;
    for result in results.results {
        println!("找到文档: {} (相似度: {:.4})", result.id, result.score);
    }
    
    Ok(())
}
```

## 🔧 配置选项

### Qdrant配置

```rust
use lumos::vector::qdrant::QdrantConfig;

let config = QdrantConfig::new("http://localhost:6334")
    .with_api_key("your-api-key".to_string())
    .with_timeout(30)
    .with_batch_size(1000);

let storage = QdrantStorage::with_config(config).await?;
```

### Weaviate配置

```rust
use lumos::vector::weaviate::WeaviateConfig;

let config = WeaviateConfig::new("http://localhost:8080")
    .with_api_key("your-api-key".to_string())
    .with_class_prefix("lumos".to_string())
    .with_batch_size(500);

let storage = WeaviateStorage::with_config(config).await?;
```

### PostgreSQL配置

```rust
use lumos::vector::postgres::PostgresConfig;

let config = PostgresConfig::new("postgresql://user:pass@localhost/db".to_string())
    .with_pool_size(10)
    .with_schema("vector_storage".to_string());

let storage = PostgresStorage::with_config(config).await?;
```

## 🧪 测试

### 运行集成测试

```bash
# 启动向量数据库
./scripts/vector-dbs.sh start

# 设置环境变量
source .env.vector-dbs

# 运行测试
cargo test vector_integration_test --features vector-all -- --nocapture
```

### 运行示例

```bash
# 运行向量数据库示例
cargo run --example vector_databases --features vector-all
```

## 🐳 Docker部署

### 生产环境配置

```yaml
# docker-compose.prod.yml
version: '3.8'
services:
  qdrant:
    image: qdrant/qdrant:v1.7.4
    environment:
      - QDRANT__SERVICE__HTTP_PORT=6333
      - QDRANT__SERVICE__GRPC_PORT=6334
    volumes:
      - qdrant_data:/qdrant/storage
    deploy:
      resources:
        limits:
          memory: 2G
        reservations:
          memory: 1G

  app:
    build: .
    environment:
      - QDRANT_URL=http://qdrant:6334
      - RUST_LOG=info
    depends_on:
      - qdrant
```

## 🔍 性能优化

### 批量操作

```rust
// 批量插入文档
let batch_size = 1000;
for chunk in documents.chunks(batch_size) {
    storage.upsert_documents("index", chunk.to_vec()).await?;
}
```

### 连接池配置

```rust
let config = PostgresConfig::new(database_url)
    .with_pool_size(20)  // 增加连接池大小
    .with_timeout(60);   // 增加超时时间
```

### 索引优化

```rust
let config = IndexConfig::new("documents", 384)
    .with_metric(SimilarityMetric::Cosine)
    .with_metadata("hnsw_ef_construction", 200)  // HNSW参数
    .with_metadata("hnsw_m", 16);
```

## 🚨 故障排除

### 常见问题

1. **连接失败**
   ```bash
   # 检查服务状态
   ./scripts/vector-dbs.sh status
   
   # 查看日志
   ./scripts/vector-dbs.sh logs qdrant
   ```

2. **特性未启用**
   ```toml
   # 确保在Cargo.toml中启用了相应特性
   lumos = { features = ["vector-qdrant"] }
   ```

3. **环境变量未设置**
   ```bash
   # 检查环境变量
   echo $QDRANT_URL
   echo $WEAVIATE_URL
   echo $DATABASE_URL
   ```

### 健康检查

```rust
// 检查向量数据库健康状态
match storage.health_check().await {
    Ok(_) => println!("✅ 数据库健康"),
    Err(e) => println!("❌ 数据库异常: {}", e),
}
```

## 📚 更多资源

- [Qdrant官方文档](https://qdrant.tech/documentation/)
- [Weaviate官方文档](https://weaviate.io/developers/weaviate)
- [pgvector文档](https://github.com/pgvector/pgvector)
- [Lumos API文档](https://docs.rs/lumos)

## 🤝 贡献

欢迎为向量数据库集成贡献代码！请查看[贡献指南](../CONTRIBUTING.md)了解更多信息。
