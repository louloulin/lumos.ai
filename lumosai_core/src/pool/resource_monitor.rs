//! Resource Monitor
//!
//! 资源监控模块，提供实时资源使用监控和自动扩缩容

use super::PoolStats;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;

/// 资源统计信息
#[derive(Debug, Clone)]
pub struct ResourceStats {
    /// CPU 使用率（0.0 - 1.0）
    pub cpu_usage: f64,
    /// 内存使用量（字节）
    pub memory_usage: u64,
    /// 内存使用率（0.0 - 1.0）
    pub memory_usage_percent: f64,
    /// 活跃连接数
    pub active_connections: usize,
    /// 空闲连接数
    pub idle_connections: usize,
    /// 总连接数
    pub total_connections: usize,
    /// 连接池利用率（0.0 - 1.0）
    pub pool_utilization: f64,
    /// 请求队列长度
    pub queue_length: usize,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    /// 错误率（0.0 - 1.0）
    pub error_rate: f64,
    /// 采样时间
    pub sampled_at: Instant,
}

impl Default for ResourceStats {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0,
            memory_usage_percent: 0.0,
            active_connections: 0,
            idle_connections: 0,
            total_connections: 0,
            pool_utilization: 0.0,
            queue_length: 0,
            avg_response_time_ms: 0.0,
            error_rate: 0.0,
            sampled_at: Instant::now(),
        }
    }
}

impl ResourceStats {
    /// 是否需要扩容
    pub fn should_scale_up(&self) -> bool {
        // 多个条件触发扩容
        self.pool_utilization > 0.85 || 
        self.queue_length > 10 ||
        self.cpu_usage > 0.8 ||
        self.memory_usage_percent > 0.85
    }
    
    /// 是否需要缩容
    pub fn should_scale_down(&self) -> bool {
        // 资源利用率低且没有排队请求
        self.pool_utilization < 0.3 && 
        self.queue_length == 0 &&
        self.cpu_usage < 0.3 &&
        self.memory_usage_percent < 0.5
    }
    
    /// 建议的扩容数量
    pub fn suggested_scale_up_count(&self) -> usize {
        if self.queue_length > 20 {
            5
        } else if self.queue_length > 10 {
            3
        } else if self.pool_utilization > 0.9 {
            2
        } else {
            1
        }
    }
    
    /// 建议的缩容数量
    pub fn suggested_scale_down_count(&self) -> usize {
        let idle = self.idle_connections;
        if idle > 10 {
            idle / 2
        } else if idle > 5 {
            2
        } else {
            1
        }
    }
}

/// 资源监控器配置
#[derive(Debug, Clone)]
pub struct ResourceMonitorConfig {
    /// 监控间隔
    pub monitor_interval: Duration,
    /// 扩容阈值（利用率）
    pub scale_up_threshold: f64,
    /// 缩容阈值（利用率）
    pub scale_down_threshold: f64,
    /// 是否启用自动扩缩容
    pub auto_scaling_enabled: bool,
    /// 最小实例数
    pub min_instances: usize,
    /// 最大实例数
    pub max_instances: usize,
    /// 扩容冷却时间
    pub scale_up_cooldown: Duration,
    /// 缩容冷却时间
    pub scale_down_cooldown: Duration,
}

impl Default for ResourceMonitorConfig {
    fn default() -> Self {
        Self {
            monitor_interval: Duration::from_secs(10),
            scale_up_threshold: 0.85,
            scale_down_threshold: 0.3,
            auto_scaling_enabled: true,
            min_instances: 5,
            max_instances: 50,
            scale_up_cooldown: Duration::from_secs(60),
            scale_down_cooldown: Duration::from_secs(300),
        }
    }
}

/// 扩缩容事件
#[derive(Debug, Clone)]
pub enum ScalingEvent {
    /// 扩容事件
    ScaleUp {
        /// 当前实例数
        current: usize,
        /// 目标实例数
        target: usize,
        /// 原因
        reason: String,
    },
    /// 缩容事件
    ScaleDown {
        /// 当前实例数
        current: usize,
        /// 目标实例数
        target: usize,
        /// 原因
        reason: String,
    },
}

