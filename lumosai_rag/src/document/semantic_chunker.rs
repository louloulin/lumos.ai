//! 语义分块（Semantic Chunking）模块
//!
//! 基于语义相似度进行智能文档分块，而不是简单的固定长度分块

use async_trait::async_trait;
use std::sync::Arc;

use crate::{embedding::EmbeddingProvider, error::Result, types::Document};

/// 智能文档分块器 trait
///
/// 与基础的 DocumentChunker 不同，SmartChunker 不需要配置参数，
/// 而是根据文档内容自动选择最佳分块策略
#[async_trait]
pub trait SmartChunker: Send + Sync {
    /// 将文档分块
    ///
    /// # 参数
    /// - `document`: 待分块的文档
    ///
    /// # 返回
    /// 分块后的文档列表
    async fn chunk(&self, document: &Document) -> Result<Vec<Document>>;
}

/// 语义分块器
///
/// 基于句子间的语义相似度进行分块，确保每个块内的内容语义连贯
pub struct SemanticChunker {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    similarity_threshold: f32,
    min_chunk_size: usize,
    max_chunk_size: usize,
}

impl SemanticChunker {
    /// 创建新的语义分块器
    ///
    /// # 参数
    /// - `embedding_provider`: 嵌入提供商
    /// - `similarity_threshold`: 相似度阈值（0.0-1.0），低于此值则分块
    /// - `min_chunk_size`: 最小块大小（字符数）
    /// - `max_chunk_size`: 最大块大小（字符数）
    pub fn new(
        embedding_provider: Arc<dyn EmbeddingProvider>,
        similarity_threshold: f32,
        min_chunk_size: usize,
        max_chunk_size: usize,
    ) -> Self {
        Self {
            embedding_provider,
            similarity_threshold,
            min_chunk_size,
            max_chunk_size,
        }
    }

    /// 将文本分割成句子
    fn split_into_sentences(&self, text: &str) -> Vec<String> {
        // 简单的句子分割（基于标点符号）
        let mut sentences = Vec::new();
        let mut current_sentence = String::new();

        for ch in text.chars() {
            current_sentence.push(ch);

            // 句子结束标记
            if matches!(ch, '.' | '!' | '?' | '。' | '！' | '？' | '\n') {
                let trimmed = current_sentence.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                    current_sentence.clear();
                }
            }
        }

        // 添加最后一个句子
        let trimmed = current_sentence.trim().to_string();
        if !trimmed.is_empty() {
            sentences.push(trimmed);
        }

        sentences
    }

    /// 计算两个嵌入向量的余弦相似度
    fn cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() || vec1.is_empty() {
            return 0.0;
        }

        let mut dot_product = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;

        for i in 0..vec1.len() {
            dot_product += vec1[i] * vec2[i];
            norm1 += vec1[i] * vec1[i];
            norm2 += vec2[i] * vec2[i];
        }

        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }

        dot_product / (norm1.sqrt() * norm2.sqrt())
    }
}

#[async_trait]
impl SmartChunker for SemanticChunker {
    async fn chunk(&self, document: &Document) -> Result<Vec<Document>> {
        // 1. 分割成句子
        let sentences = self.split_into_sentences(&document.content);

        if sentences.is_empty() {
            return Ok(vec![document.clone()]);
        }

        // 2. 为每个句子生成嵌入
        let embeddings = self.embedding_provider.embed_batch(&sentences).await?;

        // 3. 基于语义相似度分块
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        let mut current_chunk_size = 0;

        for i in 0..sentences.len() {
            let sentence = &sentences[i];
            let sentence_len = sentence.len();

            // 检查是否需要开始新块
            let should_split = if current_chunk.is_empty() {
                false
            } else if current_chunk_size + sentence_len > self.max_chunk_size {
                true
            } else if i > 0 && current_chunk_size >= self.min_chunk_size {
                // 计算与前一个句子的相似度
                let similarity = self.cosine_similarity(&embeddings[i - 1], &embeddings[i]);
                similarity < self.similarity_threshold
            } else {
                false
            };

            if should_split {
                // 创建新块
                let chunk_content = current_chunk.join(" ");
                let mut chunk_doc = document.clone();
                chunk_doc.id = format!("{}_chunk_{}", document.id, chunks.len());
                chunk_doc.content = chunk_content;
                chunk_doc.embedding = None; // 需要重新生成嵌入
                chunks.push(chunk_doc);

                // 重置当前块
                current_chunk.clear();
                current_chunk_size = 0;
            }

            // 添加句子到当前块
            current_chunk.push(sentence.clone());
            current_chunk_size += sentence_len;
        }

        // 添加最后一个块
        if !current_chunk.is_empty() {
            let chunk_content = current_chunk.join(" ");
            let mut chunk_doc = document.clone();
            chunk_doc.id = format!("{}_chunk_{}", document.id, chunks.len());
            chunk_doc.content = chunk_content;
            chunk_doc.embedding = None;
            chunks.push(chunk_doc);
        }

        Ok(chunks)
    }
}

