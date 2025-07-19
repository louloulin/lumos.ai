//! Basic LanceDB usage example
//!
//! This example demonstrates how to:
//! - Create a LanceDB storage instance
//! - Create an index
//! - Insert documents with embeddings
//! - Perform vector search
//! - Clean up resources

use std::collections::HashMap;
use lumosai_vector_lancedb::{LanceDbStorage, LanceDbConfig};
use lumosai_vector_core::{
    traits::VectorStorage,
    types::{Document, IndexConfig, SearchRequest, SimilarityMetric},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("🚀 LanceDB Basic Usage Example");
    println!("=" * 50);
    
    // 1. Create LanceDB storage with local file storage
    println!("\n📦 Creating LanceDB storage...");
    let config = LanceDbConfig::local("./example_data/lancedb");
    let storage = LanceDbStorage::new(config).await?;
    
    println!("✅ LanceDB storage created successfully");
    
    // 2. Create an index for documents
    println!("\n🔧 Creating vector index...");
    let index_config = IndexConfig::new("documents", 384)
        .with_metric(SimilarityMetric::Cosine)
        .with_description("Example document index");
    
    storage.create_index(index_config).await?;
    println!("✅ Index 'documents' created successfully");
    
    // 3. Prepare sample documents with embeddings
    println!("\n📄 Preparing sample documents...");
    let documents = vec![
        Document::new("doc1", "The quick brown fox jumps over the lazy dog")
            .with_embedding(generate_sample_embedding(384, 1))
            .with_metadata("category", "animals")
            .with_metadata("length", 43i64),
        
        Document::new("doc2", "Machine learning is a subset of artificial intelligence")
            .with_embedding(generate_sample_embedding(384, 2))
            .with_metadata("category", "technology")
            .with_metadata("length", 56i64),
        
        Document::new("doc3", "The weather today is sunny and warm")
            .with_embedding(generate_sample_embedding(384, 3))
            .with_metadata("category", "weather")
            .with_metadata("length", 35i64),
        
        Document::new("doc4", "Rust is a systems programming language")
            .with_embedding(generate_sample_embedding(384, 4))
            .with_metadata("category", "technology")
            .with_metadata("length", 38i64),
        
        Document::new("doc5", "The cat sat on the mat")
            .with_embedding(generate_sample_embedding(384, 5))
            .with_metadata("category", "animals")
            .with_metadata("length", 22i64),
    ];
    
    println!("✅ Prepared {} documents", documents.len());
    
    // 4. Insert documents into the index
    println!("\n💾 Inserting documents...");
    let document_ids = storage.upsert_documents("documents", documents).await?;
    println!("✅ Inserted {} documents with IDs: {:?}", document_ids.len(), document_ids);
    
    // 5. Perform vector search
    println!("\n🔍 Performing vector search...");
    let query_vector = generate_sample_embedding(384, 2); // Similar to doc2
    
    let search_request = SearchRequest {
        index_name: "documents".to_string(),
        vector: query_vector,
        top_k: 3,
        similarity_metric: Some(SimilarityMetric::Cosine),
        filter: None,
        include_metadata: true,
    };
    
    let search_response = storage.search(search_request).await?;
    
    println!("✅ Search completed! Found {} results:", search_response.results.len());
    for (i, result) in search_response.results.iter().enumerate() {
        println!("  {}. ID: {}, Score: {:.4}", i + 1, result.id, result.score);
        if let Some(metadata) = &result.metadata {
            if let Some(category) = metadata.get("category") {
                println!("     Category: {:?}", category);
            }
        }
    }
    
    // 6. Search with metadata filter
    println!("\n🔍 Performing filtered search (technology category)...");
    let filter = lumosai_vector_core::types::FilterCondition::Eq(
        "category".to_string(),
        lumosai_vector_core::types::MetadataValue::String("technology".to_string()),
    );
    
    let filtered_search_request = SearchRequest {
        index_name: "documents".to_string(),
        vector: query_vector,
        top_k: 5,
        similarity_metric: Some(SimilarityMetric::Cosine),
        filter: Some(filter),
        include_metadata: true,
    };
    
    let filtered_response = storage.search(filtered_search_request).await?;
    
    println!("✅ Filtered search completed! Found {} technology documents:", filtered_response.results.len());
    for (i, result) in filtered_response.results.iter().enumerate() {
        println!("  {}. ID: {}, Score: {:.4}", i + 1, result.id, result.score);
    }
    
    // 7. Get specific documents
    println!("\n📖 Retrieving specific documents...");
    let retrieved_docs = storage.get_documents(
        "documents",
        vec!["doc1".to_string(), "doc3".to_string()],
        false, // Don't include vectors
    ).await?;
    
    println!("✅ Retrieved {} documents:", retrieved_docs.len());
    for doc in &retrieved_docs {
        println!("  - {}: {}", doc.id, doc.content.as_deref().unwrap_or("No content"));
    }
    
    // 8. Update a document
    println!("\n✏️  Updating document...");
    let updated_doc = Document::new("doc1", "The quick brown fox jumps over the lazy dog (updated)")
        .with_embedding(generate_sample_embedding(384, 1))
        .with_metadata("category", "animals")
        .with_metadata("length", 52i64)
        .with_metadata("updated", true);
    
    storage.update_document("documents", updated_doc).await?;
    println!("✅ Document 'doc1' updated successfully");
    
    // 9. Delete a document
    println!("\n🗑️  Deleting document...");
    storage.delete_documents("documents", vec!["doc5".to_string()]).await?;
    println!("✅ Document 'doc5' deleted successfully");
    
    // 10. List indexes and get index info
    println!("\n📊 Getting index information...");
    let indexes = storage.list_indexes().await?;
    println!("✅ Available indexes: {:?}", indexes);
    
    let index_info = storage.describe_index("documents").await?;
    println!("✅ Index info:");
    println!("   - Name: {}", index_info.name);
    println!("   - Dimension: {}", index_info.dimension);
    println!("   - Metric: {:?}", index_info.metric);
    println!("   - Document count: {}", index_info.document_count);
    
    // 11. Health check
    println!("\n🏥 Performing health check...");
    storage.health_check().await?;
    println!("✅ Storage is healthy");
    
    // 12. Backend info
    println!("\n📋 Backend information:");
    let backend_info = storage.backend_info();
    println!("   - Name: {}", backend_info.name);
    println!("   - Version: {}", backend_info.version);
    println!("   - Features: {:?}", backend_info.features);
    
    println!("\n🎉 Example completed successfully!");
    println!("💡 Tip: Check the './example_data/lancedb' directory for the created database files");
    
    Ok(())
}

/// Generate a sample embedding vector for demonstration
/// In a real application, you would use an actual embedding model
fn generate_sample_embedding(dimension: usize, seed: u64) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let hash = hasher.finish();
    
    let mut embedding = Vec::with_capacity(dimension);
    let mut current_hash = hash;
    
    for i in 0..dimension {
        // Generate pseudo-random float between -1.0 and 1.0
        let value = ((current_hash.wrapping_add(i as u64) % 1000) as f32 / 500.0) - 1.0;
        embedding.push(value);
        
        // Update hash for next iteration
        current_hash = current_hash.wrapping_mul(1103515245).wrapping_add(12345);
    }
    
    // Normalize the vector for cosine similarity
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for x in &mut embedding {
            *x /= magnitude;
        }
    }
    
    embedding
}
