use std::path::Path;
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::error::Result;
use crate::vector::VectorStorage;

/// 文档来源类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentSource {
    /// 从目录加载文档
    Directory(String),
    /// 从URL加载文档
    Url(String),
    /// 从文本字符串加载文档
    Text(String),
    /// 从数据库加载文档
    Database(String),
}

impl DocumentSource {
    /// 从目录创建文档源
    pub fn from_directory(path: impl Into<String>) -> Self {
        Self::Directory(path.into())
    }

    /// 从URL创建文档源
    pub fn from_url(url: impl Into<String>) -> Self {
        Self::Url(url.into())
    }

    /// 从文本创建文档源
    pub fn from_text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    /// 从数据库连接字符串创建文档源
    pub fn from_database(connection_string: impl Into<String>) -> Self {
        Self::Database(connection_string.into())
    }
}

/// 分块配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkConfig {
    pub chunk_size: usize,
    pub chunk_overlap: Option<usize>,
    pub separator: Option<String>,
    pub strategy: Option<String>,
}

impl ChunkConfig {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap: None,
            separator: None,
            strategy: None,
        }
    }

    pub fn with_overlap(mut self, overlap: usize) -> Self {
        self.chunk_overlap = Some(overlap);
        self
    }

    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = Some(separator.into());
        self
    }

    pub fn with_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.strategy = Some(strategy.into());
        self
    }
}

/// 嵌入配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedConfig {
    pub model: String,
    pub dimensions: Option<usize>,
    pub max_retries: Option<usize>,
}

impl EmbedConfig {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            dimensions: None,
            max_retries: None,
        }
    }

    pub fn with_dimensions(mut self, dimensions: usize) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = Some(max_retries);
        self
    }
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    pub db: String,
    pub collection: String,
    pub connection_string: Option<String>,
}

impl StoreConfig {
    pub fn new(db: impl Into<String>, collection: impl Into<String>) -> Self {
        Self {
            db: db.into(),
            collection: collection.into(),
            connection_string: None,
        }
    }

    pub fn with_connection_string(mut self, connection_string: impl Into<String>) -> Self {
        self.connection_string = Some(connection_string.into());
        self
    }
}

/// 查询配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConfig {
    pub rerank: Option<bool>,
    pub top_k: Option<usize>,
    pub filter: Option<String>,
}

impl QueryConfig {
    pub fn new() -> Self {
        Self {
            rerank: None,
            top_k: None,
            filter: None,
        }
    }

    pub fn with_rerank(mut self, rerank: bool) -> Self {
        self.rerank = Some(rerank);
        self
    }

    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = Some(top_k);
        self
    }

    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }
}

/// 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// 原始查询文本
    pub query: String,
    /// 检索到的文档
    pub documents: Vec<crate::vector::Document>,
    /// 相似度分数
    pub scores: Option<Vec<f32>>,
    /// 上下文
    pub context: String,
    /// 额外元数据
    pub metadata: Value,
}

/// RAG管道接口
#[async_trait]
pub trait RagPipeline: Send + Sync {
    /// 处理并索引文档
    async fn process_documents(&mut self, source: DocumentSource) -> Result<usize>;
    
    /// 基于字符串查询检索内容
    async fn query(&self, query: &str, top_k: usize) -> Result<QueryResult>;
    
    /// 获取管道名称
    fn name(&self) -> &str;
    
    /// 获取管道描述
    fn description(&self) -> Option<&str>;
}

/// 基本RAG管道实现
pub struct BasicRagPipeline {
    /// 管道名称
    name: String,
    /// 管道描述
    description: Option<String>,
    /// 向量存储
    vector_store: Arc<tokio::sync::Mutex<crate::vector::MemoryVectorStorage>>,
    /// 嵌入生成器
    embedding_fn: Arc<dyn Fn(&str) -> Result<Vec<f32>> + Send + Sync>,
}

impl BasicRagPipeline {
    /// 创建新的基本RAG管道
    pub fn new(
        name: impl Into<String>,
        embedding_fn: impl Fn(&str) -> Result<Vec<f32>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            description: None,
            vector_store: Arc::new(tokio::sync::Mutex::new(crate::vector::MemoryVectorStorage::new(1536, None))),
            embedding_fn: Arc::new(embedding_fn),
        }
    }
    
    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// 处理文本文档
    async fn process_text(&mut self, text: &str) -> Result<usize> {
        let documents = split_text_into_documents(text);
        let mut count = 0;
        
        for mut doc in documents {
            // 生成嵌入
            let embedding = (self.embedding_fn)(&doc.content)?;
            doc.embedding = embedding;
            
            // 准备元数据
            let metadata = vec![HashMap::from([
                ("content".to_string(), Value::String(doc.content.clone()))
            ])];
            
            // 获取锁后调用upsert
            {
                let store = self.vector_store.lock().await;
                // 使用完全限定路径
                <dyn VectorStorage>::upsert(
                    &*store,
                    "default", 
                    vec![doc.embedding.clone()],
                    Some(vec![doc.id.clone()]), 
                    Some(metadata)
                ).await?;
            }
            
            count += 1;
        }
        
        Ok(count)
    }
}

