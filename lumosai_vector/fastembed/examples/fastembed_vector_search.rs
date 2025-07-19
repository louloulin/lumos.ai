//! Vector search example using FastEmbed
//! 
//! This example demonstrates how to use FastEmbed embeddings for
//! semantic search and similarity matching.

use lumosai_vector_fastembed::{FastEmbedProvider, FastEmbedModel};
use lumosai_vector_core::traits::EmbeddingModel;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Document {
    id: String,
    title: String,
    content: String,
    embedding: Option<Vec<f32>>,
}

impl Document {
    fn new(id: &str, title: &str, content: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            embedding: None,
        }
    }
}

#[derive(Debug)]
struct SearchResult {
    document: Document,
    similarity: f32,
}

struct VectorSearchEngine {
    provider: FastEmbedProvider,
    documents: HashMap<String, Document>,
}

impl VectorSearchEngine {
    async fn new(model: FastEmbedModel) -> Result<Self, Box<dyn std::error::Error>> {
        let provider = FastEmbedProvider::with_model(model).await?;
        
        Ok(Self {
            provider,
            documents: HashMap::new(),
        })
    }
    
    async fn add_document(&mut self, mut document: Document) -> Result<(), Box<dyn std::error::Error>> {
        // Generate embedding for the document content
        let embedding = self.provider.embed_text(&document.content).await?;
        document.embedding = Some(embedding);
        
        self.documents.insert(document.id.clone(), document);
        Ok(())
    }
    
    async fn add_documents(&mut self, documents: Vec<Document>) -> Result<(), Box<dyn std::error::Error>> {
        // Extract content for batch embedding
        let contents: Vec<String> = documents.iter()
            .map(|doc| doc.content.clone())
            .collect();
        
        // Generate embeddings in batch
        let embeddings = self.provider.embed_batch(&contents).await?;
        
        // Store documents with embeddings
        for (mut document, embedding) in documents.into_iter().zip(embeddings) {
            document.embedding = Some(embedding);
            self.documents.insert(document.id.clone(), document);
        }
        
        Ok(())
    }
    
    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        // Generate embedding for the query
        let query_embedding = self.provider.embed_text(query).await?;
        
        // Calculate similarities with all documents
        let mut results = Vec::new();
        
        for document in self.documents.values() {
            if let Some(doc_embedding) = &document.embedding {
                let similarity = cosine_similarity(&query_embedding, doc_embedding);
                results.push(SearchResult {
                    document: document.clone(),
                    similarity,
                });
            }
        }
        
        // Sort by similarity (descending)
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        
        // Return top-k results
        results.truncate(top_k);
        Ok(results)
    }
    
    fn document_count(&self) -> usize {
        self.documents.len()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🚀 LumosAI FastEmbed Vector Search Example");
    println!("==========================================");
    
    // Create search engine
    let mut search_engine = VectorSearchEngine::new(FastEmbedModel::BGESmallENV15).await?;
    
    // Example 1: Add sample documents
    println!("\n📝 Example 1: Building document index");
    
    let documents = create_sample_documents();
    println!("📚 Adding {} documents to the index...", documents.len());
    
    let start = std::time::Instant::now();
    search_engine.add_documents(documents).await?;
    let indexing_time = start.elapsed();
    
    println!("✅ Indexed {} documents in {:?}", 
             search_engine.document_count(), indexing_time);
    
    // Example 2: Semantic search
    println!("\n📝 Example 2: Semantic search");
    
    let queries = vec![
        "machine learning algorithms",
        "web development frameworks",
        "database optimization",
        "artificial intelligence applications",
        "cloud computing services",
    ];
    
    for query in queries {
        println!("\n🔍 Query: \"{}\"", query);
        
        let start = std::time::Instant::now();
        let results = search_engine.search(query, 3).await?;
        let search_time = start.elapsed();
        
        println!("   Found {} results in {:?}:", results.len(), search_time);
        
        for (i, result) in results.iter().enumerate() {
            println!("   {}. {} (similarity: {:.3})", 
                     i + 1, 
                     result.document.title, 
                     result.similarity);
            println!("      {}", 
                     truncate_text(&result.document.content, 80));
        }
    }
    
    // Example 3: Similarity threshold filtering
    println!("\n📝 Example 3: Similarity threshold filtering");
    
    let query = "programming languages";
    let all_results = search_engine.search(query, 10).await?;
    
    let thresholds = vec![0.8, 0.7, 0.6, 0.5];
    
    for threshold in thresholds {
        let filtered_results: Vec<_> = all_results.iter()
            .filter(|r| r.similarity >= threshold)
            .collect();
        
        println!("🎯 Threshold {:.1}: {} results", threshold, filtered_results.len());
        
        for result in filtered_results.iter().take(2) {
            println!("   • {} ({:.3})", result.document.title, result.similarity);
        }
    }
    
    // Example 4: Document similarity matrix
    println!("\n📝 Example 4: Document similarity analysis");
    
    await_similarity_analysis(&search_engine).await?;
    
    // Example 5: Multi-language search (if using multilingual model)
    println!("\n📝 Example 5: Multilingual search");
    
    match VectorSearchEngine::new(FastEmbedModel::MultilingualE5Small).await {
        Ok(mut ml_engine) => {
            let ml_documents = create_multilingual_documents();
            ml_engine.add_documents(ml_documents).await?;
            
            let ml_queries = vec![
                ("English", "artificial intelligence"),
                ("Spanish", "inteligencia artificial"),
                ("French", "intelligence artificielle"),
            ];
            
            for (lang, query) in ml_queries {
                println!("\n🌍 {} query: \"{}\"", lang, query);
                let results = ml_engine.search(query, 2).await?;
                
                for result in results {
                    println!("   • {} ({:.3})", result.document.title, result.similarity);
                }
            }
        }
        Err(e) => {
            println!("⚠️  Multilingual model not available: {}", e);
        }
    }
    
    println!("\n🎉 Vector search examples completed!");
    
    Ok(())
}

