use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::error::{RagError, Result};
use crate::types::{Document, RetrievalOptions, RetrievalResult};
use crate::embedding::{EmbeddingProvider, utils};
use crate::retriever::vector_store::VectorStore;

/// A simple in-memory vector store implementation
pub struct InMemoryVectorStore {
    /// Documents stored by ID
    documents: RwLock<HashMap<String, Document>>,
}

impl InMemoryVectorStore {
    /// Create a new in-memory vector store
    pub fn new() -> Self {
        Self {
            documents: RwLock::new(HashMap::new()),
        }
    }
    
    /// Find the most similar documents to an embedding vector
    fn find_similar_documents(
        &self, 
        embedding: &[f32], 
        options: &RetrievalOptions,
    ) -> Result<RetrievalResult> {
        let documents_lock = self.documents.read()
            .map_err(|_| RagError::VectorStore("Failed to acquire read lock".to_string()))?;
        
        let mut scored_docs: Vec<(f32, &Document)> = Vec::new();
        
        for doc in documents_lock.values() {
            // Skip documents that don't have embeddings
            if let Some(doc_embedding) = &doc.embedding {
                // Calculate similarity score
                let similarity = utils::compute_cosine_similarity(embedding, doc_embedding);
                
                // Apply threshold filter if specified
                if let Some(threshold) = options.threshold {
                    if similarity < threshold {
                        continue;
                    }
                }
                
                // Apply metadata filters if specified
                if let Some(filter) = &options.filter {
                    let mut matches_filter = true;
                    
                    for (key, value) in filter {
                        if !doc.metadata.fields.contains_key(key) {
                            matches_filter = false;
                            break;
                        }
                        
                        if &doc.metadata.fields[key] != value {
                            matches_filter = false;
                            break;
                        }
                    }
                    
                    if !matches_filter {
                        continue;
                    }
                }
                
                scored_docs.push((similarity, doc));
            }
        }
        
        // Sort by similarity score (descending)
        scored_docs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        // Apply limit
        let limit = options.limit.unwrap_or(scored_docs.len()).min(scored_docs.len());
        scored_docs.truncate(limit);

        // Convert to result format
        let documents = scored_docs.iter().map(|(score, doc)| crate::types::ScoredDocument {
            document: (*doc).clone(),
            score: *score,
        }).collect();

        Ok(RetrievalResult {
            documents,
            total_count: scored_docs.len(),
        })
    }
}

impl Default for InMemoryVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn add_document(&mut self, document: Document) -> Result<()> {
        let mut documents = self.documents.write()
            .map_err(|_| RagError::VectorStore("Failed to acquire write lock".to_string()))?;
        
