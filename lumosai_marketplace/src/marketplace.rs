//! 工具市场主模块实现

use std::sync::Arc;
use uuid::Uuid;

use crate::config::MarketplaceConfig;
use crate::storage::{Storage, SqliteStorage};
use crate::search::{SearchEngine, TantivySearchEngine, SearchQuery};
use crate::registry::{ToolRegistry, RegistryStatistics};
use crate::validator::{ToolValidator, DefaultToolValidator, ValidationResult};
use crate::publisher::{ToolPublisher, DefaultToolPublisher, PublishRequest, PublishResult};
use crate::discovery::{ToolDiscoveryEngine, DefaultToolDiscoveryEngine, SearchResult, UserContext};
use crate::analytics::{UsageAnalytics, DefaultUsageAnalytics, AnalyticsReport, TimeRange, UserInfo, UsageInfo};
use crate::security::{SecurityScanner, DefaultSecurityScanner};
use crate::models::{ToolPackage, ToolCategory};
use crate::error::{MarketplaceError, Result};

/// 工具市场主类
pub struct ToolMarketplace {
    config: MarketplaceConfig,
    registry: Arc<ToolRegistry>,
    validator: Arc<dyn ToolValidator>,
    publisher: Arc<dyn ToolPublisher>,
    discovery: Arc<dyn ToolDiscoveryEngine>,
    analytics: Arc<dyn UsageAnalytics>,
    security_scanner: Arc<dyn SecurityScanner>,
}

impl ToolMarketplace {
    /// 创建新的工具市场
    pub async fn new(config: MarketplaceConfig) -> Result<Self> {
        // 验证配置
        config.validate()?;
        
        // 创建存储层
        let storage: Arc<dyn Storage> = Arc::new(SqliteStorage::new(&config.database_url).await?);
        
        // 创建搜索引擎
        let search_engine: Arc<dyn SearchEngine> = Arc::new(
            TantivySearchEngine::new(&config.search_index_path).await?
        );
        
        // 创建工具注册表
        let registry = Arc::new(ToolRegistry::new(storage.clone(), search_engine.clone()).await?);
        
        // 创建验证器
        let validator: Arc<dyn ToolValidator> = Arc::new(DefaultToolValidator::new()?);
        
        // 创建安全扫描器
        let security_scanner: Arc<dyn SecurityScanner> = Arc::new(DefaultSecurityScanner::new());
        
        // 创建发布器
        let publisher: Arc<dyn ToolPublisher> = Arc::new(DefaultToolPublisher::new(
            Box::new(DefaultToolValidator::new()?),
            Box::new(DefaultSecurityScanner::new()),
            config.tool_storage_path.clone(),
        ));
        
        // 创建发现引擎
        let discovery: Arc<dyn ToolDiscoveryEngine> = Arc::new(
            DefaultToolDiscoveryEngine::new(storage.clone(), search_engine.clone())
        );
        
        // 创建分析引擎
        let analytics: Arc<dyn UsageAnalytics> = Arc::new(DefaultUsageAnalytics::new());
        
        Ok(Self {
            config,
            registry,
            validator,
            publisher,
            discovery,
            analytics,
            security_scanner,
        })
    }
    
    /// 获取配置
    pub fn config(&self) -> &MarketplaceConfig {
        &self.config
    }
    
