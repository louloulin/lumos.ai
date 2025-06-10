//! Index management for LanceDB

use std::collections::HashMap;
use lancedb::index::{Index, vector::{IvfFlatIndexBuilder, IvfPqIndexBuilder}};

use crate::{
    config::{IndexType, IndexParams},
    error::{LanceDbError, LanceDbResult},
};

/// Index manager for LanceDB tables
pub struct IndexManager {
    /// Index configurations
    configs: HashMap<String, IndexConfiguration>,
}

/// Index configuration for a specific table
#[derive(Debug, Clone)]
pub struct IndexConfiguration {
    /// Index type
    pub index_type: IndexType,
    
    /// Index parameters
    pub params: IndexParams,
    
    /// Vector column name
    pub vector_column: String,
    
    /// Index metadata
    pub metadata: HashMap<String, String>,
}

impl IndexManager {
    /// Create a new index manager
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }
    
    /// Register an index configuration
    pub fn register_index(&mut self, table_name: &str, config: IndexConfiguration) {
        self.configs.insert(table_name.to_string(), config);
    }
    
    /// Get index configuration for a table
    pub fn get_config(&self, table_name: &str) -> Option<&IndexConfiguration> {
        self.configs.get(table_name)
    }
    
    /// Create a LanceDB index based on configuration
    pub fn create_index(&self, config: &IndexConfiguration) -> LanceDbResult<Index> {
        match config.index_type {
            IndexType::IVF => {
                let mut builder = IvfFlatIndexBuilder::default();
                
                if let Some(num_clusters) = config.params.num_clusters {
                    builder = builder.num_partitions(num_clusters as u32);
                }
                
                Ok(Index::IvfFlat(builder))
            }
            
            IndexType::IVFPQ => {
                let mut builder = IvfPqIndexBuilder::default();
                
                if let Some(num_clusters) = config.params.num_clusters {
                    builder = builder.num_partitions(num_clusters as u32);
                }
                
                if let Some(num_sub_quantizers) = config.params.num_sub_quantizers {
                    builder = builder.num_sub_vectors(num_sub_quantizers as u32);
                }
                
                if let Some(bits) = config.params.bits_per_sub_quantizer {
                    builder = builder.num_bits(bits as u32);
                }
                
                Ok(Index::IvfPq(builder))
            }
            
            IndexType::HNSW => {
                // HNSW might not be available in all LanceDB versions
                Err(LanceDbError::Index("HNSW index not supported in current LanceDB version".to_string()))
            }
            
            IndexType::LSH => {
                // LSH might not be available in all LanceDB versions
                Err(LanceDbError::Index("LSH index not supported in current LanceDB version".to_string()))
            }
            
            IndexType::None => {
                Err(LanceDbError::Index("Cannot create index of type None".to_string()))
            }
        }
    }
    
    /// Get recommended index type based on data characteristics
    pub fn recommend_index_type(
        &self,
        vector_count: usize,
        _vector_dimension: usize,
        query_pattern: QueryPattern,
    ) -> IndexType {
        match query_pattern {
            QueryPattern::HighThroughput => {
                if vector_count > 1_000_000 {
                    IndexType::IVFPQ // Best for large datasets with high throughput
                } else if vector_count > 100_000 {
                    IndexType::IVF // Good balance for medium datasets
                } else {
                    IndexType::None // Brute force is fine for small datasets
                }
            }
            
            QueryPattern::HighAccuracy => {
                if vector_count > 100_000 {
                    IndexType::IVF // Better accuracy than IVFPQ
                } else {
                    IndexType::None // Brute force for highest accuracy
                }
            }
            
            QueryPattern::LowLatency => {
                if vector_count > 10_000 {
                    IndexType::IVF // Fast queries with reasonable accuracy
                } else {
                    IndexType::None // Brute force is fastest for small datasets
                }
            }
            
            QueryPattern::MemoryEfficient => {
                if vector_count > 500_000 {
                    IndexType::IVFPQ // Most memory efficient for large datasets
                } else {
                    IndexType::IVF // Good memory usage for medium datasets
                }
            }
        }
    }
    
    /// Calculate optimal index parameters
    pub fn calculate_optimal_params(
        &self,
        index_type: &IndexType,
        vector_count: usize,
        vector_dimension: usize,
    ) -> IndexParams {
        match index_type {
            IndexType::IVF => {
                // Rule of thumb: sqrt(n) clusters, but with reasonable bounds
                let num_clusters = ((vector_count as f64).sqrt() as usize)
                    .max(16)
                    .min(65536);
                
                IndexParams {
                    num_clusters: Some(num_clusters),
                    num_sub_quantizers: None,
                    bits_per_sub_quantizer: None,
                    hnsw_m: None,
                    hnsw_ef_construction: None,
                    lsh_num_tables: None,
                    lsh_num_functions: None,
                }
            }
            
            IndexType::IVFPQ => {
                let num_clusters = ((vector_count as f64).sqrt() as usize)
                    .max(16)
                    .min(65536);
                
                // PQ parameters based on vector dimension
                let num_sub_quantizers = (vector_dimension / 8).max(1).min(64);
                let bits_per_sub_quantizer = if vector_dimension > 512 { 8 } else { 4 };
                
                IndexParams {
                    num_clusters: Some(num_clusters),
                    num_sub_quantizers: Some(num_sub_quantizers),
                    bits_per_sub_quantizer: Some(bits_per_sub_quantizer),
                    hnsw_m: None,
                    hnsw_ef_construction: None,
                    lsh_num_tables: None,
                    lsh_num_functions: None,
                }
            }
            
            IndexType::HNSW => {
                // HNSW parameters
                let m = if vector_dimension > 512 { 16 } else { 32 };
                let ef_construction = m * 10;
                
                IndexParams {
                    num_clusters: None,
                    num_sub_quantizers: None,
                    bits_per_sub_quantizer: None,
                    hnsw_m: Some(m),
                    hnsw_ef_construction: Some(ef_construction),
                    lsh_num_tables: None,
                    lsh_num_functions: None,
                }
            }
            
            IndexType::LSH => {
                // LSH parameters
                let num_tables = if vector_dimension > 256 { 20 } else { 10 };
                let num_functions = (vector_dimension / 32).max(1).min(10);
                
                IndexParams {
                    num_clusters: None,
                    num_sub_quantizers: None,
                    bits_per_sub_quantizer: None,
                    hnsw_m: None,
                    hnsw_ef_construction: None,
                    lsh_num_tables: Some(num_tables),
                    lsh_num_functions: Some(num_functions),
                }
            }
            
            IndexType::None => IndexParams::default(),
        }
    }
    
    /// Validate index parameters
    pub fn validate_params(&self, index_type: &IndexType, params: &IndexParams) -> LanceDbResult<()> {
        match index_type {
            IndexType::IVF => {
                if let Some(clusters) = params.num_clusters {
                    if clusters == 0 {
                        return Err(LanceDbError::InvalidConfiguration("Number of clusters must be greater than 0".to_string()));
                    }
                    if clusters > 1_000_000 {
                        return Err(LanceDbError::InvalidConfiguration("Number of clusters too large".to_string()));
                    }
                }
            }
            
            IndexType::IVFPQ => {
                if let Some(clusters) = params.num_clusters {
                    if clusters == 0 {
                        return Err(LanceDbError::InvalidConfiguration("Number of clusters must be greater than 0".to_string()));
                    }
                }
                
                if let Some(sub_quantizers) = params.num_sub_quantizers {
                    if sub_quantizers == 0 {
                        return Err(LanceDbError::InvalidConfiguration("Number of sub-quantizers must be greater than 0".to_string()));
                    }
                    if sub_quantizers > 256 {
                        return Err(LanceDbError::InvalidConfiguration("Too many sub-quantizers".to_string()));
                    }
                }
                
                if let Some(bits) = params.bits_per_sub_quantizer {
                    if bits == 0 || bits > 16 {
                        return Err(LanceDbError::InvalidConfiguration("Bits per sub-quantizer must be between 1 and 16".to_string()));
                    }
                }
            }
            
            IndexType::HNSW => {
                if let Some(m) = params.hnsw_m {
                    if m == 0 || m > 128 {
                        return Err(LanceDbError::InvalidConfiguration("HNSW M parameter must be between 1 and 128".to_string()));
                    }
                }
                
                if let Some(ef) = params.hnsw_ef_construction {
                    if ef == 0 {
                        return Err(LanceDbError::InvalidConfiguration("HNSW ef_construction must be greater than 0".to_string()));
                    }
                }
            }
            
            IndexType::LSH => {
                if let Some(tables) = params.lsh_num_tables {
                    if tables == 0 || tables > 100 {
                        return Err(LanceDbError::InvalidConfiguration("LSH number of tables must be between 1 and 100".to_string()));
                    }
                }
                
                if let Some(functions) = params.lsh_num_functions {
                    if functions == 0 || functions > 50 {
                        return Err(LanceDbError::InvalidConfiguration("LSH number of functions must be between 1 and 50".to_string()));
                    }
                }
            }
            
            IndexType::None => {
                // No validation needed for no index
            }
        }
        
        Ok(())
    }
}

