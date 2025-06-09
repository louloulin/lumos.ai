# LumosAI FastEmbed Integration

[![Crates.io](https://img.shields.io/crates/v/lumosai-vector-fastembed.svg)](https://crates.io/crates/lumosai-vector-fastembed)
[![Documentation](https://docs.rs/lumosai-vector-fastembed/badge.svg)](https://docs.rs/lumosai-vector-fastembed)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

FastEmbed integration for LumosAI vector storage, providing fast, local embedding generation without external API dependencies.

## 🚀 Features

- **Local Processing**: Generate embeddings locally without API calls
- **Multiple Models**: Support for various pre-trained models (BGE, MiniLM, E5)
- **High Performance**: Optimized for batch processing with configurable batch sizes
- **Multilingual Support**: Models supporting 100+ languages
- **Easy Integration**: Seamless integration with LumosAI vector storage
- **Memory Efficient**: Lazy loading and configurable caching
- **Type Safe**: Full Rust type safety with comprehensive error handling

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lumosai-vector-fastembed = "0.1.0"
lumosai-vector-core = "0.1.0"
```

## 🎯 Quick Start

### Basic Usage

```rust
use lumosai_vector_fastembed::{FastEmbedProvider, FastEmbedModel};
use lumosai_vector_core::traits::EmbeddingModel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create FastEmbed provider
    let provider = FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await?;
    
    // Generate single embedding
    let embedding = provider.embed_text("Hello, world!").await?;
    println!("Embedding dimensions: {}", embedding.len());
    
    // Generate batch embeddings
    let texts = vec![
        "First document".to_string(),
        "Second document".to_string(),
        "Third document".to_string(),
    ];
    let embeddings = provider.embed_batch(&texts).await?;
    println!("Generated {} embeddings", embeddings.len());
    
    Ok(())
}
```

### Custom Configuration

```rust
use lumosai_vector_fastembed::{
    FastEmbedProvider, FastEmbedModel, FastEmbedConfigBuilder
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom configuration
    let config = FastEmbedConfigBuilder::new()
        .max_batch_size(128)
        .show_download_progress(true)
        .num_threads(4)
        .cache_dir("/tmp/fastembed_models")
        .build();
    
    // Create provider with custom config
    let provider = FastEmbedProvider::new(FastEmbedModel::BGEBaseENV15, config).await?;
    
    // Use the provider...
    let embedding = provider.embed_text("Custom configuration example").await?;
    
    Ok(())
}
```

## 🤖 Available Models

### English Models

| Model | Dimensions | Max Length | Best For |
|-------|------------|------------|----------|
| `BGESmallENV15` | 384 | 512 | Fast general purpose |
| `BGEBaseENV15` | 768 | 512 | Balanced quality/speed |
| `BGELargeENV15` | 1024 | 512 | High quality |
| `AllMiniLML6V2` | 384 | 256 | Lightweight, fast |
| `AllMiniLML12V2` | 384 | 256 | Better than L6 |

### Multilingual Models

| Model | Dimensions | Languages | Best For |
|-------|------------|-----------|----------|
| `MultilingualE5Small` | 384 | 100+ | Multilingual apps |
| `MultilingualE5Base` | 768 | 100+ | High-quality multilingual |
| `MultilingualE5Large` | 1024 | 100+ | Best multilingual quality |

### Model Selection Guide

```rust
// For fast, general purpose English text
FastEmbedModel::BGESmallENV15

// For balanced performance and quality
FastEmbedModel::BGEBaseENV15

// For highest quality English embeddings
FastEmbedModel::BGELargeENV15

// For multilingual applications
FastEmbedModel::MultilingualE5Small

// For real-time applications
FastEmbedModel::AllMiniLML6V2
```

## 🔧 Configuration Options

### FastEmbedConfig

```rust
use lumosai_vector_fastembed::FastEmbedConfigBuilder;

let config = FastEmbedConfigBuilder::new()
    .max_batch_size(256)           // Maximum texts per batch
    .show_download_progress(true)   // Show model download progress
    .num_threads(4)                // Number of processing threads
    .cache_dir("/path/to/cache")   // Model cache directory
    .build();
```

### Performance Tuning

- **Batch Size**: Larger batches are more efficient but use more memory
- **Threads**: More threads can improve performance on multi-core systems
- **Cache Directory**: Store models on fast storage (SSD) for better performance

## 📊 Performance Benchmarks

### Throughput (texts/second)

| Model | Single | Batch (256) | Memory Usage |
|-------|--------|-------------|--------------|
| BGE Small | 50 | 800 | ~500MB |
| BGE Base | 30 | 500 | ~1GB |
| BGE Large | 20 | 300 | ~2GB |
| MiniLM L6 | 80 | 1200 | ~300MB |

*Benchmarks on Intel i7-10700K, 32GB RAM*

## 🌍 Multilingual Support

```rust
use lumosai_vector_fastembed::{FastEmbedProvider, FastEmbedModel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = FastEmbedProvider::with_model(
        FastEmbedModel::MultilingualE5Small
    ).await?;
    
    let multilingual_texts = vec![
        "Hello, how are you?".to_string(),           // English
        "Hola, ¿cómo estás?".to_string(),            // Spanish  
        "Bonjour, comment allez-vous?".to_string(),  // French
        "你好，你好吗？".to_string(),                    // Chinese
    ];
    
    let embeddings = provider.embed_batch(&multilingual_texts).await?;
    
    // Calculate cross-language similarity
    let similarity = cosine_similarity(&embeddings[0], &embeddings[1]);
    println!("English-Spanish similarity: {:.3}", similarity);
    
    Ok(())
}
```

## 🔍 Vector Search Example

```rust
use lumosai_vector_fastembed::{FastEmbedProvider, FastEmbedModel};

async fn semantic_search() -> Result<(), Box<dyn std::error::Error>> {
    let provider = FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await?;
    
    // Index documents
    let documents = vec![
        "Machine learning is a subset of AI",
        "Deep learning uses neural networks", 
        "Natural language processing handles text",
        "Computer vision processes images",
    ];
    
    let doc_embeddings = provider.embed_batch(&documents.iter().map(|s| s.to_string()).collect::<Vec<_>>()).await?;
    
    // Search query
    let query = "artificial intelligence algorithms";
    let query_embedding = provider.embed_text(query).await?;
    
    // Find most similar document
    let mut similarities = Vec::new();
    for (i, doc_emb) in doc_embeddings.iter().enumerate() {
        let sim = cosine_similarity(&query_embedding, doc_emb);
        similarities.push((i, sim));
    }
    
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    println!("Most similar: {} (similarity: {:.3})", 
             documents[similarities[0].0], similarities[0].1);
    
    Ok(())
}
```

## 🧪 Examples

Run the included examples:

```bash
# Basic embedding generation
cargo run --example basic_embedding

# Batch processing optimization
cargo run --example batch_embedding

# Vector search and similarity
cargo run --example vector_search
```

## 🔧 Integration with LumosAI Vector Storage

```rust
use lumosai_vector_fastembed::{FastEmbedProvider, FastEmbedModel};
use lumosai_vector_core::traits::VectorStorage;

// Use with any LumosAI vector storage backend
let embedding_provider = FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await?;
let vector_storage = /* your vector storage implementation */;

// The embedding provider implements the EmbeddingModel trait
// and can be used with any LumosAI vector storage backend
```

## 📚 Documentation

- [API Documentation](https://docs.rs/lumosai-vector-fastembed)
- [LumosAI Core Documentation](https://docs.rs/lumosai-vector-core)
- [FastEmbed Documentation](https://docs.rs/fastembed)

## 🤝 Contributing

Contributions are welcome! Please see our [Contributing Guide](../../CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## 🙏 Acknowledgments

- [FastEmbed](https://github.com/Anush008/fastembed-rs) for the excellent local embedding library
- [Hugging Face](https://huggingface.co/) for the pre-trained models
- The Rust community for amazing crates and tools