/// 资源监控器
pub struct ResourceMonitor {
    config: ResourceMonitorConfig,
    stats: Arc<RwLock<ResourceStats>>,
    last_scale_up: Arc<RwLock<Option<Instant>>>,
    last_scale_down: Arc<RwLock<Option<Instant>>>,
}

impl ResourceMonitor {
    /// 创建新的资源监控器
    pub fn new(config: ResourceMonitorConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(ResourceStats::default())),
            last_scale_up: Arc::new(RwLock::new(None)),
            last_scale_down: Arc::new(RwLock::new(None)),
        }
    }
    
    /// 更新资源统计
    pub async fn update_stats(&self, stats: ResourceStats) {
        let mut current_stats = self.stats.write().await;
        *current_stats = stats;
    }
    
    /// 更新池统计
    pub async fn update_pool_stats(&self, pool_stats: &PoolStats) {
        let mut stats = self.stats.write().await;
        stats.active_connections = pool_stats.active;
        stats.idle_connections = pool_stats.idle;
        stats.total_connections = pool_stats.total;
        stats.pool_utilization = pool_stats.utilization;
        stats.queue_length = pool_stats.waiting;
        stats.avg_response_time_ms = pool_stats.avg_acquire_time_ms;
        stats.sampled_at = Instant::now();
    }
    
    /// 获取当前资源统计
    pub async fn get_stats(&self) -> ResourceStats {
        self.stats.read().await.clone()
    }
    
    /// 检查是否需要扩缩容
    pub async fn check_scaling(&self) -> Option<ScalingEvent> {
        if !self.config.auto_scaling_enabled {
            return None;
        }
        
        let stats = self.stats.read().await;
        let current = stats.total_connections;
        
        // 检查扩容
        if stats.should_scale_up() {
            // 检查冷却时间
            let last_scale_up = self.last_scale_up.read().await;
            if let Some(last) = *last_scale_up {
                if last.elapsed() < self.config.scale_up_cooldown {
                    return None;
                }
            }
            drop(last_scale_up);
            
            let count = stats.suggested_scale_up_count();
            let target = (current + count).min(self.config.max_instances);
            
            if target > current {
                // 更新最后扩容时间
                let mut last_scale_up = self.last_scale_up.write().await;
                *last_scale_up = Some(Instant::now());
                
                return Some(ScalingEvent::ScaleUp {
                    current,
                    target,
                    reason: format!(
                        "High utilization: pool={:.2}%, queue={}, cpu={:.2}%",
                        stats.pool_utilization * 100.0,
                        stats.queue_length,
                        stats.cpu_usage * 100.0
                    ),
                });
            }
        }
        
        // 检查缩容
        if stats.should_scale_down() {
            // 检查冷却时间
            let last_scale_down = self.last_scale_down.read().await;
            if let Some(last) = *last_scale_down {
                if last.elapsed() < self.config.scale_down_cooldown {
                    return None;
                }
            }
            drop(last_scale_down);
            
            let count = stats.suggested_scale_down_count();
            let target = (current.saturating_sub(count)).max(self.config.min_instances);
            
            if target < current {
                // 更新最后缩容时间
                let mut last_scale_down = self.last_scale_down.write().await;
                *last_scale_down = Some(Instant::now());
                
                return Some(ScalingEvent::ScaleDown {
                    current,
                    target,
                    reason: format!(
                        "Low utilization: pool={:.2}%, idle={}, cpu={:.2}%",
                        stats.pool_utilization * 100.0,
                        stats.idle_connections,
                        stats.cpu_usage * 100.0
                    ),
                });
            }
        }
        
        None
    }
    
    /// 启动监控循环
    pub async fn start_monitoring<F>(&self, mut callback: F)
    where
        F: FnMut(ScalingEvent) + Send + 'static,
    {
        let mut ticker = interval(self.config.monitor_interval);
        
        loop {
            ticker.tick().await;
            
            if let Some(event) = self.check_scaling().await {
                callback(event);
            }
        }
    }
}

impl Clone for ResourceMonitor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            stats: self.stats.clone(),
            last_scale_up: self.last_scale_up.clone(),
            last_scale_down: self.last_scale_down.clone(),
        }
    }
}

