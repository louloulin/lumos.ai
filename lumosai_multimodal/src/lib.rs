//! LumosAI 多模态能力集成
//!
//! 提供语音处理（STT/TTS）和视觉处理（图像理解/生成）能力
//!
//! # 功能模块
//!
//! - **语音处理**: 语音识别（STT）和语音合成（TTS）
//! - **视觉处理**: 图像理解和图像生成
//! - **多模态消息**: 统一的多模态消息类型
//!
//! # 对标框架
//!
//! - OpenAI API: Whisper (STT), TTS, GPT-4V, DALL-E
//! - 行业标准: 多模态 AI 能力集成
//!
//! # 使用示例
//!
//! ```rust,no_run
//! use lumosai_multimodal::{VoiceProvider, VisionProvider, OpenAIVoice, OpenAIVision};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // 语音识别
//! let voice = OpenAIVoice::new("your-api-key");
//! let text = voice.transcribe_file("audio.mp3").await?;
//!
//! // 语音合成
//! let audio = voice.synthesize("Hello, world!", "alloy").await?;
//!
//! // 图像理解
//! let vision = OpenAIVision::new("your-api-key");
//! let description = vision.describe_image("image.jpg", "What's in this image?").await?;
//!
//! // 图像生成
//! let image_url = vision.generate_image("A beautiful sunset", "1024x1024").await?;
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod providers;
pub mod types;
pub mod vision;
pub mod voice;

// 重新导出核心类型
pub use error::{MultimodalError, Result};
pub use providers::{OpenAIVision, OpenAIVoice};
pub use types::{
    AudioFormat, GenerationOptions, ImageFormat, ImageSize, MultimodalContent, MultimodalMessage,
    SynthesisOptions, TranscriptionOptions, VisionOptions,
};
pub use vision::{VisionCapabilities, VisionProvider};
pub use voice::{VoiceCapabilities, VoiceProvider};

/// 多模态能力集成的版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
