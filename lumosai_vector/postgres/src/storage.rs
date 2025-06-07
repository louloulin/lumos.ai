//! PostgreSQL vector storage implementation

use std::collections::HashMap;
use async_trait::async_trait;
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use serde_json::Value as JsonValue;
use tracing::{debug, instrument, warn};

use lumosai_vector_core::prelude::*;
use crate::{PostgresConfig, PostgresError, PostgresResult};

/// PostgreSQL vector storage implementation using pgvector
pub struct PostgresVectorStorage {
    pool: PgPool,
    config: PostgresConfig,
}

impl PostgresVectorStorage {
    /// Create a new PostgreSQL vector storage instance
    pub async fn new(database_url: &str) -> Result<Self> {
        let config = PostgresConfig::new(database_url);
        Self::with_config(config).await
    }
    
    /// Create a new PostgreSQL vector storage instance with configuration
    pub async fn with_config(config: PostgresConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.pool.max_connections)
            .min_connections(config.pool.min_connections)
            .acquire_timeout(config.pool.connect_timeout)
            .idle_timeout(config.pool.idle_timeout)
            .max_lifetime(config.pool.max_lifetime)
            .connect(&config.database_url)
            .await
            .map_err(PostgresError::from)?;
        
        let storage = Self { pool, config };
        
        // Check pgvector extension
        storage.ensure_pgvector_extension().await?;
        
