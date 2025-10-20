//! OpenAI 语音处理提供商实现

use async_trait::async_trait;
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

use crate::error::{MultimodalError, Result};
use crate::types::{AudioFormat, SynthesisOptions, TranscriptionOptions};
use crate::voice::{VoiceCapabilities, VoiceProvider};

/// OpenAI 语音处理提供商
pub struct OpenAIVoice {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

/// OpenAI Whisper API 响应
#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
}

impl OpenAIVoice {
    /// 创建新的 OpenAI 语音提供商
    ///
    /// # 参数
    /// - `api_key`: OpenAI API 密钥
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.openai.com/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// 设置自定义 API 基础 URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

#[async_trait]
impl VoiceProvider for OpenAIVoice {
    fn name(&self) -> &str {
        "OpenAI"
    }

    fn capabilities(&self) -> VoiceCapabilities {
        VoiceCapabilities {
            supported_languages: vec![
                "zh".to_string(),
                "en".to_string(),
                "ja".to_string(),
                "ko".to_string(),
                "fr".to_string(),
                "de".to_string(),
                "es".to_string(),
                "it".to_string(),
                "pt".to_string(),
                "ru".to_string(),
            ],
            supported_formats: vec![
                AudioFormat::Mp3,
                AudioFormat::Wav,
                AudioFormat::Flac,
                AudioFormat::M4a,
                AudioFormat::WebM,
            ],
            supported_voices: vec![
                "alloy".to_string(),
                "echo".to_string(),
                "fable".to_string(),
                "onyx".to_string(),
                "nova".to_string(),
                "shimmer".to_string(),
            ],
            max_audio_duration: Some(600),         // 10 分钟
            max_file_size: Some(25 * 1024 * 1024), // 25 MB
        }
    }

    async fn transcribe_file(
        &self,
        file_path: &str,
        options: Option<TranscriptionOptions>,
    ) -> Result<String> {
        // 读取文件
        let file_data = fs::read(file_path).await?;

        // 推断格式
        let ext = Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| MultimodalError::InvalidParameter("无法推断文件格式".to_string()))?;

        let format = AudioFormat::from_extension(ext)
            .ok_or_else(|| MultimodalError::UnsupportedFormat(ext.to_string()))?;

        self.transcribe_bytes(&file_data, format, options).await
    }

    async fn transcribe_bytes(
        &self,
        audio_data: &[u8],
        format: AudioFormat,
        options: Option<TranscriptionOptions>,
    ) -> Result<String> {
        let opts = options.unwrap_or_default();

        // 构建 multipart form
        let file_part = Part::bytes(audio_data.to_vec())
            .file_name(format!(
                "audio.{}",
                match format {
                    AudioFormat::Mp3 => "mp3",
                    AudioFormat::Wav => "wav",
                    AudioFormat::Flac => "flac",
                    AudioFormat::M4a => "m4a",
                    AudioFormat::WebM => "webm",
                }
            ))
            .mime_str(format.mime_type())?;

        let mut form = Form::new()
            .part("file", file_part)
            .text("model", "whisper-1");

        if let Some(lang) = opts.language {
            form = form.text("language", lang);
        }

        if let Some(prompt) = opts.prompt {
            form = form.text("prompt", prompt);
        }

        if let Some(response_format) = opts.response_format {
            form = form.text("response_format", response_format);
        }

        if let Some(temperature) = opts.temperature {
            form = form.text("temperature", temperature.to_string());
        }

        // 发送请求
        let url = format!("{}/audio/transcriptions", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(MultimodalError::ApiError(format!(
                "Whisper API 调用失败: {error_text}"
            )));
        }

        let result: WhisperResponse = response.json().await?;
        Ok(result.text)
    }

    async fn synthesize(&self, text: &str, options: Option<SynthesisOptions>) -> Result<Vec<u8>> {
        let opts = options.unwrap_or_default();

        #[derive(Serialize)]
        struct TtsRequest {
            model: String,
            input: String,
            voice: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            speed: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            response_format: Option<String>,
        }

        let request = TtsRequest {
            model: opts.model.unwrap_or_else(|| "tts-1".to_string()),
            input: text.to_string(),
            voice: opts.voice,
            speed: opts.speed,
            response_format: opts.response_format,
        };

        // 发送请求
        let url = format!("{}/audio/speech", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(MultimodalError::ApiError(format!(
                "TTS API 调用失败: {error_text}"
            )));
        }

        let audio_data = response.bytes().await?.to_vec();
        Ok(audio_data)
    }

    async fn synthesize_to_file(
        &self,
        text: &str,
        output_path: &str,
        options: Option<SynthesisOptions>,
    ) -> Result<()> {
        let audio_data = self.synthesize(text, options).await?;
        fs::write(output_path, audio_data).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_voice_creation() {
        let voice = OpenAIVoice::new("test-key");
        assert_eq!(voice.name(), "OpenAI");

        let caps = voice.capabilities();
        assert!(caps.supported_languages.contains(&"zh".to_string()));
        assert!(caps.supported_languages.contains(&"en".to_string()));
    }

    #[test]
    fn test_custom_base_url() {
        let voice = OpenAIVoice::new("test-key").with_base_url("https://custom.api.com/v1");
        assert_eq!(voice.base_url, "https://custom.api.com/v1");
    }
}
