# 向量数据库 API 参考文档

## 概述

本文档提供了LumosAI向量数据库模块的完整API参考，包括核心接口、性能优化功能和使用示例。

## 核心接口

### VectorStorage Trait

所有向量存储实现的核心接口。

```rust
#[async_trait]
pub trait VectorStorage: Send + Sync {
    // 索引管理
    async fn create_index(&self, config: IndexConfig) -> Result<()>;
    async fn delete_index(&self, index_name: &str) -> Result<()>;
    async fn list_indexes(&self) -> Result<Vec<String>>;
    async fn describe_index(&self, index_name: &str) -> Result<IndexInfo>;
    
    // 文档操作
    async fn upsert_documents(&self, index_name: &str, documents: Vec<Document>) -> Result<Vec<DocumentId>>;
    async fn get_documents(&self, index_name: &str, ids: Vec<DocumentId>, include_vectors: bool) -> Result<Vec<Document>>;
    async fn update_document(&self, index_name: &str, document: Document) -> Result<()>;
    async fn delete_documents(&self, index_name: &str, ids: Vec<DocumentId>) -> Result<()>;
    
    // 搜索功能
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse>;
    
    // 健康检查
    async fn health_check(&self) -> Result<()>;
    fn backend_info(&self) -> BackendInfo;
}
```

### 核心数据类型

#### Document
```rust
pub struct Document {
    pub id: DocumentId,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: HashMap<String, MetadataValue>,
}

impl Document {
    pub fn new(id: impl Into<DocumentId>, content: impl Into<String>) -> Self;
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self;
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<MetadataValue>) -> Self;
}
```

#### SearchRequest
```rust
pub struct SearchRequest {
    pub index_name: String,
    pub query: SearchQuery,
    pub top_k: usize,
    pub filter: Option<FilterCondition>,
    pub include_metadata: bool,
    pub include_vectors: bool,
    pub options: HashMap<String, String>,
}
```

#### SearchQuery
```rust
pub enum SearchQuery {
    Vector(Vec<f32>),
    Text(String),
    Hybrid { vector: Vec<f32>, text: String, alpha: f32 },
}
```

## 性能优化 API

### 连接池配置

```rust
pub struct ConnectionPoolConfig {
    /// 最大连接数
    pub max_connections: usize,
    /// 最小连接数  
    pub min_connections: usize,
    /// 连接超时时间
    pub connection_timeout: Duration,
    /// 空闲连接超时时间
    pub idle_timeout: Duration,
    /// 连接重试次数
    pub max_retries: u32,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            max_retries: 3,
        }
    }
}
```

### 缓存配置

```rust
pub struct CacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 缓存过期时间
    pub ttl: Duration,
    /// 是否启用LRU淘汰策略
    pub enable_lru: bool,
    /// 缓存命中率统计间隔
    pub stats_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl: Duration::from_secs(3600),
            enable_lru: true,
            stats_interval: Duration::from_secs(60),
        }
    }
}
```

### 性能监控

```rust
pub struct PerformanceMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub average_response_time: Duration,
    pub min_response_time: Duration,
    pub max_response_time: Duration,
    pub operations_per_second: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

pub struct CacheStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
    pub current_size: usize,
    pub hit_rate: f64,
}
```

## 内存存储 API

### MemoryVectorStorage

```rust
impl MemoryVectorStorage {
    /// 创建新的内存向量存储
    pub async fn new() -> Result<Self>;
    
    /// 使用自定义配置创建
    pub async fn with_config(config: MemoryConfig) -> Result<Self>;
    
    /// 获取性能指标
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics;
    
    /// 获取缓存统计
    pub async fn get_cache_stats(&self) -> CacheStats;
    
    /// 清理内存（在内存压力时）
    pub async fn cleanup(&self) -> Result<()>;
    
    /// 获取内存使用情况
    pub async fn memory_usage(&self) -> u64;
    
    /// 获取存储统计信息
    pub async fn get_stats(&self) -> StorageStats;
}
```

### 配置选项

```rust
pub struct MemoryConfig {
    /// 默认相似度度量
    pub default_similarity: SimilarityMetric,
    /// 内存阈值（MB），超过时触发清理
    pub memory_threshold_mb: Option<u32>,
    /// 是否启用统计收集
    pub enable_stats: bool,
    /// 索引配置
    pub index_config: IndexConfig,
}
```

## 使用示例

### 基础操作示例

```rust
use lumosai_vector::memory::MemoryVectorStorage;
use lumosai_vector_core::{VectorStorage, IndexConfig, Document, SearchRequest, SearchQuery};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 创建存储实例
    let storage = MemoryVectorStorage::new().await?;
    
    // 2. 创建索引
    let config = IndexConfig::new("articles", 384);
    storage.create_index(config).await?;
    
    // 3. 准备文档
    let documents = vec![
        Document::new("doc1", "人工智能的发展历程")
            .with_embedding(vec![0.1, 0.2, 0.3, /* ... */])
            .with_metadata("category", "technology")
            .with_metadata("author", "张三"),
        Document::new("doc2", "机器学习算法介绍")
            .with_embedding(vec![0.2, 0.3, 0.4, /* ... */])
            .with_metadata("category", "technology")
            .with_metadata("author", "李四"),
    ];
    
    // 4. 插入文档
    let doc_ids = storage.upsert_documents("articles", documents).await?;
    println!("插入了 {} 个文档", doc_ids.len());
    
    // 5. 向量搜索
    let query_vector = vec![0.15, 0.25, 0.35, /* ... */];
    let search_request = SearchRequest {
        index_name: "articles".to_string(),
        query: SearchQuery::Vector(query_vector),
        top_k: 5,
        filter: None,
        include_metadata: true,
        include_vectors: false,
        options: HashMap::new(),
    };
    
    let results = storage.search(search_request).await?;
    for result in results.results {
        println!("文档ID: {}, 相似度: {:.4}", result.id, result.score);
        if let Some(metadata) = result.metadata {
            println!("  分类: {:?}", metadata.get("category"));
            println!("  作者: {:?}", metadata.get("author"));
        }
    }
    
    Ok(())
}
```

