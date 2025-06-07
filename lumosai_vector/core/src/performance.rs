//! 性能优化模块
//! 
//! 提供连接池、缓存机制和性能监控功能

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use serde::{Deserialize, Serialize};

use crate::{Result, VectorError};

/// 连接池配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// 最大连接数
    pub max_connections: usize,
    /// 最小连接数
    pub min_connections: usize,
    /// 连接超时时间
    pub connection_timeout: Duration,
    /// 空闲连接超时时间
    pub idle_timeout: Duration,
    /// 连接重试次数
    pub max_retries: u32,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            max_retries: 3,
        }
    }
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 缓存过期时间
    pub ttl: Duration,
    /// 是否启用LRU淘汰策略
    pub enable_lru: bool,
    /// 缓存命中率统计间隔
    pub stats_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl: Duration::from_secs(3600),
            enable_lru: true,
            stats_interval: Duration::from_secs(60),
        }
    }
}

/// 连接池
pub struct ConnectionPool<T> {
    config: ConnectionPoolConfig,
    connections: Arc<RwLock<Vec<PooledConnection<T>>>>,
    semaphore: Arc<Semaphore>,
    stats: Arc<RwLock<ConnectionPoolStats>>,
}

/// 池化连接
pub struct PooledConnection<T> {
    connection: T,
    created_at: Instant,
    last_used: Instant,
    is_active: bool,
}

/// 连接池统计信息
#[derive(Debug, Default, Clone)]
pub struct ConnectionPoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_wait_time: Duration,
}

impl<T> ConnectionPool<T> {
    /// 创建新的连接池
    pub fn new(config: ConnectionPoolConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_connections));
        
        Self {
            config,
            connections: Arc::new(RwLock::new(Vec::new())),
            semaphore,
            stats: Arc::new(RwLock::new(ConnectionPoolStats::default())),
        }
    }
    
    /// 获取连接
    pub async fn get_connection(&self) -> Result<PooledConnection<T>>
    where
        T: Clone,
    {
        let start_time = Instant::now();
        
        // 等待可用连接槽位
        let _permit = self.semaphore.acquire().await
            .map_err(|e| VectorError::ConnectionFailed(format!("Failed to acquire connection: {}", e)))?;
        
        let mut connections = self.connections.write().await;
        
        // 查找可用连接
        if let Some(pos) = connections.iter().position(|conn| !conn.is_active) {
            let mut conn = connections.remove(pos);
            conn.is_active = true;
            conn.last_used = Instant::now();
            
            // 更新统计信息
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            stats.successful_requests += 1;
            stats.average_wait_time = start_time.elapsed();
            
            return Ok(conn);
        }
        
        // 如果没有可用连接且未达到最大连接数，创建新连接
        if connections.len() < self.config.max_connections {
            // 这里需要实际的连接创建逻辑，暂时返回错误
            return Err(VectorError::ConnectionFailed("Connection creation not implemented".to_string()));
        }

        Err(VectorError::ConnectionFailed("No available connections".to_string()))
    }
    
    /// 归还连接
    pub async fn return_connection(&self, mut connection: PooledConnection<T>) {
        connection.is_active = false;
        connection.last_used = Instant::now();
        
        let mut connections = self.connections.write().await;
        connections.push(connection);
    }
    
    /// 获取连接池统计信息
    pub async fn get_stats(&self) -> ConnectionPoolStats {
        self.stats.read().await.clone()
    }
    
    /// 清理过期连接
    pub async fn cleanup_expired_connections(&self) {
        let mut connections = self.connections.write().await;
        let now = Instant::now();
        
        connections.retain(|conn| {
            !conn.is_active && now.duration_since(conn.last_used) < self.config.idle_timeout
        });
    }
}

/// 缓存条目
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
}

/// LRU缓存
pub struct LRUCache<K, V> {
    config: CacheConfig,
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    stats: Arc<RwLock<CacheStats>>,
}

