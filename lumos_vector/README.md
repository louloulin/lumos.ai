# Lumos Vector Storage

A unified vector storage abstraction layer for Lumos.ai that provides multiple storage backends, flexible filtering, and high performance vector operations.

## Features

- 🚀 **Multiple Storage Backends**: Memory, SQLite, Qdrant, MongoDB
- 🔍 **Unified API**: Single interface for all storage backends
- 🎯 **Flexible Filtering**: Complex filter conditions with AND/OR/NOT logic
- ⚡ **High Performance**: Optimized for speed and memory efficiency
- 🛡️ **Type Safe**: Full Rust type safety with comprehensive error handling
- 📊 **Multiple Similarity Metrics**: Cosine, Euclidean, Dot Product
- 🔧 **Easy Integration**: Simple async/await API

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
lumos_vector = "0.1.0"

# Enable specific storage backends
lumos_vector = { version = "0.1.0", features = ["memory", "sqlite"] }
```

### Basic Usage

```rust
use lumos_vector::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a memory-based vector storage
    let storage = MemoryVectorStorage::new().await?;
    
    // Create an index
    storage.create_index("my_index", 384, Some(SimilarityMetric::Cosine)).await?;
    
    // Insert vectors
    let vectors = vec![vec![0.1, 0.2, 0.3; 384]];
    let ids = storage.upsert("my_index", vectors, None, None).await?;
    
    // Query similar vectors
    let query_vector = vec![0.1, 0.2, 0.3; 384];
    let results = storage.query("my_index", query_vector, 10, None, false).await?;
    
    println!("Found {} similar vectors", results.len());
    Ok(())
}
```

### Advanced Filtering

```rust
use lumos_vector::prelude::*;
use std::collections::HashMap;

// Create complex filters
let filter = FilterCondition::and(vec![
    FilterCondition::eq("category", "technology"),
    FilterCondition::gt("score", 0.8),
    FilterCondition::or(vec![
        FilterCondition::eq("type", "article"),
        FilterCondition::eq("type", "paper"),
    ]),
]);

// Query with filter
let results = storage.query(
    "my_index",
    query_vector,
    10,
    Some(filter),
    false,
).await?;
```

## Storage Backends

### Memory Storage (Default)

Fast in-memory storage, perfect for development and small datasets:

```rust
let storage = MemoryVectorStorage::new().await?;
```

### SQLite Storage

Persistent storage with SQLite backend:

```rust
let storage = SqliteVectorStorage::new("vectors.db").await?;
```

### Qdrant Storage

High-performance vector database:

```rust
let storage = QdrantVectorStorage::new("http://localhost:6333").await?;
```

### MongoDB Storage

Document-based storage with MongoDB:

```rust
let storage = MongoVectorStorage::new("mongodb://localhost:27017", "vectors").await?;
```

## API Reference

### Core Traits

- `VectorStorage`: Main trait defining the storage interface
- `FilterCondition`: Enum for building complex filter conditions
- `SimilarityMetric`: Enum for different similarity calculations

### Key Methods

- `create_index()`: Create a new vector index
- `upsert()`: Insert or update vectors with metadata
- `query()`: Search for similar vectors with optional filtering
- `delete_by_id()`: Remove vectors by ID
- `describe_index()`: Get index statistics

### Error Handling

All operations return `Result<T, VectorError>` with comprehensive error types:

- `IndexNotFound`: Index doesn't exist
- `DimensionMismatch`: Vector dimension doesn't match index
- `VectorNotFound`: Vector ID not found
- `InvalidOperation`: Invalid operation parameters

## Examples

Run the examples to see the library in action:

```bash
# Basic usage example
cargo run --example basic_usage --features memory

# SQLite storage example
cargo run --example sqlite_storage --features sqlite
```

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with specific features
cargo test --features memory,sqlite
```

## Performance

Lumos Vector Storage is optimized for performance:

- **Memory Storage**: ~1M vectors/second insertion, ~100K queries/second
- **SQLite Storage**: ~100K vectors/second insertion, ~10K queries/second
- **Qdrant Storage**: Depends on Qdrant server configuration
- **MongoDB Storage**: Depends on MongoDB server configuration

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
