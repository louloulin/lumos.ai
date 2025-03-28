use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{RagError, Result};
use crate::embedding::provider::EmbeddingProvider;

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
        
        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Failed to get error response".to_string());
            
            return Err(RagError::Embedding(format!(
                "API error: {}, details: {}", 
                response.status(), 
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_openai_embed_text() {
        // Set up mock server
        let mock_server = server_url();
        
        let _m = mock("POST", "/embeddings")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "data": [
                    {
                        "embedding": [0.1, 0.2, 0.3],
                        "index": 0
                    }
                ],
                "model": "text-embedding-ada-002",
                "usage": {
                    "prompt_tokens": 5,
                    "total_tokens": 5
                }
            }"#)
            .create();
        
        let provider = OpenAIEmbeddingProvider::new(
            "fake-api-key".to_string(),
            "text-embedding-ada-002".to_string(),
        ).with_base_url(format!("{}/v1", mock_server));
        
        let embedding = provider.embed_text("Test").await.unwrap();
        assert_eq!(embedding, vec![0.1, 0.2, 0.3]);
    }
    
    #[tokio::test]
    async fn test_openai_embed_batch() {
        // Set up mock server
        let mock_server = server_url();
        
        let _m = mock("POST", "/embeddings")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "data": [
                    {
                        "embedding": [0.1, 0.2, 0.3],
                        "index": 0
                    },
                    {
                        "embedding": [0.4, 0.5, 0.6],
                        "index": 1
                    }
                ],
                "model": "text-embedding-ada-002",
                "usage": {
                    "prompt_tokens": 10,
                    "total_tokens": 10
                }
            }"#)
            .create();
        
        let provider = OpenAIEmbeddingProvider::new(
            "fake-api-key".to_string(),
            "text-embedding-ada-002".to_string(),
        ).with_base_url(format!("{}/v1", mock_server));
        
        let embeddings = provider.embed_batch(&["First".to_string(), "Second".to_string()]).await.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0], vec![0.1, 0.2, 0.3]);
        assert_eq!(embeddings[1], vec![0.4, 0.5, 0.6]);
    }
} 