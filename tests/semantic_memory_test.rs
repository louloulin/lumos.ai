use std::sync::Arc;
use lomusai_core::{
    memory::{
        MemoryConfig, MessageRange, SemanticRecallConfig,
        semantic_memory::{SemanticSearchOptions, create_semantic_memory}
    },
    llm::{Message, Role, MockLlmProvider},
    Result
};

#[tokio::test]
async fn test_semantic_memory() -> Result<()> {
    // 创建配置
    let config = MemoryConfig {
        enabled: true,
        namespace: Some("test_semantic".to_string()),
        store_id: None,
        query: None,
        working_memory: None,
        semantic_recall: Some(SemanticRecallConfig {
            top_k: 5,
            message_range: Some(MessageRange {
                before: 1,
                after: 1,
            }),
            generate_summaries: false,
            use_embeddings: true,
            max_capacity: Some(1000),
            max_results: Some(10),
            relevance_threshold: Some(0.5),
            template: None,
        }),
        last_messages: None,
    };
    
    // 创建大小为1536维的向量测试集
    // OpenAI的嵌入向量通常是1536维的
    let dimension = 1536;
    let mut embeddings = Vec::new();
    
    // 生成四个向量
    for i in 0..4 {
        let mut vec = Vec::with_capacity(dimension);
        for j in 0..dimension {
            // 生成一些简单的向量值作为测试
            vec.push(((i * dimension + j) as f32) / (dimension as f32 * 4.0));
        }
        embeddings.push(vec);
    }
    
    // 创建Mock LLM Provider
    let mock_llm = Arc::new(MockLlmProvider::new_with_embeddings(embeddings));
    
    // 创建语义内存
    let memory = create_semantic_memory(&config, mock_llm)?;
    
    // 添加测试消息
    let message1 = Message {
        role: Role::User,
        content: "语义搜索是什么?".to_string(),
        metadata: None,
        name: None,
    };
    
    let message2 = Message {
        role: Role::Assistant,
        content: "语义搜索是一种基于意义和上下文相似度查找信息的方法，而不仅仅是关键词匹配。".to_string(),
        metadata: None,
        name: None,
    };
    
    let message3 = Message {
        role: Role::User,
        content: "嵌入向量如何用于语义搜索?".to_string(),
        metadata: None,
        name: None,
    };
    
    // 存储消息
    memory.add(&message1).await?;
    memory.add(&message2).await?;
    memory.add(&message3).await?;
    
    // 测试最近消息检索
    let recent = memory.get_recent(2).await?;
    assert_eq!(recent.len(), 2);
    
    // 测试语义搜索
    let options = SemanticSearchOptions {
        limit: 2,
        threshold: Some(0.5),
        namespace: Some("test_semantic".to_string()),
        use_window: true,
        window_size: Some((1, 1)),
        filter: None,
    };
    
    let results = memory.search("如何在AI应用中实现语义搜索?", &options).await?;
    
    // 验证结果
    assert!(!results.is_empty());
    println!("找到 {} 条相关消息", results.len());
    
    // 打印结果
    for (i, result) in results.iter().enumerate() {
        println!("结果 {}: {} - {}", i+1, result.message.role, result.message.content);
    }
    
    Ok(())
} 