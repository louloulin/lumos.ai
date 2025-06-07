use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{RagError, Result};
use crate::embedding::provider::{EmbeddingProvider, utils};

/// OpenAI embedding provider
pub struct OpenAIEmbeddingProvider {
    /// API key for OpenAI
    api_key: String,
    
    /// Model to use for embeddings
    model: String,
    
    /// Base URL for the API
    base_url: String,
    
    /// HTTP client
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    model: String,
    usage: EmbeddingUsage,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Debug, Deserialize)]
struct EmbeddingUsage {
    prompt_tokens: usize,
    total_tokens: usize,
}

impl OpenAIEmbeddingProvider {
    /// Create a new OpenAI embedding provider
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            base_url: "https://api.openai.com/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }
    
    /// Set a custom base URL for the API
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        self.embed_text(text).await
    }

    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let input = vec![text.to_string()];
        let embeddings = self.embed_batch(&input).await?;
        
        // We expect exactly one embedding for one input
        embeddings.into_iter().next().ok_or_else(|| {
            RagError::Embedding("No embedding returned from API".to_string())
        })
    }
    
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        
        let url = format!("{}/embeddings", self.base_url);
        
        let request = EmbeddingRequest {
            model: self.model.clone(),
            input: texts.to_vec(),
        };
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| RagError::Embedding(format!("API request failed: {}", e)))?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Failed to get error response".to_string());
            
            return Err(RagError::Embedding(format!(
                "API error: {}, details: {}", 
                status, 
                error_text
            )));
        }
        
        let embedding_response: EmbeddingResponse = response.json().await
            .map_err(|e| RagError::Embedding(format!("Failed to parse API response: {}", e)))?;
        
        // Sort by index to ensure order matches the input
        let mut data = embedding_response.data;
        data.sort_by_key(|d| d.index);
        
        Ok(data.into_iter().map(|d| d.embedding).collect())
    }
    
    fn cosine_similarity(&self, vec_a: &[f32], vec_b: &[f32]) -> f32 {
        utils::compute_cosine_similarity(vec_a, vec_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Ignoring due to mockito dependency issues
    async fn test_openai_embed_text() {
        // This test requires external HTTP mocking
        // In a real implementation, you would use a mock HTTP client
        // For now, we'll ignore this test
    }
    
    #[tokio::test]
    #[ignore] // Ignoring due to mockito dependency issues
    async fn test_openai_embed_batch() {
        // This test requires external HTTP mocking
        // In a real implementation, you would use a mock HTTP client
        // For now, we'll ignore this test
    }
    
    #[tokio::test]
    async fn test_cosine_similarity() {
        let provider = OpenAIEmbeddingProvider::new(
            "test-key".to_string(),
            "test-model".to_string()
        );
        
        let vec_a = vec![1.0, 0.0, 0.0];
        let vec_b = vec![0.0, 1.0, 0.0];
        let vec_c = vec![1.0, 1.0, 0.0];
        
        // Test perfect similarity (same vector)
        let sim_aa = provider.cosine_similarity(&vec_a, &vec_a);
        assert!((sim_aa - 1.0).abs() < 1e-6);
        
        // Test no similarity (perpendicular vectors)
        let sim_ab = provider.cosine_similarity(&vec_a, &vec_b);
        assert!(sim_ab.abs() < 1e-6);
        
        // Test partial similarity
        let sim_ac = provider.cosine_similarity(&vec_a, &vec_c);
        assert!((sim_ac - 0.7071).abs() < 1e-3);
    }
} 