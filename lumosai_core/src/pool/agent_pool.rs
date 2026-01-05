//! Agent Instance Object Pool
//!
//! 专门为Agent实例设计的对象池，支持复用昂贵的Agent对象和自动生命周期管理

use super::{ObjectPool, ObjectPoolConfig, HealthStatus};
use super::object_pool::PooledObject;
use crate::agent::{Agent, AgentConfig, BasicAgent};
use crate::llm::LlmProvider;
use crate::memory::Memory;
use crate::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;

/// 可池化的Agent包装器
pub struct PoolableAgent {
    agent: BasicAgent,
    created_at: Instant,
    last_used: Instant,
    request_count: u64,
    error_count: u64,
    config_hash: String, // 用于验证配置一致性
}

impl PoolableAgent {
    /// 创建新的池化Agent
    pub async fn new(
        config: &AgentConfig,
        llm_provider: Arc<dyn LlmProvider>,
        memory: Option<Arc<dyn Memory>>,
    ) -> Result<Self> {
        let agent = BasicAgent::new(config.clone(), llm_provider, memory).await?;
        let config_hash = Self::calculate_config_hash(config);

        Ok(Self {
            agent,
            created_at: Instant::now(),
            last_used: Instant::now(),
            request_count: 0,
            error_count: 0,
            config_hash,
        })
    }

    /// 执行Agent生成
    pub async fn generate(
        &mut self,
        messages: &[crate::llm::Message],
        options: &crate::agent::AgentGenerateOptions,
    ) -> Result<crate::agent::AgentGenerateResult> {
        self.last_used = Instant::now();
        self.request_count += 1;

        match self.agent.generate(messages, options).await {
            Ok(result) => Ok(result),
            Err(e) => {
                self.error_count += 1;
                Err(e)
            }
        }
    }

    /// 获取Agent配置哈希
    pub fn config_hash(&self) -> &str {
        &self.config_hash
    }

    /// 获取Agent统计信息
    pub fn stats(&self) -> AgentStats {
        AgentStats {
            created_at: self.created_at,
            last_used: self.last_used,
            request_count: self.request_count,
            error_count: self.error_count,
            error_rate: if self.request_count > 0 {
                self.error_count as f64 / self.request_count as f64
            } else {
                0.0
            },
            config_hash: self.config_hash.clone(),
        }
    }

    /// 计算配置哈希
    fn calculate_config_hash(config: &AgentConfig) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        config.name.hash(&mut hasher);
        config.instructions.hash(&mut hasher);
        // 添加其他重要配置字段的哈希

        format!("{:x}", hasher.finish())
    }
}

impl super::object_pool::Poolable for PoolableAgent {
    fn reset(&mut self) {
        // 重置Agent状态，但保持配置
        self.request_count = 0;
        self.error_count = 0;
        // 注意：Agent实例本身的状态重置需要根据具体实现来定
    }

    fn is_valid(&self) -> bool {
        // 检查Agent是否仍然有效
        let stats = self.stats();
        stats.error_rate < 0.2 && // 错误率小于20%
        self.last_used.elapsed().as_secs() < 3600 // 最近1小时内使用过
    }

    fn created_at(&self) -> Instant {
        self.created_at
    }
}

/// Agent统计信息
#[derive(Debug, Clone)]
pub struct AgentStats {
    pub created_at: Instant,
    pub last_used: Instant,
    pub request_count: u64,
    pub error_count: u64,
    pub error_rate: f64,
    pub config_hash: String,
}

/// Agent池配置
#[derive(Debug, Clone)]
pub struct AgentPoolConfig {
    /// 对象池基础配置
    pub pool_config: ObjectPoolConfig,
    /// Agent配置模板
    pub agent_config: AgentConfig,
    /// 是否允许不同配置的Agent混用
    pub allow_mixed_configs: bool,
}

impl Default for AgentPoolConfig {
    fn default() -> Self {
        Self {
            pool_config: ObjectPoolConfig::default(),
            agent_config: AgentConfig::default(),
            allow_mixed_configs: false,
        }
    }
}

/// Agent工厂trait
#[async_trait]
pub trait AgentFactory: Send + Sync {
    /// 创建Agent实例
    async fn create_agent(&self, config: &AgentConfig) -> Result<BasicAgent>;

    /// 验证配置兼容性
    fn is_config_compatible(&self, config: &AgentConfig) -> bool;
}

/// 默认Agent工厂实现
pub struct DefaultAgentFactory {
    llm_provider: Arc<dyn LlmProvider>,
    memory: Option<Arc<dyn Memory>>,
}

impl DefaultAgentFactory {
    pub fn new(llm_provider: Arc<dyn LlmProvider>, memory: Option<Arc<dyn Memory>>) -> Self {
        Self {
            llm_provider,
            memory,
        }
    }
}

#[async_trait]
impl AgentFactory for DefaultAgentFactory {
    async fn create_agent(&self, config: &AgentConfig) -> Result<BasicAgent> {
        BasicAgent::new(config.clone(), self.llm_provider.clone(), self.memory.clone()).await
    }

