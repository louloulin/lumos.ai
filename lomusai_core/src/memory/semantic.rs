//! 语义搜索内存模块
//! 
//! 提供对历史消息的语义搜索功能，支持根据语义关联度检索最相关的历史消息

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;
use chrono;

use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::error::{Error, Result};
use crate::logger::{Component, LogLevel};
use crate::llm::{Message, LlmProvider};
use crate::memory::{Memory, MemoryConfig, MessageRange};
use crate::vector::{VectorStorage, Document, FilterCondition};

/// 语义搜索内存实现
pub struct SemanticMemory {
    /// 基础组件
    base: BaseComponent,
    /// LLM提供者，用于生成嵌入向量
    llm: Arc<dyn LlmProvider>,
    /// 向量存储
    vector_storage: Arc<crate::vector::MemoryVectorStorage>,
    /// 消息存储（ID -> 消息）
    messages: Mutex<HashMap<String, Message>>,
    /// 命名空间
    namespace: String,
}

impl SemanticMemory {
    /// 创建新的语义搜索内存
    pub fn new(config: &MemoryConfig, llm: Arc<dyn LlmProvider>) -> Result<Self> {
        // 直接创建MemoryVectorStorage实例而非通过函数获取Box<dyn VectorStorage>
        let vector_storage = crate::vector::MemoryVectorStorage::new(1536, None);
        let vector_storage = Arc::new(vector_storage);
        
        let namespace = config.namespace.clone().unwrap_or_else(|| "default".to_string());
        
        let component_config = ComponentConfig {
            name: Some("SemanticMemory".to_string()),
            component: Component::Memory,
            log_level: Some(LogLevel::Info),
        };
        
        Ok(Self {
            base: BaseComponent::new(component_config),
            llm,
            vector_storage,
            messages: Mutex::new(HashMap::new()),
            namespace,
        })
    }
    
    /// 初始化向量存储索引
    async fn init_vector_storage(&self) -> Result<()> {
        // 检查索引是否存在，如果不存在则创建
        let indexes = self.vector_storage.list_indexes().await.unwrap_or_default();
        if !indexes.contains(&"default".to_string()) {
            self.vector_storage.create_index("default", 1536, None).await?;
        }
        Ok(())
    }
    
    /// 转换消息为文本
    fn message_to_text(message: &Message) -> String {
        format!("{}: {}", message.role, message.content)
    }
    
    /// 获取上下文窗口消息
    fn get_window_messages(&self, message_ids: &[String], target_index: usize, window: &MessageRange) 
        -> Vec<String> {
        let start = if target_index >= window.before {
            target_index - window.before
        } else {
            0
        };
        
        let end = std::cmp::min(target_index + window.after + 1, message_ids.len());
        
        message_ids[start..end].to_vec()
    }
    
    /// 获取语义内存统计信息
    pub async fn get_stats(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut stats = HashMap::new();
        
        // 获取索引统计信息
        if let Ok(index_stats) = self.vector_storage.describe_index("default").await {
            stats.insert("vector_dimension".to_string(), serde_json::json!(index_stats.dimension));
            stats.insert("vector_count".to_string(), serde_json::json!(index_stats.count));
            stats.insert("similarity_metric".to_string(), serde_json::json!(format!("{:?}", index_stats.metric)));
        }
        
        // 获取内存中消息数量
        let message_count = self.messages.lock()
            .map(|messages| messages.len())
            .unwrap_or(0);
        stats.insert("message_count".to_string(), serde_json::json!(message_count));
        
        // 获取命名空间
        stats.insert("namespace".to_string(), serde_json::json!(self.namespace));
        
        Ok(stats)
    }
}

#[async_trait]
impl Memory for SemanticMemory {
    async fn store(&self, message: &Message) -> Result<()> {
        // 确保索引已初始化
        self.init_vector_storage().await?;
        
        let message_id = Uuid::new_v4().to_string();
        let message_text = Self::message_to_text(message);
        
        // 创建消息的向量嵌入
        let embedding = self.llm.get_embedding(&message_text).await?;
        
        // 存储到向量数据库
        let mut metadata = HashMap::new();
        metadata.insert("namespace".to_string(), Value::String(self.namespace.clone()));
        metadata.insert("role".to_string(), Value::String(message.role.to_string()));
        metadata.insert("timestamp".to_string(), Value::Number(serde_json::Number::from(
            chrono::Utc::now().timestamp()
        )));

        // 创建文档
        let doc = Document {
            id: message_id.clone(),
            content: message_text,
            metadata,
            embedding,
        };
        
        // 直接调用实例方法，而不是通过trait
        self.vector_storage.upsert(
            "default", 
            vec![doc.embedding.clone()],
            Some(vec![doc.id.clone()]), 
            Some(vec![doc.metadata.clone()])
        ).await?;
        
        // 保存消息
        let mut messages = self.messages.lock().unwrap();
        messages.insert(message_id, message.clone());
        
        Ok(())
    }
    
