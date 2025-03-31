use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use async_trait::async_trait;

use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::error::{Error, Result};
use crate::logger::{Component, LogLevel};
use crate::memory::{SemanticRecallConfig, Memory, MessageRange};
use crate::llm::{Message, LlmProvider, LlmOptions, Role};
use crate::vector::{Document, FilterCondition};

/// 语义记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemoryEntry {
    /// 条目ID
    pub id: String,
    /// 原始内容
    pub content: String,
    /// 条目摘要
    pub summary: Option<String>,
    /// 条目嵌入向量
    pub embedding: Option<Vec<f32>>,
    /// 创建时间戳
    pub created_at: u64,
    /// 相关性分数
    pub relevance: Option<f32>,
    /// 元数据
    pub metadata: HashMap<String, Value>,
}

/// 语义记忆实现
pub struct SemanticMemory<P: LlmProvider, E: EmbeddingProvider> {
    /// 基础组件
    base: BaseComponent,
    /// 内存配置
    config: SemanticRecallConfig,
    /// 内存数据
    data: Mutex<Vec<SemanticMemoryEntry>>,
    /// LLM提供者
    llm: Arc<P>,
    /// 嵌入向量提供者
    embedding_provider: Arc<E>,
    /// 记忆检索模板
    template: String,
}

/// 嵌入向量提供者接口
#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// 生成文本嵌入向量
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    
    /// 计算两个向量的相似度 (0.0 - 1.0)
    fn similarity(&self, vec1: &[f32], vec2: &[f32]) -> f32;
}

impl<P: LlmProvider, E: EmbeddingProvider> SemanticMemory<P, E> {
    /// 创建新的语义记忆
    pub fn new(config: SemanticRecallConfig, llm: Arc<P>, embedding_provider: Arc<E>) -> Self {
        let template = config.template.clone().unwrap_or_else(|| DEFAULT_TEMPLATE.to_string());
        let component_config = ComponentConfig {
            name: Some("SemanticMemory".to_string()),
            component: Component::Memory,
            log_level: Some(LogLevel::Info),
        };
        
        Self {
            base: BaseComponent::new(component_config),
            config,
            data: Mutex::new(Vec::new()),
            llm,
            embedding_provider,
            template,
        }
    }
    
    /// 添加条目到语义记忆
    pub async fn add_entry(&self, content: String, metadata: Option<HashMap<String, Value>>) -> Result<SemanticMemoryEntry> {
        let timestamp = current_timestamp();
        let id = generate_id();
        
        // 生成摘要
        let summary = if self.config.generate_summaries {
            Some(self.generate_summary(&content).await?)
        } else {
            None
        };
        
        // 生成嵌入向量
        let embedding = if self.config.use_embeddings {
            Some(self.embedding_provider.embed(&content).await?)
        } else {
            None
        };
        
        let entry = SemanticMemoryEntry {
            id,
            content,
            summary,
            embedding,
            created_at: timestamp,
            relevance: None,
            metadata: metadata.unwrap_or_default(),
        };
        
        let mut data = self.data.lock().map_err(|_| Error::Internal("Failed to lock memory data".to_string()))?;
        
        // 检查容量限制
        if let Some(max_capacity) = self.config.max_capacity {
            if data.len() >= max_capacity {
                data.remove(0); // 移除最旧的条目
            }
        }
        
        data.push(entry.clone());
        Ok(entry)
    }
    
    /// 获取所有条目
    pub fn get_all_entries(&self) -> Result<Vec<SemanticMemoryEntry>> {
        let data = self.data.lock().map_err(|_| Error::Internal("Failed to lock memory data".to_string()))?;
        Ok(data.clone())
    }
    
    /// 获取指定ID的条目
    pub fn get_entry(&self, id: &str) -> Result<Option<SemanticMemoryEntry>> {
        let data = self.data.lock().map_err(|_| Error::Internal("Failed to lock memory data".to_string()))?;
        Ok(data.iter().find(|entry| entry.id == id).cloned())
    }
    
