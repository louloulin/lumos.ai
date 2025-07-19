//! Vector search example for LanceDB
//!
//! This example demonstrates:
//! - Different similarity metrics
//! - Complex metadata filtering
//! - Search result analysis
//! - Performance comparison

use std::time::Instant;
use lumosai_vector_lancedb::{LanceDbStorage, LanceDbConfig};
use lumosai_vector_core::{
    traits::VectorStorage,
    types::{Document, IndexConfig, SearchRequest, SimilarityMetric, FilterCondition, MetadataValue},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🚀 LanceDB Vector Search Example");
    println!("=" * 50);
    
    // 1. Setup storage and index
    println!("\n📦 Setting up LanceDB storage...");
    let config = LanceDbConfig::local("./example_data/lancedb_search");
    let storage = LanceDbStorage::new(config).await?;
    
    let index_config = IndexConfig::new("search_documents", 384)
        .with_metric(SimilarityMetric::Cosine)
        .with_description("Vector search example index");
    
    storage.create_index(index_config).await?;
    println!("✅ Storage and index created");
    
    // 2. Insert diverse documents for search testing
    println!("\n📄 Inserting diverse documents...");
    let documents = create_diverse_documents();
    storage.upsert_documents("search_documents", documents).await?;
    println!("✅ Inserted {} documents", 20);
    
    // 3. Demonstrate different similarity metrics
    println!("\n🔍 Testing different similarity metrics...");
    let query_vector = generate_technology_embedding();
    
    let metrics = vec![
        SimilarityMetric::Cosine,
        SimilarityMetric::Euclidean,
        SimilarityMetric::DotProduct,
    ];
    
    for metric in metrics {
        println!("\n📊 Using {:?} similarity:", metric);
        
        let search_request = SearchRequest {
            index_name: "search_documents".to_string(),
            vector: query_vector.clone(),
            top_k: 5,
            similarity_metric: Some(metric),
            filter: None,
            include_metadata: true,
        };
        
        let start_time = Instant::now();
        let response = storage.search(search_request).await?;
        let search_time = start_time.elapsed();
        
        println!("   Search time: {:.2}ms", search_time.as_millis());
        for (i, result) in response.results.iter().enumerate() {
            let category = result.metadata.as_ref()
                .and_then(|m| m.get("category"))
                .map(|v| format!("{:?}", v))
                .unwrap_or_else(|| "unknown".to_string());
            
            println!("   {}. {} (score: {:.4}, category: {})", 
                i + 1, result.id, result.score, category);
        }
    }
    
    // 4. Complex metadata filtering examples
    println!("\n🔍 Testing complex metadata filters...");
    
    // Filter 1: Technology documents with high priority
    println!("\n📋 Filter 1: Technology + High Priority");
    let tech_high_priority = FilterCondition::And(vec![
        FilterCondition::Eq("category".to_string(), MetadataValue::String("technology".to_string())),
        FilterCondition::Eq("priority".to_string(), MetadataValue::String("high".to_string())),
    ]);
    
    let filtered_search = SearchRequest {
        index_name: "search_documents".to_string(),
        vector: query_vector.clone(),
        top_k: 10,
        similarity_metric: Some(SimilarityMetric::Cosine),
        filter: Some(tech_high_priority),
        include_metadata: true,
    };
    
    let response = storage.search(filtered_search).await?;
    println!("   Found {} results", response.results.len());
    for result in &response.results {
        println!("   - {}: score {:.4}", result.id, result.score);
    }
    
    // Filter 2: Documents with word count > 50 OR category = science
    println!("\n📋 Filter 2: Word Count > 50 OR Science Category");
    let complex_filter = FilterCondition::Or(vec![
        FilterCondition::Gt("word_count".to_string(), MetadataValue::Integer(50)),
        FilterCondition::Eq("category".to_string(), MetadataValue::String("science".to_string())),
    ]);
    
    let complex_search = SearchRequest {
        index_name: "search_documents".to_string(),
        vector: query_vector.clone(),
        top_k: 10,
        similarity_metric: Some(SimilarityMetric::Cosine),
        filter: Some(complex_filter),
        include_metadata: true,
    };
    
    let response = storage.search(complex_search).await?;
    println!("   Found {} results", response.results.len());
    for result in &response.results {
        let word_count = result.metadata.as_ref()
            .and_then(|m| m.get("word_count"))
            .map(|v| format!("{:?}", v))
            .unwrap_or_else(|| "unknown".to_string());
        println!("   - {}: score {:.4}, words: {}", result.id, result.score, word_count);
    }
    
    // Filter 3: Text contains specific keywords
    println!("\n📋 Filter 3: Content Contains 'machine' or 'learning'");
    let keyword_filter = FilterCondition::Or(vec![
        FilterCondition::Contains("content".to_string(), "machine".to_string()),
        FilterCondition::Contains("content".to_string(), "learning".to_string()),
    ]);
    
    let keyword_search = SearchRequest {
        index_name: "search_documents".to_string(),
        vector: query_vector.clone(),
        top_k: 10,
        similarity_metric: Some(SimilarityMetric::Cosine),
        filter: Some(keyword_filter),
        include_metadata: true,
    };
    
    let response = storage.search(keyword_search).await?;
    println!("   Found {} results", response.results.len());
    for result in &response.results {
        if let Some(doc) = &result.document {
            let content_preview = doc.content.as_deref()
                .map(|c| if c.len() > 50 { &c[..50] } else { c })
                .unwrap_or("No content");
            println!("   - {}: \"{}...\"", result.id, content_preview);
        }
    }
    
    // 5. Performance comparison: with vs without filters
    println!("\n⚡ Performance Comparison: Filtered vs Unfiltered Search");
    
    let num_queries = 50;
    
    // Unfiltered search performance
    let unfiltered_start = Instant::now();
    for i in 0..num_queries {
        let query = generate_sample_embedding(384, i as u64);
        let search_request = SearchRequest {
            index_name: "search_documents".to_string(),
            vector: query,
            top_k: 10,
            similarity_metric: Some(SimilarityMetric::Cosine),
            filter: None,
            include_metadata: false,
        };
        storage.search(search_request).await?;
    }
    let unfiltered_time = unfiltered_start.elapsed();
    
    // Filtered search performance
    let filter = FilterCondition::In(
        "category".to_string(),
        vec![
            MetadataValue::String("technology".to_string()),
            MetadataValue::String("science".to_string()),
        ],
    );
    
    let filtered_start = Instant::now();
    for i in 0..num_queries {
        let query = generate_sample_embedding(384, i as u64);
        let search_request = SearchRequest {
            index_name: "search_documents".to_string(),
            vector: query,
            top_k: 10,
            similarity_metric: Some(SimilarityMetric::Cosine),
            filter: Some(filter.clone()),
            include_metadata: false,
        };
        storage.search(search_request).await?;
    }
    let filtered_time = filtered_start.elapsed();
    
    println!("   Unfiltered: {:.2}ms avg ({} queries)", 
        unfiltered_time.as_millis() as f64 / num_queries as f64, num_queries);
    println!("   Filtered: {:.2}ms avg ({} queries)", 
        filtered_time.as_millis() as f64 / num_queries as f64, num_queries);
    println!("   Overhead: {:.1}%", 
        (filtered_time.as_millis() as f64 / unfiltered_time.as_millis() as f64 - 1.0) * 100.0);
    
    // 6. Search result analysis
    println!("\n📊 Search Result Analysis");
    
    let analysis_query = generate_technology_embedding();
    let analysis_search = SearchRequest {
        index_name: "search_documents".to_string(),
        vector: analysis_query,
        top_k: 20, // Get all documents
        similarity_metric: Some(SimilarityMetric::Cosine),
        filter: None,
        include_metadata: true,
    };
    
    let response = storage.search(analysis_search).await?;
    
    // Analyze score distribution
    let scores: Vec<f32> = response.results.iter().map(|r| r.score).collect();
    let max_score = scores.iter().fold(0.0f32, |a, &b| a.max(b));
    let min_score = scores.iter().fold(1.0f32, |a, &b| a.min(b));
    let avg_score = scores.iter().sum::<f32>() / scores.len() as f32;
    
    println!("   Score distribution:");
    println!("   - Max: {:.4}", max_score);
    println!("   - Min: {:.4}", min_score);
    println!("   - Avg: {:.4}", avg_score);
    println!("   - Range: {:.4}", max_score - min_score);
    
    // Analyze by category
    let mut category_scores: std::collections::HashMap<String, Vec<f32>> = std::collections::HashMap::new();
    for result in &response.results {
        if let Some(metadata) = &result.metadata {
            if let Some(MetadataValue::String(category)) = metadata.get("category") {
                category_scores.entry(category.clone()).or_insert_with(Vec::new).push(result.score);
            }
        }
    }
    
    println!("   Average scores by category:");
    for (category, scores) in category_scores {
        let avg = scores.iter().sum::<f32>() / scores.len() as f32;
        println!("   - {}: {:.4} ({} docs)", category, avg, scores.len());
    }
    
    println!("\n🎉 Vector search example completed successfully!");
    println!("💡 Key insights:");
    println!("   - Different similarity metrics can yield different results");
    println!("   - Complex filters enable precise document retrieval");
    println!("   - Performance overhead of filtering is typically minimal");
    println!("   - Score analysis helps understand result quality");
    
    Ok(())
}

