//! Milvus client implementation

use std::collections::HashMap;
use std::time::Duration;

use crate::{
    config::MilvusConfig,
    error::{MilvusError, MilvusResult},
    types::*,
    MilvusConnection,
};

/// Milvus client for database operations
#[derive(Clone)]
pub struct MilvusClient {
    /// Connection to Milvus
    connection: MilvusConnection,
}

impl MilvusClient {
    /// Create a new Milvus client
    pub async fn new(config: MilvusConfig) -> MilvusResult<Self> {
        let connection = MilvusConnection::new(config).await?;
        
        Ok(Self { connection })
    }
    
    /// Get the connection
    pub fn connection(&self) -> &MilvusConnection {
        &self.connection
    }
    
    /// Health check
    pub async fn health_check(&self) -> MilvusResult<bool> {
        let url = format!("{}/health", self.connection.config().endpoint);
        
        let response = self.connection
            .get(&url)
            .send()
            .await?;
        
        if response.status().is_success() {
            let health: HealthResponse = response.json().await?;
            Ok(health.is_healthy)
        } else {
            Ok(false)
        }
    }
    
    /// List collections
    pub async fn list_collections(&self) -> MilvusResult<Vec<String>> {
        let url = format!("{}/v1/vector/collections", self.connection.config().endpoint);
        
        let response = self.connection
            .get(&url)
            .send()
            .await?;
        
        if response.status().is_success() {
            let collections: serde_json::Value = response.json().await?;
            
            // Extract collection names from response
            if let Some(data) = collections.get("data") {
                if let Some(collections_array) = data.as_array() {
                    let names: Vec<String> = collections_array
                        .iter()
                        .filter_map(|c| c.get("collection_name"))
                        .filter_map(|n| n.as_str())
                        .map(|s| s.to_string())
                        .collect();
                    return Ok(names);
                }
            }
            
            Ok(Vec::new())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(MilvusError::Database(format!("Failed to list collections: {}", error_text)))
        }
    }
    
    /// Check if collection exists
    pub async fn has_collection(&self, collection_name: &str) -> MilvusResult<bool> {
        let url = format!(
            "{}/v1/vector/collections/{}",
            self.connection.config().endpoint,
            collection_name
        );
        
        let response = self.connection
            .get(&url)
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }
    
