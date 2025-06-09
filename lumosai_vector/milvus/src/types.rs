//! Type definitions for Milvus integration

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use lumosai_vector_core::types::MetadataValue;

/// Milvus entity representing a document with vector and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilvusEntity {
    /// Unique identifier
    pub id: String,
    
    /// Vector embedding
    pub vector: Vec<f32>,
    
    /// Text content
    pub content: String,
    
    /// Metadata fields
    pub metadata: HashMap<String, MetadataValue>,
}

/// Authentication request
#[derive(Debug, Serialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

/// Authentication response
#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    #[serde(default)]
    pub expires_in: Option<u64>,
}

/// Collection schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSchema {
    /// Collection name
    pub name: String,
    
    /// Description
    pub description: String,
    
    /// Fields definition
    pub fields: Vec<FieldSchema>,
    
    /// Auto ID generation
    pub auto_id: bool,
}

/// Field schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    /// Field name
    pub name: String,
    
    /// Data type
    pub data_type: DataType,
    
    /// Primary key flag
    pub is_primary_key: bool,
    
    /// Auto ID flag
    pub auto_id: bool,
    
    /// Description
    pub description: String,
    
    /// Type parameters (for vector fields)
    pub type_params: Option<TypeParams>,
}

/// Data types supported by Milvus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    /// Boolean
    Bool,
    
    /// 8-bit integer
    Int8,
    
    /// 16-bit integer
    Int16,
    
    /// 32-bit integer
    Int32,
    
    /// 64-bit integer
    Int64,
    
    /// 32-bit float
    Float,
    
    /// 64-bit double
    Double,
    
    /// Variable-length string
    VarChar,
    
    /// JSON
    JSON,
    
    /// Float vector
    FloatVector,
    
    /// Binary vector
    BinaryVector,
}

/// Type parameters for fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeParams {
    /// Dimension for vector fields
    pub dim: Option<usize>,
    
    /// Max length for varchar fields
    pub max_length: Option<usize>,
}

/// Collection creation request
#[derive(Debug, Serialize)]
pub struct CreateCollectionRequest {
    /// Collection name
    pub collection_name: String,
    
    /// Schema
    pub schema: CollectionSchema,
    
    /// Shards number
    pub shards_num: usize,
    
    /// Consistency level
    pub consistency_level: String,
}

/// Index creation request
#[derive(Debug, Serialize)]
pub struct CreateIndexRequest {
    /// Collection name
    pub collection_name: String,
    
    /// Field name
    pub field_name: String,
    
    /// Index name
    pub index_name: String,
    
    /// Index parameters
    pub extra_params: IndexExtraParams,
}

/// Index parameters
#[derive(Debug, Serialize)]
pub struct IndexExtraParams {
    /// Index type
    pub index_type: String,
    
    /// Metric type
    pub metric_type: String,
    
    /// Index parameters
    pub params: serde_json::Value,
}

/// Insert request
#[derive(Debug, Serialize)]
pub struct InsertRequest {
    /// Collection name
    pub collection_name: String,
    
    /// Fields data
    pub fields_data: Vec<FieldData>,
    
    /// Number of rows
    pub num_rows: usize,
}

/// Field data for insert operations
#[derive(Debug, Serialize, Deserialize)]
pub struct FieldData {
    /// Field name
    pub field_name: String,

    /// Field type
    pub field_type: DataType,

    /// Field data
    pub field: serde_json::Value,
}

/// Search request
#[derive(Debug, Serialize)]
pub struct SearchRequest {
    /// Collection name
    pub collection_name: String,
    
    /// Vector field name
    pub vector_field_name: String,
    
    /// Search vectors
    pub vectors: Vec<Vec<f32>>,
    
    /// Search parameters
    pub search_params: SearchParams,
    
    /// Limit
    pub limit: usize,
    
    /// Output fields
    pub output_fields: Vec<String>,
    
    /// Filter expression
    pub expr: Option<String>,
}

/// Search parameters
#[derive(Debug, Serialize)]
pub struct SearchParams {
    /// Metric type
    pub metric_type: String,
    
    /// Search parameters
    pub params: serde_json::Value,
}

/// Search response
#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    /// Search results
    pub results: SearchResults,
    
    /// Status
    pub status: ResponseStatus,
}

/// Search results
#[derive(Debug, Deserialize)]
pub struct SearchResults {
    /// Number of queries
    pub num_queries: usize,
    
    /// Top K
    pub top_k: usize,
    
    /// Fields data
    pub fields_data: Vec<FieldData>,
    
    /// Scores
    pub scores: Vec<f32>,
    
    /// IDs
    pub ids: IdArray,
}

/// ID array
#[derive(Debug, Deserialize)]
pub struct IdArray {
    /// Integer IDs
    pub int_id: Option<IntArray>,
    
    /// String IDs
    pub str_id: Option<StrArray>,
}

/// Integer array
#[derive(Debug, Deserialize)]
pub struct IntArray {
    pub data: Vec<i64>,
}

/// String array
#[derive(Debug, Deserialize)]
pub struct StrArray {
    pub data: Vec<String>,
}

/// Response status
#[derive(Debug, Deserialize)]
pub struct ResponseStatus {
    /// Error code
    pub error_code: i32,
    
