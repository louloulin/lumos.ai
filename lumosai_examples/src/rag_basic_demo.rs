use lumosai_core::prelude::*;
use lumosai_core::{LlmProvider, Message, agent::types::AgentGenerateOptions, Role};
use std::sync::Arc;
use std::collections::HashMap;

/// RAG 基础功能演示
///
/// 这个示例展示了如何使用 LumosAI 的 RAG 系统：
/// 1. 创建文档和分块
/// 2. 生成嵌入向量
/// 3. 存储到向量数据库
/// 4. 执行语义检索
/// 5. 结合 Agent 进行问答

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 LumosAI RAG 基础功能演示");

    // 1. 准备示例文档
    let documents = vec![
        "人工智能（AI）是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的系统。这包括学习、推理、感知、语言理解和问题解决。",
        "机器学习是人工智能的一个子集，它使计算机能够在没有明确编程的情况下学习和改进。它基于算法和统计模型，使系统能够从数据中学习模式。",
        "深度学习是机器学习的一个分支，使用多层神经网络来模拟人脑的工作方式。它在图像识别、语音识别和自然语言处理等领域取得了突破性进展。",
        "自然语言处理（NLP）是人工智能的一个分支，专注于计算机与人类语言之间的交互。它包括文本分析、语言翻译、情感分析和对话系统。",
        "计算机视觉是人工智能的一个领域，致力于使计算机能够理解和解释视觉信息。它在医疗诊断、自动驾驶和安全监控等领域有广泛应用。",
    ];
    println!("📄 创建了 {} 个示例文档", documents.len());

    // 2. 简单的文档分块（按句子分割）
    let mut all_chunks = Vec::new();
    for (_doc_id, content) in documents.iter().enumerate() {
        // 简单按句号分割
        let sentences: Vec<&str> = content.split('。').filter(|s| !s.trim().is_empty()).collect();
        for (_chunk_id, sentence) in sentences.iter().enumerate() {
            if !sentence.trim().is_empty() {
                all_chunks.push(format!("{}。", sentence.trim()));
            }
        }
    }
    println!("📝 总共分成 {} 个文档块", all_chunks.len());
    
    // 3. 创建向量存储
    let vector_storage = memory_vector_storage(384, Some(1000))?;
    println!("💾 创建内存向量存储 (维度: 384)");

    // 4. 创建索引并存储文档
    println!("🔍 开始创建索引并存储文档...");
    vector_storage.create_index("rag_documents", 384, None).await?;

    // 为所有文档块生成嵌入
    let mut vectors = Vec::new();
    let mut ids = Vec::new();
    let mut metadata_list = Vec::new();

    for (i, chunk) in all_chunks.iter().enumerate() {
        // 生成简单的 mock 嵌入（基于文本哈希）
        let embedding = generate_mock_embedding(chunk, 384);
        vectors.push(embedding);
        ids.push(format!("chunk_{}", i));

        let mut metadata = HashMap::new();
        metadata.insert("content".to_string(), serde_json::Value::String(chunk.clone()));
        metadata_list.push(metadata);

        if (i + 1) % 5 == 0 {
            println!("   已处理 {} 个文档块", i + 1);
        }
    }

    // 批量存储到向量数据库
    let stored_ids = vector_storage.upsert(
        "rag_documents",
        vectors,
        Some(ids),
        Some(metadata_list)
    ).await?;
    println!("✅ 完成存储 {} 个文档块", stored_ids.len());

    // 5. 测试检索功能
    println!("\n🔎 测试语义检索功能:");
    let queries = vec![
        "什么是人工智能？",
        "机器学习的应用有哪些？",
        "深度学习和传统机器学习的区别",
    ];

    for query in &queries {
        println!("\n查询: {}", query);

        // 为查询生成嵌入
        let query_embedding = generate_mock_embedding(query, 384);

        // 执行向量搜索
        let search_results = vector_storage.query(
            "rag_documents",
            query_embedding,
            3,
            None,
            false
        ).await?;

        for (i, result) in search_results.iter().enumerate() {
            let default_content = "无内容".to_string();
            let chunk_text = all_chunks.get(i).unwrap_or(&default_content);
            println!("  {}. [相似度: {:.3}] {}",
                i + 1,
                result.score,
                chunk_text.chars().take(100).collect::<String>() + "..."
            );
        }
    }
    
    // 6. 创建 RAG Agent
    println!("\n🤖 创建 RAG Agent 进行问答:");
    let llm = MockLlmProvider::new("基于提供的文档内容，深度学习是机器学习的一个子集，使用多层神经网络来模拟人脑的工作方式。它在图像识别、语音识别、自然语言处理等领域取得了突破性进展。主要应用领域包括：1. 计算机视觉（图像识别、目标检测）；2. 自然语言处理（机器翻译、文本生成）；3. 语音识别和合成；4. 推荐系统；5. 医疗诊断；6. 自动驾驶等。");

    let rag_agent = quick_agent("rag_assistant",
        "你是一个专业的AI助手，能够基于提供的文档内容回答问题。")
        .model(Arc::new(llm))
        .build()?;

    // 7. RAG 问答演示
    let question = "请解释一下什么是深度学习，以及它的主要应用领域？";
    println!("\n问题: {}", question);

    // 检索相关文档
    let query_embedding = generate_mock_embedding(question, 384);
    let search_results = vector_storage.query(
        "rag_documents",
        query_embedding,
        3,
        None,
        false
    ).await?;

    // 构建上下文
    let default_content = "无内容".to_string();
    let context = search_results.iter()
        .enumerate()
        .map(|(i, result)| {
            if let Some(metadata) = &result.metadata {
                if let Some(content) = metadata.get("content") {
                    if let Some(content_str) = content.as_str() {
                        return format!("文档片段: {}", content_str);
                    }
                }
            }
            let chunk_text = all_chunks.get(i).unwrap_or(&default_content);
            format!("文档片段: {}", chunk_text)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = format!(
        "基于以下文档内容回答问题:\n\n{}\n\n问题: {}\n\n请基于上述文档内容给出准确的回答:",
        context, question
    );

    // 创建消息
    let messages = vec![Message {
        role: Role::User,
        content: prompt,
        name: None,
        metadata: Some(HashMap::new()),
    }];

    let options = AgentGenerateOptions::default();
    let response = rag_agent.generate(&messages, &options).await?;
    println!("回答: {}", response.response);
    
    println!("\n✨ RAG 基础功能演示完成！");
    Ok(())
}

/// 生成简单的 mock 嵌入向量
fn generate_mock_embedding(text: &str, dimension: usize) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let seed = hasher.finish();

    let mut rng = seed;
    let mut embedding = Vec::with_capacity(dimension);

    for _ in 0..dimension {
        // 简单的线性同余生成器
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let value = (rng as f32 / u64::MAX as f32) * 2.0 - 1.0;
        embedding.push(value);
    }

    // 归一化向量
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in &mut embedding {
            *x /= norm;
        }
    }

    embedding
}



/// Mock LLM 提供者
struct MockLlmProvider {
    response: String,
}

impl MockLlmProvider {
    fn new(response: &str) -> Self {
        Self {
            response: response.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for MockLlmProvider {
    fn name(&self) -> &str {
        "MockLLM"
    }

    async fn generate(&self, _prompt: &str, _options: &lumosai_core::llm::LlmOptions) -> lumosai_core::Result<String> {
        Ok(self.response.clone())
    }

    async fn generate_with_messages(&self, _messages: &[Message], _options: &lumosai_core::llm::LlmOptions) -> lumosai_core::Result<String> {
        Ok(self.response.clone())
    }

    async fn generate_stream<'a>(
        &'a self,
        _prompt: &'a str,
        _options: &'a lumosai_core::llm::LlmOptions,
    ) -> lumosai_core::Result<futures::stream::BoxStream<'a, lumosai_core::Result<String>>> {
        use futures::stream;
        let response = self.response.clone();
        Ok(Box::pin(stream::once(async move { Ok(response) })))
    }

    async fn get_embedding(&self, _text: &str) -> lumosai_core::Result<Vec<f32>> {
        // 简单的 mock 嵌入实现
        Ok(vec![0.1; 384])
    }
}
