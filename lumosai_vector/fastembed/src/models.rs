//! FastEmbed model definitions and metadata

use serde::{Deserialize, Serialize};

/// Available FastEmbed models with their configurations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FastEmbedModel {
    /// BGE Small English v1.5 (384 dimensions)
    /// Best for: General purpose, fast inference
    BGESmallENV15,
    
    /// BGE Base English v1.5 (768 dimensions)
    /// Best for: Balanced performance and quality
    BGEBaseENV15,
    
    /// BGE Large English v1.5 (1024 dimensions)
    /// Best for: High quality embeddings, slower inference
    BGELargeENV15,
    
    /// All MiniLM L6 v2 (384 dimensions)
    /// Best for: Fast inference, good for semantic search
    AllMiniLML6V2,
    
    /// All MiniLM L12 v2 (384 dimensions)
    /// Best for: Better quality than L6, still fast
    AllMiniLML12V2,
    
    /// Multilingual E5 Small (384 dimensions)
    /// Best for: Multilingual applications, 100+ languages
    MultilingualE5Small,
    
    /// Multilingual E5 Base (768 dimensions)
    /// Best for: High-quality multilingual embeddings
    MultilingualE5Base,
    
    /// Multilingual E5 Large (1024 dimensions)
    /// Best for: Best multilingual quality, slower inference
    MultilingualE5Large,
    
    /// Custom model with name and dimensions
    Custom {
        name: String,
        dimensions: usize,
        max_sequence_length: Option<usize>,
    },
}

impl FastEmbedModel {
    /// Get the model name string used by FastEmbed
    pub fn model_name(&self) -> &str {
        match self {
            FastEmbedModel::BGESmallENV15 => "BAAI/bge-small-en-v1.5",
            FastEmbedModel::BGEBaseENV15 => "BAAI/bge-base-en-v1.5",
            FastEmbedModel::BGELargeENV15 => "BAAI/bge-large-en-v1.5",
            FastEmbedModel::AllMiniLML6V2 => "sentence-transformers/all-MiniLM-L6-v2",
            FastEmbedModel::AllMiniLML12V2 => "sentence-transformers/all-MiniLM-L12-v2",
            FastEmbedModel::MultilingualE5Small => "intfloat/multilingual-e5-small",
            FastEmbedModel::MultilingualE5Base => "intfloat/multilingual-e5-base",
            FastEmbedModel::MultilingualE5Large => "intfloat/multilingual-e5-large",
            FastEmbedModel::Custom { name, .. } => name,
        }
    }
    
    /// Get the embedding dimensions
    pub fn dimensions(&self) -> usize {
        match self {
            FastEmbedModel::BGESmallENV15 => 384,
            FastEmbedModel::BGEBaseENV15 => 768,
            FastEmbedModel::BGELargeENV15 => 1024,
            FastEmbedModel::AllMiniLML6V2 => 384,
            FastEmbedModel::AllMiniLML12V2 => 384,
            FastEmbedModel::MultilingualE5Small => 384,
            FastEmbedModel::MultilingualE5Base => 768,
            FastEmbedModel::MultilingualE5Large => 1024,
            FastEmbedModel::Custom { dimensions, .. } => *dimensions,
        }
    }
    
    /// Get the maximum sequence length supported by the model
    pub fn max_sequence_length(&self) -> usize {
        match self {
            FastEmbedModel::BGESmallENV15 => 512,
            FastEmbedModel::BGEBaseENV15 => 512,
            FastEmbedModel::BGELargeENV15 => 512,
            FastEmbedModel::AllMiniLML6V2 => 256,
            FastEmbedModel::AllMiniLML12V2 => 256,
            FastEmbedModel::MultilingualE5Small => 512,
            FastEmbedModel::MultilingualE5Base => 512,
            FastEmbedModel::MultilingualE5Large => 512,
            FastEmbedModel::Custom { max_sequence_length, .. } => {
                max_sequence_length.unwrap_or(512)
            }
        }
    }
    
