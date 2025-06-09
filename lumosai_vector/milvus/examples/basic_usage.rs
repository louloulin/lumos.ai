//! Basic Milvus usage example
//!
//! This example demonstrates how to:
//! - Create a Milvus storage instance
//! - Create a collection
//! - Insert documents with embeddings
//! - Perform vector search
//! - Clean up resources

use std::collections::HashMap;
use lumosai_vector_milvus::{MilvusStorage, MilvusConfig};
use lumosai_vector_core::{
    traits::VectorStorage,
    types::{Document, IndexConfig, SearchRequest, SearchQuery, SimilarityMetric},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("🚀 Milvus Basic Usage Example");
    println!("{}", "=".repeat(50));
    
    // 1. Create Milvus storage
    println!("\n📦 Creating Milvus storage...");
    let config = MilvusConfig::new("http://localhost:19530")
        .with_database("default")
        .with_timeout(std::time::Duration::from_secs(30));
    
    let storage = match MilvusStorage::new(config).await {
        Ok(storage) => {
            println!("✅ Milvus storage created successfully");
            storage
        }
        Err(e) => {
            println!("❌ Failed to create Milvus storage: {}", e);
            println!("💡 Make sure Milvus is running on localhost:19530");
            println!("   You can start Milvus using Docker:");
            println!("   docker run -p 19530:19530 -p 9091:9091 milvusdb/milvus:latest");
            return Ok(());
        }
    };
    
    // 2. Create a collection for documents
    println!("\n🔧 Creating vector collection...");
    let index_config = IndexConfig::new("documents", 384)
        .with_metric(SimilarityMetric::Cosine)
        .with_option("description", "Example document collection");
    
    match storage.create_index(index_config).await {
        Ok(_) => println!("✅ Collection 'documents' created successfully"),
        Err(e) => {
            if e.to_string().contains("already exists") {
                println!("ℹ️  Collection 'documents' already exists, continuing...");
            } else {
                println!("❌ Failed to create collection: {}", e);
                return Ok(());
            }
        }
    }
    
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
    
    // 4. Insert documents into the collection
    println!("\n💾 Inserting documents...");
    match storage.upsert_documents("documents", documents).await {
        Ok(document_ids) => {
            println!("✅ Inserted {} documents with IDs: {:?}", document_ids.len(), document_ids);
        }
        Err(e) => {
            println!("❌ Failed to insert documents: {}", e);
            return Ok(());
        }
    }
    
    // Wait a moment for indexing
    println!("\n⏳ Waiting for indexing to complete...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    
    // 5. Perform vector search
    println!("\n🔍 Performing vector search...");
    let query_vector = generate_sample_embedding(384, 2); // Similar to doc2
    
    let search_request = SearchRequest {
        index_name: "documents".to_string(),
        query: SearchQuery::Vector(query_vector.clone()),
        top_k: 3,
        filter: None,
        include_metadata: true,
        include_vectors: false,
        options: HashMap::new(),
    };
    
    match storage.search(search_request).await {
        Ok(search_response) => {
            println!("✅ Search completed! Found {} results:", search_response.results.len());
            for (i, result) in search_response.results.iter().enumerate() {
                println!("  {}. ID: {}, Score: {:.4}", i + 1, result.id, result.score);
                if let Some(metadata) = &result.metadata {
                    if let Some(category) = metadata.get("category") {
                        println!("     Category: {:?}", category);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Search failed: {}", e);
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
        query: SearchQuery::Vector(query_vector),
        top_k: 5,
        filter: Some(filter),
        include_metadata: true,
        include_vectors: false,
        options: HashMap::new(),
    };
    
    match storage.search(filtered_search_request).await {
        Ok(filtered_response) => {
            println!("✅ Filtered search completed! Found {} technology documents:", filtered_response.results.len());
            for (i, result) in filtered_response.results.iter().enumerate() {
                println!("  {}. ID: {}, Score: {:.4}", i + 1, result.id, result.score);
            }
        }
        Err(e) => {
            println!("⚠️  Filtered search failed: {}", e);
            println!("   Note: Metadata filtering may require specific Milvus configuration");
        }
    }
    
    // 7. Get specific documents
    println!("\n📖 Retrieving specific documents...");
    match storage.get_documents(
        "documents",
        vec!["doc1".to_string(), "doc3".to_string()],
        false, // Don't include vectors
    ).await {
        Ok(retrieved_docs) => {
            println!("✅ Retrieved {} documents:", retrieved_docs.len());
            for doc in &retrieved_docs {
                println!("  - {}: {}", doc.id, &doc.content);
            }
        }
        Err(e) => {
            println!("❌ Failed to retrieve documents: {}", e);
        }
    }
    
    // 8. Update a document
    println!("\n✏️  Updating document...");
    let updated_doc = Document::new("doc1", "The quick brown fox jumps over the lazy dog (updated)")
        .with_embedding(generate_sample_embedding(384, 1))
        .with_metadata("category", "animals")
        .with_metadata("length", 52i64)
        .with_metadata("updated", true);
    
    match storage.update_document("documents", updated_doc).await {
        Ok(_) => println!("✅ Document 'doc1' updated successfully"),
        Err(e) => println!("❌ Failed to update document: {}", e),
    }
    
    // 9. Delete a document
    println!("\n🗑️  Deleting document...");
    match storage.delete_documents("documents", vec!["doc5".to_string()]).await {
        Ok(_) => println!("✅ Document 'doc5' deleted successfully"),
        Err(e) => println!("❌ Failed to delete document: {}", e),
    }
    
    // 10. List collections and get collection info
    println!("\n📊 Getting collection information...");
    match storage.list_indexes().await {
        Ok(indexes) => {
            println!("✅ Available collections: {:?}", indexes);
            
            if indexes.contains(&"documents".to_string()) {
                match storage.describe_index("documents").await {
                    Ok(index_info) => {
                        println!("✅ Collection info:");
                        println!("   - Name: {}", index_info.name);
                        println!("   - Dimension: {}", index_info.dimension);
                        println!("   - Metric: {:?}", index_info.metric);
                        println!("   - Vector count: {}", index_info.vector_count);
                        println!("   - Storage size: {} bytes", index_info.size_bytes);
                    }
                    Err(e) => println!("❌ Failed to get collection info: {}", e),
                }
            }
        }
        Err(e) => println!("❌ Failed to list collections: {}", e),
    }
    
    // 11. Health check
    println!("\n🏥 Performing health check...");
    match storage.health_check().await {
        Ok(_) => println!("✅ Milvus is healthy"),
        Err(e) => println!("❌ Health check failed: {}", e),
    }
    
    // 12. Backend info
    println!("\n📋 Backend information:");
    let backend_info = storage.backend_info();
    println!("   - Name: {}", backend_info.name);
    println!("   - Version: {}", backend_info.version);
    println!("   - Features: {:?}", backend_info.features);
    
    println!("\n🎉 Example completed successfully!");
    println!("💡 Tip: You can view your data in Milvus using Attu (Milvus admin tool)");
    println!("   Docker: docker run -p 3000:3000 zilliz/attu:latest");
    
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
