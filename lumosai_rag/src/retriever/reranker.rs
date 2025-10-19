//! 文档重排序（Reranking）模块
//!
//! 提供多种重排序策略，用于优化检索结果的相关性排序

use async_trait::async_trait;

use crate::{
    error::Result,
    types::ScoredDocument,
};

/// 重排序器 trait
#[async_trait]
pub trait Reranker: Send + Sync {
    /// 对文档进行重排序
    ///
    /// # 参数
    /// - `query`: 查询文本
    /// - `documents`: 待重排序的文档列表
    ///
    /// # 返回
    /// 重排序后的文档列表（按相关性降序）
    async fn rerank(&self, query: &str, documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>>;
}

/// 交叉编码器重排序（Cross-Encoder Reranking）
///
/// 使用交叉编码器模型计算查询和文档的相关性分数
pub struct CrossEncoderReranker {
    #[allow(dead_code)]
    model_name: String,
    top_k: Option<usize>,
}

impl CrossEncoderReranker {
    /// 创建新的交叉编码器重排序器
    ///
    /// # 参数
    /// - `model_name`: 模型名称（如 "cross-encoder/ms-marco-MiniLM-L-12-v2"）
    /// - `top_k`: 返回的最大文档数量
    pub fn new(model_name: impl Into<String>, top_k: Option<usize>) -> Self {
        Self {
            model_name: model_name.into(),
            top_k,
        }
    }

    /// 计算查询-文档对的相关性分数（Mock 实现）
    ///
    /// 实际实现应调用交叉编码器模型 API
    fn compute_relevance_score(&self, query: &str, document: &str) -> f32 {
        // Mock 实现：基于简单的文本匹配
        let query_lower = query.to_lowercase();
        let doc_lower = document.to_lowercase();
        
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let mut match_count = 0;
        
        for word in &query_words {
            if doc_lower.contains(word) {
                match_count += 1;
            }
        }
        
        if query_words.is_empty() {
            0.0
        } else {
            match_count as f32 / query_words.len() as f32
        }
    }
}

#[async_trait]
impl Reranker for CrossEncoderReranker {
    async fn rerank(&self, query: &str, mut documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>> {
        // 计算每个文档的重排序分数
        for doc in &mut documents {
            let relevance_score = self.compute_relevance_score(query, &doc.document.content);
            // 结合原始分数和重排序分数
            doc.score = (doc.score + relevance_score) / 2.0;
        }

        // 按新分数排序
        documents.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // 如果指定了 top_k，只返回前 k 个文档
        if let Some(k) = self.top_k {
            documents.truncate(k);
        }

        Ok(documents)
    }
}

/// 基于 LLM 的重排序
///
/// 使用大语言模型评估文档与查询的相关性
pub struct LLMReranker {
    #[allow(dead_code)]
    model_name: String,
    top_k: Option<usize>,
    #[allow(dead_code)]
    prompt_template: String,
}

impl LLMReranker {
    /// 创建新的 LLM 重排序器
    pub fn new(model_name: impl Into<String>, top_k: Option<usize>) -> Self {
        Self {
            model_name: model_name.into(),
            top_k,
            prompt_template: "Given the query: {query}\n\nRate the relevance of the following document on a scale of 0-1:\n{document}\n\nRelevance score:".to_string(),
        }
    }

    /// 设置自定义提示模板
    pub fn with_prompt_template(mut self, template: impl Into<String>) -> Self {
        self.prompt_template = template.into();
        self
    }

