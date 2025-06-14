//! 统一API入口
//! 
//! 提供简化的API接口，让开发者能够快速上手LumosAI。
//! 这个模块参考了Rig和Mastra的设计理念，提供了简洁直观的API。

use std::sync::Arc;
use crate::error::Result;
// use crate::agent::{AgentTrait, BasicAgent, AgentConfig};

/// LLM提供商便利函数
pub mod llm {
    use super::*;
    use crate::llm::*;

    /// 创建OpenAI提供商
    pub fn openai(model: &str) -> Result<Arc<dyn LlmProvider>> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| crate::error::LumosError::ConfigError {
                message: "OPENAI_API_KEY environment variable not found".to_string(),
            })?;
        let provider = OpenAiProvider::new(api_key, model.to_string());
        Ok(Arc::new(provider))
    }

    /// 创建Anthropic提供商
    pub fn anthropic(model: &str) -> Result<Arc<dyn LlmProvider>> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| crate::error::LumosError::ConfigError {
                message: "ANTHROPIC_API_KEY environment variable not found".to_string(),
            })?;
        let provider = AnthropicProvider::new(api_key, model.to_string());
        Ok(Arc::new(provider))
    }

    /// 创建Claude提供商
    pub fn claude(model: &str) -> Result<Arc<dyn LlmProvider>> {
        let provider = ClaudeProvider::from_env()?;
        Ok(Arc::new(provider))
    }

    /// 创建Qwen提供商
    pub fn qwen(model: &str) -> Result<Arc<dyn LlmProvider>> {
        let api_key = std::env::var("QWEN_API_KEY")
            .map_err(|_| crate::error::LumosError::ConfigError {
                message: "QWEN_API_KEY environment variable not found".to_string(),
            })?;
        let provider = QwenProvider::new(api_key, model.to_string(), "https://api.qwen.com".to_string());
        Ok(Arc::new(provider))
    }

    /// 创建DeepSeek提供商
    pub fn deepseek(model: &str) -> Result<Arc<dyn LlmProvider>> {
        let api_key = std::env::var("DEEPSEEK_API_KEY")
            .map_err(|_| crate::error::LumosError::ConfigError {
                message: "DEEPSEEK_API_KEY environment variable not found".to_string(),
            })?;
        let provider = DeepSeekProvider::new(api_key, Some(model.to_string()));
        Ok(Arc::new(provider))
    }

    /// 创建Cohere提供商
    pub fn cohere(model: &str) -> Result<Arc<dyn LlmProvider>> {
        let provider = CohereProvider::from_env()?;
        Ok(Arc::new(provider))
    }

    /// 创建Gemini提供商
    pub fn gemini(model: &str) -> Result<Arc<dyn LlmProvider>> {
        let provider = GeminiProvider::from_env()?;
        Ok(Arc::new(provider))
    }

    /// 创建Ollama提供商
    pub fn ollama(model: &str) -> Arc<dyn LlmProvider> {
        let provider = OllamaProvider::from_env();
        Arc::new(provider)
    }

    /// 创建Together提供商
    pub fn together(model: &str) -> Result<Arc<dyn LlmProvider>> {
        let provider = TogetherProvider::from_env()?;
        Ok(Arc::new(provider))
    }

    /// 自动选择最佳提供商
    pub fn auto() -> Result<Arc<dyn LlmProvider>> {
        // 按优先级尝试不同的提供商
        if let Ok(provider) = openai("gpt-4") {
            return Ok(provider);
        }
        if let Ok(provider) = anthropic("claude-3-sonnet") {
            return Ok(provider);
        }
        if let Ok(provider) = qwen("qwen-turbo") {
            return Ok(provider);
        }
        
        // 最后尝试本地Ollama
        Ok(ollama("llama2"))
    }
}

/// 向量存储便利函数
pub mod vector {
    use super::*;
    // use crate::vector::*;

    /// 创建内存向量存储
    pub async fn memory() -> Result<Arc<dyn std::fmt::Debug>> {
        // 简化实现，返回一个占位符
        Ok(Arc::new("memory_storage".to_string()))
    }

    /// 创建PostgreSQL向量存储
    #[cfg(feature = "postgres")]
    pub async fn postgres(url: Option<&str>) -> Result<Arc<dyn VectorStorage>> {
        use lumosai_vector_postgres::PostgresVectorStorage;
        
        let database_url = url
            .map(|s| s.to_string())
            .or_else(|| std::env::var("DATABASE_URL").ok())
            .ok_or_else(|| crate::error::LumosError::ConfigError {
                message: "Database URL not provided and DATABASE_URL not set".to_string(),
            })?;

        let storage = PostgresVectorStorage::new(&database_url).await?;
        Ok(Arc::new(storage))
    }

    /// 创建Qdrant向量存储
    #[cfg(feature = "qdrant")]
    pub async fn qdrant(url: Option<&str>) -> Result<Arc<dyn VectorStorage>> {
        use lumosai_vector_qdrant::QdrantVectorStorage;
        
        let qdrant_url = url
            .map(|s| s.to_string())
            .or_else(|| std::env::var("QDRANT_URL").ok())
            .unwrap_or_else(|| "http://localhost:6334".to_string());

        let storage = QdrantVectorStorage::new(&qdrant_url).await?;
        Ok(Arc::new(storage))
    }

    /// 创建Weaviate向量存储
    #[cfg(feature = "weaviate")]
    pub async fn weaviate(url: Option<&str>) -> Result<Arc<dyn VectorStorage>> {
        use lumosai_vector_weaviate::WeaviateVectorStorage;
        
        let weaviate_url = url
            .map(|s| s.to_string())
            .or_else(|| std::env::var("WEAVIATE_URL").ok())
            .unwrap_or_else(|| "http://localhost:8080".to_string());

        let storage = WeaviateVectorStorage::new(&weaviate_url).await?;
        Ok(Arc::new(storage))
    }

