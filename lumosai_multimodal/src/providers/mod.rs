//! 多模态提供商实现

pub mod openai_vision;
pub mod openai_voice;

pub use openai_vision::OpenAIVision;
pub use openai_voice::OpenAIVoice;
