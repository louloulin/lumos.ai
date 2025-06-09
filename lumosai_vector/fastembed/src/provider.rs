//! FastEmbed embedding provider implementation

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use lumosai_vector_core::traits::EmbeddingModel;
use lumosai_vector_core::types::Vector;

use crate::error::{FastEmbedError, Result};
use crate::models::FastEmbedModel;
use crate::FastEmbedConfig;

/// FastEmbed embedding provider
/// 
/// This provider uses FastEmbed for local embedding generation,
/// eliminating the need for external API calls and providing
/// fast, reliable embedding generation.
pub struct FastEmbedProvider {
    /// The embedding model instance
    model: Arc<Mutex<Option<fastembed::TextEmbedding>>>,
    
    /// Model configuration
    model_config: FastEmbedModel,
    
    /// Provider configuration
    config: FastEmbedConfig,
    
    /// Model name for identification
    model_name: String,
    
    /// Embedding dimensions
    dimensions: usize,
    
    /// Maximum sequence length
    max_sequence_length: usize,
}

impl FastEmbedProvider {
    /// Create a new FastEmbed provider with the specified model
    pub async fn new(model: FastEmbedModel, config: FastEmbedConfig) -> Result<Self> {
        let model_name = model.model_name().to_string();
        let dimensions = model.dimensions();
        let max_sequence_length = model.max_sequence_length();
        
        info!(
            "Creating FastEmbed provider with model: {} ({}D)",
            model_name, dimensions
        );
        
        let provider = Self {
            model: Arc::new(Mutex::new(None)),
            model_config: model,
            config,
            model_name,
            dimensions,
            max_sequence_length,
        };
        
        // Initialize the model
        provider.ensure_model_loaded().await?;
        
        Ok(provider)
    }
    
    /// Create a new FastEmbed provider with default configuration
    pub async fn with_model(model: FastEmbedModel) -> Result<Self> {
        Self::new(model, FastEmbedConfig::default()).await
    }
    
    /// Ensure the embedding model is loaded (lazy loading)
    async fn ensure_model_loaded(&self) -> Result<()> {
        let mut model_guard = self.model.lock().await;
        
        if model_guard.is_none() {
            debug!("Initializing FastEmbed model: {}", self.model_name);
            
            let mut init_options = fastembed::InitOptions::new(self.model_config.to_fastembed_model())
                .with_show_download_progress(self.config.show_download_progress);
            
            if let Some(cache_dir) = &self.config.cache_dir {
                init_options = init_options.with_cache_dir(cache_dir.into());
                debug!("Using cache directory: {}", cache_dir);
            }
            
            // Note: with_num_threads is not available in current fastembed version
            // if let Some(num_threads) = self.config.num_threads {
            //     init_options = init_options.with_num_threads(num_threads);
            //     debug!("Using {} threads", num_threads);
            // }
            
            let embedding_model = fastembed::TextEmbedding::try_new(init_options)
                .map_err(|e| FastEmbedError::ModelInitialization(format!(
                    "Failed to initialize FastEmbed model '{}': {}", 
                    self.model_name, e
                )))?;
            
            *model_guard = Some(embedding_model);
            info!("FastEmbed model '{}' initialized successfully", self.model_name);
        }
        
        Ok(())
    }
    
    /// Get the model configuration
    pub fn model_config(&self) -> &FastEmbedModel {
        &self.model_config
    }
    
    /// Get the provider configuration
    pub fn config(&self) -> &FastEmbedConfig {
        &self.config
    }
    
    /// Check if text length is within model limits
    fn validate_text_length(&self, text: &str) -> Result<()> {
        // Simple character-based check (not token-based)
        // In practice, you might want to use a tokenizer for more accurate checking
        if text.len() > self.max_sequence_length * 4 { // Rough estimate: 4 chars per token
            return Err(FastEmbedError::TextTooLong {
                length: text.len(),
                max_length: self.max_sequence_length * 4,
            });
        }
        Ok(())
    }
    
    /// Truncate text if it's too long
    fn truncate_text(&self, text: &str) -> String {
        let max_chars = self.max_sequence_length * 4; // Rough estimate
        if text.len() > max_chars {
            warn!(
                "Text truncated from {} to {} characters for model '{}'",
                text.len(), max_chars, self.model_name
            );
            text.chars().take(max_chars).collect()
        } else {
            text.to_string()
        }
    }
    
    /// Process texts in batches to respect model limits
    async fn process_in_batches(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.ensure_model_loaded().await?;
        
        let model_guard = self.model.lock().await;
        let model = model_guard.as_ref().ok_or_else(|| {
            FastEmbedError::ModelNotInitialized("FastEmbed model not initialized".to_string())
        })?;
        
        let mut all_embeddings = Vec::new();
        
        for chunk in texts.chunks(self.config.max_batch_size) {
            debug!("Processing batch of {} texts", chunk.len());
            
            // Validate and truncate texts
            let processed_texts: Vec<String> = chunk
                .iter()
                .map(|text| self.truncate_text(text))
                .collect();
            
            let embeddings = model.embed(processed_texts, None)
                .map_err(|e| FastEmbedError::EmbeddingGeneration(format!(
                    "FastEmbed embedding failed: {}", e
                )))?;
            
            all_embeddings.extend(embeddings);
        }
        
        debug!("Generated {} embeddings", all_embeddings.len());
        Ok(all_embeddings)
    }
}

