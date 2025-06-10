//! LanceDB storage implementation

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use arrow_array::RecordBatchIterator;
use arrow_schema::Schema;
use lancedb::query::{QueryBase, ExecutableQuery};
use arrow::error::ArrowError;

use futures::TryStreamExt;

use lumosai_vector_core::{
    traits::{VectorStorage, BackendInfo},
    types::*,
    error::{Result, VectorError},
};

use crate::{
    config::LanceDbConfig,
    error::{LanceDbError, LanceDbResult},
    LanceDbClient,
    utils,
};

/// LanceDB vector storage implementation
pub struct LanceDbStorage {
    /// LanceDB client
    client: LanceDbClient,
    
    /// Configuration
    config: LanceDbConfig,
    
    /// Table schemas cache
    schemas: Arc<tokio::sync::RwLock<HashMap<String, Schema>>>,
}

impl LanceDbStorage {
    /// Create a new LanceDB storage instance
    pub async fn new(config: LanceDbConfig) -> LanceDbResult<Self> {
        config.validate()?;
        
        let client = LanceDbClient::new(config.clone()).await?;
        
        Ok(Self {
            client,
            config,
            schemas: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }
    
    /// Create a new LanceDB storage instance with default configuration
    pub async fn with_uri(uri: &str) -> LanceDbResult<Self> {
        let config = LanceDbConfig::new(uri);
        Self::new(config).await
    }
    
    /// Get the client
    pub fn client(&self) -> &LanceDbClient {
        &self.client
    }
    
    /// Get the configuration
    pub fn config(&self) -> &LanceDbConfig {
        &self.config
    }
    
    /// Get or create a table schema
    async fn get_or_create_schema(&self, index_name: &str, vector_dim: usize) -> LanceDbResult<Schema> {
        let schemas = self.schemas.read().await;
        if let Some(schema) = schemas.get(index_name) {
            return Ok(schema.clone());
        }
        drop(schemas);
        
        // Create new schema
        let schema = utils::create_document_schema(vector_dim);
        
        // Cache the schema
        let mut schemas = self.schemas.write().await;
        schemas.insert(index_name.to_string(), schema.clone());
        
        Ok(schema)
    }
    
    /// Convert filter conditions to LanceDB query filter
    fn build_filter_expression(&self, filter: &FilterCondition) -> LanceDbResult<String> {
        match filter {
            FilterCondition::Eq(field, value) => {
                Ok(format!("{} = {}", field, self.format_value(value)?))
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
                let formatted_values: std::result::Result<Vec<String>, LanceDbError> = values
                    .iter()
                    .map(|v| self.format_value(v))
                    .collect();
                let values_str = formatted_values?.join(", ");
                Ok(format!("{} IN ({})", field, values_str))
            }
            FilterCondition::NotIn(field, values) => {
                let formatted_values: std::result::Result<Vec<String>, LanceDbError> = values
                    .iter()
                    .map(|v| self.format_value(v))
                    .collect();
                let values_str = formatted_values?.join(", ");
                Ok(format!("{} NOT IN ({})", field, values_str))
            }
            FilterCondition::Exists(field) => {
                Ok(format!("{} IS NOT NULL", field))
            }
            FilterCondition::NotExists(field) => {
                Ok(format!("{} IS NULL", field))
            }
            FilterCondition::Contains(field, substring) => {
                Ok(format!("{} LIKE '%{}%'", field, substring))
            }
            FilterCondition::StartsWith(field, prefix) => {
                Ok(format!("{} LIKE '{}%'", field, prefix))
            }
            FilterCondition::EndsWith(field, suffix) => {
                Ok(format!("{} LIKE '%{}'", field, suffix))
            }
            FilterCondition::Regex(field, pattern) => {
                Ok(format!("{} REGEXP '{}'", field, pattern))
            }
            FilterCondition::And(conditions) => {
                let expressions: std::result::Result<Vec<String>, LanceDbError> = conditions
                    .iter()
                    .map(|c| self.build_filter_expression(c))
                    .collect();
                Ok(format!("({})", expressions?.join(" AND ")))
            }
            FilterCondition::Or(conditions) => {
                let expressions: std::result::Result<Vec<String>, LanceDbError> = conditions
                    .iter()
                    .map(|c| self.build_filter_expression(c))
                    .collect();
                Ok(format!("({})", expressions?.join(" OR ")))
            }
            FilterCondition::Not(condition) => {
                let expression = self.build_filter_expression(condition)?;
                Ok(format!("NOT ({})", expression))
            }
        }
    }
    
    /// Format a metadata value for SQL
    fn format_value(&self, value: &MetadataValue) -> LanceDbResult<String> {
        match value {
            MetadataValue::String(s) => Ok(format!("'{}'", s.replace('\'', "''"))),
            MetadataValue::Integer(i) => Ok(i.to_string()),
            MetadataValue::Float(f) => Ok(f.to_string()),
            MetadataValue::Boolean(b) => Ok(b.to_string()),
            MetadataValue::Array(arr) => {
                let formatted: std::result::Result<Vec<String>, LanceDbError> = arr
                    .iter()
                    .map(|v| self.format_value(v))
                    .collect();
                Ok(format!("[{}]", formatted?.join(", ")))
            }
            MetadataValue::Object(_) => {
                Err(LanceDbError::InvalidData("Object values not supported in filters".to_string()))
            }
            MetadataValue::Null => Ok("NULL".to_string()),
        }
    }
    
    /// Convert similarity metric to LanceDB distance type
    fn similarity_to_distance_type(&self, metric: &SimilarityMetric) -> &'static str {
        match metric {
            SimilarityMetric::Cosine => "cosine",
            SimilarityMetric::Euclidean => "l2",
            SimilarityMetric::DotProduct => "dot",
            SimilarityMetric::Manhattan => "l1",
            SimilarityMetric::Hamming => "hamming",
        }
    }
}

#[async_trait]
impl VectorStorage for LanceDbStorage {
    type Config = LanceDbConfig;
    
