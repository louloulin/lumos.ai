use async_trait::async_trait;
use uuid::Uuid;

use crate::error::Result;
use crate::types::{Document, Metadata};

/// Trait for document parsers that process raw content into documents
#[async_trait]
pub trait DocumentParser: Send + Sync {
    /// Parse raw content into a document
    async fn parse(&self, content: &str, metadata: Metadata) -> Result<Document>;
}

/// Simple parser for plain text documents
pub struct TextParser;

#[async_trait]
impl DocumentParser for TextParser {
    async fn parse(&self, content: &str, metadata: Metadata) -> Result<Document> {
        let document = Document {
            id: Uuid::new_v4().to_string(),
            content: content.to_string(),
            metadata,
            embedding: None,
        };
        
        Ok(document)
    }
}

/// Parser for Markdown documents
pub struct MarkdownParser {
    /// Whether to strip Markdown syntax or keep it
    pub strip_syntax: bool,
}

impl MarkdownParser {
    /// Create a new Markdown parser
    pub fn new(strip_syntax: bool) -> Self {
        Self { strip_syntax }
    }
    
    /// Process Markdown content by optionally stripping syntax
    fn process_markdown(&self, content: &str) -> String {
        if self.strip_syntax {
            // Basic Markdown syntax stripping
            // A more complete implementation would use a proper Markdown parser
            let mut processed = content.to_string();
            
            // Remove headings
            processed = processed.lines()
                .map(|line| if line.starts_with('#') {
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if parts.len() > 1 { parts[1] } else { "" }
                } else {
                    line
                })
                .collect::<Vec<&str>>()
                .join("\n");
            
            // Remove links, preserving the text
            // This is a simplistic implementation
            let re = regex::Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap();
            processed = re.replace_all(&processed, "$1").to_string();
            
            processed
        } else {
            content.to_string()
        }
    }
}

#[async_trait]
impl DocumentParser for MarkdownParser {
    async fn parse(&self, content: &str, metadata: Metadata) -> Result<Document> {
        let processed_content = self.process_markdown(content);
        
        let document = Document {
            id: Uuid::new_v4().to_string(),
            content: processed_content,
            metadata,
            embedding: None,
        };
        
        Ok(document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_text_parser() {
        let parser = TextParser;
        let metadata = Metadata::new();
        
        let document = parser.parse("Test content", metadata).await.unwrap();
        
        assert_eq!(document.content, "Test content");
        assert!(document.embedding.is_none());
    }
    
    #[tokio::test]
    async fn test_markdown_parser_with_stripping() {
        let parser = MarkdownParser::new(true);
        let metadata = Metadata::new();
        
        let content = "# Heading\n\nThis is [a link](https://example.com).\n\n## Subheading\n\nMore text.";
        let document = parser.parse(content, metadata).await.unwrap();
        
        assert!(document.content.contains("Heading"));
        assert!(document.content.contains("This is a link."));
        assert!(!document.content.contains("https://example.com"));
    }
    
    #[tokio::test]
    async fn test_markdown_parser_without_stripping() {
        let parser = MarkdownParser::new(false);
        let metadata = Metadata::new();
        
        let content = "# Heading\n\nThis is [a link](https://example.com).";
        let document = parser.parse(content, metadata).await.unwrap();
        
        assert_eq!(document.content, content);
    }
} 