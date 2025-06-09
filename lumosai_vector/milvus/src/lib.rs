//! # LumosAI Milvus Integration
//!
//! This crate provides Milvus integration for LumosAI vector storage,
//! offering high-performance vector database capabilities with cloud-native features.
//!
//! ## Features
//!
//! - **High Performance**: Distributed vector database optimized for large-scale applications
//! - **Cloud Native**: Kubernetes-ready with horizontal scaling
//! - **Rich Indexing**: Multiple index types (IVF, HNSW, ANNOY, etc.)
//! - **Metadata Filtering**: Complex filtering with boolean expressions
//! - **Multi-tenancy**: Collection-based isolation and resource management
//! - **ACID Transactions**: Consistency guarantees for critical operations
//! - **Real-time**: Support for real-time data ingestion and querying
//!
//! ## Quick Start
//!
//! ```rust
//! use lumosai_vector_milvus::{MilvusStorage, MilvusConfig};
//! use lumosai_vector_core::traits::VectorStorage;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create Milvus storage
//!     let config = MilvusConfig::new("http://localhost:19530")
//!         .with_database("default")
//!         .with_auth("username", "password");
//!     let storage = MilvusStorage::new(config).await?;
//!     
//!     // Create a collection
//!     let index_config = IndexConfig::new("documents", 384)
//!         .with_metric(SimilarityMetric::Cosine);
//!     storage.create_index(index_config).await?;
//!     
//!     // Insert documents
//!     let docs = vec![
//!         Document::new("doc1", "Hello world")
//!             .with_embedding(vec![0.1; 384])
//!             .with_metadata("category", "greeting"),
//!     ];
//!     storage.upsert_documents("documents", docs).await?;
//!     
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;

pub mod storage;
pub mod config;
pub mod error;
pub mod client;
pub mod types;

pub use storage::MilvusStorage;
pub use config::{MilvusConfig, MilvusConfigBuilder};
pub use error::{MilvusError, MilvusResult};
pub use client::MilvusClient;
pub use types::*;

// Re-export core types for convenience
pub use lumosai_vector_core::types::*;
pub use lumosai_vector_core::traits::VectorStorage;

/// Milvus client for managing connections and databases
#[derive(Clone)]
pub struct MilvusConnection {
    /// HTTP client
    client: reqwest::Client,
    
    /// Configuration
    config: MilvusConfig,
    
    /// Authentication token (if using token-based auth)
    auth_token: Option<String>,
}

impl MilvusConnection {
    /// Create a new Milvus connection
    pub async fn new(config: MilvusConfig) -> MilvusResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| MilvusError::Connection(e.to_string()))?;
        
        let mut connection = Self {
            client,
            config,
            auth_token: None,
        };
        
        // Authenticate if credentials are provided
        if let Some(ref auth) = connection.config.auth {
            connection.authenticate().await?;
        }
        
        Ok(connection)
    }
    
    /// Authenticate with Milvus
    async fn authenticate(&mut self) -> MilvusResult<()> {
        if let Some(ref auth) = self.config.auth {
            let auth_request = AuthRequest {
                username: auth.username.clone(),
                password: auth.password.clone(),
            };
            
            let url = format!("{}/v1/auth/login", self.config.endpoint);
            let response = self.client
                .post(&url)
                .json(&auth_request)
                .send()
                .await
                .map_err(|e| MilvusError::Connection(e.to_string()))?;
            
            if response.status().is_success() {
                let auth_response: AuthResponse = response
                    .json()
                    .await
                    .map_err(|e| MilvusError::Serialization(e.to_string()))?;
                
                self.auth_token = Some(auth_response.token);
                tracing::info!("Successfully authenticated with Milvus");
            } else {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(MilvusError::Authentication(format!("Authentication failed: {}", error_text)));
            }
        }
        
        Ok(())
    }
    
    /// Get the HTTP client with authentication headers
    pub fn authenticated_client(&self) -> reqwest::RequestBuilder {
        let mut builder = self.client.get(&self.config.endpoint);
        
        if let Some(ref token) = self.auth_token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }
        
        builder
    }
    
    /// Make an authenticated POST request
    pub fn post(&self, url: &str) -> reqwest::RequestBuilder {
        let mut builder = self.client.post(url);
        
        if let Some(ref token) = self.auth_token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }
        
        builder
    }
    
    /// Make an authenticated GET request
    pub fn get(&self, url: &str) -> reqwest::RequestBuilder {
        let mut builder = self.client.get(url);
        
        if let Some(ref token) = self.auth_token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }
        
        builder
    }
    
    /// Make an authenticated DELETE request
    pub fn delete(&self, url: &str) -> reqwest::RequestBuilder {
        let mut builder = self.client.delete(url);
        
        if let Some(ref token) = self.auth_token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }
        
        builder
    }
    
    /// Get the configuration
    pub fn config(&self) -> &MilvusConfig {
        &self.config
    }
    
    /// Check connection health
    pub async fn health_check(&self) -> MilvusResult<()> {
        let url = format!("{}/health", self.config.endpoint);
        let response = self.get(&url)
            .send()
            .await
            .map_err(|e| MilvusError::Connection(e.to_string()))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(MilvusError::Connection("Health check failed".to_string()))
        }
    }
}

