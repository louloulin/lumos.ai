//! Milvus storage implementation

use std::collections::HashMap;
use async_trait::async_trait;

use lumosai_vector_core::{
    traits::{VectorStorage, BackendInfo},
    types::*,
    error::Result,
};

use crate::{
    config::MilvusConfig,
    error::{MilvusError, MilvusResult},
    client::MilvusClient,
    types::{CollectionSchema, MilvusEntity},
    utils,
};

/// Milvus vector storage implementation
pub struct MilvusStorage {
    /// Milvus client
    client: MilvusClient,
    
    /// Configuration
    config: MilvusConfig,
}

impl MilvusStorage {
    /// Create a new Milvus storage instance
    pub async fn new(config: MilvusConfig) -> MilvusResult<Self> {
        config.validate()?;
        
        let client = MilvusClient::new(config.clone()).await?;
        
        Ok(Self { client, config })
    }
    
    /// Get the client
    pub fn client(&self) -> &MilvusClient {
        &self.client
    }
    
    /// Get the configuration
    pub fn config(&self) -> &MilvusConfig {
        &self.config
    }
    
    /// Convert similarity metric to Milvus metric type
    fn similarity_to_metric_type(&self, metric: &SimilarityMetric) -> &'static str {
        match metric {
            SimilarityMetric::Cosine => "COSINE",
            SimilarityMetric::Euclidean => "L2",
            SimilarityMetric::DotProduct => "IP",
            SimilarityMetric::Manhattan => "L1",
            SimilarityMetric::Hamming => "HAMMING",
        }
    }
    
    /// Convert index type to Milvus index type
    fn index_type_to_milvus(&self, index_type: &crate::config::IndexType) -> &'static str {
        match index_type {
            crate::config::IndexType::FLAT => "FLAT",
            crate::config::IndexType::IVF_FLAT => "IVF_FLAT",
            crate::config::IndexType::IVF_SQ8 => "IVF_SQ8",
            crate::config::IndexType::IVF_PQ => "IVF_PQ",
            crate::config::IndexType::HNSW => "HNSW",
            crate::config::IndexType::ANNOY => "ANNOY",
            crate::config::IndexType::AUTOINDEX => "AUTOINDEX",
        }
    }
    
    /// Build index parameters based on index type
    fn build_index_params(&self, index_type: &crate::config::IndexType) -> serde_json::Value {
        let params = &self.config.index_config.index_params;
        
        match index_type {
            crate::config::IndexType::IVF_FLAT | crate::config::IndexType::IVF_SQ8 => {
                serde_json::json!({
                    "nlist": params.nlist.unwrap_or(1024)
                })
            }
            crate::config::IndexType::IVF_PQ => {
                serde_json::json!({
                    "nlist": params.nlist.unwrap_or(1024),
                    "m": params.m.unwrap_or(8),
                    "nbits": params.nbits.unwrap_or(8)
                })
            }
            crate::config::IndexType::HNSW => {
                serde_json::json!({
                    "M": params.hnsw_m.unwrap_or(16),
                    "efConstruction": params.ef_construction.unwrap_or(200)
                })
            }
            crate::config::IndexType::ANNOY => {
                serde_json::json!({
                    "n_trees": params.n_trees.unwrap_or(8)
                })
            }
            _ => serde_json::json!({}),
        }
    }
    
    /// Build search parameters
    fn build_search_params(&self, metric_type: &str) -> serde_json::Value {
        let params = &self.config.index_config.index_params;
        
        match self.config.index_config.default_index_type {
            crate::config::IndexType::IVF_FLAT | 
            crate::config::IndexType::IVF_SQ8 | 
            crate::config::IndexType::IVF_PQ => {
                serde_json::json!({
                    "nprobe": params.nprobe.unwrap_or(10)
                })
            }
            crate::config::IndexType::HNSW => {
                serde_json::json!({
                    "ef": params.ef.unwrap_or(64)
                })
            }
            _ => serde_json::json!({}),
        }
    }
    
    /// Convert filter condition to Milvus expression
    fn build_filter_expression(&self, filter: &FilterCondition) -> MilvusResult<String> {
        match filter {
            FilterCondition::Eq(field, value) => {
                Ok(format!("{} == {}", field, self.format_value(value)?))
            }
            FilterCondition::Ne(field, value) => {
                Ok(format!("{} != {}", field, self.format_value(value)?))
            }
            FilterCondition::Gt(field, value) => {
                Ok(format!("{} > {}", field, self.format_value(value)?))
            }
            FilterCondition::Gte(field, value) => {
                Ok(format!("{} >= {}", field, self.format_value(value)?))
            }
            FilterCondition::Lt(field, value) => {
                Ok(format!("{} < {}", field, self.format_value(value)?))
            }
            FilterCondition::Lte(field, value) => {
                Ok(format!("{} <= {}", field, self.format_value(value)?))
            }
            FilterCondition::In(field, values) => {
                let mut formatted_values = Vec::new();
                for value in values {
                    formatted_values.push(self.format_value(value)?);
                }
                let values_str = formatted_values.join(", ");
                Ok(format!("{} in [{}]", field, values_str))
            }
            FilterCondition::NotIn(field, values) => {
                let mut formatted_values = Vec::new();
                for value in values {
                    formatted_values.push(self.format_value(value)?);
                }
                let values_str = formatted_values.join(", ");
                Ok(format!("{} not in [{}]", field, values_str))
            }
            FilterCondition::Exists(field) => {
                Ok(format!("{} != null", field))
            }
            FilterCondition::NotExists(field) => {
                Ok(format!("{} == null", field))
            }
            FilterCondition::Contains(field, substring) => {
                // Milvus doesn't have direct string contains, use JSON contains for metadata
                if field == "metadata" {
                    Ok(format!("JSON_CONTAINS({}, '\"{}\"')", field, substring))
                } else {
                    Ok(format!("{} like '%{}%'", field, substring))
                }
            }
            FilterCondition::StartsWith(field, prefix) => {
                Ok(format!("{} like '{}%'", field, prefix))
            }
            FilterCondition::EndsWith(field, suffix) => {
                Ok(format!("{} like '%{}'", field, suffix))
            }
            FilterCondition::Regex(field, pattern) => {
                // Milvus doesn't support regex directly, convert to like if possible
                Ok(format!("{} like '{}'", field, pattern))
            }
            FilterCondition::And(conditions) => {
                let mut expressions = Vec::new();
                for condition in conditions {
                    expressions.push(self.build_filter_expression(condition)?);
                }
                Ok(format!("({})", expressions.join(" and ")))
            }
            FilterCondition::Or(conditions) => {
                let mut expressions = Vec::new();
                for condition in conditions {
                    expressions.push(self.build_filter_expression(condition)?);
                }
                Ok(format!("({})", expressions.join(" or ")))
            }
            FilterCondition::Not(condition) => {
                let expression = self.build_filter_expression(condition)?;
                Ok(format!("not ({})", expression))
            }
        }
    }
    
    /// Format a metadata value for Milvus expression
    fn format_value(&self, value: &MetadataValue) -> MilvusResult<String> {
        match value {
            MetadataValue::String(s) => Ok(format!("\"{}\"", s.replace('"', "\\\""))),
            MetadataValue::Integer(i) => Ok(i.to_string()),
            MetadataValue::Float(f) => Ok(f.to_string()),
            MetadataValue::Boolean(b) => Ok(b.to_string()),
            MetadataValue::Null => Ok("null".to_string()),
            MetadataValue::Array(_) => {
                Err(MilvusError::InvalidData("Array values not supported in filters".to_string()))
            }
            MetadataValue::Object(_) => {
                Err(MilvusError::InvalidData("Object values not supported in filters".to_string()))
            }
        }
    }
}

