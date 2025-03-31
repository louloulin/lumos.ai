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
    use crate::types::Metadata;

    // 简单测试trait默认方法的行为
    #[tokio::test]
    async fn test_default_add_documents_implementation() {
        // 创建一个简单的结构体实现VectorStore
        struct TestVectorStore {
            docs: Vec<Document>,
        }
        
        #[async_trait]
        impl VectorStore for TestVectorStore {
            async fn add_document(&mut self, document: Document) -> Result<()> {
                self.docs.push(document);
                Ok(())
            }
            
            async fn update_document(&mut self, _document: Document) -> Result<()> {
                unimplemented!()
            }
            
            async fn delete_document(&mut self, _document_id: &str) -> Result<()> {
                unimplemented!()
            }
            
            async fn query_by_text(
                &self,
                _query: &str,
                _options: &RetrievalOptions,
                _embedding_provider: &dyn EmbeddingProvider,
            ) -> Result<RetrievalResult> {
                unimplemented!()
            }
            
            async fn query_by_vector(
                &self,
                _embedding: &[f32],
                _options: &RetrievalOptions,
            ) -> Result<RetrievalResult> {
                unimplemented!()
            }
            
            async fn get_document(&self, _document_id: &str) -> Result<Option<Document>> {
                unimplemented!()
            }
            
            async fn get_documents(&self, _document_ids: &[String]) -> Result<Vec<Document>> {
                unimplemented!()
            }
            
            async fn get_all_documents(&self) -> Result<Vec<Document>> {
                unimplemented!()
            }
            
            async fn count_documents(&self) -> Result<usize> {
                Ok(self.docs.len())
            }
            
            async fn clear(&mut self) -> Result<()> {
                self.docs.clear();
                Ok(())
            }
        }
        
        // 初始化测试结构
        let mut store = TestVectorStore { docs: Vec::new() };
        
        // 准备测试文档
        let docs = vec![
            Document {
                id: "doc1".to_string(),
                content: "Test document 1".to_string(),
                metadata: Metadata::default(),
                embedding: None,
            },
            Document {
                id: "doc2".to_string(),
                content: "Test document 2".to_string(),
                metadata: Metadata::default(),
                embedding: None,
            },
        ];
        
        // 测试add_documents默认实现
        let result = VectorStore::add_documents(&mut store, docs).await;
        assert!(result.is_ok());
        
        // 验证文档是否被正确添加
        let count = store.count_documents().await.unwrap();
        assert_eq!(count, 2);
    }
} 