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
    fn cosine_similarity(vec_a: &[f32], vec_b: &[f32]) -> f32 {
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
    use mockall::predicate::*;
    use mockall::*;
    
    mock! {
        pub EmbeddingProvider {
            fn embed_text(&self, text: &str) -> Result<Vec<f32>>;
            fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
        }
        
        #[async_trait]
        impl EmbeddingProvider for EmbeddingProvider {
            async fn embed_text(&self, text: &str) -> Result<Vec<f32>>;
            async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
        }
    }
    
    #[tokio::test]
    async fn test_cosine_similarity() {
        let vec_a = vec![1.0, 0.0, 0.0];
        let vec_b = vec![0.0, 1.0, 0.0];
        let vec_c = vec![1.0, 1.0, 0.0];
        
        assert_eq!(EmbeddingProvider::cosine_similarity(&vec_a, &vec_a), 1.0);
        assert_eq!(EmbeddingProvider::cosine_similarity(&vec_a, &vec_b), 0.0);
        
        let sim_ac = EmbeddingProvider::cosine_similarity(&vec_a, &vec_c);
        assert!(sim_ac > 0.0 && sim_ac < 1.0);
    }
    
    #[tokio::test]
    async fn test_embed_document() {
        let mut mock = MockEmbeddingProvider::new();
        
        mock.expect_embed_text()
            .with(eq("test content"))
            .returning(|_| Ok(vec![0.1, 0.2, 0.3]));
            
        let mut doc = Document {
            id: "test".to_string(),
            content: "test content".to_string(),
            metadata: Metadata::new(),
            embedding: None,
        };
        
        mock.embed_document(&mut doc).await.unwrap();
        
        assert_eq!(doc.embedding, Some(vec![0.1, 0.2, 0.3]));
    }
    
    #[tokio::test]
    async fn test_embed_documents() {
        let mut mock = MockEmbeddingProvider::new();
        
        mock.expect_embed_batch()
            .with(eq(vec!["doc1".to_string(), "doc2".to_string()]))
            .returning(|_| Ok(vec![vec![0.1, 0.2], vec![0.3, 0.4]]));
            
        let mut docs = vec![
            Document {
                id: "1".to_string(),
                content: "doc1".to_string(),
                metadata: Metadata::new(),
                embedding: None,
            },
            Document {
                id: "2".to_string(),
                content: "doc2".to_string(),
                metadata: Metadata::new(),
                embedding: None,
            },
        ];
        
        mock.embed_documents(&mut docs).await.unwrap();
        
        assert_eq!(docs[0].embedding, Some(vec![0.1, 0.2]));
        assert_eq!(docs[1].embedding, Some(vec![0.3, 0.4]));
    }
} 