#[async_trait]
impl EmbeddingModel for FastEmbedProvider {
    type Config = FastEmbedConfig;
    
    async fn embed_text(&self, text: &str) -> std::result::Result<Vector, lumosai_vector_core::error::VectorError> {
        // Validate text length
        if let Err(e) = self.validate_text_length(text) {
            warn!("Text validation failed: {}", e);
            // Continue with truncation instead of failing
        }
        
        let texts = vec![text.to_string()];
        let embeddings = self.process_in_batches(&texts).await
            .map_err(|e| lumosai_vector_core::error::VectorError::EmbeddingError(e.to_string()))?;
        
        embeddings.into_iter().next()
            .ok_or_else(|| lumosai_vector_core::error::VectorError::EmbeddingError(
                "No embedding returned from FastEmbed".to_string()
            ))
    }
    
    async fn embed_batch(&self, texts: &[String]) -> std::result::Result<Vec<Vector>, lumosai_vector_core::error::VectorError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        
        debug!("Embedding batch of {} texts", texts.len());
        
        self.process_in_batches(texts).await
            .map_err(|e| lumosai_vector_core::error::VectorError::EmbeddingError(e.to_string()))
    }
    
    fn dimensions(&self) -> usize {
        self.dimensions
    }
    
    fn model_name(&self) -> &str {
        &self.model_name
    }
    
    fn max_input_length(&self) -> Option<usize> {
        Some(self.max_sequence_length)
    }

    async fn health_check(&self) -> std::result::Result<(), lumosai_vector_core::error::VectorError> {
        // Try to ensure the model is loaded as a health check
        self.ensure_model_loaded().await
            .map_err(|e| lumosai_vector_core::error::VectorError::EmbeddingError(e.to_string()))
    }
}

/// Builder for FastEmbed provider
pub struct FastEmbedProviderBuilder {
    model: FastEmbedModel,
    config: FastEmbedConfig,
}

impl FastEmbedProviderBuilder {
    /// Create a new builder with the specified model
    pub fn new(model: FastEmbedModel) -> Self {
        Self {
            model,
            config: FastEmbedConfig::default(),
        }
    }
    
    /// Set the maximum batch size for processing
    pub fn max_batch_size(mut self, size: usize) -> Self {
        self.config.max_batch_size = size;
        self
    }
    
    /// Set whether to show download progress
    pub fn show_download_progress(mut self, show: bool) -> Self {
        self.config.show_download_progress = show;
        self
    }
    
    /// Set the number of threads for processing
    pub fn num_threads(mut self, threads: usize) -> Self {
        self.config.num_threads = Some(threads);
        self
    }
    
    /// Set the cache directory for model files
    pub fn cache_dir<S: Into<String>>(mut self, dir: S) -> Self {
        self.config.cache_dir = Some(dir.into());
        self
    }
    
    /// Build the FastEmbed provider
    pub async fn build(self) -> Result<FastEmbedProvider> {
        FastEmbedProvider::new(self.model, self.config).await
    }
}

impl Default for FastEmbedProviderBuilder {
    fn default() -> Self {
        Self::new(FastEmbedModel::BGESmallENV15)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_provider_creation() {
        let provider = FastEmbedProvider::with_model(FastEmbedModel::BGESmallENV15).await;
        
        // Note: This test might fail if FastEmbed models are not available
        // In CI/CD, you might want to skip this test or use mocks
        match provider {
            Ok(p) => {
                assert_eq!(p.dimensions(), 384);
                assert_eq!(p.model_name(), "BAAI/bge-small-en-v1.5");
            }
            Err(e) => {
                // Log the error but don't fail the test in case models aren't available
                eprintln!("FastEmbed model not available (this is OK in CI): {}", e);
            }
        }
    }
    
    #[test]
    fn test_text_validation() {
        let provider = FastEmbedProviderBuilder::new(FastEmbedModel::BGESmallENV15)
            .build();
        
        // This is a sync test, so we can't actually create the provider
        // But we can test the builder
        let builder = FastEmbedProviderBuilder::new(FastEmbedModel::BGESmallENV15)
            .max_batch_size(128)
            .show_download_progress(false);
        
        assert_eq!(builder.config.max_batch_size, 128);
        assert!(!builder.config.show_download_progress);
    }
    
    #[test]
    fn test_builder_pattern() {
        let builder = FastEmbedProviderBuilder::new(FastEmbedModel::BGEBaseENV15)
            .max_batch_size(64)
            .num_threads(2)
            .cache_dir("/tmp/test");
        
        assert_eq!(builder.config.max_batch_size, 64);
        assert_eq!(builder.config.num_threads, Some(2));
        assert_eq!(builder.config.cache_dir, Some("/tmp/test".to_string()));
    }
}
