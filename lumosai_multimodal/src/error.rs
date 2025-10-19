//! 多模态错误类型定义

use thiserror::Error;

/// 多模态处理错误类型
#[derive(Error, Debug)]
pub enum MultimodalError {
    /// API 调用错误
    #[error("API 调用失败: {0}")]
    ApiError(String),

    /// 音频处理错误
    #[error("音频处理失败: {0}")]
    AudioError(String),

    /// 图像处理错误
    #[error("图像处理失败: {0}")]
    ImageError(String),

    /// 文件 I/O 错误
    #[error("文件操作失败: {0}")]
    IoError(#[from] std::io::Error),

    /// HTTP 请求错误
    #[error("HTTP 请求失败: {0}")]
    HttpError(#[from] reqwest::Error),

    /// JSON 序列化/反序列化错误
    #[error("JSON 处理失败: {0}")]
    JsonError(#[from] serde_json::Error),

    /// 不支持的格式
    #[error("不支持的格式: {0}")]
    UnsupportedFormat(String),

    /// 无效的参数
    #[error("无效的参数: {0}")]
    InvalidParameter(String),

    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// 超时错误
    #[error("操作超时")]
    Timeout,

    /// 其他错误
    #[error("其他错误: {0}")]
    Other(String),
}

/// 多模态处理结果类型
pub type Result<T> = std::result::Result<T, MultimodalError>;

impl From<image::ImageError> for MultimodalError {
    fn from(err: image::ImageError) -> Self {
        MultimodalError::ImageError(err.to_string())
    }
}

impl From<hound::Error> for MultimodalError {
    fn from(err: hound::Error) -> Self {
        MultimodalError::AudioError(err.to_string())
    }
}
