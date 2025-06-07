//! Qdrant vector storage implementation

use std::collections::HashMap;
use async_trait::async_trait;
use qdrant_client::{
    client::QdrantClient,
    qdrant::{
        CreateCollection, VectorParams, VectorsConfig, Distance,
        PointStruct, UpsertPoints, SearchPoints, DeletePoints,
        GetCollectionInfoRequest, ListCollectionsRequest,
        DeleteCollection, Filter, Condition, FieldCondition,
        Match, Value as QdrantValue, Range,
    },
};
use uuid::Uuid;
use tracing::{debug, instrument, warn};

use lumosai_vector_core::prelude::*;
use crate::{QdrantConfig, QdrantError, QdrantResult};

/// Qdrant vector storage implementation
pub struct QdrantVectorStorage {
    client: QdrantClient,
    config: QdrantConfig,
}

impl QdrantVectorStorage {
    /// Create a new Qdrant vector storage instance
    pub async fn new(url: &str) -> lumosai_vector_core::Result<Self> {
        let config = QdrantConfig::new(url);
        Self::with_config(config).await
    }

    /// Create a new Qdrant vector storage instance with configuration
    pub async fn with_config(config: QdrantConfig) -> lumosai_vector_core::Result<Self> {
        let mut client_config = qdrant_client::client::QdrantClientConfig::from_url(&config.url);
        
        if let Some(api_key) = &config.api_key {
            client_config = client_config.with_api_key(api_key);
        }
        
        let client = QdrantClient::new(Some(client_config))
            .map_err(|e| QdrantError::Connection(e.to_string()))?;
        
        // Test connection
        client.list_collections().await
            .map_err(|e| QdrantError::Connection(format!("Failed to connect: {}", e)))?;
        
        Ok(Self { client, config })
    }
    
    /// Convert similarity metric to Qdrant distance
    fn convert_metric(metric: SimilarityMetric) -> Distance {
        match metric {
            SimilarityMetric::Cosine => Distance::Cosine,
            SimilarityMetric::Euclidean => Distance::Euclid,
            SimilarityMetric::DotProduct => Distance::Dot,
            _ => Distance::Cosine, // Default fallback
        }
    }
    
    /// Convert Qdrant distance to similarity metric
    fn convert_distance(distance: Distance) -> SimilarityMetric {
        match distance {
            Distance::Cosine => SimilarityMetric::Cosine,
            Distance::Euclid => SimilarityMetric::Euclidean,
            Distance::Dot => SimilarityMetric::DotProduct,
            _ => SimilarityMetric::Cosine,
        }
    }
    
    /// Convert filter condition to Qdrant filter
    fn convert_filter(condition: FilterCondition) -> QdrantResult<Filter> {
        let filter_condition = match condition {
            FilterCondition::Eq(field, value) => {
                let qdrant_value = Self::convert_metadata_value(value)?;
                Condition::field(field, Match::value(qdrant_value))
            },
            FilterCondition::And(conditions) => {
                let mut sub_conditions = Vec::new();
                for cond in conditions {
                    let filter = Self::convert_filter(cond)?;
                    if let Some(must) = filter.must {
                        sub_conditions.extend(must);
                    }
                }
                return Ok(Filter {
                    must: Some(sub_conditions),
                    ..Default::default()
                });
            },
            FilterCondition::Or(conditions) => {
                let mut sub_conditions = Vec::new();
                for cond in conditions {
                    let filter = Self::convert_filter(cond)?;
                    if let Some(must) = filter.must {
                        sub_conditions.extend(must);
                    }
                }
                return Ok(Filter {
                    should: Some(sub_conditions),
                    ..Default::default()
                });
            },
            FilterCondition::Not(condition) => {
                let filter = Self::convert_filter(*condition)?;
                return Ok(Filter {
                    must_not: filter.must,
                    ..Default::default()
                });
            },
            FilterCondition::Gt(field, value) => {
                let qdrant_value = Self::convert_metadata_value(value)?;
                Condition::field(field, Range {
                    gt: Some(qdrant_value),
                    ..Default::default()
                })
            },
            FilterCondition::Lt(field, value) => {
                let qdrant_value = Self::convert_metadata_value(value)?;
                Condition::field(field, Range {
                    lt: Some(qdrant_value),
                    ..Default::default()
                })
            },
            FilterCondition::In(field, values) => {
                let qdrant_values: QdrantResult<Vec<_>> = values.into_iter()
                    .map(Self::convert_metadata_value)
                    .collect();
                Condition::field(field, Match::any(qdrant_values?))
            },
            _ => {
                return Err(QdrantError::Search("Unsupported filter condition".to_string()));
            }
        };
        
        Ok(Filter {
            must: Some(vec![filter_condition]),
            ..Default::default()
        })
    }
    