    async fn create_index(&self, config: IndexConfig) -> Result<()> {
        let db = self.client.connection();
        
        // Check if table already exists
        if self.client.table_exists(&config.name).await.map_err(VectorError::from)? {
            return Err(LanceDbError::already_exists(format!("Index '{}' already exists", config.name)).into());
        }
        
        // Create schema for the table
        let schema = self.get_or_create_schema(&config.name, config.dimension).await.map_err(VectorError::from)?;
        
        // Create empty table first
        let empty_batch = arrow::record_batch::RecordBatch::new_empty(Arc::new(schema.clone()));
        let batches = vec![empty_batch];
        
        let table = db
            .create_table(
                &config.name,
                RecordBatchIterator::new(
                    batches.into_iter().map(Ok::<_, ArrowError>),
                    Arc::new(schema)
                ),
            )
            .execute()
            .await
            .map_err(|e| LanceDbError::from(e))?;
        
        // Create index if auto-create is enabled and index type is specified
        if self.config.index_config.auto_create_index {
            match self.config.index_config.default_index_type {
                crate::config::IndexType::IVF => {
                    if let Some(num_clusters) = self.config.index_config.index_params.num_clusters {
                        table
                            .create_index(
                                &["vector"],
                                lancedb::index::Index::IvfFlat(
                                    lancedb::index::vector::IvfFlatIndexBuilder::default()
                                        .num_partitions(num_clusters as u32)
                                ),
                            )
                            .execute()
                            .await
                            .map_err(|e| LanceDbError::from(e))?;
                    }
                }
                crate::config::IndexType::IVFPQ => {
                    let mut builder = lancedb::index::vector::IvfPqIndexBuilder::default();
                    
                    if let Some(num_clusters) = self.config.index_config.index_params.num_clusters {
                        builder = builder.num_partitions(num_clusters as u32);
                    }
                    
                    if let Some(num_sub_quantizers) = self.config.index_config.index_params.num_sub_quantizers {
                        builder = builder.num_sub_vectors(num_sub_quantizers as u32);
                    }
                    
                    table
                        .create_index(&["vector"], lancedb::index::Index::IvfPq(builder))
                        .execute()
                        .await
                        .map_err(|e| LanceDbError::from(e))?;
                }
                crate::config::IndexType::HNSW => {
                    // HNSW index creation (if supported by LanceDB version)
                    // Note: This might not be available in all LanceDB versions
                    tracing::warn!("HNSW index type requested but may not be supported in current LanceDB version");
                }
                crate::config::IndexType::LSH => {
                    // LSH index creation (if supported)
                    tracing::warn!("LSH index type requested but may not be supported in current LanceDB version");
                }
                crate::config::IndexType::None => {
                    // No index creation
                }
            }
        }
        
        Ok(())
    }
    
    async fn list_indexes(&self) -> Result<Vec<String>> {
        let tables = self.client.list_tables().await.map_err(VectorError::from)?;
        Ok(tables)
    }
    