    /// 获取工具注册表
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }
    
    /// 搜索工具
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            text: query.to_string(),
            published_only: true,
            ..Default::default()
        };
        
        self.discovery.search(&search_query).await
    }
    
    /// 高级搜索
    pub async fn advanced_search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        self.discovery.search(query).await
    }
    
    /// 获取工具包
    pub async fn get_package(&self, id: Uuid) -> Result<Option<ToolPackage>> {
        let package = self.registry.get_package(id).await?;
        
        // 记录访问事件
        if let Some(ref pkg) = package {
            let user_info = UserInfo {
                user_id: None,
                user_type: crate::analytics::UserType::Individual,
                location: None,
                ip_address: None,
                user_agent: None,
            };
            
            // 这里可以记录查看事件，但不是下载事件
            // self.analytics.record_view(pkg.id, &user_info).await?;
        }
        
        Ok(package)
    }
    
    /// 根据名称和版本获取工具包
    pub async fn get_package_by_name_version(&self, name: &str, version: &str) -> Result<Option<ToolPackage>> {
        self.registry.get_package_by_name_version(name, version).await
    }
    
    /// 发布工具包
    pub async fn publish_package(&self, request: PublishRequest) -> Result<PublishResult> {
        self.publisher.publish(request).await
    }
    
    /// 验证工具包
    pub async fn validate_package(&self, package: &ToolPackage) -> Result<ValidationResult> {
        self.validator.validate(package).await
    }
    
    /// 下载工具包
    pub async fn download_package(&self, id: Uuid, user_info: &UserInfo) -> Result<String> {
        // 检查工具包是否存在
        let package = self.registry.get_package(id).await?
            .ok_or_else(|| MarketplaceError::ToolNotFound(id.to_string()))?;
        
        if !package.published {
            return Err(MarketplaceError::permission("工具包未发布"));
        }
        
        // 增加下载计数
        self.registry.increment_download_count(id).await?;
        
        // 记录下载事件
        self.analytics.record_download(id, user_info).await?;
        
        // 返回下载URL或路径
        Ok(format!("{}/{}.tar.gz", self.config.tool_storage_path, id))
    }
    
    /// 评分工具包
    pub async fn rate_package(&self, id: Uuid, rating: f64, user_info: &UserInfo) -> Result<()> {
        if rating < 1.0 || rating > 5.0 {
            return Err(MarketplaceError::validation("评分必须在1.0-5.0之间"));
        }
        
        // 检查工具包是否存在
        let package = self.registry.get_package(id).await?
            .ok_or_else(|| MarketplaceError::ToolNotFound(id.to_string()))?;
        
        if !package.published {
            return Err(MarketplaceError::permission("只能对已发布的工具包评分"));
        }
        
        // 记录评分事件
        self.analytics.record_rating(id, rating, user_info).await?;
        
        // 更新平均评分（简化实现）
        let new_count = package.rating_count + 1;
        let new_rating = (package.rating * package.rating_count as f64 + rating) / new_count as f64;
        
        self.registry.update_rating(id, new_rating, new_count).await?;
        
        Ok(())
    }
    
    /// 获取推荐工具
    pub async fn get_recommendations(&self, user_context: &UserContext) -> Result<Vec<SearchResult>> {
        self.discovery.recommend(user_context).await
    }
    
    /// 获取热门工具
    pub async fn get_trending(&self, category: Option<ToolCategory>, limit: usize) -> Result<Vec<SearchResult>> {
        self.discovery.get_trending(category, limit).await
    }
    
    /// 获取相似工具
    pub async fn get_similar(&self, package_id: Uuid, limit: usize) -> Result<Vec<SearchResult>> {
        self.discovery.get_similar(package_id, limit).await
    }
    
    /// 获取最新工具
    pub async fn get_recent(&self, limit: usize) -> Result<Vec<SearchResult>> {
        self.discovery.get_recent(limit).await
    }
    
    /// 按分类获取工具
    pub async fn get_by_category(&self, category: &ToolCategory) -> Result<Vec<ToolPackage>> {
        self.registry.get_packages_by_category(category).await
    }
    
    /// 记录工具使用
    pub async fn record_usage(&self, package_id: Uuid, usage_info: &UsageInfo) -> Result<()> {
        self.analytics.record_usage(package_id, usage_info).await
    }
    
    /// 生成分析报告
    pub async fn generate_analytics_report(&self, time_range: TimeRange) -> Result<AnalyticsReport> {
        self.analytics.generate_report(time_range).await
    }
    
    /// 获取工具统计信息
    pub async fn get_tool_statistics(&self, package_id: Uuid) -> Result<crate::analytics::UsageStatistics> {
        self.analytics.get_tool_statistics(package_id).await
    }
    
    /// 获取市场统计信息
    pub async fn get_marketplace_statistics(&self) -> Result<MarketplaceStatistics> {
        let registry_stats = self.registry.get_statistics().await?;
        
        Ok(MarketplaceStatistics {
            total_packages: registry_stats.total_packages,
            published_packages: registry_stats.published_packages,
            total_downloads: registry_stats.total_downloads,
            average_rating: registry_stats.average_rating,
            category_counts: registry_stats.category_counts,
            indexed_documents: registry_stats.indexed_documents,
            index_size_bytes: registry_stats.index_size_bytes,
        })
    }
    
    /// 扫描工具包安全性
    pub async fn scan_package_security(&self, package: &ToolPackage) -> Result<crate::models::SecurityAuditResult> {
        self.security_scanner.scan_package(package).await
    }
    
    /// 列出所有工具包
    pub async fn list_packages(&self, offset: u32, limit: u32) -> Result<Vec<ToolPackage>> {
        self.registry.list_packages(offset, limit).await
    }
    
    /// 删除工具包
    pub async fn delete_package(&self, id: Uuid) -> Result<()> {
        self.registry.delete_package(id).await
    }
    
    /// 更新工具包
    pub async fn update_package(&self, package: ToolPackage) -> Result<()> {
        self.registry.update_package(package).await
    }
}

