//! 语音提供者实现

// 主提供者
mod mock;
mod openai;

// 重新导出
pub use mock::MockVoice;
pub use openai::OpenAIVoice;
