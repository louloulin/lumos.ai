//! Basic usage example for Lumos Vector Storage

use lumos_vector::prelude::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Lumos Vector Storage - Basic Usage Example");
    
    // Create a memory-based vector storage
    let storage = MemoryVectorStorage::new().await?;
    println!("✅ Created memory vector storage");
    
    // Create an index for 384-dimensional vectors (common for embeddings)
    let index_name = "embeddings";
    let dimension = 384;
    storage.create_index(index_name, dimension, Some(SimilarityMetric::Cosine)).await?;
    println!("✅ Created index '{}' with dimension {}", index_name, dimension);
    
    // Create some sample vectors
    let vectors = vec![
        vec![0.1; dimension],  // Vector 1
        vec![0.2; dimension],  // Vector 2
        vec![0.3; dimension],  // Vector 3
    ];
    
    // Create metadata for each vector
    let metadata = vec![
        HashMap::from([
            ("type".to_string(), serde_json::json!("document")),
            ("category".to_string(), serde_json::json!("tech")),
            ("score".to_string(), serde_json::json!(0.95)),
        ]),
        HashMap::from([
            ("type".to_string(), serde_json::json!("document")),
            ("category".to_string(), serde_json::json!("science")),
            ("score".to_string(), serde_json::json!(0.87)),
        ]),
        HashMap::from([
            ("type".to_string(), serde_json::json!("image")),
            ("category".to_string(), serde_json::json!("tech")),
            ("score".to_string(), serde_json::json!(0.92)),
        ]),
    ];
    
    // Insert vectors with metadata
    let ids = storage.upsert(index_name, vectors, None, Some(metadata)).await?;
    println!("✅ Inserted {} vectors with IDs: {:?}", ids.len(), ids);
    
    // Query for similar vectors
    let query_vector = vec![0.15; dimension];
    let results = storage.query(
        index_name,
        query_vector,
        5,  // top_k
        None,  // no filter
        false,  // don't include vectors in results
    ).await?;
    
    println!("\n🔍 Query Results:");
    for (i, result) in results.iter().enumerate() {
        println!("  {}. ID: {}, Score: {:.4}", i + 1, result.id, result.score);
        if let Some(metadata) = &result.metadata {
            println!("     Metadata: {:?}", metadata);
        }
    }
    
    // Query with filter
    let filter = FilterCondition::eq("type", "document");
    let filtered_results = storage.query(
        index_name,
        vec![0.25; dimension],
        5,
        Some(filter),
        false,
    ).await?;
    
    println!("\n🔍 Filtered Query Results (type=document):");
    for (i, result) in filtered_results.iter().enumerate() {
        println!("  {}. ID: {}, Score: {:.4}", i + 1, result.id, result.score);
        if let Some(metadata) = &result.metadata {
            println!("     Metadata: {:?}", metadata);
        }
    }
    
    // Complex filter example
    let complex_filter = FilterCondition::and(vec![
        FilterCondition::eq("category", "tech"),
        FilterCondition::gt("score", 0.9),
    ]);
    
    let complex_results = storage.query(
        index_name,
        vec![0.2; dimension],
        5,
        Some(complex_filter),
        false,
    ).await?;
    
    println!("\n🔍 Complex Filter Results (category=tech AND score>0.9):");
    for (i, result) in complex_results.iter().enumerate() {
        println!("  {}. ID: {}, Score: {:.4}", i + 1, result.id, result.score);
        if let Some(metadata) = &result.metadata {
            println!("     Metadata: {:?}", metadata);
        }
    }
    
    // Get index statistics
    let stats = storage.describe_index(index_name).await?;
    println!("\n📊 Index Statistics:");
    println!("  Name: {}", stats.name);
    println!("  Dimension: {}", stats.dimension);
    println!("  Metric: {:?}", stats.metric);
    println!("  Vector Count: {}", stats.vector_count);
    println!("  Size (bytes): {}", stats.size_bytes);
    
    // Update a vector's metadata
    if let Some(first_id) = ids.first() {
        let mut new_metadata = HashMap::new();
        new_metadata.insert("updated".to_string(), serde_json::json!(true));
        new_metadata.insert("timestamp".to_string(), serde_json::json!("2024-01-01"));
        
        storage.update_by_id(index_name, first_id, None, Some(new_metadata)).await?;
        println!("✅ Updated metadata for vector {}", first_id);
        
        // Retrieve the updated vector
        if let Some(updated_vector) = storage.get_by_id(index_name, first_id, false).await? {
            println!("📄 Updated vector metadata: {:?}", updated_vector.metadata);
        }
    }
    
    // List all indexes
    let indexes = storage.list_indexes().await?;
    println!("\n📋 Available indexes: {:?}", indexes);
    
    // Clean up - delete the index
    storage.delete_index(index_name).await?;
    println!("✅ Deleted index '{}'", index_name);
    
    println!("\n🎉 Example completed successfully!");
    
    Ok(())
}
