//! Example demonstrating PostgreSQL vector storage usage
//! 
//! This example shows how to:
//! - Connect to PostgreSQL with pgvector extension
//! - Create vector indexes
//! - Store and search documents with embeddings
//! - Use metadata filtering
//! 
//! Prerequisites:
//! 1. PostgreSQL server running
//! 2. pgvector extension installed: CREATE EXTENSION vector;
//! 3. Set DATABASE_URL environment variable
//! 
//! Run with: cargo run --example postgres_example --features postgres

use std::collections::HashMap;
use lumosai_vector::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::init();
    
    // Get database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            println!("DATABASE_URL not set, using default: postgresql://postgres:password@localhost/lumos_vector");
            "postgresql://postgres:password@localhost/lumos_vector".to_string()
        });
    
    println!("🚀 Connecting to PostgreSQL...");
    
    // Create PostgreSQL storage with custom configuration
    let config = lumosai_vector::postgres::PostgresConfig::new(&database_url)
        .with_performance(lumosai_vector::postgres::config::PerformanceConfig {
            batch_size: 1000,
            index_type: lumosai_vector::postgres::config::VectorIndexType::Hnsw,
            index_params: lumosai_vector::postgres::config::IndexParams::default(),
            use_prepared_statements: true,
        });
    
    let storage = lumosai_vector::postgres::PostgresVectorStorage::with_config(config).await?;
    
    // Test connection
    storage.health_check().await?;
    println!("✅ Connected to PostgreSQL successfully!");
    
    // Create a vector index
    let index_name = "example_documents";
    let index_config = IndexConfig {
        name: index_name.to_string(),
        dimension: 384, // Common dimension for sentence transformers
        metric: SimilarityMetric::Cosine,
        options: HashMap::new(),
    };
    
    println!("📊 Creating vector index: {}", index_name);
    storage.create_index(index_config).await?;
    
    // Create sample documents with embeddings
    let documents = vec![
        Document {
            id: "doc_1".to_string(),
            content: "Artificial intelligence is transforming the world of technology.".to_string(),
            embedding: Some(generate_sample_embedding(384, 1)),
            metadata: create_metadata(vec![
                ("category", MetadataValue::String("technology".to_string())),
                ("author", MetadataValue::String("Alice".to_string())),
                ("score", MetadataValue::Float(0.95)),
                ("published", MetadataValue::Boolean(true)),
            ]),
        },
        Document {
            id: "doc_2".to_string(),
            content: "Machine learning algorithms are becoming more sophisticated.".to_string(),
            embedding: Some(generate_sample_embedding(384, 2)),
            metadata: create_metadata(vec![
                ("category", MetadataValue::String("technology".to_string())),
                ("author", MetadataValue::String("Bob".to_string())),
                ("score", MetadataValue::Float(0.87)),
                ("published", MetadataValue::Boolean(true)),
            ]),
        },
        Document {
            id: "doc_3".to_string(),
            content: "Climate change is a pressing global issue requiring immediate action.".to_string(),
            embedding: Some(generate_sample_embedding(384, 3)),
            metadata: create_metadata(vec![
                ("category", MetadataValue::String("environment".to_string())),
                ("author", MetadataValue::String("Carol".to_string())),
                ("score", MetadataValue::Float(0.92)),
                ("published", MetadataValue::Boolean(true)),
            ]),
        },
        Document {
            id: "doc_4".to_string(),
            content: "Renewable energy sources are becoming more cost-effective.".to_string(),
            embedding: Some(generate_sample_embedding(384, 4)),
            metadata: create_metadata(vec![
                ("category", MetadataValue::String("environment".to_string())),
                ("author", MetadataValue::String("David".to_string())),
                ("score", MetadataValue::Float(0.89)),
                ("published", MetadataValue::Boolean(false)),
            ]),
        },
    ];
    
    println!("📝 Storing {} documents...", documents.len());
    let document_ids = storage.upsert_documents(index_name, documents).await?;
    println!("✅ Stored documents with IDs: {:?}", document_ids);
    
    // Perform vector similarity search
    println!("\n🔍 Performing vector similarity search...");
    let query_embedding = generate_sample_embedding(384, 1); // Similar to doc_1
    
    let search_request = SearchRequest {
        index_name: index_name.to_string(),
        query: SearchQuery::Vector(query_embedding),
        top_k: 3,
        filter: None,
        include_metadata: true,
        include_vectors: false,
        options: HashMap::new(),
    };
    
    let search_response = storage.search(search_request).await?;
    
    println!("📊 Search Results:");
    for (i, result) in search_response.results.iter().enumerate() {
        println!("  {}. Document ID: {}", i + 1, result.id);
        println!("     Score: {:.4}", result.score);
        if let Some(content) = &result.content {
            println!("     Content: {}", content);
        }
        if let Some(metadata) = &result.metadata {
            if let Some(MetadataValue::String(category)) = metadata.get("category") {
                println!("     Category: {}", category);
            }
            if let Some(MetadataValue::String(author)) = metadata.get("author") {
                println!("     Author: {}", author);
            }
        }
        println!();
    }
    
    // Retrieve specific documents
    println!("📖 Retrieving specific documents...");
    let retrieved_docs = storage.get_documents(
        index_name, 
        vec!["doc_1".to_string(), "doc_3".to_string()], 
        true // include vectors
    ).await?;
    
    println!("Retrieved {} documents:", retrieved_docs.len());
    for doc in &retrieved_docs {
        println!("  - {}: {} (embedding: {} dimensions)", 
            doc.id, 
            doc.content,
            doc.embedding.as_ref().map(|e| e.len()).unwrap_or(0)
        );
    }
    
    // Update a document
    println!("\n✏️  Updating document...");
    let updated_doc = Document {
        id: "doc_1".to_string(),
        content: "Artificial intelligence and machine learning are revolutionizing technology.".to_string(),
        embedding: Some(generate_sample_embedding(384, 5)),
        metadata: create_metadata(vec![
            ("category", MetadataValue::String("technology".to_string())),
            ("author", MetadataValue::String("Alice".to_string())),
            ("score", MetadataValue::Float(0.98)), // Updated score
            ("published", MetadataValue::Boolean(true)),
            ("updated", MetadataValue::Boolean(true)), // New field
        ]),
    };
    
    storage.update_document(index_name, updated_doc).await?;
    println!("✅ Document updated successfully!");
    
    // Get index information
    println!("\n📊 Index Information:");
    let index_info = storage.describe_index(index_name).await?;
    println!("  Name: {}", index_info.name);
    println!("  Dimension: {}", index_info.dimension);
    println!("  Vector Count: {}", index_info.vector_count);
    println!("  Metric: {:?}", index_info.metric);
    
    // List all indexes
    let indexes = storage.list_indexes().await?;
    println!("  All indexes: {:?}", indexes);
    
    // Backend information
    let backend_info = storage.backend_info();
    println!("\n🔧 Backend Information:");
    println!("  Name: {}", backend_info.name);
    println!("  Version: {}", backend_info.version);
    println!("  Features: {:?}", backend_info.features);
    
    // Clean up (optional)
    println!("\n🧹 Cleaning up...");
    storage.delete_documents(index_name, vec!["doc_2".to_string(), "doc_4".to_string()]).await?;
    println!("✅ Deleted some documents");
    
    // Uncomment to delete the entire index
    // storage.delete_index(index_name).await?;
    // println!("✅ Deleted index: {}", index_name);
    
    println!("\n🎉 PostgreSQL vector storage example completed successfully!");
    
    Ok(())
}

/// Generate a sample embedding vector for demonstration
fn generate_sample_embedding(dimension: usize, seed: u64) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let hash = hasher.finish();
    
    (0..dimension)
        .map(|i| {
            let mut hasher = DefaultHasher::new();
            (hash + i as u64).hash(&mut hasher);
            let val = hasher.finish() as f32 / u64::MAX as f32;
            (val - 0.5) * 2.0 // Normalize to [-1, 1]
        })
        .collect()
}

/// Helper function to create metadata
fn create_metadata(pairs: Vec<(&str, MetadataValue)>) -> HashMap<String, MetadataValue> {
    pairs.into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect()
}
