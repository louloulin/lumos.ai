//! Memory index implementation

use std::collections::HashMap;
use chrono::{DateTime, Utc};

use lumosai_vector_core::prelude::*;
use lumosai_vector_core::traits::{similarity, filter::StandardFilterEvaluator};
use crate::MemoryConfig;

/// In-memory vector index
pub struct MemoryIndex {
    /// Index configuration
    config: IndexConfig,
    /// Index creation time
    created_at: DateTime<Utc>,
    /// Index last updated time
    updated_at: DateTime<Utc>,
    /// Documents stored in the index
    documents: HashMap<DocumentId, Document>,
    /// Similarity calculator
    similarity_calculator: Box<dyn SimilarityCalculator>,
    /// Filter evaluator
    filter_evaluator: StandardFilterEvaluator,
    /// Memory usage tracking
    memory_usage_bytes: u64,
}

impl MemoryIndex {
    /// Create a new memory index
    pub fn new(config: IndexConfig, _memory_config: &MemoryConfig) -> Result<Self> {
        let similarity_calculator = similarity::create_calculator(config.metric);
        
        Ok(Self {
            config,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            documents: HashMap::new(),
            similarity_calculator,
            filter_evaluator: StandardFilterEvaluator,
            memory_usage_bytes: 0,
        })
    }
    
    /// Get index information
    pub fn get_info(&self) -> IndexInfo {
        IndexInfo {
            name: self.config.name.clone(),
            dimension: self.config.dimension,
            metric: self.config.metric,
            vector_count: self.documents.len(),
            size_bytes: self.memory_usage_bytes,
            created_at: Some(self.created_at),
            updated_at: Some(self.updated_at),
            metadata: HashMap::new(),
        }
    }
    
    /// Get the dimension of vectors in this index
    pub fn dimension(&self) -> usize {
        self.config.dimension
    }
    
    /// Get the number of vectors in this index
    pub fn vector_count(&self) -> usize {
        self.documents.len()
    }
    
    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> u64 {
        self.memory_usage_bytes
    }
    
    /// Estimate memory usage for a document
    pub fn estimate_document_memory(&self, document: &Document) -> u64 {
        let mut size = 0u64;
        
        // Document ID
        size += document.id.len() as u64;
        
        // Content
        size += document.content.len() as u64;
        
        // Embedding (if present)
        if let Some(embedding) = &document.embedding {
            size += embedding.len() as u64 * 4; // f32 = 4 bytes
        }
        
        // Metadata (rough estimate)
        for (key, value) in &document.metadata {
            size += key.len() as u64;
            size += self.estimate_metadata_value_size(value);
        }
        
        size
    }
    
    fn estimate_metadata_value_size(&self, value: &MetadataValue) -> u64 {
        match value {
            MetadataValue::String(s) => s.len() as u64,
            MetadataValue::Integer(_) => 8,
            MetadataValue::Float(_) => 8,
            MetadataValue::Boolean(_) => 1,
            MetadataValue::Array(arr) => {
                arr.iter().map(|v| self.estimate_metadata_value_size(v)).sum()
            },
            MetadataValue::Object(obj) => {
                obj.iter().map(|(k, v)| k.len() as u64 + self.estimate_metadata_value_size(v)).sum()
            },
            MetadataValue::Null => 0,
        }
    }
    
    /// Insert or update a document
    pub fn upsert_document(&mut self, document: Document) -> Result<bool> {
        let was_new = !self.documents.contains_key(&document.id);
        
        if was_new {
            self.memory_usage_bytes += self.estimate_document_memory(&document);
        } else {
            // For updates, we'll just recalculate (could be optimized)
            if let Some(old_doc) = self.documents.get(&document.id) {
                self.memory_usage_bytes -= self.estimate_document_memory(old_doc);
            }
            self.memory_usage_bytes += self.estimate_document_memory(&document);
        }
        
        self.documents.insert(document.id.clone(), document);
        self.updated_at = Utc::now();
        
        Ok(was_new)
    }
    
    /// Update an existing document
    pub fn update_document(&mut self, document: Document) -> Result<()> {
        if !self.documents.contains_key(&document.id) {
            return Err(VectorError::vector_not_found(&document.id));
        }
        
        // Update memory usage
        if let Some(old_doc) = self.documents.get(&document.id) {
            self.memory_usage_bytes -= self.estimate_document_memory(old_doc);
        }
        self.memory_usage_bytes += self.estimate_document_memory(&document);
        
        self.documents.insert(document.id.clone(), document);
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// Delete a document
    pub fn delete_document(&mut self, id: &DocumentId) -> Result<Option<Document>> {
        if let Some(document) = self.documents.remove(id) {
            self.memory_usage_bytes -= self.estimate_document_memory(&document);
            self.updated_at = Utc::now();
            Ok(Some(document))
        } else {
            Ok(None)
        }
    }
    
    /// Get a document by ID
    pub fn get_document(&self, id: &DocumentId) -> Result<Option<Document>> {
        Ok(self.documents.get(id).cloned())
    }
    
    /// Search for similar documents
    pub fn search(&self, request: &SearchRequest) -> Result<Vec<SearchResult>> {
        let query_vector = match &request.query {
            SearchQuery::Vector(vector) => {
                if vector.len() != self.config.dimension {
                    return Err(VectorError::dimension_mismatch(self.config.dimension, vector.len()));
                }
                vector.clone()
            },
            SearchQuery::Text(_) => {
                return Err(VectorError::NotSupported(
                    "Text queries require an embedding model".to_string()
                ));
            },
        };
        
        let mut results = Vec::new();
        
        for (id, document) in &self.documents {
            // Apply filter if provided
            if let Some(filter) = &request.filter {
                if !self.filter_evaluator.evaluate(filter, &document.metadata)? {
                    continue;
                }
            }
            
            // Calculate similarity
            if let Some(embedding) = &document.embedding {
                let score = self.similarity_calculator.calculate_similarity(&query_vector, embedding)?;
                
                let mut result = SearchResult::new(id.clone(), score);
                
                if request.include_vectors {
                    result = result.with_vector(embedding.clone());
                }
                
                if request.include_metadata {
                    result = result.with_metadata(document.metadata.clone());
                }
                
                // Include content if available
                result = result.with_content(document.content.clone());
                
                results.push(result);
            }
        }
        
        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit to top_k
        results.truncate(request.top_k);
        
        Ok(results)
    }
}
