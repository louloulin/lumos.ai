use std::collections::HashMap;
use std::fmt::Debug;

use async_trait::async_trait;
use mongodb::{Client, Collection, Database};
use mongodb::bson::{doc, Document, Bson};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    VectorStore,
};

/// MongoDB vector document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VectorDocument {
    #[serde(rename = "_id")]
    id: String,
    vector: Vec<f32>,
    metadata: HashMap<String, Value>,
    #[serde(rename = "indexName")]
    index_name: String,
}

/// MongoDB vector store implementation
#[derive(Debug)]
pub struct MongoDBStore {
    database: Database,
    collection_name: String,
}

impl MongoDBStore {
    /// Create a new MongoDB vector store
    pub async fn new(connection_string: &str, database_name: &str) -> Result<Self, StoreError> {
        let client = Client::with_uri_str(connection_string).await
            .map_err(|e| StoreError::ConnectionError(e.to_string()))?;
            
        let database = client.database(database_name);
        
        // Test connection
        database.run_command(doc! {"ping": 1}, None).await
            .map_err(|e| StoreError::ConnectionError(e.to_string()))?;
            
        Ok(Self {
            database,
            collection_name: "vectors".to_string(),
        })
    }
    
    /// Get the vectors collection
    fn get_collection(&self) -> Collection<VectorDocument> {
        self.database.collection(&self.collection_name)
    }
    
    /// Get the indexes collection for metadata
    fn get_indexes_collection(&self) -> Collection<Document> {
        self.database.collection("vector_indexes")
    }
    
    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
    
    /// Convert VectorFilter to MongoDB filter document
    fn convert_filter(&self, filter: VectorFilter) -> Result<Document, StoreError> {
        match filter {
            VectorFilter::And { and } => {
                let conditions: Result<Vec<Document>, _> = and.into_iter()
                    .map(|f| self.convert_filter(f))
                    .collect();
                Ok(doc! { "$and": conditions? })
            },
            VectorFilter::Or { or } => {
                let conditions: Result<Vec<Document>, _> = or.into_iter()
                    .map(|f| self.convert_filter(f))
                    .collect();
                Ok(doc! { "$or": conditions? })
            },
            VectorFilter::Not { not } => {
                let condition = self.convert_filter(*not)?;
                Ok(doc! { "$not": condition })
            },
            VectorFilter::Field(fields) => {
                let mut filter_doc = Document::new();
                for (field, condition) in fields {
                    let field_path = format!("metadata.{}", field);
                    match condition {
                        crate::vector::FieldCondition::Value(value) => {
                            let bson_value = mongodb::bson::to_bson(&value)
                                .map_err(|e| StoreError::SerializationError(e.to_string()))?;
                            filter_doc.insert(field_path, bson_value);
                        },
                        crate::vector::FieldCondition::Operator(ops) => {
                            let mut op_doc = Document::new();
                            for (op, value) in ops {
                                let bson_value = mongodb::bson::to_bson(&value)
                                    .map_err(|e| StoreError::SerializationError(e.to_string()))?;
                                op_doc.insert(op, bson_value);
                            }
                            filter_doc.insert(field_path, op_doc);
                        },
                    }
                }
                Ok(filter_doc)
            },
        }
    }
}

