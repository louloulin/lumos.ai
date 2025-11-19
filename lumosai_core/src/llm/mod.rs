//! Large Language Model (LLM) module for AI text generation
//!
//! This module provides a unified interface for interacting with various
//! Large Language Model providers, including OpenAI, Anthropic, Chinese
//! providers (Qwen, Zhipu, Baidu), and open-source models.
//!
//! # Overview
//!
//! The LLM module consists of:
//!
//! - **Provider Interface** - Unified [`LlmProvider`] trait for all LLMs
//! - **Message Types** - [`Message`], [`Role`], and conversation structures
//! - **Options** - [`LlmOptions`] for controlling generation behavior
//! - **Function Calling** - OpenAI-style function calling support
//! - **Streaming** - Real-time streaming of generated text
//! - **Embeddings** - Vector embeddings for semantic search
//!
//! # Quick Start
//!
//! ## Using Factory Functions (Recommended)
//!
//! ```rust
//! use lumosai_core::llm::providers;
//!
//! # async fn example() -> lumosai_core::Result<()> {
//! // Create from environment variables
//! let provider = providers::openai_from_env()?;
//!
//! // Or auto-select available provider
//! let provider = providers::auto_provider()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Simple Text Generation
//!
//! ```rust
//! use lumosai_core::llm::{LlmProvider, LlmOptions};
//!
//! # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
//! let options = LlmOptions::default()
//!     .with_temperature(0.7)
//!     .with_max_tokens(100);
//!
//! let response = provider.generate("What is Rust?", &options).await?;
//! println!("Response: {}", response);
//! # Ok(())
//! # }
//! ```
//!
//! ## Conversation with Messages
//!
//! ```rust
//! use lumosai_core::llm::{LlmProvider, LlmOptions, Message, Role};
//!
//! # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
//! let messages = vec![
//!     Message {
//!         role: Role::System,
//!         content: "You are a helpful AI assistant.".to_string(),
//!         metadata: None,
//!         name: None,
//!     },
//!     Message {
//!         role: Role::User,
//!         content: "Explain quantum computing.".to_string(),
//!         metadata: None,
//!         name: None,
//!     },
//! ];
//!
//! let options = LlmOptions::default();
//! let response = provider.generate_with_messages(&messages, &options).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Streaming Responses
//!
//! ```rust
//! use lumosai_core::llm::{LlmProvider, LlmOptions};
//! use futures::StreamExt;
//!
//! # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
//! let options = LlmOptions::default();
//! let mut stream = provider.generate_stream("Tell me a story", &options).await?;
//!
//! while let Some(chunk) = stream.next().await {
//!     match chunk {
//!         Ok(text) => print!("{}", text),
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Supported Providers
//!
//! ## International Providers
//!
//! - **OpenAI** - GPT-3.5, GPT-4, GPT-4 Turbo ([`OpenAiProvider`])
//! - **Anthropic** - Claude 3 Opus, Sonnet, Haiku ([`AnthropicProvider`])
//! - **Cohere** - Command series models ([`CohereProvider`])
//! - **Google** - Gemini models ([`GeminiProvider`])
//! - **Together AI** - Distributed inference ([`TogetherProvider`])
//! - **Ollama** - Local open-source models ([`OllamaProvider`])
//!
//! ## Chinese Providers
//!
//! - **Qwen (通义千问)** - Alibaba's LLM ([`QwenProvider`])
//! - **Zhipu (智谱)** - GLM series models ([`ZhipuProvider`])
//! - **Baidu (百度)** - ERNIE series models ([`BaiduProvider`])
//! - **DeepSeek** - DeepSeek models ([`DeepSeekProvider`])
//!
//! # Advanced Features
//!
//! ## Function Calling
//!
//! ```rust
//! use lumosai_core::llm::{LlmProvider, LlmOptions, Message, Role};
//! use lumosai_core::llm::function_calling::{FunctionDefinition, ToolChoice};
//! use serde_json::json;
//!
//! # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
//! let messages = vec![
//!     Message {
//!         role: Role::User,
//!         content: "What's the weather in Tokyo?".to_string(),
//!         metadata: None,
//!         name: None,
//!     },
//! ];
//!
//! let functions = vec![
//!     FunctionDefinition {
//!         name: "get_weather".to_string(),
//!         description: Some("Get weather for a city".to_string()),
//!         parameters: json!({
//!             "type": "object",
//!             "properties": {
//!                 "city": {"type": "string"}
//!             },
//!             "required": ["city"]
//!         }),
//!     },
//! ];
//!
//! let options = LlmOptions::default();
//! let response = provider.generate_with_functions(
//!     &messages,
//!     &functions,
//!     &ToolChoice::Auto,
//!     &options,
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Embeddings
//!
//! ```rust
//! use lumosai_core::llm::LlmProvider;
//!
//! # async fn example(provider: &dyn LlmProvider) -> lumosai_core::Result<()> {
//! let embedding = provider.get_embedding("Hello, world!").await?;
//! println!("Embedding dimension: {}", embedding.len());
//! # Ok(())
//! # }
//! ```
//!
//! # See Also
//!
//! - [`LlmProvider`] - Core provider trait
//! - [`LlmOptions`] - Generation options
//! - [`Message`] - Message structure
//! - [`providers`] - Factory functions for creating providers
//! - [`function_calling`] - Function calling support

mod anthropic;
pub mod baidu;
pub mod claude;
pub mod cohere;
mod deepseek;
pub mod function_calling;
pub mod gemini;
pub mod huawei_maas;
pub mod mock;
pub mod ollama;
pub mod openai;
pub mod provider;
pub mod providers;
mod qwen;
pub mod test_helpers;
#[cfg(test)]
mod tests;
pub mod together;
pub mod types;
pub mod zhipu;

#[cfg(test)]
mod new_providers_test;
#[cfg(test)]
mod real_api_tests;
// Temporarily disabled due to missing imports
// mod third_party_integration_test;

pub use anthropic::AnthropicProvider;
pub use baidu::BaiduProvider;
pub use claude::ClaudeProvider;
pub use cohere::CohereProvider;
pub use deepseek::DeepSeekProvider;
pub use function_calling::{
    utils, FunctionCall, FunctionCallResult, FunctionDefinition, ToolChoice,
};
pub use gemini::GeminiProvider;
pub use huawei_maas::HuaweiMaasProvider;
pub use mock::MockLlmProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;
pub use provider::LlmProvider;
pub use qwen::{QwenApiType, QwenProvider};
pub use together::TogetherProvider;
pub use types::{LlmOptions, Message, Role};
pub use zhipu::ZhipuProvider;
pub mod function_calling_utils;
