//! Weaviate vector storage implementation

use std::collections::HashMap;
use std::time::Duration;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use tracing::{debug, instrument, warn};
use uuid::Uuid;

use lumosai_vector_core::prelude::*;
use crate::{WeaviateConfig, WeaviateError};
use crate::error::WeaviateResult;
use crate::schema::{WeaviateClass, WeaviateProperty};
use crate::filter::convert_filter_to_where;

/// Weaviate vector storage implementation
pub struct WeaviateVectorStorage {
    client: Client,
    config: WeaviateConfig,
    base_url: String,
}

impl WeaviateVectorStorage {
    /// Create a new Weaviate vector storage instance
    pub async fn new(url: &str) -> Result<Self> {
        let config = WeaviateConfig::new(url);
        Self::with_config(config).await
    }

    /// Create a new Weaviate vector storage instance with configuration
    pub async fn with_config(config: WeaviateConfig) -> Result<Self> {
        config.validate().map_err(VectorError::from)?;
        
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds));
        
        // Add authentication headers if provided
        let mut default_headers = reqwest::header::HeaderMap::new();
        
        if let Some(api_key) = &config.api_key {
            default_headers.insert(
                "Authorization",
                format!("Bearer {}", api_key).parse()
                    .map_err(|e| VectorError::InvalidConfig(format!("Invalid API key: {}", e)))?
            );
        }
        
        if let Some(token) = &config.oidc_token {
            default_headers.insert(
                "Authorization",
                format!("Bearer {}", token).parse()
                    .map_err(|e| VectorError::InvalidConfig(format!("Invalid OIDC token: {}", e)))?
            );
        }
        
        default_headers.insert("Content-Type", "application/json".parse().unwrap());
        
        let client = client_builder
            .default_headers(default_headers)
            .build()
            .map_err(|e| VectorError::ConnectionFailed(format!("Failed to create HTTP client: {}", e)))?;
        
        let base_url = config.api_url();
        
        // Test connection
        let health_url = format!("{}/meta", base_url);
        client.get(&health_url).send().await
            .map_err(|e| VectorError::ConnectionFailed(format!("Failed to connect to Weaviate: {}", e)))?;
        
        Ok(Self {
            client,
            config,
            base_url,
        })
    }
    
    /// Convert similarity metric to Weaviate distance
    fn convert_metric(metric: SimilarityMetric) -> &'static str {
        match metric {
            SimilarityMetric::Cosine => "cosine",
            SimilarityMetric::Euclidean => "l2-squared",
            SimilarityMetric::DotProduct => "dot",
            _ => "cosine", // Default fallback
        }
    }
    
    /// Convert Weaviate distance to similarity metric
    fn convert_distance(distance: &str) -> SimilarityMetric {
        match distance {
            "cosine" => SimilarityMetric::Cosine,
            "l2-squared" => SimilarityMetric::Euclidean,
            "dot" => SimilarityMetric::DotProduct,
            _ => SimilarityMetric::Cosine,
        }
    }
    
    /// Get class name with prefix
    fn class_name(&self, name: &str) -> String {
        self.config.class_name(name)
    }
    
    /// Check if a class exists
    async fn class_exists(&self, class_name: &str) -> WeaviateResult<bool> {
        let url = format!("{}/schema/{}", self.base_url, class_name);
        let response = self.client.get(&url).send().await?;
        
        Ok(response.status().is_success())
    }
    
    /// Create a Weaviate class (index)
    async fn create_class(&self, config: &IndexConfig) -> WeaviateResult<()> {
        let class_name = self.class_name(&config.name);
        
        let class_def = WeaviateClass {
            class: class_name.clone(),
            description: Some(format!("Vector index for {}", config.name)),
            vector_index_type: "hnsw".to_string(),
            vector_index_config: json!({
                "distance": Self::convert_metric(config.metric),
                "efConstruction": 128,
                "maxConnections": 64
            }),
            vectorizer: self.config.vectorizer.clone().unwrap_or_else(|| "none".to_string()),
            properties: vec![
                WeaviateProperty {
                    name: "content".to_string(),
                    data_type: vec!["text".to_string()],
                    description: Some("Document content".to_string()),
                    index: Some(true),
                },
                WeaviateProperty {
                    name: "metadata".to_string(),
                    data_type: vec!["object".to_string()],
                    description: Some("Document metadata".to_string()),
                    index: Some(false),
                },
            ],
        };
        
        let url = format!("{}/schema", self.base_url);
        let response = self.client
            .post(&url)
            .json(&class_def)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(WeaviateError::Api(format!("Failed to create class: {}", error_text)));
        }
        
        debug!("Created Weaviate class: {}", class_name);
        Ok(())
    }
    
    /// Delete a Weaviate class
    async fn delete_class(&self, class_name: &str) -> WeaviateResult<()> {
        let url = format!("{}/schema/{}", self.base_url, class_name);
        let response = self.client.delete(&url).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(WeaviateError::Api(format!("Failed to delete class: {}", error_text)));
        }
        
        debug!("Deleted Weaviate class: {}", class_name);
        Ok(())
    }
    
    /// Get class information
    async fn get_class_info(&self, class_name: &str) -> WeaviateResult<Value> {
        let url = format!("{}/schema/{}", self.base_url, class_name);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(WeaviateError::ClassNotFound(class_name.to_string()));
        }
        
        let class_info: Value = response.json().await?;
        Ok(class_info)
    }
    
    /// List all classes
    async fn list_classes(&self) -> WeaviateResult<Vec<String>> {
        let url = format!("{}/schema", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(WeaviateError::Api(format!("Failed to list classes: {}", error_text)));
        }
        
        let schema: Value = response.json().await?;
        let classes = schema["classes"].as_array()
            .ok_or_else(|| WeaviateError::Api("Invalid schema response".to_string()))?;
        
        let mut class_names = Vec::new();
        for class in classes {
            if let Some(name) = class["class"].as_str() {
                // Remove prefix if present
                if let Some(prefix) = &self.config.class_prefix {
                    if let Some(stripped) = name.strip_prefix(&format!("{}_", prefix)) {
                        class_names.push(stripped.to_string());
                    } else {
                        class_names.push(name.to_string());
                    }
                } else {
                    class_names.push(name.to_string());
                }
            }
        }
        
        Ok(class_names)
    }
}

