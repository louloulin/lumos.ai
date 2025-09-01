use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::error::{Error, Result};

/// 缓存条目
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub access_count: u64,
    pub ttl: Option<Duration>,
}

/// 缓存策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheEvictionPolicy {
    /// 最近最少使用
    LRU,
    /// 最近最少访问
    LFU,
    /// 先进先出
    FIFO,
    /// 基于TTL
    TTL,
    /// 自定义策略
    Custom(String),
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_size: usize,
    pub default_ttl: Option<Duration>,
    pub eviction_policy: CacheEvictionPolicy,
    pub cleanup_interval: Duration,
    pub enable_metrics: bool,
}

/// 缓存指标
#[derive(Debug, Clone, Default)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: usize,
    pub memory_usage: usize,
}

/// 高级缓存系统
pub struct AdvancedCache<T> {
    data: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    config: CacheConfig,
    metrics: Arc<RwLock<CacheMetrics>>,
    access_order: Arc<RwLock<Vec<String>>>,
}

/// 缓存trait
#[async_trait]
pub trait Cache<T>: Send + Sync {
    /// 获取缓存值
    async fn get(&self, key: &str) -> Option<T>;
    
    /// 设置缓存值
    async fn set(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<()>;
    
    /// 删除缓存值
    async fn remove(&self, key: &str) -> bool;
    
    /// 清空缓存
    async fn clear(&self) -> Result<()>;
    
    /// 获取缓存大小
    async fn size(&self) -> usize;
    
    /// 检查键是否存在
    async fn contains(&self, key: &str) -> bool;
    
    /// 获取缓存指标
    async fn metrics(&self) -> CacheMetrics;
}

impl<T: Clone + Send + Sync + 'static> AdvancedCache<T> {
    /// 创建新的高级缓存
    pub fn new(config: CacheConfig) -> Self {
        let cache = Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            config,
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
            access_order: Arc::new(RwLock::new(Vec::new())),
        };
        
        // 启动清理任务
        cache.start_cleanup_task();
        
