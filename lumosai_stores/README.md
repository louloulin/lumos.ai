# Lumosai Stores

Vector and data storage implementations for the Lumosai framework.

## Overview

This crate provides implementations of vector stores for use with Lumosai's RAG (Retrieval Augmented Generation) functionality. It includes support for:

- [Qdrant](https://qdrant.tech/) - A high-performance vector database
- [PostgreSQL](https://www.postgresql.org/) with [pgvector](https://github.com/pgvector/pgvector) - Vector similarity search for Postgres
- [Cloudflare Vectorize](https://developers.cloudflare.com/vectorize/) - Cloudflare's managed vector database service

## Installation

Add `lumosai_stores` to your project by adding it to your `Cargo.toml`:

```toml
[dependencies]
lumosai_stores = { path = "../lumosai_stores", features = ["qdrant"] }
```

The available features are:
- `qdrant` - Enables the Qdrant vector store
- `postgres` - Enables the PostgreSQL vector store
- `vectorize` - Enables the Cloudflare Vectorize store
- `all` - Enables all vector stores

## Usage

### Qdrant

```rust
use lumosai_stores::qdrant::QdrantStore;
use lumosai_stores::vector::{CreateIndexParams, UpsertParams, QueryParams, VectorStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Qdrant client
    let store = QdrantStore::new("http://localhost:6333", None).await?;
    
    // Create a vector index
    let create_params = CreateIndexParams {
        index_name: "my_index".to_string(),
        dimension: 128,
        metric: "cosine".to_string(),
    };
    store.create_index(create_params).await?;
    
    // Upsert vectors
    let vectors = vec![
        vec![0.1, 0.2, /* ... */], // 128-dimensional vector
        vec![0.3, 0.4, /* ... */], // 128-dimensional vector
    ];
    let metadata = vec![
        [("title".to_string(), "Document 1".into())].into_iter().collect(),
        [("title".to_string(), "Document 2".into())].into_iter().collect(),
    ];
    
    let upsert_params = UpsertParams {
        index_name: "my_index".to_string(),
        vectors,
        metadata,
        ids: None, // Auto-generate IDs
    };
    
    let ids = store.upsert(upsert_params).await?;
    println!("Inserted vector IDs: {:?}", ids);
    
    // Query vectors
    let query_params = QueryParams {
        index_name: "my_index".to_string(),
        query_vector: vec![0.1, 0.2, /* ... */], // 128-dimensional query vector
        top_k: 5,
        filter: None,
        include_vector: false,
    };
    
    let results = store.query(query_params).await?;
    println!("Query results: {:?}", results);
    
    Ok(())
}
```

### PostgreSQL

```rust
use lumosai_stores::postgres::PostgresVectorStore;
use lumosai_stores::vector::{CreateIndexParams, UpsertParams, QueryParams, VectorStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a PostgreSQL client
    let store = PostgresVectorStore::new("postgres://user:password@localhost:5432/vectors").await?;
    
    // Create a vector index
    let create_params = CreateIndexParams {
        index_name: "my_index".to_string(),
        dimension: 128,
        metric: "cosine".to_string(),
    };
    store.create_index(create_params).await?;
    
    // Rest of the code is similar to Qdrant example...
    
    Ok(())
}
```

### Cloudflare Vectorize

```rust
use lumosai_stores::vectorize::VectorizeStore;
use lumosai_stores::vector::{CreateIndexParams, UpsertParams, QueryParams, VectorStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Vectorize client with API token
    let api_token = std::env::var("CF_API_TOKEN")?;
    let account_id = std::env::var("CF_ACCOUNT_ID")?;
    
    let store = VectorizeStore::new(&api_token, &account_id).await?;
    
    // Create a vector index
    let create_params = CreateIndexParams {
        index_name: "my_index".to_string(),
        dimension: 128,
        metric: "cosine".to_string(),
    };
    store.create_index(create_params).await?;
    
    // Rest of the code is similar to Qdrant example...
    
    Ok(())
}
```

## Using with RAG

The vector stores can be used with Lumosai's RAG functionality through the `VectorStoreRetriever` adapter:

```rust
use lumosai_rag::embedding::provider::EmbeddingProvider;
use lumosai_rag::types::{Document, Metadata, RetrievalOptions};
use lumosai_stores::rag::VectorStoreRetriever;
use lumosai_stores::qdrant::QdrantStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an embedding provider (using OpenAI as an example)
    let embedding_provider = OpenAIEmbedder::new("your-openai-api-key").await?;
    
    // Create the vector store
    let store = QdrantStore::new("http://localhost:6333", None).await?;
    
    // Create a retriever adapter
    // Parameters: store, index_name, dimensions, metric
    let mut retriever = VectorStoreRetriever::new(
        store,
        "my_documents", 
        1536,  // OpenAI embedding dimensions
        "cosine"
    );
    
    // Ensure the index exists
    retriever.ensure_index().await?;
    
    // Create and embed a document
    let mut document = Document {
        id: "doc1".to_string(),
        content: "Vector databases are essential for similarity search.".to_string(),
        metadata: Metadata::new().with_source("example"),
        embedding: None,
    };
    
    // Generate embedding for the document
    embedding_provider.embed_document(&mut document).await?;
    
    // Add the document to the retriever
    retriever.add_document(document).await?;
    
    // Query similar documents
    let query = "What is a vector database?";
    let options = RetrievalOptions::default();
    
    let results = retriever.query_by_text(query, &options, &embedding_provider).await?;
    
    for (i, doc) in results.documents.iter().enumerate() {
        println!("Document {}: {}", i+1, doc.content);
        println!("Score: {}", results.scores.as_ref().unwrap()[i]);
        println!("---");
    }
    
    Ok(())
}

This adapter implements the `VectorStore` trait from `lumosai_rag`, allowing you to use any of our vector store implementations with the RAG functionality.

## Contributing

Contributions are welcome! If you'd like to add support for another vector store, please open a pull request.

## License

This project is licensed under the MIT License. 