    /// Create collection
    pub async fn create_collection(&self, schema: CollectionSchema) -> MilvusResult<()> {
        let url = format!("{}/v1/vector/collections", self.connection.config().endpoint);
        
        let request = CreateCollectionRequest {
            collection_name: schema.name.clone(),
            schema,
            shards_num: self.connection.config().collection_config.shards_num,
            consistency_level: format!("{:?}", self.connection.config().collection_config.consistency_level),
        };
        
        let response = self.connection
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            tracing::info!("Collection '{}' created successfully", request.collection_name);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(MilvusError::Collection(format!("Failed to create collection: {}", error_text)))
        }
    }
    
    /// Drop collection
    pub async fn drop_collection(&self, collection_name: &str) -> MilvusResult<()> {
        let url = format!(
            "{}/v1/vector/collections/{}",
            self.connection.config().endpoint,
            collection_name
        );
        
        let response = self.connection
            .delete(&url)
            .send()
            .await?;
        
        if response.status().is_success() {
            tracing::info!("Collection '{}' dropped successfully", collection_name);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(MilvusError::Collection(format!("Failed to drop collection: {}", error_text)))
        }
    }
    
    /// Get collection info
    pub async fn describe_collection(&self, collection_name: &str) -> MilvusResult<CollectionInfo> {
        let url = format!(
            "{}/v1/vector/collections/{}/describe",
            self.connection.config().endpoint,
            collection_name
        );
        
        let response = self.connection
            .get(&url)
            .send()
            .await?;
        
        if response.status().is_success() {
            let info: CollectionInfo = response.json().await?;
            Ok(info)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(MilvusError::Collection(format!("Failed to describe collection: {}", error_text)))
        }
    }
    
    /// Create index
    pub async fn create_index(
        &self,
        collection_name: &str,
        field_name: &str,
        index_type: &str,
        metric_type: &str,
        params: serde_json::Value,
    ) -> MilvusResult<()> {
        let url = format!("{}/v1/vector/indexes", self.connection.config().endpoint);
        
        let request = CreateIndexRequest {
            collection_name: collection_name.to_string(),
            field_name: field_name.to_string(),
            index_name: format!("{}_index", field_name),
            extra_params: IndexExtraParams {
                index_type: index_type.to_string(),
                metric_type: metric_type.to_string(),
                params,
            },
        };
        
        let response = self.connection
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            tracing::info!("Index created for collection '{}' field '{}'", collection_name, field_name);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(MilvusError::Index(format!("Failed to create index: {}", error_text)))
        }
    }
    
    /// Insert entities
    pub async fn insert(&self, collection_name: &str, entities: &[MilvusEntity]) -> MilvusResult<()> {
        if entities.is_empty() {
            return Ok(());
        }
        
        let url = format!("{}/v1/vector/insert", self.connection.config().endpoint);
        
        // Prepare field data
        let mut ids = Vec::new();
        let mut vectors = Vec::new();
        let mut contents = Vec::new();
        let mut metadata_json = Vec::new();
        
        for entity in entities {
            ids.push(entity.id.clone());
            vectors.push(entity.vector.clone());
            contents.push(entity.content.clone());
            
            let metadata_str = serde_json::to_string(&entity.metadata)?;
            metadata_json.push(metadata_str);
        }
        
        let fields_data = vec![
            FieldData {
                field_name: "id".to_string(),
                field_type: DataType::VarChar,
                field: serde_json::to_value(ids)?,
            },
            FieldData {
                field_name: "vector".to_string(),
                field_type: DataType::FloatVector,
                field: serde_json::to_value(vectors)?,
            },
            FieldData {
                field_name: "content".to_string(),
                field_type: DataType::VarChar,
                field: serde_json::to_value(contents)?,
            },
            FieldData {
                field_name: "metadata".to_string(),
                field_type: DataType::JSON,
                field: serde_json::to_value(metadata_json)?,
            },
        ];
        
        let request = InsertRequest {
            collection_name: collection_name.to_string(),
            fields_data,
            num_rows: entities.len(),
        };
        
        let response = self.connection
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            tracing::debug!("Inserted {} entities into collection '{}'", entities.len(), collection_name);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(MilvusError::Database(format!("Failed to insert entities: {}", error_text)))
        }
    }
    
    /// Search vectors
    pub async fn search(
        &self,
        collection_name: &str,
        vectors: &[Vec<f32>],
        limit: usize,
        metric_type: &str,
        params: serde_json::Value,
        output_fields: &[String],
        expr: Option<&str>,
    ) -> MilvusResult<SearchResponse> {
        let url = format!("{}/v1/vector/search", self.connection.config().endpoint);
        
        let request = SearchRequest {
            collection_name: collection_name.to_string(),
            vector_field_name: "vector".to_string(),
            vectors: vectors.to_vec(),
            search_params: SearchParams {
                metric_type: metric_type.to_string(),
                params,
            },
            limit,
            output_fields: output_fields.to_vec(),
            expr: expr.map(|s| s.to_string()),
        };
        
        let response = self.connection
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let search_response: SearchResponse = response.json().await?;
            
            if search_response.status.error_code == 0 {
                Ok(search_response)
            } else {
                Err(MilvusError::Query(format!("Search failed: {}", search_response.status.reason)))
            }
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(MilvusError::Query(format!("Failed to search: {}", error_text)))
        }
    }
    
    /// Query entities
    pub async fn query(
        &self,
        collection_name: &str,
        expr: &str,
        output_fields: &[String],
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> MilvusResult<QueryResponse> {
        let url = format!("{}/v1/vector/query", self.connection.config().endpoint);
        
        let request = QueryRequest {
            collection_name: collection_name.to_string(),
            expr: expr.to_string(),
            output_fields: output_fields.to_vec(),
            limit,
            offset,
        };
        
        let response = self.connection
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let query_response: QueryResponse = response.json().await?;
            
            if query_response.status.error_code == 0 {
                Ok(query_response)
            } else {
                Err(MilvusError::Query(format!("Query failed: {}", query_response.status.reason)))
            }
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(MilvusError::Query(format!("Failed to query: {}", error_text)))
        }
    }
    
    /// Delete entities
    pub async fn delete(&self, collection_name: &str, expr: &str) -> MilvusResult<i64> {
        let url = format!("{}/v1/vector/delete", self.connection.config().endpoint);
        
        let request = DeleteRequest {
            collection_name: collection_name.to_string(),
            expr: expr.to_string(),
        };
        
        let response = self.connection
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let delete_response: DeleteResponse = response.json().await?;
            
            if delete_response.status.error_code == 0 {
                Ok(delete_response.delete_cnt)
            } else {
                Err(MilvusError::Database(format!("Delete failed: {}", delete_response.status.reason)))
            }
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(MilvusError::Database(format!("Failed to delete: {}", error_text)))
        }
    }
    
    /// Get collection statistics
    pub async fn get_collection_stats(&self, collection_name: &str) -> MilvusResult<CollectionStats> {
        let url = format!(
            "{}/v1/vector/collections/{}/stats",
            self.connection.config().endpoint,
            collection_name
        );
        
        let response = self.connection
            .get(&url)
            .send()
            .await?;
        
        if response.status().is_success() {
            let stats: CollectionStats = response.json().await?;
            Ok(stats)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(MilvusError::Database(format!("Failed to get collection stats: {}", error_text)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MilvusConfig;
    
    #[tokio::test]
    async fn test_client_creation() {
        let config = MilvusConfig::new("http://localhost:19530");
        
        // This will fail without a running Milvus instance, but tests the creation logic
        let result = MilvusClient::new(config).await;
        
        // We expect a connection error since there's no Milvus running
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(matches!(e, MilvusError::Connection(_)));
        }
    }
    
    #[test]
    fn test_entity_creation() {
        let entity = MilvusEntity::new(
            "test_id".to_string(),
            vec![1.0, 2.0, 3.0],
            "test content".to_string(),
        );
        
        assert_eq!(entity.id, "test_id");
        assert_eq!(entity.vector.len(), 3);
        assert_eq!(entity.content, "test content");
    }
}
