//! Database Connection Pool Management
//!
//! 为不同类型的数据库提供统一的连接池管理接口

use super::{ConnectionPool, ConnectionPoolConfig, HealthStatus, PoolStats};
use std::collections::HashMap;
use super::connection_pool::PooledConnection;
use crate::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;

/// 数据库连接trait
#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    /// 执行查询
    async fn execute(&mut self, query: &str, params: &[&dyn serde::Serialize]) -> Result<u64>;

    /// 执行查询并返回结果
    async fn query<T: serde::de::DeserializeOwned>(
        &mut self,
        query: &str,
        params: &[&dyn serde::Serialize],
    ) -> Result<Vec<T>>;

    /// 执行查询并返回单个结果
    async fn query_one<T: serde::de::DeserializeOwned>(
        &mut self,
        query: &str,
        params: &[&dyn serde::Serialize],
    ) -> Result<Option<T>>;

    /// 开始事务
    async fn begin_transaction(&mut self) -> Result<()>;

    /// 提交事务
    async fn commit_transaction(&mut self) -> Result<()>;

    /// 回滚事务
    async fn rollback_transaction(&mut self) -> Result<()>;

    /// 检查连接是否在事务中
    fn is_in_transaction(&self) -> bool;

    /// 获取连接创建时间
    fn created_at(&self) -> Instant;
}

/// 数据库连接池配置
#[derive(Debug, Clone)]
pub struct DatabasePoolConfig {
    /// 连接池基础配置
    pub pool_config: ConnectionPoolConfig,
    /// 数据库URL
    pub database_url: String,
    /// 连接超时时间
    pub connection_timeout: std::time::Duration,
    /// 数据库类型
    pub database_type: DatabaseType,
}

impl Default for DatabasePoolConfig {
    fn default() -> Self {
        Self {
            pool_config: ConnectionPoolConfig::default(),
            database_url: "sqlite::memory:".to_string(),
            connection_timeout: std::time::Duration::from_secs(30),
            database_type: DatabaseType::Sqlite,
        }
    }
}

/// 数据库类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseType {
    Postgres,
    MySql,
    Sqlite,
    MongoDb,
    Redis,
}

/// 数据库连接池trait
#[async_trait]
pub trait DatabasePool: Send + Sync {
    /// 获取数据库连接
    async fn acquire(&self) -> Result<PooledConnection<Box<dyn DatabaseConnection>>>;

    /// 执行查询（自动获取和归还连接）
    async fn execute(&self, query: &str, params: &[&dyn serde::Serialize]) -> Result<u64>;

    /// 执行查询并返回结果（自动获取和归还连接）
    async fn query<T: serde::de::DeserializeOwned>(
        &self,
        query: &str,
        params: &[&dyn serde::Serialize],
    ) -> Result<Vec<T>>;

    /// 在事务中执行操作
    async fn transaction<F, Fut, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Box<dyn DatabaseConnection>) -> Fut + Send,
        Fut: std::future::Future<Output = Result<R>> + Send,
        R: Send;

    /// 获取池统计信息
    async fn stats(&self) -> Result<DatabasePoolStats>;

    /// 健康检查
    async fn health_check(&self) -> Result<HealthStatus>;
}

/// 数据库池统计信息
#[derive(Debug, Clone)]
pub struct DatabasePoolStats {
    /// 连接池基础统计
    pub pool_stats: PoolStats,
    /// 数据库类型
    pub database_type: DatabaseType,
    /// 总查询次数
    pub total_queries: u64,
    /// 总事务次数
    pub total_transactions: u64,
    /// 事务成功率
    pub transaction_success_rate: f64,
    /// 平均查询时间（毫秒）
    pub avg_query_time_ms: f64,
    /// 连接错误次数
    pub connection_errors: u64,
}

/// 通用数据库连接池实现
pub struct GenericDatabasePool<C: DatabaseConnection + 'static> {
    pool: ConnectionPool<C>,
    database_type: DatabaseType,
    stats: Arc<tokio::sync::RwLock<DatabasePoolStats>>,
}

impl<C: DatabaseConnection + 'static> GenericDatabasePool<C> {
    pub fn new(database_type: DatabaseType, config: ConnectionPoolConfig) -> Self {
        Self {
            pool: ConnectionPool::new(config),
            database_type: database_type.clone(),
            stats: Arc::new(tokio::sync::RwLock::new(DatabasePoolStats {
                pool_stats: PoolStats::default(),
                database_type,
                total_queries: 0,
                total_transactions: 0,
                transaction_success_rate: 1.0,
                avg_query_time_ms: 0.0,
                connection_errors: 0,
            })),
        }
    }
}

