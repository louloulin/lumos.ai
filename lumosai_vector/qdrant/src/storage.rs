//! Qdrant vector storage implementation

use std::collections::HashMap;
use async_trait::async_trait;
use qdrant_client::{
    Qdrant,
    qdrant::{
        CreateCollection, VectorParams, VectorsConfig, Distance,
        PointStruct, Value as QdrantValue, SearchPoints,
        UpsertPoints, DeletePoints, PointsSelector,
    },
};
use uuid::Uuid;
use tracing::{debug, instrument, warn};

use lumosai_vector_core::prelude::*;
use crate::{QdrantConfig, QdrantError};
use crate::error::QdrantResult;
use crate::filter::QdrantFilterConverter;

/// Qdrant vector storage implementation
pub struct QdrantVectorStorage {
    client: Qdrant,
    config: QdrantConfig,
}

impl QdrantVectorStorage {
    /// Create a new Qdrant vector storage instance
    pub async fn new(url: &str) -> Result<Self> {
        let config = QdrantConfig::new(url);
        Self::with_config(config).await
    }

    /// Create a new Qdrant vector storage instance with configuration
    pub async fn with_config(config: QdrantConfig) -> Result<Self> {
        let mut qdrant_config = qdrant_client::config::QdrantConfig::from_url(&config.url);

        if let Some(api_key) = &config.api_key {
            qdrant_config.api_key = Some(api_key.clone());
        }

        let client = Qdrant::new(qdrant_config)
            .map_err(|e| VectorError::ConnectionFailed(e.to_string()))?;

        // Test connection
        client.list_collections().await
            .map_err(|e| VectorError::ConnectionFailed(format!("Failed to connect: {}", e)))?;

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
        
        self.client.create_collection(create_collection).await
            .map_err(|e| VectorError::OperationFailed(format!("Failed to create collection: {}", e)))?;
        
        debug!("Created Qdrant collection: {}", collection_name);
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn list_indexes(&self) -> Result<Vec<String>> {
        let response = self.client.list_collections().await
            .map_err(|e| VectorError::OperationFailed(format!("Failed to list collections: {}", e)))?;
        
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

        let response = self.client.collection_info(collection_name.clone()).await
            .map_err(|e| VectorError::OperationFailed(format!("Failed to get collection info: {}", e)))?;

        let result = response.result.ok_or_else(|| {
            VectorError::OperationFailed("No collection info in response".to_string())
        })?;

        // For now, return basic info since the API structure is complex
        let info = IndexInfo {
            name: index_name.to_string(),
            dimension: 384, // Default dimension, should be configurable
            metric: SimilarityMetric::Cosine,
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

        self.client.delete_collection(collection_name.clone()).await
            .map_err(|e| VectorError::OperationFailed(format!("Failed to delete collection: {}", e)))?;

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
                match QdrantFilterConverter::convert_metadata_to_value(value) {
                    Ok(qdrant_value) => {
                        payload.insert(key, qdrant_value);
                    },
                    Err(_) => {
                        warn!("Skipping unsupported metadata value for key: {}", key);
                    }
                }
            }
            
            let point = PointStruct {
                id: Some(qdrant_client::qdrant::PointId {
                    point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(id)),
                }),
                vectors: Some(qdrant_client::qdrant::Vectors {
                    vectors_options: Some(qdrant_client::qdrant::vectors::VectorsOptions::Vector(
                        qdrant_client::qdrant::Vector {
                            data: embedding,
                            indices: None,
                            vectors_count: None,
                            vector: None,
                        }
                    )),
                }),
                payload,
            };
            
            points.push(point);
        }
        
        // Batch upsert
        for chunk in points.chunks(self.config.batch_size) {
            // Use the builder pattern for upsert_points
            let upsert_request = qdrant_client::qdrant::UpsertPoints {
                collection_name: collection_name.clone(),
                points: chunk.to_vec(),
                ..Default::default()
            };

            self.client.upsert_points(upsert_request).await
                .map_err(|e| VectorError::OperationFailed(format!("Failed to upsert points: {}", e)))?;
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
            Some(QdrantFilterConverter::convert_filter(condition)
                .map_err(|e| VectorError::InvalidFilter(e.to_string()))?)
        } else {
            None
        };
        
        let search_points = qdrant_client::qdrant::SearchPoints {
            collection_name,
            vector: query_vector,
            filter,
            limit: request.top_k as u64,
            with_payload: Some(qdrant_client::qdrant::WithPayloadSelector {
                selector_options: Some(qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)),
            }),
            with_vectors: if request.include_vectors {
                Some(qdrant_client::qdrant::WithVectorsSelector {
                    selector_options: Some(qdrant_client::qdrant::with_vectors_selector::SelectorOptions::Enable(true)),
                })
            } else {
                None
            },
            ..Default::default()
        };

        let response = self.client.search_points(search_points).await
            .map_err(|e| VectorError::OperationFailed(format!("Search failed: {}", e)))?;
        
        let mut results = Vec::new();
        for scored_point in response.result {
            let id = match scored_point.id.and_then(|id| id.point_id_options) {
                Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid)) => uuid,
                Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)) => num.to_string(),
                None => continue,
            };
            
            let vector = if request.include_vectors {
                // For now, return None as vector extraction is complex with new API
                None
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
        
        let points_selector = qdrant_client::qdrant::PointsSelector {
            points_selector_one_of: Some(
                qdrant_client::qdrant::points_selector::PointsSelectorOneOf::Points(
                    qdrant_client::qdrant::PointsIdsList { ids: point_ids }
                )
            ),
        };

        let delete_request = qdrant_client::qdrant::DeletePoints {
            collection_name,
            points: Some(points_selector),
            ..Default::default()
        };

        self.client.delete_points(delete_request).await
            .map_err(|e| VectorError::OperationFailed(format!("Failed to delete points: {}", e)))?;
        
        Ok(())
    }
    
    async fn get_documents(&self, _index_name: &str, _ids: Vec<DocumentId>, _include_vectors: bool) -> Result<Vec<Document>> {
        // TODO: Implement get_documents for Qdrant
        Err(VectorError::NotSupported("get_documents not yet implemented for Qdrant".to_string()))
    }
    
    async fn health_check(&self) -> Result<()> {
        self.client.list_collections().await
            .map_err(|e| VectorError::ConnectionFailed(format!("Health check failed: {}", e)))?;
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
