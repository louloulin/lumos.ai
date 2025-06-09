//! Batch operations example for LanceDB
//!
//! This example demonstrates:
//! - Batch insertion of large numbers of documents
//! - Batch search operations
//! - Performance monitoring
//! - Memory-efficient processing

use std::time::Instant;
use lumosai_vector_lancedb::{LanceDbStorage, LanceDbConfigBuilder};
use lumosai_vector_core::{
    traits::VectorStorage,
    types::{Document, IndexConfig, SearchRequest, SimilarityMetric},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🚀 LanceDB Batch Operations Example");
    println!("=" * 50);
    
    // Configuration for batch operations
    let batch_size = 1000;
    let total_documents = 10000;
    let vector_dimension = 384;
    
    // 1. Create optimized LanceDB storage for batch operations
    println!("\n📦 Creating optimized LanceDB storage...");
    let config = LanceDbConfigBuilder::new("./example_data/lancedb_batch")
        .batch_size(batch_size)
        .enable_compression(true)
        .compression_level(6)
        .cache_size(1024 * 1024 * 50) // 50MB cache
        .build()?;
    
    let storage = LanceDbStorage::new(config).await?;
    println!("✅ Storage created with batch size: {}", batch_size);
    
    // 2. Create index with optimized settings
    println!("\n🔧 Creating optimized index...");
    let index_config = IndexConfig::new("batch_documents", vector_dimension)
        .with_metric(SimilarityMetric::Cosine)
        .with_description("Batch operations index");
    
    storage.create_index(index_config).await?;
    println!("✅ Index created for {} dimensions", vector_dimension);
    
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
        let document_ids = storage.upsert_documents("batch_documents", documents).await?;
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
    
    let total_time = start_time.elapsed();
    let docs_per_second = total_inserted as f64 / total_time.as_secs_f64();
    
    println!("✅ Batch insertion completed!");
    println!("   Total documents: {}", total_inserted);
    println!("   Total time: {:.2}s", total_time.as_secs_f64());
    println!("   Throughput: {:.0} docs/sec", docs_per_second);
    
    // 4. Perform batch search operations
    println!("\n🔍 Performing batch search operations...");
    let search_start_time = Instant::now();
    
    let num_queries = 100;
    let mut total_results = 0;
    
    for query_idx in 0..num_queries {
        let query_vector = generate_sample_embedding(vector_dimension, query_idx as u64);
        
        let search_request = SearchRequest {
            index_name: "batch_documents".to_string(),
            vector: query_vector,
            top_k: 10,
            similarity_metric: Some(SimilarityMetric::Cosine),
            filter: None,
            include_metadata: false, // Faster without metadata
        };
        
        let search_response = storage.search(search_request).await?;
        total_results += search_response.results.len();
        
        if query_idx % 20 == 0 {
            println!("  Completed {} queries...", query_idx + 1);
        }
    }
    
    let search_time = search_start_time.elapsed();
    let queries_per_second = num_queries as f64 / search_time.as_secs_f64();
    
    println!("✅ Batch search completed!");
    println!("   Total queries: {}", num_queries);
    println!("   Total results: {}", total_results);
    println!("   Search time: {:.2}s", search_time.as_secs_f64());
    println!("   Throughput: {:.0} queries/sec", queries_per_second);
    
    // 5. Memory usage and performance analysis
    println!("\n📊 Performance Analysis:");
    
    // Get index information
    let index_info = storage.describe_index("batch_documents").await?;
    println!("   Documents in index: {}", index_info.document_count);
    
    if let Some(storage_size) = index_info.storage_size {
        let size_mb = storage_size as f64 / (1024.0 * 1024.0);
        println!("   Storage size: {:.2} MB", size_mb);
        
        let bytes_per_doc = storage_size as f64 / index_info.document_count as f64;
        println!("   Bytes per document: {:.0}", bytes_per_doc);
    }
    
    // 6. Demonstrate filtered batch search
    println!("\n🔍 Performing filtered batch search...");
    let filtered_search_start = Instant::now();
    
    let filter = lumosai_vector_core::types::FilterCondition::Gt(
        "doc_id".to_string(),
        lumosai_vector_core::types::MetadataValue::Integer(5000),
    );
    
    let mut filtered_results = 0;
    let filtered_queries = 50;
    
    for query_idx in 0..filtered_queries {
        let query_vector = generate_sample_embedding(vector_dimension, query_idx as u64);
        
        let search_request = SearchRequest {
            index_name: "batch_documents".to_string(),
            vector: query_vector,
            top_k: 5,
            similarity_metric: Some(SimilarityMetric::Cosine),
            filter: Some(filter.clone()),
            include_metadata: true,
        };
        
        let search_response = storage.search(search_request).await?;
        filtered_results += search_response.results.len();
    }
    
    let filtered_search_time = filtered_search_start.elapsed();
    let filtered_qps = filtered_queries as f64 / filtered_search_time.as_secs_f64();
    
    println!("✅ Filtered search completed!");
    println!("   Filtered queries: {}", filtered_queries);
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
    
    storage.delete_documents("batch_documents", ids_to_delete.clone()).await?;
    let delete_time = delete_start.elapsed();
    
    println!("✅ Batch deletion completed!");
    println!("   Deleted {} documents", ids_to_delete.len());
    println!("   Time: {:.2}s", delete_time.as_secs_f64());
    
    // 8. Final statistics
    println!("\n📈 Final Statistics:");
    let final_index_info = storage.describe_index("batch_documents").await?;
    println!("   Remaining documents: {}", final_index_info.document_count);
    
    let backend_info = storage.backend_info();
    println!("   Backend: {} v{}", backend_info.name, backend_info.version);
    println!("   Features: {:?}", backend_info.features);
    
    println!("\n🎉 Batch operations example completed successfully!");
    println!("💡 Tips for production:");
    println!("   - Adjust batch size based on available memory");
    println!("   - Use compression for storage efficiency");
    println!("   - Monitor query performance and adjust indexes");
    println!("   - Consider partitioning for very large datasets");
    
    Ok(())
}

/// Generate a batch of documents for testing
fn generate_document_batch(start_id: usize, batch_size: usize, vector_dim: usize) -> Vec<Document> {
    let mut documents = Vec::with_capacity(batch_size);
    
    for i in 0..batch_size {
        let doc_id = start_id + i;
        let content = format!("This is document number {} with some sample content for testing batch operations.", doc_id);
        
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
