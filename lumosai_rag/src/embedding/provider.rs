use async_trait::async_trait;

use crate::error::Result;
use crate::types::Document;

/// Provider for generating embeddings from text
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate an embedding for a single text
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate an embedding for a single text (alias for compatibility)
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        self.generate_embedding(text).await
    }
    
    /// Generate embeddings for multiple texts
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        
        for text in texts {
            results.push(self.embed_text(text).await?);
        }
        
        Ok(results)
    }
    
    /// Embed a document, modifying it in place to include the embedding
    async fn embed_document(&self, document: &mut Document) -> Result<()> {
        let embedding = self.embed_text(&document.content).await?;
        document.embedding = Some(embedding);
        Ok(())
    }
    
    /// Embed multiple documents, modifying them in place
    async fn embed_documents(&self, documents: &mut [Document]) -> Result<()> {
        // Extract contents for batch processing
        let contents: Vec<String> = documents.iter()
            .map(|doc| doc.content.clone())
            .collect();
        
        // Generate embeddings in batch
        let embeddings = self.embed_batch(&contents).await?;
        
        // Assign embeddings back to documents
        for (doc, embedding) in documents.iter_mut().zip(embeddings) {
            doc.embedding = Some(embedding);
        }
        
        Ok(())
    }
    
    /// Calculate the cosine similarity between two embeddings
    fn cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        utils::compute_cosine_similarity(vec1, vec2)
    }
}

/// Utility functions for embeddings
pub mod utils {
    /// Compute cosine similarity between two vectors
    pub fn compute_cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() || vec1.is_empty() {
            return 0.0;
        }
        
        let mut dot_product = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;
        
        for i in 0..vec1.len() {
            dot_product += vec1[i] * vec2[i];
            norm1 += vec1[i] * vec1[i];
            norm2 += vec2[i] * vec2[i];
        }
        
        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm1.sqrt() * norm2.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cosine_similarity() {
        // 测试余弦相似度计算
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![4.0, 5.0, 6.0];
        
        let similarity = utils::compute_cosine_similarity(&vec1, &vec2);
        let expected = 0.9746318;
        
        assert!((similarity - expected).abs() < 1e-6);
    }
    
    #[test]
    fn test_cosine_similarity_edge_cases() {
        // 测试长度不同的向量
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0];
        assert_eq!(utils::compute_cosine_similarity(&vec1, &vec2), 0.0);
        
        // 测试空向量
        let empty: Vec<f32> = vec![];
        assert_eq!(utils::compute_cosine_similarity(&empty, &vec1), 0.0);
        
        // 测试零向量
        let zeros = vec![0.0, 0.0, 0.0];
        assert_eq!(utils::compute_cosine_similarity(&zeros, &vec1), 0.0);
    }
    
    // Note: The following tests require a mock implementation
    // of EmbeddingProvider, which we will implement later.
    
    #[tokio::test]
    #[ignore]
    async fn test_embed_document() {
        // This test requires a mock implementation of EmbeddingProvider
        // which will be added in a future update
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_embed_documents() {
        // This test requires a mock implementation of EmbeddingProvider
        // which will be added in a future update
    }
} 