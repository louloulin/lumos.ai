use async_trait::async_trait;
use uuid::Uuid;
use regex::Regex;

use crate::error::{RagError, Result};
use crate::types::{ChunkingConfig, ChunkingStrategy, Document};

/// Trait for document chunkers that split documents into smaller pieces
#[async_trait]
pub trait DocumentChunker: Send + Sync {
    /// Split a document into chunks
    async fn chunk(&self, document: Document, config: &ChunkingConfig) -> Result<Vec<Document>>;
}

/// Enhanced document chunker supporting multiple strategies
pub struct EnhancedChunker;

impl EnhancedChunker {
    pub fn new() -> Self {
        Self
    }

    /// Chunk document using the specified strategy
    pub async fn chunk_with_strategy(
        &self,
        document: Document,
        config: &ChunkingConfig,
    ) -> Result<Vec<Document>> {
        match &config.strategy {
            ChunkingStrategy::Recursive { separators, is_separator_regex } => {
                self.chunk_recursive(&document, config, separators.as_ref(), *is_separator_regex).await
            }
            ChunkingStrategy::Character { separator, is_separator_regex } => {
                self.chunk_character(&document, config, separator, *is_separator_regex).await
            }
            ChunkingStrategy::Token { encoding_name: _, model_name: _ } => {
                self.chunk_token(&document, config).await
            }
            ChunkingStrategy::Markdown { headers, return_each_line, strip_headers } => {
                self.chunk_markdown(&document, config, headers.as_ref(), *return_each_line, *strip_headers).await
            }
            ChunkingStrategy::Html { headers, sections, return_each_line } => {
                self.chunk_html(&document, config, headers.as_ref(), sections.as_ref(), *return_each_line).await
            }
            ChunkingStrategy::Json { ensure_ascii: _, convert_lists: _ } => {
                self.chunk_json(&document, config).await
            }
            ChunkingStrategy::Latex => {
                self.chunk_latex(&document, config).await
            }
        }
    }

    /// Recursive character-based chunking (similar to Mastra's RecursiveCharacterTransformer)
    async fn chunk_recursive(
        &self,
        document: &Document,
        config: &ChunkingConfig,
        separators: Option<&Vec<String>>,
        is_separator_regex: bool,
    ) -> Result<Vec<Document>> {
        let default_separators = vec![
            "\n\n".to_string(),
            "\n".to_string(),
            " ".to_string(),
            "".to_string(),
        ];

        let seps = separators.unwrap_or(&default_separators);
        let chunks = self.split_text_recursive(&document.content, seps, config, is_separator_regex)?;

        self.create_chunk_documents(document, chunks)
    }

    /// Character-based chunking
    async fn chunk_character(
        &self,
        document: &Document,
        config: &ChunkingConfig,
        separator: &str,
        is_separator_regex: bool,
    ) -> Result<Vec<Document>> {
        let chunks = if is_separator_regex {
            let regex = Regex::new(separator)
                .map_err(|e| RagError::DocumentChunking(format!("Invalid regex: {}", e)))?;
            regex.split(&document.content).map(|s| s.to_string()).collect()
        } else {
            document.content.split(separator).map(|s| s.to_string()).collect()
        };

        let merged_chunks = self.merge_chunks(chunks, config)?;
        self.create_chunk_documents(document, merged_chunks)
    }

    /// Token-based chunking (simplified implementation)
    async fn chunk_token(
        &self,
        document: &Document,
        config: &ChunkingConfig,
    ) -> Result<Vec<Document>> {
        // Simple word-based tokenization for now
        // In a real implementation, you'd use a proper tokenizer like tiktoken
        let words: Vec<&str> = document.content.split_whitespace().collect();
        let chunk_size = config.chunk_size;
        let overlap = config.chunk_overlap;

        let mut chunks = Vec::new();
        let mut i = 0;

        while i < words.len() {
            let end = std::cmp::min(i + chunk_size, words.len());
            let chunk = words[i..end].join(" ");
            chunks.push(chunk);

            if end >= words.len() {
                break;
            }

            i += chunk_size.saturating_sub(overlap);
        }

        self.create_chunk_documents(document, chunks)
    }