    /// Get a description of the model
    pub fn description(&self) -> &str {
        match self {
            FastEmbedModel::BGESmallENV15 => {
                "BGE Small English v1.5 - Fast and efficient for general purpose embedding tasks"
            }
            FastEmbedModel::BGEBaseENV15 => {
                "BGE Base English v1.5 - Balanced performance and quality for English text"
            }
            FastEmbedModel::BGELargeENV15 => {
                "BGE Large English v1.5 - High quality embeddings with larger model size"
            }
            FastEmbedModel::AllMiniLML6V2 => {
                "All MiniLM L6 v2 - Lightweight model optimized for semantic search"
            }
            FastEmbedModel::AllMiniLML12V2 => {
                "All MiniLM L12 v2 - Better quality than L6 while maintaining efficiency"
            }
            FastEmbedModel::MultilingualE5Small => {
                "Multilingual E5 Small - Supports 100+ languages with good performance"
            }
            FastEmbedModel::MultilingualE5Base => {
                "Multilingual E5 Base - High-quality multilingual embeddings"
            }
            FastEmbedModel::MultilingualE5Large => {
                "Multilingual E5 Large - Best multilingual quality with larger model size"
            }
            FastEmbedModel::Custom { name, .. } => name,
        }
    }
    
    /// Get the languages supported by the model
    pub fn language_support(&self) -> Vec<&str> {
        match self {
            FastEmbedModel::BGESmallENV15 
            | FastEmbedModel::BGEBaseENV15 
            | FastEmbedModel::BGELargeENV15 
            | FastEmbedModel::AllMiniLML6V2 
            | FastEmbedModel::AllMiniLML12V2 => {
                vec!["en"] // English only
            }
            FastEmbedModel::MultilingualE5Small 
            | FastEmbedModel::MultilingualE5Base 
            | FastEmbedModel::MultilingualE5Large => {
                vec![
                    "en", "zh", "es", "fr", "de", "it", "pt", "ru", "ja", "ko",
                    "ar", "hi", "th", "vi", "id", "ms", "tl", "nl", "sv", "da",
                    "no", "fi", "pl", "cs", "sk", "hu", "ro", "bg", "hr", "sl",
                    "et", "lv", "lt", "mt", "ga", "eu", "ca", "gl", "cy", "is",
                    "mk", "sq", "sr", "bs", "me", "hr", "sl", "sk", "cs", "pl",
                    // ... and many more (100+ languages total)
                ]
            }
            FastEmbedModel::Custom { .. } => {
                vec!["unknown"] // Custom models have unknown language support
            }
        }
    }
    
    /// Check if the model supports a specific language
    pub fn supports_language(&self, language: &str) -> bool {
        self.language_support().contains(&language)
    }
    
    /// Get the model family (BGE, MiniLM, E5, etc.)
    pub fn model_family(&self) -> ModelFamily {
        match self {
            FastEmbedModel::BGESmallENV15 
            | FastEmbedModel::BGEBaseENV15 
            | FastEmbedModel::BGELargeENV15 => ModelFamily::BGE,
            
            FastEmbedModel::AllMiniLML6V2 
            | FastEmbedModel::AllMiniLML12V2 => ModelFamily::MiniLM,
            
            FastEmbedModel::MultilingualE5Small 
            | FastEmbedModel::MultilingualE5Base 
            | FastEmbedModel::MultilingualE5Large => ModelFamily::E5,
            
            FastEmbedModel::Custom { .. } => ModelFamily::Custom,
        }
    }
    
