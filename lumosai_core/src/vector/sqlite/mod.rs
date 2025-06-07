use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use uuid::Uuid;
use serde_json::Value;
use rusqlite::{params, Connection, Result as SqliteResult};

use super::{VectorStorage, IndexStats, QueryResult, SimilarityMetric, FilterCondition};
use crate::error::{Error, Result};

/// SQLite vector storage implementation
pub struct SqliteVectorStorage {
    /// Database connection
    conn: Arc<Mutex<Connection>>,
}

impl SqliteVectorStorage {
    /// Create a new SQLite vector storage with connection to database file
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)
            .map_err(|e| Error::Storage(format!("Failed to open SQLite database: {}", e)))?;
        
        // Create metadata table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                id TEXT PRIMARY KEY,
                index_name TEXT NOT NULL,
                meta_json TEXT NOT NULL
            )",
            [],
        ).map_err(|e| Error::Storage(format!("Failed to create metadata table: {}", e)))?;
        
        // Create indexes table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS indexes (
                name TEXT PRIMARY KEY,
                dimension INTEGER NOT NULL,
                metric TEXT NOT NULL,
                count INTEGER NOT NULL DEFAULT 0
            )",
            [],
        ).map_err(|e| Error::Storage(format!("Failed to create indexes table: {}", e)))?;
        
        // Create vectors table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS vectors (
                id TEXT NOT NULL,
                index_name TEXT NOT NULL,
                vector_json TEXT NOT NULL,
                PRIMARY KEY (id, index_name)
            )",
            [],
        ).map_err(|e| Error::Storage(format!("Failed to create vectors table: {}", e)))?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
    
    /// Create a new in-memory SQLite vector storage (for testing)
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| Error::Storage(format!("Failed to open in-memory SQLite database: {}", e)))?;
        
        // Create metadata table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                id TEXT PRIMARY KEY,
                index_name TEXT NOT NULL,
                meta_json TEXT NOT NULL
            )",
            [],
        ).map_err(|e| Error::Storage(format!("Failed to create metadata table: {}", e)))?;
        
        // Create indexes table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS indexes (
                name TEXT PRIMARY KEY,
                dimension INTEGER NOT NULL,
                metric TEXT NOT NULL,
                count INTEGER NOT NULL DEFAULT 0
            )",
            [],
        ).map_err(|e| Error::Storage(format!("Failed to create indexes table: {}", e)))?;
        
        // Create vectors table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS vectors (
                id TEXT NOT NULL,
                index_name TEXT NOT NULL,
                vector_json TEXT NOT NULL,
                PRIMARY KEY (id, index_name)
            )",
            [],
        ).map_err(|e| Error::Storage(format!("Failed to create vectors table: {}", e)))?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
    
    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// Calculate Euclidean distance between two vectors
    fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y) * (x - y))
            .sum::<f32>()
            .sqrt()
    }

    /// Calculate dot product between two vectors
    fn dot_product(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// Calculate similarity score based on metric
    fn calculate_similarity(&self, a: &[f32], b: &[f32], metric: SimilarityMetric) -> f32 {
        match metric {
            SimilarityMetric::Cosine => Self::cosine_similarity(a, b),
            SimilarityMetric::Euclidean => {
                let dist = Self::euclidean_distance(a, b);
                1.0 / (1.0 + dist) // Convert distance to similarity score
            }
            SimilarityMetric::DotProduct => Self::dot_product(a, b),
        }
    }
    
    /// Convert metric enum to string
    fn metric_to_string(metric: SimilarityMetric) -> String {
        match metric {
            SimilarityMetric::Cosine => "cosine".to_string(),
            SimilarityMetric::Euclidean => "euclidean".to_string(),
            SimilarityMetric::DotProduct => "dotproduct".to_string(),
        }
    }
    
    /// Convert string to metric enum
    fn string_to_metric(s: &str) -> Result<SimilarityMetric> {
        match s {
            "cosine" => Ok(SimilarityMetric::Cosine),
            "euclidean" => Ok(SimilarityMetric::Euclidean),
            "dotproduct" => Ok(SimilarityMetric::DotProduct),
            _ => Err(Error::Storage(format!("Unknown similarity metric: {}", s))),
        }
    }

    /// Evaluate filter condition against metadata
    fn evaluate_filter(&self, filter: &FilterCondition, metadata: &HashMap<String, Value>) -> bool {
        match filter {
            FilterCondition::Eq(field, value) => {
                metadata.get(field).map_or(false, |v| v == value)
            },
            FilterCondition::Gt(field, value) => {
                if let (Some(field_value), Some(filter_value)) = (metadata.get(field), value.as_f64()) {
                    field_value.as_f64().map_or(false, |v| v > filter_value)
                } else {
                    false
                }
            },
            FilterCondition::Lt(field, value) => {
                if let (Some(field_value), Some(filter_value)) = (metadata.get(field), value.as_f64()) {
                    field_value.as_f64().map_or(false, |v| v < filter_value)
                } else {
                    false
                }
            },
            FilterCondition::In(field, values) => {
                if let Some(field_value) = metadata.get(field) {
                    values.contains(field_value)
                } else {
                    false
                }
            },
            FilterCondition::And(conditions) => {
                conditions.iter().all(|c| self.evaluate_filter(c, metadata))
            },
            FilterCondition::Or(conditions) => {
                conditions.iter().any(|c| self.evaluate_filter(c, metadata))
            },
            FilterCondition::Not(condition) => {
                !self.evaluate_filter(condition, metadata)
            },
        }
    }
}

