//! Integration tests for context management functionality

use lumosai_rag::{
    context::{
        ContextManager, ContextConfig, WindowStrategy, RankingStrategy, 
        CompressionConfig, CompressionStrategy
    },
    types::{Document, ScoredDocument, RetrievalResult, Metadata},
};
use chrono::Utc;

fn create_test_document(id: &str, content: &str, score: f32) -> ScoredDocument {
    ScoredDocument {
        document: Document {
            id: id.to_string(),
            content: content.to_string(),
            metadata: Metadata::new(),
            embedding: None,
        },
        score,
    }
}

fn create_test_document_with_timestamp(
    id: &str, 
    content: &str, 
    score: f32, 
    hours_ago: i64
) -> ScoredDocument {
    let mut metadata = Metadata::new();
    metadata.created_at = Some(Utc::now() - chrono::Duration::hours(hours_ago));
    
    ScoredDocument {
        document: Document {
            id: id.to_string(),
            content: content.to_string(),
            metadata,
            embedding: None,
        },
        score,
    }
}

#[tokio::test]
async fn test_basic_context_management() {
    let config = ContextConfig {
        max_documents: 3,
        max_tokens: 1000,
        window_strategy: WindowStrategy::Fixed,
        ranking_strategy: RankingStrategy::RelevanceScore,
        compression: None,
        preserve_order: false,
        min_relevance_score: Some(0.5),
    };

    let context_manager = ContextManager::new(config);

    let retrieval_result = RetrievalResult {
        documents: vec![
            create_test_document("1", "High relevance document with important information", 0.9),
            create_test_document("2", "Medium relevance document with some useful content", 0.7),
            create_test_document("3", "Low relevance document that should be filtered out", 0.3),
            create_test_document("4", "Another medium relevance document", 0.6),
        ],
        total_count: 4,
    };

    let managed_context = context_manager.process_context(retrieval_result).await.unwrap();

    // Should filter out low relevance document and limit to max_documents
    assert_eq!(managed_context.documents.len(), 3);
    
    // Should be sorted by relevance (highest first)
    assert_eq!(managed_context.documents[0].document.id, "1");
    assert_eq!(managed_context.documents[1].document.id, "2");
    assert_eq!(managed_context.documents[2].document.id, "4");
    
    // Should have reasonable token count
    assert!(managed_context.total_tokens > 0);
    assert!(managed_context.fits_token_limit(1000));
}

#[tokio::test]
async fn test_context_with_compression() {
    let compression_config = CompressionConfig {
        strategy: CompressionStrategy::Deduplication,
        target_ratio: 0.8,
        preserve_key_info: true,
        max_iterations: 3,
    };

    let config = ContextConfig {
        max_documents: 5,
        max_tokens: 2000,
        window_strategy: WindowStrategy::Fixed,
        ranking_strategy: RankingStrategy::RelevanceScore,
        compression: Some(compression_config),
        preserve_order: false,
        min_relevance_score: None,
    };

    let context_manager = ContextManager::new(config);

    let retrieval_result = RetrievalResult {
        documents: vec![
            create_test_document("1", "Unique content about artificial intelligence", 0.9),
            create_test_document("2", "Different content about machine learning", 0.8),
            create_test_document("3", "Unique content about artificial intelligence", 0.7), // Duplicate
            create_test_document("4", "Another unique piece about deep learning", 0.6),
        ],
        total_count: 4,
    };

    let managed_context = context_manager.process_context(retrieval_result).await.unwrap();

    // Should remove duplicate content
    assert_eq!(managed_context.documents.len(), 3);
    
    // Should have compression ratio less than 1.0
    assert!(managed_context.compression_ratio < 1.0);
    
    // Verify unique documents are preserved
    let ids: Vec<&str> = managed_context.documents.iter()
        .map(|d| d.document.id.as_str())
        .collect();
    assert!(ids.contains(&"1"));
    assert!(ids.contains(&"2"));
    assert!(ids.contains(&"4"));
    assert!(!ids.contains(&"3")); // Duplicate should be removed
}