#[async_trait]
impl<C: DatabaseConnection + 'static> DatabasePool for GenericDatabasePool<C> {
    async fn acquire(&self) -> Result<PooledConnection<Box<dyn DatabaseConnection>>> {
        // 这里需要类型转换，实际实现中需要更复杂的处理
        // 暂时返回错误
        Err(crate::error::Error::UnsupportedOperation(
            "Generic database pool acquire not implemented".to_string(),
        ))
    }

    async fn execute(&self, query: &str, params: &[&dyn serde::Serialize]) -> Result<u64> {
        let mut conn = self.pool.acquire().await?;
        let start = Instant::now();
        let result = conn.as_mut().execute(query, params).await;
        let duration = start.elapsed();

        let mut stats = self.stats.write().await;
        stats.total_queries += 1;
        stats.avg_query_time_ms = (stats.avg_query_time_ms * (stats.total_queries - 1) as f64
                                 + duration.as_millis() as f64) / stats.total_queries as f64;

        result
    }

    async fn query<T: serde::de::DeserializeOwned>(
        &self,
        query: &str,
        params: &[&dyn serde::Serialize],
    ) -> Result<Vec<T>> {
        let mut conn = self.pool.acquire().await?;
        let start = Instant::now();
        let result = conn.as_mut().query(query, params).await;
        let duration = start.elapsed();

        let mut stats = self.stats.write().await;
        stats.total_queries += 1;
        stats.avg_query_time_ms = (stats.avg_query_time_ms * (stats.total_queries - 1) as f64
                                 + duration.as_millis() as f64) / stats.total_queries as f64;

        result
    }

    async fn transaction<F, Fut, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Box<dyn DatabaseConnection>) -> Fut + Send,
        Fut: std::future::Future<Output = Result<R>> + Send,
        R: Send,
    {
        let mut conn = self.pool.acquire().await?;
        let mut stats = self.stats.write().await;
        stats.total_transactions += 1;

        // 开始事务
        conn.as_mut().begin_transaction().await?;

        // 执行用户函数
        match f(conn.as_mut()).await {
            Ok(result) => {
                // 提交事务
                match conn.as_mut().commit_transaction().await {
                    Ok(_) => {
                        stats.transaction_success_rate =
                            (stats.transaction_success_rate * (stats.total_transactions - 1) as f64 + 1.0)
                            / stats.total_transactions as f64;
                        Ok(result)
                    }
                    Err(e) => {
                        // 提交失败，回滚
                        let _ = conn.as_mut().rollback_transaction().await;
                        stats.transaction_success_rate =
                            (stats.transaction_success_rate * (stats.total_transactions - 1) as f64)
                            / stats.total_transactions as f64;
                        Err(e)
                    }
                }
            }
            Err(e) => {
                // 用户函数失败，回滚事务
                let _ = conn.as_mut().rollback_transaction().await;
                stats.transaction_success_rate =
                    (stats.transaction_success_rate * (stats.total_transactions - 1) as f64)
                    / stats.total_transactions as f64;
                Err(e)
            }
        }
    }

    async fn stats(&self) -> Result<DatabasePoolStats> {
        let mut stats = self.stats.read().await.clone();
        stats.pool_stats = self.pool.stats().await?;
        Ok(stats)
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let stats = self.stats().await?;

        // 检查连接池健康状态
        if stats.pool_stats.utilization > 0.95 {
            return Ok(HealthStatus::Overloaded);
        }

        if stats.connection_errors > stats.total_queries / 10 {
            return Ok(HealthStatus::Degraded);
        }

        if stats.total_queries > 0 {
            return Ok(HealthStatus::Healthy);
        }

        Ok(HealthStatus::Idle)
    }
}

/// 数据库池管理器 - 管理多种数据库连接池
pub struct DatabasePoolManager {
    pools: Arc<tokio::sync::RwLock<HashMap<String, Box<dyn DatabasePool>>>>,
}