    fn is_config_compatible(&self, _config: &AgentConfig) -> bool {
        true // 默认工厂接受所有配置
    }
}

/// Agent实例对象池
pub struct AgentPool {
    pool: ObjectPool<PoolableAgent>,
    factory: Arc<dyn AgentFactory>,
    config: AgentPoolConfig,
    stats: Arc<tokio::sync::RwLock<AgentPoolStats>>,
}

impl AgentPool {
    /// 创建Agent池
    pub fn new(config: AgentPoolConfig, factory: Arc<dyn AgentFactory>) -> Self {
        Self {
            pool: ObjectPool::new(config.pool_config.clone()),
            factory,
            config,
            stats: Arc::new(tokio::sync::RwLock::new(AgentPoolStats::default())),
        }
    }

    /// 获取Agent实例
    pub async fn acquire(&self) -> Result<PooledObject<PoolableAgent>> {
        let mut stats = self.stats.write().await;
        stats.total_acquires += 1;

        match self.pool.acquire().await {
            Ok(agent) => {
                stats.successful_acquires += 1;
                Ok(agent)
            }
            Err(e) => {
                stats.failed_acquires += 1;
                Err(e)
            }
        }
    }

    /// 执行Agent生成（自动获取和归还Agent）
    pub async fn generate(
        &self,
        messages: &[crate::llm::Message],
        options: &crate::agent::AgentGenerateOptions,
    ) -> Result<crate::agent::AgentGenerateResult> {
        let mut agent = self.acquire().await?;
        let result = agent.as_mut().generate(messages, options).await;
        // Agent会在此处自动归还到池中
        result
    }

    /// 获取指定配置的Agent
    pub async fn acquire_with_config(&self, agent_config: &AgentConfig) -> Result<PooledObject<PoolableAgent>> {
        // 检查配置兼容性
        if !self.factory.is_config_compatible(agent_config) {
            return Err(crate::error::Error::InvalidArgument(
                "Agent configuration not compatible with pool factory".to_string(),
            ));
        }

        // 如果不允许混用配置，检查配置哈希
        if !self.config.allow_mixed_configs {
            let config_hash = PoolableAgent::calculate_config_hash(agent_config);
            let pool_config_hash = PoolableAgent::calculate_config_hash(&self.config.agent_config);

            if config_hash != pool_config_hash {
                return Err(crate::error::Error::InvalidArgument(
                    "Mixed configurations not allowed in this pool".to_string(),
                ));
            }
        }

        self.acquire().await
    }

    /// 获取池统计信息
    pub async fn stats(&self) -> Result<AgentPoolStats> {
        let mut stats = self.stats.read().await.clone();
        let pool_stats = self.pool.stats().await?;

        stats.pool_stats = pool_stats;
        Ok(stats)
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let stats = self.stats().await?;

        // 检查Agent池健康状态
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

    /// 预热池（预创建Agent实例）
    pub async fn warmup(&self, count: usize) -> Result<()> {
        for _ in 0..count {
            let agent_config = &self.config.agent_config;
            let agent = PoolableAgent::new(
                agent_config,
                // 这里需要从factory获取LLM provider，暂时使用默认方式
                todo!("Implement LLM provider retrieval from factory"),
                None, // memory
            ).await?;

            self.pool.add_idle_object(agent).await?;
        }

        Ok(())
    }
}

/// Agent池统计信息
#[derive(Debug, Clone, Default)]
pub struct AgentPoolStats {
    /// 对象池基础统计
    pub pool_stats: super::PoolStats,
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

impl AgentPoolStats {
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

    #[tokio::test]
    async fn test_poolable_agent_creation() -> Result<()> {
        let llm_provider = Arc::new(MockLlmProvider::new(vec!["Mock response".to_string()]));
        let config = AgentConfig::default();

        let agent = PoolableAgent::new(&config, llm_provider, None).await?;
        let stats = agent.stats();

        assert_eq!(stats.request_count, 0);
        assert_eq!(stats.error_count, 0);
        assert!(!stats.config_hash.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_poolable_agent_stats() {
        // 这个测试需要一个完整的Agent实例，暂时跳过
        // TODO: 实现完整的Agent mock来测试统计功能
    }

    #[test]
    fn test_config_hash_consistency() {
        let config1 = AgentConfig {
            name: "test".to_string(),
            instructions: "test instructions".to_string(),
            ..Default::default()
        };

        let config2 = config1.clone();

        let hash1 = PoolableAgent::calculate_config_hash(&config1);
        let hash2 = PoolableAgent::calculate_config_hash(&config2);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_config_hash_difference() {
        let config1 = AgentConfig {
            name: "test1".to_string(),
            instructions: "test instructions".to_string(),
            ..Default::default()
        };

        let config2 = AgentConfig {
            name: "test2".to_string(),
            instructions: "test instructions".to_string(),
            ..Default::default()
        };

        let hash1 = PoolableAgent::calculate_config_hash(&config1);
        let hash2 = PoolableAgent::calculate_config_hash(&config2);

        assert_ne!(hash1, hash2);
    }
}