#[async_trait]
impl VectorStorage for SqliteVectorStorage {
    async fn create_index(
        &self,
        index_name: &str,
        dimension: usize,
        metric: Option<SimilarityMetric>,
    ) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| Error::Storage("Failed to acquire lock".into()))?;
        
        let result = conn.execute(
            "INSERT INTO indexes (name, dimension, metric, count) VALUES (?, ?, ?, 0)",
            params![
                index_name,
                dimension as i64,
                Self::metric_to_string(metric.unwrap_or(SimilarityMetric::Cosine))
            ],
        );
        
        if let Err(e) = result {
            if e.to_string().contains("UNIQUE constraint failed") {
                return Err(Error::Storage(format!("Index {} already exists", index_name)));
            }
            return Err(Error::Storage(format!("Failed to create index: {}", e)));
        }
        
        Ok(())
    }

    async fn list_indexes(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().map_err(|_| Error::Storage("Failed to acquire lock".into()))?;
        
        let mut stmt = conn.prepare("SELECT name FROM indexes")
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;
        
        let names = stmt.query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| Error::Storage(format!("Failed to query indexes: {}", e)))?
            .collect::<SqliteResult<Vec<String>>>()
            .map_err(|e| Error::Storage(format!("Failed to collect index names: {}", e)))?;
        
        Ok(names)
    }

    async fn describe_index(&self, index_name: &str) -> Result<IndexStats> {
        let conn = self.conn.lock().map_err(|_| Error::Storage("Failed to acquire lock".into()))?;
        
        let result = conn.query_row(
            "SELECT dimension, metric, count FROM indexes WHERE name = ?",
            params![index_name],
            |row| {
                let dimension: i64 = row.get(0)?;
                let metric_str: String = row.get(1)?;
                let count: i64 = row.get(2)?;
                
                Ok((dimension, metric_str, count))
            },
        );
        
        match result {
            Ok((dimension, metric_str, count)) => {
                let metric = Self::string_to_metric(&metric_str)?;
                Ok(IndexStats {
                    dimension: dimension as usize,
                    count: count as usize,
                    metric,
                })
            }
            Err(e) => Err(Error::Storage(format!("Failed to describe index {}: {}", index_name, e))),
        }
    }

    async fn delete_index(&self, index_name: &str) -> Result<()> {
        let mut conn = self.conn.lock().map_err(|_| Error::Storage("Failed to acquire lock".into()))?;
        
        // Begin transaction
        let tx = conn.transaction()
            .map_err(|e| Error::Storage(format!("Failed to begin transaction: {}", e)))?;
        
        // Delete from indexes table
        tx.execute("DELETE FROM indexes WHERE name = ?", params![index_name])
            .map_err(|e| Error::Storage(format!("Failed to delete index: {}", e)))?;
        
        // Delete all vectors in this index
        tx.execute("DELETE FROM vectors WHERE index_name = ?", params![index_name])
            .map_err(|e| Error::Storage(format!("Failed to delete vectors: {}", e)))?;
        
        // Delete all metadata in this index
        tx.execute("DELETE FROM metadata WHERE index_name = ?", params![index_name])
            .map_err(|e| Error::Storage(format!("Failed to delete metadata: {}", e)))?;
        
        // Commit transaction
        tx.commit()
            .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;
        
        Ok(())
    }

    async fn upsert(
        &self,
        index_name: &str,
        vectors: Vec<Vec<f32>>,
        ids: Option<Vec<String>>,
        metadata: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let mut conn = self.conn.lock().map_err(|_| Error::Storage("Failed to acquire lock".into()))?;
        
        // Check if index exists
        let index_exists: bool = conn.query_row(
            "SELECT 1 FROM indexes WHERE name = ?",
            params![index_name],
            |_| Ok(true),
        ).unwrap_or(false);
        
        if !index_exists {
            return Err(Error::Storage(format!("Index {} not found", index_name)));
        }
        
        // Get index dimension
        let dimension: i64 = conn.query_row(
            "SELECT dimension FROM indexes WHERE name = ?",
            params![index_name],
            |row| row.get(0),
        ).map_err(|e| Error::Storage(format!("Failed to get index dimension: {}", e)))?;
        
        let vector_ids = ids.unwrap_or_else(|| vectors.iter().map(|_| Uuid::new_v4().to_string()).collect());
        
        if vector_ids.len() != vectors.len() {
            return Err(Error::Storage("Number of IDs must match number of vectors".into()));
        }
        
        if let Some(meta) = &metadata {
            if meta.len() != vectors.len() {
                return Err(Error::Storage("Number of metadata entries must match number of vectors".into()));
            }
        }
        
        // Begin transaction
        let tx = conn.transaction()
            .map_err(|e| Error::Storage(format!("Failed to begin transaction: {}", e)))?;
        
        for (i, (id, vector)) in vector_ids.iter().zip(vectors.iter()).enumerate() {
            if vector.len() != dimension as usize {
                return Err(Error::Storage(format!(
                    "Vector dimension mismatch: expected {}, got {}",
                    dimension,
                    vector.len()
                )));
            }
            
            // Serialize vector to JSON
            let vector_json = serde_json::to_string(vector)
                .map_err(|e| Error::Storage(format!("Failed to serialize vector: {}", e)))?;
            
            // Insert or replace vector
            tx.execute(
                "INSERT OR REPLACE INTO vectors (id, index_name, vector_json) VALUES (?, ?, ?)",
                params![id, index_name, vector_json],
            ).map_err(|e| Error::Storage(format!("Failed to insert vector: {}", e)))?;
            
            // Insert metadata if provided
            if let Some(meta) = metadata.as_ref().and_then(|m| m.get(i)) {
                let meta_json = serde_json::to_string(meta)
                    .map_err(|e| Error::Storage(format!("Failed to serialize metadata: {}", e)))?;
                
                tx.execute(
                    "INSERT OR REPLACE INTO metadata (id, index_name, meta_json) VALUES (?, ?, ?)",
                    params![id, index_name, meta_json],
                ).map_err(|e| Error::Storage(format!("Failed to insert metadata: {}", e)))?;
            }
        }
        
        // Update index count
        tx.execute(
            "UPDATE indexes SET count = (SELECT COUNT(*) FROM vectors WHERE index_name = ?) WHERE name = ?",
            params![index_name, index_name],
        ).map_err(|e| Error::Storage(format!("Failed to update index count: {}", e)))?;
        
        // Commit transaction
        tx.commit()
            .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;
        
        Ok(vector_ids)
    }

    async fn query(
        &self,
        index_name: &str,
        query_vector: Vec<f32>,
        top_k: usize,
        filter: Option<FilterCondition>,
        include_vectors: bool,
    ) -> Result<Vec<QueryResult>> {
        let conn = self.conn.lock().map_err(|_| Error::Storage("Failed to acquire lock".into()))?;
        
        // Check if index exists and get metric
        let result: Result<(i64, String, i64)> = conn.query_row(
            "SELECT dimension, metric, count FROM indexes WHERE name = ?",
            params![index_name],
            |row| {
                let dimension: i64 = row.get(0)?;
                let metric_str: String = row.get(1)?;
                let count: i64 = row.get(2)?;
                
                Ok((dimension, metric_str, count))
            },
        ).map_err(|e| Error::Storage(format!("Failed to get index info: {}", e)));
        
        let (dimension, metric_str, _) = result?;
        
        if query_vector.len() != dimension as usize {
            return Err(Error::Storage(format!(
                "Query vector dimension mismatch: expected {}, got {}",
                dimension,
                query_vector.len()
            )));
        }
        
        let metric = Self::string_to_metric(&metric_str)?;
        
        // Get all vectors
        let mut stmt = conn.prepare("
            SELECT v.id, v.vector_json, m.meta_json 
            FROM vectors v
            LEFT JOIN metadata m ON v.id = m.id AND v.index_name = m.index_name
            WHERE v.index_name = ?
        ").map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;
        
        let rows = stmt.query_map(params![index_name], |row| {
            let id: String = row.get(0)?;
            let vector_json: String = row.get(1)?;
            let meta_json: Option<String> = row.get(2).ok();
            
            Ok((id, vector_json, meta_json))
        }).map_err(|e| Error::Storage(format!("Failed to query vectors: {}", e)))?;
        
        let mut results = Vec::new();
        
        for row in rows {
            let (id, vector_json, meta_json) = row
                .map_err(|e| Error::Storage(format!("Failed to get row: {}", e)))?;
            
            let vector: Vec<f32> = serde_json::from_str(&vector_json)
                .map_err(|e| Error::Storage(format!("Failed to deserialize vector: {}", e)))?;
            
            let metadata: Option<HashMap<String, Value>> = if let Some(json) = meta_json {
                Some(serde_json::from_str(&json)
                    .map_err(|e| Error::Storage(format!("Failed to deserialize metadata: {}", e)))?)
            } else {
                None
            };
            
            // Apply filter if provided
            if let Some(filter) = &filter {
                if let Some(meta) = &metadata {
                    if !self.evaluate_filter(filter, meta) {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            
            let score = self.calculate_similarity(&query_vector, &vector, metric);
            
            results.push(QueryResult {
                id,
                score,
                vector: if include_vectors { Some(vector) } else { None },
                metadata,
            });
        }
        
        // Sort by score and limit to top_k
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        
        Ok(results)
    }

    async fn update_by_id(
        &self,
        index_name: &str,
        id: &str,
        vector: Option<Vec<f32>>,
        metadata: Option<HashMap<String, Value>>,
    ) -> Result<()> {
        let mut conn = self.conn.lock().map_err(|_| Error::Storage("Failed to acquire lock".into()))?;
        
        // Check if vector exists
        let vector_exists: bool = conn.query_row(
            "SELECT 1 FROM vectors WHERE id = ? AND index_name = ?",
            params![id, index_name],
            |_| Ok(true),
        ).unwrap_or(false);
        
        if !vector_exists {
            return Err(Error::Storage(format!("Vector with ID {} not found", id)));
        }
        
        // Begin transaction
        let tx = conn.transaction()
            .map_err(|e| Error::Storage(format!("Failed to begin transaction: {}", e)))?;
        
        // Update vector if provided
        if let Some(new_vector) = vector {
            // Get index dimension
            let dimension: i64 = tx.query_row(
                "SELECT dimension FROM indexes WHERE name = ?",
                params![index_name],
                |row| row.get(0),
            ).map_err(|e| Error::Storage(format!("Failed to get index dimension: {}", e)))?;
            
            if new_vector.len() != dimension as usize {
                return Err(Error::Storage(format!(
                    "Vector dimension mismatch: expected {}, got {}",
                    dimension,
                    new_vector.len()
                )));
            }
            
            let vector_json = serde_json::to_string(&new_vector)
                .map_err(|e| Error::Storage(format!("Failed to serialize vector: {}", e)))?;
            
            tx.execute(
                "UPDATE vectors SET vector_json = ? WHERE id = ? AND index_name = ?",
                params![vector_json, id, index_name],
            ).map_err(|e| Error::Storage(format!("Failed to update vector: {}", e)))?;
        }
        
        // Update metadata if provided
        if let Some(meta) = metadata {
            let meta_json = serde_json::to_string(&meta)
                .map_err(|e| Error::Storage(format!("Failed to serialize metadata: {}", e)))?;
            
            tx.execute(
                "INSERT OR REPLACE INTO metadata (id, index_name, meta_json) VALUES (?, ?, ?)",
                params![id, index_name, meta_json],
            ).map_err(|e| Error::Storage(format!("Failed to update metadata: {}", e)))?;
        }
        
        // Commit transaction
        tx.commit()
            .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;
        
        Ok(())
    }

    async fn delete_by_id(&self, index_name: &str, id: &str) -> Result<()> {
        let mut conn = self.conn.lock().map_err(|_| Error::Storage("Failed to acquire lock".into()))?;
        
        // Begin transaction
        let tx = conn.transaction()
            .map_err(|e| Error::Storage(format!("Failed to begin transaction: {}", e)))?;
        
        // Delete vector
        tx.execute(
            "DELETE FROM vectors WHERE id = ? AND index_name = ?",
            params![id, index_name],
        ).map_err(|e| Error::Storage(format!("Failed to delete vector: {}", e)))?;
        
        // Delete metadata
        tx.execute(
            "DELETE FROM metadata WHERE id = ? AND index_name = ?",
            params![id, index_name],
        ).map_err(|e| Error::Storage(format!("Failed to delete metadata: {}", e)))?;
        
        // Update index count
        tx.execute(
            "UPDATE indexes SET count = (SELECT COUNT(*) FROM vectors WHERE index_name = ?) WHERE name = ?",
            params![index_name, index_name],
        ).map_err(|e| Error::Storage(format!("Failed to update index count: {}", e)))?;
        
        // Commit transaction
        tx.commit()
            .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;
        
        Ok(())
    }
}

/// Create a new SQLite vector storage
pub fn create_sqlite_vector_storage<P: AsRef<Path>>(db_path: P) -> Result<SqliteVectorStorage> {
    SqliteVectorStorage::new(db_path)
}

/// Create a new in-memory SQLite vector storage (for testing)
pub fn create_sqlite_vector_storage_in_memory() -> Result<SqliteVectorStorage> {
    SqliteVectorStorage::new_in_memory()
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::approx_eq;
    
    const FLOAT_EPSILON: f32 = 1e-6;
    
    #[tokio::test]
    async fn test_sqlite_vector_operations() {
        let storage = SqliteVectorStorage::new_in_memory().unwrap();
        
        // Create index with default metric (Cosine)
        storage.create_index("test_index", 3, None).await.unwrap();
        
        // Insert vectors
        let vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
        ];
        let metadata = Some(vec![
            HashMap::from([("type".to_string(), serde_json::json!("A"))]),
            HashMap::from([("type".to_string(), serde_json::json!("B"))]),
        ]);
        
        let ids = storage.upsert("test_index", vectors, None, metadata).await.unwrap();
        assert_eq!(ids.len(), 2);
        
        // Query vectors
        let results = storage.query(
            "test_index",
            vec![1.0, 0.0, 0.0],
            2,
            Some(FilterCondition::Eq("type".to_string(), serde_json::json!("A"))),
            true,
        ).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert!(approx_eq!(f32, results[0].score, 1.0, epsilon = FLOAT_EPSILON));
        assert_eq!(results[0].vector.as_ref().unwrap(), &vec![1.0, 0.0, 0.0]);
        
        // Update vector
        storage.update_by_id(
            "test_index",
            &ids[0],
            Some(vec![0.0, 0.0, 1.0]),
            None,
        ).await.unwrap();
        
        // Delete vector
        storage.delete_by_id("test_index", &ids[0]).await.unwrap();
        
        // Check index stats
        let stats = storage.describe_index("test_index").await.unwrap();
        assert_eq!(stats.dimension, 3);
        assert_eq!(stats.count, 1);
        assert_eq!(stats.metric, SimilarityMetric::Cosine);
        
        // Delete index
        storage.delete_index("test_index").await.unwrap();
        
        let indexes = storage.list_indexes().await.unwrap();
        assert!(indexes.is_empty());
    }
    
    #[tokio::test]
    async fn test_sqlite_similarity_metrics() {
        let storage = SqliteVectorStorage::new_in_memory().unwrap();
        let test_vectors = vec![
            vec![1.0, 0.0, 0.0],  // Vector A
            vec![0.0, 1.0, 0.0],  // Vector B
            vec![1.0, 1.0, 0.0],  // Vector C
        ];
        
        // Test Cosine similarity
        storage.create_index("cosine_index", 3, Some(SimilarityMetric::Cosine)).await.unwrap();
        storage.upsert("cosine_index", test_vectors.clone(), None, None).await.unwrap();
        let cosine_results = storage.query(
            "cosine_index",
            vec![1.0, 1.0, 0.0],  // Query vector (same as C)
            3,
            None,
            false,
        ).await.unwrap();
        assert!(approx_eq!(f32, cosine_results[0].score, 1.0, epsilon = FLOAT_EPSILON)); // C should match perfectly
        assert!(cosine_results[1].score < 1.0); // A and B should have lower scores
        
        // Test Euclidean similarity
        storage.create_index("euclidean_index", 3, Some(SimilarityMetric::Euclidean)).await.unwrap();
        storage.upsert("euclidean_index", test_vectors.clone(), None, None).await.unwrap();
        let euclidean_results = storage.query(
            "euclidean_index",
            vec![1.0, 1.0, 0.0],  // Query vector (same as C)
            3,
            None,
            false,
        ).await.unwrap();
        assert!(approx_eq!(f32, euclidean_results[0].score, 1.0, epsilon = FLOAT_EPSILON)); // C should match perfectly
        assert!(euclidean_results[1].score < 1.0); // A and B should have lower scores
        
        // Test Dot product similarity
        storage.create_index("dot_index", 3, Some(SimilarityMetric::DotProduct)).await.unwrap();
        storage.upsert("dot_index", test_vectors.clone(), None, None).await.unwrap();
        let dot_results = storage.query(
            "dot_index",
            vec![1.0, 1.0, 0.0],  // Query vector (same as C)
            3,
            None,
            false,
        ).await.unwrap();
        assert!(approx_eq!(f32, dot_results[0].score, 2.0, epsilon = FLOAT_EPSILON)); // C should have dot product of 2
        assert!(dot_results[1].score < 2.0); // A and B should have lower scores
    }
} 