### 性能监控示例

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn performance_monitoring_example() -> Result<()> {
    let storage = MemoryVectorStorage::new().await?;
    
    // 执行一些操作...
    // ... 创建索引、插入文档、执行搜索 ...
    
    // 等待一段时间让统计数据积累
    sleep(Duration::from_secs(1)).await;
    
    // 获取性能指标
    let metrics = storage.get_performance_metrics().await;
    println!("性能指标:");
    println!("  总操作数: {}", metrics.total_operations);
    println!("  成功操作数: {}", metrics.successful_operations);
    println!("  平均响应时间: {:?}", metrics.average_response_time);
    println!("  最小响应时间: {:?}", metrics.min_response_time);
    println!("  最大响应时间: {:?}", metrics.max_response_time);
    
    // 获取缓存统计
    let cache_stats = storage.get_cache_stats().await;
    println!("缓存统计:");
    println!("  总请求数: {}", cache_stats.total_requests);
    println!("  缓存命中数: {}", cache_stats.cache_hits);
    println!("  缓存未命中数: {}", cache_stats.cache_misses);
    println!("  命中率: {:.2}%", cache_stats.hit_rate * 100.0);
    println!("  当前缓存大小: {}", cache_stats.current_size);
    
    Ok(())
}
```

### 并发操作示例

```rust
use std::sync::Arc;
use tokio::task::JoinSet;

async fn concurrent_operations_example() -> Result<()> {
    let storage = Arc::new(MemoryVectorStorage::new().await?);
    
    // 创建索引
    let config = IndexConfig::new("concurrent_test", 128);
    storage.create_index(config).await?;
    
    // 并发插入文档
    let mut join_set = JoinSet::new();
    
    for i in 0..10 {
        let storage_clone = Arc::clone(&storage);
        join_set.spawn(async move {
            let doc = Document::new(format!("doc_{}", i), format!("内容 {}", i))
                .with_embedding(vec![i as f32; 128])
                .with_metadata("batch", i.to_string());
            
            storage_clone.upsert_documents("concurrent_test", vec![doc]).await
        });
    }
    
    // 等待所有任务完成
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(_)) => println!("文档插入成功"),
            Ok(Err(e)) => println!("文档插入失败: {}", e),
            Err(e) => println!("任务执行失败: {}", e),
        }
    }
    
    // 验证所有文档都已插入
    let all_indexes = storage.list_indexes().await?;
    println!("当前索引: {:?}", all_indexes);
    
    Ok(())
}
```

## 错误处理

### VectorError 类型

```rust
#[derive(Debug, thiserror::Error)]
pub enum VectorError {
    #[error("Index not found: {0}")]
    IndexNotFound(String),
    
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
}
```

### 错误处理最佳实践

```rust
async fn error_handling_example() {
    let storage = MemoryVectorStorage::new().await.unwrap();
    
    // 处理索引不存在的情况
    match storage.describe_index("nonexistent").await {
        Ok(info) => println!("索引信息: {:?}", info),
        Err(VectorError::IndexNotFound(name)) => {
            println!("索引 '{}' 不存在，正在创建...", name);
            let config = IndexConfig::new(&name, 384);
            storage.create_index(config).await.unwrap();
        },
        Err(e) => println!("其他错误: {}", e),
    }
}
```

## 最佳实践

### 1. 索引设计
- 选择合适的向量维度
- 根据数据量选择合适的相似度度量
- 合理设置索引配置参数

### 2. 性能优化
- 启用缓存以提高重复查询性能
- 监控性能指标，及时发现瓶颈
- 合理配置连接池参数

### 3. 内存管理
- 设置合适的内存阈值
- 定期清理不必要的缓存
- 监控内存使用情况

### 4. 错误处理
- 始终处理可能的错误情况
- 使用适当的重试机制
- 记录详细的错误日志

## 配置参考

### 生产环境推荐配置

```rust
// 高性能配置
let cache_config = CacheConfig {
    max_entries: 10000,
    ttl: Duration::from_secs(7200), // 2小时
    enable_lru: true,
    stats_interval: Duration::from_secs(300), // 5分钟
};

let pool_config = ConnectionPoolConfig {
    max_connections: 50,
    min_connections: 10,
    connection_timeout: Duration::from_secs(30),
    idle_timeout: Duration::from_secs(600), // 10分钟
    max_retries: 5,
};
```

### 开发环境配置

```rust
// 开发环境配置
let cache_config = CacheConfig {
    max_entries: 100,
    ttl: Duration::from_secs(300), // 5分钟
    enable_lru: true,
    stats_interval: Duration::from_secs(60),
};

let pool_config = ConnectionPoolConfig {
    max_connections: 5,
    min_connections: 1,
    connection_timeout: Duration::from_secs(10),
    idle_timeout: Duration::from_secs(60),
    max_retries: 3,
};
```
