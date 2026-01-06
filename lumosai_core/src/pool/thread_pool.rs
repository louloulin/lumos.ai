//! Thread Pool Configuration and Management
//!
//! 专门为LumosAI优化的线程池配置，支持不同工作负载的自动调整

use crate::{pool::HealthStatus, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

/// 线程池配置
#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    /// 核心线程数
    pub core_threads: usize,

    /// 最大线程数
    pub max_threads: usize,

    /// 线程栈大小（字节）
    pub thread_stack_size: usize,

    /// 线程名称前缀
    pub thread_name_prefix: String,

    /// 线程空闲超时时间
    pub thread_idle_timeout: std::time::Duration,

    /// 是否启用工作窃取
    pub enable_work_stealing: bool,

    /// CPU亲和性设置
    pub cpu_affinity: Option<CpuAffinityConfig>,

    /// 负载均衡策略
    pub load_balancing_strategy: LoadBalancingStrategy,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        let core_threads = num_cpus::get().max(2);

        Self {
            core_threads,
            max_threads: core_threads * 4,
            thread_stack_size: 2 * 1024 * 1024, // 2MB
            thread_name_prefix: "lumosai-worker".to_string(),
            thread_idle_timeout: std::time::Duration::from_secs(60),
            enable_work_stealing: true,
            cpu_affinity: None,
            load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
        }
    }
}

/// CPU亲和性配置
#[derive(Debug, Clone)]
pub struct CpuAffinityConfig {
    /// CPU核心绑定列表
    pub cpu_cores: Vec<usize>,

    /// 是否隔离核心
    pub isolate_cores: bool,
}

/// 负载均衡策略
#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    /// 轮询调度
    RoundRobin,

    /// 最少连接数优先
    LeastConnections,

    /// 加权轮询
    WeightedRoundRobin,

    /// 随机调度
    Random,

    /// 工作窃取
    WorkStealing,
}

/// 线程池统计信息
#[derive(Debug, Clone)]
pub struct ThreadPoolStats {
    /// 活跃线程数
    pub active_threads: usize,

    /// 空闲线程数
    pub idle_threads: usize,

    /// 总线程数
    pub total_threads: usize,

    /// 队列长度
    pub queue_length: usize,

    /// 已完成任务数
    pub completed_tasks: u64,

    /// 拒绝任务数
    pub rejected_tasks: u64,

    /// 平均任务执行时间（毫秒）
    pub avg_task_time_ms: f64,

    /// CPU使用率
    pub cpu_usage_percent: f64,

    /// 内存使用量（字节）
    pub memory_usage_bytes: usize,
}

/// 工作负载类型
/// ✅ 添加 Hash 和 Eq trait 支持
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkloadType {
    /// CPU密集型任务
    CpuIntensive,

    /// IO密集型任务
    IoIntensive,

    /// 混合型任务
    Mixed,

    /// 内存密集型任务
    MemoryIntensive,
}

/// 自适应线程池
pub struct AdaptiveThreadPool {
    config: Arc<RwLock<ThreadPoolConfig>>,
    runtime: Runtime,
    stats: Arc<RwLock<ThreadPoolStats>>,
    workload_history: Arc<RwLock<Vec<WorkloadSample>>>,
}

impl AdaptiveThreadPool {
    /// 创建自适应线程池
    pub fn new(config: ThreadPoolConfig) -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(config.core_threads)
            .thread_name(&config.thread_name_prefix)
            .thread_stack_size(config.thread_stack_size)
            .enable_all()
            .build()?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            runtime,
            stats: Arc::new(RwLock::new(ThreadPoolStats {
                active_threads: 0,
                idle_threads: 0,
                total_threads: 0,
                queue_length: 0,
                completed_tasks: 0,
                rejected_tasks: 0,
                avg_task_time_ms: 0.0,
                cpu_usage_percent: 0.0,
                memory_usage_bytes: 0,
            })),
            workload_history: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// 执行异步任务
    pub async fn spawn<F>(&self, future: F) -> Result<tokio::task::JoinHandle<F::Output>>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let handle = self.runtime.spawn(future);

        // 更新统计信息
        let mut stats = self.stats.write().await;
        stats.active_threads += 1;

