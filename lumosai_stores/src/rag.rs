//! Integration with lumosai_rag for vector store retrieval
//!
//! This module provides implementations of the VectorStore trait from lumosai_rag
//! using the vector stores defined in this crate.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use lumosai_rag::error::{RagError, Result};
use lumosai_rag::embedding::EmbeddingProvider;
use lumosai_rag::types::{Document, RetrievalOptions, RetrievalResult, Metadata};
use lumosai_rag::retriever::VectorStore as RagVectorStore;

use crate::vector::{VectorStore, CreateIndexParams, UpsertParams, QueryParams, QueryResult, VectorFilter};
use crate::error::StoreError;

/// A vector store retriever that implements the lumosai_rag VectorStore trait
/// using any lumosai_stores vector store implementation
pub struct VectorStoreRetriever<T>
where
    T: VectorStore,
{
    /// The underlying vector store
    store: Arc<Mutex<T>>,
    /// The name of the index to use
    index_name: String,
    /// The dimension of the vectors
    dimensions: usize,
    /// The distance metric
    metric: String,
}

impl<T> VectorStoreRetriever<T>
where
    T: VectorStore,
{
    /// Create a new vector store retriever
    pub fn new(store: T, index_name: impl Into<String>, dimensions: usize, metric: impl Into<String>) -> Self {
        Self {
            store: Arc::new(Mutex::new(store)),
            index_name: index_name.into(),
            dimensions,
            metric: metric.into(),
        }
    }
    
    /// Ensure the index exists, creating it if necessary
    pub async fn ensure_index(&self) -> Result<()> {
        let store = self.store.lock().await;
        
        // Check if the index exists
        match store.describe_index(&self.index_name).await {
            Ok(_) => Ok(()), // Index already exists
            Err(StoreError::IndexError(_)) | Err(StoreError::QueryError(_)) => {
                // Create the index
                let params = CreateIndexParams {
                    index_name: self.index_name.clone(),
                    dimension: self.dimensions,
                    metric: self.metric.clone(),
                };
                
                store.create_index(params).await
                    .map_err(|e| RagError::VectorStore(format!("Failed to create index: {}", e)))
            },
            Err(e) => Err(RagError::VectorStore(format!("Failed to check index: {}", e))),
        }
    }
    
    /// Convert a RAG Document to vector metadata
    fn document_to_metadata(document: &Document) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();
        
        // Add document ID
        metadata.insert("id".to_string(), document.id.clone().into());
        
        // Add document content
        metadata.insert("content".to_string(), document.content.clone().into());
        
        // Add document metadata
        if let Some(source) = &document.metadata.source {
            metadata.insert("source".to_string(), source.clone().into());
        }
        
        if let Some(created_at) = &document.metadata.created_at {
            metadata.insert("created_at".to_string(), created_at.to_string().into());
        }
        
        // Add all custom fields
        for (key, value) in &document.metadata.fields {
            metadata.insert(key.clone(), serde_json::to_value(value).unwrap_or_default());
        }
        
        metadata
    }
    
    /// Convert vector store metadata to a RAG Document
    fn metadata_to_document(id: String, metadata: HashMap<String, serde_json::Value>, embedding: Option<Vec<f32>>) -> Document {
        let mut doc_metadata = Metadata::new();
        
        // Extract content
        let content = metadata.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        // Extract source
        if let Some(source) = metadata.get("source").and_then(|v| v.as_str()) {
            doc_metadata = doc_metadata.with_source(source);
        }
        
        // Extract created_at
        if let Some(created_at) = metadata.get("created_at").and_then(|v| v.as_str()) {
            if let Ok(dt) = created_at.parse::<chrono::DateTime<chrono::Utc>>() {
                doc_metadata.created_at = Some(dt);
            }
        }
        
        // Add all other fields as custom metadata
        for (key, value) in metadata {
            if !["id", "content", "source", "created_at"].contains(&key.as_str()) {
                doc_metadata.add(key, value);
            }
        }
        
        Document {
            id,
            content,
            metadata: doc_metadata,
            embedding,
        }
    }
    
    /// Convert a RAG filter to a vector store filter
    fn convert_filter(filter: &HashMap<String, serde_json::Value>) -> Option<crate::vector::VectorFilter> {
        if filter.is_empty() {
            return None;
        }
        
        // 将每个 serde_json::Value 转换为 FieldCondition::Value
        let mut field_conditions = HashMap::new();
        for (key, value) in filter {
            field_conditions.insert(key.clone(), crate::vector::FieldCondition::Value(value.clone()));
        }
        
        Some(crate::vector::VectorFilter::Field(field_conditions))
    }
    
    /// Convert RetrievalOptions to vector store filter
    fn options_to_filter(options: &RetrievalOptions) -> Option<crate::vector::VectorFilter> {
        options.filter.as_ref().and_then(|filter| Self::convert_filter(filter))
    }
}