    /// Markdown-aware chunking
    async fn chunk_markdown(
        &self,
        document: &Document,
        config: &ChunkingConfig,
        headers: Option<&Vec<String>>,
        return_each_line: bool,
        strip_headers: bool,
    ) -> Result<Vec<Document>> {
        if let Some(header_levels) = headers {
            self.chunk_by_headers(&document.content, header_levels, return_each_line, strip_headers, config, document).await
        } else {
            // Default markdown chunking by paragraphs
            let chunks: Vec<String> = document.content
                .split("\n\n")
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string())
                .collect();

            let merged_chunks = self.merge_chunks(chunks, config)?;
            self.create_chunk_documents(document, merged_chunks)
        }
    }

    /// HTML-aware chunking
    async fn chunk_html(
        &self,
        document: &Document,
        config: &ChunkingConfig,
        headers: Option<&Vec<String>>,
        sections: Option<&Vec<String>>,
        return_each_line: bool,
    ) -> Result<Vec<Document>> {
        if let Some(header_tags) = headers {
            self.chunk_by_html_headers(&document.content, header_tags, return_each_line, config, document).await
        } else if let Some(section_tags) = sections {
            self.chunk_by_html_sections(&document.content, section_tags, config, document).await
        } else {
            return Err(RagError::DocumentChunking(
                "HTML chunking requires either headers or sections to be specified".into()
            ));
        }
    }

    /// JSON-aware chunking
    async fn chunk_json(
        &self,
        document: &Document,
        config: &ChunkingConfig,
    ) -> Result<Vec<Document>> {
        // Parse JSON and chunk by objects/arrays
        let json_value: serde_json::Value = serde_json::from_str(&document.content)
            .map_err(|e| RagError::DocumentChunking(format!("Invalid JSON: {}", e)))?;

        let chunks = self.chunk_json_value(&json_value, config.chunk_size)?;
        self.create_chunk_documents(document, chunks)
    }

    /// LaTeX-aware chunking
    async fn chunk_latex(
        &self,
        document: &Document,
        config: &ChunkingConfig,
    ) -> Result<Vec<Document>> {
        // LaTeX-specific separators
        let latex_separators = vec![
            "\\section{".to_string(),
            "\\subsection{".to_string(),
            "\\subsubsection{".to_string(),
            "\\paragraph{".to_string(),
            "\n\n".to_string(),
            "\n".to_string(),
        ];

        let chunks = self.split_text_recursive(&document.content, &latex_separators, config, false)?;
        self.create_chunk_documents(document, chunks)
    }

    /// Helper method to split text recursively
    fn split_text_recursive(
        &self,
        text: &str,
        separators: &[String],
        config: &ChunkingConfig,
        is_separator_regex: bool,
    ) -> Result<Vec<String>> {
        if separators.is_empty() {
            return Ok(vec![text.to_string()]);
        }

        let separator = &separators[0];
        let remaining_separators = &separators[1..];

        let splits: Vec<String> = if is_separator_regex {
            let regex = Regex::new(separator)
                .map_err(|e| RagError::DocumentChunking(format!("Invalid regex: {}", e)))?;
            regex.split(text).map(|s| s.to_string()).collect()
        } else {
            text.split(separator).map(|s| s.to_string()).collect()
        };

        let mut final_chunks = Vec::new();

        for split in splits {
            let split_str: String = split;
            if split_str.len() <= config.chunk_size {
                final_chunks.push(split_str);
            } else if !remaining_separators.is_empty() {
                let sub_chunks = self.split_text_recursive(&split_str, remaining_separators, config, is_separator_regex)?;
                final_chunks.extend(sub_chunks);
            } else {
                // Force split by characters if no more separators
                let char_chunks = self.force_split_by_chars(&split_str, config.chunk_size);
                final_chunks.extend(char_chunks);
            }
        }

        Ok(self.merge_chunks(final_chunks, config)?)
    }

    /// Force split text by characters when no separators work
    fn force_split_by_chars(&self, text: &str, chunk_size: usize) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        let mut chunks = Vec::new();

        for chunk_chars in chars.chunks(chunk_size) {
            let chunk: String = chunk_chars.iter().collect();
            chunks.push(chunk);
        }

        chunks
    }

    /// Merge small chunks together to reach target size
    fn merge_chunks(&self, chunks: Vec<String>, config: &ChunkingConfig) -> Result<Vec<String>> {
        if chunks.is_empty() {
            return Ok(chunks);
        }

        let mut merged = Vec::new();
        let mut current_chunk = String::new();

        for chunk in chunks {
            let chunk = chunk.trim();
            if chunk.is_empty() {
                continue;
            }

            // Check if adding this chunk would exceed the limit
            if !current_chunk.is_empty() &&
               current_chunk.len() + chunk.len() + 1 > config.chunk_size {
                // Save current chunk and start a new one
                merged.push(current_chunk.clone());
                current_chunk = chunk.to_string();
            } else {
                // Add to current chunk
                if !current_chunk.is_empty() {
                    current_chunk.push(' ');
                }
                current_chunk.push_str(chunk);
            }
        }

        // Add the last chunk
        if !current_chunk.is_empty() {
            merged.push(current_chunk);
        }

        Ok(merged)
    }

    /// Create chunk documents from text chunks
    fn create_chunk_documents(&self, original: &Document, chunks: Vec<String>) -> Result<Vec<Document>> {
        let mut documents = Vec::with_capacity(chunks.len());

        for (i, chunk_content) in chunks.into_iter().enumerate() {
            if chunk_content.trim().is_empty() {
                continue;
            }

            let mut chunk_metadata = original.metadata.clone();
            chunk_metadata.add("chunk_index", i as i64);
            chunk_metadata.add("parent_document_id", original.id.clone());
            chunk_metadata.add("chunk_type", "text");

            let chunk_doc = Document {
                id: format!("{}-chunk-{}", original.id, i),
                content: chunk_content.trim().to_string(),
                metadata: chunk_metadata,
                embedding: None,
            };

            documents.push(chunk_doc);
        }

        Ok(documents)
    }

    /// Chunk by markdown headers
    async fn chunk_by_headers(
        &self,
        content: &str,
        headers: &[String],
        return_each_line: bool,
        strip_headers: bool,
        config: &ChunkingConfig,
        document: &Document,
    ) -> Result<Vec<Document>> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut _current_header = String::new();

        for line in content.lines() {
            let mut is_header = false;

            // Check if line is a header
            for header_level in headers {
                if line.starts_with(header_level) {
                    // Save previous chunk
                    if !current_chunk.is_empty() {
                        chunks.push(current_chunk.clone());
                        current_chunk.clear();
                    }

                    _current_header = if strip_headers {
                        line.trim_start_matches(header_level).trim().to_string()
                    } else {
                        line.to_string()
                    };

                    if !strip_headers {
                        current_chunk.push_str(line);
                        current_chunk.push('\n');
                    }

                    is_header = true;
                    break;
                }
            }

            if !is_header {
                if return_each_line {
                    if !current_chunk.is_empty() {
                        chunks.push(current_chunk.clone());
                        current_chunk.clear();
                    }
                    current_chunk.push_str(line);
                } else {
                    current_chunk.push_str(line);
                    current_chunk.push('\n');
                }
            }
        }

        // Add the last chunk
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        let merged_chunks = self.merge_chunks(chunks, config)?;
        self.create_chunk_documents(document, merged_chunks)
    }

    /// Chunk by HTML headers
    async fn chunk_by_html_headers(
        &self,
        content: &str,
        headers: &[String],
        return_each_line: bool,
        config: &ChunkingConfig,
        document: &Document,
    ) -> Result<Vec<Document>> {
        // Simple HTML header detection (in a real implementation, use an HTML parser)
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for line in content.lines() {
            let mut is_header = false;

            for header_tag in headers {
                if line.contains(&format!("<{}>", header_tag)) ||
                   line.contains(&format!("<{} ", header_tag)) {
                    if !current_chunk.is_empty() {
                        chunks.push(current_chunk.clone());
                        current_chunk.clear();
                    }
                    is_header = true;
                    break;
                }
            }

            if return_each_line && !is_header {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.clone());
                    current_chunk.clear();
                }
            }

            current_chunk.push_str(line);
            current_chunk.push('\n');
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        let merged_chunks = self.merge_chunks(chunks, config)?;
        self.create_chunk_documents(document, merged_chunks)
    }

    /// Chunk by HTML sections
    async fn chunk_by_html_sections(
        &self,
        content: &str,
        sections: &[String],
        config: &ChunkingConfig,
        document: &Document,
    ) -> Result<Vec<Document>> {
        // Simple implementation - split by section tags
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for line in content.lines() {
            let mut _is_section_start = false;

            for section_tag in sections {
                if line.contains(&format!("<{}>", section_tag)) ||
                   line.contains(&format!("<{} ", section_tag)) {
                    if !current_chunk.is_empty() {
                        chunks.push(current_chunk.clone());
                        current_chunk.clear();
                    }
                    _is_section_start = true;
                    break;
                }
            }

            current_chunk.push_str(line);
            current_chunk.push('\n');
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        let merged_chunks = self.merge_chunks(chunks, config)?;
        self.create_chunk_documents(document, merged_chunks)
    }

    /// Chunk JSON value recursively
    fn chunk_json_value(&self, value: &serde_json::Value, max_size: usize) -> Result<Vec<String>> {
        let mut chunks = Vec::new();

        match value {
            serde_json::Value::Object(obj) => {
                for (key, val) in obj {
                    let val_str = serde_json::to_string_pretty(val)
                        .map_err(|e| RagError::DocumentChunking(format!("JSON serialization error: {}", e)))?;

                    if val_str.len() <= max_size {
                        chunks.push(format!("{}: {}", key, val_str));
                    } else {
                        let sub_chunks = self.chunk_json_value(val, max_size)?;
                        for sub_chunk in sub_chunks {
                            chunks.push(format!("{}: {}", key, sub_chunk));
                        }
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let val_str = serde_json::to_string_pretty(val)
                        .map_err(|e| RagError::DocumentChunking(format!("JSON serialization error: {}", e)))?;

                    if val_str.len() <= max_size {
                        chunks.push(format!("[{}]: {}", i, val_str));
                    } else {
                        let sub_chunks = self.chunk_json_value(val, max_size)?;
                        for sub_chunk in sub_chunks {
                            chunks.push(format!("[{}]: {}", i, sub_chunk));
                        }
                    }
                }
            }
            _ => {
                let val_str = serde_json::to_string_pretty(value)
                    .map_err(|e| RagError::DocumentChunking(format!("JSON serialization error: {}", e)))?;
                chunks.push(val_str);
            }
        }

        Ok(chunks)
    }
}

#[async_trait]
impl DocumentChunker for EnhancedChunker {
    async fn chunk(&self, document: Document, config: &ChunkingConfig) -> Result<Vec<Document>> {
        self.chunk_with_strategy(document, config).await
    }
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
        // Use character-based chunking by default
        self.chunk_by_chars(text)
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
        let chunks = match &config.strategy {
            crate::types::ChunkingStrategy::Token { .. } => {
                // A proper implementation would use a tokenizer library
                // For this example, we'll approximate with sentences
                self.chunk_by_sentences(&document.content)?
            }
            _ => {
                self.chunk_by_chars(&document.content)?
            }
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
            strategy: crate::types::ChunkingStrategy::Character {
                separator: " ".to_string(),
                is_separator_regex: false,
            },
            ..Default::default()
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
            strategy: crate::types::ChunkingStrategy::Character {
                separator: ".".to_string(),
                is_separator_regex: false,
            },
            ..Default::default()
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