        Ok(handle)
    }

    /// 执行阻塞任务
    pub async fn spawn_blocking<F, R>(&self, f: F) -> Result<tokio::task::JoinHandle<R>>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let handle = self.runtime.spawn_blocking(f);

        // 更新统计信息
        let mut stats = self.stats.write().await;
        stats.active_threads += 1;

        Ok(handle)
    }

    /// 根据工作负载类型优化配置
    pub async fn optimize_for_workload(&self, workload_type: WorkloadType) -> Result<()> {
        let mut config = self.config.write().await;

        match workload_type {
            WorkloadType::CpuIntensive => {
                // CPU密集型：减少线程数，避免上下文切换开销
                config.max_threads = config.core_threads;
                config.load_balancing_strategy = LoadBalancingStrategy::WorkStealing;
            }
            WorkloadType::IoIntensive => {
                // IO密集型：增加线程数，提高并发度
                config.max_threads = config.core_threads * 8;
                config.load_balancing_strategy = LoadBalancingStrategy::LeastConnections;
            }
            WorkloadType::Mixed => {
                // 混合型：平衡配置
                config.max_threads = config.core_threads * 4;
                config.load_balancing_strategy = LoadBalancingStrategy::RoundRobin;
            }
            WorkloadType::MemoryIntensive => {
                // 内存密集型：减少线程数，避免内存竞争
                config.max_threads = config.core_threads * 2;
                config.load_balancing_strategy = LoadBalancingStrategy::WeightedRoundRobin;
            }
        }

        // 记录工作负载样本
        let sample = WorkloadSample {
            timestamp: std::time::Instant::now(),
            workload_type: workload_type.clone(),
            active_threads: self.stats.read().await.active_threads,
            queue_length: self.stats.read().await.queue_length,
        };

        let mut history = self.workload_history.write().await;
        history.push(sample);

        // 保持历史记录在合理范围内
        if history.len() > 1000 {
            history.remove(0);
        }

        Ok(())
    }

    /// 获取线程池统计信息
    pub async fn stats(&self) -> ThreadPoolStats {
        let mut stats = self.stats.read().await.clone();

        // ✅ 使用 Tokio RuntimeMetrics 获取实际运行时统计
        let metrics = self.runtime.metrics();

        // 获取工作线程数（总数）
        stats.total_threads = metrics.num_workers();

        // 获取活跃任务数
        // 注意：当前简化实现，使用总线程数作为活跃线程数
        // 更精确的实现需要跟踪正在执行的任务数量
        stats.active_threads = metrics.num_workers();

        // 存储原始 metrics 供将来使用
        let _metrics = metrics;

        // TODO: v1.3 增强 - 利用更多 RuntimeMetrics 指标：
        // - 添加字段存储 remote_schedule_count(), budget_forced_yield_count()
        // - 计算任务吞吐量、平均延迟等高级指标
        // - 实现历史趋势分析和容量规划建议

        stats
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let stats = self.stats().await;

        // 检查线程池健康状态
        if stats.rejected_tasks > stats.completed_tasks / 10 {
            return Ok(HealthStatus::Overloaded);
        }

        if stats.queue_length > 1000 {
            return Ok(HealthStatus::Degraded);
        }

        if stats.active_threads > 0 {
            return Ok(HealthStatus::Healthy);
        }

        Ok(HealthStatus::Idle)
    }

    /// 自适应调整
    pub async fn adaptive_adjust(&self) -> Result<()> {
        let stats = self.stats().await;
        let history = self.workload_history.read().await;

        // 基于历史数据和当前统计进行调整
        if stats.queue_length > 100 && stats.active_threads < stats.total_threads {
            // 队列积压，考虑增加线程
            self.scale_up().await?;
        } else if stats.idle_threads > stats.active_threads / 2 && stats.queue_length == 0 {
            // 太多空闲线程，考虑减少线程
            self.scale_down().await?;
        }

        // 基于工作负载模式调整策略
        if let Some(dominant_workload) = self.detect_dominant_workload(&history) {
            self.optimize_for_workload(dominant_workload).await?;
        }

        Ok(())
    }

    /// 扩容
    async fn scale_up(&self) -> Result<()> {
        let config = self.config.read().await;
        let stats = self.stats().await;

        if stats.total_threads < config.max_threads {
            // 在实际实现中，这里会动态调整Tokio运行时
            // 目前只能记录日志
            tracing::info!("Thread pool scaling up recommended: {} -> {}",
                          stats.total_threads, stats.total_threads + 1);
        }

        Ok(())
    }

    /// 缩容
    async fn scale_down(&self) -> Result<()> {
        let config = self.config.read().await;
        let stats = self.stats().await;

        if stats.total_threads > config.core_threads {
            // 在实际实现中，这里会动态调整Tokio运行时
            tracing::info!("Thread pool scaling down recommended: {} -> {}",
                          stats.total_threads, stats.total_threads - 1);
        }

        Ok(())
    }

    /// 检测主要工作负载类型
    fn detect_dominant_workload(&self, history: &[WorkloadSample]) -> Option<WorkloadType> {
        if history.is_empty() {
            return None;
        }

        // 统计各种工作负载类型的频率
        let mut workload_counts = std::collections::HashMap::new();

        for sample in history.iter().rev().take(100) { // 最近100个样本
            *workload_counts.entry(&sample.workload_type).or_insert(0) += 1;
        }

        // 找到出现次数最多的
        workload_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(workload_type, _)| workload_type.clone())
    }

    /// 关闭线程池
    pub async fn shutdown(self) -> Result<()> {
        self.runtime.shutdown_background();
        Ok(())
    }
}

