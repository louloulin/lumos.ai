//! Collection management example for Milvus
//!
//! This example demonstrates:
//! - Creating collections with different configurations
//! - Index management and optimization
//! - Collection statistics and monitoring
//! - Multi-tenancy and resource management

use lumosai_vector_milvus::{
    MilvusStorage, MilvusConfigBuilder,
    config::{IndexType, ConsistencyLevel},
};
use lumosai_vector_core::{
    traits::VectorStorage,
    types::{Document, IndexConfig, SimilarityMetric},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🚀 Milvus Collection Management Example");
    println!("=" * 50);
    
    // 1. Create Milvus storage with advanced configuration
    println!("\n📦 Creating Milvus storage with advanced configuration...");
    let config = MilvusConfigBuilder::new("http://localhost:19530")
        .database("collection_demo")
        .batch_size(500)
        .consistency_level(ConsistencyLevel::Strong)
        .shards_num(2)
        .replica_number(1)
        .build()?;
    
    let storage = match MilvusStorage::new(config).await {
        Ok(storage) => {
            println!("✅ Milvus storage created successfully");
            storage
        }
        Err(e) => {
            println!("❌ Failed to create Milvus storage: {}", e);
            println!("💡 Make sure Milvus is running on localhost:19530");
            return Ok(());
        }
    };
    
    // 2. Create multiple collections with different configurations
    println!("\n🔧 Creating collections with different configurations...");
    
    let collections = vec![
        ("documents_small", 128, SimilarityMetric::Cosine, "Small dimension documents"),
        ("documents_medium", 384, SimilarityMetric::Euclidean, "Medium dimension documents"),
        ("documents_large", 768, SimilarityMetric::DotProduct, "Large dimension documents"),
        ("images_embeddings", 512, SimilarityMetric::Cosine, "Image embeddings collection"),
    ];
    
    for (name, dimension, metric, description) in &collections {
        println!("\n📋 Creating collection: {}", name);
        
        let index_config = IndexConfig::new(name, *dimension)
            .with_metric(metric.clone())
            .with_description(description);
        
        match storage.create_index(index_config).await {
            Ok(_) => println!("✅ Collection '{}' created ({}D, {:?})", name, dimension, metric),
            Err(e) => {
                if e.to_string().contains("already exists") {
                    println!("ℹ️  Collection '{}' already exists", name);
                } else {
                    println!("❌ Failed to create collection '{}': {}", name, e);
                }
            }
        }
    }
    
    // 3. List all collections
    println!("\n📊 Listing all collections...");
    match storage.list_indexes().await {
        Ok(collection_names) => {
            println!("✅ Found {} collections:", collection_names.len());
            for (i, name) in collection_names.iter().enumerate() {
                println!("  {}. {}", i + 1, name);
            }
        }
        Err(e) => {
            println!("❌ Failed to list collections: {}", e);
        }
    }
    
    // 4. Get detailed information for each collection
    println!("\n📋 Getting detailed collection information...");
    for (name, expected_dim, metric, _) in &collections {
        match storage.describe_index(name).await {
            Ok(info) => {
                println!("\n📄 Collection: {}", name);
                println!("   - Dimension: {}", info.dimension);
                println!("   - Metric: {:?}", info.metric);
                println!("   - Document count: {}", info.document_count);
                if let Some(size) = info.storage_size {
                    println!("   - Storage size: {:.2} MB", size as f64 / (1024.0 * 1024.0));
                }
                
                // Verify configuration
                if info.dimension != *expected_dim {
                    println!("   ⚠️  Dimension mismatch: expected {}, got {}", expected_dim, info.dimension);
                }
            }
            Err(e) => {
                println!("❌ Failed to get info for collection '{}': {}", name, e);
            }
        }
    }
    
    // 5. Insert sample data into collections
    println!("\n💾 Inserting sample data into collections...");
    
    for (name, dimension, _, _) in &collections {
        println!("\n📝 Inserting data into '{}'...", name);
        
        let documents = generate_sample_documents(*dimension, 10);
        
        match storage.upsert_documents(name, documents).await {
            Ok(ids) => {
                println!("✅ Inserted {} documents into '{}'", ids.len(), name);
            }
            Err(e) => {
                println!("❌ Failed to insert into '{}': {}", name, e);
            }
        }
    }
    
    // Wait for indexing
    println!("\n⏳ Waiting for indexing to complete...");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    
    // 6. Test search performance across different collections
    println!("\n🔍 Testing search performance across collections...");
    
    for (name, dimension, metric, _) in &collections {
        println!("\n🎯 Testing search in '{}'...", name);
        
        let query_vector = generate_sample_embedding(*dimension, 42);
        let search_request = lumosai_vector_core::types::SearchRequest {
            index_name: name.to_string(),
            vector: query_vector,
            top_k: 5,
            similarity_metric: Some(metric.clone()),
            filter: None,
            include_metadata: true,
        };
        
        let start_time = std::time::Instant::now();
        match storage.search(search_request).await {
            Ok(response) => {
                let search_time = start_time.elapsed();
                println!("✅ Search completed in {:.2}ms", search_time.as_millis());
                println!("   Found {} results", response.results.len());
                
                if !response.results.is_empty() {
                    println!("   Top result: ID={}, Score={:.4}", 
                        response.results[0].id, response.results[0].score);
                }
            }
            Err(e) => {
                println!("❌ Search failed: {}", e);
            }
        }
    }
    
    // 7. Demonstrate collection statistics monitoring
    println!("\n📊 Collection Statistics Monitoring...");
    
    for (name, _, _, _) in &collections {
        match storage.describe_index(name).await {
            Ok(info) => {
                println!("\n📈 Statistics for '{}':", name);
                println!("   - Documents: {}", info.document_count);
                
                if let Some(size) = info.storage_size {
                    let size_mb = size as f64 / (1024.0 * 1024.0);
                    println!("   - Storage: {:.2} MB", size_mb);
                    
                    if info.document_count > 0 {
                        let avg_size = size as f64 / info.document_count as f64;
                        println!("   - Avg doc size: {:.0} bytes", avg_size);
                    }
                }
                
                // Calculate efficiency metrics
                let expected_vector_size = info.dimension * 4; // 4 bytes per float32
                let efficiency = if info.storage_size.is_some() && info.document_count > 0 {
                    let actual_size = info.storage_size.unwrap() as f64 / info.document_count as f64;
                    expected_vector_size as f64 / actual_size * 100.0
                } else {
                    0.0
                };
                
                if efficiency > 0.0 {
                    println!("   - Storage efficiency: {:.1}%", efficiency);
                }
            }
            Err(e) => {
                println!("❌ Failed to get statistics for '{}': {}", name, e);
            }
        }
    }
    
    // 8. Demonstrate multi-collection search
    println!("\n🔍 Multi-Collection Search Demonstration...");
    
    let query_384 = generate_sample_embedding(384, 123);
    let collections_384d = ["documents_medium"];
    
    println!("\n🎯 Searching across 384D collections...");
    for collection_name in &collections_384d {
        let search_request = lumosai_vector_core::types::SearchRequest {
            index_name: collection_name.to_string(),
            vector: query_384.clone(),
            top_k: 3,
            similarity_metric: Some(SimilarityMetric::Cosine),
            filter: None,
            include_metadata: false,
        };
        
        match storage.search(search_request).await {
            Ok(response) => {
                println!("✅ Collection '{}': {} results", collection_name, response.results.len());
                for (i, result) in response.results.iter().enumerate() {
                    println!("   {}. ID: {}, Score: {:.4}", i + 1, result.id, result.score);
                }
            }
            Err(e) => {
                println!("❌ Search in '{}' failed: {}", collection_name, e);
            }
        }
    }
    
    // 9. Collection maintenance operations
    println!("\n🔧 Collection Maintenance Operations...");
    
    // Health check
    println!("\n🏥 Performing health check...");
    match storage.health_check().await {
        Ok(_) => println!("✅ All collections are healthy"),
        Err(e) => println!("❌ Health check failed: {}", e),
    }
    
    // Backend information
    println!("\n📋 Backend Information:");
    let backend_info = storage.backend_info();
    println!("   - Name: {}", backend_info.name);
    println!("   - Version: {}", backend_info.version);
    println!("   - Features: {:?}", backend_info.features);
    
    // 10. Cleanup demonstration (optional)
    println!("\n🧹 Cleanup Options:");
    println!("   To clean up test collections, you can run:");
    for (name, _, _, _) in &collections {
        println!("   - Delete '{}': storage.delete_index(\"{}\").await", name, name);
    }
    
    println!("\n🎉 Collection management example completed successfully!");
    println!("💡 Key takeaways:");
    println!("   - Different collections can have different dimensions and metrics");
    println!("   - Monitor storage efficiency and performance across collections");
    println!("   - Use appropriate consistency levels for your use case");
    println!("   - Consider sharding and replication for production workloads");
    
    Ok(())
}

/// Generate sample documents for testing
fn generate_sample_documents(dimension: usize, count: usize) -> Vec<Document> {
    let mut documents = Vec::with_capacity(count);
    
    for i in 0..count {
        let content = format!("Sample document {} with dimension {}", i, dimension);
        let embedding = generate_sample_embedding(dimension, i as u64);
        
        let document = Document::new(&format!("doc_{}_{}", dimension, i), &content)
            .with_embedding(embedding)
            .with_metadata("dimension", dimension as i64)
            .with_metadata("index", i as i64)
            .with_metadata("type", "sample");
        
        documents.push(document);
    }
    
    documents
}

/// Generate a sample embedding vector
fn generate_sample_embedding(dimension: usize, seed: u64) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let hash = hasher.finish();
    
    let mut embedding = Vec::with_capacity(dimension);
    let mut current_hash = hash;
    
    for i in 0..dimension {
        let value = ((current_hash.wrapping_add(i as u64) % 1000) as f32 / 500.0) - 1.0;
        embedding.push(value);
        current_hash = current_hash.wrapping_mul(1103515245).wrapping_add(12345);
    }
    
    // Normalize the vector
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for x in &mut embedding {
            *x /= magnitude;
        }
    }
    
    embedding
}
