//! RAG Integration for Agents
//!
//! This module provides simplified RAG integration for agents,
//! making it easy to create agents with knowledge base capabilities.

use std::sync::Arc;
use serde_json::Value;

use super::builder::AgentBuilder;
use super::executor::BasicAgent;
use super::trait_def::Agent;
use crate::error::Result;
use crate::llm::{Message, Role};
use crate::vector::{MemoryVectorStorage, VectorStorage};

/// RAG 配置
#[derive(Clone)]
pub struct RagConfig {
    /// 向量存储
    pub vector_store: Arc<MemoryVectorStorage>,
    /// 检索数量 (top_k)
    pub top_k: usize,
    /// 相似度阈值
    pub similarity_threshold: f32,
    /// 是否启用缓存
    pub enable_cache: bool,
}

impl RagConfig {
    /// 创建默认的 RAG 配置
    pub fn new(vector_store: Arc<MemoryVectorStorage>) -> Self {
        Self {
            vector_store,
            top_k: 5,
            similarity_threshold: 0.7,
            enable_cache: true,
        }
    }

    /// 设置检索数量
    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k;
        self
    }

    /// 设置相似度阈值
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.similarity_threshold = threshold;
        self
    }

    /// 设置缓存选项
    pub fn with_cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }
}

/// RAG 增强的 Agent
///
/// 这个结构体包装了一个 BasicAgent 并提供 RAG 功能
pub struct RagAgent {
    /// 基础 Agent
    agent: BasicAgent,
    /// RAG 配置
    rag_config: RagConfig,
}

impl RagAgent {
    /// 创建新的 RAG Agent
    pub fn new(agent: BasicAgent, rag_config: RagConfig) -> Self {
        Self { agent, rag_config }
    }

    /// 使用 RAG 增强生成响应
    ///
    /// 1. 从向量存储中检索相关文档
    /// 2. 将文档作为上下文注入
    /// 3. 调用 Agent 生成响应
    pub async fn generate_with_rag(&self, query: &str) -> Result<String> {
        // 1. 将查询转换为嵌入向量（简化版：使用查询长度作为特征）
        let query_embedding = self.create_simple_embedding(query);

        // 2. 从向量存储中检索相关文档
        let search_results = self
            .rag_config
            .vector_store
            .query("default", query_embedding, self.rag_config.top_k, None, false)
            .await
            .unwrap_or_default();

        // 3. 构建上下文
        let context = self.build_context(&search_results);

        // 4. 构建增强的 prompt
        let enhanced_prompt = if context.is_empty() {
            query.to_string()
        } else {
            format!(
                "Context from knowledge base:\n{}\n\nQuestion: {}",
                context, query
            )
        };

        // 5. 调用 Agent 生成响应
        self.agent.generate_simple(&enhanced_prompt).await
    }

    /// 简化的嵌入向量生成（基于查询特征）
    fn create_simple_embedding(&self, text: &str) -> Vec<f32> {
        // 简化实现：基于文本特征生成确定性向量
        let mut embedding = vec![0.0f32; 384];
        let text_lower = text.to_lowercase();

        // 使用文本特征填充向量
        for (i, ch) in text_lower.chars().enumerate() {
            if i >= 384 {
                break;
            }
            embedding[i] = (ch as u32 as f32) / 1000.0;
        }

        // 填充剩余维度
        let len = text.len() as f32 / 100.0;
        for i in text_lower.chars().count()..384 {
            embedding[i] = len + (i as f32 / 1000.0);
        }

        embedding
    }

    /// 从搜索结果构建上下文字符串
    fn build_context(&self, results: &[crate::vector::QueryResult]) -> String {
        if results.is_empty() {
            return String::new();
        }

        let mut context_parts = Vec::new();

        for (i, result) in results.iter().enumerate() {
            // 尝试从 metadata 中提取文本
            if let Some(metadata) = &result.metadata {
                if let Some(text_value) = metadata.get("text") {
                    if let Some(text) = text_value.as_str() {
                        // 只包含相似度超过阈值的文档
                        if result.score >= self.rag_config.similarity_threshold {
                            context_parts.push(format!("{}. {}", i + 1, text));
                        }
                    }
                }
            }
        }

        context_parts.join("\n\n")
    }

    /// 获取底层 Agent 的引用
    pub fn agent(&self) -> &BasicAgent {
        &self.agent
    }

    /// 批量添加文档到知识库
    pub async fn add_documents(&self, documents: Vec<(&str, &str)>) -> Result<()> {
        let mut embeddings = Vec::new();
        let mut ids = Vec::new();
        let mut metadata_list = Vec::new();

        for (id, text) in documents {
            let embedding = self.create_simple_embedding(text);
            embeddings.push(embedding);
            ids.push(id.to_string());

            let mut metadata = std::collections::HashMap::new();
            metadata.insert(
                "text".to_string(),
                serde_json::Value::String(text.to_string()),
            );
            metadata_list.push(metadata);
        }

        self.rag_config
            .vector_store
            .upsert("default", embeddings, Some(ids), Some(metadata_list))
            .await
            .map_err(|e| crate::error::Error::Storage(format!("Failed to add documents: {}", e)))?;

        Ok(())
    }
}

/// RAG Builder 扩展
pub trait RagIntegrationExt {
    /// 添加 RAG 能力（简化方法）
    ///
    /// 这个方法将 Agent 包装为 RagAgent，提供知识库检索能力
    fn with_rag_simple(self, vector_store: Arc<MemoryVectorStorage>) -> Result<RagAgent>;

    /// 添加 RAG 能力（高级配置）
    fn with_rag(self, rag_config: RagConfig) -> Result<RagAgent>;
}

impl RagIntegrationExt for AgentBuilder {
    fn with_rag_simple(self, vector_store: Arc<MemoryVectorStorage>) -> Result<RagAgent> {
        let agent = self.build()?;
        let rag_config = RagConfig::new(vector_store);
        Ok(RagAgent::new(agent, rag_config))
    }

    fn with_rag(self, rag_config: RagConfig) -> Result<RagAgent> {
        let agent = self.build()?;
        Ok(RagAgent::new(agent, rag_config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试已移至单独的测试文件 lumosai_core/tests/rag_integration_tests.rs
    // 以避免在库编译时的依赖问题
}

