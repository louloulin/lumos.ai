//! Connection Pool Implementation
//!
//! 连接池实现，支持 HTTP、数据库、向量数据库等连接管理

use super::{PoolConfig, PoolStats};
use crate::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::timeout;

/// 连接池配置
pub type ConnectionPoolConfig = PoolConfig;

/// 连接特征
#[async_trait::async_trait]
pub trait Connection: Send + Sync + 'static {
    /// 检查连接是否健康
    async fn is_healthy(&self) -> bool;

    /// 重置连接状态
    async fn reset(&mut self) -> Result<()>;

    /// 获取连接创建时间
    fn created_at(&self) -> Instant;
}

/// 池化连接包装器
pub struct PooledConnection<C: Connection> {
    connection: Option<C>,
    pool: Arc<ConnectionPoolInner<C>>,
    acquired_at: Instant,
}

impl<C: Connection> PooledConnection<C> {
    /// 获取内部连接的引用
    pub fn as_ref(&self) -> &C {
        self.connection.as_ref().unwrap()
    }

    /// 获取内部连接的可变引用
    pub fn as_mut(&mut self) -> &mut C {
        self.connection.as_mut().unwrap()
    }
}

impl<C: Connection> Drop for PooledConnection<C> {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                pool.release(conn).await;
            });
        }
    }
}

/// 连接工厂
#[async_trait::async_trait]
pub trait ConnectionFactory<C: Connection>: Send + Sync {
    /// 创建新连接
    async fn create(&self) -> Result<C>;
}

/// 连接池内部实现
struct ConnectionPoolInner<C: Connection> {
    config: ConnectionPoolConfig,
    idle_connections: Arc<Mutex<Vec<C>>>,
    semaphore: Arc<Semaphore>,
    stats: Arc<Mutex<PoolStats>>,
    factory: Arc<dyn ConnectionFactory<C>>,
}

impl<C: Connection + 'static> ConnectionPoolInner<C> {
    /// 释放连接回池中
    async fn release(&self, mut conn: C) {
        // 检查连接是否健康
        if !conn.is_healthy().await {
            // 不健康的连接直接丢弃
            let mut stats = self.stats.lock().await;
            stats.total_destroys += 1;
            stats.total -= 1;
            stats.active -= 1;
            drop(stats);

            // 释放信号量
            self.semaphore.add_permits(1);
            return;
        }

        // 重置连接状态
        if let Err(_) = conn.reset().await {
            // 重置失败，丢弃连接
            let mut stats = self.stats.lock().await;
            stats.total_destroys += 1;
            stats.total -= 1;
            stats.active -= 1;
            drop(stats);

            self.semaphore.add_permits(1);
            return;
        }

        // 检查连接生命周期
        if let Some(max_lifetime) = self.config.max_lifetime {
            if conn.created_at().elapsed() > max_lifetime {
                // 连接超过最大生命周期，丢弃
                let mut stats = self.stats.lock().await;
                stats.total_destroys += 1;
                stats.total -= 1;
                stats.active -= 1;
                drop(stats);

                self.semaphore.add_permits(1);
                return;
            }
        }

        // 归还连接到空闲池
        let mut idle = self.idle_connections.lock().await;
        idle.push(conn);
        drop(idle);

        // 更新统计
        let mut stats = self.stats.lock().await;
        stats.total_releases += 1;
        stats.active -= 1;
        stats.idle += 1;
        stats.calculate_utilization();
    }
}

/// 连接池
pub struct ConnectionPool<C: Connection> {
    inner: Arc<ConnectionPoolInner<C>>,
}

impl<C: Connection + 'static> ConnectionPool<C> {
    /// 创建新的连接池
    pub fn new(config: ConnectionPoolConfig, factory: Arc<dyn ConnectionFactory<C>>) -> Self {
        let inner = Arc::new(ConnectionPoolInner {
            config: config.clone(),
            idle_connections: Arc::new(Mutex::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(config.max_size)),
            stats: Arc::new(Mutex::new(PoolStats::default())),
            factory,
        });

        Self { inner }
    }

    /// 获取连接
    pub async fn acquire(&self) -> Result<PooledConnection<C>> {
        let start = Instant::now();

        // 尝试获取信号量许可
        let permit = match timeout(
            self.inner.config.acquire_timeout,
            self.inner.semaphore.acquire(),
        )
        .await
        {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => {
                let mut stats = self.inner.stats.lock().await;
                stats.acquire_timeouts += 1;
                return Err(crate::Error::Timeout(
                    "Failed to acquire connection".to_string(),
                ));
            }
            Err(_) => {
                let mut stats = self.inner.stats.lock().await;
                stats.acquire_timeouts += 1;
                return Err(crate::Error::Timeout(
                    "Connection acquire timeout".to_string(),
                ));
            }
        };

        // 忘记许可（我们手动管理）
        permit.forget();

        // 尝试从空闲池获取连接
        let mut idle = self.inner.idle_connections.lock().await;
        let conn = if let Some(conn) = idle.pop() {
            drop(idle);
            conn
        } else {
            drop(idle);
            // 创建新连接
            let conn = self.inner.factory.create().await?;

            let mut stats = self.inner.stats.lock().await;
            stats.total_creates += 1;
            stats.total += 1;
            drop(stats);

            conn
        };

        // 更新统计
        let mut stats = self.inner.stats.lock().await;
        stats.total_acquires += 1;
        stats.active += 1;
        if stats.idle > 0 {
            stats.idle -= 1;
        }

        // 更新平均获取时间
        let acquire_time = start.elapsed().as_millis() as f64;
        if stats.total_acquires == 1 {
            stats.avg_acquire_time_ms = acquire_time;
        } else {
            stats.avg_acquire_time_ms =
                (stats.avg_acquire_time_ms * (stats.total_acquires - 1) as f64 + acquire_time)
                    / stats.total_acquires as f64;
        }

        stats.calculate_utilization();
        drop(stats);

        Ok(PooledConnection {
            connection: Some(conn),
            pool: self.inner.clone(),
            acquired_at: Instant::now(),
        })
    }

    /// 获取池统计信息
    pub async fn stats(&self) -> PoolStats {
        self.inner.stats.lock().await.clone()
    }

    /// 预热连接池（创建最小数量的连接）
    pub async fn warmup(&self) -> Result<()> {
        let min_size = self.inner.config.min_size;
        let mut connections = Vec::new();

        for _ in 0..min_size {
            let conn = self.inner.factory.create().await?;
            connections.push(conn);
        }

        let mut idle = self.inner.idle_connections.lock().await;
        idle.extend(connections);

        let mut stats = self.inner.stats.lock().await;
        stats.total = min_size;
        stats.idle = min_size;
        stats.total_creates = min_size as u64;
        stats.calculate_utilization();

        Ok(())
    }
}

impl<C: Connection> Clone for ConnectionPool<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
