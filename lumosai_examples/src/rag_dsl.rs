use lumosai_core::Result;
use lumosai_core::rag::{DocumentSource, RagPipelineBuilder, RagPipeline};
use lumos_macro::rag_pipeline;

#[tokio::main]
async fn main() -> Result<()> {
    println!("RAG Pipeline DSL示例");
    
    // 演示手动创建RAG管道
    println!("🔧 手动创建RAG管道示例");

    let manual_pipeline = RagPipelineBuilder::new("manual_pipeline")
        .add_source(DocumentSource::from_text("这是一个API文档示例。它包含了关于Lumos宏的使用说明和示例代码。"))
        .build()
        .await?;

    // 执行查询
    println!("对知识库执行查询...");

    let query = "如何使用Lumos宏创建工具？";
    println!("查询: {}", query);

    let result = manual_pipeline.query(query, 5).await?;

    println!("查询完成！找到 {} 个相关文档", result.documents.len());
    println!("查询结果:");
    println!("- 查询: {}", result.query);
    println!("- 上下文: {}", result.context);
    for (i, doc) in result.documents.iter().enumerate() {
        println!("文档 #{}: {}", i + 1, doc.id);
        println!("内容: {}", doc.content.chars().take(150).collect::<String>());
        if let Some(scores) = &result.scores {
            if i < scores.len() {
                println!("相关度: {:.3}", scores[i]);
            }
        }
        println!("---");
    }

    // 创建另一个RAG管道示例
    println!("\n📚 第二个RAG管道示例");

    let second_pipeline = RagPipelineBuilder::new("second_pipeline")
        .add_source(DocumentSource::from_text("另一个测试文档，用于演示手动创建的RAG管道。这个文档包含了更多的技术细节。"))
        .build()
        .await?;

    let second_result = second_pipeline.query("什么是技术细节？", 3).await?;

    println!("第二个管道查询结果:");
    println!("- 查询: {}", second_result.query);
    println!("- 找到 {} 个相关文档", second_result.documents.len());
    println!("- 上下文: {}", second_result.context);

    // 演示使用宏创建RAG管道（暂时注释掉，因为宏解析有问题）
    println!("\n🚀 宏功能开发中...");
    println!("宏语法示例:");
    println!(r#"
    rag_pipeline! {{
        name: "macro_pipeline",
        source: DocumentSource::from_text("文档内容"),
        pipeline: {{
            chunk: {{ chunk_size: 800 }},
            embed: {{ model: "text-embedding-ada-002" }},
            store: {{ db: "memory", collection: "docs" }}
        }}
    }}
    "#);

    println!("\n✅ RAG DSL 示例完成！");

    Ok(())
}