#[async_trait]
impl VectorStorage for WeaviateVectorStorage {
    type Config = WeaviateConfig;

    #[instrument(skip(self))]
    async fn create_index(&self, config: IndexConfig) -> Result<()> {
        let class_name = self.class_name(&config.name);

        // Check if class already exists
        if self.class_exists(&class_name).await.map_err(VectorError::from)? {
            return Err(VectorError::IndexAlreadyExists(config.name));
        }

        // Create the class
        self.create_class(&config).await.map_err(VectorError::from)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn list_indexes(&self) -> Result<Vec<String>> {
        self.list_classes().await.map_err(VectorError::from)
    }

    #[instrument(skip(self))]
    async fn describe_index(&self, index_name: &str) -> Result<IndexInfo> {
        let class_name = self.class_name(index_name);
        let class_info = self.get_class_info(&class_name).await.map_err(VectorError::from)?;

        // Extract information from class definition
        let vector_config = &class_info["vectorIndexConfig"];
        let distance = vector_config["distance"].as_str().unwrap_or("cosine");
        let metric = Self::convert_distance(distance);

        // Get vector count (this requires a separate query)
        let count_url = format!("{}/graphql", self.base_url);
        let count_query = json!({
            "query": format!(
                "{{ Aggregate {{ {} {{ meta {{ count }} }} }} }}",
                class_name
            )
        });

        let count_response = self.client
            .post(&count_url)
            .json(&count_query)
            .send()
            .await
            .map_err(|e| VectorError::OperationFailed(format!("Failed to get count: {}", e)))?;

        let count_data: Value = count_response.json().await
            .map_err(|e| VectorError::OperationFailed(format!("Failed to parse count response: {}", e)))?;

        let vector_count = count_data["data"]["Aggregate"][class_name][0]["meta"]["count"]
            .as_u64()
            .unwrap_or(0) as usize;

        let info = IndexInfo {
            name: index_name.to_string(),
            dimension: 0, // Weaviate doesn't expose this directly
            metric,
            vector_count,
            size_bytes: 0, // Not available
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        };

        Ok(info)
    }

    #[instrument(skip(self))]
    async fn delete_index(&self, index_name: &str) -> Result<()> {
        let class_name = self.class_name(index_name);
        self.delete_class(&class_name).await.map_err(VectorError::from)
    }

    async fn upsert_documents(&self, index_name: &str, documents: Vec<Document>) -> Result<Vec<DocumentId>> {
        let class_name = self.class_name(index_name);
        let mut ids = Vec::new();

        // Process documents in batches
        for chunk in documents.chunks(self.config.batch_size) {
            let mut objects = Vec::new();

            for doc in chunk {
                let id = doc.id.clone();
                ids.push(id.clone());

                let mut properties = json!({
                    "content": doc.content,
                    "metadata": doc.metadata
                });

                let object = json!({
                    "class": class_name,
                    "id": id,
                    "properties": properties,
                    "vector": doc.embedding
                });

                objects.push(object);
            }

            // Batch insert
            let batch_url = format!("{}/batch/objects", self.base_url);
            let batch_request = json!({
                "objects": objects
            });

            let response = self.client
                .post(&batch_url)
                .json(&batch_request)
                .send()
                .await
                .map_err(|e| VectorError::OperationFailed(format!("Batch upsert failed: {}", e)))?;

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(VectorError::OperationFailed(format!("Batch upsert failed: {}", error_text)));
            }
        }

        debug!("Upserted {} documents to class: {}", ids.len(), class_name);
        Ok(ids)
    }

    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        let class_name = self.class_name(&request.index_name);