#[async_trait]
impl VectorStorage for MilvusStorage {
    type Config = MilvusConfig;
    
    async fn create_index(&self, config: IndexConfig) -> Result<()> {
        // Check if collection already exists
        if self.client.has_collection(&config.name).await.map_err(|e| lumosai_vector_core::error::VectorError::from(e))? {
            return Err(lumosai_vector_core::error::VectorError::IndexAlreadyExists(format!("Collection '{}' already exists", config.name)));
        }

        // Create collection schema
        let schema = CollectionSchema::document_schema(&config.name, config.dimension);

        // Create collection
        self.client.create_collection(schema).await.map_err(|e| lumosai_vector_core::error::VectorError::from(e))?;
        
        // Create index if auto-create is enabled
        if self.config.index_config.auto_create_index {
            let index_type = self.index_type_to_milvus(&self.config.index_config.default_index_type);
            let metric_type = self.similarity_to_metric_type(&config.metric);
            let params = self.build_index_params(&self.config.index_config.default_index_type);
            
            self.client
                .create_index(&config.name, "vector", index_type, metric_type, params)
                .await
                .map_err(|e| lumosai_vector_core::error::VectorError::from(e))?;
        }
        
        Ok(())
    }
    
    async fn list_indexes(&self) -> Result<Vec<String>> {
        let collections = self.client.list_collections().await.map_err(|e| lumosai_vector_core::error::VectorError::from(e))?;
        Ok(collections)
    }
    
