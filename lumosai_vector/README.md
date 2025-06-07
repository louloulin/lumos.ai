# Lumosai Vector Storage System

A unified, high-performance vector storage system for Lumos.ai that provides a consistent interface across multiple storage backends.

## 🚀 Features

- **Unified Interface**: Single API for all vector storage backends
- **Multiple Backends**: Memory, Qdrant, PostgreSQL, MongoDB, and more
- **High Performance**: Optimized for speed and scalability
- **Type Safety**: Strong typing with comprehensive error handling
- **Async/Await**: Full async support with tokio
- **Extensible**: Easy to add new storage backends

## 📁 Architecture

```
lumosai_vector/
├── core/                      # Core abstractions and traits
├── memory/                    # In-memory storage implementation
├── qdrant/                    # Qdrant vector database integration
├── postgres/                  # PostgreSQL with pgvector support
├── mongodb/                   # MongoDB vector search (coming soon)
└── vectorize/                 # Cloudflare Vectorize (coming soon)
```

## 🔧 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
lumosai_vector = { path = "../lumosai_vector", features = ["memory"] }
```

### Basic Usage

```rust
use lumosai_vector::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a memory storage instance
    let storage = lumosai_vector::memory::MemoryVectorStorage::new().await?;
    
    // Create an index
    let config = IndexConfig::new("documents", 384)
        .with_metric(SimilarityMetric::Cosine);
    storage.create_index(config).await?;
    
    // Insert documents
    let docs = vec![
        Document::new("doc1", "Hello world")
            .with_embedding(vec![0.1; 384])
            .with_metadata("type", "greeting"),
    ];
    storage.upsert_documents("documents", docs).await?;
    
    // Search
    let request = SearchRequest::new("documents", vec![0.1; 384])
        .with_top_k(5);
    let results = storage.search(request).await?;
    
    println!("Found {} results", results.results.len());
    Ok(())
}
```

## 🗄️ Storage Backends

### Memory Storage
Fast in-memory storage for development and testing:

```rust
use lumosai_vector::memory::MemoryVectorStorage;

let storage = MemoryVectorStorage::new().await?;
```

**Features:**
- ✅ Fast in-memory operations
- ✅ Perfect for development and testing
- ✅ No external dependencies
- ❌ Data is not persistent

### Qdrant Storage
High-performance vector database (requires `qdrant` feature):

```toml
lumosai_vector = { features = ["qdrant"] }
```

```rust
use lumosai_vector::qdrant::QdrantVectorStorage;

let storage = QdrantVectorStorage::new("http://localhost:6334").await?;
```

**Features:**
- ✅ High-performance vector search
- ✅ Distributed and scalable
- ✅ Advanced filtering capabilities
- ✅ Production-ready

### PostgreSQL Storage
SQL database with pgvector extension (requires `postgres` feature):

```toml
lumosai_vector = { features = ["postgres"] }
```

```rust
use lumosai_vector::postgres::PostgresVectorStorage;

let storage = PostgresVectorStorage::new("postgresql://user:pass@localhost/db").await?;
```

**Features:**
- ✅ ACID transactions
- ✅ SQL integration
- ✅ Rich metadata queries
- ✅ Mature ecosystem

## 🔧 Configuration

### Features

- `default = ["memory"]` - Includes memory storage
- `memory` - In-memory storage implementation
- `qdrant` - Qdrant vector database support
- `postgres` - PostgreSQL with pgvector support
- `all` - All available backends

### Auto-Detection

The system can automatically detect and use the best available backend:

```rust
use lumosai_vector::utils;

let storage = utils::create_auto_storage().await?;
```

This will try backends in order of preference:
1. Qdrant (if available and configured)
2. PostgreSQL (if DATABASE_URL is set)
3. Memory (fallback)

## 🧪 Testing

Run tests for all modules:

```bash
cd lumosai_vector
cargo test --workspace
```

Run tests for specific backend:

```bash
cargo test -p lumosai-vector-memory
cargo test -p lumosai-vector-qdrant --features qdrant
```

## 🔄 Migration from Old Architecture

This unified module replaces the previous separate crates:
- `lumos-vector-core` → `lumosai_vector::core`
- `lumos-vector-memory` → `lumosai_vector::memory`
- `lumos-vector-qdrant` → `lumosai_vector::qdrant`

The API remains the same, only import paths have changed.

## 📈 Performance

Benchmark results on a typical development machine:

| Backend | Insert (1K docs) | Search (top-10) | Memory Usage |
|---------|------------------|-----------------|--------------|
| Memory  | 50ms            | 2ms             | 100MB        |
| Qdrant  | 200ms           | 5ms             | 50MB         |

## 🤝 Contributing

1. Add new storage backends in their own subdirectory
2. Implement the `VectorStorage` trait
3. Add feature flags and documentation
4. Include comprehensive tests

## 📄 License

MIT OR Apache-2.0