        let query_vector = match request.query {
            SearchQuery::Vector(vector) => vector,
            SearchQuery::Text(_) => {
                return Err(VectorError::NotSupported(
                    "Text queries not supported by Weaviate storage".to_string()
                ));
            }
        };

        // Build GraphQL query
        let mut query_parts = vec![
            format!("nearVector: {{ vector: {:?} }}", query_vector),
        ];

        if let Some(filter) = request.filter {
            let where_clause = convert_filter_to_where(filter)?;
            query_parts.push(format!("where: {}", where_clause));
        }

        let fields = if request.include_vectors {
            "_additional { id score vector } content metadata"
        } else {
            "_additional { id score } content metadata"
        };

        let graphql_query = format!(
            "{{ Get {{ {} ({}, limit: {}) {{ {} }} }} }}",
            class_name,
            query_parts.join(", "),
            request.top_k,
            fields
        );

        let query_request = json!({
            "query": graphql_query
        });

        let url = format!("{}/graphql", self.base_url);
        let response = self.client
            .post(&url)
            .json(&query_request)
            .send()
            .await
            .map_err(|e| VectorError::OperationFailed(format!("Search failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(VectorError::OperationFailed(format!("Search failed: {}", error_text)));
        }

        let response_data: Value = response.json().await
            .map_err(|e| VectorError::OperationFailed(format!("Failed to parse search response: {}", e)))?;

        // Parse results
        let mut results = Vec::new();
        if let Some(objects) = response_data["data"]["Get"][&class_name].as_array() {
            for obj in objects {
                let additional = &obj["_additional"];
                let id = additional["id"].as_str().unwrap_or_default().to_string();
                let score = additional["score"].as_f64().unwrap_or(0.0) as f32;

                let vector = if request.include_vectors {
                    additional["vector"].as_array()
                        .map(|v| v.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                // Extract metadata
                let mut metadata = Metadata::new();
                if let Some(meta_obj) = obj["metadata"].as_object() {
                    for (key, value) in meta_obj {
                        if let Some(str_val) = value.as_str() {
                            metadata.insert(key.clone(), MetadataValue::String(str_val.to_string()));
                        } else if let Some(num_val) = value.as_i64() {
                            metadata.insert(key.clone(), MetadataValue::Integer(num_val));
                        } else if let Some(float_val) = value.as_f64() {
                            metadata.insert(key.clone(), MetadataValue::Float(float_val));
                        } else if let Some(bool_val) = value.as_bool() {
                            metadata.insert(key.clone(), MetadataValue::Boolean(bool_val));
                        }
                    }
                }

                let result = SearchResult::new(id, score)
                    .with_vector(vector)
                    .with_metadata(metadata);

                results.push(result);
            }
        }

        Ok(SearchResponse::new(results))
    }

    async fn update_document(&self, index_name: &str, document: Document) -> Result<()> {
        // For Weaviate, update is the same as upsert
        self.upsert_documents(index_name, vec![document]).await?;
        Ok(())
    }

    async fn delete_documents(&self, index_name: &str, ids: Vec<DocumentId>) -> Result<()> {
        let class_name = self.class_name(index_name);

        for id in ids {
            let url = format!("{}/objects/{}/{}", self.base_url, class_name, id);
            let response = self.client.delete(&url).send().await
                .map_err(|e| VectorError::OperationFailed(format!("Failed to delete document: {}", e)))?;

            if !response.status().is_success() && response.status() != reqwest::StatusCode::NOT_FOUND {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                warn!("Failed to delete document {}: {}", id, error_text);
            }
        }

        Ok(())
    }

    async fn get_documents(&self, index_name: &str, ids: Vec<DocumentId>, include_vectors: bool) -> Result<Vec<Document>> {
        let class_name = self.class_name(index_name);
        let mut documents = Vec::new();

        for id in ids {
            let fields = if include_vectors {
                "_additional { vector } content metadata"
            } else {
                "content metadata"
            };

            let url = format!("{}/objects/{}/{}?include={}", self.base_url, class_name, id, fields);
            let response = self.client.get(&url).send().await
                .map_err(|e| VectorError::OperationFailed(format!("Failed to get document: {}", e)))?;

            if response.status() == reqwest::StatusCode::NOT_FOUND {
                continue; // Skip missing documents
            }

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(VectorError::OperationFailed(format!("Failed to get document: {}", error_text)));
            }

            let obj: Value = response.json().await
                .map_err(|e| VectorError::OperationFailed(format!("Failed to parse document: {}", e)))?;

            let content = obj["properties"]["content"].as_str().map(|s| s.to_string());
            let embedding = if include_vectors {
                obj["_additional"]["vector"].as_array()
                    .map(|v| v.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect())
            } else {
                None
            };

            // Extract metadata
            let mut metadata = Metadata::new();
            if let Some(meta_obj) = obj["properties"]["metadata"].as_object() {
                for (key, value) in meta_obj {
                    if let Some(str_val) = value.as_str() {
                        metadata.insert(key.clone(), MetadataValue::String(str_val.to_string()));
                    } else if let Some(num_val) = value.as_i64() {
                        metadata.insert(key.clone(), MetadataValue::Integer(num_val));
                    } else if let Some(float_val) = value.as_f64() {
                        metadata.insert(key.clone(), MetadataValue::Float(float_val));
                    } else if let Some(bool_val) = value.as_bool() {
                        metadata.insert(key.clone(), MetadataValue::Boolean(bool_val));
                    }
                }
            }

            let mut document = Document::new(id, content.unwrap_or_default());

            // Add metadata individually
            for (key, value) in metadata {
                document = document.with_metadata(key, value);
            }

            if let Some(emb) = embedding {
                document = document.with_embedding(emb);
            }

            documents.push(document);
        }

        Ok(documents)
    }

    async fn health_check(&self) -> Result<()> {
        let url = format!("{}/meta", self.base_url);
        self.client.get(&url).send().await
            .map_err(|e| VectorError::ConnectionFailed(format!("Health check failed: {}", e)))?;
        Ok(())
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo::new("weaviate", "1.0.0")
            .with_feature("semantic_search")
            .with_feature("schema_management")
            .with_feature("graphql")
            .with_feature("batch_operations")
    }
}
