use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use futures::TryStreamExt;
use serde_json::{json, Value};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
    PgPool, Row,
};
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

/// PostgreSQL vector store implementation
#[derive(Debug)]
pub struct PostgresVectorStore {
    pool: Arc<Mutex<PgPool>>,
    filter_translator: PostgresFilterTranslator,
}

impl PostgresVectorStore {
    /// Create a new PostgreSQL vector store from a connection string
    pub async fn new(connection_string: &str) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(connection_string)
            .await
            .map_err(|e| StoreError::ConnectionError(e.to_string()))?;
            
        Self::initialize_extensions(&pool).await?;
            
        Ok(Self {
            pool: Arc::new(Mutex::new(pool)),
            filter_translator: PostgresFilterTranslator,
        })
    }
    
    /// Create a new PostgreSQL vector store from connection parameters
    pub async fn from_options(
        host: &str,
        port: u16,
        database: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, StoreError> {
        let options = PgConnectOptions::new()
            .host(host)
            .port(port)
            .database(database)
            .username(username)
            .password(password);
            
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| StoreError::ConnectionError(e.to_string()))?;
            
        Self::initialize_extensions(&pool).await?;
            
        Ok(Self {
            pool: Arc::new(Mutex::new(pool)),
            filter_translator: PostgresFilterTranslator,
        })
    }
    
    /// Initialize required PostgreSQL extensions
    async fn initialize_extensions(pool: &PgPool) -> Result<(), StoreError> {
        // Create the vector extension if it doesn't exist
        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(pool)
            .await
            .map_err(|e| StoreError::InternalError(format!("Failed to create vector extension: {}", e)))?;
            
        // Create the pgcrypto extension for UUID generation
        sqlx::query("CREATE EXTENSION IF NOT EXISTS pgcrypto")
            .execute(pool)
            .await
            .map_err(|e| StoreError::InternalError(format!("Failed to create pgcrypto extension: {}", e)))?;
            
        Ok(())
    }
    
    /// Checks if an index exists
    async fn index_exists(&self, index_name: &str) -> Result<bool, StoreError> {
        let pool = self.pool.lock().await;
        
        let row = sqlx::query(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_name = $1
            )"
        )
        .bind(index_name)
        .fetch_one(&*pool)
        .await
        .map_err(|e| StoreError::QueryError(e.to_string()))?;
        
        Ok(row.get::<bool, _>(0))
    }
    
    /// Get the supported distance function for a metric
    fn get_distance_function(&self, metric: &str) -> &'static str {
        match metric.to_lowercase().as_str() {
            "cosine" => "cosine_distance",
            "euclidean" => "l2_distance",
            "dotproduct" => "dot_product", // Lower means more similar for dot_product
            _ => "cosine_distance", // Default
        }
    }
    
    /// Convert a SQL result row to a QueryResult
    fn row_to_query_result(&self, row: PgRow) -> Result<QueryResult, StoreError> {
        let id: String = row.try_get("id").map_err(|e| StoreError::QueryError(e.to_string()))?;
        let score: f32 = row.try_get("score").map_err(|e| StoreError::QueryError(e.to_string()))?;
        let metadata_value: Value = row.try_get("metadata").map_err(|e| StoreError::QueryError(e.to_string()))?;
        
        let metadata = match metadata_value {
            Value::Object(map) => map.into_iter().collect(),
            _ => HashMap::new(),
        };
        
        let vector = if let Ok(vec_str) = row.try_get::<String, _>("embedding") {
            // Convert '[1,2,3]' format to vector
            let vec_value: Value = serde_json::from_str(&vec_str)
                .map_err(|e| StoreError::SerializationError(e.to_string()))?;
                
            if let Value::Array(arr) = vec_value {
                let values = arr.into_iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect();
                Some(values)
            } else {
                None
            }
        } else {
            None
        };
        
        Ok(QueryResult {
            id,
            score,
            metadata,
            vector,
        })
    }
}

#[async_trait]
impl VectorStore for PostgresVectorStore {
    #[instrument(skip(self))]
    async fn create_index(&self, params: CreateIndexParams) -> Result<(), StoreError> {
        let pool = self.pool.lock().await;
        
        // Check if the index already exists
        if self.index_exists(&params.index_name).await? {
            return Err(StoreError::IndexError(format!("Index '{}' already exists", params.index_name)));
        }
        
        // Create the vector table
        let create_table_sql = format!(
            "CREATE TABLE {} (
                id TEXT PRIMARY KEY,
                embedding vector({}) NOT NULL,
                metadata JSONB NOT NULL DEFAULT '{{}}'::jsonb
            )",
            params.index_name, params.dimension
        );
        
        sqlx::query(&create_table_sql)
            .execute(&*pool)
            .await
            .map_err(|e| StoreError::IndexError(format!("Failed to create table: {}", e)))?;
            