#[async_trait]
impl RagPipeline for BasicRagPipeline {
    async fn process_documents(&mut self, source: DocumentSource) -> Result<usize> {
        match source {
            DocumentSource::Text(text) => {
                self.process_text(&text).await
            },
            DocumentSource::Directory(dir_path) => {
                let path = Path::new(&dir_path);
                if !path.exists() || !path.is_dir() {
                    return Err(crate::error::Error::InvalidInput(format!("Directory not found: {}", dir_path)));
                }
                
                let mut total_count = 0;
                
                // 遍历目录，只处理文本文件
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    let path = entry.path();
                    
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "txt" || ext == "md" || ext == "rst" {
                                let content = std::fs::read_to_string(&path)?;
                                total_count += self.process_text(&content).await?;
                            }
                        }
                    }
                }
                
                Ok(total_count)
            },
            DocumentSource::Url(url) => {
                // 简单实现，实际项目中可能需要使用reqwest等库
                Err(crate::error::Error::Other(format!("URL document source not implemented yet: {}", url)))
            },
            DocumentSource::Database(connection_string) => {
                // 简单实现，实际项目中需要连接数据库
                Err(crate::error::Error::Other(format!("Database document source not implemented yet: {}", connection_string)))
            }
        }
    }
    
    async fn query(&self, query: &str, top_k: usize) -> Result<QueryResult> {
        // 生成查询嵌入
        let query_embedding = (self.embedding_fn)(query)?;
        
        // 使用向量存储搜索 - 从锁中获取数据
        let vector_results = {
            let store = self.vector_store.lock().await;
            // 使用完全限定路径
            <dyn VectorStorage>::query(
                &*store,
                "default", 
                query_embedding, 
                top_k, 
                None, 
                true
            ).await?
        };
        
        // 提取文档和分数
        let documents: Vec<crate::vector::Document> = vector_results.iter().map(|result| {
            let mut doc = crate::vector::Document::new(result.id.clone(), "");
            if let Some(metadata) = &result.metadata {
                if let Some(content) = metadata.get("content").and_then(|v| v.as_str()) {
                    doc.content = content.to_string();
                }
            }
            if let Some(vec) = &result.vector {
                doc.embedding = vec.clone();
            }
            doc
        }).collect();
        
        let scores = vector_results.iter().map(|result| result.score).collect();
        
        // 构建上下文
        let context = documents.iter()
            .map(|doc| doc.content.clone())
            .collect::<Vec<String>>()
            .join("\n\n");
        
        Ok(QueryResult {
            query: query.to_string(),
            documents,
            scores: Some(scores),
            context,
            metadata: serde_json::json!({}),
        })
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// 将文本分割成多个文档
fn split_text_into_documents(text: &str) -> Vec<crate::vector::Document> {
    // 简单实现，按段落分割
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    
    paragraphs.iter().enumerate()
        .filter(|(_, p)| !p.trim().is_empty())
        .map(|(i, p)| {
            crate::vector::Document::new(
                format!("doc_{}", i),
                p.trim().to_string()
            )
        })
        .collect()
}

/// 创建基本的RAG管道
pub fn create_basic_rag_pipeline(
    name: impl Into<String>,
    embedding_fn: impl Fn(&str) -> Result<Vec<f32>> + Send + Sync + 'static,
) -> impl RagPipeline {
    BasicRagPipeline::new(name, embedding_fn)
}

/// 配置化的RAG管道结构体（用于宏生成）
pub struct RagPipelineBuilder {
    pub name: String,
    pub sources: Vec<DocumentSource>,
    pub chunk_config: Option<ChunkConfig>,
    pub embed_config: Option<EmbedConfig>,
    pub store_config: Option<StoreConfig>,
    pub query_config: Option<QueryConfig>,
}

impl RagPipelineBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            sources: Vec::new(),
            chunk_config: None,
            embed_config: None,
            store_config: None,
            query_config: None,
        }
    }

    pub fn add_source(mut self, source: DocumentSource) -> Self {
        self.sources.push(source);
        self
    }

    pub fn with_chunk_config(mut self, config: ChunkConfig) -> Self {
        self.chunk_config = Some(config);
        self
    }

    pub fn with_embed_config(mut self, config: EmbedConfig) -> Self {
        self.embed_config = Some(config);
        self
    }

    pub fn with_store_config(mut self, config: StoreConfig) -> Self {
        self.store_config = Some(config);
        self
    }

    pub fn with_query_config(mut self, config: QueryConfig) -> Self {
        self.query_config = Some(config);
        self
    }

    pub async fn build(self) -> Result<BasicRagPipeline> {
        // 创建一个简单的嵌入函数（实际项目中应该使用真实的嵌入模型）
        let embedding_fn = |text: &str| -> Result<Vec<f32>> {
            // 简单的哈希嵌入，实际项目中应该使用真实的嵌入模型
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            use std::hash::{Hash, Hasher};
            text.hash(&mut hasher);
            let hash = hasher.finish();

            // 生成1536维的伪嵌入向量
            let mut embedding = vec![0.0; 1536];
            for i in 0..1536 {
                embedding[i] = ((hash.wrapping_add(i as u64)) as f32) / (u64::MAX as f32);
            }
            Ok(embedding)
        };

        let mut pipeline = BasicRagPipeline::new(self.name, embedding_fn);

        // 处理所有文档源
        for source in self.sources {
            pipeline.process_documents(source).await?;
        }

        Ok(pipeline)
    }
}