/// 自适应分块器
///
/// 根据文档类型和内容自动选择最佳分块策略
pub struct AdaptiveChunker {
    default_chunk_size: usize,
    overlap_size: usize,
}

impl AdaptiveChunker {
    /// 创建新的自适应分块器
    pub fn new(default_chunk_size: usize, overlap_size: usize) -> Self {
        Self {
            default_chunk_size,
            overlap_size,
        }
    }

    /// 检测文档类型
    fn detect_document_type(&self, content: &str) -> DocumentType {
        // 简单的启发式检测
        if content.contains("```") || content.contains("def ") || content.contains("function ") {
            DocumentType::Code
        } else if content.contains("# ") || content.contains("## ") {
            DocumentType::Markdown
        } else if content.lines().count() > 100 && content.len() / content.lines().count() < 100 {
            DocumentType::List
        } else {
            DocumentType::Prose
        }
    }

    /// 根据文档类型选择分块大小
    fn get_chunk_size_for_type(&self, doc_type: DocumentType) -> usize {
        match doc_type {
            DocumentType::Code => self.default_chunk_size / 2, // 代码块较小
            DocumentType::Markdown => self.default_chunk_size,
            DocumentType::List => self.default_chunk_size / 4, // 列表项较小
            DocumentType::Prose => self.default_chunk_size,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DocumentType {
    Code,
    Markdown,
    List,
    Prose,
}

#[async_trait]
impl SmartChunker for AdaptiveChunker {
    async fn chunk(&self, document: &Document) -> Result<Vec<Document>> {
        // 1. 检测文档类型
        let doc_type = self.detect_document_type(&document.content);
        let chunk_size = self.get_chunk_size_for_type(doc_type);

        // 2. 根据类型进行分块
        let mut chunks = Vec::new();
        let content = &document.content;
        let content_len = content.len();

        if content_len <= chunk_size {
            return Ok(vec![document.clone()]);
        }

        let mut start = 0;
        let mut chunk_index = 0;

        while start < content_len {
            let end = (start + chunk_size).min(content_len);

            // 尝试在单词边界处分割
            let actual_end = if end < content_len {
                content[start..end]
                    .rfind(|c: char| c.is_whitespace())
                    .map(|pos| start + pos)
                    .unwrap_or(end)
            } else {
                end
            };

            let chunk_content = content[start..actual_end].trim().to_string();

            if !chunk_content.is_empty() {
                let mut chunk_doc = document.clone();
                chunk_doc.id = format!("{}_chunk_{}", document.id, chunk_index);
                chunk_doc.content = chunk_content;
                chunk_doc.embedding = None;
                chunks.push(chunk_doc);
                chunk_index += 1;
            }

            // 移动到下一个块，考虑重叠
            start = if actual_end > self.overlap_size {
                actual_end - self.overlap_size
            } else {
                actual_end
            };
        }

        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Metadata;

    fn create_test_document(id: &str, content: &str) -> Document {
        Document {
            id: id.to_string(),
            content: content.to_string(),
            metadata: Metadata::new(),
            embedding: None,
        }
    }

    #[tokio::test]
    async fn test_adaptive_chunker() {
        let chunker = AdaptiveChunker::new(100, 20);
        let doc = create_test_document(
            "test",
            "This is a test document. It has multiple sentences. Each sentence should be preserved. The chunker should split at word boundaries.",
        );

        let chunks = chunker.chunk(&doc).await.unwrap();
        assert!(!chunks.is_empty());

        // 验证每个块都有唯一 ID
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.id, format!("test_chunk_{}", i));
        }
    }

    #[test]
    fn test_document_type_detection() {
        let chunker = AdaptiveChunker::new(1000, 100);

        let code_doc = create_test_document(
            "code",
            "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```",
        );
        assert!(matches!(
            chunker.detect_document_type(&code_doc.content),
            DocumentType::Code
        ));

        let markdown_doc = create_test_document("md", "# Title\n## Subtitle\nContent");
        assert!(matches!(
            chunker.detect_document_type(&markdown_doc.content),
            DocumentType::Markdown
        ));
    }
}
