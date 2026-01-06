//! 智谱 AI 嵌入提供商
//!
//! 集成智谱 AI 的嵌入模型 API

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    embedding::provider::EmbeddingProvider,
    error::{RagError, Result},
};

/// 智谱 AI 嵌入请求
#[derive(Debug, Serialize)]
struct ZhipuEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

/// 智谱 AI 嵌入响应
#[derive(Debug, Deserialize)]
struct ZhipuEmbeddingResponse {
    data: Vec<EmbeddingData>,
    model: String,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
    object: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: usize,
    total_tokens: usize,
}

/// 智谱 AI 嵌入提供商
pub struct ZhipuEmbeddingProvider {
    api_key: String,
    model: String,
    base_url: String,
    client: Client,
}

impl ZhipuEmbeddingProvider {
    /// 创建新的智谱 AI 嵌入提供商
    ///
    /// # 参数
    /// - `api_key`: 智谱 AI API 密钥
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "embedding-2".to_string(),
            base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            client: Client::new(),
        }
    }

    /// 设置自定义模型
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// 设置自定义 API 基础 URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

#[async_trait]
impl EmbeddingProvider for ZhipuEmbeddingProvider {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| RagError::Embedding("No embedding returned".to_string()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let request = ZhipuEmbeddingRequest {
            model: self.model.clone(),
            input: texts.to_vec(),
        };

        let url = format!("{}/embeddings", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| RagError::Embedding(format!("HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(RagError::Embedding(format!(
                "API request failed: {error_text}"
            )));
        }

        let result: ZhipuEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| RagError::Embedding(format!("Failed to parse response: {e}")))?;

        // 按索引排序并提取嵌入向量
        let mut embeddings: Vec<_> = result.data.into_iter().collect();
        embeddings.sort_by_key(|d| d.index);

        Ok(embeddings.into_iter().map(|d| d.embedding).collect())
    }
}

/// 本地嵌入提供商（用于测试和离线场景）
///
/// 使用简单的 TF-IDF 或词袋模型生成嵌入
pub struct LocalEmbeddingProvider {
    dimension: usize,
}

impl LocalEmbeddingProvider {
    /// 创建新的本地嵌入提供商
    ///
    /// # 参数
    /// - `dimension`: 嵌入向量维度
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }

    /// 生成简单的词袋嵌入（Mock 实现）
    fn generate_bow_embedding(&self, text: &str) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let words: Vec<&str> = text.split_whitespace().collect();
        let mut embedding = vec![0.0; self.dimension];

        for word in words {
            // 使用哈希将单词映射到嵌入维度
            let mut hasher = DefaultHasher::new();
            word.hash(&mut hasher);
            let hash = hasher.finish();
            let index = (hash as usize) % self.dimension;

            // 增加对应维度的值
            embedding[index] += 1.0;
        }

        // 归一化
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        embedding
    }
}

#[async_trait]
impl EmbeddingProvider for LocalEmbeddingProvider {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        Ok(self.generate_bow_embedding(text))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|text| self.generate_bow_embedding(text))
            .collect())
    }
}

/// 缓存嵌入提供商
///
/// 包装其他提供商，添加缓存功能以减少 API 调用
pub struct CachedEmbeddingProvider {
    inner: Box<dyn EmbeddingProvider>,
    cache: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vec<f32>>>>,
    max_cache_size: usize,
}

impl CachedEmbeddingProvider {
    /// 创建新的缓存嵌入提供商
    ///
    /// # 参数
    /// - `inner`: 内部嵌入提供商
    /// - `max_cache_size`: 最大缓存条目数
    pub fn new(inner: Box<dyn EmbeddingProvider>, max_cache_size: usize) -> Self {
        Self {
            inner,
            cache: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            max_cache_size,
        }
    }

    /// 清空缓存
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }

    /// 获取缓存大小
    pub async fn cache_size(&self) -> usize {
        self.cache.read().await.len()
    }
}

#[async_trait]
impl EmbeddingProvider for CachedEmbeddingProvider {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(embedding) = cache.get(text) {
                return Ok(embedding.clone());
            }
        }

        // 生成新嵌入
        let embedding = self.inner.generate_embedding(text).await?;

        // 存入缓存
        {
            let mut cache = self.cache.write().await;

            // 如果缓存已满，移除最旧的条目（简单的 FIFO 策略）
            if cache.len() >= self.max_cache_size {
                if let Some(key) = cache.keys().next().cloned() {
                    cache.remove(&key);
                }
            }

            cache.insert(text.to_string(), embedding.clone());
        }

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        let mut uncached_texts = Vec::new();
        let mut uncached_indices = Vec::new();

        // 检查哪些文本已缓存
        {
            let cache = self.cache.read().await;
            for (i, text) in texts.iter().enumerate() {
                if let Some(embedding) = cache.get(text) {
                    results.push(Some(embedding.clone()));
                } else {
                    results.push(None);
                    uncached_texts.push(text.clone());
                    uncached_indices.push(i);
                }
            }
        }

        // 为未缓存的文本生成嵌入
        if !uncached_texts.is_empty() {
            let new_embeddings = self.inner.embed_batch(&uncached_texts).await?;

            // 更新缓存和结果
            {
                let mut cache = self.cache.write().await;
                for (text, embedding) in uncached_texts.iter().zip(new_embeddings.iter()) {
                    // 缓存管理
                    if cache.len() >= self.max_cache_size {
                        if let Some(key) = cache.keys().next().cloned() {
                            cache.remove(&key);
                        }
                    }
                    cache.insert(text.clone(), embedding.clone());
                }
            }

            // 填充结果
            for (idx, embedding) in uncached_indices.iter().zip(new_embeddings) {
                results[*idx] = Some(embedding);
            }
        }

        // 提取所有嵌入
        results
            .into_iter()
            .map(|opt| opt.ok_or_else(|| RagError::Embedding("Missing embedding".to_string())))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_embedding_provider() {
        let provider = LocalEmbeddingProvider::new(128);

        let text = "This is a test document";
        let embedding = provider.generate_embedding(text).await.unwrap();

        assert_eq!(embedding.len(), 128);

        // 验证归一化
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[tokio::test]
    async fn test_cached_embedding_provider() {
        let inner = Box::new(LocalEmbeddingProvider::new(64));
        let provider = CachedEmbeddingProvider::new(inner, 10);

        let text = "Test text";

        // 第一次调用
        let embedding1 = provider.generate_embedding(text).await.unwrap();
        assert_eq!(provider.cache_size().await, 1);

        // 第二次调用（应该从缓存获取）
        let embedding2 = provider.generate_embedding(text).await.unwrap();
        assert_eq!(embedding1, embedding2);
        assert_eq!(provider.cache_size().await, 1);
    }
}
