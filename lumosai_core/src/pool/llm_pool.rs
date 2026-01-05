//! LLM Provider Connection Pool
//!
//! 专门为LLM提供者设计的连接池，支持并发访问和自动健康检查

use super::{ConnectionPool, ConnectionPoolConfig, PooledConnection, PoolStats};
use crate::llm::{LlmProvider, LlmOptions, Message};
use crate::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// LLM连接包装器
pub struct LlmConnection {
    provider: Arc<dyn LlmProvider>,
    created_at: Instant,
    last_used: Instant,
    request_count: u64,
    error_count: u64,
}

impl LlmConnection {
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        let now = Instant::now();
        Self {
            provider,
            created_at: now,
            last_used: now,
            request_count: 0,
            error_count: 0,
        }
    }

    /// 执行LLM生成请求
    pub async fn generate(&mut self, prompt: &str, options: &LlmOptions) -> Result<String> {
        self.last_used = Instant::now();
        self.request_count += 1;

        match self.provider.generate(prompt, options).await {
            Ok(result) => Ok(result),
            Err(e) => {
                self.error_count += 1;
                Err(e)
            }
        }
    }

    /// 执行LLM对话请求
    pub async fn generate_with_messages(&mut self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        self.last_used = Instant::now();
        self.request_count += 1;

        match self.provider.generate_with_messages(messages, options).await {
            Ok(result) => Ok(result),
            Err(e) => {
                self.error_count += 1;
                Err(e)
            }
        }
    }

    /// 获取连接统计信息
    pub fn stats(&self) -> LlmConnectionStats {
        LlmConnectionStats {
            created_at: self.created_at,
            last_used: self.last_used,
            request_count: self.request_count,
            error_count: self.error_count,
            error_rate: if self.request_count > 0 {
                self.error_count as f64 / self.request_count as f64
            } else {
                0.0
            },
        }
    }
}

#[async_trait]
impl super::connection_pool::Connection for LlmConnection {
    async fn is_healthy(&self) -> bool {
        // 检查错误率是否过高
        let stats = self.stats();
        stats.error_rate < 0.1 && // 错误率小于10%
        self.last_used.elapsed() < Duration::from_secs(300) // 最近5分钟内使用过
    }

    async fn reset(&mut self) -> Result<()> {
        // LLM连接通常不需要重置，但可以重置统计信息
        self.request_count = 0;
        self.error_count = 0;
        Ok(())
    }

    fn created_at(&self) -> Instant {
        self.created_at
    }
}

/// LLM连接统计信息
#[derive(Debug, Clone)]
pub struct LlmConnectionStats {
    pub created_at: Instant,
    pub last_used: Instant,
    pub request_count: u64,
    pub error_count: u64,
    pub error_rate: f64,
}

/// LLM提供者连接池
pub struct LlmConnectionPool {
    pool: ConnectionPool<LlmConnection>,
    provider_factory: Arc<dyn LlmProviderFactory>,
    stats: Arc<tokio::sync::RwLock<LlmPoolStats>>,
}

impl LlmConnectionPool {
    /// 创建LLM连接池
    pub fn new(
        config: ConnectionPoolConfig,
        provider_factory: Arc<dyn LlmProviderFactory>,
    ) -> Self {
        Self {
            pool: ConnectionPool::new(config),
            provider_factory,
            stats: Arc::new(tokio::sync::RwLock::new(LlmPoolStats::default())),
        }
    }

    /// 获取LLM连接
    pub async fn acquire(&self) -> Result<PooledConnection<LlmConnection>> {
        let mut stats = self.stats.write().await;
        stats.total_acquires += 1;

        match self.pool.acquire().await {
            Ok(conn) => {
                stats.successful_acquires += 1;
                Ok(conn)
            }
            Err(e) => {
                stats.failed_acquires += 1;
                Err(e)
            }
        }
    }

