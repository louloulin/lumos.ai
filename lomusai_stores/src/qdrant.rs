use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use qdrant_client::prelude::*;
use qdrant_client::qdrant::{
    FieldCondition as QdrantFieldCondition,
    Filter as QdrantFilter,
    PointStruct,
    SearchPoints,
    VectorParams,
    VectorsConfig,
    Distance,
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

const BATCH_SIZE: usize = 100;

/// Distance metric mapping from our format to Qdrant's
const DISTANCE_MAPPING: &[(&str, Distance)] = &[
    ("cosine", Distance::Cosine),
    ("euclidean", Distance::Euclid),
    ("dotproduct", Distance::Dot),
];

/// Qdrant vector store implementation
#[derive(Debug)]
pub struct QdrantStore {
    client: Arc<Mutex<QdrantClient>>,
    filter_translator: QdrantFilterTranslator,
}

impl QdrantStore {
    /// Create a new Qdrant vector store
    pub async fn new(url: &str, api_key: Option<&str>) -> Result<Self, StoreError> {
        let config = QdrantClientConfig::from_url(url);
        let config = if let Some(key) = api_key {
            config.with_api_key(key)
        } else {
            config
        };
        
        let client = QdrantClient::new(Some(config))
            .map_err(|e| StoreError::ConnectionError(e.to_string()))?;
            
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            filter_translator: QdrantFilterTranslator,
        })
    }
    
    /// Parse a point ID from string to Qdrant's PointId
    fn parse_point_id(&self, id: &str) -> PointId {
        if let Ok(uuid) = Uuid::parse_str(id) {
            return uuid.as_bytes().to_vec().into();
        }
        
        // Try parsing as integer
        if let Ok(num) = id.parse::<u64>() {
            return num.into();
        }
        
        // Use as string
        id.to_string().into()
    }
}

