//! LLM模块提供了与大型语言模型交互的接口和实现

pub mod types;
pub mod provider;
pub mod mock;
pub mod function_calling;
pub mod openai;
mod anthropic;
mod qwen;
mod deepseek;
pub mod cohere;
pub mod gemini;
pub mod ollama;
pub mod together;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod new_providers_test;


pub use types::{Message, LlmOptions, Role};
pub use provider::LlmProvider;
pub use mock::MockLlmProvider;
pub use openai::OpenAiProvider;
pub use anthropic::AnthropicProvider;
pub use qwen::{QwenProvider, QwenApiType};
pub use deepseek::DeepSeekProvider;
pub use cohere::CohereProvider;
pub use gemini::GeminiProvider;
pub use ollama::OllamaProvider;
pub use together::TogetherProvider;
pub use function_calling::{
    FunctionDefinition, 
    FunctionCall, 
    FunctionCallResult, 
    ToolChoice,
    utils
};
pub mod function_calling_utils;