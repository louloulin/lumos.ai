use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::error::{Error, Result};
use crate::logger::{Component, LogLevel};
use crate::memory::{WorkingMemoryConfig, Memory, MessageRange};
use crate::llm::{Message, LlmProvider, LlmOptions, Role};

/// 工作内存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryEntry {
    /// 条目ID
    pub id: String,
    /// 条目内容
    pub content: String,
    /// 条目类型
    pub content_type: String,
    /// 创建时间戳
    pub created_at: u64,
    /// 更新时间戳
    pub updated_at: u64,
    /// 元数据
    pub metadata: HashMap<String, Value>,
}

/// 工作内存实现
pub struct WorkingMemory<P: LlmProvider> {
    /// 基础组件
    base: BaseComponent,
    /// 内存配置
    config: WorkingMemoryConfig,
    /// 内存数据
    data: Mutex<Vec<WorkingMemoryEntry>>,
    /// LLM提供者
    llm: Arc<P>,
    /// 内存模板
    template: String,
}

impl<P: LlmProvider> WorkingMemory<P> {
    /// 创建新的工作内存
    pub fn new(config: WorkingMemoryConfig, llm: Arc<P>) -> Self {
        let template = config.template.clone().unwrap_or_else(|| DEFAULT_TEMPLATE.to_string());
        let component_config = ComponentConfig {
            name: Some("WorkingMemory".to_string()),
            component: Component::Memory,
            log_level: Some(LogLevel::Info),
        };
        
        Self {
            base: BaseComponent::new(component_config),
            config,
            data: Mutex::new(Vec::new()),
            llm,
            template,
        }
    }
    
    /// 添加条目到工作内存
    pub fn add_entry(&self, content: String, content_type: Option<String>, metadata: Option<HashMap<String, Value>>) -> Result<WorkingMemoryEntry> {
        let timestamp = current_timestamp();
        let id = generate_id();
        
        let entry = WorkingMemoryEntry {
            id,
            content,
            content_type: content_type.unwrap_or_else(|| "text".to_string()),
            created_at: timestamp,
            updated_at: timestamp,
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
    pub fn get_all_entries(&self) -> Result<Vec<WorkingMemoryEntry>> {
        let data = self.data.lock().map_err(|_| Error::Internal("Failed to lock memory data".to_string()))?;
        Ok(data.clone())
    }
    
    /// 获取指定ID的条目
    pub fn get_entry(&self, id: &str) -> Result<Option<WorkingMemoryEntry>> {
        let data = self.data.lock().map_err(|_| Error::Internal("Failed to lock memory data".to_string()))?;
        Ok(data.iter().find(|entry| entry.id == id).cloned())
    }
    
    /// 更新条目
    pub fn update_entry(&self, id: &str, content: String) -> Result<Option<WorkingMemoryEntry>> {
        let mut data = self.data.lock().map_err(|_| Error::Internal("Failed to lock memory data".to_string()))?;
        
        if let Some(entry) = data.iter_mut().find(|entry| entry.id == id) {
            entry.content = content;
            entry.updated_at = current_timestamp();
            return Ok(Some(entry.clone()));
        }
        
        Ok(None)
    }
    
    /// 删除条目
    pub fn delete_entry(&self, id: &str) -> Result<bool> {
        let mut data = self.data.lock().map_err(|_| Error::Internal("Failed to lock memory data".to_string()))?;
        let initial_len = data.len();
        data.retain(|entry| entry.id != id);
        Ok(data.len() < initial_len)
    }
    
    /// 生成工作内存总结
    pub async fn generate_summary(&self) -> Result<String> {
        let entries = self.get_all_entries()?;
        if entries.is_empty() {
            return Ok("No information available in working memory.".to_string());
        }
        
        // 构建提示
        let entries_text = entries.iter()
            .map(|entry| format!("- {}: {}", entry.content_type, entry.content))
            .collect::<Vec<_>>()
            .join("\n");
        
        let prompt = format!("{}\n\n{}", self.template, entries_text);
        
        // 使用LLM生成总结
        let options = LlmOptions::default();
        self.llm.generate(&prompt, &options).await
    }
    
    /// 生成消息列表
    pub fn to_messages(&self) -> Result<Vec<Message>> {
        let summary = tokio::runtime::Runtime::new()
            .map_err(|e| Error::Internal(format!("Failed to create runtime: {}", e)))?
            .block_on(self.generate_summary())?;
        
        Ok(vec![Message {
            role: Role::System,
            content: summary,
            name: Some("working_memory".to_string()),
            metadata: None,
        }])
    }
}

impl<P: LlmProvider> Base for WorkingMemory<P> {
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
impl<P: LlmProvider + 'static> Memory for WorkingMemory<P> {
    async fn store(&self, message: &Message) -> Result<()> {
        // 只存储用户和助手的消息
        if message.role == Role::User || message.role == Role::Assistant {
            let content_type = if message.role == Role::User {
                "user_message"
            } else {
                "assistant_message"
            };
            
            let mut metadata = HashMap::new();
            if let Some(name) = &message.name {
                metadata.insert("name".to_string(), Value::String(name.clone()));
            }
            
            self.add_entry(message.content.clone(), Some(content_type.to_string()), Some(metadata))?;
        }
        Ok(())
    }
    
    async fn retrieve(&self, _config: &crate::memory::MemoryConfig) -> Result<Vec<Message>> {
        self.to_messages()
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

/// 默认工作内存模板
const DEFAULT_TEMPLATE: &str = r#"
You are an assistant with a working memory. Below is the information stored in your working memory.
Please create a concise and well-organized summary of this information to inform your responses.

WORKING MEMORY CONTENTS:
"#; 