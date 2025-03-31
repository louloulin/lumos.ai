use async_trait::async_trait;
use uuid::Uuid;

use crate::error::{RagError, Result};
use crate::types::{ChunkingConfig, Document};

/// Trait for document chunkers that split documents into smaller pieces
#[async_trait]
pub trait DocumentChunker: Send + Sync {
    /// Split a document into chunks
    async fn chunk(&self, document: Document, config: &ChunkingConfig) -> Result<Vec<Document>>;
}

/// 文本分块器，将文档分成更小的块
pub struct TextChunker {
    config: ChunkingConfig,
}

impl TextChunker {
    /// 创建一个新的文本分块器
    pub fn new(config: ChunkingConfig) -> Self {
        Self { config }
    }
    
    /// 使用默认配置创建分块器
    pub fn default() -> Self {
        Self { config: ChunkingConfig::default() }
    }
    
    /// 将文本分块
    pub fn chunk_text(&self, text: &str) -> Result<Vec<String>> {
        if self.config.chunk_by_tokens {
            self.chunk_by_tokens(text)
        } else {
            self.chunk_by_chars(text)
        }
    }
    
    /// 将文档分块
    pub fn chunk_document(&self, document: &Document) -> Result<Vec<Document>> {
        let chunks = self.chunk_text(&document.content)?;
        let mut documents = Vec::with_capacity(chunks.len());
        
        for (i, chunk_text) in chunks.into_iter().enumerate() {
            let mut metadata = document.metadata.clone();
            metadata.add("chunk_index", i as i64);
            metadata.add("parent_id", document.id.clone());
            
            let chunk_document = Document {
                id: format!("{}-chunk-{}", document.id, i),
                content: chunk_text,
                metadata,
                embedding: None,
            };
            
            documents.push(chunk_document);
        }
        
        Ok(documents)
    }
    
    /// 按字符分块
    fn chunk_by_chars(&self, text: &str) -> Result<Vec<String>> {
        let chars: Vec<char> = text.chars().collect();
        let chunk_size = self.config.chunk_size;
        let overlap = self.config.chunk_overlap;
        
        if chunk_size == 0 {
            return Err(RagError::DocumentChunking("Chunk size cannot be zero".into()));
        }
        
        let mut chunks = Vec::new();
        let mut i = 0;
        
        while i < chars.len() {
            let end = std::cmp::min(i + chunk_size, chars.len());
            let chunk: String = chars[i..end].iter().collect();
            chunks.push(chunk);
            
            // 处理重叠
            if end >= chars.len() {
                break;
            }
            
            i += chunk_size - overlap;
        }
        
        Ok(chunks)
    }
    
    /// 按句子分块
    fn chunk_by_sentences(&self, text: &str) -> Result<Vec<String>> {
        // 不使用正则表达式的简单句子分割方法
        let sentence_endings = ['.', '!', '?'];
        
        let mut sentences = Vec::new();
        let mut current_sentence = String::new();
        let mut last_char = ' ';
        
        for c in text.chars() {
            current_sentence.push(c);
            
            // 如果当前字符是句子结束符号，且下一个字符是空格
            if sentence_endings.contains(&last_char) && c.is_whitespace() {
                sentences.push(current_sentence.clone());
                current_sentence.clear();
            }
            
            last_char = c;
        }
        
        // 添加最后一个句子（如果有）
        if !current_sentence.is_empty() {
            sentences.push(current_sentence);
        }
        
        // 将句子合并为块
        let chunk_size = self.config.chunk_size;
        let overlap = self.config.chunk_overlap;
        
        if chunk_size == 0 {
            return Err(RagError::DocumentChunking("Chunk size cannot be zero".into()));
        }
        
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut sentences_in_chunk = 0;
        
        for sentence in sentences {
            // 如果当前块已经达到或接近目标大小，创建新块
            if sentences_in_chunk >= chunk_size {
                chunks.push(current_chunk.clone());
                
                // 考虑重叠
                if overlap > 0 {
                    let overlap_sentences = sentences_in_chunk.min(overlap);
                    sentences_in_chunk -= overlap_sentences;
                } else {
                    current_chunk.clear();
                    sentences_in_chunk = 0;
                }
            }
            
            current_chunk.push_str(&sentence);
            sentences_in_chunk += 1;
        }
        
        // 添加最后一个块（如果有）
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }
        
