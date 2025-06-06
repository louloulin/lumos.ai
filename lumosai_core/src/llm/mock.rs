//! Mock LLM Provider for testing

use std::sync::Mutex;
use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use futures::StreamExt;
// Removed unused imports:
// use serde_json::Value;
// use std::sync::Arc;
use float_cmp::approx_eq;

use crate::error::{Error, Result};
use crate::llm::{LlmProvider, LlmOptions, Message};

/// Mock LLM provider for testing
pub struct MockLlmProvider {
    /// Generated responses for generate method
    responses: Mutex<Vec<String>>,
    /// Generated embeddings for get_embedding method
    embeddings: Mutex<Vec<Vec<f32>>>,
}

impl MockLlmProvider {
    /// Create a new mock LLM provider with predefined responses
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses: Mutex::new(responses),
            embeddings: Mutex::new(vec![vec![0.1, 0.2, 0.3]]), // Default embedding
        }
    }
    
    /// Create a mock LLM provider with predefined embeddings
    pub fn new_with_embeddings(embeddings: Vec<Vec<f32>>) -> Self {
        Self {
            responses: Mutex::new(vec!["This is a mock response".to_string()]),
            embeddings: Mutex::new(embeddings),
        }
    }
    
    /// Create a mock LLM provider with embeddings that increase in value based on input index
    /// 
    /// This is useful for creating a sequence of test embeddings where each has a specific pattern
    /// for example: [[0.1, 0.2, 0.3], [0.2, 0.3, 0.4], [0.3, 0.4, 0.5]]
    pub fn new_with_sequential_embeddings(start_value: f32, step: f32, dims: usize, count: usize) -> Self {
        let mut embeddings = Vec::with_capacity(count);
        
        for i in 0..count {
            let base_value = start_value + (i as f32 * step);
            let embedding = (0..dims).map(|j| base_value + (j as f32 * 0.01)).collect();
            embeddings.push(embedding);
        }
        
        Self {
            responses: Mutex::new(vec!["This is a mock response".to_string()]),
            embeddings: Mutex::new(embeddings),
        }
    }
    
    /// Add a response to the mock
    pub fn add_response(&self, response: String) {
        let mut responses = self.responses.lock().unwrap();
        responses.push(response);
    }
    
    /// Add an embedding to the mock
    pub fn add_embedding(&self, embedding: Vec<f32>) {
        let mut embeddings = self.embeddings.lock().unwrap();
        embeddings.push(embedding);
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    fn name(&self) -> &str {
        "mock"
    }

    async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> Result<String> {
        let mut responses = self.responses.lock().unwrap();
        
        if responses.is_empty() {
            Ok("Default mock response".to_string())
        } else {
            // 总是移除并返回第一个响应
            Ok(responses.remove(0))
        }
    }
    
    async fn generate_with_messages(&self, _messages: &[Message], _options: &LlmOptions) -> Result<String> {
        // 复用generate方法
        self.generate("", _options).await
    }
    
    async fn generate_stream<'a>(
        &'a self,
        _prompt: &'a str,
        _options: &'a LlmOptions,
    ) -> Result<BoxStream<'a, Result<String>>> {
        let mut responses = self.responses.lock().unwrap();
        
        let response = if responses.is_empty() {
            "Default mock response".to_string()
        } else {
            responses.remove(0)
        };
        
        // 将响应分成多个块以模拟流式传输
        let chunks = response
            .chars()
            .collect::<Vec<_>>()
            .chunks(5)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<_>>();
        
        let stream = stream::iter(chunks)
            .map(Ok)
            .boxed();
        
        Ok(stream)
    }
    
    async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        let mut embeddings = self.embeddings.lock().unwrap();

        if embeddings.is_empty() {
            Err(Error::Unavailable("No embeddings available".to_string()))
        } else {
            // 总是移除并返回第一个嵌入向量
            Ok(embeddings.remove(0))
        }
    }

    fn supports_function_calling(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const FLOAT_EPSILON: f32 = 1e-6;
    
    #[tokio::test]
    async fn test_mock_llm_generate() {
        // 初始化时提供两个响应
        let mock = MockLlmProvider::new(vec![
            "First response".to_string(),
            "Second response".to_string(),
        ]);
        
        let options = LlmOptions::default();
        
        // 第一次调用，获取第一个响应并删除它
        let response1 = mock.generate("test", &options).await.unwrap();
        assert_eq!(response1, "First response");
        
        // 第二次调用，获取第二个响应并删除它
        let response2 = mock.generate("test", &options).await.unwrap();
        assert_eq!(response2, "Second response");
        
        // 第三次调用，没有更多响应，返回默认响应
        let response3 = mock.generate("test", &options).await.unwrap();
        assert_eq!(response3, "Default mock response");
    }
    
    #[tokio::test]
    async fn test_mock_llm_embedding() {
        // 初始化时提供两个嵌入向量
        let mock = MockLlmProvider::new_with_embeddings(vec![
            vec![0.1, 0.2, 0.3],
            vec![0.4, 0.5, 0.6],
        ]);
        
        // 第一次调用，获取第一个嵌入向量并删除它
        let embedding1 = mock.get_embedding("test").await.unwrap();
        assert_eq!(embedding1, vec![0.1, 0.2, 0.3]);
        
        // 第二次调用，获取第二个嵌入向量并删除它
        let embedding2 = mock.get_embedding("test").await.unwrap();
        assert_eq!(embedding2, vec![0.4, 0.5, 0.6]);
        
        // 第三次调用，没有更多嵌入向量，应返回错误
        let result = mock.get_embedding("test").await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_sequential_embeddings() {
        // 创建3个顺序增加的嵌入向量，每个有4个维度
        let mock = MockLlmProvider::new_with_sequential_embeddings(0.1, 0.2, 4, 3);
        
        // 获取并验证第一个嵌入向量
        let embedding1 = mock.get_embedding("test").await.unwrap();
        assert_approx_vectors(&embedding1, &[0.1, 0.11, 0.12, 0.13]);
        
        // 获取并验证第二个嵌入向量
        let embedding2 = mock.get_embedding("test").await.unwrap();
        assert_approx_vectors(&embedding2, &[0.3, 0.31, 0.32, 0.33]);
        
        // 获取并验证第三个嵌入向量
        let embedding3 = mock.get_embedding("test").await.unwrap();
        assert_approx_vectors(&embedding3, &[0.5, 0.51, 0.52, 0.53]);
        
        // 确认没有更多的嵌入向量
        let result = mock.get_embedding("test").await;
        assert!(result.is_err());
    }
    
    // 辅助函数：验证向量元素在误差范围内相等
    fn assert_approx_vectors(actual: &[f32], expected: &[f32]) {
        assert_eq!(actual.len(), expected.len(), "向量长度不同");
        
        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert!(
                approx_eq!(f32, *a, *e, epsilon = FLOAT_EPSILON),
                "向量元素 {} 不相等: {} != {}", i, a, e
            );
        }
    }
} 