//! Batch embedding processing example
//! 
//! This example demonstrates efficient batch processing of large numbers
//! of texts using FastEmbed with different optimization strategies.

use lumosai_vector_fastembed::{
    FastEmbedProvider, FastEmbedModel, FastEmbedConfigBuilder
};
use lumosai_vector_core::traits::EmbeddingModel;
use std::time::Instant;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🚀 LumosAI FastEmbed Batch Processing Example");
    println!("==============================================");
    
    // Generate test data
    let test_data = generate_test_data(1000);
    println!("📊 Generated {} test sentences", test_data.len());
    
    // Example 1: Default batch processing
    println!("\n📝 Example 1: Default batch processing");
    await_batch_processing_example(&test_data, "Default", 256).await?;
    
    // Example 2: Small batch processing
    println!("\n📝 Example 2: Small batch processing");
    await_batch_processing_example(&test_data, "Small", 64).await?;
    
    // Example 3: Large batch processing
    println!("\n📝 Example 3: Large batch processing");
    await_batch_processing_example(&test_data, "Large", 512).await?;
    
    // Example 4: Memory-efficient streaming
    println!("\n📝 Example 4: Memory-efficient streaming");
    await_streaming_example(&test_data).await?;
    
    // Example 5: Parallel processing comparison
    println!("\n📝 Example 5: Parallel vs Sequential processing");
    await_parallel_comparison(&test_data).await?;
    
    // Example 6: Different models performance
    println!("\n📝 Example 6: Model performance comparison");
    await_model_comparison(&test_data[..100]).await?;
    
    println!("\n🎉 All batch processing examples completed!");
    
    Ok(())
}

async fn await_batch_processing_example(
    data: &[String], 
    name: &str, 
    batch_size: usize
) -> Result<(), Box<dyn std::error::Error>> {
    let config = FastEmbedConfigBuilder::new()
        .max_batch_size(batch_size)
        .show_download_progress(false)
        .build();
    
    let provider = FastEmbedProvider::new(FastEmbedModel::BGESmallENV15, config).await?;
    
    let start = Instant::now();
    let embeddings = provider.embed_batch(data).await?;
    let duration = start.elapsed();
    
    println!("✅ {} batch (size {}): {} texts in {:?}", 
             name, batch_size, embeddings.len(), duration);
    println!("   Throughput: {:.1} texts/sec", 
             embeddings.len() as f64 / duration.as_secs_f64());
    println!("   Average: {:.2}ms per text", 
             duration.as_millis() as f64 / embeddings.len() as f64);
    
    Ok(())
}

async fn await_streaming_example(data: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let provider = FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await?;
    
    let chunk_size = 100;
    let mut total_processed = 0;
    let start = Instant::now();
    
    println!("🔄 Processing {} texts in chunks of {}", data.len(), chunk_size);
    
    for (i, chunk) in data.chunks(chunk_size).enumerate() {
        let chunk_start = Instant::now();
        let embeddings = provider.embed_batch(chunk).await?;
        let chunk_duration = chunk_start.elapsed();
        
        total_processed += embeddings.len();
        
        println!("   Chunk {}: {} texts in {:?} ({:.1} texts/sec)", 
                 i + 1, 
                 embeddings.len(), 
                 chunk_duration,
                 embeddings.len() as f64 / chunk_duration.as_secs_f64());
        
        // Simulate some processing time
        sleep(Duration::from_millis(10)).await;
    }
    
    let total_duration = start.elapsed();
    println!("✅ Streaming complete: {} texts in {:?}", total_processed, total_duration);
    println!("   Overall throughput: {:.1} texts/sec", 
             total_processed as f64 / total_duration.as_secs_f64());
    
    Ok(())
}