        // Create index for vector search
        let create_index_sql = format!(
            "CREATE INDEX {}_embedding_idx ON {} USING ivfflat (embedding vector_l2_ops)",
            params.index_name, params.index_name
        );
        
        sqlx::query(&create_index_sql)
            .execute(&*pool)
            .await
            .map_err(|e| StoreError::IndexError(format!("Failed to create vector index: {}", e)))?;
            
        // Create index for metadata
        let create_metadata_idx = format!(
            "CREATE INDEX {}_metadata_idx ON {} USING GIN (metadata)",
            params.index_name, params.index_name
        );
        
        sqlx::query(&create_metadata_idx)
            .execute(&*pool)
            .await
            .map_err(|e| StoreError::IndexError(format!("Failed to create metadata index: {}", e)))?;
            
        debug!("Created PostgreSQL vector index: {}", params.index_name);
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn upsert(&self, params: UpsertParams) -> Result<Vec<String>, StoreError> {
        let pool = self.pool.lock().await;
        
        // Check if the index exists
        if !self.index_exists(&params.index_name).await? {
            return Err(StoreError::IndexError(format!("Index '{}' does not exist", params.index_name)));
        }
        
        let ids = params.ids.unwrap_or_else(|| {
            (0..params.vectors.len())
                .map(|_| Uuid::new_v4().to_string())
                .collect()
        });
        
        let mut tx = pool.begin().await.map_err(|e| StoreError::QueryError(e.to_string()))?;
        
        for (i, id) in ids.iter().enumerate() {
            let vector = params.vectors.get(i).cloned().unwrap_or_default();
            let metadata = params.metadata.get(i).cloned().unwrap_or_default();
            
            let vector_str = json!(vector).to_string();
            let metadata_json = json!(metadata);
            
            let sql = format!(
                "INSERT INTO {} (id, embedding, metadata) 
                VALUES ($1, $2::vector, $3::jsonb)
                ON CONFLICT (id) DO UPDATE 
                SET embedding = $2::vector, 
                    metadata = $3::jsonb",
                params.index_name
            );
            
            sqlx::query(&sql)
                .bind(id)
                .bind(vector_str)
                .bind(metadata_json)
                .execute(&mut *tx)
                .await
                .map_err(|e| StoreError::VectorError(e.to_string()))?;
        }
        
        tx.commit().await.map_err(|e| StoreError::QueryError(e.to_string()))?;
        
        debug!("Upserted {} vectors into PostgreSQL index: {}", ids.len(), params.index_name);
        Ok(ids)
    }
    
    #[instrument(skip(self))]
    async fn query(&self, params: QueryParams) -> Result<Vec<QueryResult>, StoreError> {
        let pool = self.pool.lock().await;
        
        // Check if the index exists
        if !self.index_exists(&params.index_name).await? {
            return Err(StoreError::IndexError(format!("Index '{}' does not exist", params.index_name)));
        }
        
        let vector_str = json!(params.query_vector).to_string();
        let distance_fn = self.get_distance_function(&"cosine"); // Default to cosine
        
        let where_clause = if let Some(filter) = params.filter {
            let filter_sql = self.filter_translator.translate_to_sql(filter)?;
            format!("WHERE {}", filter_sql)
        } else {
            String::new()
        };
        
        let select_embedding = if params.include_vector {
            ", embedding::text as embedding"
        } else {
            ""
        };
        
        let sql = format!(
            "SELECT 
                id, 
                metadata, 
                {}(embedding, $1::vector) as score{}
            FROM {} 
            {}
            ORDER BY score ASC 
            LIMIT $2",
            distance_fn, select_embedding, params.index_name, where_clause
        );
        
        let rows = sqlx::query(&sql)
            .bind(vector_str)
            .bind(params.top_k as i32)
            .fetch_all(&*pool)
            .await
            .map_err(|e| StoreError::QueryError(e.to_string()))?;
            
        let results = rows.into_iter()
            .map(|row| self.row_to_query_result(row))
            .collect::<Result<Vec<_>, _>>()?;
            
        debug!("Queried PostgreSQL index: {}, found {} results", params.index_name, results.len());
        Ok(results)
    }
    
    #[instrument(skip(self))]
    async fn list_indexes(&self) -> Result<Vec<String>, StoreError> {
        let pool = self.pool.lock().await;
        
        let rows = sqlx::query(
            "SELECT table_name FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_type = 'BASE TABLE'"
        )
        .fetch_all(&*pool)
        .await
        .map_err(|e| StoreError::QueryError(e.to_string()))?;
        
        let tables = rows.into_iter()
            .map(|row| row.get::<String, _>(0))
            .collect();
            
        debug!("Listed PostgreSQL vector indexes");
        Ok(tables)
    }
    