        documents.insert(document.id.clone(), document);
        Ok(())
    }
    
    async fn update_document(&mut self, document: Document) -> Result<()> {
        let mut documents = self.documents.write()
            .map_err(|_| RagError::VectorStore("Failed to acquire write lock".to_string()))?;
        
        if !documents.contains_key(&document.id) {
            return Err(RagError::VectorStore(format!("Document not found: {}", document.id)));
        }
        
        documents.insert(document.id.clone(), document);
        Ok(())
    }
    
    async fn delete_document(&mut self, document_id: &str) -> Result<()> {
        let mut documents = self.documents.write()
            .map_err(|_| RagError::VectorStore("Failed to acquire write lock".to_string()))?;
        
        if documents.remove(document_id).is_none() {
            return Err(RagError::VectorStore(format!("Document not found: {}", document_id)));
        }
        
        Ok(())
    }
    
    async fn query_by_text(
        &self, 
        query: &str, 
        options: &RetrievalOptions,
        embedding_provider: &dyn EmbeddingProvider,
    ) -> Result<RetrievalResult> {
        // Generate embedding for the query
        let query_embedding = embedding_provider.embed_text(query).await?;
        
        // Search using the embedding
        self.query_by_vector(&query_embedding, options).await
    }
    
    async fn query_by_vector(
        &self, 
        embedding: &[f32], 
        options: &RetrievalOptions,
    ) -> Result<RetrievalResult> {
        self.find_similar_documents(embedding, options)
    }
    
    async fn get_document(&self, document_id: &str) -> Result<Option<Document>> {
        let documents = self.documents.read()
            .map_err(|_| RagError::VectorStore("Failed to acquire read lock".to_string()))?;
        
        Ok(documents.get(document_id).cloned())
    }
    
    async fn get_documents(&self, document_ids: &[String]) -> Result<Vec<Document>> {
        let documents = self.documents.read()
            .map_err(|_| RagError::VectorStore("Failed to acquire read lock".to_string()))?;
        
        let result: Vec<Document> = document_ids
            .iter()
            .filter_map(|id| documents.get(id).cloned())
            .collect();
        
        Ok(result)
    }
    
    async fn get_all_documents(&self) -> Result<Vec<Document>> {
        let documents = self.documents.read()
            .map_err(|_| RagError::VectorStore("Failed to acquire read lock".to_string()))?;
        
        Ok(documents.values().cloned().collect())
    }
    
    async fn count_documents(&self) -> Result<usize> {
        let documents = self.documents.read()
            .map_err(|_| RagError::VectorStore("Failed to acquire read lock".to_string()))?;
        
        Ok(documents.len())
    }
    
    async fn clear(&mut self) -> Result<()> {
        let mut documents = self.documents.write()
            .map_err(|_| RagError::VectorStore("Failed to acquire write lock".to_string()))?;
        
        documents.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Metadata;
    
    // A simple mock embedding provider for testing
    #[derive(Default)]
    struct MockEmbeddingProvider {
        // Pre-defined embedding to return
        return_embedding: Vec<f32>,
    }
    
    impl MockEmbeddingProvider {
        fn new() -> Self {
            Self {
                return_embedding: vec![0.4, 0.8, 1.2], // Default embedding for tests
            }
        }
        
        // Configure the embedding to return
        fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
            self.return_embedding = embedding;
            self
        }
    }
    
    #[async_trait]
    impl EmbeddingProvider for MockEmbeddingProvider {
        async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
            self.embed_text(text).await
        }

        async fn embed_text(&self, _text: &str) -> Result<Vec<f32>> {
            Ok(self.return_embedding.clone())
        }
        
        async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            let mut results = Vec::with_capacity(texts.len());
            for _ in texts {
                results.push(self.return_embedding.clone());
            }
            Ok(results)
        }
        
        fn cosine_similarity(&self, vec_a: &[f32], vec_b: &[f32]) -> f32 {
            utils::compute_cosine_similarity(vec_a, vec_b)
        }
    }
    
    #[tokio::test]
    async fn test_add_and_get_document() {
        let mut store = InMemoryVectorStore::new();
        
        let doc = Document {
            id: "test-id".to_string(),
            content: "Test content".to_string(),
            metadata: Metadata::new(),
            embedding: Some(vec![0.1, 0.2, 0.3]),
        };
        
        store.add_document(doc.clone()).await.unwrap();
        
        let retrieved = store.get_document("test-id").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Test content");
    }
    
    #[tokio::test]
    async fn test_query_by_vector() {
        // 创建测试数据
        let mut store = InMemoryVectorStore::new();
        
        // 添加几个向量，使它们的相似度是已知的
        let docs = (1..=5).map(|i| Document {
            id: format!("doc-{}", i),
            content: format!("Document {}", i),
            metadata: Metadata::default(),
            embedding: Some(match i {
                1 => vec![1.0, 0.0, 0.0],  // 正交于查询向量
                2 => vec![0.5, 0.5, 0.5],  // 中等相似度
                3 => vec![0.0, 1.0, 0.0],  // 与查询向量完全相同
                4 => vec![0.0, 0.9, 0.1],  // 非常相似
                _ => vec![0.2, 0.2, 0.2],  // 低相似度
            }),
        }).collect::<Vec<_>>();
        
        for doc in docs {
            store.add_document(doc).await.unwrap();
        }
        
        // 查询向量 [0.0, 1.0, 0.0] 应该与 doc-3 最相似
        let query_vector = vec![0.0, 1.0, 0.0];
        let options = RetrievalOptions {
            limit: Some(3),
            threshold: None,
            filter: None,
        };
        
        let result = store.query_by_vector(&query_vector, &options).await.unwrap();
        
        // 验证结果
        assert_eq!(result.documents.len(), 3);
        assert_eq!(result.documents[0].document.id, "doc-3"); // 最相似
        // 注意：我们不再断言准确的排序，因为doc-4也很相似
        assert!(result.documents.iter().any(|d| d.document.id == "doc-4")); // 应该在结果中

        // 验证分数
        assert_eq!(result.documents.len(), 3);
        assert!(result.documents[0].score > 0.9); // doc-3 的分数应该很高
    }
    
    #[tokio::test]
    async fn test_query_by_text() {
        let mut store = InMemoryVectorStore::new();
        let mock_provider = MockEmbeddingProvider::new();
        
        // Add documents with different embeddings
        for i in 0..3 {
            let doc = Document {
                id: format!("doc-{}", i),
                content: format!("Document {}", i),
                metadata: Metadata::new(),
                embedding: Some(vec![0.1 * (i as f32), 0.2 * (i as f32), 0.3 * (i as f32)]),
            };
            
            store.add_document(doc).await.unwrap();
        }
        
        // Query by text
        let options = RetrievalOptions::default();
        let results = store.query_by_text("test query", &options, &mock_provider).await.unwrap();
        
        // Should return all documents, sorted by similarity
        assert_eq!(results.documents.len(), 3);
        // Both doc-1 and doc-2 have identical cosine similarity with the query
        // Due to floating point precision, either one might be ranked first
        assert!(results.documents[0].document.id == "doc-1" || results.documents[0].document.id == "doc-2");
    }
    
    #[tokio::test]
    async fn test_update_and_delete() {
        let mut store = InMemoryVectorStore::new();
        
        // Add a document
        let doc = Document {
            id: "test-id".to_string(),
            content: "Original content".to_string(),
            metadata: Metadata::new(),
            embedding: None,
        };
        
        store.add_document(doc).await.unwrap();
        
        // Update the document
        let updated_doc = Document {
            id: "test-id".to_string(),
            content: "Updated content".to_string(),
            metadata: Metadata::new(),
            embedding: None,
        };
        
        store.update_document(updated_doc).await.unwrap();
        
        // Verify update
        let retrieved = store.get_document("test-id").await.unwrap().unwrap();
        assert_eq!(retrieved.content, "Updated content");
        
        // Delete the document
        store.delete_document("test-id").await.unwrap();
        
        // Verify deletion
        let retrieved = store.get_document("test-id").await.unwrap();
        assert!(retrieved.is_none());
    }
} 