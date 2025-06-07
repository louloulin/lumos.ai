//! Integration tests for PostgreSQL vector storage

use std::collections::HashMap;
use lumosai_vector_core::prelude::*;
use lumosai_vector_postgres::{PostgresVectorStorage, PostgresConfig};

// Note: These tests require a running PostgreSQL instance with pgvector extension
// Set DATABASE_URL environment variable to run these tests

#[tokio::test]
#[ignore] // Ignore by default since it requires PostgreSQL setup
async fn test_postgres_basic_operations() -> Result<()> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost/test_lumos_vector".to_string());
    
    let storage = PostgresVectorStorage::new(&database_url).await?;
    
    // Create index
    let index_config = IndexConfig {
        name: "test_index".to_string(),
        dimension: 384,
        metric: SimilarityMetric::Cosine,
        metadata: HashMap::new(),
    };
    
    storage.create_index(index_config).await?;
    
    // Create test documents
    let documents = vec![
        Document {
            id: "doc1".to_string(),
            content: "This is the first document".to_string(),
            embedding: Some(vec![0.1; 384]),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), MetadataValue::String("tech".to_string()));
                meta.insert("score".to_string(), MetadataValue::Float(0.95));
                meta
            },
        },
        Document {
            id: "doc2".to_string(),
            content: "This is the second document".to_string(),
            embedding: Some(vec![0.2; 384]),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), MetadataValue::String("science".to_string()));
                meta.insert("score".to_string(), MetadataValue::Float(0.87));
                meta
            },
        },
    ];
    
    // Upsert documents
    let ids = storage.upsert_documents("test_index", documents.clone()).await?;
    assert_eq!(ids.len(), 2);
    
    // Search
    let search_request = SearchRequest {
        index_name: "test_index".to_string(),
        query: SearchQuery::Vector(vec![0.15; 384]),
        top_k: 10,
        filter: None,
        include_metadata: true,
        include_vectors: false,
    };
    
    let response = storage.search(search_request).await?;
    assert_eq!(response.results.len(), 2);
    
    // Get documents
    let retrieved_docs = storage.get_documents("test_index", vec!["doc1".to_string()], true).await?;
    assert_eq!(retrieved_docs.len(), 1);
    assert_eq!(retrieved_docs[0].id, "doc1");
    assert!(retrieved_docs[0].embedding.is_some());
    
    // Delete documents
    storage.delete_documents("test_index", vec!["doc1".to_string()]).await?;
    
    // Verify deletion
    let remaining_docs = storage.get_documents("test_index", vec!["doc1".to_string()], false).await?;
    assert_eq!(remaining_docs.len(), 0);
    
    // Clean up
    storage.delete_index("test_index").await?;
    
    Ok(())
}

#[tokio::test]
#[ignore] // Ignore by default since it requires PostgreSQL setup
async fn test_postgres_configuration() -> Result<()> {
    let config = PostgresConfig::new("postgresql://localhost/test")
        .with_performance(lumosai_vector_postgres::config::PerformanceConfig {
            batch_size: 500,
            index_type: lumosai_vector_postgres::config::VectorIndexType::Hnsw,
            index_params: lumosai_vector_postgres::config::IndexParams::default(),
            use_prepared_statements: true,
        });
    
    assert_eq!(config.database_url, "postgresql://localhost/test");
    assert_eq!(config.performance.batch_size, 500);
    
    Ok(())
}

#[tokio::test]
#[ignore] // Ignore by default since it requires PostgreSQL setup
async fn test_postgres_health_check() -> Result<()> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost/test_lumos_vector".to_string());
    
    let storage = PostgresVectorStorage::new(&database_url).await?;
    
    // Health check should pass
    storage.health_check().await?;
    
    // Backend info should be correct
    let info = storage.backend_info();
    assert_eq!(info.name, "PostgreSQL");
    assert!(info.features.contains(&"persistent".to_string()));
    assert!(info.features.contains(&"transactions".to_string()));
    
    Ok(())
}

#[test]
fn test_postgres_config_defaults() {
    let config = PostgresConfig::default();
    
    assert_eq!(config.database_url, "postgresql://localhost/lumos_vector");
    assert_eq!(config.pool.max_connections, 10);
    assert_eq!(config.table.schema, "public");
    assert_eq!(config.performance.batch_size, 1000);
    
    // Test table name generation
    let table_name = config.table_name("test_index");
    assert_eq!(table_name, "public.lumos_test_index");
    
    // Test index name generation
    let index_name = config.index_name("test_index", "embedding");
    assert_eq!(index_name, "lumos_test_index_embedding_idx");
}

#[test]
fn test_postgres_error_conversion() {
    use lumosai_vector_postgres::PostgresError;
    
    let postgres_err = PostgresError::Connection("test connection error".to_string());
    let vector_err: VectorError = postgres_err.into();
    
    match vector_err {
        VectorError::ConnectionFailed { .. } => {
            // Expected
        },
        _ => panic!("Expected ConnectionFailed error"),
    }
}

#[test]
fn test_vector_index_sql_generation() {
    use lumosai_vector_postgres::config::{VectorIndexType, IndexParams};
    
    let params = IndexParams::default();
    
    // Test HNSW index SQL
    let hnsw_sql = VectorIndexType::Hnsw.create_index_sql(
        "test_table", 
        "test_idx", 
        &params
    );
    assert!(hnsw_sql.contains("USING hnsw"));
    assert!(hnsw_sql.contains("vector_cosine_ops"));
    
    // Test IVFFlat index SQL
    let ivf_sql = VectorIndexType::IvfFlat.create_index_sql(
        "test_table", 
        "test_idx", 
        &params
    );
    assert!(ivf_sql.contains("USING ivfflat"));
    assert!(ivf_sql.contains("vector_cosine_ops"));
    
    // Test no index
    let no_index_sql = VectorIndexType::None.create_index_sql(
        "test_table", 
        "test_idx", 
        &params
    );
    assert!(no_index_sql.is_empty());
}

#[test]
fn test_search_params_sql_generation() {
    use lumosai_vector_postgres::config::{VectorIndexType, IndexParams};
    
    let params = IndexParams::default();
    
    // Test HNSW search params
    let hnsw_params = VectorIndexType::Hnsw.search_params_sql(&params);
    assert_eq!(hnsw_params.len(), 1);
    assert!(hnsw_params[0].contains("SET hnsw.ef_search"));
    
    // Test IVFFlat search params
    let ivf_params = VectorIndexType::IvfFlat.search_params_sql(&params);
    assert_eq!(ivf_params.len(), 1);
    assert!(ivf_params[0].contains("SET ivfflat.probes"));
    
    // Test no index search params
    let no_params = VectorIndexType::None.search_params_sql(&params);
    assert!(no_params.is_empty());
}