    #[instrument(skip(self))]
    async fn describe_index(&self, index_name: &str) -> Result<IndexStats, StoreError> {
        let pool = self.pool.lock().await;
        
        // Check if the index exists
        if !self.index_exists(index_name).await? {
            return Err(StoreError::IndexError(format!("Index '{}' does not exist", index_name)));
        }
        
        // Get vector dimension
        let dimension_sql = format!(
            "SELECT 
                vector_dims(embedding) as dimension,
                count(*) as count
            FROM {}
            LIMIT 1",
            index_name
        );
        
        let row = sqlx::query(&dimension_sql)
            .fetch_optional(&*pool)
            .await
            .map_err(|e| StoreError::QueryError(e.to_string()))?;
            
        let dimension = match row {
            Some(r) => r.try_get::<i32, _>(0).unwrap_or(0) as usize,
            None => 0,
        };
        
        // Get vector count
        let count_sql = format!("SELECT COUNT(*) FROM {}", index_name);
        
        let count_row = sqlx::query(&count_sql)
            .fetch_one(&*pool)
            .await
            .map_err(|e| StoreError::QueryError(e.to_string()))?;
            
        let count = count_row.try_get::<i64, _>(0).unwrap_or(0) as usize;
        
        Ok(IndexStats {
            dimension,
            count,
            metric: "cosine".to_string(), // pgvector always uses cosine
        })
    }
    
