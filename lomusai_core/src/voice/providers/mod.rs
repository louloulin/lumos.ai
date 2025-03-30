//! 语音提供者实现

// 主提供者
mod openai;
mod mock;

// 重新导出
pub use openai::OpenAIVoice;
pub use mock::MockVoice; 