#[async_trait]
impl VectorStore for QdrantStore {
    #[instrument(skip(self))]
    async fn create_index(&self, params: CreateIndexParams) -> Result<(), StoreError> {
        let client = self.client.lock().await;
        
        let distance = DISTANCE_MAPPING
            .iter()
            .find(|(name, _)| name == &params.metric.to_lowercase())
            .map(|(_, dist)| *dist)
            .unwrap_or(Distance::Cosine);
            
        let vector_config = VectorsConfig {
            config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                VectorParams {
                    size: params.dimension as u64,
                    distance: distance.into(),
                    ..Default::default()
                }
            )),
        };
        
        client.create_collection(
            &params.index_name,
            Some(vector_config),
            None,
        ).await?;
        
        debug!("Created Qdrant collection: {}", params.index_name);
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
        
        let points = ids.iter().enumerate().map(|(i, id)| {
            let vector = params.vectors.get(i)
                .cloned()
                .unwrap_or_default();
                
            let payload = params.metadata.get(i)
                .cloned()
                .unwrap_or_default();
                
            PointStruct {
                id: Some(self.parse_point_id(id)),
                vectors: Some(vector.into()),
                payload: payload.into_iter().collect(),
            }
        }).collect::<Vec<_>>();
        
        // Batch insert points
        for chunk in points.chunks(BATCH_SIZE) {
            client.upsert_points(
                &params.index_name,
                chunk.to_vec(),
                None,
            ).await?;
        }
        
        debug!("Inserted {} vectors into Qdrant collection: {}", ids.len(), params.index_name);
        Ok(ids)
    }
    
    #[instrument(skip(self))]
    async fn query(&self, params: QueryParams) -> Result<Vec<QueryResult>, StoreError> {
        let client = self.client.lock().await;
        
        let filter = if let Some(filter) = params.filter {
            let qdrant_filter = self.filter_translator.translate_filter(Some(filter)).await?;
            Some(qdrant_filter)
        } else {
            None
        };
        
        let search_params = SearchPoints {
            collection_name: params.index_name.clone(),
            vector: params.query_vector,
            filter,
            limit: params.top_k as u64,
            with_payload: Some(true.into()),
            with_vectors: Some(params.include_vector),
            ..Default::default()
        };
        
        let results = client.search_points(&search_params).await?;
        
        let query_results = results.result.into_iter().map(|point| {
            let id = match point.id {
                Some(PointId::Num(n)) => n.to_string(),
                Some(PointId::Uuid(u)) => u.to_string(),
                Some(PointId::String(s)) => s,
                None => "unknown".to_string(),
            };
            
            let metadata = point.payload.into_iter()
                .map(|(k, v)| (k, serde_json::to_value(v).unwrap_or(Value::Null)))
                .collect();
                
            let vector = if params.include_vector {
                point.vectors.map(|v| match v {
                    qdrant_client::qdrant::Vectors::Vector(vector) => vector,
                    _ => Vec::new(),
                })
            } else {
                None
            };
            
            QueryResult {
                id,
                score: point.score,
                metadata,
                vector,
            }
        }).collect();
        
        debug!("Queried Qdrant collection: {}, found {} results", params.index_name, query_results.len());
        Ok(query_results)
    }
    
    #[instrument(skip(self))]
    async fn list_indexes(&self) -> Result<Vec<String>, StoreError> {
        let client = self.client.lock().await;
        
        let collections = client.list_collections().await?;
        let names = collections.collections
            .into_iter()
            .map(|c| c.name)
            .collect();
            
        debug!("Listed Qdrant collections");
        Ok(names)
    }
    
    #[instrument(skip(self))]
    async fn describe_index(&self, index_name: &str) -> Result<IndexStats, StoreError> {
        let client = self.client.lock().await;
        
        let collection_info = client.collection_info(index_name).await?;
        
        let config = collection_info.config
            .ok_or_else(|| StoreError::InternalError("Missing collection config".to_string()))?;
            
        let vectors_config = config.params
            .ok_or_else(|| StoreError::InternalError("Missing vector params".to_string()))?
            .vectors
            .ok_or_else(|| StoreError::InternalError("Missing vectors config".to_string()))?;
            
        let dimension = match vectors_config.config {
            Some(qdrant_client::qdrant::vectors_config::Config::Params(p)) => p.size as usize,
            _ => 0,
        };
        
        let distance = match vectors_config.config {
            Some(qdrant_client::qdrant::vectors_config::Config::Params(p)) => {
                match Distance::try_from(p.distance).unwrap_or(Distance::Cosine) {
                    Distance::Cosine => "cosine",
                    Distance::Euclid => "euclidean",
                    Distance::Dot => "dotproduct",
                    _ => "unknown",
                }
            },
            _ => "unknown",
        };
        
        let count = collection_info.points_count.unwrap_or(0) as usize;
        
        Ok(IndexStats {
            dimension,
            count,
            metric: distance.to_string(),
        })
    }
    
    #[instrument(skip(self))]
    async fn delete_index(&self, index_name: &str) -> Result<(), StoreError> {
        let client = self.client.lock().await;
        
        client.delete_collection(index_name).await?;
        
        debug!("Deleted Qdrant collection: {}", index_name);
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
        let point_id = self.parse_point_id(id);
        
        if let Some(vector) = vector {
            client.update_vectors(
                index_name,
                vec![PointVectors {
                    id: Some(point_id.clone()),
                    vectors: Some(vector.into()),
                }],
                None,
            ).await?;
        }
        
        if let Some(metadata) = metadata {
            let payload = metadata.into_iter()
                .map(|(k, v)| {
                    let payload = serde_json::from_value(v)
                        .map_err(|e| StoreError::SerializationError(e.to_string()))?;
                    Ok((k, payload))
                })
                .collect::<Result<HashMap<_, _>, StoreError>>()?;
                
            client.set_payload(
                index_name,
                &[point_id],
                &payload,
                None,
            ).await?;
        }
        
        debug!("Updated vector in Qdrant collection: {}, id: {}", index_name, id);
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn delete_vectors(&self, index_name: &str, ids: &[String]) -> Result<(), StoreError> {
        let client = self.client.lock().await;
        
        let point_ids = ids.iter()
            .map(|id| self.parse_point_id(id))
            .collect::<Vec<_>>();
            
        client.delete_points(
            index_name,
            &point_ids,
            None,
        ).await?;
        
        debug!("Deleted {} vectors from Qdrant collection: {}", ids.len(), index_name);
        Ok(())
    }
}

