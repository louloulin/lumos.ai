# 🚀 LumosAI Milvus Integration

High-performance Milvus vector database integration for LumosAI, providing enterprise-grade distributed vector storage capabilities.

## ✨ Features

### 🏗️ Enterprise Architecture
- **Distributed Storage**: Horizontal scaling with sharding and replication
- **Cloud Native**: Kubernetes-ready with container orchestration
- **Multi-tenancy**: Collection-based isolation and resource management
- **ACID Transactions**: Consistency guarantees for critical operations

### 🚀 High Performance
- **Optimized Indexing**: Multiple index types (IVF, HNSW, ANNOY, etc.)
- **Batch Operations**: High-throughput bulk insert and query operations
- **Smart Caching**: Intelligent caching for frequently accessed data
- **Parallel Processing**: Concurrent operations for maximum throughput

### 🔍 Advanced Search
- **Vector Similarity**: Multiple similarity metrics (Cosine, Euclidean, Dot Product)
- **Metadata Filtering**: Complex boolean expressions for precise filtering
- **Hybrid Search**: Combine vector similarity with metadata constraints
- **Real-time Updates**: Support for real-time data ingestion and querying

### ☁️ Cloud Integration
- **Multi-cloud Support**: Deploy on AWS, Azure, GCP, or on-premises
- **Auto-scaling**: Dynamic resource allocation based on workload
- **Monitoring**: Built-in metrics and observability features
- **Backup & Recovery**: Automated backup and disaster recovery

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
lumosai-vector-milvus = "0.1.0"
lumosai-vector-core = "0.1.0"
```

### Basic Usage

```rust
use lumosai_vector_milvus::{MilvusStorage, MilvusConfig};
use lumosai_vector_core::{traits::VectorStorage, types::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Milvus storage
    let config = MilvusConfig::new("http://localhost:19530")
        .with_database("my_database")
        .with_auth("username", "password");
    let storage = MilvusStorage::new(config).await?;
    
    // Create a collection
    let index_config = IndexConfig::new("documents", 384)
        .with_metric(SimilarityMetric::Cosine);
    storage.create_index(index_config).await?;
    
    // Insert documents
    let docs = vec![
        Document::new("doc1", "Hello world")
            .with_embedding(vec![0.1; 384])
            .with_metadata("category", "greeting"),
    ];
    storage.upsert_documents("documents", docs).await?;
    
    // Search vectors
    let search_request = SearchRequest {
        index_name: "documents".to_string(),
        vector: vec![0.1; 384],
        top_k: 10,
        similarity_metric: Some(SimilarityMetric::Cosine),
        filter: None,
        include_metadata: true,
    };
    let results = storage.search(search_request).await?;
    
    Ok(())
}
```

## 🔧 Configuration

### Basic Configuration

```rust
use lumosai_vector_milvus::{MilvusConfig, MilvusConfigBuilder};

// Simple configuration
let config = MilvusConfig::new("http://localhost:19530");

// Advanced configuration
let config = MilvusConfigBuilder::new("http://localhost:19530")
    .database("production_db")
    .auth("admin", "secure_password")
    .batch_size(2000)
    .consistency_level(ConsistencyLevel::Strong)
    .shards_num(4)
    .replica_number(2)
    .build()?;
```

### Authentication

```rust
// Username/password authentication
let config = MilvusConfig::new("http://localhost:19530")
    .with_auth("username", "password");

// Token-based authentication
let config = MilvusConfig::new("http://localhost:19530")
    .with_token("your-auth-token");
```

### Performance Tuning

```rust
use lumosai_vector_milvus::config::*;

let config = MilvusConfigBuilder::new("http://localhost:19530")
    .batch_size(1000)                    // Batch size for bulk operations
    .default_index_type(IndexType::HNSW) // Default index type
    .consistency_level(ConsistencyLevel::Eventually) // Consistency level
    .build()?;
```

## 📊 Index Types

Milvus supports multiple index types optimized for different use cases:

### Vector Indexes

| Index Type | Use Case | Memory | Query Speed | Build Time |
|------------|----------|---------|-------------|------------|
| **FLAT** | Small datasets, exact search | High | Fast | Instant |
| **IVF_FLAT** | Balanced performance | Medium | Medium | Fast |
| **IVF_SQ8** | Memory-optimized | Low | Medium | Fast |
| **IVF_PQ** | Large datasets, memory-efficient | Low | Fast | Medium |
| **HNSW** | High-performance search | High | Very Fast | Slow |
| **ANNOY** | Read-heavy workloads | Medium | Fast | Medium |

### Index Configuration

```rust
use lumosai_vector_milvus::config::*;

// HNSW index for high performance
let config = MilvusConfigBuilder::new("http://localhost:19530")
    .default_index_type(IndexType::HNSW)
    .build()?;

// IVF_PQ for memory efficiency
let config = MilvusConfigBuilder::new("http://localhost:19530")
    .default_index_type(IndexType::IVF_PQ)
    .build()?;
```

## 🔍 Advanced Search

### Metadata Filtering

```rust
use lumosai_vector_core::types::*;

// Simple equality filter
let filter = FilterCondition::Eq(
    "category".to_string(),
    MetadataValue::String("technology".to_string())
);

