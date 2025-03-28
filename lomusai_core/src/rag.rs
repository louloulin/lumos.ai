use std::path::Path;
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;

use crate::error::Result;
use crate::vector::Document;

/// RAG管道接口，支持文档加载、处理和查询
#[async_trait]
pub trait RagPipeline: Send + Sync {
    /// 处理文档并存储向量
    async fn process_documents(&self, documents: Vec<Document>) -> Result<usize>;
    
    /// 根据查询获取相关文档
    async fn query(&self, query: &str, top_k: usize) -> Result<Vec<Document>>;
    
    /// 获取管道名称
    fn name(&self) -> &str;
    
    /// 获取管道描述
    fn description(&self) -> Option<&str>;
}

/// 文档源类型
pub enum DocumentSource {
    /// 来自目录的文档
    Directory(String),
    /// 来自URL的文档
    Url(String),
    /// 来自内存字符串的文档
    Text(String),
}

impl DocumentSource {
    /// 从目录创建文档源
    pub fn from_directory<P: AsRef<Path>>(path: P) -> Self {
        DocumentSource::Directory(path.as_ref().to_string_lossy().to_string())
    }
    
    /// 从URL创建文档源
    pub fn from_url(url: &str) -> Self {
        DocumentSource::Url(url.to_string())
    }
    
    /// 从文本创建文档源
    pub fn from_text(text: &str) -> Self {
        DocumentSource::Text(text.to_string())
    }
}

/// RAG查询结果
pub struct QueryResult {
    /// 查询的原始文本
    pub query: String,
    /// 检索到的相关文档
    pub documents: Vec<Document>,
    /// 相关性分数
    pub scores: Vec<f32>,
    /// 额外的元数据
    pub metadata: Value,
}

/// 基本的RAG管道实现
pub struct BasicRagPipeline {
    name: String,
    description: Option<String>,
    documents: Vec<Document>,
}

impl BasicRagPipeline {
    /// 创建新的基本RAG管道
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            documents: Vec::new(),
        }
    }
    
    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// 添加文档
    pub fn add_document(&mut self, document: Document) {
        self.documents.push(document);
    }
}

#[async_trait]
impl RagPipeline for BasicRagPipeline {
    async fn process_documents(&self, documents: Vec<Document>) -> Result<usize> {
        // 简单实现，实际应用中会处理文档并生成嵌入
        Ok(documents.len())
    }
    
    async fn query(&self, query: &str, top_k: usize) -> Result<Vec<Document>> {
        // 简单实现，实际应用中会基于相似度排序
        let mut results = self.documents.clone();
        results.truncate(top_k);
        Ok(results)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// 创建基本的RAG管道
pub fn create_basic_rag_pipeline(name: impl Into<String>) -> impl RagPipeline {
    BasicRagPipeline::new(name)
} 