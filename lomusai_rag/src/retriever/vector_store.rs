use async_trait::async_trait;

use crate::error::Result;
use crate::types::{Document, RetrievalOptions, RetrievalResult};
use crate::embedding::EmbeddingProvider;

/// Interface for vector stores that store and retrieve documents based on embeddings
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Add a document to the store
    async fn add_document(&mut self, document: Document) -> Result<()>;
    
    /// Add multiple documents to the store
    async fn add_documents(&mut self, documents: Vec<Document>) -> Result<()> {
        for doc in documents {
            self.add_document(doc).await?;
        }
        Ok(())
    }
    
    /// Update a document in the store
    async fn update_document(&mut self, document: Document) -> Result<()>;
    
    /// Delete a document from the store
    async fn delete_document(&mut self, document_id: &str) -> Result<()>;
    
    /// Retrieve documents by content similarity to the query
    async fn query_by_text(
        &self, 
        query: &str, 
        options: &RetrievalOptions,
        embedding_provider: &dyn EmbeddingProvider,
    ) -> Result<RetrievalResult>;
    
    /// Retrieve documents by similarity to the provided embedding vector
    async fn query_by_vector(
        &self, 
        embedding: &[f32], 
        options: &RetrievalOptions,
    ) -> Result<RetrievalResult>;
    
    /// Get a document by ID
    async fn get_document(&self, document_id: &str) -> Result<Option<Document>>;
    
    /// Get documents by IDs
    async fn get_documents(&self, document_ids: &[String]) -> Result<Vec<Document>>;
    
    /// Get all documents in the store
    async fn get_all_documents(&self) -> Result<Vec<Document>>;
    
    /// Count the number of documents in the store
    async fn count_documents(&self) -> Result<usize>;
    
    /// Clear all documents from the store
    async fn clear(&mut self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    use crate::types::Metadata;
    
    mock! {
        pub VectorStore {
            fn add_document(&mut self, document: Document) -> Result<()>;
            fn update_document(&mut self, document: Document) -> Result<()>;
            fn delete_document(&mut self, document_id: &str) -> Result<()>;
            fn query_by_text(
                &self, 
                query: &str, 
                options: &RetrievalOptions,
                embedding_provider: &dyn EmbeddingProvider,
            ) -> Result<RetrievalResult>;
            fn query_by_vector(
                &self, 
                embedding: &[f32], 
                options: &RetrievalOptions,
            ) -> Result<RetrievalResult>;
            fn get_document(&self, document_id: &str) -> Result<Option<Document>>;
            fn get_documents(&self, document_ids: &[String]) -> Result<Vec<Document>>;
            fn get_all_documents(&self) -> Result<Vec<Document>>;
            fn count_documents(&self) -> Result<usize>;
            fn clear(&mut self) -> Result<()>;
        }
        
        #[async_trait]
        impl VectorStore for VectorStore {
            async fn add_document(&mut self, document: Document) -> Result<()>;
            async fn update_document(&mut self, document: Document) -> Result<()>;
            async fn delete_document(&mut self, document_id: &str) -> Result<()>;
            async fn query_by_text(
                &self, 
                query: &str, 
                options: &RetrievalOptions,
                embedding_provider: &dyn EmbeddingProvider,
            ) -> Result<RetrievalResult>;
            async fn query_by_vector(
                &self, 
                embedding: &[f32], 
                options: &RetrievalOptions,
            ) -> Result<RetrievalResult>;
            async fn get_document(&self, document_id: &str) -> Result<Option<Document>>;
            async fn get_documents(&self, document_ids: &[String]) -> Result<Vec<Document>>;
            async fn get_all_documents(&self) -> Result<Vec<Document>>;
            async fn count_documents(&self) -> Result<usize>;
            async fn clear(&mut self) -> Result<()>;
        }
    }
} 