// Complex boolean filter
let filter = FilterCondition::And(vec![
    FilterCondition::Eq("category".to_string(), MetadataValue::String("tech".to_string())),
    FilterCondition::Gt("score".to_string(), MetadataValue::Integer(80)),
    FilterCondition::Or(vec![
        FilterCondition::Contains("content".to_string(), "AI".to_string()),
        FilterCondition::Contains("content".to_string(), "ML".to_string()),
    ]),
]);

let search_request = SearchRequest {
    index_name: "documents".to_string(),
    vector: query_vector,
    top_k: 10,
    filter: Some(filter),
    include_metadata: true,
    similarity_metric: Some(SimilarityMetric::Cosine),
};
```

### Similarity Metrics

```rust
// Cosine similarity (recommended for normalized vectors)
SimilarityMetric::Cosine

// Euclidean distance (L2)
SimilarityMetric::Euclidean

// Dot product (for specific use cases)
SimilarityMetric::DotProduct

// Manhattan distance (L1)
SimilarityMetric::Manhattan
```

## 📈 Performance Optimization

### Batch Operations

```rust
// Batch insert for high throughput
let batch_size = 1000;
for chunk in documents.chunks(batch_size) {
    storage.upsert_documents("collection", chunk.to_vec()).await?;
}

// Parallel search operations
let futures: Vec<_> = queries.into_iter().map(|query| {
    storage.search(query)
}).collect();
let results = futures::future::join_all(futures).await;
```

### Connection Optimization

```rust
let config = MilvusConfigBuilder::new("http://localhost:19530")
    .batch_size(2000)                    // Larger batches
    .consistency_level(ConsistencyLevel::Eventually) // Relaxed consistency
    .build()?;
```

### Index Optimization

```rust
// For high-dimensional vectors (>512D)
IndexType::IVF_PQ

// For low-latency queries
IndexType::HNSW

// For memory-constrained environments
IndexType::IVF_SQ8
```

## 🏗️ Production Deployment

### Docker Setup

```bash
# Start Milvus standalone
docker run -d \
  --name milvus \
  -p 19530:19530 \
  -p 9091:9091 \
  -v milvus_data:/var/lib/milvus \
  milvusdb/milvus:latest

# Start with custom configuration
docker run -d \
  --name milvus \
  -p 19530:19530 \
  -v $(pwd)/milvus.yaml:/milvus/configs/milvus.yaml \
  -v milvus_data:/var/lib/milvus \
  milvusdb/milvus:latest
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: milvus
spec:
  replicas: 3
  selector:
    matchLabels:
      app: milvus
  template:
    metadata:
      labels:
        app: milvus
    spec:
      containers:
      - name: milvus
        image: milvusdb/milvus:latest
        ports:
        - containerPort: 19530
        env:
        - name: MILVUS_CONFIG_PATH
          value: "/milvus/configs/milvus.yaml"
```

### High Availability

```rust
// Configure for high availability
let config = MilvusConfigBuilder::new("http://milvus-cluster:19530")
    .replica_number(3)                   // 3 replicas
    .shards_num(4)                       // 4 shards
    .consistency_level(ConsistencyLevel::Strong) // Strong consistency
    .build()?;
```

## 📊 Monitoring and Metrics

### Health Checks

```rust
// Regular health checks
match storage.health_check().await {
    Ok(_) => println!("Milvus is healthy"),
    Err(e) => println!("Health check failed: {}", e),
}

// Collection statistics
let info = storage.describe_index("collection_name").await?;
println!("Documents: {}", info.document_count);
println!("Storage size: {:?}", info.storage_size);
```

### Performance Metrics

```rust
// Backend information
let backend_info = storage.backend_info();
println!("Backend: {} v{}", backend_info.name, backend_info.version);
println!("Features: {:?}", backend_info.features);
```

## 🔧 Troubleshooting

### Common Issues

1. **Connection Failed**
   ```bash
   # Check if Milvus is running
   curl http://localhost:9091/health
   
   # Check logs
   docker logs milvus
   ```

2. **Authentication Error**
   ```rust
   // Verify credentials
   let config = MilvusConfig::new("http://localhost:19530")
       .with_auth("correct_username", "correct_password");
   ```

3. **Index Creation Failed**
   ```rust
   // Check dimension consistency
   let config = IndexConfig::new("collection", 384); // Must match embedding dimension
   ```

4. **Search Performance Issues**
   ```rust
   // Optimize index type
   let config = MilvusConfigBuilder::new("http://localhost:19530")
       .default_index_type(IndexType::HNSW) // For better search performance
       .build()?;
   ```

### Debug Mode

```rust
// Enable debug logging
env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
```

## 📚 Examples

- [`basic_usage.rs`](examples/basic_usage.rs) - Basic operations and setup
- [`batch_operations.rs`](examples/batch_operations.rs) - High-throughput batch processing
- [`collection_management.rs`](examples/collection_management.rs) - Advanced collection management

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](../../CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## 🔗 Links

- [Milvus Documentation](https://milvus.io/docs)
- [LumosAI Documentation](https://docs.lumosai.dev)
- [API Reference](https://docs.rs/lumosai-vector-milvus)

## 🆘 Support

- [GitHub Issues](https://github.com/louloulin/lumos.ai/issues)
- [Discord Community](https://discord.gg/lumosai)
- [Documentation](https://docs.lumosai.dev)

---

**Built with ❤️ by the LumosAI team**
