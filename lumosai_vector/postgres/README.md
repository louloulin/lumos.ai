# Lumos Vector PostgreSQL Storage

高性能的PostgreSQL向量存储实现，基于pgvector扩展，为Lumos.ai向量存储系统提供企业级的持久化存储能力。

## 特性

- 🚀 **高性能**: 基于pgvector扩展的原生向量操作
- 🏗️ **企业级**: 支持连接池、事务、批量操作
- 🔍 **向量索引**: 支持HNSW和IVFFlat索引类型
- 📊 **元数据存储**: 使用JSONB高效存储和查询元数据
- ⚙️ **灵活配置**: 丰富的配置选项和性能调优
- 🔒 **类型安全**: 完整的Rust类型安全保证

## 快速开始

### 1. 安装依赖

```toml
[dependencies]
lumosai-vector-postgres = "0.1.0"
lumosai-vector-core = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### 2. 设置PostgreSQL

```bash
# 安装PostgreSQL和pgvector扩展
# Ubuntu/Debian:
sudo apt-get install postgresql postgresql-contrib
sudo apt-get install postgresql-14-pgvector

# 或使用Docker:
docker run -d \
  --name postgres-vector \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=lumos_vector \
  -p 5432:5432 \
  pgvector/pgvector:pg16
```

### 3. 创建数据库和扩展

```sql
-- 连接到PostgreSQL
psql -h localhost -U postgres -d lumos_vector

-- 创建pgvector扩展
CREATE EXTENSION vector;

-- 验证安装
SELECT * FROM pg_extension WHERE extname = 'vector';
```

### 4. 基本使用

```rust
use lumosai_vector_core::prelude::*;
use lumosai_vector_postgres::PostgresVectorStorage;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // 连接到PostgreSQL
    let storage = PostgresVectorStorage::new(
        "postgresql://postgres:password@localhost/lumos_vector"
    ).await?;
    
    // 创建向量索引
    let index_config = IndexConfig {
        name: "documents".to_string(),
        dimension: 384,
        metric: SimilarityMetric::Cosine,
        metadata: HashMap::new(),
    };
    storage.create_index(index_config).await?;
    
    // 创建文档
    let documents = vec![
        Document {
            id: "doc1".to_string(),
            content: "人工智能正在改变世界".to_string(),
            embedding: Some(vec![0.1; 384]),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), MetadataValue::String("tech".to_string()));
                meta
            },
        }
    ];
    
    // 存储文档
    storage.upsert_documents("documents", documents).await?;
    
    // 向量搜索
    let search_request = SearchRequest {
        index_name: "documents".to_string(),
        query: SearchQuery::Vector(vec![0.1; 384]),
        top_k: 10,
        filter: None,
        include_metadata: true,
        include_vectors: false,
    };
    
    let results = storage.search(search_request).await?;
    println!("找到 {} 个相似文档", results.results.len());
    
    Ok(())
}
```

## 高级配置

### 连接池配置

```rust
use lumosai_vector_postgres::{PostgresVectorStorage, PostgresConfig};
use lumosai_vector_postgres::config::{PoolConfig, PerformanceConfig, VectorIndexType};
use std::time::Duration;

let config = PostgresConfig::new("postgresql://localhost/lumos_vector")
    .with_pool(PoolConfig {
        max_connections: 20,
        min_connections: 5,
        connect_timeout: Duration::from_secs(30),
        idle_timeout: Some(Duration::from_secs(600)),
        max_lifetime: Some(Duration::from_secs(1800)),
    })
    .with_performance(PerformanceConfig {
        batch_size: 2000,
        index_type: VectorIndexType::Hnsw,
        index_params: Default::default(),
        use_prepared_statements: true,
    });

let storage = PostgresVectorStorage::with_config(config).await?;
```

### 向量索引优化

```rust
use lumosai_vector_postgres::config::{VectorIndexType, HnswParams, IvfFlatParams};

// HNSW索引 - 高召回率
let hnsw_config = PerformanceConfig {
    index_type: VectorIndexType::Hnsw,
    index_params: IndexParams {
        hnsw: HnswParams {
            m: 16,              // 连接数
            ef_construction: 64, // 构建时候选列表大小
            ef_search: 40,       // 搜索时候选列表大小
        },
        ..Default::default()
    },
    ..Default::default()
};

// IVFFlat索引 - 高性能
let ivf_config = PerformanceConfig {
    index_type: VectorIndexType::IvfFlat,
    index_params: IndexParams {
        ivf_flat: IvfFlatParams {
            lists: 100,  // 聚类数量
            probes: 10,  // 搜索时探测的聚类数
        },
        ..Default::default()
    },
    ..Default::default()
};
```

## 性能基准测试

运行性能基准测试：

```bash
# 设置数据库连接
export DATABASE_URL="postgresql://postgres:password@localhost/bench_lumos_vector"

# 运行基准测试
cd lumosai_vector/postgres
cargo bench postgres_benchmark
```

### 典型性能指标

在标准硬件上（Intel i7, 16GB RAM, SSD）的性能表现：

| 操作 | 文档数量 | 维度 | 性能 |
|------|----------|------|------|
| 批量插入 | 1,000 | 384 | ~500ms |
| 向量搜索 | 10,000 | 384 | ~10ms (top-10) |
| 文档检索 | 100 | 384 | ~5ms |
| 批量删除 | 1,000 | 384 | ~100ms |

## 数据库架构

### 表结构

```sql
-- 自动创建的表结构示例
CREATE TABLE lumos_documents (
    id TEXT PRIMARY KEY,
    content TEXT,
    embedding vector(384),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 向量索引
CREATE INDEX lumos_documents_embedding_idx 
ON lumos_documents 
USING hnsw (embedding vector_cosine_ops) 
WITH (m = 16, ef_construction = 64);

-- 元数据索引
CREATE INDEX lumos_documents_metadata_idx 
ON lumos_documents 
USING GIN (metadata);
```

### 配置选项

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `max_connections` | 10 | 最大连接数 |
| `batch_size` | 1000 | 批量操作大小 |
| `index_type` | HNSW | 向量索引类型 |
| `auto_create_tables` | true | 自动创建表 |
| `auto_create_indexes` | true | 自动创建索引 |

## 故障排除

### 常见问题

1. **pgvector扩展未安装**
   ```
   错误: pgvector extension is not installed
   解决: CREATE EXTENSION vector;
   ```

2. **连接超时**
   ```
   错误: connection timed out
   解决: 增加 connect_timeout 配置
   ```

3. **内存不足**
   ```
   错误: out of memory
   解决: 减少 batch_size 或增加系统内存
   ```

### 监控和调试

```rust
// 启用调试日志
use tracing_subscriber;
tracing_subscriber::init();

// 健康检查
storage.health_check().await?;

// 获取后端信息
let info = storage.backend_info();
println!("Backend: {} v{}", info.name, info.version);
```

## 许可证

MIT OR Apache-2.0

## 贡献

欢迎提交Issue和Pull Request！请参考[贡献指南](../../CONTRIBUTING.md)。
