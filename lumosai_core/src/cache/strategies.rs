//! 智能缓存策略
//!
//! 提供针对不同场景的缓存策略：
//! - LLM 响应缓存
//! - 向量嵌入缓存
//! - 工具执行结果缓存
//! - 向量检索缓存

use super::{Cache, CacheKeyGenerator};
use crate::error::Result;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

/// 缓存策略 trait
#[async_trait::async_trait]
pub trait CacheStrategy: Send + Sync {
    /// 生成缓存键
    fn generate_key(&self, params: &Value) -> String;

    /// 获取 TTL
    fn get_ttl(&self, params: &Value) -> Duration;

    /// 判断是否应该缓存
    fn should_cache(&self, params: &Value, result: &Value) -> bool;

    /// 获取缓存值
    async fn get(&self, params: &Value) -> Option<Value>;

    /// 设置缓存值
    async fn set(&self, params: &Value, result: Value) -> Result<()>;
}

/// LLM 响应缓存策略
pub struct LlmCacheStrategy {
    cache: Arc<dyn Cache>,
    default_ttl: Duration,
}

impl LlmCacheStrategy {
    /// 创建新的 LLM 缓存策略
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self {
            cache,
            default_ttl: Duration::from_secs(3600), // 1 hour
        }
    }

    /// 设置默认 TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }
}

#[async_trait::async_trait]
impl CacheStrategy for LlmCacheStrategy {
    fn generate_key(&self, params: &Value) -> String {
        let model = params["model"].as_str().unwrap_or("unknown");
        let prompt = params["prompt"].as_str().unwrap_or("");
        let temperature = params["temperature"].as_f64().unwrap_or(0.7) as f32;

        CacheKeyGenerator::llm_key(model, prompt, temperature)
    }

    fn get_ttl(&self, _params: &Value) -> Duration {
        self.default_ttl
    }

    fn should_cache(&self, params: &Value, result: &Value) -> bool {
        // 不缓存流式响应
        if params.get("stream").and_then(|v| v.as_bool()).unwrap_or(false) {
            return false;
        }

        // 不缓存错误响应
        if result.get("error").is_some() {
            return false;
        }

        // 不缓存空响应
        if let Some(text) = result.get("text").and_then(|v| v.as_str()) {
            if text.is_empty() {
                return false;
            }
        }

        true
    }

    async fn get(&self, params: &Value) -> Option<Value> {
        let key = self.generate_key(params);
        self.cache.get(&key).await
    }

    async fn set(&self, params: &Value, result: Value) -> Result<()> {
        if !self.should_cache(params, &result) {
            return Ok(());
        }

        let key = self.generate_key(params);
        let ttl = self.get_ttl(params);

        self.cache.set(key, result, Some(ttl)).await
    }
}

/// 向量嵌入缓存策略
pub struct VectorCacheStrategy {
    cache: Arc<dyn Cache>,
    default_ttl: Duration,
}

impl VectorCacheStrategy {
    /// 创建新的向量嵌入缓存策略
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self {
            cache,
            default_ttl: Duration::from_secs(86400), // 24 hours
        }
    }

    /// 设置默认 TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }
}

#[async_trait::async_trait]
impl CacheStrategy for VectorCacheStrategy {
    fn generate_key(&self, params: &Value) -> String {
        let model = params["model"].as_str().unwrap_or("unknown");
        let text = params["text"].as_str().unwrap_or("");

        CacheKeyGenerator::embedding_key(model, text)
    }

    fn get_ttl(&self, _params: &Value) -> Duration {
        self.default_ttl
    }

    fn should_cache(&self, _params: &Value, result: &Value) -> bool {
        // 检查是否有有效的嵌入向量
        if let Some(embedding) = result.get("embedding").and_then(|v| v.as_array()) {
            !embedding.is_empty()
        } else {
            false
        }
    }

    async fn get(&self, params: &Value) -> Option<Value> {
        let key = self.generate_key(params);
        self.cache.get(&key).await
    }

    async fn set(&self, params: &Value, result: Value) -> Result<()> {
        if !self.should_cache(params, &result) {
            return Ok(());
        }

        let key = self.generate_key(params);
        let ttl = self.get_ttl(params);

        self.cache.set(key, result, Some(ttl)).await
    }
}

/// 工具执行结果缓存策略
pub struct ToolCacheStrategy {
    cache: Arc<dyn Cache>,
    default_ttl: Duration,
}

impl ToolCacheStrategy {
    /// 创建新的工具缓存策略
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self {
            cache,
            default_ttl: Duration::from_secs(1800), // 30 minutes
        }
    }

    /// 设置默认 TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }
}

#[async_trait::async_trait]
impl CacheStrategy for ToolCacheStrategy {
    fn generate_key(&self, params: &Value) -> String {
        let tool_name = params["tool_name"].as_str().unwrap_or("unknown");
        let args = params.get("args").unwrap_or(&Value::Null);

        CacheKeyGenerator::tool_key(tool_name, args)
    }

    fn get_ttl(&self, params: &Value) -> Duration {
        // 支持自定义 TTL
        if let Some(ttl_secs) = params.get("cache_ttl").and_then(|v| v.as_u64()) {
            Duration::from_secs(ttl_secs)
        } else {
            self.default_ttl
        }
    }

    fn should_cache(&self, params: &Value, result: &Value) -> bool {
        // 检查是否明确禁用缓存
        if params
            .get("no_cache")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            return false;
        }

        // 不缓存错误结果
        if result.get("error").is_some() {
            return false;
        }

        // 不缓存空结果
        if result.is_null() {
            return false;
        }

        true
    }

    async fn get(&self, params: &Value) -> Option<Value> {
        let key = self.generate_key(params);
        self.cache.get(&key).await
    }

    async fn set(&self, params: &Value, result: Value) -> Result<()> {
        if !self.should_cache(params, &result) {
            return Ok(());
        }

        let key = self.generate_key(params);
        let ttl = self.get_ttl(params);

        self.cache.set(key, result, Some(ttl)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::{CacheConfig, LruCache};
    use serde_json::json;

    #[tokio::test]
    async fn test_llm_cache_strategy() {
        let cache = Arc::new(LruCache::new(CacheConfig::default()));
        let strategy = LlmCacheStrategy::new(cache);

        let params = json!({
            "model": "gpt-4",
            "prompt": "Hello, world!",
            "temperature": 0.7
        });

        let result = json!({
            "text": "Hi there!"
        });

        // 设置缓存
        strategy.set(&params, result.clone()).await.unwrap();

        // 获取缓存
        let cached = strategy.get(&params).await;
        assert_eq!(cached, Some(result));
    }

    #[tokio::test]
    async fn test_llm_cache_should_not_cache_stream() {
        let cache = Arc::new(LruCache::new(CacheConfig::default()));
        let strategy = LlmCacheStrategy::new(cache);

        let params = json!({
            "model": "gpt-4",
            "prompt": "Hello",
            "stream": true
        });

        let result = json!({"text": "Hi"});

        assert!(!strategy.should_cache(&params, &result));
    }

    #[tokio::test]
    async fn test_tool_cache_strategy() {
        let cache = Arc::new(LruCache::new(CacheConfig::default()));
        let strategy = ToolCacheStrategy::new(cache);

        let params = json!({
            "tool_name": "calculator",
            "args": {"a": 1, "b": 2}
        });

        let result = json!({"result": 3});

        strategy.set(&params, result.clone()).await.unwrap();

        let cached = strategy.get(&params).await;
        assert_eq!(cached, Some(result));
    }
}