    /// Convert to fastembed EmbeddingModel enum
    pub fn to_fastembed_model(&self) -> fastembed::EmbeddingModel {
        match self {
            FastEmbedModel::BGESmallENV15 => fastembed::EmbeddingModel::BGESmallENV15,
            FastEmbedModel::BGEBaseENV15 => fastembed::EmbeddingModel::BGEBaseENV15,
            FastEmbedModel::BGELargeENV15 => fastembed::EmbeddingModel::BGELargeENV15,
            FastEmbedModel::AllMiniLML6V2 => fastembed::EmbeddingModel::AllMiniLML6V2,
            FastEmbedModel::AllMiniLML12V2 => fastembed::EmbeddingModel::AllMiniLML12V2,
            FastEmbedModel::MultilingualE5Small => fastembed::EmbeddingModel::MultilingualE5Small,
            FastEmbedModel::MultilingualE5Base => fastembed::EmbeddingModel::MultilingualE5Base,
            FastEmbedModel::MultilingualE5Large => fastembed::EmbeddingModel::MultilingualE5Large,
            FastEmbedModel::Custom { .. } => {
                // For custom models, default to BGE Small
                // In practice, custom models would need special handling
                fastembed::EmbeddingModel::BGESmallENV15
            }
        }
    }
    
    /// Get recommended use cases for the model
    pub fn use_cases(&self) -> Vec<&str> {
        match self {
            FastEmbedModel::BGESmallENV15 => {
                vec!["semantic search", "document similarity", "clustering", "classification"]
            }
            FastEmbedModel::BGEBaseENV15 => {
                vec!["semantic search", "document similarity", "RAG systems", "question answering"]
            }
            FastEmbedModel::BGELargeENV15 => {
                vec!["high-quality RAG", "research applications", "complex similarity tasks"]
            }
            FastEmbedModel::AllMiniLML6V2 => {
                vec!["fast semantic search", "real-time applications", "mobile deployment"]
            }
            FastEmbedModel::AllMiniLML12V2 => {
                vec!["semantic search", "document clustering", "content recommendation"]
            }
            FastEmbedModel::MultilingualE5Small => {
                vec!["multilingual search", "cross-language retrieval", "international applications"]
            }
            FastEmbedModel::MultilingualE5Base => {
                vec!["multilingual RAG", "cross-language similarity", "global content analysis"]
            }
            FastEmbedModel::MultilingualE5Large => {
                vec!["high-quality multilingual RAG", "research", "enterprise multilingual systems"]
            }
            FastEmbedModel::Custom { .. } => {
                vec!["custom applications"]
            }
        }
    }
}

/// Model family classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFamily {
    /// BGE (Beijing Academy of Artificial Intelligence) models
    BGE,
    /// MiniLM models (Microsoft)
    MiniLM,
    /// E5 models (Microsoft)
    E5,
    /// Custom models
    Custom,
}

/// Detailed information about a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model name
    pub name: String,
    /// Embedding dimensions
    pub dimensions: usize,
    /// Maximum sequence length
    pub max_sequence_length: usize,
    /// Model description
    pub description: String,
    /// Supported languages
    pub language_support: Vec<String>,
}

impl ModelInfo {
    /// Create model info from a FastEmbedModel
    pub fn from_model(model: &FastEmbedModel) -> Self {
        Self {
            name: model.model_name().to_string(),
            dimensions: model.dimensions(),
            max_sequence_length: model.max_sequence_length(),
            description: model.description().to_string(),
            language_support: model.language_support().iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_properties() {
        let model = FastEmbedModel::BGESmallENV15;
        assert_eq!(model.model_name(), "BAAI/bge-small-en-v1.5");
        assert_eq!(model.dimensions(), 384);
        assert_eq!(model.max_sequence_length(), 512);
        assert!(model.supports_language("en"));
        assert!(!model.supports_language("zh"));
    }
    
    #[test]
    fn test_multilingual_model() {
        let model = FastEmbedModel::MultilingualE5Small;
        assert!(model.supports_language("en"));
        assert!(model.supports_language("zh"));
        assert!(model.supports_language("es"));
        assert_eq!(model.model_family(), ModelFamily::E5);
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
        assert_eq!(model.model_family(), ModelFamily::Custom);
    }
    
    #[test]
    fn test_model_info() {
        let model = FastEmbedModel::BGEBaseENV15;
        let info = ModelInfo::from_model(&model);
        
        assert_eq!(info.name, "BAAI/bge-base-en-v1.5");
        assert_eq!(info.dimensions, 768);
        assert!(info.language_support.contains(&"en".to_string()));
    }
}
