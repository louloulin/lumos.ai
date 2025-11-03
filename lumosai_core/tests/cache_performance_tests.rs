//! 缓存性能测试
//!
//! P0-2.2 任务：验证缓存机制优化效果
//!
//! 测试范围：
//! 1. LRU 缓存性能
//! 2. 多层缓存性能
//! 3. 缓存命中率
//! 4. 缓存策略效果
//! 5. 并发缓存访问

use lumosai_core::cache::{
    Cache, CacheConfig, CacheKeyGenerator, CacheStrategy, LlmCacheStrategy, LruCache,
    MultiLevelCache, MultiLevelCacheConfig, ToolCacheStrategy, VectorCacheStrategy,
};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

// ============================================================================
// LRU 缓存性能测试
// ============================================================================

#[tokio::test]
async fn test_lru_cache_write_performance() {
    let config = CacheConfig {
        max_entries: 10000,
        default_ttl: Duration::from_secs(3600),
        enable_lru: true,
        ..Default::default()
    };

    let cache = LruCache::new(config);
    let start = Instant::now();

    // 写入 1000 个条目
    for i in 0..1000 {
        cache
            .set(format!("key{}", i), json!({"value": i}), None)
            .await
            .unwrap();
    }

    let elapsed = start.elapsed();
    println!("✅ LRU cache write 1000 entries: {:?}", elapsed);

    assert!(elapsed.as_millis() < 100, "Write should be fast");
}

#[tokio::test]
async fn test_lru_cache_read_performance() {
    let config = CacheConfig {
        max_entries: 10000,
        default_ttl: Duration::from_secs(3600),
        enable_lru: true,
        ..Default::default()
    };

    let cache = LruCache::new(config);

    // 预填充数据
    for i in 0..1000 {
        cache
            .set(format!("key{}", i), json!({"value": i}), None)
            .await
            .unwrap();
    }

    let start = Instant::now();

    // 读取 1000 次
    for i in 0..1000 {
        cache.get(&format!("key{}", i)).await;
    }

    let elapsed = start.elapsed();
    println!("✅ LRU cache read 1000 entries: {:?}", elapsed);

    assert!(elapsed.as_millis() < 50, "Read should be very fast");
}

#[tokio::test]
async fn test_lru_cache_hit_rate() {
    let config = CacheConfig {
        max_entries: 100,
        default_ttl: Duration::from_secs(3600),
        enable_lru: true,
        ..Default::default()
    };

    let cache = LruCache::new(config);

    // 写入 50 个条目
    for i in 0..50 {
        cache
            .set(format!("key{}", i), json!({"value": i}), None)
            .await
            .unwrap();
    }

    // 50 次命中 + 50 次未命中
    for i in 0..100 {
        cache.get(&format!("key{}", i)).await;
    }

    let stats = cache.stats().await;
    println!("📊 Cache stats:");
    println!("   Total requests: {}", stats.total_requests);
    println!("   Hits: {}", stats.l1_hits);
    println!("   Misses: {}", stats.misses);
    println!("   Hit rate: {:.2}%", stats.hit_rate * 100.0);

    assert_eq!(stats.total_requests, 100);
    assert_eq!(stats.l1_hits, 50);
    assert_eq!(stats.misses, 50);
    assert_eq!(stats.hit_rate, 0.5);
}

// ============================================================================
// 多层缓存性能测试
// ============================================================================

#[tokio::test]
async fn test_multi_level_cache_l1_hit_performance() {
    let config = MultiLevelCacheConfig::default();
    let cache = MultiLevelCache::new(config);

    // 预填充 L1
    for i in 0..100 {
        cache
            .set(format!("key{}", i), json!({"value": i}), None)
            .await
            .unwrap();
    }

    let start = Instant::now();

    // 100 次 L1 命中
    for i in 0..100 {
        cache.get(&format!("key{}", i)).await;
    }

    let elapsed = start.elapsed();
    println!("✅ Multi-level cache L1 hit (100 times): {:?}", elapsed);

    let stats = cache.stats().await;
    assert_eq!(stats.l1_hits, 100);
    assert_eq!(stats.hit_rate, 1.0);
    assert!(elapsed.as_millis() < 50, "L1 hits should be very fast");
}