        cache
    }
    
    /// 启动清理任务
    fn start_cleanup_task(&self) {
        let data = self.data.clone();
        let metrics = self.metrics.clone();
        let interval = self.config.cleanup_interval;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                if let Ok(mut cache) = data.write() {
                    let now = SystemTime::now();
                    let mut to_remove = Vec::new();
                    
                    for (key, entry) in cache.iter() {
                        if let Some(ttl) = entry.ttl {
                            if let Ok(elapsed) = now.duration_since(entry.created_at) {
                                if elapsed > ttl {
                                    to_remove.push(key.clone());
                                }
                            }
                        }
                    }
                    
                    for key in to_remove {
                        cache.remove(&key);
                        if let Ok(mut m) = metrics.write() {
                            m.evictions += 1;
                            m.size = cache.len();
                        }
                    }
                }
            }
        });
    }
    
    /// 应用驱逐策略
    fn apply_eviction_policy(&self) -> Result<()> {
        let mut data = self.data.write()
            .map_err(|e| Error::Lock(format!("Failed to lock cache data: {}", e)))?;
        
        if data.len() <= self.config.max_size {
            return Ok(());
        }
        
        let keys_to_remove = match self.config.eviction_policy {
            CacheEvictionPolicy::LRU => self.get_lru_keys(&data)?,
            CacheEvictionPolicy::LFU => self.get_lfu_keys(&data)?,
            CacheEvictionPolicy::FIFO => self.get_fifo_keys(&data)?,
            CacheEvictionPolicy::TTL => self.get_ttl_keys(&data)?,
            CacheEvictionPolicy::Custom(_) => {
                // 默认使用LRU
                self.get_lru_keys(&data)?
            }
        };
        
        for key in keys_to_remove {
            data.remove(&key);
            if let Ok(mut metrics) = self.metrics.write() {
                metrics.evictions += 1;
            }
        }
        
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.size = data.len();
        }
        
        Ok(())
    }
    
    /// 获取LRU键
    fn get_lru_keys(&self, data: &HashMap<String, CacheEntry<T>>) -> Result<Vec<String>> {
        let access_order = self.access_order.read()
            .map_err(|e| Error::Lock(format!("Failed to lock access order: {}", e)))?;
        
        let remove_count = data.len() - self.config.max_size + 1;
        Ok(access_order.iter().take(remove_count).cloned().collect())
    }
    
    /// 获取LFU键
    fn get_lfu_keys(&self, data: &HashMap<String, CacheEntry<T>>) -> Result<Vec<String>> {
        let mut entries: Vec<_> = data.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.access_count);
        
        let remove_count = data.len() - self.config.max_size + 1;
        Ok(entries.iter().take(remove_count).map(|(k, _)| (*k).clone()).collect())
    }
    
    /// 获取FIFO键
    fn get_fifo_keys(&self, data: &HashMap<String, CacheEntry<T>>) -> Result<Vec<String>> {
        let mut entries: Vec<_> = data.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.created_at);
        
        let remove_count = data.len() - self.config.max_size + 1;
        Ok(entries.iter().take(remove_count).map(|(k, _)| (*k).clone()).collect())
    }
    
    /// 获取TTL键
    fn get_ttl_keys(&self, data: &HashMap<String, CacheEntry<T>>) -> Result<Vec<String>> {
        let now = SystemTime::now();
        let mut expired_keys = Vec::new();
        
        for (key, entry) in data.iter() {
            if let Some(ttl) = entry.ttl {
                if let Ok(elapsed) = now.duration_since(entry.created_at) {
                    if elapsed > ttl {
                        expired_keys.push(key.clone());
                    }
                }
            }
        }
        
        if expired_keys.len() >= data.len() - self.config.max_size + 1 {
            Ok(expired_keys)
        } else {
            // 如果过期的键不够，使用LRU策略
            self.get_lru_keys(data)
        }
    }
    
    /// 更新访问顺序
    fn update_access_order(&self, key: &str) -> Result<()> {
        let mut access_order = self.access_order.write()
            .map_err(|e| Error::Lock(format!("Failed to lock access order: {}", e)))?;
        
        // 移除旧位置
        access_order.retain(|k| k != key);
        // 添加到末尾
        access_order.push(key.to_string());
        
        Ok(())
    }
    
    /// 计算缓存键的哈希值
    pub fn hash_key(key: &str) -> String {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[async_trait]
impl<T: Clone + Send + Sync + 'static> Cache<T> for AdvancedCache<T> {
    async fn get(&self, key: &str) -> Option<T> {
        let now = SystemTime::now();
        
        // 读取缓存
        if let Ok(mut data) = self.data.write() {
            if let Some(entry) = data.get_mut(key) {
                // 检查TTL
                if let Some(ttl) = entry.ttl {
                    if let Ok(elapsed) = now.duration_since(entry.created_at) {
                        if elapsed > ttl {
                            // 过期，移除并返回None
                            data.remove(key);
                            if let Ok(mut metrics) = self.metrics.write() {
                                metrics.misses += 1;
                                metrics.size = data.len();
                            }
                            return None;
                        }
                    }
                }
                
                // 更新访问信息
                entry.last_accessed = now;
                entry.access_count += 1;
                
                // 更新指标
                if let Ok(mut metrics) = self.metrics.write() {
                    metrics.hits += 1;
                }
                
                // 更新访问顺序
                let _ = self.update_access_order(key);
                
                return Some(entry.value.clone());
            }
        }
        
        // 缓存未命中
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.misses += 1;
        }
        
        None
    }
    
    async fn set(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<()> {
        let now = SystemTime::now();
        let effective_ttl = ttl.or(self.config.default_ttl);
        
        let entry = CacheEntry {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            ttl: effective_ttl,
        };
        
        // 写入缓存
        {
            let mut data = self.data.write()
                .map_err(|e| Error::Lock(format!("Failed to lock cache data: {}", e)))?;
            
            data.insert(key.to_string(), entry);
            
            if let Ok(mut metrics) = self.metrics.write() {
                metrics.size = data.len();
            }
        }
        
        // 更新访问顺序
        self.update_access_order(key)?;
        
        // 应用驱逐策略
        self.apply_eviction_policy()?;
        
        Ok(())
    }
    
    async fn remove(&self, key: &str) -> bool {
        if let Ok(mut data) = self.data.write() {
            let removed = data.remove(key).is_some();
            
            if removed {
                if let Ok(mut metrics) = self.metrics.write() {
                    metrics.size = data.len();
                }
                
                // 从访问顺序中移除
                if let Ok(mut access_order) = self.access_order.write() {
                    access_order.retain(|k| k != key);
                }
            }
            
            removed
        } else {
            false
        }
    }
    
    async fn clear(&self) -> Result<()> {
        let mut data = self.data.write()
            .map_err(|e| Error::Lock(format!("Failed to lock cache data: {}", e)))?;
        
        data.clear();
        
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.size = 0;
        }
        
        if let Ok(mut access_order) = self.access_order.write() {
            access_order.clear();
        }
        
        Ok(())
    }
    
    async fn size(&self) -> usize {
        if let Ok(data) = self.data.read() {
            data.len()
        } else {
            0
        }
    }
    
    async fn contains(&self, key: &str) -> bool {
        if let Ok(data) = self.data.read() {
            data.contains_key(key)
        } else {
            false
        }
    }
    
    async fn metrics(&self) -> CacheMetrics {
        if let Ok(metrics) = self.metrics.read() {
            metrics.clone()
        } else {
            CacheMetrics::default()
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            default_ttl: Some(Duration::from_secs(3600)), // 1小时
            eviction_policy: CacheEvictionPolicy::LRU,
            cleanup_interval: Duration::from_secs(300), // 5分钟
            enable_metrics: true,
        }
    }
}

/// 分布式缓存接口
#[async_trait]
pub trait DistributedCache<T>: Cache<T> {
    /// 同步缓存到其他节点
    async fn sync_to_nodes(&self, key: &str, value: &T) -> Result<()>;
    
    /// 从其他节点获取缓存
    async fn fetch_from_nodes(&self, key: &str) -> Option<T>;
    
    /// 使缓存失效
    async fn invalidate(&self, key: &str) -> Result<()>;
    
    /// 批量操作
    async fn batch_set(&self, items: Vec<(String, T, Option<Duration>)>) -> Result<()>;
    async fn batch_get(&self, keys: Vec<String>) -> Result<HashMap<String, T>>;
}

/// 缓存管理器
pub struct CacheManager {
    caches: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            caches: HashMap::new(),
        }
    }
    
    /// 注册缓存
    pub fn register_cache<T: Clone + Send + Sync + 'static>(
        &mut self,
        name: &str,
        cache: AdvancedCache<T>,
    ) {
        self.caches.insert(name.to_string(), Box::new(cache));
    }
    
    /// 获取缓存
    pub fn get_cache<T: Clone + Send + Sync + 'static>(&self, name: &str) -> Option<&AdvancedCache<T>> {
        self.caches.get(name)?.downcast_ref()
    }
    
    /// 列出所有缓存
    pub fn list_caches(&self) -> Vec<String> {
        self.caches.keys().cloned().collect()
    }
    
    /// 清空所有缓存
    pub async fn clear_all(&self) -> Result<()> {
        // 注意：这里需要类型擦除，实际实现会更复杂
        Ok(())
    }
}
