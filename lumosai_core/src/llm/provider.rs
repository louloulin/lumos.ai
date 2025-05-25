use async_trait::async_trait;
use futures::stream::BoxStream;
use serde_json::Value;

use crate::Result;
use super::types::{LlmOptions, Message};
use super::function_calling::{FunctionDefinition, FunctionCall, ToolChoice};

/// Trait representing an LLM provider
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generate text from a prompt
    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String>;
    
    /// Generate text from a sequence of messages
    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String>;
    
    /// Generate a stream of text from a prompt
    async fn generate_stream<'a>(
        &'a self, 
        prompt: &'a str, 
        options: &'a LlmOptions
    ) -> Result<BoxStream<'a, Result<String>>>;
    
    /// Get embeddings for a text
    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>>;
    
    /// Check if the provider supports OpenAI function calling
    fn supports_function_calling(&self) -> bool {
        false
    }
    
    /// Generate text with function calling support
    async fn generate_with_functions(
        &self,
        messages: &[Message],
        functions: &[FunctionDefinition],
        tool_choice: &ToolChoice,
        options: &LlmOptions,
    ) -> Result<FunctionCallingResponse> {
        // Default implementation for providers that don't support function calling
        let _ = (functions, tool_choice);
        let response = self.generate_with_messages(messages, options).await?;
        Ok(FunctionCallingResponse {
            content: Some(response),
            function_calls: Vec::new(),
            finish_reason: "stop".to_string(),
        })
    }
}

/// Response from a function calling enabled LLM
#[derive(Debug, Clone)]
pub struct FunctionCallingResponse {
    /// Text content from the model (if any)
    pub content: Option<String>,
    /// Function calls made by the model
    pub function_calls: Vec<FunctionCall>,
    /// Reason the generation finished
    pub finish_reason: String,
} 