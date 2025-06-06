//! 工具市场错误类型定义

use thiserror::Error;

/// 工具市场错误类型
#[derive(Error, Debug)]
pub enum MarketplaceError {
    /// 数据库错误
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
    
    /// Redis错误
    #[error("Redis错误: {0}")]
    Redis(#[from] redis::RedisError),
    
    /// 搜索引擎错误
    #[error("搜索引擎错误: {0}")]
    Search(#[from] tantivy::TantivyError),
    
    /// HTTP错误
    #[error("HTTP错误: {0}")]
    Http(#[from] reqwest::Error),
    
    /// JSON序列化错误
    #[error("JSON序列化错误: {0}")]
    Json(#[from] serde_json::Error),
    
    /// IO错误
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    
    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),
    
    /// 验证错误
    #[error("验证错误: {0}")]
    Validation(String),
    
    /// 工具不存在
    #[error("工具不存在: {0}")]
    ToolNotFound(String),
    
    /// 工具已存在
    #[error("工具已存在: {0}")]
    ToolAlreadyExists(String),
    
    /// 权限错误
    #[error("权限错误: {0}")]
    Permission(String),
    
    /// 安全扫描错误
    #[error("安全扫描错误: {0}")]
    SecurityScan(String),
    
    /// 发布错误
    #[error("发布错误: {0}")]
    Publish(String),
    
    /// 下载错误
    #[error("下载错误: {0}")]
    Download(String),
    
    /// 解压错误
    #[error("解压错误: {0}")]
    Extract(String),
    
    /// 版本错误
    #[error("版本错误: {0}")]
    Version(String),
    
    /// 依赖错误
    #[error("依赖错误: {0}")]
    Dependency(String),
    
    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
    
    /// 网络错误
    #[error("网络错误: {0}")]
    Network(String),
    
    /// 超时错误
    #[error("操作超时")]
    Timeout,
    
    /// 不支持的操作
    #[error("不支持的操作: {0}")]
    Unsupported(String),
}

/// 工具市场结果类型
pub type Result<T> = std::result::Result<T, MarketplaceError>;

impl From<&str> for MarketplaceError {
    fn from(msg: &str) -> Self {
        MarketplaceError::Internal(msg.to_string())
    }
}

impl From<String> for MarketplaceError {
    fn from(msg: String) -> Self {
        MarketplaceError::Internal(msg)
    }
}

/// 错误转换辅助函数
impl MarketplaceError {
    /// 创建配置错误
    pub fn config(msg: impl Into<String>) -> Self {
        MarketplaceError::Config(msg.into())
    }
    
    /// 创建验证错误
    pub fn validation(msg: impl Into<String>) -> Self {
        MarketplaceError::Validation(msg.into())
    }
    
    /// 创建权限错误
    pub fn permission(msg: impl Into<String>) -> Self {
        MarketplaceError::Permission(msg.into())
    }
    
    /// 创建安全扫描错误
    pub fn security_scan(msg: impl Into<String>) -> Self {
        MarketplaceError::SecurityScan(msg.into())
    }
    
    /// 创建发布错误
    pub fn publish(msg: impl Into<String>) -> Self {
        MarketplaceError::Publish(msg.into())
    }
    
    /// 创建下载错误
    pub fn download(msg: impl Into<String>) -> Self {
        MarketplaceError::Download(msg.into())
    }
    
    /// 创建版本错误
    pub fn version(msg: impl Into<String>) -> Self {
        MarketplaceError::Version(msg.into())
    }
    
    /// 创建依赖错误
    pub fn dependency(msg: impl Into<String>) -> Self {
        MarketplaceError::Dependency(msg.into())
    }
    
    /// 创建网络错误
    pub fn network(msg: impl Into<String>) -> Self {
        MarketplaceError::Network(msg.into())
    }
    
    /// 检查是否为临时错误（可重试）
    pub fn is_temporary(&self) -> bool {
        matches!(
            self,
            MarketplaceError::Network(_) |
            MarketplaceError::Timeout |
            MarketplaceError::Http(_) |
            MarketplaceError::Redis(_)
        )
    }
    
    /// 检查是否为致命错误（不可重试）
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            MarketplaceError::Config(_) |
            MarketplaceError::Permission(_) |
            MarketplaceError::ToolNotFound(_) |
            MarketplaceError::ToolAlreadyExists(_) |
            MarketplaceError::Validation(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = MarketplaceError::config("测试配置错误");
        assert!(matches!(err, MarketplaceError::Config(_)));
        assert_eq!(err.to_string(), "配置错误: 测试配置错误");
    }
    
    #[test]
    fn test_error_classification() {
        let network_err = MarketplaceError::network("连接失败");
        assert!(network_err.is_temporary());
        assert!(!network_err.is_fatal());
        
        let config_err = MarketplaceError::config("配置无效");
        assert!(!config_err.is_temporary());
        assert!(config_err.is_fatal());
    }
    
    #[test]
    fn test_error_conversion() {
        let err: MarketplaceError = "测试错误".into();
        assert!(matches!(err, MarketplaceError::Internal(_)));
        
        let err: MarketplaceError = "测试错误".to_string().into();
        assert!(matches!(err, MarketplaceError::Internal(_)));
    }
}
