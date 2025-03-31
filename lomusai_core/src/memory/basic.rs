use async_trait::async_trait;
use std::sync::Arc;
use serde_json::Value;

use crate::error::Result;
use crate::llm::Message;
use crate::memory::{Memory, MemoryConfig};
use crate::memory::working::WorkingMemory;
use crate::memory::semantic_memory::{SemanticMemoryTrait, SemanticSearchOptions};

/// BasicMemory 是一个简单的内存实现
pub struct BasicMemory {
    /// 内部工作内存
    working_memory: Option<Arc<dyn WorkingMemory>>,
    /// 内部语义内存
    semantic_memory: Option<Arc<dyn SemanticMemoryTrait>>,
}

impl BasicMemory {
    /// 创建一个新的基本内存实例
    pub fn new(
        working_memory: Option<Arc<dyn WorkingMemory>>,
        semantic_memory: Option<Arc<dyn SemanticMemoryTrait>>,
    ) -> Self {
        Self {
            working_memory,
            semantic_memory,
        }
    }
}

#[async_trait]
impl Memory for BasicMemory {
    async fn store(&self, message: &Message) -> Result<()> {
        if let Some(ref working_memory) = self.working_memory {
            // 将消息序列化为JSON值
            let message_value = serde_json::to_value(message)
                .map_err(|e| crate::error::Error::Json(e))?;
                
            working_memory.set_value("last_message", message_value).await?;
        }
        
        if let Some(ref semantic_memory) = self.semantic_memory {
            semantic_memory.add(message).await?;
        }
        
        Ok(())
    }
    
    async fn retrieve(&self, config: &MemoryConfig) -> Result<Vec<Message>> {
        let mut results = Vec::new();
        
        if let Some(ref semantic_memory) = self.semantic_memory {
            if let Some(ref semantic_recall) = config.semantic_recall {
                if let Some(ref query) = config.query {
                    // 使用默认选项创建搜索配置
                    let mut options = SemanticSearchOptions::default();
                    
                    // 使用MemoryConfig中的相关配置
                    options.limit = semantic_recall.top_k;
                    options.threshold = semantic_recall.relevance_threshold;
                    options.namespace = config.namespace.clone();
                    
                    // 执行搜索
                    let search_results = semantic_memory.search(query, &options).await?;
                    
                    for result in search_results {
                        results.push(result.message);
                    }
                }
            }
        }
        
        Ok(results)
    }
} 