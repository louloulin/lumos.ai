//! 多层缓存系统
//!
//! P0-2.2 任务：实现企业级多层缓存架构
//!
//! 功能特性：
//! - L1: 内存 LRU 缓存（最快）
//! - L2: Redis 缓存（可选）
//! - L3: 持久化缓存（可选）
//! - 智能缓存策略（LLM 响应、向量嵌入、工具执行结果）
//! - 缓存预热和失效
//! - 详细的缓存统计和监控

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub mod lru;
pub mod multi_level;
pub mod strategies;

pub use lru::LruCache;
pub use multi_level::{MultiLevelCache, MultiLevelCacheConfig};
pub use strategies::{CacheStrategy, LlmCacheStrategy, ToolCacheStrategy, VectorCacheStrategy};

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 默认 TTL（生存时间）
    pub default_ttl: Duration,
    /// 是否启用 LRU 淘汰
    pub enable_lru: bool,
    /// 统计信息更新间隔
    pub stats_interval: Duration,
    /// 是否启用缓存预热
    pub enable_warmup: bool,
    /// 预热数据源
    pub warmup_keys: Vec<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl: Duration::from_secs(3600), // 1 hour
            enable_lru: true,
            stats_interval: Duration::from_secs(60),
            enable_warmup: false,
            warmup_keys: Vec::new(),
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// 总请求数
    pub total_requests: u64,
    /// L1 缓存命中数
    pub l1_hits: u64,
    /// L2 缓存命中数
    pub l2_hits: u64,
    /// L3 缓存命中数
    pub l3_hits: u64,
    /// 缓存未命中数
    pub misses: u64,
    /// 淘汰的条目数
    pub evictions: u64,
    /// 当前缓存大小
    pub current_size: usize,
    /// 总命中率
    pub hit_rate: f64,
    /// L1 命中率
    pub l1_hit_rate: f64,
    /// L2 命中率
    pub l2_hit_rate: f64,
    /// L3 命中率
    pub l3_hit_rate: f64,
    /// 平均访问时间（微秒）
    pub avg_access_time_us: f64,
}

impl CacheStats {
    /// 计算命中率
    pub fn calculate_hit_rates(&mut self) {
        if self.total_requests > 0 {
            let total_hits = self.l1_hits + self.l2_hits + self.l3_hits;
            self.hit_rate = total_hits as f64 / self.total_requests as f64;
            self.l1_hit_rate = self.l1_hits as f64 / self.total_requests as f64;
            self.l2_hit_rate = self.l2_hits as f64 / self.total_requests as f64;
            self.l3_hit_rate = self.l3_hits as f64 / self.total_requests as f64;
        }
    }

    /// 获取总命中数
    pub fn total_hits(&self) -> u64 {
        self.l1_hits + self.l2_hits + self.l3_hits
    }
}

/// 缓存条目
#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    /// 缓存值
    pub value: V,
    /// 创建时间
    pub created_at: Instant,
    /// 最后访问时间
    pub last_accessed: Instant,
    /// 访问计数
    pub access_count: u64,
    /// TTL
    pub ttl: Duration,
}

impl<V> CacheEntry<V> {
    /// 创建新的缓存条目
    pub fn new(value: V, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            ttl,
        }
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    /// 更新访问信息
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }
}

/// 缓存 trait
#[async_trait::async_trait]
pub trait Cache: Send + Sync {
    /// 获取缓存值
    async fn get(&self, key: &str) -> Option<Value>;

    /// 设置缓存值
    async fn set(&self, key: String, value: Value, ttl: Option<Duration>) -> Result<()>;

    /// 删除缓存条目
    async fn remove(&self, key: &str) -> Option<Value>;

    /// 清空缓存
    async fn clear(&self) -> Result<()>;

    /// 获取缓存统计信息
    async fn stats(&self) -> CacheStats;

    /// 检查键是否存在
    async fn contains(&self, key: &str) -> bool {
        self.get(key).await.is_some()
    }

    /// 批量获取
    async fn get_many(&self, keys: &[String]) -> HashMap<String, Value> {
        let mut results = HashMap::new();
        for key in keys {
            if let Some(value) = self.get(key).await {
                results.insert(key.clone(), value);
            }
        }
        results
    }

    /// 批量设置
    async fn set_many(&self, entries: HashMap<String, Value>, ttl: Option<Duration>) -> Result<()> {
        for (key, value) in entries {
            self.set(key, value, ttl).await?;
        }
        Ok(())
    }
}

/// 缓存键生成器
pub struct CacheKeyGenerator;

impl CacheKeyGenerator {
    /// 生成 LLM 缓存键
    pub fn llm_key(model: &str, prompt: &str, temperature: f32) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();
        hasher.write(model.as_bytes());
        hasher.write(prompt.as_bytes());
        hasher.write(&temperature.to_le_bytes());

        format!("llm:{}:{:x}", model, hasher.finish())
    }

    /// 生成向量嵌入缓存键
    pub fn embedding_key(model: &str, text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();
        hasher.write(model.as_bytes());
        hasher.write(text.as_bytes());

        format!("embedding:{}:{:x}", model, hasher.finish())
    }

    /// 生成工具执行缓存键
    pub fn tool_key(tool_name: &str, args: &Value) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();
        hasher.write(tool_name.as_bytes());
        if let Ok(args_str) = serde_json::to_string(args) {
            hasher.write(args_str.as_bytes());
        }

        format!("tool:{}:{:x}", tool_name, hasher.finish())
    }

    /// 生成向量检索缓存键
    pub fn search_key(index: &str, query: &[f32], top_k: usize) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();
        hasher.write(index.as_bytes());
        for &val in query {
            hasher.write(&val.to_le_bytes());
        }
        hasher.write(&top_k.to_le_bytes());

        format!("search:{}:{:x}", index, hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_expiration() {
        let entry = CacheEntry::new("test_value".to_string(), Duration::from_millis(100));
        assert!(!entry.is_expired());

        std::thread::sleep(Duration::from_millis(150));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_stats_calculation() {
        let mut stats = CacheStats {
            total_requests: 100,
            l1_hits: 60,
            l2_hits: 20,
            l3_hits: 10,
            misses: 10,
            ..Default::default()
        };

        stats.calculate_hit_rates();

        assert_eq!(stats.hit_rate, 0.9);
        assert_eq!(stats.l1_hit_rate, 0.6);
        assert_eq!(stats.l2_hit_rate, 0.2);
        assert_eq!(stats.l3_hit_rate, 0.1);
    }

    #[test]
    fn test_cache_key_generation() {
        let key1 = CacheKeyGenerator::llm_key("gpt-4", "Hello", 0.7);
        let key2 = CacheKeyGenerator::llm_key("gpt-4", "Hello", 0.7);
        let key3 = CacheKeyGenerator::llm_key("gpt-4", "World", 0.7);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}