    /// Convert metadata value to Qdrant value
    fn convert_metadata_value(value: MetadataValue) -> QdrantResult<QdrantValue> {
        let qdrant_value = match value {
            MetadataValue::String(s) => QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(s)),
            },
            MetadataValue::Integer(i) => QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::IntegerValue(i)),
            },
            MetadataValue::Float(f) => QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::DoubleValue(f)),
            },
            MetadataValue::Boolean(b) => QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::BoolValue(b)),
            },
            _ => {
                return Err(QdrantError::Serialization(
                    "Unsupported metadata value type for Qdrant".to_string()
                ));
            }
        };
        Ok(qdrant_value)
    }
    
    /// Convert Qdrant payload to metadata
    fn convert_payload(payload: HashMap<String, QdrantValue>) -> Metadata {
        payload.into_iter()
            .filter_map(|(k, v)| {
                let metadata_value = match v.kind? {
                    qdrant_client::qdrant::value::Kind::StringValue(s) => MetadataValue::String(s),
                    qdrant_client::qdrant::value::Kind::IntegerValue(i) => MetadataValue::Integer(i),
                    qdrant_client::qdrant::value::Kind::DoubleValue(f) => MetadataValue::Float(f),
                    qdrant_client::qdrant::value::Kind::BoolValue(b) => MetadataValue::Boolean(b),
                    _ => return None,
                };
                Some((k, metadata_value))
            })
            .collect()
    }
    
    /// Get collection name with prefix
    fn collection_name(&self, name: &str) -> String {
        self.config.collection_name(name)
    }
}

#[async_trait]
impl VectorStorage for QdrantVectorStorage {
    type Config = QdrantConfig;
    
