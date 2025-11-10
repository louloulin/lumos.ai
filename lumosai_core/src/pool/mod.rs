//! Resource Pool Module
//!
//! 资源池管理模块，提供连接池、对象池等资源管理功能
//!
//! # 功能
//!
//! - **连接池**: HTTP、数据库、向量数据库连接池
//! - **对象池**: Agent、Tool、内存对象池
//! - **资源监控**: 实时资源使用监控和统计
//! - **自动扩缩容**: 基于负载的自动资源调整
//!
//! # 示例
//!
//! ```rust
//! use lumosai_core::pool::{ConnectionPool, PoolConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 创建 HTTP 连接池
//!     let pool = ConnectionPool::new(PoolConfig {
//!         min_size: 5,
//!         max_size: 20,
//!         idle_timeout: std::time::Duration::from_secs(300),
//!         ..Default::default()
//!     });
//!
//!     // 获取连接
//!     let conn = pool.acquire().await?;
//!     
//!     // 使用连接...
//!     
//!     // 连接自动归还到池中
//!     Ok(())
//! }
//! ```

pub mod connection_pool;
pub mod object_pool;
pub mod resource_monitor;

pub use connection_pool::{ConnectionPool, ConnectionPoolConfig, PooledConnection};
pub use object_pool::{ObjectPool, ObjectPoolConfig};
pub use resource_monitor::{ResourceMonitor, ResourceStats};

use crate::Result;
use std::time::Duration;

/// 池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// 最小连接数/对象数
    pub min_size: usize,
    /// 最大连接数/对象数
    pub max_size: usize,
    /// 空闲超时时间
    pub idle_timeout: Duration,
    /// 连接/对象最大生命周期
    pub max_lifetime: Option<Duration>,
    /// 获取连接/对象的超时时间
    pub acquire_timeout: Duration,
    /// 是否启用自动扩缩容
    pub auto_scaling: bool,
    /// 健康检查间隔
    pub health_check_interval: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_size: 5,
            max_size: 20,
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Some(Duration::from_secs(3600)),
            acquire_timeout: Duration::from_secs(30),
            auto_scaling: true,
            health_check_interval: Duration::from_secs(60),
        }
    }
}

/// 池统计信息
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// 当前活跃连接/对象数
    pub active: usize,
    /// 当前空闲连接/对象数
    pub idle: usize,
    /// 总连接/对象数
    pub total: usize,
    /// 等待队列长度
    pub waiting: usize,
    /// 总获取次数
    pub total_acquires: u64,
    /// 总归还次数
    pub total_releases: u64,
    /// 总创建次数
    pub total_creates: u64,
    /// 总销毁次数
    pub total_destroys: u64,
    /// 获取超时次数
    pub acquire_timeouts: u64,
    /// 平均获取时间（毫秒）
    pub avg_acquire_time_ms: f64,
    /// 资源利用率（0.0 - 1.0）
    pub utilization: f64,
}

impl PoolStats {
    /// 计算资源利用率
    pub fn calculate_utilization(&mut self) {
        if self.total > 0 {
            self.utilization = self.active as f64 / self.total as f64;
        } else {
            self.utilization = 0.0;
        }
    }

    /// 是否需要扩容
    pub fn should_scale_up(&self, threshold: f64) -> bool {
        self.utilization > threshold && self.waiting > 0
    }

    /// 是否需要缩容
    pub fn should_scale_down(&self, threshold: f64) -> bool {
        self.utilization < threshold && self.idle > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.min_size, 5);
        assert_eq!(config.max_size, 20);
        assert!(config.auto_scaling);
    }

    #[test]
    fn test_pool_stats_utilization() {
        let mut stats = PoolStats {
            active: 8,
            idle: 2,
            total: 10,
            ..Default::default()
        };

        stats.calculate_utilization();
        assert_eq!(stats.utilization, 0.8);
    }

    #[test]
    fn test_pool_stats_should_scale_up() {
        let stats = PoolStats {
            active: 18,
            idle: 2,
            total: 20,
            waiting: 5,
            utilization: 0.9,
            ..Default::default()
        };

        assert!(stats.should_scale_up(0.85));
        assert!(!stats.should_scale_up(0.95));
    }

    #[test]
    fn test_pool_stats_should_scale_down() {
        let stats = PoolStats {
            active: 2,
            idle: 8,
            total: 10,
            waiting: 0,
            utilization: 0.2,
            ..Default::default()
        };

        assert!(stats.should_scale_down(0.3));
        assert!(!stats.should_scale_down(0.1));
    }

    #[test]
    fn test_pool_stats_zero_total() {
        let mut stats = PoolStats {
            active: 0,
            idle: 0,
            total: 0,
            ..Default::default()
        };

        stats.calculate_utilization();
        assert_eq!(stats.utilization, 0.0);
    }
}