/// Create diverse documents for search testing
fn create_diverse_documents() -> Vec<Document> {
    vec![
        // Technology documents
        Document::new("tech1", "Machine learning algorithms are revolutionizing artificial intelligence and data science applications")
            .with_embedding(generate_technology_embedding())
            .with_metadata("category", "technology")
            .with_metadata("priority", "high")
            .with_metadata("word_count", 12i64),
        
        Document::new("tech2", "Rust programming language offers memory safety without garbage collection")
            .with_embedding(generate_technology_embedding())
            .with_metadata("category", "technology")
            .with_metadata("priority", "medium")
            .with_metadata("word_count", 10i64),
        
        Document::new("tech3", "Cloud computing platforms enable scalable and distributed application deployment")
            .with_embedding(generate_technology_embedding())
            .with_metadata("category", "technology")
            .with_metadata("priority", "high")
            .with_metadata("word_count", 10i64),
        
        // Science documents
        Document::new("sci1", "Quantum mechanics describes the behavior of matter and energy at the molecular, atomic, nuclear, and even smaller microscopic levels")
            .with_embedding(generate_science_embedding())
            .with_metadata("category", "science")
            .with_metadata("priority", "high")
            .with_metadata("word_count", 20i64),
        
        Document::new("sci2", "DNA sequencing technologies have advanced rapidly, enabling personalized medicine and genetic research")
            .with_embedding(generate_science_embedding())
            .with_metadata("category", "science")
            .with_metadata("priority", "medium")
            .with_metadata("word_count", 13i64),
        
        Document::new("sci3", "Climate change research shows significant impacts on global weather patterns and ecosystems")
            .with_embedding(generate_science_embedding())
            .with_metadata("category", "science")
            .with_metadata("priority", "high")
            .with_metadata("word_count", 12i64),
        
        // Business documents
        Document::new("biz1", "Digital transformation strategies are essential for modern business competitiveness and growth")
            .with_embedding(generate_business_embedding())
            .with_metadata("category", "business")
            .with_metadata("priority", "medium")
            .with_metadata("word_count", 12i64),
        
        Document::new("biz2", "Supply chain optimization reduces costs and improves customer satisfaction")
            .with_embedding(generate_business_embedding())
            .with_metadata("category", "business")
            .with_metadata("priority", "low")
            .with_metadata("word_count", 10i64),
        
        // Add more documents for better testing...
        Document::new("mixed1", "Artificial intelligence applications in business are transforming customer service and decision making processes")
            .with_embedding(generate_mixed_embedding())
            .with_metadata("category", "technology")
            .with_metadata("priority", "high")
            .with_metadata("word_count", 15i64),
    ]
}

/// Generate technology-focused embedding
fn generate_technology_embedding() -> Vec<f32> {
    generate_sample_embedding(384, 12345)
}

/// Generate science-focused embedding
fn generate_science_embedding() -> Vec<f32> {
    generate_sample_embedding(384, 67890)
}

/// Generate business-focused embedding
fn generate_business_embedding() -> Vec<f32> {
    generate_sample_embedding(384, 54321)
}

/// Generate mixed-topic embedding
fn generate_mixed_embedding() -> Vec<f32> {
    generate_sample_embedding(384, 98765)
}

/// Generate a sample embedding vector
fn generate_sample_embedding(dimension: usize, seed: u64) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let hash = hasher.finish();
    
    let mut embedding = Vec::with_capacity(dimension);
    let mut current_hash = hash;
    
    for i in 0..dimension {
        let value = ((current_hash.wrapping_add(i as u64) % 1000) as f32 / 500.0) - 1.0;
        embedding.push(value);
        current_hash = current_hash.wrapping_mul(1103515245).wrapping_add(12345);
    }
    
    // Normalize the vector
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for x in &mut embedding {
            *x /= magnitude;
        }
    }
    
    embedding
}
