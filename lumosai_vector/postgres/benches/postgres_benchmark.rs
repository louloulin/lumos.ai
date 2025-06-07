//! PostgreSQL vector storage performance benchmarks
//! 
//! Run with: cd lumosai_vector/postgres && cargo bench postgres_benchmark

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;
use tokio::runtime::Runtime;
use lumosai_vector_core::prelude::*;
use lumosai_vector_postgres::PostgresVectorStorage;

// Helper function to generate test documents
fn generate_documents(count: usize, dimension: usize) -> Vec<Document> {
    (0..count)
        .map(|i| {
            let embedding: Vec<f32> = (0..dimension)
                .map(|j| ((i * dimension + j) as f32).sin())
                .collect();
            
            let mut metadata = HashMap::new();
            metadata.insert("id".to_string(), MetadataValue::Integer(i as i64));
            metadata.insert("category".to_string(), MetadataValue::String(format!("cat_{}", i % 10)));
            metadata.insert("score".to_string(), MetadataValue::Float((i as f32) / (count as f32)));
            
            Document {
                id: format!("doc_{}", i),
                content: format!("This is document number {} with some content for testing", i),
                embedding: Some(embedding),
                metadata,
            }
        })
        .collect()
}

// Helper function to generate query vector
fn generate_query_vector(dimension: usize) -> Vec<f32> {
    (0..dimension)
        .map(|i| (i as f32 * 0.1).cos())
        .collect()
}

async fn setup_storage() -> Result<PostgresVectorStorage> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost/bench_lumos_vector".to_string());
    
    let storage = PostgresVectorStorage::new(&database_url).await?;
    
    // Create test index
    let index_config = IndexConfig {
        name: "bench_index".to_string(),
        dimension: 384,
        metric: SimilarityMetric::Cosine,
        metadata: HashMap::new(),
    };
    
    // Clean up any existing index
    let _ = storage.delete_index("bench_index").await;
    
    storage.create_index(index_config).await?;
    
    Ok(storage)
}

fn bench_upsert_documents(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Skip if DATABASE_URL is not set
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping PostgreSQL benchmarks - DATABASE_URL not set");
        return;
    }
    
    let storage = rt.block_on(async {
        setup_storage().await.expect("Failed to setup storage")
    });
    
    let mut group = c.benchmark_group("postgres_upsert");
    
    for &doc_count in &[10, 100, 1000] {
        let documents = generate_documents(doc_count, 384);
        
        group.bench_with_input(
            BenchmarkId::new("documents", doc_count),
            &documents,
            |b, docs| {
                b.to_async(&rt).iter(|| async {
                    let result = storage.upsert_documents("bench_index", black_box(docs.clone())).await;
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

fn bench_search_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Skip if DATABASE_URL is not set
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping PostgreSQL benchmarks - DATABASE_URL not set");
        return;
    }
    
    let storage = rt.block_on(async {
        let storage = setup_storage().await.expect("Failed to setup storage");
        
        // Insert test data
        let documents = generate_documents(1000, 384);
        storage.upsert_documents("bench_index", documents).await.expect("Failed to insert documents");
        
        storage
    });
    
    let mut group = c.benchmark_group("postgres_search");
    
    for &top_k in &[1, 10, 50, 100] {
        let query_vector = generate_query_vector(384);
        
        group.bench_with_input(
            BenchmarkId::new("top_k", top_k),
            &top_k,
            |b, &k| {
                b.to_async(&rt).iter(|| async {
                    let request = SearchRequest {
                        index_name: "bench_index".to_string(),
                        query: SearchQuery::Vector(black_box(query_vector.clone())),
                        top_k: k,
                        filter: None,
                        include_metadata: true,
                        include_vectors: false,
                    };
                    
                    let result = storage.search(black_box(request)).await;
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_upsert_documents, bench_search_performance);
criterion_main!(benches);
