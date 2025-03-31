use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use cloudflare::framework::{
    async_api::Client,
    auth::Credentials,
    Environment,
    HttpApiClientConfig,
};
use cloudflare::endpoints::vectorize::{
    CreateIndexRequest,
    DeleteVectorsRequest,
    IndexDetails,
    InsertVectorsParams,
    InsertVectorsRequest,
    ListIndexesResponse,
    QueryRequest,
    QueryResponse,
    UpsertRequest,
    UpsertVectorsResult,
    VectorMetadata,
};
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::StoreError;
use crate::vector::{
    CreateIndexParams,
    IndexStats,
    QueryParams,
    QueryResult,
    UpsertParams,
    VectorFilter,
    VectorFilterTranslator,
    VectorStore,
};

/// Cloudflare Vectorize store implementation
#[derive(Debug)]
pub struct VectorizeStore {
    client: Arc<Mutex<Client>>,
    account_id: String,
    filter_translator: VectorizeFilterTranslator,
}

impl VectorizeStore {
    /// Create a new Vectorize store with API token
    pub async fn new(api_token: &str, account_id: &str) -> Result<Self, StoreError> {
        let credentials = Credentials::UserAuthToken {
            token: api_token.to_string(),
        };
        
        let config = HttpApiClientConfig::default();
        let client = Client::new(credentials, config, Environment::Production)
            .map_err(|e| StoreError::ConnectionError(e.to_string()))?;
            
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            account_id: account_id.to_string(),
            filter_translator: VectorizeFilterTranslator,
        })
    }
    
    /// Create a new Vectorize store with API key and email
    pub async fn with_api_key(api_key: &str, email: &str, account_id: &str) -> Result<Self, StoreError> {
        let credentials = Credentials::UserAuthKey {
            key: api_key.to_string(),
            email: email.to_string(),
        };
        
        let config = HttpApiClientConfig::default();
        let client = Client::new(credentials, config, Environment::Production)
            .map_err(|e| StoreError::ConnectionError(e.to_string()))?;
            
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            account_id: account_id.to_string(),
            filter_translator: VectorizeFilterTranslator,
        })
    }
    
    /// Convert metadata to the format expected by Vectorize
    fn convert_metadata(&self, metadata: HashMap<String, Value>) -> VectorMetadata {
        let mut result = VectorMetadata::new();
        
        for (key, value) in metadata {
            match value {
                Value::String(s) => { result.insert(key, s); },
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        result.insert(key, i.to_string());
                    } else if let Some(f) = n.as_f64() {
                        result.insert(key, f.to_string());
                    }
                },
                Value::Bool(b) => { result.insert(key, b.to_string()); },
                _ => {
                    // Convert complex types to JSON string
                    if let Ok(s) = serde_json::to_string(&value) {
                        result.insert(key, s);
                    }
                }
            }
        }
        
        result
    }
}

