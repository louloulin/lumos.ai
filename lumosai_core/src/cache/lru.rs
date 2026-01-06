//! LRU (Least Recently Used) 缓存实现
//!
//! 特性：
//! - 基于访问时间的 LRU 淘汰策略
//! - 支持 TTL（生存时间）
//! - 线程安全（使用 RwLock）
//! - 详细的统计信息

use super::{Cache, CacheConfig, CacheEntry, CacheStats};
use crate::error::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// LRU 缓存
pub struct LruCache {
    config: CacheConfig,
    entries: Arc<RwLock<HashMap<String, CacheEntry<Value>>>>,
    stats: Arc<RwLock<CacheStats>>,
}

impl LruCache {
    /// 创建新的 LRU 缓存
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// 淘汰最久未使用的条目
    async fn evict_lru(&self, entries: &mut HashMap<String, CacheEntry<Value>>) {
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

    /// 清理过期条目
    pub async fn cleanup_expired(&self) {
        let mut entries = self.entries.write().await;
        let before_size = entries.len();

        entries.retain(|_, entry| !entry.is_expired());

        let after_size = entries.len();
        let removed = before_size - after_size;

        if removed > 0 {
            let mut stats = self.stats.write().await;
            stats.evictions += removed as u64;
            stats.current_size = after_size;

            tracing::debug!("Cleaned up {} expired cache entries", removed);
        }
    }

    /// 获取缓存大小
    pub async fn size(&self) -> usize {
        self.entries.read().await.len()
    }

    /// 获取缓存容量
    pub fn capacity(&self) -> usize {
        self.config.max_entries
    }
}

#[async_trait::async_trait]
impl Cache for LruCache {
    async fn get(&self, key: &str) -> Option<Value> {
        let start = Instant::now();

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        drop(stats); // 释放锁

        let mut entries = self.entries.write().await;

        let result = if let Some(entry) = entries.get_mut(key) {
            // 检查是否过期
            if entry.is_expired() {
                entries.remove(key);
                let current_size = entries.len();
                drop(entries); // 释放锁

                let mut stats = self.stats.write().await;
                stats.misses += 1;
                stats.current_size = current_size;
                stats.calculate_hit_rates();
                return None;
            }

            // 更新访问信息
            entry.touch();

            // 克隆值
            let value = entry.value.clone();
            let current_size = entries.len();
            drop(entries); // 释放锁

            let mut stats = self.stats.write().await;
            stats.l1_hits += 1;
            stats.current_size = current_size;
            stats.calculate_hit_rates();

            // 更新平均访问时间
            let elapsed = start.elapsed().as_micros() as f64;
            stats.avg_access_time_us =
                (stats.avg_access_time_us * (stats.total_requests - 1) as f64 + elapsed)
                    / stats.total_requests as f64;

            Some(value)
        } else {
            let current_size = entries.len();
            drop(entries); // 释放锁

            let mut stats = self.stats.write().await;
            stats.misses += 1;
            stats.current_size = current_size;
            stats.calculate_hit_rates();
            None
        };

        result
    }

    async fn set(&self, key: String, value: Value, ttl: Option<Duration>) -> Result<()> {
        let mut entries = self.entries.write().await;

        // 如果缓存已满且不包含该键，执行 LRU 淘汰
        if entries.len() >= self.config.max_entries && !entries.contains_key(&key) {
            if self.config.enable_lru {
                self.evict_lru(&mut entries).await;
            } else {
                // 如果不启用 LRU，拒绝插入
                return Err(crate::error::Error::CacheFull);
            }
        }

        let ttl = ttl.unwrap_or(self.config.default_ttl);
        let entry = CacheEntry::new(value, ttl);

        entries.insert(key, entry);

        let mut stats = self.stats.write().await;
        stats.current_size = entries.len();

        Ok(())
    }

    async fn remove(&self, key: &str) -> Option<Value> {
        let mut entries = self.entries.write().await;
        let result = entries.remove(key).map(|entry| entry.value);

        let mut stats = self.stats.write().await;
        stats.current_size = entries.len();

        result
    }

    async fn clear(&self) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.clear();

        let mut stats = self.stats.write().await;
        stats.current_size = 0;

        Ok(())
    }

    async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
}

impl Clone for LruCache {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            entries: Arc::clone(&self.entries),
            stats: Arc::clone(&self.stats),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_lru_basic_operations() {
        let config = CacheConfig {
            max_entries: 3,
            default_ttl: Duration::from_secs(60),
            enable_lru: true,
            ..Default::default()
        };

        let cache = LruCache::new(config);

        // 测试设置和获取
        cache
            .set("key1".to_string(), json!("value1"), None)
            .await
            .unwrap();
        assert_eq!(cache.get("key1").await, Some(json!("value1")));

        // 测试不存在的键
        assert_eq!(cache.get("nonexistent").await, None);
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let config = CacheConfig {
            max_entries: 2,
            default_ttl: Duration::from_secs(60),
            enable_lru: true,
            ..Default::default()
        };

        let cache = LruCache::new(config);

        // 插入两个条目
        cache
            .set("key1".to_string(), json!("value1"), None)
            .await
            .unwrap();
        cache
            .set("key2".to_string(), json!("value2"), None)
            .await
            .unwrap();

        // 访问 key1，使其成为最近使用的
        cache.get("key1").await;

        // 插入第三个条目，应该淘汰 key2
        cache
            .set("key3".to_string(), json!("value3"), None)
            .await
            .unwrap();

        assert_eq!(cache.get("key1").await, Some(json!("value1")));
        assert_eq!(cache.get("key2").await, None); // 被淘汰
        assert_eq!(cache.get("key3").await, Some(json!("value3")));
    }

    #[tokio::test]
    async fn test_lru_expiration() {
        let config = CacheConfig {
            max_entries: 10,
            default_ttl: Duration::from_millis(100),
            enable_lru: true,
            ..Default::default()
        };

        let cache = LruCache::new(config);

        cache
            .set("key1".to_string(), json!("value1"), None)
            .await
            .unwrap();

        // 立即获取应该成功
        assert_eq!(cache.get("key1").await, Some(json!("value1")));

        // 等待过期
        tokio::time::sleep(Duration::from_millis(150)).await;

        // 过期后应该返回 None
        assert_eq!(cache.get("key1").await, None);
    }

    #[tokio::test]
    async fn test_lru_stats() {
        let config = CacheConfig {
            max_entries: 10,
            default_ttl: Duration::from_secs(60),
            enable_lru: true,
            ..Default::default()
        };

        let cache = LruCache::new(config);

        cache
            .set("key1".to_string(), json!("value1"), None)
            .await
            .unwrap();

        cache.get("key1").await; // hit
        cache.get("key2").await; // miss

        let stats = cache.stats().await;
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.l1_hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }
}
