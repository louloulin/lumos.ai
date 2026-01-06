//! 视觉处理 trait 定义

use crate::error::Result;
use crate::types::{GenerationOptions, ImageFormat, MultimodalMessage, VisionOptions};
use async_trait::async_trait;

/// 视觉处理能力
#[derive(Debug, Clone)]
pub struct VisionCapabilities {
    /// 支持的图像格式
    pub supported_formats: Vec<ImageFormat>,
    /// 支持图像理解
    pub supports_understanding: bool,
    /// 支持图像生成
    pub supports_generation: bool,
    /// 支持图像编辑
    pub supports_editing: bool,
    /// 最大图像尺寸（像素）
    pub max_image_size: Option<(u32, u32)>,
    /// 最大文件大小（字节）
    pub max_file_size: Option<usize>,
}

/// 视觉处理提供商 trait
///
/// 定义图像理解和图像生成的统一接口
#[async_trait]
pub trait VisionProvider: Send + Sync {
    /// 获取提供商名称
    fn name(&self) -> &str;

    /// 获取提供商能力
    fn capabilities(&self) -> VisionCapabilities;

    /// 图像理解（从文件）
    ///
    /// # 参数
    /// - `image_path`: 图像文件路径
    /// - `prompt`: 提示问题
    /// - `options`: 视觉选项
    ///
    /// # 返回
    /// AI 对图像的描述/回答
    async fn describe_image(
        &self,
        image_path: &str,
        prompt: &str,
        options: Option<VisionOptions>,
    ) -> Result<String>;

    /// 图像理解（从 URL）
    ///
    /// # 参数
    /// - `image_url`: 图像 URL
    /// - `prompt`: 提示问题
    /// - `options`: 视觉选项
    ///
    /// # 返回
    /// AI 对图像的描述/回答
    async fn describe_image_url(
        &self,
        image_url: &str,
        prompt: &str,
        options: Option<VisionOptions>,
    ) -> Result<String>;

    /// 图像理解（从字节数据）
    ///
    /// # 参数
    /// - `image_data`: 图像数据
    /// - `format`: 图像格式
    /// - `prompt`: 提示问题
    /// - `options`: 视觉选项
    ///
    /// # 返回
    /// AI 对图像的描述/回答
    async fn describe_image_bytes(
        &self,
        image_data: &[u8],
        format: ImageFormat,
        prompt: &str,
        options: Option<VisionOptions>,
    ) -> Result<String>;

    /// 多图像理解
    ///
    /// # 参数
    /// - `image_urls`: 图像 URL 列表
    /// - `prompt`: 提示问题
    /// - `options`: 视觉选项
    ///
    /// # 返回
    /// AI 对多张图像的描述/回答
    async fn describe_multiple_images(
        &self,
        image_urls: &[String],
        prompt: &str,
        options: Option<VisionOptions>,
    ) -> Result<String>;

    /// 图像生成
    ///
    /// # 参数
    /// - `prompt`: 生成提示
    /// - `options`: 生成选项
    ///
    /// # 返回
    /// 生成的图像 URL
    async fn generate_image(
        &self,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<String>;

    /// 图像生成（多张）
    ///
    /// # 参数
    /// - `prompt`: 生成提示
    /// - `options`: 生成选项
    ///
    /// # 返回
    /// 生成的图像 URL 列表
    async fn generate_images(
        &self,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<Vec<String>>;

    /// 图像编辑（可选实现）
    ///
    /// # 参数
    /// - `image_path`: 原始图像路径
    /// - `mask_path`: 遮罩图像路径（可选）
    /// - `prompt`: 编辑提示
    /// - `options`: 生成选项
    ///
    /// # 返回
    /// 编辑后的图像 URL
    async fn edit_image(
        &self,
        _image_path: &str,
        _mask_path: Option<&str>,
        _prompt: &str,
        _options: Option<GenerationOptions>,
    ) -> Result<String> {
        Err(crate::error::MultimodalError::Other(
            "图像编辑未实现".to_string(),
        ))
    }

    /// 图像变体生成（可选实现）
    ///
    /// # 参数
    /// - `image_path`: 原始图像路径
    /// - `options`: 生成选项
    ///
    /// # 返回
    /// 变体图像 URL 列表
    async fn create_variation(
        &self,
        _image_path: &str,
        _options: Option<GenerationOptions>,
    ) -> Result<Vec<String>> {
        Err(crate::error::MultimodalError::Other(
            "图像变体生成未实现".to_string(),
        ))
    }

    /// 多模态对话（可选实现）
    ///
    /// # 参数
    /// - `messages`: 多模态消息列表
    /// - `options`: 视觉选项
    ///
    /// # 返回
    /// AI 回复
    async fn chat(
        &self,
        _messages: &[MultimodalMessage],
        _options: Option<VisionOptions>,
    ) -> Result<String> {
        Err(crate::error::MultimodalError::Other(
            "多模态对话未实现".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vision_capabilities() {
        let caps = VisionCapabilities {
            supported_formats: vec![ImageFormat::Png, ImageFormat::Jpeg],
            supports_understanding: true,
            supports_generation: true,
            supports_editing: false,
            max_image_size: Some((4096, 4096)),
            max_file_size: Some(20 * 1024 * 1024),
        };

        assert_eq!(caps.supported_formats.len(), 2);
        assert!(caps.supports_understanding);
        assert!(caps.supports_generation);
    }
}
