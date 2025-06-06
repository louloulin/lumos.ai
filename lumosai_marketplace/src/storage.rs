//! 工具市场存储层实现

use async_trait::async_trait;
use sqlx::{Pool, Sqlite, SqlitePool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;

use crate::models::*;
use crate::error::{MarketplaceError, Result};

/// 存储层trait
#[async_trait]
pub trait Storage: Send + Sync {
    /// 初始化存储
    async fn initialize(&self) -> Result<()>;
    
    /// 保存工具包
    async fn save_package(&self, package: &ToolPackage) -> Result<()>;
    
    /// 获取工具包
    async fn get_package(&self, id: Uuid) -> Result<Option<ToolPackage>>;
    
    /// 根据名称和版本获取工具包
    async fn get_package_by_name_version(&self, name: &str, version: &str) -> Result<Option<ToolPackage>>;
    
    /// 更新工具包
    async fn update_package(&self, package: &ToolPackage) -> Result<()>;
    
    /// 删除工具包
    async fn delete_package(&self, id: Uuid) -> Result<()>;
    
    /// 列出所有工具包
    async fn list_packages(&self, offset: u32, limit: u32) -> Result<Vec<ToolPackage>>;
    
    /// 按分类搜索工具包
    async fn search_by_category(&self, category: &ToolCategory) -> Result<Vec<ToolPackage>>;
    
    /// 按关键词搜索工具包
    async fn search_by_keywords(&self, keywords: &[String]) -> Result<Vec<ToolPackage>>;
    
    /// 获取热门工具包
    async fn get_popular_packages(&self, limit: u32) -> Result<Vec<ToolPackage>>;
    
    /// 增加下载计数
    async fn increment_download_count(&self, id: Uuid) -> Result<()>;
    
    /// 更新评分
    async fn update_rating(&self, id: Uuid, rating: f64, count: u32) -> Result<()>;
    
    /// 获取统计信息
    async fn get_statistics(&self) -> Result<StorageStatistics>;
}

/// 存储统计信息
#[derive(Debug, Clone)]
pub struct StorageStatistics {
    /// 总工具包数
    pub total_packages: u64,
    /// 已发布工具包数
    pub published_packages: u64,
    /// 总下载次数
    pub total_downloads: u64,
    /// 平均评分
    pub average_rating: f64,
    /// 按分类统计
    pub category_counts: HashMap<ToolCategory, u64>,
}

/// SQLite存储实现
pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    /// 创建新的SQLite存储
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Self { pool })
    }
    
    /// 获取数据库连接池
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl Storage for SqliteStorage {
    async fn initialize(&self) -> Result<()> {
        // 创建工具包表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tool_packages (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                description TEXT NOT NULL,
                author TEXT NOT NULL,
                author_email TEXT,
                license TEXT NOT NULL,
                homepage TEXT,
                repository TEXT,
                keywords TEXT NOT NULL, -- JSON array
                categories TEXT NOT NULL, -- JSON array
                dependencies TEXT NOT NULL, -- JSON object
                lumos_version TEXT NOT NULL,
                manifest TEXT NOT NULL, -- JSON object
                metadata TEXT NOT NULL, -- JSON object
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                published_at TEXT,
                download_count INTEGER NOT NULL DEFAULT 0,
                rating REAL NOT NULL DEFAULT 0.0,
                rating_count INTEGER NOT NULL DEFAULT 0,
                published BOOLEAN NOT NULL DEFAULT FALSE,
                verified BOOLEAN NOT NULL DEFAULT FALSE,
                security_audit TEXT, -- JSON object
                performance_benchmark TEXT, -- JSON object
                UNIQUE(name, version)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        // 创建索引
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_packages_name ON tool_packages(name)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_packages_published ON tool_packages(published)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_packages_download_count ON tool_packages(download_count DESC)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_packages_rating ON tool_packages(rating DESC)")
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    async fn save_package(&self, package: &ToolPackage) -> Result<()> {
        let keywords_json = serde_json::to_string(&package.keywords)?;
        let categories_json = serde_json::to_string(&package.categories)?;
        let dependencies_json = serde_json::to_string(&package.dependencies)?;
        let manifest_json = serde_json::to_string(&package.manifest)?;
        let metadata_json = serde_json::to_string(&package.metadata)?;
        let security_audit_json = package.security_audit.as_ref()
            .map(|audit| serde_json::to_string(audit))
            .transpose()?;
        let performance_benchmark_json = package.performance_benchmark.as_ref()
            .map(|benchmark| serde_json::to_string(benchmark))
            .transpose()?;
        
        sqlx::query(
            r#"
            INSERT INTO tool_packages (
                id, name, version, description, author, author_email, license,
                homepage, repository, keywords, categories, dependencies,
                lumos_version, manifest, metadata, created_at, updated_at,
                published_at, download_count, rating, rating_count,
                published, verified, security_audit, performance_benchmark
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25
            )
            "#,
        )
        .bind(package.id.to_string())
        .bind(&package.name)
        .bind(package.version.to_string())
        .bind(&package.description)
        .bind(&package.author)
        .bind(&package.author_email)
        .bind(&package.license)
        .bind(&package.homepage)
        .bind(&package.repository)
        .bind(keywords_json)
        .bind(categories_json)
        .bind(dependencies_json)
        .bind(&package.lumos_version)
        .bind(manifest_json)
        .bind(metadata_json)
        .bind(package.created_at.to_rfc3339())
        .bind(package.updated_at.to_rfc3339())
        .bind(package.published_at.map(|dt| dt.to_rfc3339()))
        .bind(package.download_count as i64)
        .bind(package.rating)
        .bind(package.rating_count as i64)
        .bind(package.published)
        .bind(package.verified)
        .bind(security_audit_json)
        .bind(performance_benchmark_json)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn get_package(&self, id: Uuid) -> Result<Option<ToolPackage>> {
        let row = sqlx::query("SELECT * FROM tool_packages WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        
        match row {
            Some(row) => Ok(Some(self.row_to_package(row)?)),
            None => Ok(None),
        }
    }
    
    async fn get_package_by_name_version(&self, name: &str, version: &str) -> Result<Option<ToolPackage>> {
        let row = sqlx::query("SELECT * FROM tool_packages WHERE name = ? AND version = ?")
            .bind(name)
            .bind(version)
            .fetch_optional(&self.pool)
            .await?;
        
        match row {
            Some(row) => Ok(Some(self.row_to_package(row)?)),
            None => Ok(None),
        }
    }
    
    async fn update_package(&self, package: &ToolPackage) -> Result<()> {
        let keywords_json = serde_json::to_string(&package.keywords)?;
        let categories_json = serde_json::to_string(&package.categories)?;
        let dependencies_json = serde_json::to_string(&package.dependencies)?;
        let manifest_json = serde_json::to_string(&package.manifest)?;
        let metadata_json = serde_json::to_string(&package.metadata)?;
        let security_audit_json = package.security_audit.as_ref()
            .map(|audit| serde_json::to_string(audit))
            .transpose()?;
        let performance_benchmark_json = package.performance_benchmark.as_ref()
            .map(|benchmark| serde_json::to_string(benchmark))
            .transpose()?;
        
        sqlx::query(
            r#"
            UPDATE tool_packages SET
                description = ?2, author = ?3, author_email = ?4, license = ?5,
                homepage = ?6, repository = ?7, keywords = ?8, categories = ?9,
                dependencies = ?10, lumos_version = ?11, manifest = ?12,
                metadata = ?13, updated_at = ?14, published_at = ?15,
                download_count = ?16, rating = ?17, rating_count = ?18,
                published = ?19, verified = ?20, security_audit = ?21,
                performance_benchmark = ?22
            WHERE id = ?1
            "#,
        )
        .bind(package.id.to_string())
        .bind(&package.description)
        .bind(&package.author)
        .bind(&package.author_email)
        .bind(&package.license)
        .bind(&package.homepage)
        .bind(&package.repository)
        .bind(keywords_json)
        .bind(categories_json)
        .bind(dependencies_json)
        .bind(&package.lumos_version)
        .bind(manifest_json)
        .bind(metadata_json)
        .bind(package.updated_at.to_rfc3339())
        .bind(package.published_at.map(|dt| dt.to_rfc3339()))
        .bind(package.download_count as i64)
        .bind(package.rating)
        .bind(package.rating_count as i64)
        .bind(package.published)
        .bind(package.verified)
        .bind(security_audit_json)
        .bind(performance_benchmark_json)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn delete_package(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM tool_packages WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    async fn list_packages(&self, offset: u32, limit: u32) -> Result<Vec<ToolPackage>> {
        let rows = sqlx::query("SELECT * FROM tool_packages ORDER BY created_at DESC LIMIT ? OFFSET ?")
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;
        
        let mut packages = Vec::new();
        for row in rows {
            packages.push(self.row_to_package(row)?);
        }
        
        Ok(packages)
    }
    
    async fn search_by_category(&self, category: &ToolCategory) -> Result<Vec<ToolPackage>> {
        let category_str = serde_json::to_string(category)?;
        let rows = sqlx::query("SELECT * FROM tool_packages WHERE categories LIKE ? AND published = TRUE")
            .bind(format!("%{}%", category_str.trim_matches('"')))
            .fetch_all(&self.pool)
            .await?;
        
        let mut packages = Vec::new();
        for row in rows {
            packages.push(self.row_to_package(row)?);
        }
        
        Ok(packages)
    }
    
    async fn search_by_keywords(&self, keywords: &[String]) -> Result<Vec<ToolPackage>> {
        let mut query = "SELECT * FROM tool_packages WHERE published = TRUE AND (".to_string();
        let mut conditions = Vec::new();
        
        for keyword in keywords {
            conditions.push(format!(
                "(name LIKE '%{}%' OR description LIKE '%{}%' OR keywords LIKE '%{}%')",
                keyword, keyword, keyword
            ));
        }
        
        query.push_str(&conditions.join(" OR "));
        query.push_str(") ORDER BY rating DESC, download_count DESC");
        
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await?;
        
        let mut packages = Vec::new();
        for row in rows {
            packages.push(self.row_to_package(row)?);
        }
        
        Ok(packages)
    }
    
    async fn get_popular_packages(&self, limit: u32) -> Result<Vec<ToolPackage>> {
        let rows = sqlx::query(
            "SELECT * FROM tool_packages WHERE published = TRUE ORDER BY download_count DESC, rating DESC LIMIT ?"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;
        
        let mut packages = Vec::new();
        for row in rows {
            packages.push(self.row_to_package(row)?);
        }
        
        Ok(packages)
    }
    
    async fn increment_download_count(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE tool_packages SET download_count = download_count + 1 WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    async fn update_rating(&self, id: Uuid, rating: f64, count: u32) -> Result<()> {
        sqlx::query("UPDATE tool_packages SET rating = ?, rating_count = ? WHERE id = ?")
            .bind(rating)
            .bind(count as i64)
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    async fn get_statistics(&self) -> Result<StorageStatistics> {
        // 获取总数统计
        let total_row = sqlx::query("SELECT COUNT(*) as count FROM tool_packages")
            .fetch_one(&self.pool)
            .await?;
        let total_packages: i64 = total_row.get("count");
        
        let published_row = sqlx::query("SELECT COUNT(*) as count FROM tool_packages WHERE published = TRUE")
            .fetch_one(&self.pool)
            .await?;
        let published_packages: i64 = published_row.get("count");
        
        let downloads_row = sqlx::query("SELECT SUM(download_count) as total FROM tool_packages")
            .fetch_one(&self.pool)
            .await?;
        let total_downloads: Option<i64> = downloads_row.get("total");
        
        let rating_row = sqlx::query("SELECT AVG(rating) as avg FROM tool_packages WHERE rating_count > 0")
            .fetch_one(&self.pool)
            .await?;
        let average_rating: Option<f64> = rating_row.get("avg");
        
        // 按分类统计（简化实现）
        let category_counts = HashMap::new();
        
        Ok(StorageStatistics {
            total_packages: total_packages as u64,
            published_packages: published_packages as u64,
            total_downloads: total_downloads.unwrap_or(0) as u64,
            average_rating: average_rating.unwrap_or(0.0),
            category_counts,
        })
    }
}

impl SqliteStorage {
    /// 将数据库行转换为工具包
    fn row_to_package(&self, row: sqlx::sqlite::SqliteRow) -> Result<ToolPackage> {
        let id_str: String = row.get("id");
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| MarketplaceError::Internal(format!("Invalid UUID: {}", e)))?;
        
        let version_str: String = row.get("version");
        let version = semver::Version::parse(&version_str)
            .map_err(|e| MarketplaceError::Version(format!("Invalid version: {}", e)))?;
        
        let keywords_json: String = row.get("keywords");
        let keywords: Vec<String> = serde_json::from_str(&keywords_json)?;
        
        let categories_json: String = row.get("categories");
        let categories: Vec<ToolCategory> = serde_json::from_str(&categories_json)?;
        
        let dependencies_json: String = row.get("dependencies");
        let dependencies: HashMap<String, String> = serde_json::from_str(&dependencies_json)?;
        
        let manifest_json: String = row.get("manifest");
        let manifest: ToolManifest = serde_json::from_str(&manifest_json)?;
        
        let metadata_json: String = row.get("metadata");
        let metadata: HashMap<String, Value> = serde_json::from_str(&metadata_json)?;
        
        let created_at_str: String = row.get("created_at");
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| MarketplaceError::Internal(format!("Invalid datetime: {}", e)))?
            .with_timezone(&Utc);
        
        let updated_at_str: String = row.get("updated_at");
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| MarketplaceError::Internal(format!("Invalid datetime: {}", e)))?
            .with_timezone(&Utc);
        
        let published_at_str: Option<String> = row.get("published_at");
        let published_at = published_at_str
            .map(|s| DateTime::parse_from_rfc3339(&s))
            .transpose()
            .map_err(|e| MarketplaceError::Internal(format!("Invalid datetime: {}", e)))?
            .map(|dt| dt.with_timezone(&Utc));
        
        let security_audit_json: Option<String> = row.get("security_audit");
        let security_audit = security_audit_json
            .map(|json| serde_json::from_str(&json))
            .transpose()?;
        
        let performance_benchmark_json: Option<String> = row.get("performance_benchmark");
        let performance_benchmark = performance_benchmark_json
            .map(|json| serde_json::from_str(&json))
            .transpose()?;
        
        Ok(ToolPackage {
            id,
            name: row.get("name"),
            version,
            description: row.get("description"),
            author: row.get("author"),
            author_email: row.get("author_email"),
            license: row.get("license"),
            homepage: row.get("homepage"),
            repository: row.get("repository"),
            keywords,
            categories,
            dependencies,
            lumos_version: row.get("lumos_version"),
            manifest,
            metadata,
            created_at,
            updated_at,
            published_at,
            download_count: row.get::<i64, _>("download_count") as u64,
            rating: row.get("rating"),
            rating_count: row.get::<i64, _>("rating_count") as u32,
            published: row.get("published"),
            verified: row.get("verified"),
            security_audit,
            performance_benchmark,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    async fn create_test_storage() -> SqliteStorage {
        let temp_file = NamedTempFile::new().unwrap();
        let database_url = format!("sqlite://{}", temp_file.path().display());
        let storage = SqliteStorage::new(&database_url).await.unwrap();
        storage.initialize().await.unwrap();
        storage
    }
    
    #[tokio::test]
    async fn test_storage_initialization() {
        let storage = create_test_storage().await;
        let stats = storage.get_statistics().await.unwrap();
        assert_eq!(stats.total_packages, 0);
    }
    
    #[tokio::test]
    async fn test_package_crud() {
        let storage = create_test_storage().await;
        
        // 创建测试工具包
        let package = create_test_package();
        
        // 保存
        storage.save_package(&package).await.unwrap();
        
        // 获取
        let retrieved = storage.get_package(package.id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, package.name);
        assert_eq!(retrieved.version, package.version);
        
        // 更新
        let mut updated_package = retrieved;
        updated_package.description = "Updated description".to_string();
        storage.update_package(&updated_package).await.unwrap();
        
        // 验证更新
        let retrieved_updated = storage.get_package(package.id).await.unwrap().unwrap();
        assert_eq!(retrieved_updated.description, "Updated description");
        
        // 删除
        storage.delete_package(package.id).await.unwrap();
        let deleted = storage.get_package(package.id).await.unwrap();
        assert!(deleted.is_none());
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
            manifest: ToolManifest {
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
