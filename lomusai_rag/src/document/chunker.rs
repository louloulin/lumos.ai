use async_trait::async_trait;
use uuid::Uuid;

use crate::error::Result;
use crate::types::{ChunkingConfig, Document};

/// Trait for document chunkers that split documents into smaller pieces
#[async_trait]
pub trait DocumentChunker: Send + Sync {
    /// Split a document into chunks
    async fn chunk(&self, document: Document, config: &ChunkingConfig) -> Result<Vec<Document>>;
}

/// Simple chunker for text documents
pub struct TextChunker;

impl TextChunker {
    /// Split text by character count
    fn split_by_chars(&self, text: &str, chunk_size: usize, chunk_overlap: usize) -> Vec<String> {
        if text.len() <= chunk_size {
            return vec![text.to_string()];
        }
        
        let mut chunks = Vec::new();
        let mut start = 0;
        
        while start < text.len() {
            // Determine the end position for this chunk
            let mut end = start + chunk_size;
            if end >= text.len() {
                end = text.len();
            } else {
                // Try to find a good split point (whitespace) near the end
                let substr = &text[end.saturating_sub(20)..end.min(text.len())];
                if let Some(pos) = substr.rfind(char::is_whitespace) {
                    end = end.saturating_sub(20) + pos + 1; // +1 to include the whitespace
                }
            }
            
            // Add the chunk
            chunks.push(text[start..end].to_string());
            
            // Move the start position for the next chunk, considering overlap
            start = if end == text.len() {
                end  // No more chunks
            } else {
                end - chunk_overlap
            };
        }
        
        chunks
    }
    
    /// Split text by sentences (approximation)
    fn split_by_sentences(&self, text: &str, target_size: usize, chunk_overlap: usize) -> Vec<String> {
        // Simple sentence splitter using common ending punctuation
        let sentence_split_regex = regex::Regex::new(r"(?<=[.!?])\s+").unwrap();
        let sentences: Vec<&str> = sentence_split_regex.split(text).collect();
        
        if sentences.len() == 1 || sentences.iter().map(|s| s.len()).sum::<usize>() <= target_size {
            return vec![text.to_string()];
        }
        
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut last_sentences = Vec::new();
        
        for sentence in sentences {
            if current_chunk.len() + sentence.len() > target_size && !current_chunk.is_empty() {
                // The current chunk will exceed the target size if we add this sentence
                chunks.push(current_chunk.clone());
                
                // Prepare for overlap
                current_chunk = String::new();
                
                // Add overlapping sentences from the end of the previous chunk
                // This is an approximation, more sophisticated overlap would track tokens
                let overlap_size = chunk_overlap.min(last_sentences.len());
                for i in last_sentences.len() - overlap_size..last_sentences.len() {
                    current_chunk.push_str(last_sentences[i]);
                    current_chunk.push_str(" ");
                }
                
                last_sentences.clear();
            }
            
            current_chunk.push_str(sentence);
            current_chunk.push_str(" ");
            last_sentences.push(sentence);
        }
        
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }
        
        chunks
    }
}

#[async_trait]
impl DocumentChunker for TextChunker {
    async fn chunk(&self, document: Document, config: &ChunkingConfig) -> Result<Vec<Document>> {
        let chunks = if config.chunk_by_tokens {
            // A proper implementation would use a tokenizer library
            // For this example, we'll approximate with sentences
            self.split_by_sentences(&document.content, config.chunk_size, config.chunk_overlap)
        } else {
            self.split_by_chars(&document.content, config.chunk_size, config.chunk_overlap)
        };
        
        let mut chunked_docs = Vec::with_capacity(chunks.len());
        
        for (i, chunk_content) in chunks.into_iter().enumerate() {
            // Create metadata for the chunk
            let mut chunk_metadata = document.metadata.clone();
            chunk_metadata.add("chunk_index", i);
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
        let chunker = TextChunker;
        let doc = Document {
            id: "test".to_string(),
            content: "This is a test document. It has multiple sentences. We want to chunk it.".to_string(),
            metadata: Metadata::new(),
            embedding: None,
        };
        
        let config = ChunkingConfig {
            chunk_size: 20,
            chunk_overlap: 5,
            chunk_by_tokens: false,
        };
        
        let chunks = chunker.chunk(doc, &config).await.unwrap();
        
        assert!(chunks.len() > 1);
        
        // Check that each chunk is approximately the right size
        for chunk in &chunks {
            assert!(chunk.content.len() <= config.chunk_size + 20); // Allow some flexibility
        }
        
        // Check that the chunks have appropriate metadata
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.metadata.fields.get("chunk_index").unwrap(), &serde_json::json!(i));
            assert_eq!(chunk.metadata.fields.get("parent_document_id").unwrap(), &serde_json::json!("test"));
        }
    }
    
    #[tokio::test]
    async fn test_text_chunker_by_sentences() {
        let chunker = TextChunker;
        let doc = Document {
            id: "test".to_string(),
            content: "This is a test document. It has multiple sentences. We want to chunk it properly. Each sentence should be handled correctly. Let's see if it works as expected.".to_string(),
            metadata: Metadata::new(),
            embedding: None,
        };
        
        let config = ChunkingConfig {
            chunk_size: 50,
            chunk_overlap: 10,
            chunk_by_tokens: true,
        };
        
        let chunks = chunker.chunk(doc, &config).await.unwrap();
        
        assert!(chunks.len() > 1);
    }
} 