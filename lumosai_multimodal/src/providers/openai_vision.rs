//! OpenAI 视觉处理提供商实现

use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

use crate::error::{MultimodalError, Result};
use crate::types::{GenerationOptions, ImageFormat, VisionOptions};
use crate::vision::{VisionCapabilities, VisionProvider};

/// OpenAI 视觉处理提供商
pub struct OpenAIVision {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

/// GPT-4V 消息内容
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Serialize)]
struct ImageUrl {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: MessageContent,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Debug, Serialize)]
struct ImageGenerationRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImageGenerationResponse {
    data: Vec<ImageData>,
}

#[derive(Debug, Deserialize)]
struct ImageData {
    url: String,
}

impl OpenAIVision {
    /// 创建新的 OpenAI 视觉提供商
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

    /// 将图像文件转换为 base64 data URL
    async fn image_to_data_url(&self, image_path: &str) -> Result<String> {
        let image_data = fs::read(image_path).await?;

        let ext = Path::new(image_path)
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| MultimodalError::InvalidParameter("无法推断图像格式".to_string()))?;

        let format = ImageFormat::from_extension(ext)
            .ok_or_else(|| MultimodalError::UnsupportedFormat(ext.to_string()))?;

        let base64_data = general_purpose::STANDARD.encode(&image_data);
        Ok(format!(
            "data:{};base64,{}",
            format.mime_type(),
            base64_data
        ))
    }
}

#[async_trait]
impl VisionProvider for OpenAIVision {
    fn name(&self) -> &str {
        "OpenAI"
    }

    fn capabilities(&self) -> VisionCapabilities {
        VisionCapabilities {
            supported_formats: vec![
                ImageFormat::Png,
                ImageFormat::Jpeg,
                ImageFormat::WebP,
                ImageFormat::Gif,
            ],
            supports_understanding: true,
            supports_generation: true,
            supports_editing: true,
            max_image_size: Some((4096, 4096)),
            max_file_size: Some(20 * 1024 * 1024), // 20 MB
        }
    }

    async fn describe_image(
        &self,
        image_path: &str,
        prompt: &str,
        options: Option<VisionOptions>,
    ) -> Result<String> {
        let image_url = self.image_to_data_url(image_path).await?;
        self.describe_image_url(&image_url, prompt, options).await
    }

    async fn describe_image_url(
        &self,
        image_url: &str,
        prompt: &str,
        options: Option<VisionOptions>,
    ) -> Result<String> {
        let opts = options.unwrap_or_default();

        let request = ChatRequest {
            model: opts.model.unwrap_or_else(|| "gpt-4o".to_string()),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: MessageContent::Parts(vec![
                    ContentPart::Text {
                        text: prompt.to_string(),
                    },
                    ContentPart::ImageUrl {
                        image_url: ImageUrl {
                            url: image_url.to_string(),
                            detail: opts.detail,
                        },
                    },
                ]),
            }],
            max_tokens: opts.max_tokens,
        };

        let url = format!("{}/chat/completions", self.base_url);
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
                "GPT-4V API 调用失败: {error_text}"
            )));
        }

        let result: ChatResponse = response.json().await?;
        Ok(result.choices[0].message.content.clone())
    }

    async fn describe_image_bytes(
        &self,
        image_data: &[u8],
        format: ImageFormat,
        prompt: &str,
        options: Option<VisionOptions>,
    ) -> Result<String> {
        let base64_data = general_purpose::STANDARD.encode(image_data);
        let data_url = format!("data:{};base64,{}", format.mime_type(), base64_data);
        self.describe_image_url(&data_url, prompt, options).await
    }

    async fn describe_multiple_images(
        &self,
        image_urls: &[String],
        prompt: &str,
        options: Option<VisionOptions>,
    ) -> Result<String> {
        let opts = options.unwrap_or_default();

        let mut content_parts = vec![ContentPart::Text {
            text: prompt.to_string(),
        }];

        for url in image_urls {
            content_parts.push(ContentPart::ImageUrl {
                image_url: ImageUrl {
                    url: url.clone(),
                    detail: opts.detail.clone(),
                },
            });
        }

        let request = ChatRequest {
            model: opts.model.unwrap_or_else(|| "gpt-4o".to_string()),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: MessageContent::Parts(content_parts),
            }],
            max_tokens: opts.max_tokens,
        };

        let url = format!("{}/chat/completions", self.base_url);
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
                "GPT-4V API 调用失败: {error_text}"
            )));
        }

        let result: ChatResponse = response.json().await?;
        Ok(result.choices[0].message.content.clone())
    }

    async fn generate_image(
        &self,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<String> {
        let images = self.generate_images(prompt, options).await?;
        Ok(images.into_iter().next().unwrap_or_default())
    }

    async fn generate_images(
        &self,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<Vec<String>> {
        let opts = options.unwrap_or_default();

        let request = ImageGenerationRequest {
            model: opts.model.unwrap_or_else(|| "dall-e-3".to_string()),
            prompt: prompt.to_string(),
            n: opts.n,
            size: Some(opts.size.to_string()),
            quality: opts.quality,
            style: opts.style,
        };

        let url = format!("{}/images/generations", self.base_url);
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
                "DALL-E API 调用失败: {error_text}"
            )));
        }

        let result: ImageGenerationResponse = response.json().await?;
        Ok(result.data.into_iter().map(|d| d.url).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_vision_creation() {
        let vision = OpenAIVision::new("test-key");
        assert_eq!(vision.name(), "OpenAI");

        let caps = vision.capabilities();
        assert!(caps.supports_understanding);
        assert!(caps.supports_generation);
    }
}