        Ok(storage)
    }
    
    /// Ensure pgvector extension is installed
    async fn ensure_pgvector_extension(&self) -> PostgresResult<()> {
        let result = sqlx::query("SELECT 1 FROM pg_extension WHERE extname = 'vector'")
            .fetch_optional(&self.pool)
            .await?;
        
        if result.is_none() {
            return Err(crate::error::pgvector_extension_error());
        }
        
        Ok(())
    }
    
    /// Create table for an index if it doesn't exist
    async fn ensure_table(&self, index_name: &str, dimension: usize) -> PostgresResult<()> {
        let table_name = self.config.table_name(index_name);
        
        let create_table_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                content TEXT,
                embedding vector({}),
                metadata JSONB DEFAULT '{{}}',
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW()
            )
            "#,
            table_name, dimension
        );
        
        sqlx::query(&create_table_sql)
            .execute(&self.pool)
            .await?;
        
        // Create updated_at trigger
        let trigger_sql = format!(
            r#"
            CREATE OR REPLACE FUNCTION update_updated_at_column()
            RETURNS TRIGGER AS $$
            BEGIN
                NEW.updated_at = NOW();
                RETURN NEW;
            END;
            $$ language 'plpgsql';
            
            DROP TRIGGER IF EXISTS update_{}_updated_at ON {};
            CREATE TRIGGER update_{}_updated_at
                BEFORE UPDATE ON {}
                FOR EACH ROW
                EXECUTE FUNCTION update_updated_at_column();
            "#,
            index_name, table_name, index_name, table_name
        );
        
        sqlx::query(&trigger_sql)
            .execute(&self.pool)
            .await?;
        
        debug!("Ensured table exists: {}", table_name);
        Ok(())
    }
    
    /// Create vector index if configured
    async fn ensure_vector_index(&self, index_name: &str) -> PostgresResult<()> {
        if !self.config.table.auto_create_indexes {
            return Ok(());
        }
        
        let table_name = self.config.table_name(index_name);
        let idx_name = self.config.index_name(index_name, "embedding");
        
        // Check if index already exists
        let exists = sqlx::query(
            "SELECT 1 FROM pg_indexes WHERE tablename = $1 AND indexname = $2"
        )
        .bind(format!("{}{}", self.config.table.table_prefix.as_deref().unwrap_or(""), index_name))
        .bind(&idx_name)
        .fetch_optional(&self.pool)
        .await?;
        
        if exists.is_some() {
            return Ok(());
        }
        
        let index_sql = self.config.performance.index_type
            .create_index_sql(&table_name, &idx_name, &self.config.performance.index_params);
        
        if !index_sql.is_empty() {
            sqlx::query(&index_sql)
                .execute(&self.pool)
                .await
                .map_err(|e| crate::error::index_creation_error(&idx_name, &e.to_string()))?;
            
            debug!("Created vector index: {}", idx_name);
        }
        
        Ok(())
    }
    
    /// Convert similarity metric to PostgreSQL operator
    fn similarity_operator(metric: SimilarityMetric) -> &'static str {
        match metric {
            SimilarityMetric::Cosine => "<=>",
            SimilarityMetric::Euclidean => "<->",
            SimilarityMetric::DotProduct => "<#>",
            _ => "<=>", // Default to cosine
        }
    }
    
    /// Convert metadata to JSONB
    fn metadata_to_jsonb(metadata: &Metadata) -> PostgresResult<JsonValue> {
        let mut json_map = serde_json::Map::new();
        
        for (key, value) in metadata {
            let json_value = match value {
                MetadataValue::String(s) => JsonValue::String(s.clone()),
                MetadataValue::Integer(i) => JsonValue::Number((*i).into()),
                MetadataValue::Float(f) => {
                    JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into()))
                },
                MetadataValue::Boolean(b) => JsonValue::Bool(*b),
                MetadataValue::Array(arr) => {
                    let json_arr: std::result::Result<Vec<_>, PostgresError> = arr.iter()
                        .map(|v| Self::metadata_value_to_json(v))
                        .collect();
                    JsonValue::Array(json_arr?)
                },
                MetadataValue::Object(obj) => {
                    let mut json_obj = serde_json::Map::new();
                    for (k, v) in obj {
                        json_obj.insert(k.clone(), Self::metadata_value_to_json(v)?);
                    }
                    JsonValue::Object(json_obj)
                },
                MetadataValue::Null => JsonValue::Null,
            };
            json_map.insert(key.clone(), json_value);
        }
        
        Ok(JsonValue::Object(json_map))
    }
    
    /// Convert single metadata value to JSON
    fn metadata_value_to_json(value: &MetadataValue) -> PostgresResult<JsonValue> {
        match value {
            MetadataValue::String(s) => Ok(JsonValue::String(s.clone())),
            MetadataValue::Integer(i) => Ok(JsonValue::Number((*i).into())),
            MetadataValue::Float(f) => {
                Ok(JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into())))
            },
            MetadataValue::Boolean(b) => Ok(JsonValue::Bool(*b)),
            MetadataValue::Array(arr) => {
                let json_arr: std::result::Result<Vec<_>, PostgresError> = arr.iter()
                    .map(Self::metadata_value_to_json)
                    .collect();
                Ok(JsonValue::Array(json_arr?))
            },
            MetadataValue::Object(obj) => {
                let mut json_obj = serde_json::Map::new();
                for (k, v) in obj {
                    json_obj.insert(k.clone(), Self::metadata_value_to_json(v)?);
                }
                Ok(JsonValue::Object(json_obj))
            },
            MetadataValue::Null => Ok(JsonValue::Null),
        }
    }
    
    /// Convert JSONB to metadata
    fn jsonb_to_metadata(json: JsonValue) -> Metadata {
        match json {
            JsonValue::Object(map) => {
                map.into_iter()
                    .filter_map(|(k, v)| {
                        Self::json_value_to_metadata_value(v).map(|mv| (k, mv))
                    })
                    .collect()
            },
            _ => HashMap::new(),
        }
    }
    
    /// Convert JSON value to metadata value
    fn json_value_to_metadata_value(value: JsonValue) -> Option<MetadataValue> {
        match value {
            JsonValue::String(s) => Some(MetadataValue::String(s)),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Some(MetadataValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Some(MetadataValue::Float(f))
                } else {
                    None
                }
            },
            JsonValue::Bool(b) => Some(MetadataValue::Boolean(b)),
            JsonValue::Array(arr) => {
                let metadata_arr: Option<Vec<_>> = arr.into_iter()
                    .map(Self::json_value_to_metadata_value)
                    .collect();
                metadata_arr.map(MetadataValue::Array)
            },
            JsonValue::Object(obj) => {
                let metadata_obj: Option<HashMap<_, _>> = obj.into_iter()
                    .map(|(k, v)| Self::json_value_to_metadata_value(v).map(|mv| (k, mv)))
                    .collect();
                metadata_obj.map(MetadataValue::Object)
            },
            JsonValue::Null => Some(MetadataValue::Null),
        }
    }
    
    /// Set search parameters for the current session
    async fn set_search_params(&self) -> PostgresResult<()> {
        let params = self.config.performance.index_type
            .search_params_sql(&self.config.performance.index_params);

        for param_sql in params {
            sqlx::query(&param_sql)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl VectorStorage for PostgresVectorStorage {
    type Config = PostgresConfig;

    #[instrument(skip(self))]
    async fn create_index(&self, config: IndexConfig) -> Result<()> {
        self.ensure_table(&config.name, config.dimension).await?;
        self.ensure_vector_index(&config.name).await?;

        debug!("Created PostgreSQL index: {}", config.name);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn list_indexes(&self) -> Result<Vec<String>> {
        let prefix = self.config.table.table_prefix.as_deref().unwrap_or("");
        let schema = &self.config.table.schema;

        let query = format!(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = $1 AND table_name LIKE $2"
        );

        let rows = sqlx::query(&query)
            .bind(schema)
            .bind(format!("{}%", prefix))
            .fetch_all(&self.pool)
            .await
            .map_err(PostgresError::from)?;

        let mut indexes = Vec::new();
        for row in rows {
            let table_name: String = row.try_get("table_name").map_err(PostgresError::from)?;
            if let Some(stripped) = table_name.strip_prefix(prefix) {
                indexes.push(stripped.to_string());
            } else {
                indexes.push(table_name);
            }
        }

        Ok(indexes)
    }

    #[instrument(skip(self))]
    async fn describe_index(&self, index_name: &str) -> Result<IndexInfo> {
        let table_name = self.config.table_name(index_name);

        // Get table info
        let table_info = sqlx::query(
            r#"
            SELECT
                column_name,
                data_type,
                character_maximum_length
            FROM information_schema.columns
            WHERE table_schema = $1 AND table_name = $2 AND column_name = 'embedding'
            "#
        )
        .bind(&self.config.table.schema)
        .bind(format!("{}{}", self.config.table.table_prefix.as_deref().unwrap_or(""), index_name))
        .fetch_optional(&self.pool)
        .await
        .map_err(PostgresError::from)?;

        let dimension = if let Some(row) = table_info {
            // Extract dimension from vector type
            let data_type: String = row.try_get("data_type").map_err(PostgresError::from)?;
            if data_type.contains("vector") {
                // Parse dimension from vector(n) format
                384 // Default for now, would need to parse from type
            } else {
                return Err(VectorError::index_not_found(index_name));
            }
        } else {
            return Err(VectorError::index_not_found(index_name));
        };

        // Get row count
        let count_query = format!("SELECT COUNT(*) as count FROM {}", table_name);
        let count_row = sqlx::query(&count_query)
            .fetch_one(&self.pool)
            .await
            .map_err(PostgresError::from)?;
        let vector_count: i64 = count_row.try_get("count").map_err(PostgresError::from)?;

        let info = IndexInfo {
            name: index_name.to_string(),
            dimension,
            metric: SimilarityMetric::Cosine, // Default, could be stored in metadata
            vector_count: vector_count as usize,
            size_bytes: 0, // Would need to calculate
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        };

        Ok(info)
    }

    #[instrument(skip(self))]
    async fn delete_index(&self, index_name: &str) -> Result<()> {
        let table_name = self.config.table_name(index_name);

        let drop_sql = format!("DROP TABLE IF EXISTS {} CASCADE", table_name);
        sqlx::query(&drop_sql)
            .execute(&self.pool)
            .await
            .map_err(PostgresError::from)?;

        debug!("Deleted PostgreSQL table: {}", table_name);
        Ok(())
    }

    async fn upsert_documents(&self, index_name: &str, documents: Vec<Document>) -> Result<Vec<DocumentId>> {
        let table_name = self.config.table_name(index_name);
        let mut ids = Vec::new();

        // Process in batches
        for chunk in documents.chunks(self.config.performance.batch_size) {
            let mut query_builder = sqlx::QueryBuilder::new(
                format!("INSERT INTO {} (id, content, embedding, metadata) ", table_name)
            );

            query_builder.push_values(chunk, |mut b, doc| {
                let embedding = doc.embedding.as_ref()
                    .ok_or_else(|| VectorError::InvalidVector("Document must have embedding".to_string()))
                    .unwrap();

                let metadata_json = Self::metadata_to_jsonb(&doc.metadata).unwrap();

                b.push_bind(&doc.id)
                    .push_bind(&doc.content)
                    .push_bind(embedding)
                    .push_bind(metadata_json);

                ids.push(doc.id.clone());
            });

            query_builder.push(" ON CONFLICT (id) DO UPDATE SET content = EXCLUDED.content, embedding = EXCLUDED.embedding, metadata = EXCLUDED.metadata, updated_at = NOW()");

            let query = query_builder.build();
            query.execute(&self.pool).await.map_err(PostgresError::from)?;
        }

        debug!("Upserted {} documents to table: {}", ids.len(), table_name);
        Ok(ids)
    }

    #[instrument(skip(self, request))]
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        let table_name = self.config.table_name(&request.index_name);

        // Set search parameters
        self.set_search_params().await?;

        let query_vector = match &request.query {
            SearchQuery::Vector(vec) => vec.clone(),
            SearchQuery::Text(_) => {
                return Err(VectorError::NotSupported("Text search not implemented for PostgreSQL backend".to_string()));
            },
        };

        // Build the search query
        let operator = Self::similarity_operator(SimilarityMetric::Cosine); // TODO: Get from index config
        let mut query = format!(
            "SELECT id, content, embedding, metadata, (embedding {} $1) as distance FROM {} ",
            operator, table_name
        );

        let mut bind_index = 2;

        // Add filter conditions if present
        if let Some(_filter) = &request.filter {
            // TODO: Implement filter conversion to SQL WHERE clause
            warn!("Filters not yet implemented for PostgreSQL backend");
        }

        query.push_str(&format!(" ORDER BY distance LIMIT {}", request.top_k));

        let rows = sqlx::query(&query)
            .bind(&query_vector)
            .fetch_all(&self.pool)
            .await
            .map_err(PostgresError::from)?;

        let mut results = Vec::new();
        for row in rows {
            let id: String = row.try_get("id").map_err(PostgresError::from)?;
            let content: String = row.try_get("content").map_err(PostgresError::from)?;
            let distance: f32 = row.try_get("distance").map_err(PostgresError::from)?;
            let metadata_json: JsonValue = row.try_get("metadata").map_err(PostgresError::from)?;

            let embedding = if request.include_vectors {
                let embedding_data: Vec<f32> = row.try_get("embedding").map_err(PostgresError::from)?;
                Some(embedding_data)
            } else {
                None
            };

            let metadata = if request.include_metadata {
                Self::jsonb_to_metadata(metadata_json)
            } else {
                HashMap::new()
            };

            let result = SearchResult {
                id,
                content: Some(content),
                vector: embedding,
                metadata: Some(metadata),
                score: 1.0 - distance, // Convert distance to similarity score
            };

            results.push(result);
        }

        Ok(SearchResponse {
            results,
            total_count: None, // Could implement with separate count query
            execution_time_ms: None,
            metadata: HashMap::new(),
        })
    }

    #[instrument(skip(self))]
    async fn update_document(&self, index_name: &str, document: Document) -> Result<()> {
        // For PostgreSQL, update is the same as upsert
        self.upsert_documents(index_name, vec![document]).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete_documents(&self, index_name: &str, ids: Vec<DocumentId>) -> Result<()> {
        let table_name = self.config.table_name(index_name);

        if ids.is_empty() {
            return Ok(());
        }

        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${}", i)).collect();
        let query = format!(
            "DELETE FROM {} WHERE id IN ({})",
            table_name,
            placeholders.join(", ")
        );

        let mut sqlx_query = sqlx::query(&query);
        for id in &ids {
            sqlx_query = sqlx_query.bind(id);
        }

        let result = sqlx_query.execute(&self.pool).await.map_err(PostgresError::from)?;
        let deleted_count = result.rows_affected() as usize;

        debug!("Deleted {} documents from table: {}", deleted_count, table_name);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_documents(&self, index_name: &str, ids: Vec<DocumentId>, include_vectors: bool) -> Result<Vec<Document>> {
        let table_name = self.config.table_name(index_name);

        if ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${}", i)).collect();
        let vector_select = if include_vectors { ", embedding" } else { "" };
        let query = format!(
            "SELECT id, content, metadata{} FROM {} WHERE id IN ({})",
            vector_select,
            table_name,
            placeholders.join(", ")
        );

        let mut sqlx_query = sqlx::query(&query);
        for id in &ids {
            sqlx_query = sqlx_query.bind(id);
        }

        let rows = sqlx_query.fetch_all(&self.pool).await.map_err(PostgresError::from)?;

        let mut documents = Vec::new();
        for row in rows {
            let id: String = row.try_get("id").map_err(PostgresError::from)?;
            let content: String = row.try_get("content").map_err(PostgresError::from)?;
            let metadata_json: JsonValue = row.try_get("metadata").map_err(PostgresError::from)?;

            let embedding = if include_vectors {
                let embedding_data: Vec<f32> = row.try_get("embedding").map_err(PostgresError::from)?;
                Some(embedding_data)
            } else {
                None
            };

            let metadata = Self::jsonb_to_metadata(metadata_json);

            let document = Document {
                id,
                content,
                embedding,
                metadata,
            };

            documents.push(document);
        }

        Ok(documents)
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(PostgresError::from)?;

        // Check pgvector extension
        self.ensure_pgvector_extension().await?;

        Ok(())
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            name: "PostgreSQL".to_string(),
            version: "1.0.0".to_string(),
            features: vec![
                "persistent".to_string(),
                "transactions".to_string(),
                "sql_queries".to_string(),
                "metadata_filtering".to_string(),
                "vector_indexes".to_string(),
            ],
            metadata: HashMap::new(),
        }
    }
}