#[tokio::test]
async fn test_multi_level_cache_mixed_performance() {
    let config = MultiLevelCacheConfig::default();
    let cache = MultiLevelCache::new(config);

    // 预填充部分数据
    for i in 0..50 {
        cache
            .set(format!("key{}", i), json!({"value": i}), None)
            .await
            .unwrap();
    }

    let start = Instant::now();

    // 50 次命中 + 50 次未命中
    for i in 0..100 {
        cache.get(&format!("key{}", i)).await;
    }

    let elapsed = start.elapsed();
    println!("✅ Multi-level cache mixed access (100 times): {:?}", elapsed);

    let stats = cache.stats().await;
    println!("📊 Multi-level cache stats:");
    println!("   L1 hits: {}", stats.l1_hits);
    println!("   Misses: {}", stats.misses);
    println!("   Hit rate: {:.2}%", stats.hit_rate * 100.0);

    assert_eq!(stats.l1_hits, 50);
    assert_eq!(stats.misses, 50);
}

// ============================================================================
// 缓存策略性能测试
// ============================================================================

#[tokio::test]
async fn test_llm_cache_strategy_performance() {
    let cache = Arc::new(LruCache::new(CacheConfig::default()));
    let strategy = LlmCacheStrategy::new(cache);

    let start = Instant::now();

    // 模拟 100 次 LLM 调用（50 次重复）
    for i in 0..100 {
        let params = json!({
            "model": "gpt-4",
            "prompt": format!("Question {}", i % 50),
            "temperature": 0.7
        });

        if let Some(_cached) = strategy.get(&params).await {
            // 缓存命中
        } else {
            // 缓存未命中，模拟 LLM 调用并缓存
            let result = json!({"text": format!("Answer {}", i % 50)});
            strategy.set(&params, result).await.unwrap();
        }
    }

    let elapsed = start.elapsed();
    println!("✅ LLM cache strategy (100 calls, 50 unique): {:?}", elapsed);

    // 验证缓存效果
    let params = json!({
        "model": "gpt-4",
        "prompt": "Question 0",
        "temperature": 0.7
    });

    assert!(strategy.get(&params).await.is_some());
}

#[tokio::test]
async fn test_tool_cache_strategy_performance() {
    let cache = Arc::new(LruCache::new(CacheConfig::default()));
    let strategy = ToolCacheStrategy::new(cache);

    let start = Instant::now();

    // 模拟 100 次工具调用（50 次重复）
    for i in 0..100 {
        let params = json!({
            "tool_name": "calculator",
            "args": {"a": i % 50, "b": 2}
        });

        if let Some(_cached) = strategy.get(&params).await {
            // 缓存命中
        } else {
            // 缓存未命中，执行工具并缓存
            let result = json!({"result": (i % 50) + 2});
            strategy.set(&params, result).await.unwrap();
        }
    }

    let elapsed = start.elapsed();
    println!("✅ Tool cache strategy (100 calls, 50 unique): {:?}", elapsed);
}

// ============================================================================
// 并发缓存访问测试
// ============================================================================

#[tokio::test]
async fn test_concurrent_cache_access() {
    let cache = Arc::new(LruCache::new(CacheConfig {
        max_entries: 1000,
        default_ttl: Duration::from_secs(3600),
        enable_lru: true,
        ..Default::default()
    }));

    // 预填充数据
    for i in 0..100 {
        cache
            .set(format!("key{}", i), json!({"value": i}), None)
            .await
            .unwrap();
    }

    let start = Instant::now();

    // 10 个并发任务，每个读取 100 次
    let mut handles = Vec::new();
    for _ in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = tokio::spawn(async move {
            for i in 0..100 {
                cache_clone.get(&format!("key{}", i % 100)).await;
            }
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }

    let elapsed = start.elapsed();
    println!("✅ Concurrent cache access (10 tasks × 100 reads): {:?}", elapsed);

    let stats = cache.stats().await;
    println!("📊 Concurrent access stats:");
    println!("   Total requests: {}", stats.total_requests);
    println!("   Hits: {}", stats.l1_hits);
    println!("   Hit rate: {:.2}%", stats.hit_rate * 100.0);

    assert_eq!(stats.total_requests, 1000);
    assert!(stats.hit_rate > 0.9, "Hit rate should be > 90%");
}

// ============================================================================
// 缓存键生成性能测试
// ============================================================================

#[test]
fn test_cache_key_generation_performance() {
    let start = Instant::now();

    // 生成 10000 个缓存键
    for i in 0..10000 {
        let _key = CacheKeyGenerator::llm_key("gpt-4", &format!("prompt{}", i), 0.7);
    }

    let elapsed = start.elapsed();
    println!("✅ Generate 10000 cache keys: {:?}", elapsed);

    assert!(elapsed.as_millis() < 100, "Key generation should be fast");
}