    /// 自动选择最佳向量存储
    pub async fn auto() -> Result<Arc<dyn std::fmt::Debug>> {
        // 按优先级尝试不同的存储
        #[cfg(feature = "postgres")]
        if let Ok(storage) = postgres(None).await {
            return Ok(storage);
        }
        
        #[cfg(feature = "qdrant")]
        if let Ok(storage) = qdrant(None).await {
            return Ok(storage);
        }
        
        #[cfg(feature = "weaviate")]
        if let Ok(storage) = weaviate(None).await {
            return Ok(storage);
        }
        
        // 最后使用内存存储
        memory().await
    }
}

/// Agent便利函数
pub mod agent {
    use super::*;

    /// 创建简单的Agent（占位符实现）
    pub async fn simple(model: &str, instructions: &str) -> Result<String> {
        // 简化实现，返回一个描述字符串
        Ok(format!("Simple agent with model: {}, instructions: {}", model, instructions))
    }

    /// 创建带工具的Agent（占位符实现）
    pub async fn with_tools(
        model: &str,
        instructions: &str,
        _tools: Vec<String>  // 简化为字符串列表
    ) -> Result<String> {
        Ok(format!("Agent with tools - model: {}, instructions: {}", model, instructions))
    }

    /// 创建RAG Agent（占位符实现）
    pub async fn rag(
        model: &str,
        instructions: &str,
        _vector_storage: Arc<dyn std::fmt::Debug>
    ) -> Result<String> {
        Ok(format!("RAG agent - model: {}, instructions: {}", model, instructions))
    }
}

/// 云服务便利函数
pub mod cloud {
    use super::*;
    use crate::cloud::*;

    /// 创建AWS适配器
    pub fn aws() -> Result<Box<dyn CloudAdapter>> {
        let adapter = AwsAdapter::from_env()?;
        Ok(Box::new(adapter))
    }

    /// 创建Azure适配器
    pub fn azure() -> Result<Box<dyn CloudAdapter>> {
        let adapter = AzureAdapter::from_env()?;
        Ok(Box::new(adapter))
    }

    /// 创建GCP适配器
    pub fn gcp() -> Result<Box<dyn CloudAdapter>> {
        let adapter = GcpAdapter::from_env()?;
        Ok(Box::new(adapter))
    }

    /// 自动选择云适配器
    pub fn auto() -> Result<Box<dyn CloudAdapter>> {
        // 按优先级尝试不同的云提供商
        if let Ok(adapter) = aws() {
            return Ok(adapter);
        }
        if let Ok(adapter) = azure() {
            return Ok(adapter);
        }
        if let Ok(adapter) = gcp() {
            return Ok(adapter);
        }
        
        Err(crate::error::LumosError::ConfigError {
            message: "No cloud provider credentials found".to_string(),
        })
    }

    /// 快速部署应用
    pub async fn deploy(
        app_name: &str,
        image: &str,
        provider: Option<&str>
    ) -> Result<String> {
        let adapter = if let Some(p) = provider {
            crate::cloud::create_adapter(p)?
        } else {
            auto()?
        };

        let config = DeploymentConfig {
            name: app_name.to_string(),
            version: "1.0.0".to_string(),
            image: image.to_string(),
            environment: std::collections::HashMap::new(),
            resources: crate::cloud::ResourceConfig {
                cpu: 1.0,
                memory: 1024,
                storage: None,
                gpu: None,
            },
            network: crate::cloud::NetworkConfig {
                ports: vec![crate::cloud::PortMapping {
                    container_port: 8080,
                    host_port: None,
                    protocol: "HTTP".to_string(),
                }],
                public: true,
                domain: None,
                ssl: None,
            },
            storage: None,
            autoscaling: None,
            health_check: None,
        };

        let result = adapter.deploy_application(&config).await?;
        Ok(result.deployment_id)
    }
}

/// 一站式便利函数
pub mod quick {
    use super::*;

    /// 快速创建聊天机器人
    pub async fn chatbot(instructions: &str) -> Result<String> {
        agent::simple("gpt-4", instructions).await
    }

    /// 快速创建RAG系统
    pub async fn rag_system(instructions: &str) -> Result<(String, Arc<dyn std::fmt::Debug>)> {
        let storage = vector::auto().await?;
        let agent = agent::rag("gpt-4", instructions, storage.clone()).await?;
        Ok((agent, storage))
    }

    /// 快速部署到云端
    pub async fn deploy_to_cloud(app_name: &str, image: &str) -> Result<String> {
        cloud::deploy(app_name, image, None).await
    }

    /// 一键设置完整的AI应用
    pub async fn ai_app(
        _name: &str,
        instructions: &str,
        _tools: Vec<String>
    ) -> Result<(String, Arc<dyn std::fmt::Debug>)> {
        let storage = vector::auto().await?;
        let agent = agent::with_tools("gpt-4", instructions, _tools).await?;
        Ok((agent, storage))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_auto() {
        // 这个测试需要环境变量配置
        if std::env::var("OPENAI_API_KEY").is_ok() {
            let provider = llm::auto().unwrap();
            assert_eq!(provider.name(), "openai");
        }
    }

    #[tokio::test]
    async fn test_vector_memory() {
        let storage = vector::memory().await.unwrap();
        // 简化测试
        assert!(format!("{:?}", storage).contains("memory_storage"));
    }

    #[tokio::test]
    async fn test_quick_chatbot() {
        // 简化测试
        let agent = quick::chatbot("You are a helpful assistant.").await.unwrap();
        assert!(agent.contains("gpt-4"));
    }
}