/// 市场统计信息
#[derive(Debug, Clone)]
pub struct MarketplaceStatistics {
    /// 总工具包数
    pub total_packages: u64,
    /// 已发布工具包数
    pub published_packages: u64,
    /// 总下载次数
    pub total_downloads: u64,
    /// 平均评分
    pub average_rating: f64,
    /// 按分类统计
    pub category_counts: std::collections::HashMap<ToolCategory, u64>,
    /// 索引文档数
    pub indexed_documents: u64,
    /// 索引大小
    pub index_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MarketplaceConfig;
    use tempfile::TempDir;
    
    async fn create_test_marketplace() -> ToolMarketplace {
        let temp_dir = TempDir::new().unwrap();
        let config = MarketplaceConfig {
            database_url: "sqlite://:memory:".to_string(),
            search_index_path: temp_dir.path().join("search").to_string_lossy().to_string(),
            tool_storage_path: temp_dir.path().join("storage").to_string_lossy().to_string(),
            security_scanning_enabled: false, // 简化测试
            analytics_enabled: true,
            ..Default::default()
        };
        
        ToolMarketplace::new(config).await.unwrap()
    }
    
    #[tokio::test]
    async fn test_marketplace_creation() {
        let marketplace = create_test_marketplace().await;
        let stats = marketplace.get_marketplace_statistics().await.unwrap();
        assert_eq!(stats.total_packages, 0);
    }
    
    #[tokio::test]
    async fn test_search_empty_marketplace() {
        let marketplace = create_test_marketplace().await;
        let results = marketplace.search("test").await.unwrap();
        assert!(results.is_empty());
    }
    
    #[tokio::test]
    async fn test_get_nonexistent_package() {
        let marketplace = create_test_marketplace().await;
        let package = marketplace.get_package(Uuid::new_v4()).await.unwrap();
        assert!(package.is_none());
    }
    
    #[tokio::test]
    async fn test_invalid_rating() {
        let marketplace = create_test_marketplace().await;
        let user_info = UserInfo {
            user_id: Some("test_user".to_string()),
            user_type: crate::analytics::UserType::Individual,
            location: None,
            ip_address: None,
            user_agent: None,
        };
        
        let result = marketplace.rate_package(Uuid::new_v4(), 6.0, &user_info).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("评分必须在1.0-5.0之间"));
    }
}