    /// 删除条目
    pub fn delete_entry(&self, id: &str) -> Result<bool> {
        let mut data = self.data.lock().map_err(|_| Error::Internal("Failed to lock memory data".to_string()))?;
        let initial_len = data.len();
        data.retain(|entry| entry.id != id);
        Ok(data.len() < initial_len)
    }
    
    /// 基于语义搜索相关条目
    pub async fn search(&self, query: &str, limit: Option<usize>) -> Result<Vec<SemanticMemoryEntry>> {
        let limit = limit.unwrap_or_else(|| self.config.max_results.unwrap_or(5));
        
        if !self.config.use_embeddings {
            return Err(Error::Internal("Semantic search requires embeddings to be enabled".to_string()));
        }
        
        let query_embedding = self.embedding_provider.embed(query).await?;
        
        let mut entries = self.get_all_entries()?;
        
        // 计算相关性分数
        for entry in &mut entries {
            if let Some(embedding) = &entry.embedding {
                entry.relevance = Some(self.embedding_provider.similarity(&query_embedding, embedding));
            } else {
                entry.relevance = Some(0.0);
            }
        }
        
        // 按相关性排序
        entries.sort_by(|a, b| {
            b.relevance.unwrap_or(0.0).partial_cmp(&a.relevance.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // 限制结果数量
        entries.truncate(limit);
        
        // 过滤低相关性结果
        if let Some(threshold) = self.config.relevance_threshold {
            entries.retain(|entry| entry.relevance.unwrap_or(0.0) >= threshold);
        }
        
        Ok(entries)
    }
    
    /// 生成内容摘要
    async fn generate_summary(&self, content: &str) -> Result<String> {
        let prompt = format!("Please summarize the following text in a concise manner:\n\n{}", content);
        let options = LlmOptions::default();
        self.llm.generate(&prompt, &options).await
    }
    
    /// 生成记忆检索结果
    pub async fn retrieve_relevant_memories(&self, query: &str) -> Result<String> {
        let entries = self.search(query, None).await?;
        
        if entries.is_empty() {
            return Ok("No relevant memories found.".to_string());
        }
        
        // 构建记忆内容
        let memories_text = entries.iter()
            .enumerate()
            .map(|(i, entry)| {
                let relevance = entry.relevance.unwrap_or(0.0) * 100.0;
                let content = if let Some(summary) = &entry.summary {
                    summary.clone()
                } else {
                    entry.content.clone()
                };
                
                format!("Memory {}: {} (relevance: {:.1}%)", i + 1, content, relevance)
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        
        // 使用模板
        let formatted_result = format!("{}\n\n{}", self.template, memories_text);
        
        Ok(formatted_result)
    }
    
    /// 将记忆转换为消息列表
    pub async fn to_messages(&self, query: &str) -> Result<Vec<Message>> {
        let memories = self.retrieve_relevant_memories(query).await?;
        
        Ok(vec![Message {
            role: Role::System,
            content: memories,
            name: Some("semantic_memory".to_string()),
            metadata: None,
        }])
    }
}

impl<P: LlmProvider, E: EmbeddingProvider> Base for SemanticMemory<P, E> {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }
    
    fn component(&self) -> Component {
        self.base.component()
    }
    
    fn logger(&self) -> Arc<dyn crate::logger::Logger> {
        self.base.logger()
    }
    
    fn set_logger(&mut self, logger: Arc<dyn crate::logger::Logger>) {
        self.base.set_logger(logger);
    }
    
    fn telemetry(&self) -> Option<Arc<dyn crate::telemetry::TelemetrySink>> {
        self.base.telemetry()
    }
    
    fn set_telemetry(&mut self, telemetry: Arc<dyn crate::telemetry::TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
}

#[async_trait::async_trait]
impl<P: LlmProvider + 'static, E: EmbeddingProvider + 'static> Memory for SemanticMemory<P, E> {
    async fn store(&self, message: &Message) -> Result<()> {
        // 只存储用户和助手的消息
        if message.role == Role::User || message.role == Role::Assistant {
            let mut metadata = HashMap::new();
            metadata.insert("role".to_string(), Value::String(message.role.to_string()));
            
            if let Some(name) = &message.name {
                metadata.insert("name".to_string(), Value::String(name.clone()));
            }
            
            self.add_entry(message.content.clone(), Some(metadata)).await?;
        }
        Ok(())
    }
    
    async fn retrieve(&self, config: &crate::memory::MemoryConfig) -> Result<Vec<Message>> {
        if let Some(query) = &config.query {
            self.to_messages(query).await
        } else {
            // 如果没有提供查询，返回空列表
            Ok(Vec::new())
        }
    }
}

/// 生成当前时间戳（秒）
fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 生成唯一ID
fn generate_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

/// 默认记忆检索模板
const DEFAULT_TEMPLATE: &str = r#"
Based on the conversation history, here are some relevant memories that might be helpful for your response:
"#;

/// 内存语义搜索接口
/// 
/// 提供对历史消息记录的语义搜索功能

/// 语义搜索选项
#[derive(Debug, Clone)]
pub struct SemanticSearchOptions {
    /// 最大结果数量
    pub limit: usize,
    /// 相似度阈值
    pub threshold: Option<f32>,
    /// 命名空间过滤
    pub namespace: Option<String>,
    /// 使用上下文窗口
    pub use_window: bool,
    /// 上下文窗口大小
    pub window_size: Option<(usize, usize)>,
    /// 额外的过滤条件
    pub filter: Option<FilterCondition>,
}

impl Default for SemanticSearchOptions {
    fn default() -> Self {
        Self {
            limit: 5,
            threshold: Some(0.7),
            namespace: None,
            use_window: true,
            window_size: Some((1, 1)),
            filter: None,
        }
    }
}

/// 语义内存搜索结果
#[derive(Debug)]
pub struct SemanticSearchResult {
    /// 消息
    pub message: Message,
    /// 相似度分数
    pub score: f32,
    /// 相关上下文
    pub context: Option<Vec<Message>>,
}

/// 语义内存接口
#[async_trait]
pub trait SemanticMemoryTrait: Send + Sync {
    /// 向内存中添加消息
    async fn add(&self, message: &Message) -> Result<()>;
    
    /// 语义搜索相关消息
    async fn search(&self, query: &str, options: &SemanticSearchOptions) -> Result<Vec<SemanticSearchResult>>;
    
    /// 检索最近的消息
    async fn get_recent(&self, limit: usize) -> Result<Vec<Message>>;
    
    /// 获取指定消息的上下文
    async fn get_context(&self, message_id: &str, before: usize, after: usize) -> Result<Vec<Message>>;
    
    /// 清空内存
    async fn clear(&self) -> Result<()>;
}

/// 创建语义内存
pub fn create_semantic_memory(
    config: &crate::memory::MemoryConfig,
    llm: Arc<dyn crate::llm::LlmProvider>,
) -> Result<Arc<dyn SemanticMemoryTrait>> {
    if let Some(store_id) = &config.store_id {
        match store_id.as_str() {
            "vector" => {
                let mem = crate::memory::semantic::SemanticMemory::new(config, llm)?;
                Ok(Arc::new(SemanticMemoryAdapter::new(mem)))
            },
            _ => Err(Error::Configuration(format!("Unsupported memory store: {}", store_id))),
        }
    } else {
        // 默认使用向量存储
        let mem = crate::memory::semantic::SemanticMemory::new(config, llm)?;
        Ok(Arc::new(SemanticMemoryAdapter::new(mem)))
    }
}

/// 语义内存适配器
struct SemanticMemoryAdapter {
    inner: crate::memory::semantic::SemanticMemory,
}

impl SemanticMemoryAdapter {
    fn new(inner: crate::memory::semantic::SemanticMemory) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl SemanticMemoryTrait for SemanticMemoryAdapter {
    async fn add(&self, message: &Message) -> Result<()> {
        self.inner.store(message).await
    }
    
    async fn search(&self, query: &str, options: &SemanticSearchOptions) -> Result<Vec<SemanticSearchResult>> {
        // 构建内存查询配置
        let config = crate::memory::MemoryConfig {
            enabled: true,
            namespace: options.namespace.clone(),
            semantic_recall: Some(crate::memory::SemanticRecallConfig {
                top_k: options.limit,
                message_range: if options.use_window {
                    options.window_size.map(|(before, after)| crate::memory::MessageRange {
                        before,
                        after,
                    })
                } else {
                    None
                },
                generate_summaries: true,
                use_embeddings: true,
                max_capacity: None,
                max_results: Some(options.limit),
                relevance_threshold: options.threshold,
                template: None,
            }),
            ..Default::default()
        };
        
        // 执行搜索
        let messages = self.inner.retrieve(&config).await?;
        
        // 转换为结果
        let results = messages.into_iter()
            .map(|msg| SemanticSearchResult {
                message: msg.clone(),
                score: 1.0, // 目前简化，未获取实际分数
                context: None,
            })
            .collect();
        
        Ok(results)
    }
    
    async fn get_recent(&self, limit: usize) -> Result<Vec<Message>> {
        let config = crate::memory::MemoryConfig {
            enabled: true,
            last_messages: Some(limit),
            ..Default::default()
        };
        
        self.inner.retrieve(&config).await
    }
    
    async fn get_context(&self, _message_id: &str, _before: usize, _after: usize) -> Result<Vec<Message>> {
        // 暂未实现，目前返回空
        Ok(Vec::new())
    }
    
    async fn clear(&self) -> Result<()> {
        // 暂未实现
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{Role, MockLlmProvider};
    use tokio::time::{timeout, Duration};
    
    // 生成指定长度的测试向量
    fn create_test_vector(seed: f32, length: usize) -> Vec<f32> {
        (0..length).map(|i| seed + (i as f32 * 0.01)).collect()
    }
    
    #[tokio::test]
    async fn test_semantic_memory_search() {
        // 创建配置
        let config = crate::memory::MemoryConfig {
            enabled: true,
            namespace: Some("test".to_string()),
            store_id: None,
            working_memory: None,
            semantic_recall: Some(crate::memory::SemanticRecallConfig {
                top_k: 5,
                message_range: Some(crate::memory::MessageRange {
                    before: 1,
                    after: 1,
                }),
                generate_summaries: true,
                use_embeddings: true,
                max_capacity: None,
                max_results: Some(5),
                relevance_threshold: None,
                template: None,
            }),
            last_messages: None,
            query: Some("语义搜索是什么".to_string()),
        };
        
        // 创建Mock LLM - 提供足够的嵌入向量，维度为1536
        let mock_llm = Arc::new(MockLlmProvider::new_with_embeddings(vec![
            create_test_vector(0.1, 1536), // 第一条消息的嵌入
            create_test_vector(0.4, 1536), // 第二条消息的嵌入
            create_test_vector(0.7, 1536), // 查询的嵌入
        ]));
        
        // 创建语义内存
        let semantic_memory = create_semantic_memory(&config, mock_llm).unwrap();
        
        // 添加消息
        let message1 = Message {
            role: Role::User,
            content: "你好，我想了解一下语义搜索".to_string(),
            metadata: None,
            name: None,
        };
        
        let message2 = Message {
            role: Role::Assistant,
            content: "语义搜索是一种基于语义相似度的搜索方法，它能找到语义相关的内容".to_string(),
            metadata: None,
            name: None,
        };
        
        // 使用超时机制执行异步操作
        let result = timeout(Duration::from_secs(5), async {
            semantic_memory.add(&message1).await.unwrap();
            semantic_memory.add(&message2).await.unwrap();
            
            // 执行搜索
            let options = SemanticSearchOptions::default();
            let results = semantic_memory.search("语义搜索是什么", &options).await.unwrap();
            
            // 验证结果
            assert!(!results.is_empty());
            results
        }).await;
        
        // 确保测试在超时内完成
        assert!(result.is_ok(), "测试超时");
    }
} 