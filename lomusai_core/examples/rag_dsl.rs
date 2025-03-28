use lomusai_core::Result;
use lomusai_core::rag::{DocumentSource, RagPipeline};
use lumos_macro::rag_pipeline;

#[tokio::main]
async fn main() -> Result<()> {
    println!("RAG Pipeline DSL示例");
    
    // 使用rag_pipeline!宏定义一个知识库处理管道
    let kb = rag_pipeline! {
        name: "documentation_kb",
        
        source: DocumentSource::from_directory("./docs/api"),
        
        pipeline: {
            chunk: {
                chunk_size: 1000,
                chunk_overlap: 200,
                separator: "\n\n",
                strategy: "recursive"
            },
            
            embed: {
                model: "text-embedding-3-small",
                dimensions: 1536,
                max_retries: 3
            },
            
            store: {
                db: "memory",
                collection: "api_docs",
                options: {
                    "max_vectors": 10000,
                    "similarity": "cosine"
                }
            }
        },
        
        query_pipeline: {
            rerank: true,
            top_k: 5,
            filter: r#"{ "type": "api_reference" }"#,
            hybrid_search: {
                enabled: true,
                weight: {
                    semantic: 0.7,
                    keyword: 0.3
                }
            }
        }
    };

    // 执行查询
    println!("对知识库执行查询...");
    
    let query = "如何使用Lumos宏创建工具？";
    println!("查询: {}", query);
    
    let results = kb.query(query).await?;
    
    println!("查询完成！找到 {} 个相关文档", results.len());
    for (i, doc) in results.iter().enumerate() {
        println!("结果 #{}: {} (相关度: {})", i + 1, doc.title, doc.relevance);
        println!("摘要: {}", doc.content.chars().take(150).collect::<String>());
        println!("---");
    }
    
    // 创建一个更复杂的RAG管道，包含多个数据源
    let advanced_kb = rag_pipeline! {
        name: "product_knowledge",
        
        sources: [
            DocumentSource::from_directory("./docs/product"),
            DocumentSource::from_url("https://api.company.com/docs"),
            DocumentSource::from_database("postgresql://user:pass@localhost/product_db"),
        ],
        
        pipeline: {
            preprocess: {
                remove_headers: true,
                normalize_whitespace: true,
                extract_code_blocks: true
            },
            
            chunk: {
                chunk_size: 800,
                chunk_overlap: 150,
                strategy: "sentence"
            },
            
            embed: {
                model: "custom-embedding-model",
                dimensions: 768,
                batch_size: 32
            },
            
            store: {
                db: "pinecone",
                collection: "product_knowledge",
                connection_string: env!("PINECONE_API_KEY")
            }
        },
        
        query_pipeline: {
            rerank: {
                model: "rerank-model",
                top_n: 10
            },
            top_k: 3,
            metadata_boost: {
                "recency": 1.2,
                "popularity": 0.8
            }
        }
    };
    
    println!("\n高级RAG管道创建完成，准备处理多数据源...");
    
    Ok(())
} 