async fn await_similarity_analysis(engine: &VectorSearchEngine) -> Result<(), Box<dyn std::error::Error>> {
    let sample_docs: Vec<_> = engine.documents.values().take(5).collect();
    
    println!("📊 Document similarity matrix (top 5 documents):");
    print!("     ");
    for (i, doc) in sample_docs.iter().enumerate() {
        print!("{:>8}", format!("Doc{}", i + 1));
    }
    println!();
    
    for (i, doc1) in sample_docs.iter().enumerate() {
        print!("Doc{} ", i + 1);
        
        for doc2 in sample_docs.iter() {
            let similarity = if let (Some(emb1), Some(emb2)) = (&doc1.embedding, &doc2.embedding) {
                cosine_similarity(emb1, emb2)
            } else {
                0.0
            };
            print!("{:>8.3}", similarity);
        }
        println!();
    }
    
    Ok(())
}

fn create_sample_documents() -> Vec<Document> {
    vec![
        Document::new("1", "Introduction to Machine Learning", 
                     "Machine learning is a subset of artificial intelligence that focuses on algorithms that can learn from data without being explicitly programmed."),
        
        Document::new("2", "Web Development with React", 
                     "React is a popular JavaScript library for building user interfaces, especially for web applications with dynamic content."),
        
        Document::new("3", "Database Optimization Techniques", 
                     "Database optimization involves improving query performance, indexing strategies, and data structure design for efficient data retrieval."),
        
        Document::new("4", "Deep Learning Neural Networks", 
                     "Deep learning uses neural networks with multiple layers to learn complex patterns in data for tasks like image recognition and natural language processing."),
        
        Document::new("5", "Cloud Computing Architecture", 
                     "Cloud computing provides on-demand access to computing resources including servers, storage, databases, and software over the internet."),
        
        Document::new("6", "Python Programming Fundamentals", 
                     "Python is a high-level programming language known for its simplicity and readability, widely used in data science and web development."),
        
        Document::new("7", "Microservices Design Patterns", 
                     "Microservices architecture breaks down applications into small, independent services that communicate through well-defined APIs."),
        
        Document::new("8", "Data Science and Analytics", 
                     "Data science combines statistics, programming, and domain expertise to extract insights and knowledge from structured and unstructured data."),
        
        Document::new("9", "Cybersecurity Best Practices", 
                     "Cybersecurity involves protecting digital systems, networks, and data from cyber threats through various security measures and protocols."),
        
        Document::new("10", "Mobile App Development", 
                     "Mobile app development involves creating software applications for mobile devices using platforms like iOS, Android, and cross-platform frameworks."),
    ]
}

fn create_multilingual_documents() -> Vec<Document> {
    vec![
        Document::new("ml1", "Artificial Intelligence", 
                     "Artificial intelligence is the simulation of human intelligence in machines that are programmed to think and learn."),
        
        Document::new("ml2", "Inteligencia Artificial", 
                     "La inteligencia artificial es la simulación de la inteligencia humana en máquinas programadas para pensar y aprender."),
        
        Document::new("ml3", "Intelligence Artificielle", 
                     "L'intelligence artificielle est la simulation de l'intelligence humaine dans des machines programmées pour penser et apprendre."),
        
        Document::new("ml4", "Machine Learning", 
                     "Machine learning enables computers to learn and improve from experience without being explicitly programmed."),
        
        Document::new("ml5", "Aprendizaje Automático", 
                     "El aprendizaje automático permite a las computadoras aprender y mejorar a partir de la experiencia sin ser programadas explícitamente."),
    ]
}

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

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len])
    }
}
