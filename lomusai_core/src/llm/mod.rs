//! LLM模块提供了与大型语言模型交互的接口和实现

pub mod types;
pub mod provider;
pub mod mock;
mod openai;
mod anthropic;
#[cfg(test)]
mod tests;

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use std::fmt;

pub use types::{Message, LlmOptions, Role};
pub use provider::LlmProvider;
pub use mock::MockLlmProvider;
pub use openai::OpenAiProvider;
pub use anthropic::AnthropicProvider; 