#[async_trait]
impl VectorStore for VectorizeStore {
    #[instrument(skip(self))]
    async fn create_index(&self, params: CreateIndexParams) -> Result<(), StoreError> {
        let client = self.client.lock().await;
        
        let distance_metric = match params.metric.to_lowercase().as_str() {
            "cosine" => "cosine",
            "euclidean" => "euclidean",
            "dotproduct" => "dotProduct",
            _ => "cosine", // Default
        };
        
        let request = CreateIndexRequest {
            name: params.index_name.clone(),
            dimensions: params.dimension,
            metric: distance_metric.to_string(),
        };
        
        client.request_handle(&request, &self.account_id)
            .await
            .map_err(|e| StoreError::IndexError(format!("Failed to create index: {}", e)))?;
            
        debug!("Created Vectorize index: {}", params.index_name);
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn upsert(&self, params: UpsertParams) -> Result<Vec<String>, StoreError> {
        let client = self.client.lock().await;
        
        let ids = params.ids.unwrap_or_else(|| {
            (0..params.vectors.len())
                .map(|_| Uuid::new_v4().to_string())
                .collect()
        });
        
        // Prepare vectors for NDJSON format
        let mut vectors_data = Vec::new();
        
        for (i, id) in ids.iter().enumerate() {
            let vector = params.vectors.get(i).cloned().unwrap_or_default();
            let metadata = params.metadata.get(i)
                .cloned()
                .unwrap_or_default();
                
            let metadata = self.convert_metadata(metadata);
            
            let params = InsertVectorsParams {
                id: id.clone(),
                values: vector,
                metadata: Some(metadata),
            };
            
            vectors_data.push(params);
        }
        
        let request = UpsertRequest {
            name: params.index_name.clone(),
            vectors: vectors_data,
        };
        
        let result: UpsertVectorsResult = client.request_handle(&request, &self.account_id)
            .await
            .map_err(|e| StoreError::VectorError(format!("Failed to upsert vectors: {}", e)))?;
            
        debug!("Upserted {} vectors into Vectorize index: {}", result.count, params.index_name);
        Ok(ids)
    }
    
    #[instrument(skip(self))]
    async fn query(&self, params: QueryParams) -> Result<Vec<QueryResult>, StoreError> {
        let client = self.client.lock().await;
        
        let mut request = QueryRequest {
            name: params.index_name.clone(),
            vector: params.query_vector,
            top_k: params.top_k,
            return_vectors: params.include_vector,
            ..Default::default()
        };
        
        if let Some(filter) = params.filter {
            let vectorize_filter = self.filter_translator.translate_filter(Some(filter)).await?;
            request.set_filter(Some(vectorize_filter));
        }
        
        let result: QueryResponse = client.request_handle(&request, &self.account_id)
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to query index: {}", e)))?;
            
        let query_results = result.vectors.into_iter()
            .map(|v| {
                // Convert metadata back to map
                let metadata = v.metadata.unwrap_or_default().into_iter()
                    .map(|(k, v)| {
                        // Try to parse as JSON first
                        if let Ok(parsed) = serde_json::from_str::<Value>(&v) {
                            (k, parsed)
                        } else {
                            (k, Value::String(v))
                        }
                    })
                    .collect();
                    
                QueryResult {
                    id: v.id,
                    score: v.score,
                    metadata,
                    vector: if params.include_vector { Some(v.values) } else { None },
                }
            })
            .collect();
            
        debug!("Queried Vectorize index: {}, found {} results", params.index_name, query_results.len());
        Ok(query_results)
    }
    
    #[instrument(skip(self))]
    async fn list_indexes(&self) -> Result<Vec<String>, StoreError> {
        let client = self.client.lock().await;
        
        let response: ListIndexesResponse = client.request_handle(&(), &self.account_id)
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to list indexes: {}", e)))?;
            
        let indexes = response.indexes.into_iter()
            .map(|index| index.name)
            .collect();
            
        debug!("Listed Vectorize indexes");
        Ok(indexes)
    }
    
    #[instrument(skip(self))]
    async fn describe_index(&self, index_name: &str) -> Result<IndexStats, StoreError> {
        let client = self.client.lock().await;
        
        let response: IndexDetails = client.request_handle(&index_name, &self.account_id)
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to get index details: {}", e)))?;
            
        Ok(IndexStats {
            dimension: response.dimensions,
            count: response.count,
            metric: response.metric,
        })
    }
    
    #[instrument(skip(self))]
    async fn delete_index(&self, index_name: &str) -> Result<(), StoreError> {
        let client = self.client.lock().await;
        
        client.request_handle(&index_name, &self.account_id)
            .await
            .map_err(|e| StoreError::IndexError(format!("Failed to delete index: {}", e)))?;
            
        debug!("Deleted Vectorize index: {}", index_name);
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn update_vector_by_id(
        &self, 
        index_name: &str, 
        id: &str, 
        vector: Option<Vec<f32>>, 
        metadata: Option<HashMap<String, Value>>,
    ) -> Result<(), StoreError> {
        let client = self.client.lock().await;
        
        // Vectorize requires both vector and metadata for updates
        // If one is missing, we need to retrieve the current values
        
        if vector.is_none() && metadata.is_none() {
            return Err(StoreError::VectorError("Either vector or metadata must be provided".to_string()));
        }
        
        // If both are provided, do direct update
        if let (Some(vec), Some(meta)) = (&vector, &metadata) {
            let params = InsertVectorsParams {
                id: id.to_string(),
                values: vec.clone(),
                metadata: Some(self.convert_metadata(meta.clone())),
            };
            
            let request = UpsertRequest {
                name: index_name.to_string(),
                vectors: vec![params],
            };
            
            client.request_handle(&request, &self.account_id)
                .await
                .map_err(|e| StoreError::VectorError(format!("Failed to update vector: {}", e)))?;
                
            debug!("Updated vector in Vectorize index: {}, id: {}", index_name, id);
            return Ok(());
        }
        
        // Otherwise, we need to query the current values
        let query_request = QueryRequest {
            name: index_name.to_string(),
            id: Some(id.to_string()),
            return_vectors: true,
            ..Default::default()
        };
        
        let result: QueryResponse = client.request_handle(&query_request, &self.account_id)
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to retrieve vector: {}", e)))?;
            
        if result.vectors.is_empty() {
            return Err(StoreError::VectorError(format!("Vector with id {} not found", id)));
        }
        
        let existing = &result.vectors[0];
        
        // Prepare update with existing or new values
        let update_vector = vector.unwrap_or_else(|| existing.values.clone());
        let update_metadata = metadata.map(|m| self.convert_metadata(m))
            .or_else(|| existing.metadata.clone());
            
        let params = InsertVectorsParams {
            id: id.to_string(),
            values: update_vector,
            metadata: update_metadata,
        };
        
        let request = UpsertRequest {
            name: index_name.to_string(),
            vectors: vec![params],
        };
        
        client.request_handle(&request, &self.account_id)
            .await
            .map_err(|e| StoreError::VectorError(format!("Failed to update vector: {}", e)))?;
            
        debug!("Updated vector in Vectorize index: {}, id: {}", index_name, id);
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn delete_vectors(&self, index_name: &str, ids: &[String]) -> Result<(), StoreError> {
        let client = self.client.lock().await;
        
        let request = DeleteVectorsRequest {
            name: index_name.to_string(),
            ids: ids.to_vec(),
        };
        
        client.request_handle(&request, &self.account_id)
            .await
            .map_err(|e| StoreError::VectorError(format!("Failed to delete vectors: {}", e)))?;
            
        debug!("Deleted {} vectors from Vectorize index: {}", ids.len(), index_name);
        Ok(())
    }
}

/// Vectorize-specific filter translator
#[derive(Debug, Default)]
pub struct VectorizeFilterTranslator;

#[async_trait]
impl VectorFilterTranslator for VectorizeFilterTranslator {
    async fn translate_filter<T: Send + Sync>(&self, filter: Option<VectorFilter>) -> Result<T, StoreError> {
        if filter.is_none() {
            return Err(StoreError::FilterError("No filter provided".to_string()));
        }
        
        // Convert to JSON format expected by Vectorize
        let json_filter = match self.filter_to_json(filter.unwrap())? {
            Some(f) => f,
            None => return Err(StoreError::FilterError("Invalid filter".to_string())),
        };
        
        serde_json::from_value(json_filter)
            .map_err(|e| StoreError::SerializationError(e.to_string()))
    }
}

impl VectorizeFilterTranslator {
    /// Convert a VectorFilter to a JSON Value for Vectorize
    fn filter_to_json(&self, filter: VectorFilter) -> Result<Option<Value>, StoreError> {
        match filter {
            VectorFilter::And { and } => {
                let mut conditions = Vec::new();
                for f in and {
                    if let Some(cond) = self.filter_to_json(f)? {
                        conditions.push(cond);
                    }
                }
                
                if conditions.is_empty() {
                    Ok(None)
                } else if conditions.len() == 1 {
                    Ok(Some(conditions[0].clone()))
                } else {
                    Ok(Some(json!({ "$and": conditions })))
                }
            },
            VectorFilter::Or { or } => {
                let mut conditions = Vec::new();
                for f in or {
                    if let Some(cond) = self.filter_to_json(f)? {
                        conditions.push(cond);
                    }
                }
                
                if conditions.is_empty() {
                    Ok(None)
                } else if conditions.len() == 1 {
                    Ok(Some(conditions[0].clone()))
                } else {
                    Ok(Some(json!({ "$or": conditions })))
                }
            },
            VectorFilter::Not { not } => {
                if let Some(inner) = self.filter_to_json(*not)? {
                    Ok(Some(json!({ "$not": inner })))
                } else {
                    Ok(None)
                }
            },
            VectorFilter::Field(conditions) => {
                if conditions.len() != 1 {
                    return Err(StoreError::FilterError(
                        "Field filter must contain exactly one field".to_string()
                    ));
                }
                
                let (field, condition) = conditions.into_iter().next().unwrap();
                
                match condition {
                    crate::vector::FieldCondition::Value(value) => {
                        Ok(Some(json!({ field: value })))
                    },
                    crate::vector::FieldCondition::Operator(ops) => {
                        let mut field_ops = serde_json::Map::new();
                        
                        for (op, value) in ops {
                            field_ops.insert(op, value);
                        }
                        
                        Ok(Some(json!({ field: field_ops })))
                    },
                }
            },
        }
    }
} 