    async fn describe_index(&self, index_name: &str) -> Result<IndexInfo> {
        let db = self.client.connection();
        
        if !self.client.table_exists(index_name).await.map_err(VectorError::from)? {
            return Err(LanceDbError::not_found(format!("Index '{}' not found", index_name)).into());
        }
        
        let table = db.open_table(index_name).execute().await.map_err(|e| LanceDbError::from(e))?;
        
        // Get table statistics
        let count = table.count_rows(None).await.map_err(|e| LanceDbError::from(e))?;
        let schema = table.schema().await.map_err(|e| LanceDbError::from(e))?;
        
        // Extract vector dimension from schema
        let vector_field = schema.field_with_name("vector").map_err(|e| LanceDbError::arrow(e.to_string()))?;
        let dimension = match vector_field.data_type() {
            arrow_schema::DataType::List(_field) => {
                // For list type, we need to infer dimension from data or use a default
                // This is a limitation of the current approach
                384 // Default dimension, should be improved
            }
            _ => return Err(LanceDbError::invalid_data("Invalid vector field type").into()),
        };
        
        Ok(IndexInfo {
            name: index_name.to_string(),
            dimension,
            metric: SimilarityMetric::Cosine, // Default, should be stored in metadata
            vector_count: count,
            size_bytes: 0, // LanceDB doesn't provide direct size info
            metadata: HashMap::new(),
            created_at: None,
            updated_at: None,
        })
    }
    
    async fn delete_index(&self, index_name: &str) -> Result<()> {
        if !self.client.table_exists(index_name).await.map_err(VectorError::from)? {
            return Err(LanceDbError::not_found(format!("Index '{}' not found", index_name)).into());
        }
        
        self.client.drop_table(index_name).await.map_err(VectorError::from)?;
        
        // Remove from schema cache
        let mut schemas = self.schemas.write().await;
        schemas.remove(index_name);
        
        Ok(())
    }

    async fn upsert_documents(&self, index_name: &str, documents: Vec<Document>) -> Result<Vec<DocumentId>> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }

        let db = self.client.connection();

        if !self.client.table_exists(index_name).await.map_err(VectorError::from)? {
            return Err(LanceDbError::not_found(format!("Index '{}' not found", index_name)).into());
        }

        // Validate documents have embeddings
        for doc in &documents {
            if doc.embedding.is_none() {
                return Err(LanceDbError::invalid_data(format!("Document '{}' missing embedding", doc.id)).into());
            }
        }

        // Get vector dimension from first document
        let vector_dim = documents[0].embedding.as_ref().unwrap().len();
        let schema = self.get_or_create_schema(index_name, vector_dim).await.map_err(VectorError::from)?;

        // Convert documents to record batch
        let batch = utils::documents_to_record_batch(&documents, &schema).map_err(VectorError::from)?;

        // Open table and add data
        let table = db.open_table(index_name).execute().await.map_err(|e| LanceDbError::from(e))?;

        // Use add operation for inserting data
        table
            .add(RecordBatchIterator::new(
                vec![batch].into_iter().map(Ok::<_, ArrowError>),
                Arc::new(schema)
            ))
            .execute()
            .await
            .map_err(|e| LanceDbError::from(e))?;