    #[instrument(skip(self))]
    async fn delete_index(&self, index_name: &str) -> Result<(), StoreError> {
        let pool = self.pool.lock().await;
        
        // Check if the index exists
        if !self.index_exists(index_name).await? {
            return Err(StoreError::IndexError(format!("Index '{}' does not exist", index_name)));
        }
        
        let sql = format!("DROP TABLE IF EXISTS {}", index_name);
        
        sqlx::query(&sql)
            .execute(&*pool)
            .await
            .map_err(|e| StoreError::IndexError(e.to_string()))?;
            
        debug!("Deleted PostgreSQL vector index: {}", index_name);
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
        let pool = self.pool.lock().await;
        
        // Check if the index exists
        if !self.index_exists(index_name).await? {
            return Err(StoreError::IndexError(format!("Index '{}' does not exist", index_name)));
        }
        
        let mut tx = pool.begin().await.map_err(|e| StoreError::QueryError(e.to_string()))?;
        
        // Update vector if provided
        if let Some(vec) = vector {
            let vector_str = json!(vec).to_string();
            
            let sql = format!(
                "UPDATE {} SET embedding = $1::vector WHERE id = $2",
                index_name
            );
            
            sqlx::query(&sql)
                .bind(vector_str)
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(|e| StoreError::VectorError(e.to_string()))?;
        }
        
        // Update metadata if provided
        if let Some(meta) = metadata {
            let metadata_json = json!(meta);
            
            let sql = format!(
                "UPDATE {} SET metadata = $1::jsonb WHERE id = $2",
                index_name
            );
            
            sqlx::query(&sql)
                .bind(metadata_json)
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(|e| StoreError::VectorError(e.to_string()))?;
        }
        
        tx.commit().await.map_err(|e| StoreError::QueryError(e.to_string()))?;
        
        debug!("Updated vector in PostgreSQL index: {}, id: {}", index_name, id);
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn delete_vectors(&self, index_name: &str, ids: &[String]) -> Result<(), StoreError> {
        let pool = self.pool.lock().await;
        
        // Check if the index exists
        if !self.index_exists(index_name).await? {
            return Err(StoreError::IndexError(format!("Index '{}' does not exist", index_name)));
        }
        
        let sql = format!(
            "DELETE FROM {} WHERE id = ANY($1)",
            index_name
        );
        
        sqlx::query(&sql)
            .bind(ids)
            .execute(&*pool)
            .await
            .map_err(|e| StoreError::VectorError(e.to_string()))?;
            
        debug!("Deleted {} vectors from PostgreSQL index: {}", ids.len(), index_name);
        Ok(())
    }
}

/// PostgreSQL-specific filter translator
#[derive(Debug, Default)]
pub struct PostgresFilterTranslator;

impl PostgresFilterTranslator {
    /// Translate a VectorFilter to a SQL WHERE clause
    pub fn translate_to_sql(&self, filter: VectorFilter) -> Result<String, StoreError> {
        match filter {
            VectorFilter::And { and } => {
                let conditions = and.into_iter()
                    .map(|f| self.translate_to_sql(f))
                    .collect::<Result<Vec<_>, _>>()?;
                
                Ok(format!("({})", conditions.join(" AND ")))
            },
            VectorFilter::Or { or } => {
                let conditions = or.into_iter()
                    .map(|f| self.translate_to_sql(f))
                    .collect::<Result<Vec<_>, _>>()?;
                
                Ok(format!("({})", conditions.join(" OR ")))
            },
            VectorFilter::Not { not } => {
                let inner = self.translate_to_sql(*not)?;
                Ok(format!("NOT ({})", inner))
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
                        let value_sql = self.value_to_sql(value)?;
                        Ok(format!("metadata->>'{}' = {}", field, value_sql))
                    },
                    crate::vector::FieldCondition::Operator(ops) => {
                        self.translate_operators(field, ops)
                    },
                }
            },
        }
    }
    
    /// Translate a field with operators to a SQL WHERE clause
    fn translate_operators(&self, field: String, ops: HashMap<String, Value>) -> Result<String, StoreError> {
        let mut conditions = Vec::new();
        
        for (op, value) in ops {
            let value_sql = self.value_to_sql(value)?;
            
            match op.as_str() {
                "$eq" => {
                    conditions.push(format!("metadata->>'{}' = {}", field, value_sql));
                },
                "$ne" => {
                    conditions.push(format!("metadata->>'{}' <> {}", field, value_sql));
                },
                "$gt" => {
                    conditions.push(format!("(metadata->>'{}'::text)::float > {}", field, value_sql));
                },
                "$gte" => {
                    conditions.push(format!("(metadata->>'{}'::text)::float >= {}", field, value_sql));
                },
                "$lt" => {
                    conditions.push(format!("(metadata->>'{}'::text)::float < {}", field, value_sql));
                },
                "$lte" => {
                    conditions.push(format!("(metadata->>'{}'::text)::float <= {}", field, value_sql));
                },
                "$in" => {
                    if let Value::Array(array) = value {
                        let items = array.into_iter()
                            .map(|v| self.value_to_sql(v))
                            .collect::<Result<Vec<_>, _>>()?
                            .join(", ");
                            
                        conditions.push(format!("metadata->>'{}' IN ({})", field, items));
                    } else {
                        return Err(StoreError::FilterError("$in requires an array value".to_string()));
                    }
                },
                "$nin" => {
                    if let Value::Array(array) = value {
                        let items = array.into_iter()
                            .map(|v| self.value_to_sql(v))
                            .collect::<Result<Vec<_>, _>>()?
                            .join(", ");
                            
                        conditions.push(format!("metadata->>'{}' NOT IN ({})", field, items));
                    } else {
                        return Err(StoreError::FilterError("$nin requires an array value".to_string()));
                    }
                },
                "$regex" => {
                    if let Value::String(pattern) = value {
                        conditions.push(format!("metadata->>'{}' ~ {}", field, self.value_to_sql(Value::String(pattern))?));
                    } else {
                        return Err(StoreError::FilterError("$regex requires a string value".to_string()));
                    }
                },
                "$exists" => {
                    if let Value::Bool(exists) = value {
                        if exists {
                            conditions.push(format!("metadata ? '{}'", field));
                        } else {
                            conditions.push(format!("NOT (metadata ? '{}')", field));
                        }
                    } else {
                        return Err(StoreError::FilterError("$exists requires a boolean value".to_string()));
                    }
                },
                _ => {
                    return Err(StoreError::FilterError(format!("Unsupported operator: {}", op)));
                }
            }
        }
        
        if conditions.is_empty() {
            Ok("TRUE".to_string())
        } else {
            Ok(format!("({})", conditions.join(" AND ")))
        }
    }
    
    /// Convert a JSON Value to a SQL literal
    fn value_to_sql(&self, value: Value) -> Result<String, StoreError> {
        match value {
            Value::Null => Ok("NULL".to_string()),
            Value::Bool(b) => Ok(if b { "TRUE".to_string() } else { "FALSE".to_string() }),
            Value::Number(n) => Ok(n.to_string()),
            Value::String(s) => Ok(format!("'{}'", s.replace('\'', "''"))),
            Value::Array(_) => {
                let json_str = serde_json::to_string(&value)
                    .map_err(|e| StoreError::SerializationError(e.to_string()))?;
                Ok(format!("'{}'", json_str.replace('\'', "''")))
            },
            Value::Object(_) => {
                let json_str = serde_json::to_string(&value)
                    .map_err(|e| StoreError::SerializationError(e.to_string()))?;
                Ok(format!("'{}'", json_str.replace('\'', "''")))
            },
        }
    }
}

#[async_trait]
impl VectorFilterTranslator for PostgresFilterTranslator {
    async fn translate_filter<T: Send + Sync>(&self, filter: Option<VectorFilter>) -> Result<T, StoreError> {
        if filter.is_none() {
            return Err(StoreError::FilterError("No filter provided".to_string()));
        }
        
        let sql = self.translate_to_sql(filter.unwrap())?;
        
        // Create a JSON value and convert to the target type
        let value = json!(sql);
        
        serde_json::from_value(value)
            .map_err(|e| StoreError::SerializationError(e.to_string()))
    }
} 