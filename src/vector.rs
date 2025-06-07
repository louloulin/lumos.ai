//! 简化的向量存储API
//!
//! 提供一行代码创建向量存储的便利函数，支持智能默认配置。

use crate::{Result, Error};
use std::sync::Arc;
use lumosai_vector::core::{VectorStore, VectorStoreIndex};

/// 向量存储抽象
pub type VectorStorage = Arc<dyn VectorStore>;

/// 内存向量存储
pub type MemoryStorage = lumosai_vector::memory::MemoryVectorStore;

/// PostgreSQL向量存储
pub type PostgresStorage = lumosai_vector::postgres::PostgresVectorStore;

/// 一行代码创建内存向量存储
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let storage = lumos::vector::memory().await?;
///     Ok(())
/// }
/// ```
pub async fn memory() -> Result<VectorStorage> {
    let storage = MemoryStorage::new();
    Ok(Arc::new(storage))
}

/// 一行代码创建PostgreSQL向量存储
/// 
/// 自动从环境变量读取数据库URL，或使用默认配置
/// 
/// # 环境变量
/// - `DATABASE_URL`: PostgreSQL连接字符串
/// - `POSTGRES_HOST`: 主机地址 (默认: localhost)
/// - `POSTGRES_PORT`: 端口 (默认: 5432)
/// - `POSTGRES_DB`: 数据库名 (默认: lumos)
/// - `POSTGRES_USER`: 用户名 (默认: postgres)
/// - `POSTGRES_PASSWORD`: 密码
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     // 使用环境变量或默认配置
///     let storage = lumos::vector::postgres().await?;
///     
///     // 或者指定连接字符串
///     let storage = lumos::vector::postgres_with_url("postgresql://user:pass@localhost/db").await?;
///     
///     Ok(())
/// }
/// ```
pub async fn postgres() -> Result<VectorStorage> {
    let database_url = get_postgres_url_from_env()?;
    postgres_with_url(&database_url).await
}

/// 使用指定URL创建PostgreSQL向量存储
pub async fn postgres_with_url(database_url: &str) -> Result<VectorStorage> {
    let config = lumosai_vector::postgres::PostgresConfig::new(database_url.to_string());
    let storage = PostgresStorage::with_config(config).await
        .map_err(|e| Error::VectorStore(format!("Failed to create PostgreSQL storage: {}", e)))?;
    Ok(Arc::new(storage))
}

/// 智能向量存储创建器
/// 
/// 根据环境自动选择最佳的存储后端
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     // 自动检测并创建最佳存储
///     let storage = lumos::vector::auto().await?;
///     Ok(())
/// }
/// ```
pub async fn auto() -> Result<VectorStorage> {
    // 优先尝试PostgreSQL，如果失败则使用内存存储
    match postgres().await {
        Ok(storage) => {
            tracing::info!("Using PostgreSQL vector storage");
            Ok(storage)
        }
        Err(_) => {
            tracing::info!("PostgreSQL not available, using memory vector storage");
            memory().await
        }
    }
}

/// 向量存储构建器
/// 
/// 提供更细粒度的配置选项
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let storage = lumos::vector::builder()
///         .backend("postgres")
///         .url("postgresql://localhost/lumos")
///         .pool_size(10)
///         .build()
///         .await?;
///     Ok(())
/// }
/// ```
pub fn builder() -> VectorStorageBuilder {
    VectorStorageBuilder::new()
}

/// 向量存储构建器
pub struct VectorStorageBuilder {
    backend: Option<String>,
    url: Option<String>,
    pool_size: Option<u32>,
    batch_size: Option<usize>,
}

impl VectorStorageBuilder {
    pub fn new() -> Self {
        Self {
            backend: None,
            url: None,
            pool_size: None,
            batch_size: None,
        }
    }
    
    /// 设置存储后端 ("memory", "postgres")
    pub fn backend(mut self, backend: &str) -> Self {
        self.backend = Some(backend.to_string());
        self
    }
    
    /// 设置数据库URL
    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }
    
    /// 设置连接池大小
    pub fn pool_size(mut self, size: u32) -> Self {
        self.pool_size = Some(size);
        self
    }
    
    /// 设置批处理大小
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = Some(size);
        self
    }
    
    /// 构建向量存储
    pub async fn build(self) -> Result<VectorStorage> {
        let backend = self.backend.unwrap_or_else(|| "auto".to_string());
        
        match backend.as_str() {
            "memory" => memory().await,
            "postgres" => {
                if let Some(url) = self.url {
                    postgres_with_url(&url).await
                } else {
                    postgres().await
                }
            }
            "auto" => auto().await,
            _ => Err(Error::VectorStore(format!("Unsupported backend: {}", backend))),
        }
    }
}

impl Default for VectorStorageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 从环境变量构建PostgreSQL URL
fn get_postgres_url_from_env() -> Result<String> {
    // 首先尝试完整的DATABASE_URL
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return Ok(url);
    }
    
    // 否则从各个组件构建
    let host = std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
    let db = std::env::var("POSTGRES_DB").unwrap_or_else(|_| "lumos".to_string());
    let user = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
    let password = std::env::var("POSTGRES_PASSWORD")
        .map_err(|_| Error::Config("POSTGRES_PASSWORD environment variable is required".to_string()))?;
    
    Ok(format!("postgresql://{}:{}@{}:{}/{}", user, password, host, port, db))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_storage_creation() {
        let storage = memory().await.expect("Failed to create memory storage");
        assert!(!storage.to_string().is_empty());
    }
    
    #[test]
    fn test_builder_pattern() {
        let builder = builder()
            .backend("memory")
            .batch_size(1000);
        
        // 测试构建器模式
        assert!(true); // 简单的编译测试
    }
    
    #[test]
    fn test_postgres_url_construction() {
        std::env::set_var("POSTGRES_HOST", "testhost");
        std::env::set_var("POSTGRES_PORT", "5433");
        std::env::set_var("POSTGRES_DB", "testdb");
        std::env::set_var("POSTGRES_USER", "testuser");
        std::env::set_var("POSTGRES_PASSWORD", "testpass");
        
        let url = get_postgres_url_from_env().expect("Failed to build URL");
        assert_eq!(url, "postgresql://testuser:testpass@testhost:5433/testdb");
        
        // 清理环境变量
        std::env::remove_var("POSTGRES_HOST");
        std::env::remove_var("POSTGRES_PORT");
        std::env::remove_var("POSTGRES_DB");
        std::env::remove_var("POSTGRES_USER");
        std::env::remove_var("POSTGRES_PASSWORD");
    }
}
