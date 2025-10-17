use anyhow::Result;
use lumosai::SimpleAgent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Document {
    id: String,
    title: String,
    content: String,
    doc_type: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug)]
struct KnowledgeBase {
    documents: HashMap<String, Document>,
    agent: SimpleAgent,
}

impl KnowledgeBase {
    async fn new() -> Result<Self> {
        let system_prompt = "你是一个知识库助手，能够基于文档内容回答问题。";
        let agent = lumosai::agent::simple("gpt-3.5-turbo", system_prompt).await.map_err(|e| anyhow::anyhow!(e))?;
        
        Ok(Self {
            documents: HashMap::new(),
            agent,
        })
    }
    
    fn add_document(&mut self, title: &str, content: &str, doc_type: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let document = Document {
            id: id.clone(),
            title: title.to_string(),
            content: content.to_string(),
            doc_type: doc_type.to_string(),
            created_at: Utc::now(),
        };
        
        self.documents.insert(id.clone(), document);
        id
    }
    
    fn search(&self, query: &str) -> Vec<&Document> {
        let query_lower = query.to_lowercase();
        self.documents
            .values()
            .filter(|doc| {
                doc.title.to_lowercase().contains(&query_lower) ||
                doc.content.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
    
    async fn answer(&self, question: &str) -> Result<String> {
        let relevant_docs = self.search(question);
        
        if relevant_docs.is_empty() {
            return Ok("抱歉，我没有找到相关的文档信息。".to_string());
        }
        
        let context = relevant_docs
            .iter()
            .map(|doc| format!("标题：{}\n内容：{}", doc.title, doc.content))
            .collect::<Vec<_>>()
            .join("\n\n");
        
        let prompt = format!(
            "基于以下文档回答问题：\n\n{}\n\n问题：{}",
            context, question
        );
        
        self.agent.chat(&prompt).await.map_err(|e| anyhow::anyhow!(e))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 知识库演示");
    
    // 创建知识库
    let mut kb = KnowledgeBase::new().await?;
    
    // 添加文档
    println!("\n📚 添加文档到知识库...");
    
    kb.add_document(
        "人工智能基础",
        "人工智能（AI）是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的系统。",
        "技术"
    );
    
    kb.add_document(
        "机器学习概述",
        "机器学习是人工智能的一个子集，它使计算机能够在没有明确编程的情况下学习和改进。",
        "技术"
    );
    
    kb.add_document(
        "深度学习介绍",
        "深度学习是机器学习的一个分支，使用多层神经网络来模拟人脑的工作方式。",
        "技术"
    );
    
    kb.add_document(
        "自然语言处理",
        "自然语言处理（NLP）是人工智能的一个领域，专注于计算机与人类语言之间的交互。",
        "技术"
    );
    
    println!("✅ 已添加 {} 个文档", kb.documents.len());
    
    // 测试搜索功能
    println!("\n🔍 测试搜索功能:");
    let search_results = kb.search("机器学习");
    println!("搜索 '机器学习' 找到 {} 个相关文档:", search_results.len());
    for doc in search_results {
        println!("  - {}", doc.title);
    }
    
    // 测试问答功能
    println!("\n❓ 测试问答功能:");
    
    let questions = vec![
        "什么是人工智能？",
        "机器学习和深度学习有什么区别？",
        "自然语言处理的应用有哪些？",
    ];
    
    for question in questions {
        println!("\n问题: {}", question);
        match kb.answer(question).await {
            Ok(answer) => println!("回答: {}", answer),
            Err(e) => println!("错误: {}", e),
        }
    }
    
    println!("\n✨ 知识库演示完成！");
    Ok(())
}
