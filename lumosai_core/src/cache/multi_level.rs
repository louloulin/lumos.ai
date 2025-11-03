//! 多层缓存实现
//!
//! 架构：
//! - L1: 内存 LRU 缓存（最快，容量小）
//! - L2: Redis 缓存（快，容量中等，可选）
//! - L3: 持久化缓存（慢，容量大，可选）
//!
//! 特性：
//! - 自动回填（从 L2/L3 回填到 L1）
//! - 分层统计
//! - 缓存预热
//! - 智能失效

use super::{Cache, CacheConfig, CacheStats, LruCache};
use crate::error::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 多层缓存配置
#[derive(Debug, Clone)]
pub struct MultiLevelCacheConfig {
    /// L1 缓存配置
    pub l1_config: CacheConfig,
    /// 是否启用 L2 缓存
    pub enable_l2: bool,
    /// 是否启用 L3 缓存
    pub enable_l3: bool,
    /// 是否启用自动回填
    pub enable_backfill: bool,
}

impl Default for MultiLevelCacheConfig {
    fn default() -> Self {
        Self {
            l1_config: CacheConfig {
                max_entries: 1000,
                default_ttl: Duration::from_secs(300), // 5 minutes
                enable_lru: true,
                ..Default::default()
            },
            enable_l2: false, // Redis 默认禁用
            enable_l3: false, // 持久化默认禁用
            enable_backfill: true,
        }
    }
}

/// 多层缓存
pub struct MultiLevelCache {
    /// L1: 内存缓存
    l1: Arc<LruCache>,
    /// L2: Redis 缓存（可选）
    l2: Option<Arc<dyn Cache>>,
    /// L3: 持久化缓存（可选）
    l3: Option<Arc<dyn Cache>>,
    /// 配置
    config: MultiLevelCacheConfig,
    /// 统计信息
    stats: Arc<RwLock<CacheStats>>,
}