        // Return document IDs
        Ok(documents.into_iter().map(|doc| doc.id).collect())
    }

    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        let db = self.client.connection();

        if !self.client.table_exists(&request.index_name).await.map_err(VectorError::from)? {
            return Err(LanceDbError::not_found(format!("Index '{}' not found", request.index_name)).into());
        }

        let table = db.open_table(&request.index_name).execute().await.map_err(|e| LanceDbError::from(e))?;

        // Build query - extract vector from SearchQuery
        let vector = match &request.query {
            SearchQuery::Vector(v) => v.clone(),
            SearchQuery::Text(_) => return Err(LanceDbError::InvalidData("Text search not supported yet".to_string()).into()),
        };

        let mut query = table
            .vector_search(vector)
            .map_err(|e| LanceDbError::from(e))?
            .limit(request.top_k);

        // Add filter if provided
        if let Some(filter) = &request.filter {
            let filter_expr = self.build_filter_expression(filter).map_err(VectorError::from)?;
            query = query.only_if(&filter_expr);
        }

        // Note: Similarity metric configuration would be set during index creation
        // LanceDB 0.20.0 doesn't support runtime metric changes in vector search

        // Execute query
        let mut results = query.execute().await.map_err(|e| LanceDbError::from(e))?;

        // Collect all batches from the stream
        let mut all_batches = Vec::new();
        while let Some(batch) = results.try_next().await.map_err(|e| LanceDbError::from(e))? {
            all_batches.push(batch);
        }

        // Convert batches to documents
        let mut documents = Vec::new();
        for batch in all_batches {
            let batch_docs = utils::record_batch_to_documents(&batch).map_err(VectorError::from)?;
            documents.extend(batch_docs);
        }

        // Create search results with scores
        // Note: LanceDB should provide scores, but the exact API may vary
        let search_results: Vec<SearchResult> = documents
            .into_iter()
            .enumerate()
            .map(|(i, doc)| SearchResult {
                id: doc.id,
                vector: if request.include_vectors { doc.embedding } else { None },
                content: Some(doc.content),
                score: 1.0 - (i as f32 * 0.01), // Placeholder scoring
                metadata: Some(doc.metadata),
            })
            .collect();

        Ok(SearchResponse {
            results: search_results,
            total_count: None,
            execution_time_ms: None,
            metadata: HashMap::new(),
        })
    }

    async fn update_document(&self, index_name: &str, document: Document) -> Result<()> {
        // For LanceDB, update is the same as upsert
        self.upsert_documents(index_name, vec![document]).await?;
        Ok(())
    }

    async fn delete_documents(&self, index_name: &str, ids: Vec<DocumentId>) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let db = self.client.connection();

        if !self.client.table_exists(index_name).await.map_err(VectorError::from)? {
            return Err(LanceDbError::not_found(format!("Index '{}' not found", index_name)).into());
        }

        let table = db.open_table(index_name).execute().await.map_err(|e| LanceDbError::from(e))?;

        // Build delete condition
        let ids_str: Vec<String> = ids.iter().map(|id| format!("'{}'", id)).collect();
        let delete_condition = format!("id IN ({})", ids_str.join(", "));

        // Execute delete
        table
            .delete(&delete_condition)
            .await
            .map_err(|e| LanceDbError::from(e))?;

        Ok(())
    }

    async fn get_documents(&self, index_name: &str, ids: Vec<DocumentId>, include_vectors: bool) -> Result<Vec<Document>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let db = self.client.connection();

        if !self.client.table_exists(index_name).await.map_err(VectorError::from)? {
            return Err(LanceDbError::not_found(format!("Index '{}' not found", index_name)).into());
        }

        let table = db.open_table(index_name).execute().await.map_err(|e| LanceDbError::from(e))?;

        // Build query condition
        let ids_str: Vec<String> = ids.iter().map(|id| format!("'{}'", id)).collect();
        let condition = format!("id IN ({})", ids_str.join(", "));

        // Select columns based on include_vectors flag
        let columns = if include_vectors {
            vec!["id".to_string(), "content".to_string(), "vector".to_string(), "metadata".to_string()]
        } else {
            vec!["id".to_string(), "content".to_string(), "metadata".to_string()]
        };

        // Execute query
        let mut results = table
            .query()
            .only_if(&condition)
            .select(lancedb::query::Select::Columns(columns))
            .execute()
            .await
            .map_err(|e| LanceDbError::from(e))?;

        // Collect all batches from the stream
        let mut all_batches = Vec::new();
        while let Some(batch) = results.try_next().await.map_err(|e| LanceDbError::from(e))? {
            all_batches.push(batch);
        }

        // Convert batches to documents
        let mut documents = Vec::new();
        for batch in all_batches {
            let batch_docs = utils::record_batch_to_documents(&batch).map_err(VectorError::from)?;
            documents.extend(batch_docs);
        }

        // Remove embeddings if not requested
        if !include_vectors {
            for doc in &mut documents {
                doc.embedding = None;
            }
        }

        Ok(documents)
    }

    async fn health_check(&self) -> Result<()> {
        // Try to list tables to check connection
        self.client.list_tables().await.map_err(VectorError::from)?;
        Ok(())
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo::new("lancedb", "0.8.0")
            .with_feature("vector_search")
            .with_feature("metadata_filtering")
            .with_feature("batch_operations")
            .with_feature("transactions")
            .with_feature("versioning")
            .with_feature("compression")
            .with_feature("cloud_storage")
            .with_metadata("uri", self.config.uri.clone())
            .with_metadata("batch_size", self.config.performance.batch_size as i64)
            .with_metadata("compression_enabled", self.config.performance.enable_compression)
    }
}
