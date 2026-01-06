//! 多模态类型定义

use serde::{Deserialize, Serialize};

/// 音频格式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioFormat {
    /// MP3 格式
    Mp3,
    /// WAV 格式
    Wav,
    /// FLAC 格式
    Flac,
    /// M4A 格式
    M4a,
    /// WebM 格式
    WebM,
}

impl AudioFormat {
    /// 从文件扩展名推断格式
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp3" => Some(AudioFormat::Mp3),
            "wav" => Some(AudioFormat::Wav),
            "flac" => Some(AudioFormat::Flac),
            "m4a" => Some(AudioFormat::M4a),
            "webm" => Some(AudioFormat::WebM),
            _ => None,
        }
    }

    /// 获取 MIME 类型
    pub fn mime_type(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::Wav => "audio/wav",
            AudioFormat::Flac => "audio/flac",
            AudioFormat::M4a => "audio/m4a",
            AudioFormat::WebM => "audio/webm",
        }
    }
}

/// 图像格式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImageFormat {
    /// PNG 格式
    Png,
    /// JPEG 格式
    Jpeg,
    /// WebP 格式
    WebP,
    /// GIF 格式
    Gif,
}

impl ImageFormat {
    /// 从文件扩展名推断格式
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(ImageFormat::Png),
            "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
            "webp" => Some(ImageFormat::WebP),
            "gif" => Some(ImageFormat::Gif),
            _ => None,
        }
    }

    /// 获取 MIME 类型
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::WebP => "image/webp",
            ImageFormat::Gif => "image/gif",
        }
    }
}

/// 图像尺寸
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImageSize {
    /// 256x256
    Small,
    /// 512x512
    Medium,
    /// 1024x1024
    Large,
    /// 1792x1024 (横向)
    LandscapeHD,
    /// 1024x1792 (纵向)
    PortraitHD,
    /// 自定义尺寸
    Custom(u32, u32),
}

impl ImageSize {
    /// 获取尺寸字符串（OpenAI 格式）
    pub fn to_string(&self) -> String {
        match self {
            ImageSize::Small => "256x256".to_string(),
            ImageSize::Medium => "512x512".to_string(),
            ImageSize::Large => "1024x1024".to_string(),
            ImageSize::LandscapeHD => "1792x1024".to_string(),
            ImageSize::PortraitHD => "1024x1792".to_string(),
            ImageSize::Custom(w, h) => format!("{w}x{h}"),
        }
    }
}

/// 语音转文本选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionOptions {
    /// 语言代码（如 "zh", "en"）
    pub language: Option<String>,
    /// 提示文本（用于提高准确性）
    pub prompt: Option<String>,
    /// 响应格式（json, text, srt, vtt）
    pub response_format: Option<String>,
    /// 温度参数（0-1）
    pub temperature: Option<f32>,
}

impl Default for TranscriptionOptions {
    fn default() -> Self {
        Self {
            language: None,
            prompt: None,
            response_format: Some("json".to_string()),
            temperature: Some(0.0),
        }
    }
}

/// 文本转语音选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisOptions {
    /// 语音模型（alloy, echo, fable, onyx, nova, shimmer）
    pub voice: String,
    /// 模型版本（tts-1, tts-1-hd）
    pub model: Option<String>,
    /// 语速（0.25-4.0）
    pub speed: Option<f32>,
    /// 响应格式（mp3, opus, aac, flac）
    pub response_format: Option<String>,
}

impl Default for SynthesisOptions {
    fn default() -> Self {
        Self {
            voice: "alloy".to_string(),
            model: Some("tts-1".to_string()),
            speed: Some(1.0),
            response_format: Some("mp3".to_string()),
        }
    }
}

/// 视觉理解选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionOptions {
    /// 模型名称（gpt-4-vision-preview, gpt-4o）
    pub model: Option<String>,
    /// 最大 token 数
    pub max_tokens: Option<u32>,
    /// 详细程度（low, high, auto）
    pub detail: Option<String>,
}

impl Default for VisionOptions {
    fn default() -> Self {
        Self {
            model: Some("gpt-4o".to_string()),
            max_tokens: Some(300),
            detail: Some("auto".to_string()),
        }
    }
}

/// 图像生成选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOptions {
    /// 模型名称（dall-e-2, dall-e-3）
    pub model: Option<String>,
    /// 图像尺寸
    pub size: ImageSize,
    /// 图像质量（standard, hd）
    pub quality: Option<String>,
    /// 生成数量（1-10，仅 dall-e-2 支持多张）
    pub n: Option<u32>,
    /// 风格（vivid, natural）
    pub style: Option<String>,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            model: Some("dall-e-3".to_string()),
            size: ImageSize::Large,
            quality: Some("standard".to_string()),
            n: Some(1),
            style: Some("vivid".to_string()),
        }
    }
}

/// 多模态内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MultimodalContent {
    /// 文本内容
    Text { text: String },
    /// 图像 URL
    ImageUrl { url: String, detail: Option<String> },
    /// 音频数据
    Audio { data: Vec<u8>, format: AudioFormat },
}

/// 多模态消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalMessage {
    /// 角色（user, assistant, system）
    pub role: String,
    /// 内容列表
    pub content: Vec<MultimodalContent>,
}

impl MultimodalMessage {
    /// 创建文本消息
    pub fn text(role: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: vec![MultimodalContent::Text { text: text.into() }],
        }
    }

    /// 创建图像消息
    pub fn image(role: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: vec![MultimodalContent::ImageUrl {
                url: url.into(),
                detail: Some("auto".to_string()),
            }],
        }
    }

    /// 创建混合消息（文本 + 图像）
    pub fn text_and_image(
        role: impl Into<String>,
        text: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            role: role.into(),
            content: vec![
                MultimodalContent::Text { text: text.into() },
                MultimodalContent::ImageUrl {
                    url: url.into(),
                    detail: Some("auto".to_string()),
                },
            ],
        }
    }
}