    async fn describe_index(&self, index_name: &str) -> Result<IndexInfo> {
        if !self.client.has_collection(index_name).await.map_err(|e| lumosai_vector_core::error::VectorError::from(e))? {
            return Err(MilvusError::not_found(format!("Collection '{}' not found", index_name)).into());
        }
        
        let collection_info = self.client.describe_collection(index_name).await.map_err(|e| lumosai_vector_core::error::VectorError::from(e))?;
        let stats = self.client.get_collection_stats(index_name).await.map_err(|e| lumosai_vector_core::error::VectorError::from(e))?;
        
        // Extract vector dimension from schema
        let dimension = collection_info.schema.fields
            .iter()
            .find(|f| f.name == "vector")
            .and_then(|f| f.type_params.as_ref())
            .and_then(|tp| tp.dim)
            .unwrap_or(384); // Default dimension
        
        Ok(IndexInfo {
            name: index_name.to_string(),
            dimension,
            metric: SimilarityMetric::Cosine, // Default, should be stored in metadata
            vector_count: stats.row_count as usize,
            size_bytes: stats.data_size as u64,
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        })
    }
    
    async fn delete_index(&self, index_name: &str) -> Result<()> {
        if !self.client.has_collection(index_name).await.map_err(|e| lumosai_vector_core::error::VectorError::from(e))? {
            return Err(lumosai_vector_core::error::VectorError::IndexNotFound(format!("Collection '{}' not found", index_name)));
        }

        self.client.drop_collection(index_name).await.map_err(|e| lumosai_vector_core::error::VectorError::from(e))?;
        Ok(())
    }

