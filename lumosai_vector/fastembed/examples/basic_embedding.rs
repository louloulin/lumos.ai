//! Basic FastEmbed embedding example
//!
//! This example demonstrates how to use FastEmbed for generating embeddings
//! with different models and configurations.

use lumosai_vector_core::traits::EmbeddingModel;
use lumosai_vector_fastembed::{
    FastEmbedClient, FastEmbedConfigBuilder, FastEmbedModel, FastEmbedProvider,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🚀 LumosAI FastEmbed Basic Example");
    println!("==================================");

    // Example 1: Simple embedding with default model
    println!("\n📝 Example 1: Simple embedding with BGE Small model");

    let provider = FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await?;

    let text = "Hello, world! This is a test sentence for embedding generation.";
    let embedding = provider.embed_text(text).await?;

    println!("✅ Generated embedding for: \"{}\"", text);
    println!("   Dimensions: {}", embedding.len());
    println!(
        "   First 5 values: {:?}",
        &embedding[..5.min(embedding.len())]
    );

    // Example 2: Batch embedding
    println!("\n📝 Example 2: Batch embedding");

    let texts = vec![
        "The quick brown fox jumps over the lazy dog.".to_string(),
        "Machine learning is a subset of artificial intelligence.".to_string(),
        "Rust is a systems programming language focused on safety and performance.".to_string(),
        "Vector databases are optimized for similarity search.".to_string(),
    ];

    let embeddings = provider.embed_batch(&texts).await?;

    println!("✅ Generated {} embeddings", embeddings.len());
    for (i, (text, embedding)) in texts.iter().zip(embeddings.iter()).enumerate() {
        println!(
            "   Text {}: \"{}...\" -> {}D vector",
            i + 1,
            &text[..50.min(text.len())],
            embedding.len()
        );
    }

    // Example 3: Different models comparison
    println!("\n📝 Example 3: Comparing different models");

    let models = vec![
        (FastEmbedModel::BGESmallENV15, "BGE Small (384D)"),
        (FastEmbedModel::BGEBaseENV15, "BGE Base (768D)"),
        (FastEmbedModel::AllMiniLML6V2, "MiniLM L6 (384D)"),
    ];

    let test_text = "Artificial intelligence and machine learning.";

    for (model, description) in models {
        match FastEmbedProvider::with_model(model).await {
            Ok(provider) => {
                let embedding = provider.embed_text(test_text).await?;
                println!(
                    "✅ {}: {}D embedding generated",
                    description,
                    embedding.len()
                );
            }
            Err(e) => {
                println!("⚠️  {}: Model not available ({})", description, e);
            }
        }
    }

    // Example 4: Custom configuration
    println!("\n📝 Example 4: Custom configuration");

    let config = FastEmbedConfigBuilder::new()
        .max_batch_size(64)
        .show_download_progress(true)
        .num_threads(2)
        .build();

    let provider = FastEmbedProvider::new(FastEmbedModel::BGESmallENV15, config).await?;

    let large_batch: Vec<String> = (0..100)
        .map(|i| format!("This is test sentence number {} for batch processing.", i))
        .collect();

    let start = std::time::Instant::now();
    let embeddings = provider.embed_batch(&large_batch).await?;
    let duration = start.elapsed();

    println!("✅ Processed {} texts in {:?}", embeddings.len(), duration);
    println!(
        "   Average: {:.2}ms per text",
        duration.as_millis() as f64 / embeddings.len() as f64
    );

    // Example 5: Model information
    println!("\n📝 Example 5: Model information");

    let client = FastEmbedClient::new();
    let available_models = FastEmbedClient::available_models();

    println!("📋 Available models:");
    for model in available_models {
        let info = FastEmbedClient::model_info(&model);
        println!("   • {} ({}D)", info.name, info.dimensions);
        println!(
            "     Languages: {:?}",
            &info.language_support[..3.min(info.language_support.len())]
        );
        println!("     Max length: {} tokens", info.max_sequence_length);
    }

    // Example 6: Multilingual embedding
    println!("\n📝 Example 6: Multilingual embedding");

    match FastEmbedProvider::with_model(FastEmbedModel::MultilingualE5Small).await {
        Ok(provider) => {
            let multilingual_texts = vec![
                "Hello, how are you?".to_string(),          // English
                "Hola, ¿cómo estás?".to_string(),           // Spanish
                "Bonjour, comment allez-vous?".to_string(), // French
                "你好，你好吗？".to_string(),               // Chinese
            ];

            let embeddings = provider.embed_batch(&multilingual_texts).await?;

            println!("✅ Generated multilingual embeddings:");
            for (text, embedding) in multilingual_texts.iter().zip(embeddings.iter()) {
                println!("   \"{}\" -> {}D vector", text, embedding.len());
            }

            // Calculate similarity between English and Spanish
            if embeddings.len() >= 2 {
                let similarity = cosine_similarity(&embeddings[0], &embeddings[1]);
                println!(
                    "   Similarity between English and Spanish: {:.3}",
                    similarity
                );
            }
        }
        Err(e) => {
            println!("⚠️  Multilingual model not available: {}", e);
        }
    }

    println!("\n🎉 All examples completed successfully!");

    Ok(())
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}
