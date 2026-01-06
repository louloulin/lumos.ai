//! 语音处理 trait 定义

use crate::error::Result;
use crate::types::{AudioFormat, SynthesisOptions, TranscriptionOptions};
use async_trait::async_trait;

/// 语音处理能力
#[derive(Debug, Clone)]
pub struct VoiceCapabilities {
    /// 支持的语音识别语言
    pub supported_languages: Vec<String>,
    /// 支持的音频格式
    pub supported_formats: Vec<AudioFormat>,
    /// 支持的语音模型
    pub supported_voices: Vec<String>,
    /// 最大音频时长（秒）
    pub max_audio_duration: Option<u32>,
    /// 最大文件大小（字节）
    pub max_file_size: Option<usize>,
}

/// 语音处理提供商 trait
///
/// 定义语音识别（STT）和语音合成（TTS）的统一接口
#[async_trait]
pub trait VoiceProvider: Send + Sync {
    /// 获取提供商名称
    fn name(&self) -> &str;

    /// 获取提供商能力
    fn capabilities(&self) -> VoiceCapabilities;

    /// 语音转文本（从文件）
    ///
    /// # 参数
    /// - `file_path`: 音频文件路径
    /// - `options`: 转录选项
    ///
    /// # 返回
    /// 识别出的文本
    async fn transcribe_file(
        &self,
        file_path: &str,
        options: Option<TranscriptionOptions>,
    ) -> Result<String>;

    /// 语音转文本（从字节数据）
    ///
    /// # 参数
    /// - `audio_data`: 音频数据
    /// - `format`: 音频格式
    /// - `options`: 转录选项
    ///
    /// # 返回
    /// 识别出的文本
    async fn transcribe_bytes(
        &self,
        audio_data: &[u8],
        format: AudioFormat,
        options: Option<TranscriptionOptions>,
    ) -> Result<String>;

    /// 文本转语音
    ///
    /// # 参数
    /// - `text`: 要合成的文本
    /// - `options`: 合成选项
    ///
    /// # 返回
    /// 音频数据（字节）
    async fn synthesize(&self, text: &str, options: Option<SynthesisOptions>) -> Result<Vec<u8>>;

    /// 文本转语音（保存到文件）
    ///
    /// # 参数
    /// - `text`: 要合成的文本
    /// - `output_path`: 输出文件路径
    /// - `options`: 合成选项
    async fn synthesize_to_file(
        &self,
        text: &str,
        output_path: &str,
        options: Option<SynthesisOptions>,
    ) -> Result<()>;

    /// 流式语音转文本（可选实现）
    ///
    /// # 参数
    /// - `audio_stream`: 音频流
    /// - `options`: 转录选项
    ///
    /// # 返回
    /// 文本流
    async fn transcribe_stream(
        &self,
        _audio_stream: tokio::sync::mpsc::Receiver<Vec<u8>>,
        _options: Option<TranscriptionOptions>,
    ) -> Result<tokio::sync::mpsc::Receiver<String>> {
        Err(crate::error::MultimodalError::Other(
            "流式转录未实现".to_string(),
        ))
    }

    /// 流式文本转语音（可选实现）
    ///
    /// # 参数
    /// - `text`: 要合成的文本
    /// - `options`: 合成选项
    ///
    /// # 返回
    /// 音频流
    async fn synthesize_stream(
        &self,
        _text: &str,
        _options: Option<SynthesisOptions>,
    ) -> Result<tokio::sync::mpsc::Receiver<Vec<u8>>> {
        Err(crate::error::MultimodalError::Other(
            "流式合成未实现".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_capabilities() {
        let caps = VoiceCapabilities {
            supported_languages: vec!["zh".to_string(), "en".to_string()],
            supported_formats: vec![AudioFormat::Mp3, AudioFormat::Wav],
            supported_voices: vec!["alloy".to_string(), "echo".to_string()],
            max_audio_duration: Some(600),
            max_file_size: Some(25 * 1024 * 1024),
        };

        assert_eq!(caps.supported_languages.len(), 2);
        assert_eq!(caps.supported_formats.len(), 2);
    }
}