#[async_trait]
impl VectorStore for MongoDBStore {
    #[instrument(skip(self))]
    async fn create_index(&self, params: CreateIndexParams) -> Result<(), StoreError> {
        let indexes_collection = self.get_indexes_collection();
        
        let index_doc = doc! {
            "_id": &params.index_name,
            "dimension": params.dimension as i32,
            "metric": &params.metric,
            "count": 0i32,
            "created_at": mongodb::bson::DateTime::now(),
        };
        
        indexes_collection.insert_one(index_doc, None).await
            .map_err(|e| {
                if e.to_string().contains("duplicate key") {
                    StoreError::IndexAlreadyExists(params.index_name.clone())
                } else {
                    StoreError::DatabaseError(e.to_string())
                }
            })?;
            
        debug!("Created MongoDB vector index: {}", params.index_name);
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn upsert(&self, params: UpsertParams) -> Result<Vec<String>, StoreError> {
        let collection = self.get_collection();
        
        let ids = params.ids.unwrap_or_else(|| {
            (0..params.vectors.len())
                .map(|_| Uuid::new_v4().to_string())
                .collect()
        });
        
        if ids.len() != params.vectors.len() {
            return Err(StoreError::InvalidInput("Number of IDs must match number of vectors".to_string()));
        }
        
        let documents: Vec<VectorDocument> = ids.iter().enumerate().map(|(i, id)| {
            let vector = params.vectors.get(i).cloned().unwrap_or_default();
            let metadata = params.metadata.get(i).cloned().unwrap_or_default();
            
            VectorDocument {
                id: id.clone(),
                vector,
                metadata,
                index_name: params.index_name.clone(),
            }
        }).collect();
        
        // Use replace_one for each document to handle upserts
        for doc in documents {
            let filter = doc! { "_id": &doc.id, "indexName": &params.index_name };
            collection.replace_one(filter, &doc, mongodb::options::ReplaceOptions::builder().upsert(true).build()).await
                .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        }
        
        // Update index count
        let indexes_collection = self.get_indexes_collection();
        let count = collection.count_documents(doc! { "indexName": &params.index_name }, None).await
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
            
        indexes_collection.update_one(
            doc! { "_id": &params.index_name },
            doc! { "$set": { "count": count as i32 } },
            None
        ).await.map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        debug!("Inserted {} vectors into MongoDB index: {}", ids.len(), params.index_name);
        Ok(ids)
    }
    
    #[instrument(skip(self))]
    async fn query(&self, params: QueryParams) -> Result<Vec<QueryResult>, StoreError> {
        let collection = self.get_collection();
        
        let mut filter = doc! { "indexName": &params.index_name };
        
        if let Some(vector_filter) = params.filter {
            let additional_filter = self.convert_filter(vector_filter)?;
            filter.extend(additional_filter);
        }
        
        let cursor = collection.find(filter, None).await
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
            
        let documents: Vec<VectorDocument> = cursor.try_collect().await
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
            
        // Calculate similarities and sort
        let mut results: Vec<QueryResult> = documents.into_iter().map(|doc| {
            let score = Self::cosine_similarity(&params.query_vector, &doc.vector);
            QueryResult {
                id: doc.id,
                score,
                metadata: doc.metadata,
                vector: if params.include_vector { Some(doc.vector) } else { None },
            }
        }).collect();
        
        // Sort by score (descending) and take top_k
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(params.top_k);
        
        debug!("Queried MongoDB index: {}, found {} results", params.index_name, results.len());
        Ok(results)
    }
    
    #[instrument(skip(self))]
    async fn list_indexes(&self) -> Result<Vec<String>, StoreError> {
        let indexes_collection = self.get_indexes_collection();
        
        let cursor = indexes_collection.find(doc! {}, None).await
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
            
        let documents: Vec<Document> = cursor.try_collect().await
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
            
        let names = documents.into_iter()
            .filter_map(|doc| doc.get_str("_id").ok().map(|s| s.to_string()))
            .collect();
            
        debug!("Listed MongoDB vector indexes");
        Ok(names)
    }
    
    #[instrument(skip(self))]
    async fn describe_index(&self, index_name: &str) -> Result<IndexStats, StoreError> {
        let indexes_collection = self.get_indexes_collection();
        
        let index_doc = indexes_collection.find_one(doc! { "_id": index_name }, None).await
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?
            .ok_or_else(|| StoreError::IndexNotFound(index_name.to_string()))?;
            
        let dimension = index_doc.get_i32("dimension")
            .map_err(|e| StoreError::DatabaseError(e.to_string()))? as usize;
        let count = index_doc.get_i32("count")
            .map_err(|e| StoreError::DatabaseError(e.to_string()))? as usize;
        let metric = index_doc.get_str("metric")
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?
            .to_string();
            
        Ok(IndexStats {
            dimension,
            count,
            metric,
        })
    }
    
    #[instrument(skip(self))]
    async fn delete_index(&self, index_name: &str) -> Result<(), StoreError> {
        let collection = self.get_collection();
        let indexes_collection = self.get_indexes_collection();
        
        // Delete all vectors in the index
        collection.delete_many(doc! { "indexName": index_name }, None).await
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
            
        // Delete the index metadata
        indexes_collection.delete_one(doc! { "_id": index_name }, None).await
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
            
        debug!("Deleted MongoDB vector index: {}", index_name);
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
        let collection = self.get_collection();
        
        let mut update_doc = Document::new();
        
        if let Some(vector) = vector {
            update_doc.insert("vector", vector);
        }
        
        if let Some(metadata) = metadata {
            update_doc.insert("metadata", mongodb::bson::to_bson(&metadata)
                .map_err(|e| StoreError::SerializationError(e.to_string()))?);
        }
        
        if update_doc.is_empty() {
            return Ok(());
        }
        
        collection.update_one(
            doc! { "_id": id, "indexName": index_name },
            doc! { "$set": update_doc },
            None
        ).await.map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        debug!("Updated vector in MongoDB index: {}, id: {}", index_name, id);
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn delete_vectors(&self, index_name: &str, ids: &[String]) -> Result<(), StoreError> {
        let collection = self.get_collection();
        
        collection.delete_many(
            doc! { "_id": { "$in": ids }, "indexName": index_name },
            None
        ).await.map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        // Update index count
        let indexes_collection = self.get_indexes_collection();
        let count = collection.count_documents(doc! { "indexName": index_name }, None).await
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
            
        indexes_collection.update_one(
            doc! { "_id": index_name },
            doc! { "$set": { "count": count as i32 } },
            None
        ).await.map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        debug!("Deleted {} vectors from MongoDB index: {}", ids.len(), index_name);
        Ok(())
    }
}
