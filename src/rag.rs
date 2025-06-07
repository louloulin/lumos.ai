//! 简化的RAG系统API
//!
//! 提供一行代码创建RAG系统的便利函数，支持智能默认配置。

use crate::{Result, Error, vector::VectorStorage};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

/// RAG系统抽象
pub type RagSystem = Arc<dyn RagTrait>;

/// 简单RAG实现
pub type SimpleRag = lumosai_rag::pipeline::RagPipeline;

/// 文档类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: Document,
    pub score: f32,
    pub chunk_index: Option<usize>,
}

/// RAG系统trait
#[async_trait::async_trait]
pub trait RagTrait: Send + Sync {
    /// 添加文档
    async fn add_document(&self, content: &str) -> Result<String>;
    
    /// 添加带元数据的文档
    async fn add_document_with_metadata(
        &self, 
        content: &str, 
        metadata: std::collections::HashMap<String, serde_json::Value>
    ) -> Result<String>;
    
    /// 搜索相关文档
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
    
    /// 生成回答
    async fn answer(&self, question: &str) -> Result<String>;
    
    /// 删除文档
    async fn delete_document(&self, doc_id: &str) -> Result<()>;
}

/// 一行代码创建简单RAG系统
/// 
/// # 参数
/// - `storage`: 向量存储
/// - `embedding_provider`: 嵌入模型提供商 ("openai", "local")
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let storage = lumos::vector::memory().await?;
///     let rag = lumos::rag::simple(storage, "openai").await?;
///     
///     // 添加文档
///     rag.add_document("AI is transforming the world").await?;
///     
///     // 搜索
///     let results = rag.search("What is AI doing?", 5).await?;
///     
///     // 生成回答
///     let answer = rag.answer("Tell me about AI").await?;
///     
///     Ok(())
/// }
/// ```
pub async fn simple(storage: VectorStorage, embedding_provider: &str) -> Result<RagSystem> {
    let builder = builder()
        .storage(storage)
        .embedding_provider(embedding_provider)
        .chunking_strategy("recursive")
        .chunk_size(1000)
        .chunk_overlap(200);
    
    builder.build().await
}

/// 创建带有智能默认配置的RAG系统
/// 
/// 自动检测最佳配置并创建RAG系统
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let rag = lumos::rag::auto().await?;
///     Ok(())
/// }
/// ```
pub async fn auto() -> Result<RagSystem> {
    let storage = crate::vector::auto().await?;
    simple(storage, "openai").await
}

/// RAG系统构建器
/// 
/// 提供更细粒度的配置选项
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let storage = lumos::vector::memory().await?;
///     
///     let rag = lumos::rag::builder()
///         .storage(storage)
///         .embedding_provider("openai")
///         .chunking_strategy("semantic")
///         .chunk_size(800)
///         .chunk_overlap(100)
///         .retrieval_strategy("hybrid")
///         .top_k(10)
///         .build()
///         .await?;
///     
///     Ok(())
/// }
/// ```
pub fn builder() -> RagBuilder {
    RagBuilder::new()
}

/// RAG构建器
pub struct RagBuilder {
    storage: Option<VectorStorage>,
    embedding_provider: Option<String>,
    chunking_strategy: Option<String>,
    chunk_size: Option<usize>,
    chunk_overlap: Option<usize>,
    retrieval_strategy: Option<String>,
    top_k: Option<usize>,
    llm_provider: Option<String>,
}

impl RagBuilder {
    pub fn new() -> Self {
        Self {
            storage: None,
            embedding_provider: None,
            chunking_strategy: None,
            chunk_size: None,
            chunk_overlap: None,
            retrieval_strategy: None,
            top_k: None,
            llm_provider: None,
        }
    }
    
    /// 设置向量存储
    pub fn storage(mut self, storage: VectorStorage) -> Self {
        self.storage = Some(storage);
        self
    }
    
    /// 设置嵌入模型提供商
    pub fn embedding_provider(mut self, provider: &str) -> Self {
        self.embedding_provider = Some(provider.to_string());
        self
    }
    
    /// 设置分块策略
    pub fn chunking_strategy(mut self, strategy: &str) -> Self {
        self.chunking_strategy = Some(strategy.to_string());
        self
    }
    
    /// 设置分块大小
    pub fn chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }
    
    /// 设置分块重叠
    pub fn chunk_overlap(mut self, overlap: usize) -> Self {
        self.chunk_overlap = Some(overlap);
        self
    }
    
    /// 设置检索策略
    pub fn retrieval_strategy(mut self, strategy: &str) -> Self {
        self.retrieval_strategy = Some(strategy.to_string());
        self
    }
    
    /// 设置返回结果数量
    pub fn top_k(mut self, k: usize) -> Self {
        self.top_k = Some(k);
        self
    }
    
    /// 设置LLM提供商
    pub fn llm_provider(mut self, provider: &str) -> Self {
        self.llm_provider = Some(provider.to_string());
        self
    }
    
    /// 构建RAG系统
    pub async fn build(self) -> Result<RagSystem> {
        let storage = self.storage
            .ok_or_else(|| Error::Config("Storage is required".to_string()))?;
        
        let embedding_provider = self.embedding_provider
            .unwrap_or_else(|| "openai".to_string());
        
        // 创建嵌入提供商
        let embedding = create_embedding_provider(&embedding_provider).await?;
        
        // 创建分块策略
        let chunking_strategy = self.chunking_strategy
            .unwrap_or_else(|| "recursive".to_string());
        let chunk_size = self.chunk_size.unwrap_or(1000);
        let chunk_overlap = self.chunk_overlap.unwrap_or(200);
        
        let chunker = create_chunking_strategy(&chunking_strategy, chunk_size, chunk_overlap)?;
        
        // 创建检索策略
        let retrieval_strategy = self.retrieval_strategy
            .unwrap_or_else(|| "vector".to_string());
        let top_k = self.top_k.unwrap_or(5);
        
        let retriever = create_retrieval_strategy(&retrieval_strategy, top_k)?;
        
        // 创建RAG管道
        let pipeline = lumosai_rag::pipeline::RagPipeline::builder()
            .vector_store(storage)
            .embedding_provider(embedding)
            .chunking_strategy(chunker)
            .retrieval_strategy(retriever)
            .build()
            .await
            .map_err(|e| Error::Rag(format!("Failed to create RAG pipeline: {}", e)))?;
        
        Ok(Arc::new(SimpleRagWrapper::new(pipeline)))
    }
}