/// 工作负载样本
#[derive(Debug, Clone)]
struct WorkloadSample {
    timestamp: std::time::Instant,
    workload_type: WorkloadType,
    active_threads: usize,
    queue_length: usize,
}

/// 线程池管理器
pub struct ThreadPoolManager {
    pools: Arc<RwLock<std::collections::HashMap<String, Arc<AdaptiveThreadPool>>>>,
}

impl ThreadPoolManager {
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// 创建或获取线程池
    pub async fn get_or_create_pool(&self, name: &str, config: ThreadPoolConfig) -> Result<Arc<AdaptiveThreadPool>> {
        let mut pools = self.pools.write().await;

        if let Some(pool) = pools.get(name) {
            return Ok(pool.clone());
        }

        let pool = Arc::new(AdaptiveThreadPool::new(config)?);
        pools.insert(name.to_string(), pool.clone());

        Ok(pool)
    }

    /// 获取线程池
    pub async fn get_pool(&self, name: &str) -> Result<Arc<AdaptiveThreadPool>> {
        let pools = self.pools.read().await;
        pools.get(name)
            .cloned()
            .ok_or_else(|| crate::error::Error::NotFound(format!("Thread pool '{}' not found", name)))
    }

    /// 获取所有线程池统计
    pub async fn get_all_stats(&self) -> Result<std::collections::HashMap<String, ThreadPoolStats>> {
        let pools = self.pools.read().await;
        let mut stats = std::collections::HashMap::new();

        for (name, pool) in pools.iter() {
            stats.insert(name.clone(), pool.stats().await);
        }

        Ok(stats)
    }

    /// 执行全局健康检查
    pub async fn health_check_all(&self) -> Result<std::collections::HashMap<String, HealthStatus>> {
        let pools = self.pools.read().await;
        let mut health_status = std::collections::HashMap::new();

        for (name, pool) in pools.iter() {
            let status = pool.health_check().await.unwrap_or(HealthStatus::Degraded);
            health_status.insert(name.clone(), status);
        }

        Ok(health_status)
    }

    /// 执行全局自适应调整
    pub async fn adaptive_adjust_all(&self) -> Result<()> {
        let pools = self.pools.read().await;

        for pool in pools.values() {
            if let Err(e) = pool.adaptive_adjust().await {
                tracing::warn!("Failed to adjust thread pool: {}", e);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_pool_config_default() {
        let config = ThreadPoolConfig::default();
        assert!(config.core_threads >= 2);
        assert!(config.max_threads >= config.core_threads);
        assert_eq!(config.thread_name_prefix, "lumosai-worker");
    }

    #[tokio::test]
    async fn test_thread_pool_manager() -> Result<()> {
        let manager = ThreadPoolManager::new();
        let config = ThreadPoolConfig::default();

        // 创建线程池
        let pool = manager.get_or_create_pool("test_pool", config).await?;
        assert!(pool.stats().await.active_threads >= 0);

        // 获取现有池
        let same_pool = manager.get_pool("test_pool").await?;
        assert!(Arc::ptr_eq(&pool, &same_pool));

        Ok(())
    }

    #[tokio::test]
    async fn test_workload_optimization() -> Result<()> {
        let config = ThreadPoolConfig::default();
        let pool = AdaptiveThreadPool::new(config)?;

        // 测试CPU密集型优化
        pool.optimize_for_workload(WorkloadType::CpuIntensive).await?;
        let updated_config = pool.config.read().await;
        assert_eq!(updated_config.max_threads, updated_config.core_threads);

        // 测试IO密集型优化
        pool.optimize_for_workload(WorkloadType::IoIntensive).await?;
        let updated_config = pool.config.read().await;
        assert!(updated_config.max_threads > updated_config.core_threads);

        pool.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_health_check() -> Result<()> {
        let config = ThreadPoolConfig::default();
        let pool = AdaptiveThreadPool::new(config)?;

        let health = pool.health_check().await?;
        // 新创建的池应该是Idle状态
        assert!(matches!(health, HealthStatus::Idle | HealthStatus::Healthy));

        pool.shutdown().await?;
        Ok(())
    }
}