#[async_trait]
impl<T> RagVectorStore for VectorStoreRetriever<T>
where
    T: VectorStore + 'static,
{
    async fn add_document(&mut self, document: Document) -> Result<()> {
        // Ensure index exists
        self.ensure_index().await?;
        
        let mut store = self.store.lock().await;
        
        // Extract the document content and metadata
        let metadata = Self::document_to_metadata(&document);
        
        // Get the embedding or error
        let embedding = document.embedding.as_ref()
            .ok_or_else(|| RagError::VectorStore("Document must have embedding".to_string()))?;
        
        // Create parameters for upsert
        let params = UpsertParams {
            index_name: self.index_name.clone(),
            vectors: vec![embedding.clone()],
            metadata: vec![metadata],
            ids: Some(vec![document.id.clone()]),
        };
        
        // Upsert the document
        store.upsert(params)
            .await
            .map_err(|e| RagError::VectorStore(format!("Failed to add document: {}", e)))?;
        
        Ok(())
    }
    
    async fn update_document(&mut self, document: Document) -> Result<()> {
        // Use the same implementation as add_document for upsert behavior
        self.add_document(document).await
    }
    
    async fn delete_document(&mut self, document_id: &str) -> Result<()> {
        let mut store = self.store.lock().await;
        
        store.delete_vectors(&self.index_name, &[document_id.to_string()])
            .await
            .map_err(|e| RagError::VectorStore(format!("Failed to delete document: {}", e)))?;
            
        Ok(())
    }
    
    async fn query_by_text(
        &self,
        query: &str,
        options: &RetrievalOptions,
        embedding_provider: &dyn EmbeddingProvider,
    ) -> Result<RetrievalResult> {
        // Generate embedding for the query
        let embedding = embedding_provider.embed_text(query).await?;
        
        // Query by vector
        self.query_by_vector(&embedding, options).await
    }
    
    async fn query_by_vector(
        &self,
        embedding: &[f32],
        options: &RetrievalOptions,
    ) -> Result<RetrievalResult> {
        let store = self.store.lock().await;
        
        // Convert options to filter
        let filter = Self::options_to_filter(options);
        
        // Create query parameters
        let params = QueryParams {
            index_name: self.index_name.clone(),
            query_vector: embedding.to_vec(),
            top_k: options.limit,
            filter,
            include_vector: true,
        };
        
        // Query the vector store
        let results = store.query(params)
            .await
            .map_err(|e| RagError::VectorStore(format!("Failed to query: {}", e)))?;
        
        // Convert results to Documents
        let mut documents = Vec::with_capacity(results.len());
        let mut scores = Vec::with_capacity(results.len());
        
        for result in results {
            documents.push(Self::metadata_to_document(
                result.id,
                result.metadata,
                result.vector,
            ));
            scores.push(result.score);
        }
        
        Ok(RetrievalResult {
            documents,
            scores: Some(scores),
        })
    }
    
    async fn get_document(&self, document_id: &str) -> Result<Option<Document>> {
        // The vector store trait doesn't have a direct get_by_id method,
        // so we'll implement a simple workaround using a query with a filter
        
        let store = self.store.lock().await;
        
        // Create a metadata filter for the document ID
        let mut filter_map = HashMap::new();
        filter_map.insert("id".to_string(), document_id.to_string().into());
        
        let filter = Self::convert_filter(&filter_map);
        
        // Create query parameters
        let params = QueryParams {
            index_name: self.index_name.clone(),
            // We need a query vector, but it won't matter because we're filtering by ID
            // Using a zero vector is safe here
            query_vector: vec![0.0; self.dimensions],
            top_k: 1,
            filter,
            include_vector: true,
        };
        
        // Execute the query
        match store.query(params).await {
            Ok(results) if !results.is_empty() => {
                let result = &results[0];
                Ok(Some(Self::metadata_to_document(
                    result.id.clone(),
                    result.metadata.clone(),
                    result.vector.clone(),
                )))
            },
            Ok(_) => Ok(None),
            Err(e) => Err(RagError::VectorStore(format!("Failed to get document: {}", e))),
        }
    }
    
    async fn get_documents(&self, document_ids: &[String]) -> Result<Vec<Document>> {
        // Since we don't have a direct get_by_ids method, we'll need to query each ID individually
        let mut documents = Vec::with_capacity(document_ids.len());
        
        for id in document_ids {
            if let Some(doc) = self.get_document(id).await? {
                documents.push(doc);
            }
        }
        
        Ok(documents)
    }
    
    async fn get_all_documents(&self) -> Result<Vec<Document>> {
        // This implementation is not efficient for large collections
        // We'll query with a large limit and no filter to get as many documents as possible
        
        let store = self.store.lock().await;
        
        // Create query parameters with large limit and no filter
        let params = QueryParams {
            index_name: self.index_name.clone(),
            query_vector: vec![0.0; self.dimensions], // Zero vector
            top_k: 10000, // Arbitrary large number
            filter: None,
            include_vector: true,
        };
        
        // Execute the query
        match store.query(params).await {
            Ok(results) => {
                let documents = results.into_iter()
                    .map(|result| Self::metadata_to_document(
                        result.id,
                        result.metadata,
                        result.vector,
                    ))
                    .collect();
                    
                Ok(documents)
            },
            Err(e) => Err(RagError::VectorStore(format!("Failed to get all documents: {}", e))),
        }
    }
    
    async fn count_documents(&self) -> Result<usize> {
        // Use describe_index to get stats if available
        let store = self.store.lock().await;
        
        match store.describe_index(&self.index_name).await {
            Ok(stats) => Ok(stats.count),
            Err(e) => Err(RagError::VectorStore(format!("Failed to count documents: {}", e))),
        }
    }
    
    async fn clear(&mut self) -> Result<()> {
        let mut store = self.store.lock().await;
        
        // Delete the index and recreate it
        match store.delete_index(&self.index_name).await {
            Ok(_) => {
                // Recreate with same parameters
                let params = CreateIndexParams {
                    index_name: self.index_name.clone(),
                    dimension: self.dimensions,
                    metric: self.metric.clone(),
                };
                
                store.create_index(params)
                    .await
                    .map_err(|e| RagError::VectorStore(format!("Failed to recreate index: {}", e)))
            },
            Err(e) => Err(RagError::VectorStore(format!("Failed to clear documents: {}", e))),
        }
    }
} 