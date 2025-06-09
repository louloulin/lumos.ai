//! # LumosAI FastEmbed Integration
//!
//! This crate provides FastEmbed integration for LumosAI vector storage,
//! enabling local embedding generation without external API dependencies.
//!
//! ## Features
//!
//! - **Local Processing**: Generate embeddings locally without API calls
//! - **Multiple Models**: Support for various pre-trained models
//! - **High Performance**: Optimized for batch processing
//! - **Easy Integration**: Seamless integration with LumosAI vector storage
//!
//! ## Quick Start
//!
//! ```rust
//! use lumosai_vector_fastembed::{FastEmbedProvider, FastEmbedModel};
//! use lumosai_vector_core::traits::EmbeddingModel;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create FastEmbed provider
//!     let provider = FastEmbedProvider::new(FastEmbedModel::BGESmallENV15).await?;
//!     
//!     // Generate embedding
//!     let embedding = provider.embed_text("Hello, world!").await?;
//!     println!("Embedding dimensions: {}", embedding.len());
//!     
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod models;
pub mod provider;
pub mod error;

pub use models::{FastEmbedModel, ModelInfo};
pub use provider::FastEmbedProvider;
pub use error::{FastEmbedError, Result};

// Re-export core types for convenience
pub use lumosai_vector_core::types::{Vector, Metadata};
pub use lumosai_vector_core::traits::EmbeddingModel;

/// FastEmbed client for managing embedding models
#[derive(Clone)]
pub struct FastEmbedClient {
    /// Cache of initialized models
    models: Arc<Mutex<HashMap<String, Arc<fastembed::TextEmbedding>>>>,
    
    /// Default cache directory for models
    cache_dir: Option<String>,
    
    /// Default configuration
    config: FastEmbedConfig,
}

/// Configuration for FastEmbed client
#[derive(Debug, Clone)]
pub struct FastEmbedConfig {
    /// Maximum batch size for processing
    pub max_batch_size: usize,
    
    /// Show download progress
    pub show_download_progress: bool,
    
    /// Number of threads for processing
    pub num_threads: Option<usize>,
    
    /// Cache directory for model files
    pub cache_dir: Option<String>,
}

impl Default for FastEmbedConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 256,
            show_download_progress: true,
            num_threads: None,
            cache_dir: None,
        }
    }
}

impl FastEmbedClient {
    /// Create a new FastEmbed client with default configuration
    pub fn new() -> Self {
        Self {
            models: Arc::new(Mutex::new(HashMap::new())),
            cache_dir: None,
            config: FastEmbedConfig::default(),
        }
    }
    
    /// Create a new FastEmbed client with custom configuration
    pub fn with_config(config: FastEmbedConfig) -> Self {
        Self {
            models: Arc::new(Mutex::new(HashMap::new())),
            cache_dir: config.cache_dir.clone(),
            config,
        }
    }
    
    /// Create an embedding provider for the specified model
    pub async fn embedding_provider(&self, model: FastEmbedModel) -> Result<FastEmbedProvider> {
        FastEmbedProvider::new(model, self.config.clone()).await
    }
    
    /// Get or create a model instance
    async fn get_or_create_model(
        &self,
        model: &FastEmbedModel,
    ) -> Result<Arc<fastembed::TextEmbedding>> {
        let model_key = model.model_name().to_string();
        let mut models = self.models.lock().await;
        
        if let Some(existing_model) = models.get(&model_key) {
            return Ok(existing_model.clone());
        }
        
        // Create new model instance
        let mut init_options = fastembed::InitOptions::new(model.to_fastembed_model())
            .with_show_download_progress(self.config.show_download_progress);
        
        if let Some(cache_dir) = &self.config.cache_dir {
            init_options = init_options.with_cache_dir(cache_dir.into());
        }
        
        // Note: with_num_threads is not available in current fastembed version
        // if let Some(num_threads) = self.config.num_threads {
        //     init_options = init_options.with_num_threads(num_threads);
        // }
        
        let embedding_model = fastembed::TextEmbedding::try_new(init_options)
            .map_err(|e| FastEmbedError::ModelInitialization(e.to_string()))?;
        
        let model_arc = Arc::new(embedding_model);
        models.insert(model_key, model_arc.clone());
        
        Ok(model_arc)
    }
    
    /// List available models
    pub fn available_models() -> Vec<FastEmbedModel> {
        vec![
            FastEmbedModel::BGESmallENV15,
            FastEmbedModel::BGEBaseENV15,
            FastEmbedModel::BGELargeENV15,
            FastEmbedModel::AllMiniLML6V2,
            FastEmbedModel::AllMiniLML12V2,
            FastEmbedModel::MultilingualE5Small,
            FastEmbedModel::MultilingualE5Base,
            FastEmbedModel::MultilingualE5Large,
        ]
    }
    
    /// Get model information
    pub fn model_info(model: &FastEmbedModel) -> ModelInfo {
        ModelInfo {
            name: model.model_name().to_string(),
            dimensions: model.dimensions(),
            max_sequence_length: model.max_sequence_length(),
            description: model.description().to_string(),
            language_support: model.language_support().iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Default for FastEmbedClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for FastEmbed configuration
pub struct FastEmbedConfigBuilder {
    config: FastEmbedConfig,
}

impl FastEmbedConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: FastEmbedConfig::default(),
        }
    }
    
    /// Set the maximum batch size
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
    
    /// Build the configuration
    pub fn build(self) -> FastEmbedConfig {
        self.config
    }
}

impl Default for FastEmbedConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_creation() {
        let client = FastEmbedClient::new();
        assert_eq!(client.config.max_batch_size, 256);
        assert!(client.config.show_download_progress);
    }
    
    #[test]
    fn test_config_builder() {
        let config = FastEmbedConfigBuilder::new()
            .max_batch_size(128)
            .show_download_progress(false)
            .num_threads(4)
            .cache_dir("/tmp/fastembed")
            .build();
        
        assert_eq!(config.max_batch_size, 128);
        assert!(!config.show_download_progress);
        assert_eq!(config.num_threads, Some(4));
        assert_eq!(config.cache_dir, Some("/tmp/fastembed".to_string()));
    }
    
    #[test]
    fn test_available_models() {
        let models = FastEmbedClient::available_models();
        assert!(!models.is_empty());
        assert!(models.contains(&FastEmbedModel::BGESmallENV15));
    }
    
    #[test]
    fn test_model_info() {
        let info = FastEmbedClient::model_info(&FastEmbedModel::BGESmallENV15);
        assert_eq!(info.name, "BAAI/bge-small-en-v1.5");
        assert_eq!(info.dimensions, 384);
    }
}
