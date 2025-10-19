//! 多模态提供商实现

pub mod openai_voice;
pub mod openai_vision;

pub use openai_voice::OpenAIVoice;
pub use openai_vision::OpenAIVision;