impl MultiLevelCache {
    /// 创建新的多层缓存
    pub fn new(config: MultiLevelCacheConfig) -> Self {
        let l1 = Arc::new(LruCache::new(config.l1_config.clone()));

        Self {
            l1,
            l2: None,
            l3: None,
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// 设置 L2 缓存
    pub fn with_l2(mut self, l2: Arc<dyn Cache>) -> Self {
        self.l2 = Some(l2);
        self
    }

    /// 设置 L3 缓存
    pub fn with_l3(mut self, l3: Arc<dyn Cache>) -> Self {
        self.l3 = Some(l3);
        self
    }

    /// 缓存预热
    pub async fn warmup(&self, keys: Vec<String>) -> Result<usize> {
        let mut warmed = 0;

        for key in keys {
            // 尝试从 L2/L3 加载到 L1
            if let Some(value) = self.get_from_lower_levels(&key).await {
                if self.l1.set(key.clone(), value, None).await.is_ok() {
                    warmed += 1;
                }
            }
        }

        tracing::info!("Cache warmup completed: {} keys loaded", warmed);
        Ok(warmed)
    }

    /// 从下层缓存获取
    async fn get_from_lower_levels(&self, key: &str) -> Option<Value> {
        // 尝试 L2
        if let Some(l2) = &self.l2 {
            if let Some(value) = l2.get(key).await {
                return Some(value);
            }
        }

        // 尝试 L3
        if let Some(l3) = &self.l3 {
            if let Some(value) = l3.get(key).await {
                return Some(value);
            }
        }

        None
    }

    /// 回填到上层缓存
    async fn backfill(&self, key: &str, value: &Value, level: usize) {
        if !self.config.enable_backfill {
            return;
        }

        // 回填到 L1
        if level > 1 {
            if let Err(e) = self.l1.set(key.to_string(), value.clone(), None).await {
                tracing::warn!("Failed to backfill L1 cache: {}", e);
            }
        }

        // 回填到 L2
        if level > 2 {
            if let Some(l2) = &self.l2 {
                if let Err(e) = l2.set(key.to_string(), value.clone(), None).await {
                    tracing::warn!("Failed to backfill L2 cache: {}", e);
                }
            }
        }
    }

    /// 获取 L1 缓存统计
    pub async fn l1_stats(&self) -> CacheStats {
        self.l1.stats().await
    }

    /// 清理过期条目
    pub async fn cleanup_expired(&self) {
        self.l1.cleanup_expired().await;
    }
}

#[async_trait::async_trait]
impl Cache for MultiLevelCache {
    async fn get(&self, key: &str) -> Option<Value> {
        let start = Instant::now();

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        drop(stats); // 释放锁

        // L1 查找
        if let Some(value) = self.l1.get(key).await {
            let mut stats = self.stats.write().await;
            stats.l1_hits += 1;
            stats.calculate_hit_rates();

            let elapsed = start.elapsed().as_micros() as f64;
            stats.avg_access_time_us =
                (stats.avg_access_time_us * (stats.total_requests - 1) as f64 + elapsed)
                    / stats.total_requests as f64;

            return Some(value);
        }

        // L2 查找
        if let Some(l2) = &self.l2 {
            if let Some(value) = l2.get(key).await {
                // 回填 L1
                self.backfill(key, &value, 2).await;

                let mut stats = self.stats.write().await;
                stats.l2_hits += 1;
                stats.calculate_hit_rates();

                let elapsed = start.elapsed().as_micros() as f64;
                stats.avg_access_time_us =
                    (stats.avg_access_time_us * (stats.total_requests - 1) as f64 + elapsed)
                        / stats.total_requests as f64;

                return Some(value);
            }
        }

        // L3 查找
        if let Some(l3) = &self.l3 {
            if let Some(value) = l3.get(key).await {
                // 回填 L1 和 L2
                self.backfill(key, &value, 3).await;

                let mut stats = self.stats.write().await;
                stats.l3_hits += 1;
                stats.calculate_hit_rates();

                let elapsed = start.elapsed().as_micros() as f64;
                stats.avg_access_time_us =
                    (stats.avg_access_time_us * (stats.total_requests - 1) as f64 + elapsed)
                        / stats.total_requests as f64;

                return Some(value);
            }
        }

        // 未命中
        let mut stats = self.stats.write().await;
        stats.misses += 1;
        stats.calculate_hit_rates();

        None
    }

    async fn set(&self, key: String, value: Value, ttl: Option<Duration>) -> Result<()> {
        // 写入所有层级
        self.l1.set(key.clone(), value.clone(), ttl).await?;

        if let Some(l2) = &self.l2 {
            if let Err(e) = l2.set(key.clone(), value.clone(), ttl).await {
                tracing::warn!("Failed to set L2 cache: {}", e);
            }
        }

        if let Some(l3) = &self.l3 {
            if let Err(e) = l3.set(key, value, ttl).await {
                tracing::warn!("Failed to set L3 cache: {}", e);
            }
        }

        Ok(())
    }

    async fn remove(&self, key: &str) -> Option<Value> {
        // 从所有层级删除
        let result = self.l1.remove(key).await;

        if let Some(l2) = &self.l2 {
            l2.remove(key).await;
        }

        if let Some(l3) = &self.l3 {
            l3.remove(key).await;
        }

        result
    }

    async fn clear(&self) -> Result<()> {
        self.l1.clear().await?;

        if let Some(l2) = &self.l2 {
            l2.clear().await?;
        }

        if let Some(l3) = &self.l3 {
            l3.clear().await?;
        }

        Ok(())
    }

    async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_multi_level_basic() {
        let config = MultiLevelCacheConfig::default();
        let cache = MultiLevelCache::new(config);

        cache
            .set("key1".to_string(), json!("value1"), None)
            .await
            .unwrap();

        assert_eq!(cache.get("key1").await, Some(json!("value1")));
    }

    #[tokio::test]
    async fn test_multi_level_stats() {
        let config = MultiLevelCacheConfig::default();
        let cache = MultiLevelCache::new(config);

        cache
            .set("key1".to_string(), json!("value1"), None)
            .await
            .unwrap();

        cache.get("key1").await; // L1 hit
        cache.get("key2").await; // miss

        let stats = cache.stats().await;
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.l1_hits, 1);
        assert_eq!(stats.misses, 1);
    }
}