    /// Reason
    pub reason: String,
}

/// Collection info
#[derive(Debug, Deserialize)]
pub struct CollectionInfo {
    /// Collection name
    pub name: String,
    
    /// Collection ID
    pub id: i64,
    
    /// Schema
    pub schema: CollectionSchema,
    
    /// Shards number
    pub shards_num: usize,
    
    /// Consistency level
    pub consistency_level: String,
    
    /// Created time
    pub created_utc_timestamps: u64,
}

/// Index info
#[derive(Debug, Deserialize)]
pub struct IndexInfo {
    /// Index name
    pub index_name: String,
    
    /// Field name
    pub field_name: String,
    
    /// Index type
    pub index_type: String,
    
    /// Metric type
    pub metric_type: String,
    
    /// Index parameters
    pub params: serde_json::Value,
}

/// Collection statistics
#[derive(Debug, Deserialize)]
pub struct CollectionStats {
    /// Row count
    pub row_count: i64,
    
    /// Data size
    pub data_size: i64,
    
    /// Index size
    pub index_size: i64,
}

/// Query request
#[derive(Debug, Serialize)]
pub struct QueryRequest {
    /// Collection name
    pub collection_name: String,
    
    /// Filter expression
    pub expr: String,
    
    /// Output fields
    pub output_fields: Vec<String>,
    
    /// Limit
    pub limit: Option<usize>,
    
    /// Offset
    pub offset: Option<usize>,
}

/// Query response
#[derive(Debug, Deserialize)]
pub struct QueryResponse {
    /// Fields data
    pub fields_data: Vec<FieldData>,
    
    /// Status
    pub status: ResponseStatus,
}

/// Delete request
#[derive(Debug, Serialize)]
pub struct DeleteRequest {
    /// Collection name
    pub collection_name: String,
    
    /// Filter expression
    pub expr: String,
}

/// Delete response
#[derive(Debug, Deserialize)]
pub struct DeleteResponse {
    /// Delete count
    pub delete_cnt: i64,
    
    /// Status
    pub status: ResponseStatus,
}

/// Health check response
#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    /// Is healthy
    pub is_healthy: bool,
    
    /// Reasons
    pub reasons: Vec<String>,
}

impl MilvusEntity {
    /// Create a new Milvus entity
    pub fn new(id: String, vector: Vec<f32>, content: String) -> Self {
        Self {
            id,
            vector,
            content,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata field
    pub fn with_metadata<K: Into<String>, V: Into<MetadataValue>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl CollectionSchema {
    /// Create a new collection schema
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            fields: Vec::new(),
            auto_id: false,
        }
    }
    
    /// Add a field to the schema
    pub fn add_field(mut self, field: FieldSchema) -> Self {
        self.fields.push(field);
        self
    }
    
    /// Create a standard document schema with ID, vector, content, and metadata fields
    pub fn document_schema(name: &str, vector_dim: usize) -> Self {
        Self::new(name, "Document collection with vector embeddings")
            .add_field(FieldSchema {
                name: "id".to_string(),
                data_type: DataType::VarChar,
                is_primary_key: true,
                auto_id: false,
                description: "Document ID".to_string(),
                type_params: Some(TypeParams {
                    dim: None,
                    max_length: Some(255),
                }),
            })
            .add_field(FieldSchema {
                name: "vector".to_string(),
                data_type: DataType::FloatVector,
                is_primary_key: false,
                auto_id: false,
                description: "Vector embedding".to_string(),
                type_params: Some(TypeParams {
                    dim: Some(vector_dim),
                    max_length: None,
                }),
            })
            .add_field(FieldSchema {
                name: "content".to_string(),
                data_type: DataType::VarChar,
                is_primary_key: false,
                auto_id: false,
                description: "Document content".to_string(),
                type_params: Some(TypeParams {
                    dim: None,
                    max_length: Some(65535),
                }),
            })
            .add_field(FieldSchema {
                name: "metadata".to_string(),
                data_type: DataType::JSON,
                is_primary_key: false,
                auto_id: false,
                description: "Document metadata".to_string(),
                type_params: None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_milvus_entity() {
        let entity = MilvusEntity::new(
            "test_id".to_string(),
            vec![1.0, 2.0, 3.0],
            "test content".to_string(),
        ).with_metadata("category", "test");
        
        assert_eq!(entity.id, "test_id");
        assert_eq!(entity.vector, vec![1.0, 2.0, 3.0]);
        assert_eq!(entity.content, "test content");
        assert!(entity.metadata.contains_key("category"));
    }
    
    #[test]
    fn test_collection_schema() {
        let schema = CollectionSchema::document_schema("test_collection", 384);
        
        assert_eq!(schema.name, "test_collection");
        assert_eq!(schema.fields.len(), 4);
        
        // Check ID field
        let id_field = &schema.fields[0];
        assert_eq!(id_field.name, "id");
        assert!(id_field.is_primary_key);
        assert!(matches!(id_field.data_type, DataType::VarChar));
        
        // Check vector field
        let vector_field = &schema.fields[1];
        assert_eq!(vector_field.name, "vector");
        assert!(matches!(vector_field.data_type, DataType::FloatVector));
        assert_eq!(vector_field.type_params.as_ref().unwrap().dim, Some(384));
    }
}