    async fn upsert_documents(&self, index_name: &str, documents: Vec<Document>) -> Result<Vec<DocumentId>> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }

        if !self.client.has_collection(index_name).await.map_err(|e| lumosai_vector_core::error::VectorError::from(e))? {
            return Err(lumosai_vector_core::error::VectorError::IndexNotFound(format!("Collection '{}' not found", index_name)));
        }

        // Validate documents have embeddings
        for doc in &documents {
            if doc.embedding.is_none() {
                return Err(lumosai_vector_core::error::VectorError::InvalidVector(format!("Document '{}' missing embedding", doc.id)));
            }
        }

        // Convert documents to Milvus entities
        let entities = utils::documents_to_entities(&documents).map_err(|e| lumosai_vector_core::error::VectorError::from(e))?;

        // Insert entities in batches
        let batch_size = self.config.performance.batch_size;
        for chunk in entities.chunks(batch_size) {
            self.client.insert(index_name, chunk).await.map_err(|e| lumosai_vector_core::error::VectorError::from(e))?;
        }

        // Return document IDs
        Ok(documents.into_iter().map(|doc| doc.id).collect())
    }

    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        if !self.client.has_collection(&request.index_name).await.map_err(|e| lumosai_vector_core::error::VectorError::from(e))? {
            return Err(lumosai_vector_core::error::VectorError::IndexNotFound(format!("Collection '{}' not found", request.index_name)));
        }

        // Extract vector from query
        let query_vector = match &request.query {
            SearchQuery::Vector(vector) => vector.clone(),
            SearchQuery::Text(_) => {
                return Err(lumosai_vector_core::error::VectorError::OperationFailed("Text queries not supported yet".to_string()));
            }
        };

        let metric_type = "COSINE"; // Default metric
        let search_params = self.build_search_params(metric_type);

        let output_fields = if request.include_metadata {
            vec!["id".to_string(), "content".to_string(), "metadata".to_string()]
        } else {
            vec!["id".to_string()]
        };

        let filter_expr = request.filter.as_ref()
            .map(|f| self.build_filter_expression(f))
            .transpose()
            .map_err(|e| lumosai_vector_core::error::VectorError::from(e))?;

        let search_response = self.client
            .search(
                &request.index_name,
                &[query_vector],
                request.top_k,
                metric_type,
                search_params,
                &output_fields,
                filter_expr.as_deref(),
            )
            .await
            .map_err(|e| lumosai_vector_core::error::VectorError::from(e))?;

        // Convert Milvus response to SearchResponse
        let mut results = Vec::new();

        if let Some(ids) = &search_response.results.ids.str_id {
            for (i, id) in ids.data.iter().enumerate() {
                let score = search_response.results.scores.get(i).copied().unwrap_or(0.0);

                let mut result = SearchResult::new(id.clone(), score);

                if request.include_metadata {
                    result.metadata = Some(HashMap::new());
                }

                if request.include_vectors {
                    // Would need to extract vector from response
                }

                results.push(result);
            }
        }

        let total_count = results.len();
        Ok(SearchResponse::new(results)
            .with_total_count(total_count)
            .with_execution_time(0))
    }

    async fn update_document(&self, index_name: &str, document: Document) -> Result<()> {
        // For Milvus, update is the same as upsert
        self.upsert_documents(index_name, vec![document]).await?;
        Ok(())
    }

    async fn delete_documents(&self, index_name: &str, ids: Vec<DocumentId>) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        if !self.client.has_collection(index_name).await.map_err(|e| lumosai_vector_core::error::VectorError::from(e))? {
            return Err(lumosai_vector_core::error::VectorError::index_not_found(format!("Collection '{}' not found", index_name)));
        }

        // Build delete expression
        let ids_str: Vec<String> = ids.iter().map(|id| format!("\"{}\"", id)).collect();
        let delete_expr = format!("id in [{}]", ids_str.join(", "));

        // Execute delete
        self.client
            .delete(index_name, &delete_expr)
            .await
            .map_err(|e| lumosai_vector_core::error::VectorError::from(e))?;

        Ok(())
    }

    async fn get_documents(&self, index_name: &str, ids: Vec<DocumentId>, include_vectors: bool) -> Result<Vec<Document>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        if !self.client.has_collection(index_name).await.map_err(|e| lumosai_vector_core::error::VectorError::from(e))? {
            return Err(lumosai_vector_core::error::VectorError::IndexNotFound(format!("Collection '{}' not found", index_name)));
        }

        // Build query expression
        let ids_str: Vec<String> = ids.iter().map(|id| format!("\"{}\"", id)).collect();
        let query_expr = format!("id in [{}]", ids_str.join(", "));

        // Select fields based on include_vectors flag
        let output_fields = if include_vectors {
            vec!["id".to_string(), "content".to_string(), "vector".to_string(), "metadata".to_string()]
        } else {
            vec!["id".to_string(), "content".to_string(), "metadata".to_string()]
        };

        // Execute query
        let query_response = self.client
            .query(index_name, &query_expr, &output_fields, None, None)
            .await
            .map_err(|e| lumosai_vector_core::error::VectorError::from(e))?;

        // Convert response to documents
        // This is a simplified implementation - in practice, you'd need to parse the field data properly
        let mut documents = Vec::new();

        // Parse the field data to reconstruct documents
        // This would require more complex parsing of the Milvus response format
        // For now, return empty documents with IDs
        for id in &ids {
            let mut doc = Document::new(id, "");
            if !include_vectors {
                doc.embedding = None;
            }
            documents.push(doc);
        }

        Ok(documents)
    }

    async fn health_check(&self) -> Result<()> {
        let is_healthy = self.client.health_check().await.map_err(|e| lumosai_vector_core::error::VectorError::from(e))?;

        if is_healthy {
            Ok(())
        } else {
            Err(lumosai_vector_core::error::VectorError::ConnectionFailed("Milvus service is not healthy".to_string()))
        }
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo::new("milvus", "2.3.0")
            .with_feature("vector_search")
            .with_feature("metadata_filtering")
            .with_feature("batch_operations")
            .with_feature("distributed")
            .with_feature("cloud_native")
            .with_feature("multi_tenancy")
            .with_feature("acid_transactions")
            .with_metadata("endpoint", self.config.endpoint.clone())
            .with_metadata("database", self.config.database.clone())
            .with_metadata("batch_size", self.config.performance.batch_size as i64)
            .with_metadata("consistency_level", format!("{:?}", self.config.collection_config.consistency_level))
    }
}
