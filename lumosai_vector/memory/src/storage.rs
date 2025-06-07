//! Memory vector storage implementation

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use async_trait::async_trait;

use lumosai_vector_core::prelude::*;
use crate::{MemoryConfig, MemoryIndex};

/// High-performance in-memory vector storage
pub struct MemoryVectorStorage {
    /// Storage configuration
    config: MemoryConfig,
    /// Indexes stored in memory
    indexes: Arc<RwLock<HashMap<String, MemoryIndex>>>,
    /// Storage statistics
    stats: Arc<RwLock<StorageStats>>,
    /// Performance monitor
    performance_monitor: Arc<PerformanceMonitor>,
    /// Search result cache
    search_cache: Arc<LRUCache<String, SearchResponse>>,
}

/// Storage statistics
#[derive(Debug, Default)]
pub struct StorageStats {
    /// Total number of indexes
    index_count: usize,
    /// Total number of vectors across all indexes
    total_vectors: usize,
    /// Total memory usage in bytes
    memory_usage_bytes: u64,
    /// Number of search operations
    search_count: u64,
    /// Total search time in milliseconds
    total_search_time_ms: u64,
}

impl MemoryVectorStorage {
    /// Create a new memory vector storage with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(MemoryConfig::default()).await
    }
    
    /// Create a new memory vector storage with specified capacity
    pub async fn with_capacity(capacity: usize) -> Result<Self> {
        let config = MemoryConfig::new().with_initial_capacity(capacity);
        Self::with_config(config).await
    }
    
    /// Create a new memory vector storage with custom configuration
    pub async fn with_config(config: MemoryConfig) -> Result<Self> {
        let cache_config = CacheConfig {
            max_entries: 100,
            ttl: std::time::Duration::from_secs(300), // 5 minutes
            enable_lru: true,
            stats_interval: std::time::Duration::from_secs(60),
        };

        Ok(Self {
            config,
            indexes: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(StorageStats::default())),
            performance_monitor: Arc::new(PerformanceMonitor::new()),
            search_cache: Arc::new(LRUCache::new(cache_config)),
        })
    }
    
    /// Get storage statistics
    pub async fn get_stats(&self) -> StorageStats {
        self.stats.read().await.clone()
    }
    
    /// Get memory usage in bytes
    pub async fn memory_usage(&self) -> u64 {
        self.stats.read().await.memory_usage_bytes
    }
    
    /// Cleanup unused memory (if configured)
    pub async fn cleanup(&self) -> Result<()> {
        if let Some(threshold_mb) = self.config.memory_threshold_mb {
            let current_usage_mb = self.memory_usage().await / (1024 * 1024);
            if current_usage_mb > threshold_mb as u64 {
                // Clear search cache to free memory
                self.search_cache.clear().await;

                // Trigger garbage collection or other cleanup
                // For now, this is a no-op, but could be extended
            }
        }
        Ok(())
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> lumosai_vector_core::PerformanceMetrics {
        self.performance_monitor.get_metrics().await
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> lumosai_vector_core::CacheStats {
        self.search_cache.get_stats().await
    }
}

#[async_trait]
impl VectorStorage for MemoryVectorStorage {
    type Config = MemoryConfig;
    
    async fn create_index(&self, config: IndexConfig) -> Result<()> {
        let mut indexes = self.indexes.write().await;
        
        if indexes.contains_key(&config.name) {
            return Err(VectorError::index_already_exists(&config.name));
        }
        
        let index = MemoryIndex::new(config.clone(), &self.config)?;
        indexes.insert(config.name.clone(), index);
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.index_count += 1;
        
        Ok(())
    }
    
    async fn list_indexes(&self) -> Result<Vec<String>> {
        let indexes = self.indexes.read().await;
        Ok(indexes.keys().cloned().collect())
    }
    
    async fn describe_index(&self, index_name: &str) -> Result<IndexInfo> {
        let indexes = self.indexes.read().await;
        let index = indexes.get(index_name)
            .ok_or_else(|| VectorError::index_not_found(index_name))?;
        
        Ok(index.get_info())
    }
    
    async fn delete_index(&self, index_name: &str) -> Result<()> {
        let mut indexes = self.indexes.write().await;
        
        if !indexes.contains_key(index_name) {
            return Err(VectorError::index_not_found(index_name));
        }
        
        let removed_index = indexes.remove(index_name).unwrap();
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.index_count -= 1;
        stats.total_vectors -= removed_index.vector_count();
        stats.memory_usage_bytes -= removed_index.memory_usage();
        
        Ok(())
    }
    
    async fn upsert_documents(&self, index_name: &str, documents: Vec<Document>) -> Result<Vec<DocumentId>> {
        let mut indexes = self.indexes.write().await;
        let index = indexes.get_mut(index_name)
            .ok_or_else(|| VectorError::index_not_found(index_name))?;
        
        let mut document_ids = Vec::new();
        let mut vectors_added = 0;
        let mut memory_added = 0;
        
        for document in documents {
            let embedding = document.embedding.as_ref()
                .ok_or_else(|| VectorError::InvalidVector("Document must have embedding".to_string()))?;
            
            if embedding.len() != index.dimension() {
                return Err(VectorError::dimension_mismatch(index.dimension(), embedding.len()));
            }
            
            // Check capacity limits
            if let Some(max_vectors) = self.config.max_vectors_per_index {
                if index.vector_count() >= max_vectors {
                    return Err(VectorError::ResourceLimitExceeded(
                        format!("Index {} has reached maximum capacity of {} vectors", index_name, max_vectors)
                    ));
                }
            }
            
            let was_new = index.upsert_document(document.clone())?;
            document_ids.push(document.id.clone());
            
            if was_new {
                vectors_added += 1;
                memory_added += index.estimate_document_memory(&document);
            }
        }
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_vectors += vectors_added;
        stats.memory_usage_bytes += memory_added;
        
        Ok(document_ids)
    }
    
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        let start_time = Instant::now();

        // Generate cache key for the search request
        let cache_key = format!("{}_{}_{}",
            request.index_name,
            request.top_k,
            serde_json::to_string(&request.query).unwrap_or_default()
        );

        // Check cache first
        if let Some(cached_response) = self.search_cache.get(&cache_key).await {
            let duration = start_time.elapsed();
            self.performance_monitor.record_operation(duration, true).await;
            return Ok(cached_response);
        }

        let indexes = self.indexes.read().await;
        let index = indexes.get(&request.index_name)
            .ok_or_else(|| VectorError::index_not_found(&request.index_name))?;

        let results = index.search(&request)?;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        let duration = start_time.elapsed();

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.search_count += 1;
            stats.total_search_time_ms += execution_time_ms;
        }

        let response = SearchResponse::new(results)
            .with_execution_time(execution_time_ms);

        // Cache the response
        self.search_cache.set(cache_key, response.clone()).await;

        // Record performance metrics
        self.performance_monitor.record_operation(duration, true).await;

        Ok(response)
    }
    
    async fn update_document(&self, index_name: &str, document: Document) -> Result<()> {
        let mut indexes = self.indexes.write().await;
        let index = indexes.get_mut(index_name)
            .ok_or_else(|| VectorError::index_not_found(index_name))?;
        
        if let Some(embedding) = &document.embedding {
            if embedding.len() != index.dimension() {
                return Err(VectorError::dimension_mismatch(index.dimension(), embedding.len()));
            }
        }
        
        index.update_document(document)?;
        Ok(())
    }
    
    async fn delete_documents(&self, index_name: &str, ids: Vec<DocumentId>) -> Result<()> {
        let mut indexes = self.indexes.write().await;
        let index = indexes.get_mut(index_name)
            .ok_or_else(|| VectorError::index_not_found(index_name))?;
        
        let mut vectors_removed = 0;
        let mut memory_freed = 0;
        
        for id in ids {
            if let Some(removed_doc) = index.delete_document(&id)? {
                vectors_removed += 1;
                memory_freed += index.estimate_document_memory(&removed_doc);
            }
        }
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_vectors -= vectors_removed;
        stats.memory_usage_bytes = stats.memory_usage_bytes.saturating_sub(memory_freed);
        
        Ok(())
    }
    
    async fn get_documents(&self, index_name: &str, ids: Vec<DocumentId>, include_vectors: bool) -> Result<Vec<Document>> {
        let indexes = self.indexes.read().await;
        let index = indexes.get(index_name)
            .ok_or_else(|| VectorError::index_not_found(index_name))?;
        
        let mut documents = Vec::new();
        for id in ids {
            if let Some(mut document) = index.get_document(&id)? {
                if !include_vectors {
                    document.embedding = None;
                }
                documents.push(document);
            }
        }
        
        Ok(documents)
    }
    
    async fn health_check(&self) -> Result<()> {
        // Check if we can acquire locks
        let _indexes = self.indexes.read().await;
        let _stats = self.stats.read().await;
        
        // Check memory usage if configured
        if let Some(threshold_mb) = self.config.memory_threshold_mb {
            let current_usage_mb = self.memory_usage().await / (1024 * 1024);
            if current_usage_mb > threshold_mb as u64 * 2 {
                return Err(VectorError::ResourceLimitExceeded(
                    format!("Memory usage {} MB exceeds critical threshold", current_usage_mb)
                ));
            }
        }
        
        Ok(())
    }
    
    fn backend_info(&self) -> BackendInfo {
        BackendInfo::new("memory", env!("CARGO_PKG_VERSION"))
            .with_feature("high_performance")
            .with_feature("thread_safe")
            .with_feature("complex_filtering")
            .with_feature("multiple_metrics")
            .with_metadata("initial_capacity", MetadataValue::Integer(self.config.initial_capacity as i64))
            .with_metadata("approximate_search", MetadataValue::Boolean(self.config.enable_approximate))
    }
}

impl Clone for MemoryVectorStorage {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            indexes: Arc::clone(&self.indexes),
            stats: Arc::clone(&self.stats),
            performance_monitor: Arc::clone(&self.performance_monitor),
            search_cache: Arc::clone(&self.search_cache),
        }
    }
}

impl Clone for StorageStats {
    fn clone(&self) -> Self {
        Self {
            index_count: self.index_count,
            total_vectors: self.total_vectors,
            memory_usage_bytes: self.memory_usage_bytes,
            search_count: self.search_count,
            total_search_time_ms: self.total_search_time_ms,
        }
    }
}
