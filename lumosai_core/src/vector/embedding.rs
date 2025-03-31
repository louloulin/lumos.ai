use std::sync::Arc;
use async_trait::async_trait;
use rand::Rng;

use super::types::EmbeddingService;
use crate::error::Result;

/// Random embedding service for testing
pub struct RandomEmbeddingService {
    /// Embedding dimension
    dimension: usize,
    /// Model name
    model_name: String,
}

impl RandomEmbeddingService {
    /// Create a new random embedding service
    pub fn new(dimension: usize, model_name: String) -> Self {
        Self {
            dimension,
            model_name,
        }
    }

    /// Create a default random embedding service with 384 dimensions
    pub fn default() -> Self {
        Self::new(384, "random-embedding-384".to_string())
    }
}

#[async_trait]
impl EmbeddingService for RandomEmbeddingService {
    async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut rng = rand::thread_rng();
        let mut embeddings = Vec::with_capacity(texts.len());

        for _ in texts {
            let embedding: Vec<f32> = (0..self.dimension)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();

            // Normalize the vector to unit length
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            let normalized: Vec<f32> = embedding.iter().map(|x| x / norm).collect();

            embeddings.push(normalized);
        }

        Ok(embeddings)
    }

    fn embedding_dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }
}

/// Create a new random embedding service
pub fn create_random_embedding() -> Arc<dyn EmbeddingService> {
    Arc::new(RandomEmbeddingService::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_random_embedding() {
        let service = RandomEmbeddingService::default();
        let texts = vec![
            "Hello, world!".to_string(),
            "How are you?".to_string(),
        ];

        let embeddings = service.embed_texts(&texts).await.unwrap();
        
        // Check count
        assert_eq!(embeddings.len(), texts.len());
        
        // Check dimensions
        for embedding in &embeddings {
            assert_eq!(embedding.len(), service.embedding_dimension());
            
            // Check normalization (length should be approximately 1.0)
            let length: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((length - 1.0).abs() < 1e-5);
        }
    }
} 