    #[instrument(skip(self))]
    async fn create_index(&self, config: IndexConfig) -> Result<()> {
        let collection_name = self.collection_name(&config.name);
        
        let vectors_config = VectorsConfig {
            config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                VectorParams {
                    size: config.dimension as u64,
                    distance: Self::convert_metric(config.metric).into(),
                    ..Default::default()
                }
            )),
        };
        
        let create_collection = CreateCollection {
            collection_name: collection_name.clone(),
            vectors_config: Some(vectors_config),
            ..Default::default()
        };
        
        self.client.create_collection(&create_collection).await
            .map_err(|e| QdrantError::Collection(format!("Failed to create collection: {}", e)))?;
        
        debug!("Created Qdrant collection: {}", collection_name);
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn list_indexes(&self) -> Result<Vec<String>> {
        let response = self.client.list_collections().await
            .map_err(|e| QdrantError::Collection(format!("Failed to list collections: {}", e)))?;
        
        let mut indexes = Vec::new();
        for collection in response.collections {
            let name = collection.name;
            // Remove prefix if present
            if let Some(prefix) = &self.config.collection_prefix {
                if let Some(stripped) = name.strip_prefix(&format!("{}_", prefix)) {
                    indexes.push(stripped.to_string());
                } else {
                    indexes.push(name);
                }
            } else {
                indexes.push(name);
            }
        }
        
        Ok(indexes)
    }
    
    #[instrument(skip(self))]
    async fn describe_index(&self, index_name: &str) -> Result<IndexInfo> {
        let collection_name = self.collection_name(index_name);
        
        let request = GetCollectionInfoRequest {
            collection_name: collection_name.clone(),
        };
        
        let response = self.client.collection_info(&request).await
            .map_err(|e| QdrantError::Collection(format!("Failed to get collection info: {}", e)))?;
        
        let result = response.result.ok_or_else(|| {
            QdrantError::Collection("No collection info in response".to_string())
        })?;
        
        let config = result.config.ok_or_else(|| {
            QdrantError::Collection("No config in collection info".to_string())
        })?;
        
        let vectors_config = config.params.ok_or_else(|| {
            QdrantError::Collection("No vector params in config".to_string())
        })?;
        
        let vector_params = match vectors_config.vectors_config {
            Some(qdrant_client::qdrant::vectors_config::Config::Params(params)) => params,
            _ => return Err(QdrantError::Collection("Invalid vector config".to_string()).into()),
        };
        
        let info = IndexInfo {
            name: index_name.to_string(),
            dimension: vector_params.size as usize,
            metric: Self::convert_distance(Distance::from_i32(vector_params.distance).unwrap_or(Distance::Cosine)),
            vector_count: result.vectors_count.unwrap_or(0) as usize,
            size_bytes: 0, // Qdrant doesn't provide this directly
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        };
        
        Ok(info)
    }
    
    #[instrument(skip(self))]
    async fn delete_index(&self, index_name: &str) -> Result<()> {
        let collection_name = self.collection_name(index_name);
        
        let delete_collection = DeleteCollection {
            collection_name: collection_name.clone(),
        };
        
        self.client.delete_collection(&delete_collection).await
            .map_err(|e| QdrantError::Collection(format!("Failed to delete collection: {}", e)))?;
        
        debug!("Deleted Qdrant collection: {}", collection_name);
        Ok(())
    }
    
    async fn upsert_documents(&self, index_name: &str, documents: Vec<Document>) -> Result<Vec<DocumentId>> {
        let collection_name = self.collection_name(index_name);
        let mut points = Vec::new();
        let mut ids = Vec::new();
        
        for doc in documents {
            let embedding = doc.embedding.ok_or_else(|| {
                VectorError::InvalidVector("Document must have embedding".to_string())
            })?;
            
            let id = doc.id.clone();
            ids.push(id.clone());
            
            // Convert metadata to Qdrant payload
            let mut payload = HashMap::new();
            for (key, value) in doc.metadata {
                if let Ok(qdrant_value) = Self::convert_metadata_value(value) {
                    payload.insert(key, qdrant_value);
                } else {
                    warn!("Skipping unsupported metadata value for key: {}", key);
                }
            }
            
            let point = PointStruct {
                id: Some(qdrant_client::qdrant::PointId {
                    point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(id)),
                }),
                vectors: Some(qdrant_client::qdrant::Vectors {
                    vectors_options: Some(qdrant_client::qdrant::vectors::VectorsOptions::Vector(
                        qdrant_client::qdrant::Vector { data: embedding }
                    )),
                }),
                payload,
            };
            
            points.push(point);
        }
        
        // Batch upsert
        for chunk in points.chunks(self.config.batch_size) {
            let upsert_points = UpsertPoints {
                collection_name: collection_name.clone(),
                points: chunk.to_vec(),
                ..Default::default()
            };
            
            self.client.upsert_points(&upsert_points).await
                .map_err(|e| QdrantError::Point(format!("Failed to upsert points: {}", e)))?;
        }
        
        debug!("Upserted {} documents to collection: {}", ids.len(), collection_name);
        Ok(ids)
    }
    
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        let collection_name = self.collection_name(&request.index_name);
        
        let query_vector = match request.query {
            SearchQuery::Vector(vector) => vector,
            SearchQuery::Text(_) => {
                return Err(VectorError::NotSupported(
                    "Text queries not supported by Qdrant storage".to_string()
                ));
            }
        };
        
        let filter = if let Some(condition) = request.filter {
            Some(Self::convert_filter(condition)?)
        } else {
            None
        };
        
        let search_points = SearchPoints {
            collection_name,
            vector: query_vector,
            filter,
            limit: request.top_k as u64,
            with_payload: Some(true.into()),
            with_vectors: Some(request.include_vectors),
            ..Default::default()
        };
        
        let response = self.client.search_points(&search_points).await
            .map_err(|e| QdrantError::Search(format!("Search failed: {}", e)))?;
        
        let mut results = Vec::new();
        for scored_point in response.result {
            let id = match scored_point.id.and_then(|id| id.point_id_options) {
                Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid)) => uuid,
                Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)) => num.to_string(),
                None => continue,
            };
            
            let vector = if request.include_vectors {
                scored_point.vectors.and_then(|v| match v.vectors_options {
                    Some(qdrant_client::qdrant::vectors::VectorsOptions::Vector(vec)) => Some(vec.data),
                    _ => None,
                })
            } else {
                None
            };
            
            let metadata = Self::convert_payload(scored_point.payload);
            
            let result = SearchResult::new(id, scored_point.score)
                .with_vector(vector.unwrap_or_default())
                .with_metadata(metadata);
            
            results.push(result);
        }
        
        Ok(SearchResponse::new(results))
    }
    
    async fn update_document(&self, index_name: &str, document: Document) -> Result<()> {
        // For Qdrant, update is the same as upsert
        self.upsert_documents(index_name, vec![document]).await?;
        Ok(())
    }
    
    async fn delete_documents(&self, index_name: &str, ids: Vec<DocumentId>) -> Result<()> {
        let collection_name = self.collection_name(index_name);
        
        let point_ids: Vec<_> = ids.into_iter()
            .map(|id| qdrant_client::qdrant::PointId {
                point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(id)),
            })
            .collect();
        
        let delete_points = DeletePoints {
            collection_name,
            points: Some(qdrant_client::qdrant::PointsSelector {
                points_selector_one_of: Some(
                    qdrant_client::qdrant::points_selector::PointsSelectorOneOf::Points(
                        qdrant_client::qdrant::PointsIdsList { ids: point_ids }
                    )
                ),
            }),
            ..Default::default()
        };
        
        self.client.delete_points(&delete_points).await
            .map_err(|e| QdrantError::Point(format!("Failed to delete points: {}", e)))?;
        
        Ok(())
    }
    
    async fn get_documents(&self, _index_name: &str, _ids: Vec<DocumentId>, _include_vectors: bool) -> Result<Vec<Document>> {
        // TODO: Implement get_documents for Qdrant
        Err(VectorError::NotSupported("get_documents not yet implemented for Qdrant".to_string()))
    }
    
    async fn health_check(&self) -> Result<()> {
        self.client.list_collections().await
            .map_err(|e| QdrantError::Connection(format!("Health check failed: {}", e)))?;
        Ok(())
    }
    
    fn backend_info(&self) -> BackendInfo {
        BackendInfo::new("qdrant", "1.14.0")
            .with_feature("high_performance")
            .with_feature("distributed")
            .with_feature("filtering")
            .with_feature("batch_operations")
    }
}
