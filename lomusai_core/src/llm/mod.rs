//! LLM模块提供了与大型语言模型交互的接口和实现

pub mod types;
pub mod provider;
pub mod mock;
mod openai;
mod anthropic;
mod qwen;
#[cfg(test)]
mod tests;


pub use types::{Message, LlmOptions, Role};
pub use provider::LlmProvider;
pub use mock::MockLlmProvider;
pub use openai::OpenAiProvider;
pub use anthropic::AnthropicProvider;
pub use qwen::{QwenProvider, QwenApiType}; 