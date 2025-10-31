//! LLM模块提供了与大型语言模型交互的接口和实现

mod anthropic;
pub mod baidu;
pub mod claude;
pub mod cohere;
mod deepseek;
pub mod function_calling;
pub mod gemini;
pub mod mock;
pub mod ollama;
pub mod openai;
pub mod provider;
pub mod providers;
mod qwen;
#[cfg(test)]
mod tests;
pub mod test_helpers;
pub mod together;
pub mod types;
pub mod zhipu;

#[cfg(test)]
mod new_providers_test;
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
pub use mock::MockLlmProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;
pub use provider::LlmProvider;
pub use qwen::{QwenApiType, QwenProvider};
pub use together::TogetherProvider;
pub use types::{LlmOptions, Message, Role};
pub use zhipu::ZhipuProvider;
pub mod function_calling_utils;
