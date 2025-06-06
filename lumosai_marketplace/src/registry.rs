//! 工具注册表实现

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{ToolPackage, ToolCategory};
use crate::storage::{Storage, SqliteStorage};
use crate::search::{SearchEngine, TantivySearchEngine, SearchQuery};
use crate::error::{MarketplaceError, Result};

/// 工具注册表
pub struct ToolRegistry {
    storage: Arc<dyn Storage>,
    search_engine: Arc<dyn SearchEngine>,
}

/// 工具元数据（简化版本）
pub type ToolMetadata = crate::models::ToolPackage;

impl ToolRegistry {
    /// 创建新的工具注册表
    pub async fn new(
        storage: Arc<dyn Storage>,
        search_engine: Arc<dyn SearchEngine>,
    ) -> Result<Self> {
        let registry = Self {
            storage,
            search_engine,
        };
        
        // 初始化存储和搜索引擎
        registry.storage.initialize().await?;
        registry.search_engine.initialize().await?;
        
        Ok(registry)
    }
    
    /// 注册工具包
    pub async fn register_package(&self, package: ToolPackage) -> Result<()> {
        // 检查是否已存在
        if let Some(_) = self.storage.get_package_by_name_version(&package.name, &package.version.to_string()).await? {
            return Err(MarketplaceError::ToolAlreadyExists(format!("{}:{}", package.name, package.version)));
        }
        
        // 保存到存储
        self.storage.save_package(&package).await?;
        
        // 添加到搜索索引
        self.search_engine.index_package(&package).await?;
        
        Ok(())
    }
    
    /// 获取工具包
    pub async fn get_package(&self, id: Uuid) -> Result<Option<ToolPackage>> {
        self.storage.get_package(id).await
    }
    
    /// 根据名称和版本获取工具包
    pub async fn get_package_by_name_version(&self, name: &str, version: &str) -> Result<Option<ToolPackage>> {
        self.storage.get_package_by_name_version(name, version).await
    }
    
    /// 更新工具包
    pub async fn update_package(&self, package: ToolPackage) -> Result<()> {
        // 更新存储
        self.storage.update_package(&package).await?;
        
        // 更新搜索索引
        self.search_engine.update_package_index(&package).await?;
        
        Ok(())
    }
    
    /// 删除工具包
    pub async fn delete_package(&self, id: Uuid) -> Result<()> {
        // 从存储删除
        self.storage.delete_package(id).await?;
        
        // 从搜索索引删除
        self.search_engine.delete_package_index(id).await?;
        
        Ok(())
    }
    
    /// 搜索工具包
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<ToolPackage>> {
        let search_results = self.search_engine.search(query).await?;
        
        let mut packages = Vec::new();
        for result in search_results {
            if let Some(package) = self.storage.get_package(result.package_id).await? {
                packages.push(package);
            }
        }
        
        Ok(packages)
    }
    
    /// 按分类获取工具包
    pub async fn get_packages_by_category(&self, category: &ToolCategory) -> Result<Vec<ToolPackage>> {
        self.storage.search_by_category(category).await
    }
    
    /// 获取热门工具包
    pub async fn get_popular_packages(&self, limit: u32) -> Result<Vec<ToolPackage>> {
        self.storage.get_popular_packages(limit).await
    }
    
    /// 列出所有工具包
    pub async fn list_packages(&self, offset: u32, limit: u32) -> Result<Vec<ToolPackage>> {
        self.storage.list_packages(offset, limit).await
    }
    
    /// 增加下载计数
    pub async fn increment_download_count(&self, id: Uuid) -> Result<()> {
        self.storage.increment_download_count(id).await
    }
    
    /// 更新评分
    pub async fn update_rating(&self, id: Uuid, rating: f64, count: u32) -> Result<()> {
        self.storage.update_rating(id, rating, count).await
    }
    
    /// 获取统计信息
    pub async fn get_statistics(&self) -> Result<RegistryStatistics> {
        let storage_stats = self.storage.get_statistics().await?;
        let index_stats = self.search_engine.get_index_stats().await?;
        
        Ok(RegistryStatistics {
            total_packages: storage_stats.total_packages,
            published_packages: storage_stats.published_packages,
            total_downloads: storage_stats.total_downloads,
            average_rating: storage_stats.average_rating,
            category_counts: storage_stats.category_counts,
            indexed_documents: index_stats.document_count,
            index_size_bytes: index_stats.index_size_bytes,
        })
    }
}

/// 注册表统计信息
#[derive(Debug, Clone)]
pub struct RegistryStatistics {
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
    use tempfile::{TempDir, NamedTempFile};
    use std::collections::HashMap;
    
    async fn create_test_registry() -> ToolRegistry {
        let temp_file = NamedTempFile::new().unwrap();
        let database_url = format!("sqlite://{}", temp_file.path().display());
        let storage = Arc::new(SqliteStorage::new(&database_url).await.unwrap());
        
        let temp_dir = TempDir::new().unwrap();
        let search_engine = Arc::new(TantivySearchEngine::new(temp_dir.path()).await.unwrap());
        
        ToolRegistry::new(storage, search_engine).await.unwrap()
    }
    
    #[tokio::test]
    async fn test_registry_creation() {
        let registry = create_test_registry().await;
        let stats = registry.get_statistics().await.unwrap();
        assert_eq!(stats.total_packages, 0);
    }
    
    #[tokio::test]
    async fn test_package_registration() {
        let registry = create_test_registry().await;
        let package = create_test_package();
        
        // 注册工具包
        registry.register_package(package.clone()).await.unwrap();
        
        // 验证注册成功
        let retrieved = registry.get_package(package.id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, package.name);
        
        // 验证统计信息更新
        let stats = registry.get_statistics().await.unwrap();
        assert_eq!(stats.total_packages, 1);
    }
    
    #[tokio::test]
    async fn test_package_search() {
        let registry = create_test_registry().await;
        let package = create_test_package();
        
        registry.register_package(package.clone()).await.unwrap();
        
        // 搜索工具包
        let query = SearchQuery {
            text: "test".to_string(),
            ..Default::default()
        };
        
        let results = registry.search(&query).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].id, package.id);
    }
    
    #[tokio::test]
    async fn test_duplicate_registration() {
        let registry = create_test_registry().await;
        let package = create_test_package();
        
        // 第一次注册应该成功
        registry.register_package(package.clone()).await.unwrap();
        
        // 第二次注册应该失败
        let result = registry.register_package(package).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MarketplaceError::ToolAlreadyExists(_)));
    }
    
    fn create_test_package() -> ToolPackage {
        use chrono::Utc;
        use semver::Version;
        
        ToolPackage {
            id: Uuid::new_v4(),
            name: "test_tool".to_string(),
            version: Version::new(1, 0, 0),
            description: "Test tool description".to_string(),
            author: "Test Author".to_string(),
            author_email: Some("test@example.com".to_string()),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec!["test".to_string()],
            categories: vec![ToolCategory::Utility],
            dependencies: HashMap::new(),
            lumos_version: "0.1.0".to_string(),
            manifest: crate::models::ToolManifest {
                tools: vec![],
                entry_point: "main.rs".to_string(),
                exports: vec![],
                permissions: vec![],
                config_schema: None,
                rust_version: None,
                build_script: None,
            },
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            published_at: None,
            download_count: 0,
            rating: 0.0,
            rating_count: 0,
            published: false,
            verified: false,
            security_audit: None,
            performance_benchmark: None,
        }
    }
}
