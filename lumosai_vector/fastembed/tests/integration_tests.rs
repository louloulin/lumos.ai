//! Integration tests for FastEmbed provider

use lumosai_vector_fastembed::{
    FastEmbedProvider, FastEmbedModel, FastEmbedConfigBuilder, FastEmbedClient
};
use lumosai_vector_core::traits::EmbeddingModel;

#[tokio::test]
async fn test_basic_embedding() {
    // Skip test if FastEmbed models are not available (e.g., in CI)
    let provider = match FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Skipping test: FastEmbed model not available");
            return;
        }
    };
    
    let text = "This is a test sentence for embedding generation.";
    let embedding = provider.embed_text(text).await.unwrap();
    
    assert_eq!(embedding.len(), 384); // BGE Small has 384 dimensions
    assert!(embedding.iter().any(|&x| x != 0.0)); // Should not be all zeros
}

#[tokio::test]
async fn test_batch_embedding() {
    let provider = match FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Skipping test: FastEmbed model not available");
            return;
        }
    };
    
    let texts = vec![
        "First test sentence.".to_string(),
        "Second test sentence.".to_string(),
        "Third test sentence.".to_string(),
    ];
    
    let embeddings = provider.embed_batch(&texts).await.unwrap();
    
    assert_eq!(embeddings.len(), 3);
    for embedding in embeddings {
        assert_eq!(embedding.len(), 384);
        assert!(embedding.iter().any(|&x| x != 0.0));
    }
}

#[tokio::test]
async fn test_empty_batch() {
    let provider = match FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Skipping test: FastEmbed model not available");
            return;
        }
    };
    
    let texts: Vec<String> = vec![];
    let embeddings = provider.embed_batch(&texts).await.unwrap();
    
    assert_eq!(embeddings.len(), 0);
}

#[tokio::test]
async fn test_custom_config() {
    let config = FastEmbedConfigBuilder::new()
        .max_batch_size(64)
        .show_download_progress(false)
        .build();
    
    let provider = match FastEmbedProvider::new(FastEmbedModel::BGESmallENV15, config).await {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Skipping test: FastEmbed model not available");
            return;
        }
    };
    
    assert_eq!(provider.dimensions(), 384);
    assert_eq!(provider.model_name(), "BAAI/bge-small-en-v1.5");
    assert_eq!(provider.config().max_batch_size, 64);
}

#[tokio::test]
async fn test_model_properties() {
    // Test model properties without actually loading the model
    let model = FastEmbedModel::BGESmallENV15;
    assert_eq!(model.model_name(), "BAAI/bge-small-en-v1.5");
    assert_eq!(model.dimensions(), 384);
    assert_eq!(model.max_sequence_length(), 512);
    assert!(model.supports_language("en"));
    assert!(!model.supports_language("zh"));
    
    let model = FastEmbedModel::MultilingualE5Small;
    assert!(model.supports_language("en"));
    assert!(model.supports_language("zh"));
    assert!(model.supports_language("es"));
}

#[tokio::test]
async fn test_similarity_calculation() {
    let provider = match FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Skipping test: FastEmbed model not available");
            return;
        }
    };
    
    let text1 = "The cat sits on the mat.";
    let text2 = "A cat is sitting on a mat.";
    let text3 = "The weather is sunny today.";
    
    let embedding1 = provider.embed_text(text1).await.unwrap();
    let embedding2 = provider.embed_text(text2).await.unwrap();
    let embedding3 = provider.embed_text(text3).await.unwrap();
    
    let similarity_12 = cosine_similarity(&embedding1, &embedding2);
    let similarity_13 = cosine_similarity(&embedding1, &embedding3);
    
    // Similar sentences should have higher similarity
    assert!(similarity_12 > similarity_13);
    assert!(similarity_12 > 0.5); // Should be reasonably similar
}

#[tokio::test]
async fn test_large_batch_processing() {
    let provider = match FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Skipping test: FastEmbed model not available");
            return;
        }
    };
    
    // Create a large batch to test chunking
    let texts: Vec<String> = (0..300)
        .map(|i| format!("This is test sentence number {}.", i))
        .collect();
    
    let embeddings = provider.embed_batch(&texts).await.unwrap();
    
    assert_eq!(embeddings.len(), 300);
    for embedding in embeddings {
        assert_eq!(embedding.len(), 384);
    }
}

#[tokio::test]
async fn test_client_functionality() {
    let client = FastEmbedClient::new();
    
    // Test available models
    let models = FastEmbedClient::available_models();
    assert!(!models.is_empty());
    assert!(models.contains(&FastEmbedModel::BGESmallENV15));
    
    // Test model info
    let info = FastEmbedClient::model_info(&FastEmbedModel::BGESmallENV15);
    assert_eq!(info.name, "BAAI/bge-small-en-v1.5");
    assert_eq!(info.dimensions, 384);
    
    // Test provider creation through client
    match client.embedding_provider(FastEmbedModel::BGESmallENV15).await {
        Ok(provider) => {
            assert_eq!(provider.dimensions(), 384);
        }
        Err(_) => {
            eprintln!("Skipping provider test: FastEmbed model not available");
        }
    }
}

#[test]
fn test_error_handling() {
    use lumosai_vector_fastembed::error::FastEmbedError;
    
    let err = FastEmbedError::TextTooLong {
        length: 1000,
        max_length: 512,
    };
    
    assert!(err.is_recoverable());
    assert_eq!(err.category(), "text_length");
    
    let err = FastEmbedError::ModelInitialization("Test error".to_string());
    assert!(!err.is_recoverable());
    assert_eq!(err.category(), "model_init");
}

#[test]
fn test_config_builder() {
    let config = FastEmbedConfigBuilder::new()
        .max_batch_size(128)
        .show_download_progress(false)
        .num_threads(4)
        .cache_dir("/tmp/test")
        .build();
    
    assert_eq!(config.max_batch_size, 128);
    assert!(!config.show_download_progress);
    assert_eq!(config.num_threads, Some(4));
    assert_eq!(config.cache_dir, Some("/tmp/test".to_string()));
}

#[test]
fn test_custom_model() {
    let model = FastEmbedModel::Custom {
        name: "custom-model".to_string(),
        dimensions: 512,
        max_sequence_length: Some(1024),
    };
    
    assert_eq!(model.model_name(), "custom-model");
    assert_eq!(model.dimensions(), 512);
    assert_eq!(model.max_sequence_length(), 1024);
}

// Helper function for similarity calculation
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

// Benchmark tests (only run with --features bench)
#[cfg(feature = "bench")]
mod benchmarks {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn bench_single_embedding() {
        let provider = match FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await {
            Ok(p) => p,
            Err(_) => return,
        };
        
        let text = "This is a benchmark test sentence.";
        let iterations = 100;
        
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = provider.embed_text(text).await.unwrap();
        }
        let duration = start.elapsed();
        
        println!("Single embedding: {:.2}ms per text", 
                 duration.as_millis() as f64 / iterations as f64);
    }
    
    #[tokio::test]
    async fn bench_batch_embedding() {
        let provider = match FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await {
            Ok(p) => p,
            Err(_) => return,
        };
        
        let texts: Vec<String> = (0..100)
            .map(|i| format!("Benchmark test sentence number {}.", i))
            .collect();
        
        let start = Instant::now();
        let embeddings = provider.embed_batch(&texts).await.unwrap();
        let duration = start.elapsed();
        
        println!("Batch embedding: {:.2}ms per text", 
                 duration.as_millis() as f64 / embeddings.len() as f64);
    }
}