/// Create a new Milvus storage instance
pub async fn create_milvus_storage(endpoint: &str) -> MilvusResult<MilvusStorage> {
    let config = MilvusConfig::new(endpoint);
    MilvusStorage::new(config).await
}

/// Create a new Milvus storage instance with configuration
pub async fn create_milvus_storage_with_config(config: MilvusConfig) -> MilvusResult<MilvusStorage> {
    MilvusStorage::new(config).await
}

/// Utility functions for Milvus operations
pub mod utils {
    use super::*;
    use lumosai_vector_core::types::{Document, MetadataValue};
    
    /// Convert LumosAI documents to Milvus entities
    pub fn documents_to_entities(documents: &[Document]) -> MilvusResult<Vec<MilvusEntity>> {
        let mut entities = Vec::new();
        
        for doc in documents {
            let embedding = doc.embedding.as_ref()
                .ok_or_else(|| MilvusError::InvalidData("Document missing embedding".to_string()))?;
            
            let entity = MilvusEntity {
                id: doc.id.clone(),
                vector: embedding.clone(),
                content: doc.content.clone(),
                metadata: doc.metadata.clone(),
            };
            
            entities.push(entity);
        }
        
        Ok(entities)
    }
    
    /// Convert Milvus entities to LumosAI documents
    pub fn entities_to_documents(entities: &[MilvusEntity]) -> Vec<Document> {
        entities.iter().map(|entity| {
            let mut document = Document::new(&entity.id, &entity.content);
            document.embedding = Some(entity.vector.clone());
            document.metadata = entity.metadata.clone();
            document
        }).collect()
    }
    
    /// Convert metadata value to Milvus-compatible value
    pub fn metadata_value_to_milvus(value: &MetadataValue) -> serde_json::Value {
        match value {
            MetadataValue::String(s) => serde_json::Value::String(s.clone()),
            MetadataValue::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            MetadataValue::Float(f) => {
                serde_json::Number::from_f64(*f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
            MetadataValue::Boolean(b) => serde_json::Value::Bool(*b),
            MetadataValue::Null => serde_json::Value::Null,
            MetadataValue::Array(arr) => {
                let json_arr: Vec<serde_json::Value> = arr
                    .iter()
                    .map(metadata_value_to_milvus)
                    .collect();
                serde_json::Value::Array(json_arr)
            }
            MetadataValue::Object(obj) => {
                let json_obj: serde_json::Map<String, serde_json::Value> = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), metadata_value_to_milvus(v)))
                    .collect();
                serde_json::Value::Object(json_obj)
            }
        }
    }
    
    /// Validate vector dimensions
    pub fn validate_vector_dimension(vectors: &[Vec<f32>]) -> MilvusResult<usize> {
        if vectors.is_empty() {
            return Err(MilvusError::InvalidData("No vectors provided".to_string()));
        }
        
        let expected_dim = vectors[0].len();
        if expected_dim == 0 {
            return Err(MilvusError::InvalidData("Vector dimension cannot be zero".to_string()));
        }
        
        for (i, vector) in vectors.iter().enumerate() {
            if vector.len() != expected_dim {
                return Err(MilvusError::InvalidData(
                    format!("Vector {} has dimension {} but expected {}", i, vector.len(), expected_dim)
                ));
            }
        }
        
        Ok(expected_dim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_creation() {
        let config = MilvusConfig::new("http://localhost:19530");
        assert_eq!(config.endpoint, "http://localhost:19530");
        assert_eq!(config.database, "default");
    }
    
    #[test]
    fn test_utils_document_conversion() {
        let doc = Document::new("test", "content")
            .with_embedding(vec![1.0, 2.0, 3.0])
            .with_metadata("key", "value");
        
        let entities = utils::documents_to_entities(&[doc]).unwrap();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].id, "test");
        assert_eq!(entities[0].vector, vec![1.0, 2.0, 3.0]);
        
        let docs = utils::entities_to_documents(&entities);
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].id, "test");
    }
    
    #[test]
    fn test_vector_validation() {
        let vectors = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
        ];
        
        let dim = utils::validate_vector_dimension(&vectors).unwrap();
        assert_eq!(dim, 3);
        
        let invalid_vectors = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0], // Wrong dimension
        ];
        
        assert!(utils::validate_vector_dimension(&invalid_vectors).is_err());
    }
}