async fn await_parallel_comparison(data: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let test_data = &data[..200]; // Use smaller dataset for comparison
    
    // Sequential processing
    println!("🔄 Sequential processing...");
    let provider = FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await?;
    
    let start = Instant::now();
    let _embeddings = provider.embed_batch(test_data).await?;
    let sequential_duration = start.elapsed();
    
    println!("✅ Sequential: {} texts in {:?}", test_data.len(), sequential_duration);
    
    // Parallel processing (multiple smaller batches)
    println!("🔄 Parallel processing...");
    let chunk_size = 50;
    let chunks: Vec<&[String]> = test_data.chunks(chunk_size).collect();
    
    let start = Instant::now();
    let mut handles = Vec::new();
    
    for chunk in chunks {
        let provider = FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await?;
        let chunk_data = chunk.to_vec();
        
        let handle = tokio::spawn(async move {
            provider.embed_batch(&chunk_data).await
        });
        
        handles.push(handle);
    }
    
    let mut total_embeddings = 0;
    for handle in handles {
        match handle.await? {
            Ok(embeddings) => total_embeddings += embeddings.len(),
            Err(e) => eprintln!("Chunk processing failed: {}", e),
        }
    }
    
    let parallel_duration = start.elapsed();
    
    println!("✅ Parallel: {} texts in {:?}", total_embeddings, parallel_duration);
    
    let speedup = sequential_duration.as_secs_f64() / parallel_duration.as_secs_f64();
    println!("📈 Speedup: {:.2}x", speedup);
    
    Ok(())
}

async fn await_model_comparison(data: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let models = vec![
        (FastEmbedModel::BGESmallENV15, "BGE Small"),
        (FastEmbedModel::AllMiniLML6V2, "MiniLM L6"),
    ];
    
    println!("🔄 Comparing model performance on {} texts", data.len());
    
    for (model, name) in models {
        match FastEmbedProvider::with_model(model).await {
            Ok(provider) => {
                let start = Instant::now();
                let embeddings = provider.embed_batch(data).await?;
                let duration = start.elapsed();
                
                println!("✅ {}: {} texts in {:?} ({:.1} texts/sec, {}D)", 
                         name,
                         embeddings.len(), 
                         duration,
                         embeddings.len() as f64 / duration.as_secs_f64(),
                         embeddings[0].len());
            }
            Err(e) => {
                println!("⚠️  {}: Model not available ({})", name, e);
            }
        }
    }
    
    Ok(())
}

fn generate_test_data(count: usize) -> Vec<String> {
    let templates = vec![
        "The quick brown fox jumps over the lazy dog number {}.",
        "Machine learning model {} is processing natural language text.",
        "Vector database {} stores high-dimensional embeddings efficiently.",
        "Artificial intelligence system {} generates human-like responses.",
        "Deep learning algorithm {} learns patterns from training data.",
        "Neural network {} transforms input into meaningful representations.",
        "Language model {} understands context and generates coherent text.",
        "Embedding model {} converts text into numerical vector representations.",
        "Search engine {} retrieves relevant documents using semantic similarity.",
        "Recommendation system {} suggests items based on user preferences.",
    ];
    
    (0..count)
        .map(|i| {
            let template = &templates[i % templates.len()];
            template.replace("{}", &i.to_string())
        })
        .collect()
}

/// Performance metrics for batch processing
#[derive(Debug)]
struct PerformanceMetrics {
    total_texts: usize,
    total_duration: Duration,
    throughput: f64,
    avg_per_text: f64,
}

impl PerformanceMetrics {
    fn new(total_texts: usize, total_duration: Duration) -> Self {
        let throughput = total_texts as f64 / total_duration.as_secs_f64();
        let avg_per_text = total_duration.as_millis() as f64 / total_texts as f64;
        
        Self {
            total_texts,
            total_duration,
            throughput,
            avg_per_text,
        }
    }
    
    fn display(&self, name: &str) {
        println!("📊 {} Performance:", name);
        println!("   Total texts: {}", self.total_texts);
        println!("   Total time: {:?}", self.total_duration);
        println!("   Throughput: {:.1} texts/sec", self.throughput);
        println!("   Average: {:.2}ms per text", self.avg_per_text);
    }
}
