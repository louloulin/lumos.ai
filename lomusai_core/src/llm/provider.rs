use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::Result;
use super::types::LlmOptions;

/// Trait representing an LLM provider
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generate text from a prompt
    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String>;
    
    /// Generate a stream of text from a prompt
    async fn generate_stream<'a>(
        &'a self, 
        prompt: &'a str, 
        options: &'a LlmOptions
    ) -> Result<BoxStream<'a, Result<String>>>;
    
    /// Get embeddings for a text
    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>>;
} 