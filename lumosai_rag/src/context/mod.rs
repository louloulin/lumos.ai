//! Context management module for RAG systems
//! 
//! This module provides functionality for managing context windows, document ranking,
//! and context compression for RAG applications.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    types::{ScoredDocument, RetrievalResult},
    error::Result,
};

pub mod window;
pub mod compression;
pub mod ranking;

pub use window::*;
pub use compression::*;
pub use ranking::*;

/// Context management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Maximum number of documents in context
    pub max_documents: usize,
    
    /// Maximum total tokens in context
    pub max_tokens: usize,
    
    /// Context window strategy
    pub window_strategy: WindowStrategy,
    
    /// Document ranking strategy
    pub ranking_strategy: RankingStrategy,
    
    /// Context compression configuration
    pub compression: Option<CompressionConfig>,
    
    /// Whether to preserve document order
    pub preserve_order: bool,
    
    /// Minimum relevance score threshold
    pub min_relevance_score: Option<f32>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_documents: 10,
            max_tokens: 4000,
            window_strategy: WindowStrategy::Fixed,
            ranking_strategy: RankingStrategy::RelevanceScore,
            compression: None,
            preserve_order: false,
            min_relevance_score: Some(0.1),
        }
    }
}

/// Context window management strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowStrategy {
    /// Fixed window size
    Fixed,
    /// Sliding window with overlap
    Sliding { overlap: usize },
    /// Adaptive window based on content
    Adaptive { min_size: usize, max_size: usize },
    /// Hierarchical context with different levels
    Hierarchical { levels: Vec<usize> },
}

/// Document ranking strategies for context selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RankingStrategy {
    /// Use relevance scores from retrieval
    RelevanceScore,
    /// Rank by recency (requires timestamp metadata)
    Recency,
    /// Rank by document length
    Length,
    /// Custom ranking function
    Custom,
    /// Hybrid ranking combining multiple factors
    Hybrid {
        relevance_weight: f32,
        recency_weight: f32,
        length_weight: f32,
    },
}

/// Context compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Compression strategy
    pub strategy: CompressionStrategy,
    
    /// Target compression ratio (0.0 to 1.0)
    pub target_ratio: f32,
    
    /// Whether to preserve key information
    pub preserve_key_info: bool,
    
    /// Maximum compression iterations
    pub max_iterations: usize,
}

/// Context compression strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionStrategy {
    /// Remove redundant sentences
    Deduplication,
    /// Extract key sentences
    Extraction { max_sentences: usize },
    /// Summarize content
    Summarization { max_length: usize },
    /// Hybrid approach
    Hybrid,
}

/// Context manager for handling document context windows
pub struct ContextManager {
    config: ContextConfig,
    window_manager: Box<dyn ContextWindow>,
    ranker: Box<dyn DocumentRanker>,
    compressor: Option<Box<dyn ContextCompressor>>,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new(config: ContextConfig) -> Self {
        let window_manager = Self::create_window_manager(&config.window_strategy);
        let ranker = Self::create_ranker(&config.ranking_strategy);
        let compressor = config.compression.as_ref()
            .map(|comp_config| Self::create_compressor(comp_config));

        Self {
            config,
            window_manager,
            ranker,
            compressor,
        }
    }

    /// Process retrieval results into a managed context
    pub async fn process_context(&self, results: RetrievalResult) -> Result<ManagedContext> {
        // Keep a copy of original documents for compression ratio calculation
        let original_documents = results.documents.clone();

        // Filter by minimum relevance score
        let mut documents = if let Some(min_score) = self.config.min_relevance_score {
            results.documents.into_iter()
                .filter(|doc| doc.score >= min_score)
                .collect()
        } else {
            results.documents
        };

        // Rank documents
        documents = self.ranker.rank_documents(documents).await?;

        // Apply window management
        let windowed_docs = self.window_manager.apply_window(
            documents,
            self.config.max_documents,
            self.config.max_tokens,
        ).await?;

        // Apply compression if configured
        let final_docs = if let Some(compressor) = &self.compressor {
            compressor.compress_context(windowed_docs).await?
        } else {
            windowed_docs
        };

        let total_tokens = self.estimate_tokens(&final_docs);
        let compression_ratio = self.calculate_compression_ratio(&original_documents, &final_docs);

        Ok(ManagedContext {
            documents: final_docs,
            total_tokens,
            compression_ratio,
            metadata: HashMap::new(),
        })
    }

    /// Estimate token count for documents
    fn estimate_tokens(&self, documents: &[ScoredDocument]) -> usize {
        documents.iter()
            .map(|doc| doc.document.content.split_whitespace().count())
            .sum()
    }

    /// Calculate compression ratio
    fn calculate_compression_ratio(&self, original: &[ScoredDocument], compressed: &[ScoredDocument]) -> f32 {
        let original_tokens = self.estimate_tokens(original);
        let compressed_tokens = self.estimate_tokens(compressed);
        
        if original_tokens == 0 {
            1.0
        } else {
            compressed_tokens as f32 / original_tokens as f32
        }
    }

    /// Create window manager based on strategy
    fn create_window_manager(strategy: &WindowStrategy) -> Box<dyn ContextWindow> {
        match strategy {
            WindowStrategy::Fixed => Box::new(FixedWindow::new()),
            WindowStrategy::Sliding { overlap } => Box::new(SlidingWindow::new(*overlap)),
            WindowStrategy::Adaptive { min_size, max_size } => {
                Box::new(AdaptiveWindow::new(*min_size, *max_size))
            }
            WindowStrategy::Hierarchical { levels } => {
                Box::new(HierarchicalWindow::new(levels.clone()))
            }
        }
    }

    /// Create ranker based on strategy
    fn create_ranker(strategy: &RankingStrategy) -> Box<dyn DocumentRanker> {
        match strategy {
            RankingStrategy::RelevanceScore => Box::new(RelevanceRanker::new()),
            RankingStrategy::Recency => Box::new(RecencyRanker::new()),
            RankingStrategy::Length => Box::new(LengthRanker::new()),
            RankingStrategy::Custom => Box::new(CustomRanker::new()),
            RankingStrategy::Hybrid { relevance_weight, recency_weight, length_weight } => {
                Box::new(HybridRanker::new(*relevance_weight, *recency_weight, *length_weight))
            }
        }
    }

    /// Create compressor based on configuration
    fn create_compressor(config: &CompressionConfig) -> Box<dyn ContextCompressor> {
        match &config.strategy {
            CompressionStrategy::Deduplication => Box::new(DeduplicationCompressor::new()),
            CompressionStrategy::Extraction { max_sentences } => {
                Box::new(ExtractionCompressor::new(*max_sentences))
            }
            CompressionStrategy::Summarization { max_length } => {
                Box::new(SummarizationCompressor::new(*max_length))
            }
            CompressionStrategy::Hybrid => Box::new(HybridCompressor::new()),
        }
    }
}

/// Managed context result
#[derive(Debug, Clone)]
pub struct ManagedContext {
    /// Selected and processed documents
    pub documents: Vec<ScoredDocument>,
    
    /// Total estimated tokens
    pub total_tokens: usize,
    
    /// Compression ratio applied
    pub compression_ratio: f32,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ManagedContext {
    /// Get the context as a single text string
    pub fn to_text(&self) -> String {
        self.documents
            .iter()
            .map(|doc| doc.document.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Get documents with scores
    pub fn get_documents(&self) -> &[ScoredDocument] {
        &self.documents
    }

    /// Check if context fits within token limit
    pub fn fits_token_limit(&self, limit: usize) -> bool {
        self.total_tokens <= limit
    }
}
