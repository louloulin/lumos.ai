use anyhow::Result;
use lumosai::SimpleAgent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocumentChunk {
    id: String,
    content: String,
    source: String,
    chunk_index: usize,
}

#[derive(Debug)]
struct DocumentQA {
    chunks: Vec<DocumentChunk>,
    agent: SimpleAgent,
}

impl DocumentQA {
    async fn new() -> Result<Self> {
        let system_prompt = r#"
你是一个文档问答助手。基于提供的文档片段回答用户问题。
请确保：
1. 回答基于提供的文档内容
2. 如果文档中没有相关信息，请说明
3. 引用具体的文档片段
4. 提供准确和有用的回答
"#;
        
        let agent = lumosai::agent::simple("gpt-3.5-turbo", system_prompt).await.map_err(|e| anyhow::anyhow!(e))?;
        
        Ok(Self {
            chunks: Vec::new(),
            agent,
        })
    }
    
    fn add_document(&mut self, source: &str, content: &str, chunk_size: usize) {
        let words: Vec<&str> = content.split_whitespace().collect();
        let chunks: Vec<_> = words.chunks(chunk_size).collect();
        
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_content = chunk.join(" ");
            let chunk_id = format!("{}_{}", source, i);
            
            self.chunks.push(DocumentChunk {
                id: chunk_id,
                content: chunk_content,
                source: source.to_string(),
                chunk_index: i,
            });
        }
    }
    
    fn search_relevant_chunks(&self, query: &str, limit: usize) -> Vec<&DocumentChunk> {
        let query_lower = query.to_lowercase();
        let mut scored_chunks: Vec<(&DocumentChunk, f32)> = self.chunks
            .iter()
            .filter_map(|chunk| {
                let content_lower = chunk.content.to_lowercase();
                if content_lower.contains(&query_lower) {
                    // 简单的相关性评分
                    let score = query_lower.split_whitespace()
                        .map(|word| if content_lower.contains(word) { 1.0 } else { 0.0 })
                        .sum::<f32>() / query_lower.split_whitespace().count() as f32;
                    Some((chunk, score))
                } else {
                    None
                }
            })
            .collect();
        
        scored_chunks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored_chunks.into_iter().take(limit).map(|(chunk, _)| chunk).collect()
    }
    
    async fn answer_question(&self, question: &str) -> Result<String> {
        let relevant_chunks = self.search_relevant_chunks(question, 3);
        
        if relevant_chunks.is_empty() {
            return Ok("抱歉，我在文档中没有找到与您问题相关的信息。".to_string());
        }
        
        let context = relevant_chunks
            .iter()
            .map(|chunk| format!("来源：{}\n内容：{}", chunk.source, chunk.content))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");
        
        let prompt = format!(
            "基于以下文档片段回答问题：\n\n{}\n\n问题：{}\n\n请提供准确的回答并引用相关的文档片段。",
            context, question
        );
        
        self.agent.chat(&prompt).await.map_err(|e| anyhow::anyhow!(e))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("📄 文档问答演示");
    
    // 创建文档问答系统
    let mut qa = DocumentQA::new().await?;
    
    // 添加示例文档
    println!("\n📚 添加文档...");
    
    let rust_doc = r#"
    Rust 是一种系统编程语言，专注于安全、速度和并发。Rust 通过所有权系统防止内存安全问题，
    如空指针解引用、缓冲区溢出和内存泄漏。所有权系统是 Rust 的核心特性，它在编译时检查内存安全，
    无需垃圾回收器。Rust 还提供零成本抽象，意味着高级特性不会带来运行时开销。
    Rust 的包管理器 Cargo 简化了依赖管理和项目构建。Rust 广泛应用于系统编程、
    Web 开发、网络编程和区块链开发等领域。
    "#;
    
    let ai_doc = r#"
    人工智能（AI）是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的系统。
    机器学习是 AI 的一个子集，它使计算机能够在没有明确编程的情况下学习和改进。
    深度学习是机器学习的一个分支，使用多层神经网络来模拟人脑的工作方式。
    自然语言处理（NLP）是 AI 的一个领域，专注于计算机与人类语言之间的交互。
    计算机视觉使计算机能够理解和解释视觉信息。强化学习是一种机器学习方法，
    通过与环境交互来学习最优行为策略。
    "#;
    
    let web_doc = r#"
    Web 开发涉及创建和维护网站和 Web 应用程序。前端开发专注于用户界面和用户体验，
    使用 HTML、CSS 和 JavaScript 等技术。后端开发处理服务器端逻辑、数据库和 API。
    全栈开发者同时掌握前端和后端技术。现代 Web 开发框架如 React、Vue.js 和 Angular
    简化了前端开发。Node.js 使 JavaScript 能够用于后端开发。RESTful API 和 GraphQL
    是常用的 API 设计模式。响应式设计确保网站在不同设备上都能良好显示。
    "#;
    
    qa.add_document("Rust编程语言", rust_doc, 50);
    qa.add_document("人工智能概述", ai_doc, 50);
    qa.add_document("Web开发基础", web_doc, 50);
    
    println!("✅ 已添加 {} 个文档片段", qa.chunks.len());
    
    // 测试问答
    println!("\n❓ 测试文档问答:");
    
    let questions = vec![
        "什么是 Rust 的所有权系统？",
        "机器学习和深度学习有什么区别？",
        "前端开发使用哪些技术？",
        "什么是零成本抽象？",
        "自然语言处理是什么？",
        "响应式设计的作用是什么？",
    ];
    
    for question in questions {
        println!("\n" + "=".repeat(60).as_str());
        println!("问题: {}", question);
        println!("-".repeat(60));
        
        // 显示相关文档片段
        let relevant_chunks = qa.search_relevant_chunks(question, 2);
        if !relevant_chunks.is_empty() {
            println!("📋 相关文档片段:");
            for (i, chunk) in relevant_chunks.iter().enumerate() {
                println!("{}. 来源: {} (片段 {})", i + 1, chunk.source, chunk.chunk_index);
                println!("   内容: {}...", 
                    if chunk.content.len() > 100 {
                        &chunk.content[..100]
                    } else {
                        &chunk.content
                    }
                );
            }
            println!();
        }
        
        // 生成回答
        match qa.answer_question(question).await {
            Ok(answer) => {
                println!("🤖 回答:");
                println!("{}", answer);
            }
            Err(e) => {
                println!("❌ 错误: {}", e);
            }
        }
    }
    
    println!("\n" + "=".repeat(60).as_str());
    println!("✨ 文档问答演示完成！");
    
    // 显示统计信息
    let mut source_counts = HashMap::new();
    for chunk in &qa.chunks {
        *source_counts.entry(&chunk.source).or_insert(0) += 1;
    }
    
    println!("\n📊 文档统计:");
    for (source, count) in source_counts {
        println!("  {}: {} 个片段", source, count);
    }
    
    Ok(())
}