/// 缓存统计信息
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
    pub current_size: usize,
    pub hit_rate: f64,
}

impl<K, V> LRUCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    /// 创建新的LRU缓存
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }
    
    /// 获取缓存值
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.get_mut(key) {
            let now = Instant::now();
            
            // 检查是否过期
            if now.duration_since(entry.created_at) > self.config.ttl {
                entries.remove(key);
                stats.cache_misses += 1;
                stats.current_size = entries.len();
                return None;
            }
            
            // 更新访问信息
            entry.last_accessed = now;
            entry.access_count += 1;
            
            stats.cache_hits += 1;
            stats.hit_rate = stats.cache_hits as f64 / stats.total_requests as f64;
            
            Some(entry.value.clone())
        } else {
            stats.cache_misses += 1;
            stats.hit_rate = stats.cache_hits as f64 / stats.total_requests as f64;
            None
        }
    }
    
    /// 设置缓存值
    pub async fn set(&self, key: K, value: V) {
        let mut entries = self.entries.write().await;
        let now = Instant::now();
        
        // 如果缓存已满，执行LRU淘汰
        if entries.len() >= self.config.max_entries && !entries.contains_key(&key) {
            self.evict_lru(&mut entries).await;
        }
        
        let entry = CacheEntry {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
        };
        
        entries.insert(key, entry);
        
        let mut stats = self.stats.write().await;
        stats.current_size = entries.len();
    }
    
    /// 删除缓存条目
    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().await;
        let result = entries.remove(key).map(|entry| entry.value);
        
        let mut stats = self.stats.write().await;
        stats.current_size = entries.len();
        
        result
    }
    
    /// 清空缓存
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
        
        let mut stats = self.stats.write().await;
        stats.current_size = 0;
    }
    
    /// 获取缓存统计信息
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
    
    /// LRU淘汰策略
    async fn evict_lru(&self, entries: &mut HashMap<K, CacheEntry<V>>) {
        if entries.is_empty() {
            return;
        }
        
        // 找到最久未访问的条目
        let mut oldest_key = None;
        let mut oldest_time = Instant::now();
        
        for (key, entry) in entries.iter() {
            if entry.last_accessed < oldest_time {
                oldest_time = entry.last_accessed;
                oldest_key = Some(key.clone());
            }
        }
        
        if let Some(key) = oldest_key {
            entries.remove(&key);
            
            let mut stats = self.stats.write().await;
            stats.evictions += 1;
        }
    }
}

/// 性能监控器
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PerformanceMetrics>>,
}

/// 性能指标
#[derive(Debug, Default, Clone)]
pub struct PerformanceMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub average_response_time: Duration,
    pub min_response_time: Duration,
    pub max_response_time: Duration,
    pub operations_per_second: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        }
    }
    
    /// 记录操作
    pub async fn record_operation(&self, duration: Duration, success: bool) {
        let mut metrics = self.metrics.write().await;
        
        metrics.total_operations += 1;
        
        if success {
            metrics.successful_operations += 1;
        } else {
            metrics.failed_operations += 1;
        }
        
        // 更新响应时间统计
        if metrics.total_operations == 1 {
            metrics.min_response_time = duration;
            metrics.max_response_time = duration;
            metrics.average_response_time = duration;
        } else {
            if duration < metrics.min_response_time {
                metrics.min_response_time = duration;
            }
            if duration > metrics.max_response_time {
                metrics.max_response_time = duration;
            }
            
            // 计算移动平均
            let total_time = metrics.average_response_time.as_nanos() as f64 * (metrics.total_operations - 1) as f64;
            metrics.average_response_time = Duration::from_nanos(
                ((total_time + duration.as_nanos() as f64) / metrics.total_operations as f64) as u64
            );
        }
    }
    
    /// 获取性能指标
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }
    
    /// 重置指标
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = PerformanceMetrics::default();
    }
}