    /// 使用 LLM 评估文档相关性（Mock 实现）
    async fn evaluate_relevance(&self, query: &str, document: &str) -> Result<f32> {
        // Mock 实现：基于文档长度和查询匹配度
        let query_lower = query.to_lowercase();
        let doc_lower = document.to_lowercase();
        
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let mut match_score = 0.0;
        
        for word in &query_words {
            if doc_lower.contains(word) {
                match_score += 1.0;
            }
        }
        
        let normalized_score = if query_words.is_empty() {
            0.0
        } else {
            match_score / query_words.len() as f32
        };
        
        // 考虑文档长度（较短的文档可能更精确）
        let length_penalty = 1.0 / (1.0 + (document.len() as f32 / 1000.0));
        
        Ok((normalized_score + length_penalty) / 2.0)
    }
}

#[async_trait]
impl Reranker for LLMReranker {
    async fn rerank(&self, query: &str, mut documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>> {
        // 使用 LLM 评估每个文档的相关性
        for doc in &mut documents {
            let relevance_score = self.evaluate_relevance(query, &doc.document.content).await?;
            // 结合原始分数和 LLM 评估分数
            doc.score = (doc.score * 0.3) + (relevance_score * 0.7); // LLM 权重更高
        }

        // 按新分数排序
        documents.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // 如果指定了 top_k，只返回前 k 个文档
        if let Some(k) = self.top_k {
            documents.truncate(k);
        }

        Ok(documents)
    }
}

/// 多样性重排序
///
/// 在保持相关性的同时，增加结果的多样性
pub struct DiversityReranker {
    similarity_threshold: f32,
    top_k: Option<usize>,
}

impl DiversityReranker {
    /// 创建新的多样性重排序器
    ///
    /// # 参数
    /// - `similarity_threshold`: 相似度阈值（0.0-1.0），超过此值的文档被认为是重复的
    /// - `top_k`: 返回的最大文档数量
    pub fn new(similarity_threshold: f32, top_k: Option<usize>) -> Self {
        Self {
            similarity_threshold,
            top_k,
        }
    }

    /// 计算两个文档的相似度（基于内容）
    fn compute_similarity(&self, doc1: &str, doc2: &str) -> f32 {
        // 简单的 Jaccard 相似度
        let words1: std::collections::HashSet<_> = doc1.split_whitespace().collect();
        let words2: std::collections::HashSet<_> = doc2.split_whitespace().collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
}

#[async_trait]
impl Reranker for DiversityReranker {
    async fn rerank(&self, _query: &str, documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>> {
        let mut result = Vec::new();
        let mut remaining = documents;

        // 按原始分数排序
        remaining.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        while !remaining.is_empty() {
            // 取出分数最高的文档
            let best = remaining.remove(0);
            result.push(best.clone());

            // 如果达到 top_k，停止
            if let Some(k) = self.top_k {
                if result.len() >= k {
                    break;
                }
            }

            // 移除与已选文档过于相似的文档
            remaining.retain(|doc| {
                let similarity = self.compute_similarity(&best.document.content, &doc.document.content);
                similarity < self.similarity_threshold
            });
        }

        Ok(result)
    }
}

/// 组合重排序器
///
/// 按顺序应用多个重排序器
pub struct ChainReranker {
    rerankers: Vec<Box<dyn Reranker>>,
}

impl ChainReranker {
    /// 创建新的组合重排序器
    pub fn new() -> Self {
        Self {
            rerankers: Vec::new(),
        }
    }

    /// 添加一个重排序器到链中
    pub fn add_reranker(mut self, reranker: Box<dyn Reranker>) -> Self {
        self.rerankers.push(reranker);
        self
    }
}

#[async_trait]
impl Reranker for ChainReranker {
    async fn rerank(&self, query: &str, mut documents: Vec<ScoredDocument>) -> Result<Vec<ScoredDocument>> {
        // 依次应用每个重排序器
        for reranker in &self.rerankers {
            documents = reranker.rerank(query, documents).await?;
        }
        Ok(documents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Document, Metadata};

    fn create_test_document(id: &str, content: &str, score: f32) -> ScoredDocument {
        ScoredDocument {
            document: Document {
                id: id.to_string(),
                content: content.to_string(),
                metadata: Metadata::new(),
                embedding: None,
            },
            score,
        }
    }

    #[tokio::test]
    async fn test_cross_encoder_reranker() {
        let reranker = CrossEncoderReranker::new("test-model", Some(2));
        let documents = vec![
            create_test_document("1", "The cat sat on the mat", 0.5),
            create_test_document("2", "Dogs are great pets", 0.6),
            create_test_document("3", "Cats and dogs are animals", 0.7),
        ];

        let result = reranker.rerank("cat", documents).await.unwrap();
        assert_eq!(result.len(), 2); // top_k = 2
        assert!(result[0].score >= result[1].score);
    }

    #[tokio::test]
    async fn test_diversity_reranker() {
        let reranker = DiversityReranker::new(0.5, Some(3));
        let documents = vec![
            create_test_document("1", "The cat sat on the mat", 0.9),
            create_test_document("2", "The cat sat on the rug", 0.8), // Similar to 1
            create_test_document("3", "Dogs are great pets", 0.7),
        ];

        let result = reranker.rerank("cat", documents).await.unwrap();
        // Should filter out document 2 due to similarity with document 1
        assert!(result.len() <= 2);
    }
}