    /// 执行LLM生成（自动获取和归还连接）
    pub async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        let mut conn = self.acquire().await?;
        let result = conn.as_mut().generate(prompt, options).await;
        // 连接会在此处自动归还到池中
        result
    }

    /// 执行LLM对话（自动获取和归还连接）
    pub async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        let mut conn = self.acquire().await?;
        let result = conn.as_mut().generate_with_messages(messages, options).await;
        // 连接会在此处自动归还到池中
        result
    }

    /// 获取池统计信息
    pub async fn stats(&self) -> Result<LlmPoolStats> {
        let mut stats = self.stats.read().await.clone();
        let pool_stats = self.pool.stats().await?;

        stats.pool_stats = pool_stats;
        Ok(stats)
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let stats = self.stats().await?;

        // 检查连接池健康状态
        if stats.pool_stats.utilization > 0.95 {
            return Ok(HealthStatus::Overloaded);
        }

        if stats.failed_acquires as f64 / stats.total_acquires as f64 > 0.1 {
            return Ok(HealthStatus::Degraded);
        }

        if stats.total_acquires > 0 {
            return Ok(HealthStatus::Healthy);
        }

        Ok(HealthStatus::Idle)
    }
}

/// LLM提供者工厂trait
#[async_trait]
pub trait LlmProviderFactory: Send + Sync {
    /// 创建新的LLM提供者实例
    async fn create_provider(&self) -> Result<Arc<dyn LlmProvider>>;
}

/// 健康状态
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 空闲
    Idle,
    /// 降级
    Degraded,
    /// 过载
    Overloaded,
}

/// LLM池统计信息
#[derive(Debug, Clone, Default)]
pub struct LlmPoolStats {
    /// 连接池基础统计
    pub pool_stats: PoolStats,
    /// 总获取次数
    pub total_acquires: u64,
    /// 成功获取次数
    pub successful_acquires: u64,
    /// 失败获取次数
    pub failed_acquires: u64,
    /// 总请求数
    pub total_requests: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    /// 错误率
    pub error_rate: f64,
}

impl LlmPoolStats {
    /// 计算错误率
    pub fn calculate_error_rate(&mut self) {
        if self.total_requests > 0 {
            self.error_rate = self.failed_acquires as f64 / self.total_requests as f64;
        } else {
            self.error_rate = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::mock::MockLlmProvider;
    use std::sync::Arc;

    struct MockLlmProviderFactory;

    #[async_trait]
    impl LlmProviderFactory for MockLlmProviderFactory {
        async fn create_provider(&self) -> Result<Arc<dyn LlmProvider>> {
            Ok(Arc::new(MockLlmProvider::new(vec!["Mock response".to_string()])))
        }
    }

    #[tokio::test]
    async fn test_llm_connection_pool() -> Result<()> {
        let factory = Arc::new(MockLlmProviderFactory);
        let config = ConnectionPoolConfig {
            min_size: 1,
            max_size: 5,
            idle_timeout: Duration::from_secs(60),
            acquire_timeout: Duration::from_secs(5),
            ..Default::default()
        };

        let pool = LlmConnectionPool::new(config, factory);

        // 测试生成请求
        let result = pool.generate("Test prompt", &LlmOptions::default()).await?;
        assert_eq!(result, "Mock response");

        // 测试统计信息
        let stats = pool.stats().await?;
        assert_eq!(stats.total_acquires, 1);
        assert_eq!(stats.successful_acquires, 1);

        // 测试健康检查
        let health = pool.health_check().await?;
        assert_eq!(health, HealthStatus::Healthy);

        Ok(())
    }

    #[tokio::test]
    async fn test_llm_connection_stats() {
        let provider = Arc::new(MockLlmProvider::new(vec!["Response".to_string()]));
        let mut connection = LlmConnection::new(provider);

        // 初始状态
        let stats = connection.stats();
        assert_eq!(stats.request_count, 0);
        assert_eq!(stats.error_count, 0);

        // 模拟请求
        let _ = connection.generate("test", &LlmOptions::default()).await;
        let stats = connection.stats();
        assert_eq!(stats.request_count, 1);
        assert_eq!(stats.error_count, 0);
    }
}