/// Qdrant-specific filter translator
#[derive(Debug, Default)]
pub struct QdrantFilterTranslator;

#[async_trait]
impl VectorFilterTranslator for QdrantFilterTranslator {
    async fn translate_filter<T: Send + Sync>(&self, filter: Option<VectorFilter>) -> Result<T, StoreError> {
        if filter.is_none() {
            return Err(StoreError::FilterError("No filter provided".to_string()));
        }
        
        let qdrant_filter = self.translate_vector_filter(filter.unwrap())?;
        
        serde_json::to_value(qdrant_filter)
            .map_err(|e| StoreError::SerializationError(e.to_string()))
            .and_then(|v| {
                serde_json::from_value(v)
                    .map_err(|e| StoreError::SerializationError(e.to_string()))
            })
    }
}

impl QdrantFilterTranslator {
    /// Translate our VectorFilter to Qdrant's Filter
    fn translate_vector_filter(&self, filter: VectorFilter) -> Result<QdrantFilter, StoreError> {
        match filter {
            VectorFilter::And { and } => {
                let mut conditions = Vec::new();
                for f in and {
                    conditions.push(self.translate_vector_filter(f)?);
                }
                
                Ok(QdrantFilter {
                    must: Some(conditions),
                    ..Default::default()
                })
            },
            VectorFilter::Or { or } => {
                let mut conditions = Vec::new();
                for f in or {
                    conditions.push(self.translate_vector_filter(f)?);
                }
                
                Ok(QdrantFilter {
                    should: Some(conditions),
                    ..Default::default()
                })
            },
            VectorFilter::Not { not } => {
                let inner = self.translate_vector_filter(*not)?;
                
                Ok(QdrantFilter {
                    must_not: Some(vec![inner]),
                    ..Default::default()
                })
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
                        Ok(QdrantFilter {
                            must: Some(vec![QdrantFilter {
                                field: Some(QdrantFieldCondition {
                                    key: field,
                                    r#match: Some(self.value_to_match(value)?),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }]),
                            ..Default::default()
                        })
                    },
                    crate::vector::FieldCondition::Operator(ops) => {
                        self.translate_operators(field, ops)
                    },
                }
            },
        }
    }
    
    /// Translate a field with operators to Qdrant's Filter
    fn translate_operators(&self, field: String, ops: HashMap<String, Value>) -> Result<QdrantFilter, StoreError> {
        let mut filter = QdrantFilter::default();
        
        for (op, value) in ops {
            match op.as_str() {
                "$eq" => {
                    filter.field = Some(QdrantFieldCondition {
                        key: field.clone(),
                        r#match: Some(self.value_to_match(value)?),
                        ..Default::default()
                    });
                },
                "$ne" => {
                    filter.must_not = Some(vec![QdrantFilter {
                        field: Some(QdrantFieldCondition {
                            key: field.clone(),
                            r#match: Some(self.value_to_match(value)?),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }]);
                },
                "$gt" => {
                    filter.field = Some(QdrantFieldCondition {
                        key: field.clone(),
                        range: Some(qdrant_client::qdrant::Range {
                            gt: Some(self.value_to_float(value)?),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                },
                "$gte" => {
                    filter.field = Some(QdrantFieldCondition {
                        key: field.clone(),
                        range: Some(qdrant_client::qdrant::Range {
                            gte: Some(self.value_to_float(value)?),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                },
                "$lt" => {
                    filter.field = Some(QdrantFieldCondition {
                        key: field.clone(),
                        range: Some(qdrant_client::qdrant::Range {
                            lt: Some(self.value_to_float(value)?),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                },
                "$lte" => {
                    filter.field = Some(QdrantFieldCondition {
                        key: field.clone(),
                        range: Some(qdrant_client::qdrant::Range {
                            lte: Some(self.value_to_float(value)?),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                },
                "$in" => {
                    let array = self.value_to_array(value)?;
                    filter.field = Some(QdrantFieldCondition {
                        key: field.clone(),
                        r#match: Some(qdrant_client::qdrant::Match {
                            matches: Some(qdrant_client::qdrant::r#match::Matches::OneOf(
                                qdrant_client::qdrant::RepeatedValues {
                                    values: array,
                                }
                            )),
                        }),
                        ..Default::default()
                    });
                },
                "$nin" => {
                    let array = self.value_to_array(value)?;
                    filter.must_not = Some(vec![QdrantFilter {
                        field: Some(QdrantFieldCondition {
                            key: field.clone(),
                            r#match: Some(qdrant_client::qdrant::Match {
                                matches: Some(qdrant_client::qdrant::r#match::Matches::OneOf(
                                    qdrant_client::qdrant::RepeatedValues {
                                        values: array,
                                    }
                                )),
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }]);
                },
                _ => {
                    return Err(StoreError::FilterError(format!("Unsupported operator: {}", op)));
                }
            }
        }
        
        Ok(filter)
    }
    
    /// Convert a JSON Value to a Qdrant Match
    fn value_to_match(&self, value: Value) -> Result<qdrant_client::qdrant::Match, StoreError> {
        match value {
            Value::String(s) => Ok(qdrant_client::qdrant::Match {
                matches: Some(qdrant_client::qdrant::r#match::Matches::Text(s)),
            }),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(qdrant_client::qdrant::Match {
                        matches: Some(qdrant_client::qdrant::r#match::Matches::Integer(i)),
                    })
                } else if let Some(f) = n.as_f64() {
                    Ok(qdrant_client::qdrant::Match {
                        matches: Some(qdrant_client::qdrant::r#match::Matches::Float(f)),
                    })
                } else {
                    Err(StoreError::FilterError("Invalid number format".to_string()))
                }
            },
            Value::Bool(b) => Ok(qdrant_client::qdrant::Match {
                matches: Some(qdrant_client::qdrant::r#match::Matches::Boolean(b)),
            }),
            Value::Array(a) => {
                let values = a.into_iter()
                    .map(|v| serde_json::to_value(v).unwrap_or(Value::Null))
                    .map(|v| qdrant_client::qdrant::Value {
                        kind: Some(self.json_to_qdrant_value(v)?),
                    })
                    .collect();
                    
                Ok(qdrant_client::qdrant::Match {
                    matches: Some(qdrant_client::qdrant::r#match::Matches::OneOf(
                        qdrant_client::qdrant::RepeatedValues {
                            values,
                        }
                    )),
                })
            },
            _ => Err(StoreError::FilterError("Unsupported value type".to_string())),
        }
    }
    
    /// Convert a JSON Value to a Qdrant Value kind
    fn json_to_qdrant_value(&self, value: Value) -> Result<qdrant_client::qdrant::value::Kind, StoreError> {
        match value {
            Value::Null => Ok(qdrant_client::qdrant::value::Kind::NullValue(0)),
            Value::Bool(b) => Ok(qdrant_client::qdrant::value::Kind::BoolValue(b)),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(qdrant_client::qdrant::value::Kind::IntegerValue(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(qdrant_client::qdrant::value::Kind::DoubleValue(f))
                } else {
                    Err(StoreError::FilterError("Invalid number format".to_string()))
                }
            },
            Value::String(s) => Ok(qdrant_client::qdrant::value::Kind::StringValue(s)),
            _ => Err(StoreError::FilterError("Unsupported value type".to_string())),
        }
    }
    
    /// Convert a JSON Value to a float
    fn value_to_float(&self, value: Value) -> Result<f64, StoreError> {
        match value {
            Value::Number(n) => {
                n.as_f64().ok_or_else(|| StoreError::FilterError("Invalid float format".to_string()))
            },
            Value::String(s) => {
                s.parse::<f64>().map_err(|_| StoreError::FilterError("Invalid float string".to_string()))
            },
            _ => Err(StoreError::FilterError("Value is not a number".to_string())),
        }
    }
    
    /// Convert a JSON Value to an array of Qdrant Values
    fn value_to_array(&self, value: Value) -> Result<Vec<qdrant_client::qdrant::Value>, StoreError> {
        match value {
            Value::Array(a) => {
                a.into_iter()
                    .map(|v| {
                        Ok(qdrant_client::qdrant::Value {
                            kind: Some(self.json_to_qdrant_value(v)?),
                        })
                    })
                    .collect()
            },
            _ => Err(StoreError::FilterError("Value is not an array".to_string())),
        }
    }
} 