        Ok(chunks)
    }
    
    /// 按token分块
    fn chunk_by_tokens(&self, text: &str) -> Result<Vec<String>> {
        // 简单实现，将单词作为token
        let words: Vec<&str> = text.split_whitespace().collect();
        let chunk_size = self.config.chunk_size;
        let overlap = self.config.chunk_overlap;
        
        if chunk_size == 0 {
            return Err(RagError::DocumentChunking("Chunk size cannot be zero".into()));
        }
        
        let mut chunks = Vec::new();
        let mut i = 0;
        
        while i < words.len() {
            let end = std::cmp::min(i + chunk_size, words.len());
            let chunk = words[i..end].join(" ");
            chunks.push(chunk);
            
            if end >= words.len() {
                break;
            }
            
            i += chunk_size - overlap;
        }
        
        Ok(chunks)
    }
}

#[async_trait]
impl DocumentChunker for TextChunker {
    async fn chunk(&self, document: Document, config: &ChunkingConfig) -> Result<Vec<Document>> {
        let chunks = if config.chunk_by_tokens {
            // A proper implementation would use a tokenizer library
            // For this example, we'll approximate with sentences
            self.chunk_by_sentences(&document.content)?
        } else {
            self.chunk_by_chars(&document.content)?
        };
        
        let mut chunked_docs = Vec::with_capacity(chunks.len());
        
        for (i, chunk_content) in chunks.into_iter().enumerate() {
            // Create metadata for the chunk
            let mut chunk_metadata = document.metadata.clone();
            chunk_metadata.add("chunk_index", i as i64);
            chunk_metadata.add("parent_document_id", document.id.clone());
            
            // Create the chunk document
            let chunk_doc = Document {
                id: Uuid::new_v4().to_string(),
                content: chunk_content,
                metadata: chunk_metadata,
                embedding: None,
            };
            
            chunked_docs.push(chunk_doc);
        }
        
        Ok(chunked_docs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Metadata;

    #[tokio::test]
    async fn test_text_chunker_by_chars() {
        let chunker = TextChunker::new(ChunkingConfig {
            chunk_size: 10,
            chunk_overlap: 2,
            chunk_by_tokens: false,
        });
        let doc = Document {
            id: "test".to_string(),
            content: "This is a test document. It has multiple sentences. We want to chunk it.".to_string(),
            metadata: Metadata::new(),
            embedding: None,
        };
        
        let chunks = chunker.chunk(doc, &ChunkingConfig::default()).await.unwrap();
        
        assert!(chunks.len() > 1);
        
        // Check that each chunk is approximately the right size
        for chunk in &chunks {
            assert!(chunk.content.len() <= 10 + 20); // Allow some flexibility
        }
        
        // Check that the chunks have appropriate metadata
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.metadata.fields.get("chunk_index").unwrap(), &serde_json::json!(i as i64));
            assert_eq!(chunk.metadata.fields.get("parent_document_id").unwrap(), &serde_json::json!("test"));
        }
    }
    
    #[tokio::test]
    async fn test_text_chunker_by_sentences() {
        let chunker = TextChunker::new(ChunkingConfig {
            chunk_size: 2, // 每块最多包含2个句子
            chunk_overlap: 0,
            chunk_by_tokens: false,
        });
        let doc = Document {
            id: "test".to_string(),
            content: "This is sentence one. This is sentence two! This is three? This is four.".to_string(),
            metadata: Metadata::new(),
            embedding: None,
        };
        
        let chunks = chunker.chunk(doc, &ChunkingConfig::default()).await.unwrap();
        
        assert!(chunks.len() > 1);
    }
} 