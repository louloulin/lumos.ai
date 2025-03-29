#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use serde_json::{json, Value};
    
    use crate::vector::{
        CreateIndexParams,
        QueryParams,
        UpsertParams,
        VectorFilter,
        VectorStore,
    };
    
    #[cfg(feature = "qdrant")]
    mod qdrant_tests {
        use super::*;
        use crate::qdrant::QdrantStore;
        
        #[tokio::test]
        #[ignore] // Requires a Qdrant server
        async fn test_qdrant_basic_operations() {
            // Create a Qdrant client
            let store = QdrantStore::new("http://localhost:6333", None)
                .await
                .expect("Failed to create Qdrant store");
                
            // Create a test index
            let index_name = format!("test_index_{}", uuid::Uuid::new_v4());
            let create_params = CreateIndexParams {
                index_name: index_name.clone(),
                dimension: 4,
                metric: "cosine".to_string(),
            };
            
            // Create the index
            store.create_index(create_params)
                .await
                .expect("Failed to create index");
                
            // Insert vectors
            let vectors = vec![
                vec![1.0, 2.0, 3.0, 4.0],
                vec![5.0, 6.0, 7.0, 8.0],
                vec![9.0, 10.0, 11.0, 12.0],
            ];
            
            let metadata = vec![
                HashMap::from([
                    ("name".to_string(), json!("item1")),
                    ("category".to_string(), json!("test")),
                    ("score".to_string(), json!(10)),
                ]),
                HashMap::from([
                    ("name".to_string(), json!("item2")),
                    ("category".to_string(), json!("test")),
                    ("score".to_string(), json!(20)),
                ]),
                HashMap::from([
                    ("name".to_string(), json!("item3")),
                    ("category".to_string(), json!("prod")),
                    ("score".to_string(), json!(30)),
                ]),
            ];
            
            let upsert_params = UpsertParams {
                index_name: index_name.clone(),
                vectors: vectors.clone(),
                metadata: metadata.clone(),
                ids: None,
            };
            
            let ids = store.upsert(upsert_params)
                .await
                .expect("Failed to insert vectors");
                
            assert_eq!(ids.len(), 3);
            
            // Query vector, similar to the first one
            let query_params = QueryParams {
                index_name: index_name.clone(),
                query_vector: vec![1.0, 2.0, 3.0, 4.0],
                top_k: 2,
                filter: None,
                include_vector: true,
            };
            
            let results = store.query(query_params)
                .await
                .expect("Failed to query vectors");
                
            assert_eq!(results.len(), 2);
            assert_eq!(results[0].metadata.get("name").unwrap(), &json!("item1"));
            
            // Query with filter
            let filter = VectorFilter::Field(HashMap::from([
                ("category".to_string(), crate::vector::FieldCondition::Value(json!("prod"))),
            ]));
            
            let query_params = QueryParams {
                index_name: index_name.clone(),
                query_vector: vec![1.0, 2.0, 3.0, 4.0],
                top_k: 10,
                filter: Some(filter),
                include_vector: false,
            };
            
            let results = store.query(query_params)
                .await
                .expect("Failed to query vectors with filter");
                
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].metadata.get("category").unwrap(), &json!("prod"));
            
            // Update a vector
            let update_metadata = HashMap::from([
                ("name".to_string(), json!("updated_item")),
                ("category".to_string(), json!("updated")),
            ]);
            
            store.update_vector_by_id(
                &index_name,
                &ids[0],
                None,
                Some(update_metadata),
            )
            .await
            .expect("Failed to update vector");
            
            // Query to verify update
            let query_params = QueryParams {
                index_name: index_name.clone(),
                query_vector: vec![1.0, 2.0, 3.0, 4.0],
                top_k: 1,
                filter: None,
                include_vector: false,
            };
            
            let results = store.query(query_params)
                .await
                .expect("Failed to query after update");
                
            assert_eq!(results[0].metadata.get("name").unwrap(), &json!("updated_item"));
            
            // Delete vectors
            store.delete_vectors(&index_name, &[ids[0].clone()])
                .await
                .expect("Failed to delete vectors");
                
            // Delete the index
            store.delete_index(&index_name)
                .await
                .expect("Failed to delete index");
        }
    }
    
    #[cfg(feature = "postgres")]
    mod postgres_tests {
        use super::*;
        use crate::postgres::PostgresVectorStore;
        
        #[tokio::test]
        #[ignore] // Requires a PostgreSQL server with pgvector extension
        async fn test_postgres_basic_operations() {
            // Create a PostgreSQL client
            let store = PostgresVectorStore::new("postgres://postgres:password@localhost:5432/vectors")
                .await
                .expect("Failed to create PostgreSQL store");
                
            // Create a test index
            let index_name = format!("test_index_{}", uuid::Uuid::new_v4());
            let create_params = CreateIndexParams {
                index_name: index_name.clone(),
                dimension: 4,
                metric: "cosine".to_string(),
            };
            
            // Create the index
            store.create_index(create_params)
                .await
                .expect("Failed to create index");
                
            // Insert vectors (same as Qdrant test)
            let vectors = vec![
                vec![1.0, 2.0, 3.0, 4.0],
                vec![5.0, 6.0, 7.0, 8.0],
                vec![9.0, 10.0, 11.0, 12.0],
            ];
            
            let metadata = vec![
                HashMap::from([
                    ("name".to_string(), json!("item1")),
                    ("category".to_string(), json!("test")),
                    ("score".to_string(), json!(10)),
                ]),
                HashMap::from([
                    ("name".to_string(), json!("item2")),
                    ("category".to_string(), json!("test")),
                    ("score".to_string(), json!(20)),
                ]),
                HashMap::from([
                    ("name".to_string(), json!("item3")),
                    ("category".to_string(), json!("prod")),
                    ("score".to_string(), json!(30)),
                ]),
            ];
            
            let upsert_params = UpsertParams {
                index_name: index_name.clone(),
                vectors: vectors.clone(),
                metadata: metadata.clone(),
                ids: None,
            };
            
            let ids = store.upsert(upsert_params)
                .await
                .expect("Failed to insert vectors");
                
            assert_eq!(ids.len(), 3);
            
            // Delete the index
            store.delete_index(&index_name)
                .await
                .expect("Failed to delete index");
        }
    }
    
    #[cfg(feature = "vectorize")]
    mod vectorize_tests {
        use super::*;
        use crate::vectorize::VectorizeStore;
        
        #[tokio::test]
        #[ignore] // Requires Cloudflare API credentials
        async fn test_vectorize_basic_operations() {
            // These would typically come from environment variables
            let api_token = std::env::var("CF_API_TOKEN").expect("CF_API_TOKEN not set");
            let account_id = std::env::var("CF_ACCOUNT_ID").expect("CF_ACCOUNT_ID not set");
            
            // Create a Vectorize client
            let store = VectorizeStore::new(&api_token, &account_id)
                .await
                .expect("Failed to create Vectorize store");
                
            // Create a test index
            let index_name = format!("test_index_{}", uuid::Uuid::new_v4());
            let create_params = CreateIndexParams {
                index_name: index_name.clone(),
                dimension: 4,
                metric: "cosine".to_string(),
            };
            
            // Create the index
            store.create_index(create_params)
                .await
                .expect("Failed to create index");
                
            // List indexes to confirm creation
            let indexes = store.list_indexes()
                .await
                .expect("Failed to list indexes");
                
            assert!(indexes.contains(&index_name));
            
            // Delete the index
            store.delete_index(&index_name)
                .await
                .expect("Failed to delete index");
        }
    }
    
    #[cfg(feature = "qdrant")]
    mod rag_tests {
        use std::sync::Arc;
        use async_trait::async_trait;
        use lomusai_rag::error::Result as RagResult;
        use lomusai_rag::embedding::EmbeddingProvider;
        use lomusai_rag::types::{Document, Metadata, RetrievalOptions, RetrievalResult};
        use lomusai_rag::retriever::VectorStore as RagVectorStore;
        use crate::rag::VectorStoreRetriever;
        use crate::qdrant::QdrantStore;
        
        // A simple mock embedding provider for testing
        struct MockEmbedder;
        
        #[async_trait]
        impl EmbeddingProvider for MockEmbedder {
            async fn embed_text(&self, _text: &str) -> RagResult<Vec<f32>> {
                // Return a simple 4-dimensional embedding for testing
                Ok(vec![0.1, 0.2, 0.3, 0.4])
            }
        }
        
        #[tokio::test]
        #[ignore] // Requires a Qdrant server
        async fn test_qdrant_vector_store_retriever() {
            use uuid::Uuid;
            
            // Create a unique index name for this test
            let index_name = format!("test_retriever_{}", Uuid::new_v4());
            
            // Create the Qdrant store
            let qdrant_url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());
            let store = QdrantStore::new(&qdrant_url, None).await.expect("Failed to create Qdrant store");
            
            // Create the retriever
            let mut retriever = VectorStoreRetriever::new(store, &index_name, 4, "cosine");
            
            // Create the embedder
            let embedder = Arc::new(MockEmbedder);
            
            // Ensure the index exists
            retriever.ensure_index().await.expect("Failed to ensure index");
            
            // Create a test document
            let mut doc = Document {
                id: "test1".to_string(),
                content: "This is a test document about vector search".to_string(),
                metadata: Metadata::new().with_source("test"),
                embedding: None,
            };
            
            // Embed the document
            embedder.embed_document(&mut doc).await.expect("Failed to embed document");
            
            // Add the document to the retriever
            retriever.add_document(doc.clone()).await.expect("Failed to add document");
            
            // Query with the same embedding
            let options = RetrievalOptions {
                limit: 5,
                threshold: None,
                filter: None,
            };
            
            // Since we're using a mock embedder that returns the same embedding for any text,
            // we can query with any text and get the same result
            let results = retriever.query_by_text("test query", &options, &*embedder)
                .await
                .expect("Failed to query");
            
            // Verify results
            assert_eq!(results.documents.len(), 1, "Expected 1 result");
            assert_eq!(results.documents[0].id, "test1");
            assert_eq!(results.documents[0].content, "This is a test document about vector search");
            
            // Get document by ID
            let get_result = retriever.get_document("test1").await.expect("Failed to get document");
            assert!(get_result.is_some(), "Document should exist");
            assert_eq!(get_result.unwrap().id, "test1");
            
            // Count documents
            let count = retriever.count_documents().await.expect("Failed to count documents");
            assert_eq!(count, 1, "Expected 1 document");
            
            // Delete the document
            retriever.delete_document("test1").await.expect("Failed to delete document");
            
            // Verify document is deleted
            let count_after = retriever.count_documents().await.expect("Failed to count documents");
            assert_eq!(count_after, 0, "Expected 0 documents after deletion");
            
            // Clean up - delete the test index
            let store = retriever.store.lock().await;
            let _ = store.delete_index(&index_name).await;
        }
    }
} 