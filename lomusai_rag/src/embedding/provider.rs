use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::Result;
use crate::types::Document;

/// Provider for generating embeddings from text
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate an embedding for a single text
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>>;
    
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
    fn cosine_similarity(&self, vec_a: &[f32], vec_b: &[f32]) -> f32 {
        utils::compute_cosine_similarity(vec_a, vec_b)
    }
}

/// Utility functions for embeddings
pub mod utils {
    /// Compute cosine similarity between two vectors
    pub fn compute_cosine_similarity(vec_a: &[f32], vec_b: &[f32]) -> f32 {
        if vec_a.len() != vec_b.len() || vec_a.is_empty() {
            return 0.0;
        }
        
        let dot_product: f32 = vec_a.iter().zip(vec_b.iter())
            .map(|(a, b)| a * b)
            .sum();
            
        let mag_a: f32 = vec_a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag_b: f32 = vec_b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (mag_a * mag_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Metadata;
    
    #[tokio::test]
    async fn test_cosine_similarity() {
        // Test the basic utility function
        let vec_a = vec![1.0, 0.0, 0.0];
        let vec_b = vec![0.0, 1.0, 0.0]; 
        let vec_c = vec![1.0, 1.0, 0.0];
        
        let sim_aa = utils::compute_cosine_similarity(&vec_a, &vec_a);
        assert!((sim_aa - 1.0).abs() < 1e-6);
        
        let sim_ab = utils::compute_cosine_similarity(&vec_a, &vec_b);
        assert!(sim_ab.abs() < 1e-6);
        
        let sim_ac = utils::compute_cosine_similarity(&vec_a, &vec_c);
        assert!((sim_ac - 0.7071).abs() < 1e-3);
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