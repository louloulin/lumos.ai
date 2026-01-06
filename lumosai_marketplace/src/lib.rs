//! Lumos.ai工具市场建设模块
//!
//! 提供完整的工具生态系统，包括工具发布、发现、评估、安全扫描等功能。

pub mod analytics;
pub mod api;
pub mod config;
pub mod discovery;
pub mod error;
pub mod marketplace;
pub mod models;
pub mod publisher;
pub mod registry;
pub mod search;
pub mod security;
pub mod storage;
pub mod validator;

// Re-export main types
pub use analytics::{AnalyticsReport, UsageAnalytics, UsageStatistics};
pub use config::MarketplaceConfig;
pub use discovery::{SearchQuery, SearchResult, ToolDiscoveryEngine};
pub use error::{MarketplaceError, Result};
pub use marketplace::ToolMarketplace;
pub use models::*;
pub use publisher::{PublishRequest, PublishResult, ToolPublisher};
pub use registry::{ToolMetadata, ToolPackage, ToolRegistry};
pub use security::{SecurityAuditResult, SecurityLevel, SecurityScanner};
pub use validator::{ToolValidator, ValidationResult, ValidationRule};

/// 工具市场快速设置
///
/// # Example
///
/// ```rust
/// use lumosai_marketplace::quick_setup;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let marketplace = quick_setup().await?;
///
///     // 搜索工具
///     let results = marketplace.search("web scraping").await?;
///     println!("找到 {} 个工具", results.len());
///
///     Ok(())
/// }
/// ```
pub async fn quick_setup() -> Result<ToolMarketplace> {
    let config = MarketplaceConfig::default();
    ToolMarketplace::new(config).await
}

/// 工具市场构建器
///
/// # Example
///
/// ```rust
/// use lumosai_marketplace::MarketplaceBuilder;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let marketplace = MarketplaceBuilder::new()
///         .database_url("sqlite://marketplace.db")
///         .redis_url("redis://localhost:6379")
///         .enable_security_scanning(true)
///         .enable_analytics(true)
///         .build()
///         .await?;
///
///     Ok(())
/// }
/// ```
pub struct MarketplaceBuilder {
    config: MarketplaceConfig,
}

impl MarketplaceBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: MarketplaceConfig::default(),
        }
    }

    /// 设置数据库URL
    pub fn database_url(mut self, url: impl Into<String>) -> Self {
        self.config.database_url = url.into();
        self
    }

    /// 设置Redis URL
    pub fn redis_url(mut self, url: impl Into<String>) -> Self {
        self.config.redis_url = Some(url.into());
        self
    }

    /// 启用安全扫描
    pub fn enable_security_scanning(mut self, enabled: bool) -> Self {
        self.config.security_scanning_enabled = enabled;
        self
    }

    /// 启用分析功能
    pub fn enable_analytics(mut self, enabled: bool) -> Self {
        self.config.analytics_enabled = enabled;
        self
    }

    /// 设置搜索索引路径
    pub fn search_index_path(mut self, path: impl Into<String>) -> Self {
        self.config.search_index_path = path.into();
        self
    }

    /// 构建工具市场
    pub async fn build(self) -> Result<ToolMarketplace> {
        ToolMarketplace::new(self.config).await
    }
}

impl Default for MarketplaceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quick_setup() {
        let result = quick_setup().await;
        assert!(result.is_ok(), "快速设置应该成功");
    }

    #[tokio::test]
    async fn test_marketplace_builder() {
        let result = MarketplaceBuilder::new()
            .database_url("sqlite://:memory:")
            .enable_security_scanning(false)
            .enable_analytics(false)
            .build()
            .await;

        assert!(result.is_ok(), "构建器应该成功创建市场");
    }
}
