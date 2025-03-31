use std::sync::Arc;
use std::error::Error;

use lumosai_rag::embedding::provider::EmbeddingProvider;
use lumosai_rag::types::{Document, Metadata, RetrievalOptions};
use lumosai_stores::rag::VectorStoreRetriever;

#[cfg(feature = "qdrant")]
use lumosai_stores::qdrant::QdrantStore;

#[cfg(feature = "postgres")]
use lumosai_stores::postgres::PostgresVectorStore;

#[cfg(feature = "vectorize")]
use lumosai_stores::vectorize::VectorizeStore;

// A simple mock embedding provider for the example
struct SimpleEmbedder;

#[async_trait::async_trait]
impl EmbeddingProvider for SimpleEmbedder {
    async fn embed_text(&self, text: &str) -> lumosai_rag::error::Result<Vec<f32>> {
        // Return a simple 4-dimensional embedding based on text length
        let text_len = text.len() as f32;
        Ok(vec![
            text_len * 0.01,
            text_len * 0.02,
            text_len * 0.03,
            text_len * 0.04,
        ])
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create an embedding provider
    let embedding_provider = Arc::new(SimpleEmbedder);
    
    // Example documents
    let documents = vec![
        "Vector databases are essential for similarity search in machine learning applications.",
        "Natural language processing uses embeddings to represent text as vectors.",
        "Retrieval Augmented Generation (RAG) combines search with generative AI.",
        "Lumosai provides tools for building modern AI applications.",
    ];
    
    // Choose a vector store based on available features
    #[cfg(feature = "qdrant")]
    {
        println!("Using Qdrant vector store");
        run_example_with_qdrant(embedding_provider.clone(), &documents).await?;
    }
    
    #[cfg(feature = "postgres")]
    {
        println!("\nUsing PostgreSQL vector store");
        run_example_with_postgres(embedding_provider.clone(), &documents).await?;
    }
    
    #[cfg(feature = "vectorize")]
    {
        // Only run if environment variables are set
        if let (Ok(_), Ok(_)) = (std::env::var("CF_API_TOKEN"), std::env::var("CF_ACCOUNT_ID")) {
            println!("\nUsing Cloudflare Vectorize store");
            run_example_with_vectorize(embedding_provider.clone(), &documents).await?;
        } else {
            println!("\nSkipping Vectorize example - API credentials not set");
        }
    }
    
    Ok(())
}

#[cfg(feature = "qdrant")]
async fn run_example_with_qdrant(
    embedder: Arc<dyn EmbeddingProvider>,
    documents: &[&str],
) -> Result<(), Box<dyn Error>> {
    // Create the Qdrant store
    let qdrant_url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());
    let store = QdrantStore::new(&qdrant_url, None).await?;
    
    // Create a retriever
    let mut retriever = VectorStoreRetriever::new(
        store,
        "example_rag_qdrant",
        4,  // 4-dimensional vectors for the example
        "cosine",
    );
    
    // Run the demo
    run_retrieval_demo(&mut retriever, embedder, documents).await?;
    
    Ok(())
}

#[cfg(feature = "postgres")]
async fn run_example_with_postgres(
    embedder: Arc<dyn EmbeddingProvider>,
    documents: &[&str],
) -> Result<(), Box<dyn Error>> {
    // Create the PostgreSQL store
    let pg_url = std::env::var("POSTGRES_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/vectors".to_string());
    
    let store = PostgresVectorStore::new(&pg_url).await?;
    
    // Create a retriever
    let mut retriever = VectorStoreRetriever::new(
        store,
        "example_rag_postgres",
        4,  // 4-dimensional vectors for the example
        "cosine",
    );
    
    // Run the demo
    run_retrieval_demo(&mut retriever, embedder, documents).await?;
    
    Ok(())
}

#[cfg(feature = "vectorize")]
async fn run_example_with_vectorize(
    embedder: Arc<dyn EmbeddingProvider>,
    documents: &[&str],
) -> Result<(), Box<dyn Error>> {
    // Create the Vectorize store
    let api_token = std::env::var("CF_API_TOKEN")?;
    let account_id = std::env::var("CF_ACCOUNT_ID")?;
    
    let store = VectorizeStore::new(&api_token, &account_id).await?;
    
    // Create a retriever
    let mut retriever = VectorStoreRetriever::new(
        store,
        "example_rag_vectorize",
        4,  // 4-dimensional vectors for the example
        "cosine",
    );
    
    // Run the demo
    run_retrieval_demo(&mut retriever, embedder, documents).await?;
    
    Ok(())
}

async fn run_retrieval_demo(
    retriever: &mut impl lumosai_rag::retriever::VectorStore,
    embedder: Arc<dyn EmbeddingProvider>,
    documents: &[&str],
) -> Result<(), Box<dyn Error>> {
    // Ensure the index exists
    println!("Creating/ensuring index...");
    
    // Clear any existing documents
    let _ = retriever.clear().await;
    
    // Add documents
    println!("Adding {} documents...", documents.len());
    for (i, &text) in documents.iter().enumerate() {
        let id = format!("doc_{}", i+1);
        let mut doc = Document {
            id: id.clone(),
            content: text.to_string(),
            metadata: Metadata::new()
                .with_source("example")
                .add("index", (i+1).to_string()),
            embedding: None,
        };
        
        // Generate embedding
        embedder.embed_document(&mut doc).await?;
        
        // Add to retriever
        retriever.add_document(doc).await?;
        println!("  Added document: {}", id);
    }
    
    // Query for similar documents
    println!("\nQuerying documents...");
    let queries = vec![
        "machine learning and vector search",
        "NLP and embeddings",
        "generative AI and RAG",
    ];
    
    for query in queries {
        println!("\nQuery: '{}'", query);
        
        let options = RetrievalOptions {
            limit: 2,
            threshold: None,
            filter: None,
        };
        
        let results = retriever.query_by_text(query, &options, &*embedder).await?;
        
        println!("  Found {} relevant documents:", results.documents.len());
        for (i, doc) in results.documents.iter().enumerate() {
            let score = results.scores.as_ref().unwrap()[i];
            println!("  - [{}] {} (score: {:.4})", doc.metadata.fields.get("index").unwrap(), doc.content, score);
        }
    }
    
    // Demonstration of filtering
    println!("\nQuerying with metadata filter...");
    let mut filter = std::collections::HashMap::new();
    filter.insert("index".to_string(), "2".into());
    
    let options = RetrievalOptions {
        limit: 5,
        threshold: None,
        filter: Some(filter),
    };
    
    let results = retriever.query_by_text("vector search", &options, &*embedder).await?;
    
    println!("  Found {} documents with index=2:", results.documents.len());
    for (i, doc) in results.documents.iter().enumerate() {
        let score = results.scores.as_ref().unwrap()[i];
        println!("  - [{}] {} (score: {:.4})", doc.metadata.fields.get("index").unwrap(), doc.content, score);
    }
    
    println!("\nDemonstration complete!");
    
    Ok(())
} 