    async fn retrieve(&self, config: &MemoryConfig) -> Result<Vec<Message>> {
        // 确保索引已初始化
        self.init_vector_storage().await?;
        
        // 使用最后消息
        if let Some(limit) = config.last_messages {
            let messages = self.messages.lock().unwrap();
            let mut ids: Vec<String> = messages.keys().cloned().collect();
            
            // 按时间戳排序（假设ID包含时间戳或按序插入）
            ids.sort(); 
            
            // 取最后的limit条消息
            let ids = if ids.len() > limit {
                ids[ids.len() - limit..].to_vec()
            } else {
                ids
            };
            
            return Ok(ids.iter()
                .filter_map(|id| messages.get(id).cloned())
                .collect());
        }
        
        // 使用语义回忆
        if let Some(semantic_config) = &config.semantic_recall {
            // 使用config中的query或默认值
            let query = config.query.as_deref().unwrap_or("最近的对话");
            let embedding = match self.llm.get_embedding(query).await {
                Ok(embedding) => embedding,
                Err(_) => return Err(Error::Unavailable(format!("获取嵌入向量失败: {}", query))),
            };
            
            // 创建过滤条件
            let filter = FilterCondition::Eq {
                field_name: "namespace".to_string(),
                value: Value::String(self.namespace.clone()),
            };
            
            // 查询向量数据库
            let results = match self.vector_storage.query(
                "default",
                embedding,
                semantic_config.top_k,
                Some(filter),
                false
            ).await {
                Ok(results) => results,
                Err(_) => return Err(Error::Storage("查询语义向量失败".to_string())),
            };
            
            // 收集消息ID
            let message_ids: Vec<String> = results.iter()
                .map(|result| result.id.clone())
                .collect();
            
            if message_ids.is_empty() {
                return Ok(Vec::new());
            }
            
            // 获取消息
            let messages = self.messages.lock().unwrap();
            
            // 使用集合进行去重，避免返回重复消息
            let mut retrieved_message_ids = std::collections::HashSet::new();
            let mut retrieved_messages = Vec::new();
            
            // 应用消息窗口
            if let Some(window) = &semantic_config.message_range {
                for (i, _) in message_ids.iter().enumerate() {
                    let window_ids = self.get_window_messages(&message_ids, i, window);
                    for window_id in &window_ids {
                        // 避免重复添加相同ID的消息
                        if !retrieved_message_ids.contains(window_id) {
                            retrieved_message_ids.insert(window_id.clone());
                            if let Some(msg) = messages.get(window_id) {
                                retrieved_messages.push(msg.clone());
                            }
                        }
                    }
                }
            } else {
                // 不使用窗口，直接返回匹配的消息
                for id in &message_ids {
                    if !retrieved_message_ids.contains(id) {
                        retrieved_message_ids.insert(id.clone());
                        if let Some(msg) = messages.get(id) {
                            retrieved_messages.push(msg.clone());
                        }
                    }
                }
            }
            
            // 根据检索结果排序
            if !retrieved_messages.is_empty() {
                // 创建ID到分数的映射，用于排序
                let mut id_to_score = HashMap::new();
                for (idx, result) in results.iter().enumerate() {
                    id_to_score.insert(result.id.clone(), (results.len() - idx) as f32);
                }
                
                // 按相关性排序
                retrieved_messages.sort_by(|a, b| {
                    // Clone message data for comparisons to avoid borrowing issues
                    let a_content = a.content.clone();
                    let a_role = a.role.clone();
                    let b_content = b.content.clone();
                    let b_role = b.role.clone();
                    
                    // Create longer-lived values
                    let empty_string = String::new();
                    
                    let a_id = messages.iter()
                        .find(|(_, msg)| msg.content == a_content && msg.role == a_role)
                        .map(|(id, _)| id)
                        .unwrap_or(&empty_string);
                    
                    let b_id = messages.iter()
                        .find(|(_, msg)| msg.content == b_content && msg.role == b_role)
                        .map(|(id, _)| id)
                        .unwrap_or(&empty_string);
                    
                    let a_score = id_to_score.get(a_id).unwrap_or(&0.0);
                    let b_score = id_to_score.get(b_id).unwrap_or(&0.0);
                    
                    b_score.partial_cmp(a_score).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            
            return Ok(retrieved_messages);
        }
        
        // 默认返回空
        Ok(Vec::new())
    }
}

impl Base for SemanticMemory {
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

/// 创建语义搜索内存
pub fn create_semantic_memory<P: LlmProvider + 'static>(
    config: &MemoryConfig,
    llm: Arc<P>
) -> Result<SemanticMemory> {
    if !config.enabled {
        return Err(Error::Configuration("Memory not enabled".to_string()));
    }
    
    SemanticMemory::new(config, llm as Arc<dyn LlmProvider>)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{MockLlmProvider, Role};
    
    // 测试用例辅助函数 - 创建符合特定嵌入向量维度的配置
    fn create_test_config(namespace: &str, query: Option<&str>) -> MemoryConfig {
        MemoryConfig {
            enabled: true,
            namespace: Some(namespace.to_string()),
            store_id: None,
            working_memory: None,
            semantic_recall: Some(SemanticRecallConfig {
                top_k: 5,
                message_range: Some(MessageRange {
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
            query: query.map(|q| q.to_string()),
        }
    }
    
    #[tokio::test]
    async fn test_semantic_memory_store() {
        // 创建配置
        let config = create_test_config("test_store", None);
        
        // 创建Mock LLM，提供1536维的嵌入向量
        let mock_llm = Arc::new(MockLlmProvider::new_with_sequential_embeddings(
            0.1, 0.05, 1536, 1 // 只需要一个向量用于存储测试
        ));
        
        // 创建语义内存
        let semantic_memory = SemanticMemory::new(&config, Arc::clone(&mock_llm) as Arc<dyn LlmProvider>).unwrap();
        
        // 创建消息
        let message = Message {
            role: Role::User,
            content: "你好，我想了解一下语义搜索".to_string(),
            metadata: None,
            name: None,
        };
        
        // 存储消息
        let result = semantic_memory.store(&message).await;
        assert!(result.is_ok(), "存储消息失败: {:?}", result);
        
        // 验证内存统计
        let stats = semantic_memory.get_stats().await.unwrap();
        assert_eq!(stats.get("message_count").and_then(|v| v.as_u64()), Some(1));
    }
    
    #[tokio::test]
    async fn test_semantic_memory_retrieve() {
        // 创建配置
        let config = create_test_config("test_retrieve", Some("语义搜索是什么"));
        
        // 创建Mock LLM，提供1536维的嵌入向量: 两个用于存储消息，一个用于查询
        let mock_llm = Arc::new(MockLlmProvider::new_with_sequential_embeddings(
            0.1, 0.1, 1536, 3
        ));
        
        // 创建语义内存
        let semantic_memory = SemanticMemory::new(&config, Arc::clone(&mock_llm) as Arc<dyn LlmProvider>).unwrap();
        
        // 创建测试消息
        let message1 = Message {
            role: Role::User,
            content: "你好，我想了解一下语义搜索".to_string(),
            metadata: None,
            name: None,
        };
        
        let message2 = Message {
            role: Role::Assistant,
            content: "语义搜索是一种基于语义理解的搜索技术".to_string(),
            metadata: None,
            name: None,
        };
        
        // 存储消息
        let store_result1 = semantic_memory.store(&message1).await;
        assert!(store_result1.is_ok(), "存储第一条消息失败: {:?}", store_result1);
        
        let store_result2 = semantic_memory.store(&message2).await;
        assert!(store_result2.is_ok(), "存储第二条消息失败: {:?}", store_result2);
        
        // 检索消息
        let results = semantic_memory.retrieve(&config).await;
        assert!(results.is_ok(), "检索消息失败: {:?}", results);
        
        let messages = results.unwrap();
        assert!(!messages.is_empty(), "检索结果不应为空");
        
        // 验证检索结果内容
        let contains_message1 = messages.iter().any(|msg| 
            msg.role == Role::User && msg.content.contains("语义搜索"));
        assert!(contains_message1, "检索结果应包含用户询问的消息");
        
        let contains_message2 = messages.iter().any(|msg| 
            msg.role == Role::Assistant && msg.content.contains("语义理解"));
        assert!(contains_message2, "检索结果应包含助手回复的消息");
    }
    
    #[tokio::test]
    async fn test_semantic_memory_stats() {
        // 创建配置
        let config = create_test_config("test_stats", None);
        
        // 创建Mock LLM
        let mock_llm = Arc::new(MockLlmProvider::new_with_sequential_embeddings(
            0.1, 0.1, 1536, 1
        ));
        
        // 创建语义内存
        let semantic_memory = SemanticMemory::new(&config, Arc::clone(&mock_llm) as Arc<dyn LlmProvider>).unwrap();
        
        // 存储一条消息
        let message = Message {
            role: Role::User,
            content: "测试统计信息".to_string(),
            metadata: None,
            name: None,
        };
        
        let store_result = semantic_memory.store(&message).await;
        assert!(store_result.is_ok());
        
        // 获取统计信息
        let stats = semantic_memory.get_stats().await;
        assert!(stats.is_ok());
        
        let stats = stats.unwrap();
        assert_eq!(stats.get("namespace").and_then(|v| v.as_str()), Some("test_stats"));
        assert_eq!(stats.get("message_count").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(stats.get("vector_dimension").and_then(|v| v.as_u64()), Some(1536));
        assert_eq!(stats.get("vector_count").and_then(|v| v.as_u64()), Some(1));
    }
} 