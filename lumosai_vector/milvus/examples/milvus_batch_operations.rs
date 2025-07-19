//! Batch operations example for Milvus
//!
//! This example demonstrates:
//! - Batch insertion of large numbers of documents
//! - Batch search operations
//! - Performance monitoring
//! - Distributed processing capabilities

use std::time::Instant;
use lumosai_vector_milvus::{MilvusStorage, MilvusConfigBuilder};
use lumosai_vector_core::{
    traits::VectorStorage,
    types::{Document, IndexConfig, SearchRequest, SearchQuery, SimilarityMetric},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🚀 Milvus Batch Operations Example");
    println!("{}", "=".repeat(50));
    
    // Configuration for batch operations
    let batch_size = 1000;
    let total_documents = 10000;
    let vector_dimension = 384;
    
    // 1. Create optimized Milvus storage for batch operations
    println!("\n📦 Creating optimized Milvus storage...");
    let config = MilvusConfigBuilder::new("http://localhost:19530")
        .database("batch_test")
        .batch_size(batch_size)
        .consistency_level(lumosai_vector_milvus::config::ConsistencyLevel::Eventually)
        .build()?;
    
    let storage = match MilvusStorage::new(config).await {
        Ok(storage) => {
            println!("✅ Storage created with batch size: {}", batch_size);
            storage
        }
        Err(e) => {
            println!("❌ Failed to create Milvus storage: {}", e);
            println!("💡 Make sure Milvus is running on localhost:19530");
            return Ok(());
        }
    };
    
    // 2. Create collection with optimized settings
    println!("\n🔧 Creating optimized collection...");
    let index_config = IndexConfig::new("batch_documents", vector_dimension)
        .with_metric(SimilarityMetric::Cosine)
        .with_option("description", "Batch operations collection");
    
    match storage.create_index(index_config).await {
        Ok(_) => println!("✅ Collection created for {} dimensions", vector_dimension),
        Err(e) => {
            if e.to_string().contains("already exists") {
                println!("ℹ️  Collection already exists, continuing...");
            } else {
                println!("❌ Failed to create collection: {}", e);
                return Ok(());
            }
        }
    }
    
    // 3. Generate and insert documents in batches
    println!("\n💾 Inserting {} documents in batches of {}...", total_documents, batch_size);
    let start_time = Instant::now();
    
    let mut total_inserted = 0;
    let num_batches = (total_documents + batch_size - 1) / batch_size;
    
    for batch_idx in 0..num_batches {
        let batch_start = batch_idx * batch_size;
        let batch_end = std::cmp::min(batch_start + batch_size, total_documents);
        let current_batch_size = batch_end - batch_start;
        
        // Generate batch of documents
        let batch_start_time = Instant::now();
        let documents = generate_document_batch(batch_start, current_batch_size, vector_dimension);
        let generation_time = batch_start_time.elapsed();
        
        // Insert batch
        let insert_start_time = Instant::now();
        match storage.upsert_documents("batch_documents", documents).await {
            Ok(document_ids) => {
                let insert_time = insert_start_time.elapsed();
                total_inserted += document_ids.len();
                
                println!(
                    "  Batch {}/{}: {} docs | Gen: {:.2}ms | Insert: {:.2}ms | Total: {}",
                    batch_idx + 1,
                    num_batches,
                    current_batch_size,
                    generation_time.as_millis(),
                    insert_time.as_millis(),
                    total_inserted
                );
            }
            Err(e) => {
                println!("❌ Failed to insert batch {}: {}", batch_idx + 1, e);
                continue;
            }
        }
        
        // Small delay to avoid overwhelming Milvus
        if batch_idx % 5 == 0 && batch_idx > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    
    let total_time = start_time.elapsed();
    let docs_per_second = total_inserted as f64 / total_time.as_secs_f64();
    
    println!("✅ Batch insertion completed!");
    println!("   Total documents: {}", total_inserted);
    println!("   Total time: {:.2}s", total_time.as_secs_f64());
    println!("   Throughput: {:.0} docs/sec", docs_per_second);
    
    // Wait for indexing to complete
    println!("\n⏳ Waiting for indexing to complete...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    
    // 4. Perform batch search operations
    println!("\n🔍 Performing batch search operations...");
    let search_start_time = Instant::now();
    
    let num_queries = 100;
    let mut total_results = 0;
    let mut successful_queries = 0;
    
    for query_idx in 0..num_queries {
        let query_vector = generate_sample_embedding(vector_dimension, query_idx as u64);
        
        let search_request = SearchRequest {
            index_name: "batch_documents".to_string(),
            query: SearchQuery::Vector(query_vector),
            top_k: 10,
            filter: None,
            include_metadata: false, // Faster without metadata
            include_vectors: false,
            options: std::collections::HashMap::new(),
        };
        
        match storage.search(search_request).await {
            Ok(search_response) => {
                total_results += search_response.results.len();
                successful_queries += 1;
            }
            Err(e) => {
                if query_idx < 5 {
                    println!("⚠️  Query {} failed: {}", query_idx + 1, e);
                }
            }
        }
        
        if query_idx % 20 == 0 && query_idx > 0 {
            println!("  Completed {} queries...", query_idx + 1);
        }
    }
    
    let search_time = search_start_time.elapsed();
    let queries_per_second = successful_queries as f64 / search_time.as_secs_f64();
    
    println!("✅ Batch search completed!");
    println!("   Total queries: {}", num_queries);
    println!("   Successful queries: {}", successful_queries);
    println!("   Total results: {}", total_results);
    println!("   Search time: {:.2}s", search_time.as_secs_f64());
    println!("   Throughput: {:.0} queries/sec", queries_per_second);
    
    // 5. Performance analysis
    println!("\n📊 Performance Analysis:");
    
    // Get collection information
    match storage.describe_index("batch_documents").await {
        Ok(index_info) => {
            println!("   Documents in collection: {}", index_info.vector_count);

            let size_mb = index_info.size_bytes as f64 / (1024.0 * 1024.0);
            println!("   Storage size: {:.2} MB", size_mb);

            if index_info.vector_count > 0 {
                let bytes_per_doc = index_info.size_bytes as f64 / index_info.vector_count as f64;
                println!("   Bytes per document: {:.0}", bytes_per_doc);
            }
        }
        Err(e) => {
            println!("⚠️  Could not get collection info: {}", e);
        }
    }
    
    // 6. Demonstrate filtered batch search
    println!("\n🔍 Performing filtered batch search...");
    let filtered_search_start = Instant::now();
    
    let filter = lumosai_vector_core::types::FilterCondition::Gt(
        "doc_id".to_string(),
        lumosai_vector_core::types::MetadataValue::Integer(5000),
    );
    
    let mut filtered_results = 0;
    let mut successful_filtered_queries = 0;
    let filtered_queries = 50;
    
    for query_idx in 0..filtered_queries {
        let query_vector = generate_sample_embedding(vector_dimension, query_idx as u64);
        
        let search_request = SearchRequest {
            index_name: "batch_documents".to_string(),
            query: SearchQuery::Vector(query_vector),
            top_k: 5,
            filter: Some(filter.clone()),
            include_metadata: true,
            include_vectors: false,
            options: std::collections::HashMap::new(),
        };
        
        match storage.search(search_request).await {
            Ok(search_response) => {
                filtered_results += search_response.results.len();
                successful_filtered_queries += 1;
            }
            Err(e) => {
                if query_idx < 3 {
                    println!("⚠️  Filtered query {} failed: {}", query_idx + 1, e);
                }
            }
        }
    }
    
    let filtered_search_time = filtered_search_start.elapsed();
    let filtered_qps = successful_filtered_queries as f64 / filtered_search_time.as_secs_f64();
    
    println!("✅ Filtered search completed!");
    println!("   Filtered queries: {}", filtered_queries);
    println!("   Successful queries: {}", successful_filtered_queries);
    println!("   Results found: {}", filtered_results);
    println!("   Time: {:.2}s", filtered_search_time.as_secs_f64());
    println!("   Throughput: {:.0} queries/sec", filtered_qps);
    
    // 7. Batch deletion example
    println!("\n🗑️  Performing batch deletion...");
    let delete_start = Instant::now();
    
    // Delete documents with IDs ending in 0 (every 10th document)
    let mut ids_to_delete = Vec::new();
    for i in (0..total_documents).step_by(10) {
        ids_to_delete.push(format!("doc_{}", i));
    }
    
    match storage.delete_documents("batch_documents", ids_to_delete.clone()).await {
        Ok(_) => {
            let delete_time = delete_start.elapsed();
            println!("✅ Batch deletion completed!");
            println!("   Deleted {} documents", ids_to_delete.len());
            println!("   Time: {:.2}s", delete_time.as_secs_f64());
        }
        Err(e) => {
            println!("❌ Batch deletion failed: {}", e);
        }
    }
    
    // 8. Final statistics
    println!("\n📈 Final Statistics:");
    match storage.describe_index("batch_documents").await {
        Ok(final_index_info) => {
            println!("   Remaining documents: {}", final_index_info.vector_count);
        }
        Err(e) => {
            println!("⚠️  Could not get final collection info: {}", e);
        }
    }
    
    let backend_info = storage.backend_info();
    println!("   Backend: {} v{}", backend_info.name, backend_info.version);
    println!("   Features: {:?}", backend_info.features);
    
    println!("\n🎉 Batch operations example completed successfully!");
    println!("💡 Tips for production:");
    println!("   - Use appropriate batch sizes based on your Milvus cluster capacity");
    println!("   - Consider using eventually consistent mode for better performance");
    println!("   - Monitor Milvus metrics and adjust parallelism accordingly");
    println!("   - Use collection partitioning for very large datasets");
    
    Ok(())
}

/// Generate a batch of documents for testing
fn generate_document_batch(start_id: usize, batch_size: usize, vector_dim: usize) -> Vec<Document> {
    let mut documents = Vec::with_capacity(batch_size);
    
    for i in 0..batch_size {
        let doc_id = start_id + i;
        let content = format!("This is document number {} with some sample content for testing batch operations in Milvus.", doc_id);
        
        let mut document = Document::new(&format!("doc_{}", doc_id), &content)
            .with_embedding(generate_sample_embedding(vector_dim, doc_id as u64))
            .with_metadata("doc_id", doc_id as i64)
            .with_metadata("batch_id", (start_id / 1000) as i64)
            .with_metadata("content_length", content.len() as i64);
        
        // Add some categorical metadata for filtering
        let category = match doc_id % 5 {
            0 => "technology",
            1 => "science",
            2 => "business",
            3 => "health",
            _ => "general",
        };
        document = document.with_metadata("category", category);
        
        // Add priority metadata
        let priority = if doc_id % 10 == 0 { "high" } else { "normal" };
        document = document.with_metadata("priority", priority);
        
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