impl DatabasePoolManager {
    pub fn new() -> Self {
        Self {
            pools: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 注册数据库连接池
    pub async fn register_pool(&self, name: &str, pool: Box<dyn DatabasePool>) -> Result<()> {
        let mut pools = self.pools.write().await;
        pools.insert(name.to_string(), pool);
        Ok(())
    }

    /// 获取数据库连接池
    pub async fn get_pool(&self, name: &str) -> Result<Box<dyn DatabasePool>> {
        let pools = self.pools.read().await;
        pools.get(name)
            .cloned()
            .ok_or_else(|| crate::error::Error::NotFound(format!("Database pool '{}' not found", name)))
    }

    /// 获取所有池的统计信息
    pub async fn get_all_stats(&self) -> Result<HashMap<String, DatabasePoolStats>> {
        let pools = self.pools.read().await;
        let mut stats = HashMap::new();

        for (name, pool) in pools.iter() {
            if let Ok(pool_stats) = pool.stats().await {
                stats.insert(name.clone(), pool_stats);
            }
        }

        Ok(stats)
    }

    /// 执行全局健康检查
    pub async fn health_check_all(&self) -> Result<HashMap<String, HealthStatus>> {
        let pools = self.pools.read().await;
        let mut health_status = HashMap::new();

        for (name, pool) in pools.iter() {
            let status = pool.health_check().await.unwrap_or(HealthStatus::Degraded);
            health_status.insert(name.clone(), status);
        }

        Ok(health_status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock数据库连接实现
    struct MockDatabaseConnection {
        created_at: Instant,
        in_transaction: bool,
    }

    impl MockDatabaseConnection {
        fn new() -> Self {
            Self {
                created_at: Instant::now(),
                in_transaction: false,
            }
        }
    }

    #[async_trait]
    impl DatabaseConnection for MockDatabaseConnection {
        async fn execute(&mut self, _query: &str, _params: &[&dyn serde::Serialize]) -> Result<u64> {
            Ok(1)
        }

        async fn query<T: serde::de::DeserializeOwned>(
            &mut self,
            _query: &str,
            _params: &[&dyn serde::Serialize],
        ) -> Result<Vec<T>> {
            Ok(vec![])
        }

        async fn query_one<T: serde::de::DeserializeOwned>(
            &mut self,
            _query: &str,
            _params: &[&dyn serde::Serialize],
        ) -> Result<Option<T>> {
            Ok(None)
        }

        async fn begin_transaction(&mut self) -> Result<()> {
            self.in_transaction = true;
            Ok(())
        }

        async fn commit_transaction(&mut self) -> Result<()> {
            self.in_transaction = false;
            Ok(())
        }

        async fn rollback_transaction(&mut self) -> Result<()> {
            self.in_transaction = false;
            Ok(())
        }

        fn is_in_transaction(&self) -> bool {
            self.in_transaction
        }

        fn created_at(&self) -> Instant {
            self.created_at
        }
    }

    #[async_trait]
    impl super::connection_pool::Connection for MockDatabaseConnection {
        async fn is_healthy(&self) -> bool {
            true
        }

        async fn reset(&mut self) -> Result<()> {
            self.in_transaction = false;
            Ok(())
        }

        fn created_at(&self) -> Instant {
            self.created_at
        }
    }

    #[tokio::test]
    async fn test_database_pool_manager() -> Result<()> {
        let manager = DatabasePoolManager::new();

        // 创建模拟数据库池
        let pool = GenericDatabasePool::<MockDatabaseConnection>::new(
            DatabaseType::Sqlite,
            ConnectionPoolConfig::default(),
        );

        // 注册池
        manager.register_pool("test_db", Box::new(pool)).await?;

        // 获取池
        let retrieved_pool = manager.get_pool("test_db").await?;
        assert!(retrieved_pool.stats().await.is_ok());

        // 获取所有统计
        let all_stats = manager.get_all_stats().await?;
        assert_eq!(all_stats.len(), 1);
        assert!(all_stats.contains_key("test_db"));

        Ok(())
    }

    #[tokio::test]
    async fn test_health_check_all() -> Result<()> {
        let manager = DatabasePoolManager::new();

        // 创建模拟数据库池
        let pool = GenericDatabasePool::<MockDatabaseConnection>::new(
            DatabaseType::Postgres,
            ConnectionPoolConfig::default(),
        );

        manager.register_pool("postgres", Box::new(pool)).await?;

        // 执行健康检查
        let health_status = manager.health_check_all().await?;
        assert_eq!(health_status.len(), 1);
        assert!(health_status.contains_key("postgres"));

        Ok(())
    }
}