impl Default for RagBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 简单RAG包装器，实现RagTrait
struct SimpleRagWrapper {
    pipeline: lumosai_rag::pipeline::RagPipeline,
}

impl SimpleRagWrapper {
    fn new(pipeline: lumosai_rag::pipeline::RagPipeline) -> Self {
        Self { pipeline }
    }
}

#[async_trait::async_trait]
impl RagTrait for SimpleRagWrapper {
    async fn add_document(&self, content: &str) -> Result<String> {
        let doc_id = uuid::Uuid::new_v4().to_string();
        let document = lumosai_rag::document::Document::new(doc_id.clone(), content.to_string());
        
        self.pipeline.add_document(document).await
            .map_err(|e| Error::Rag(format!("Failed to add document: {}", e)))?;
        
        Ok(doc_id)
    }
    
    async fn add_document_with_metadata(
        &self, 
        content: &str, 
        metadata: std::collections::HashMap<String, serde_json::Value>
    ) -> Result<String> {
        let doc_id = uuid::Uuid::new_v4().to_string();
        let mut document = lumosai_rag::document::Document::new(doc_id.clone(), content.to_string());
        document.metadata = metadata;
        
        self.pipeline.add_document(document).await
            .map_err(|e| Error::Rag(format!("Failed to add document: {}", e)))?;
        
        Ok(doc_id)
    }
    
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let results = self.pipeline.search(query, limit).await
            .map_err(|e| Error::Rag(format!("Failed to search: {}", e)))?;
        
        // 转换结果格式
        let search_results = results.into_iter().map(|result| {
            SearchResult {
                document: Document {
                    id: result.document.id,
                    content: result.document.content,
                    metadata: result.document.metadata,
                },
                score: result.score,
                chunk_index: result.chunk_index,
            }
        }).collect();
        
        Ok(search_results)
    }
    
    async fn answer(&self, question: &str) -> Result<String> {
        let answer = self.pipeline.answer(question).await
            .map_err(|e| Error::Rag(format!("Failed to generate answer: {}", e)))?;
        
        Ok(answer)
    }
    
    async fn delete_document(&self, doc_id: &str) -> Result<()> {
        self.pipeline.delete_document(doc_id).await
            .map_err(|e| Error::Rag(format!("Failed to delete document: {}", e)))?;
        
        Ok(())
    }
}

// 辅助函数
async fn create_embedding_provider(provider: &str) -> Result<Arc<dyn lumosai_rag::embedding::EmbeddingProvider>> {
    match provider {
        "openai" => {
            let provider = lumosai_rag::embedding::OpenAIEmbeddingProvider::new()
                .map_err(|e| Error::Config(format!("Failed to create OpenAI embedding provider: {}", e)))?;
            Ok(Arc::new(provider))
        }
        _ => Err(Error::Config(format!("Unsupported embedding provider: {}", provider))),
    }
}

fn create_chunking_strategy(
    strategy: &str, 
    chunk_size: usize, 
    chunk_overlap: usize
) -> Result<Arc<dyn lumosai_rag::chunking::ChunkingStrategy>> {
    match strategy {
        "recursive" => {
            let strategy = lumosai_rag::chunking::RecursiveChunkingStrategy::new(chunk_size, chunk_overlap);
            Ok(Arc::new(strategy))
        }
        "markdown" => {
            let strategy = lumosai_rag::chunking::MarkdownChunkingStrategy::new(chunk_size, chunk_overlap);
            Ok(Arc::new(strategy))
        }
        _ => Err(Error::Config(format!("Unsupported chunking strategy: {}", strategy))),
    }
}

fn create_retrieval_strategy(
    strategy: &str, 
    top_k: usize
) -> Result<Arc<dyn lumosai_rag::retrieval::RetrievalStrategy>> {
    match strategy {
        "vector" => {
            let strategy = lumosai_rag::retrieval::VectorRetrievalStrategy::new(top_k);
            Ok(Arc::new(strategy))
        }
        "hybrid" => {
            let strategy = lumosai_rag::retrieval::HybridRetrievalStrategy::new(top_k);
            Ok(Arc::new(strategy))
        }
        _ => Err(Error::Config(format!("Unsupported retrieval strategy: {}", strategy))),
    }
}