#[tokio::test]
async fn test_context_with_recency_ranking() {
    let config = ContextConfig {
        max_documents: 3,
        max_tokens: 1000,
        window_strategy: WindowStrategy::Fixed,
        ranking_strategy: RankingStrategy::Recency,
        compression: None,
        preserve_order: false,
        min_relevance_score: None,
    };

    let context_manager = ContextManager::new(config);

    let retrieval_result = RetrievalResult {
        documents: vec![
            create_test_document_with_timestamp("1", "Old document", 0.9, 24), // 1 day ago
            create_test_document_with_timestamp("2", "Recent document", 0.7, 1), // 1 hour ago
            create_test_document_with_timestamp("3", "Very old document", 0.8, 72), // 3 days ago
            create_test_document_with_timestamp("4", "Medium age document", 0.6, 12), // 12 hours ago
        ],
        total_count: 4,
    };

    let managed_context = context_manager.process_context(retrieval_result).await.unwrap();

    assert_eq!(managed_context.documents.len(), 3);
    
    // Should be sorted by recency (most recent first)
    assert_eq!(managed_context.documents[0].document.id, "2"); // Most recent
    assert_eq!(managed_context.documents[1].document.id, "4"); // Medium age
    assert_eq!(managed_context.documents[2].document.id, "1"); // Older
    // "3" (very old) should not be included due to max_documents limit
}

#[tokio::test]
async fn test_adaptive_window_strategy() {
    let config = ContextConfig {
        max_documents: 3,
        max_tokens: 500,
        window_strategy: WindowStrategy::Adaptive { min_size: 1, max_size: 5 },
        ranking_strategy: RankingStrategy::RelevanceScore,
        compression: None,
        preserve_order: false,
        min_relevance_score: None,
    };

    let context_manager = ContextManager::new(config);

    let retrieval_result = RetrievalResult {
        documents: vec![
            create_test_document("1", "Very high quality document with excellent relevance", 0.95),
            create_test_document("2", "Good quality document with decent content", 0.8),
            create_test_document("3", "Average quality document", 0.6),
            create_test_document("4", "Lower quality document", 0.4),
        ],
        total_count: 4,
    };

    let managed_context = context_manager.process_context(retrieval_result).await.unwrap();

    // Adaptive window should prioritize high-quality documents
    assert!(!managed_context.documents.is_empty());
    assert_eq!(managed_context.documents[0].document.id, "1"); // Highest quality first
    
    // Should respect token limits
    assert!(managed_context.fits_token_limit(500));
}

#[tokio::test]
async fn test_context_to_text() {
    let config = ContextConfig::default();
    let context_manager = ContextManager::new(config);

    let retrieval_result = RetrievalResult {
        documents: vec![
            create_test_document("1", "First document content.", 0.9),
            create_test_document("2", "Second document content.", 0.8),
        ],
        total_count: 2,
    };

    let managed_context = context_manager.process_context(retrieval_result).await.unwrap();
    let text = managed_context.to_text();

    assert!(text.contains("First document content."));
    assert!(text.contains("Second document content."));
    assert!(text.contains("\n\n")); // Should be joined with double newlines
}

#[tokio::test]
async fn test_hybrid_ranking_strategy() {
    let config = ContextConfig {
        max_documents: 3,
        max_tokens: 1000,
        window_strategy: WindowStrategy::Fixed,
        ranking_strategy: RankingStrategy::Hybrid {
            relevance_weight: 0.6,
            recency_weight: 0.3,
            length_weight: 0.1,
        },
        compression: None,
        preserve_order: false,
        min_relevance_score: None,
    };

    let context_manager = ContextManager::new(config);

    let retrieval_result = RetrievalResult {
        documents: vec![
            create_test_document_with_timestamp("1", "Short but very relevant", 0.9, 24),
            create_test_document_with_timestamp("2", "Medium relevance but very recent", 0.7, 1),
            create_test_document_with_timestamp("3", "Long document with medium relevance and medium age", 0.6, 12),
        ],
        total_count: 3,
    };

    let managed_context = context_manager.process_context(retrieval_result).await.unwrap();

    assert_eq!(managed_context.documents.len(), 3);
    
    // Hybrid ranking should consider all factors
    // The exact order depends on the hybrid scoring algorithm
    assert!(!managed_context.documents.is_empty());
}
