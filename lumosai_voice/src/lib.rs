//! LumosAI 语音处理模块
//!
//! 提供语音合成、语音识别等功能

// 简化实现，移除复杂的 providers 模块

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 语音选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceOptions {
    pub voice: String,
    pub speed: f32,
    pub pitch: f32,
}

/// 听取选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenOptions {
    pub language: String,
    pub timeout: u64,
}

/// 语音提供商 trait
#[async_trait]
pub trait VoiceProvider: Send + Sync {
    async fn speak(&self, text: &str, options: &VoiceOptions) -> Result<Vec<u8>>;
    async fn listen(&self, audio: &[u8], options: &ListenOptions) -> Result<String>;
}

/// 复合语音提供商
pub struct CompositeVoice {
    providers: Vec<Box<dyn VoiceProvider>>,
}

impl Default for CompositeVoice {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeVoice {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn add_provider(mut self, provider: Box<dyn VoiceProvider>) -> Self {
        self.providers.push(provider);
        self
    }
}

#[async_trait]
impl VoiceProvider for CompositeVoice {
    async fn speak(&self, text: &str, options: &VoiceOptions) -> Result<Vec<u8>> {
        for provider in &self.providers {
            if let Ok(result) = provider.speak(text, options).await {
                return Ok(result);
            }
        }
        Err(VoiceError::NoProviderAvailable)
    }

    async fn listen(&self, audio: &[u8], options: &ListenOptions) -> Result<String> {
        for provider in &self.providers {
            if let Ok(result) = provider.listen(audio, options).await {
                return Ok(result);
            }
        }
        Err(VoiceError::NoProviderAvailable)
    }
}

/// 语音错误类型
#[derive(Debug, thiserror::Error)]
pub enum VoiceError {
    #[error("语音合成失败: {0}")]
    SpeechSynthesisFailed(String),

    #[error("语音识别失败: {0}")]
    SpeechRecognitionFailed(String),

    #[error("没有可用的语音提供商")]
    NoProviderAvailable,

    #[error("音频格式不支持: {0}")]
    UnsupportedAudioFormat(String),
}

pub type Result<T> = std::result::Result<T, VoiceError>;
