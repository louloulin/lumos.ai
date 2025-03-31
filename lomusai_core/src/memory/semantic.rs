//! 语义搜索内存模块
//! 
//! 提供对历史消息的语义搜索功能，支持根据语义关联度检索最相关的历史消息

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono;
use tokio::time::{timeout, Duration};

use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::error::{Error, Result};
use crate::logger::{Component, LogLevel};
use crate::llm::{Message, LlmProvider, Role};
use crate::memory::{Memory, SemanticRecallConfig, MemoryConfig, MessageRange};
use crate::vector::{VectorStorage, Document, QueryResult, FilterCondition, create_vector_storage};

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
        format!("{}: {}", message.role.to_string(), message.content)
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
            ids.sort(); // 假设ID包含时间戳或按序插入
            
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
            let embedding = self.llm.get_embedding(query).await?;
            
            // 创建过滤条件
            let filter = FilterCondition::Eq {
                field_name: "namespace".to_string(),
                value: Value::String(self.namespace.clone()),
            };
            
            // 直接调用实例方法，而不是通过trait
            let results = self.vector_storage.query(
                "default",
                embedding,
                semantic_config.top_k,
                Some(filter),
                false
            ).await?;
            
            // 收集消息ID
            let message_ids: Vec<String> = results.iter()
                .map(|result| result.id.clone())
                .collect();
            
            // 获取消息
            let messages = self.messages.lock().unwrap();
            let mut retrieved_messages = Vec::new();
            
            // 应用消息窗口
            if let Some(window) = &semantic_config.message_range {
                for (i, id) in message_ids.iter().enumerate() {
                    let window_ids = self.get_window_messages(&message_ids, i, window);
                    for window_id in window_ids {
                        if let Some(msg) = messages.get(&window_id) {
                            retrieved_messages.push(msg.clone());
                        }
                    }
                }
            } else {
                // 不使用窗口，直接返回匹配的消息
                for id in &message_ids {
                    if let Some(msg) = messages.get(id) {
                        retrieved_messages.push(msg.clone());
                    }
                }
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
    
    // 生成指定长度的测试向量
    fn create_test_vector(seed: f32, length: usize) -> Vec<f32> {
        (0..length).map(|i| seed + (i as f32 * 0.01)).collect()
    }
    
    #[tokio::test]
    async fn test_semantic_memory_store() {
        // 创建配置
        let config = MemoryConfig {
            enabled: true,
            namespace: Some("test".to_string()),
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
            query: None,
        };
        
        // 创建Mock LLM，提供1536维的嵌入向量
        let mock_llm = Arc::new(MockLlmProvider::new_with_embeddings(vec![
            create_test_vector(0.1, 1536), // 第一条消息的嵌入
        ]));
        
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
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_semantic_memory_retrieve() {
        // 创建配置
        let config = MemoryConfig {
            enabled: true,
            namespace: Some("test".to_string()),
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
            query: Some("语义搜索是什么".to_string()),
        };
        
        // 创建Mock LLM，提供1536维的嵌入向量
        let mock_llm = Arc::new(MockLlmProvider::new_with_embeddings(vec![
            create_test_vector(0.1, 1536), // 第一条消息的嵌入
            create_test_vector(0.4, 1536), // 查询的嵌入
        ]));
        
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
        let store_result = semantic_memory.store(&message).await;
        assert!(store_result.is_ok());
        
        // 检索消息
        let results = semantic_memory.retrieve(&config).await;
        assert!(results.is_ok());
        
        let results = results.unwrap();
        assert!(!results.is_empty());
    }
} 