/// Query pattern for index recommendation
#[derive(Debug, Clone, Copy)]
pub enum QueryPattern {
    /// High throughput queries (many queries per second)
    HighThroughput,
    
    /// High accuracy requirements (precision/recall)
    HighAccuracy,
    
    /// Low latency requirements (fast response time)
    LowLatency,
    
    /// Memory efficient (low memory usage)
    MemoryEfficient,
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexConfiguration {
    /// Create a new index configuration
    pub fn new(index_type: IndexType, vector_column: &str) -> Self {
        Self {
            index_type,
            params: IndexParams::default(),
            vector_column: vector_column.to_string(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set index parameters
    pub fn with_params(mut self, params: IndexParams) -> Self {
        self.params = params;
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_index_manager() {
        let mut manager = IndexManager::new();
        
        let config = IndexConfiguration::new(IndexType::IVF, "vector");
        manager.register_index("test_table", config);
        
        assert!(manager.get_config("test_table").is_some());
        assert!(manager.get_config("nonexistent").is_none());
    }
    
    #[test]
    fn test_index_recommendation() {
        let manager = IndexManager::new();
        
        // Small dataset should use no index
        let index_type = manager.recommend_index_type(1000, 384, QueryPattern::HighAccuracy);
        assert!(matches!(index_type, IndexType::None));
        
        // Large dataset should use IVFPQ for high throughput
        let index_type = manager.recommend_index_type(2_000_000, 384, QueryPattern::HighThroughput);
        assert!(matches!(index_type, IndexType::IVFPQ));
    }
    
    #[test]
    fn test_optimal_params() {
        let manager = IndexManager::new();
        
        let params = manager.calculate_optimal_params(&IndexType::IVF, 100_000, 384);
        assert!(params.num_clusters.is_some());
        assert!(params.num_clusters.unwrap() > 0);
        
        let params = manager.calculate_optimal_params(&IndexType::IVFPQ, 1_000_000, 768);
        assert!(params.num_clusters.is_some());
        assert!(params.num_sub_quantizers.is_some());
        assert!(params.bits_per_sub_quantizer.is_some());
    }
    
    #[test]
    fn test_params_validation() {
        let manager = IndexManager::new();
        
        let valid_params = IndexParams {
            num_clusters: Some(256),
            ..Default::default()
        };
        assert!(manager.validate_params(&IndexType::IVF, &valid_params).is_ok());
        
        let invalid_params = IndexParams {
            num_clusters: Some(0),
            ..Default::default()
        };
        assert!(manager.validate_params(&IndexType::IVF, &invalid_params).is_err());
    }
}
