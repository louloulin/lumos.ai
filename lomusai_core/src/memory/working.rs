//! 工作内存模块，提供工作内存的实现和操作

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::error::{Error, Result};
use crate::logger::Component;

/// 工作内存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryConfig {
    /// 是否启用工作内存
    pub enabled: bool,
    /// 内存模板
    pub template: Option<String>,
    /// 内容类型
    pub content_type: Option<String>,
    /// 最大容量
    pub max_capacity: Option<usize>,
}

/// 工作内存内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryContent {
    /// 内容
    pub content: Value,
    /// 内容类型
    pub content_type: String,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// 元数据
    pub metadata: HashMap<String, Value>,
}

impl Default for WorkingMemoryContent {
    fn default() -> Self {
        Self {
            content: Value::Object(serde_json::Map::new()),
            content_type: "application/json".to_string(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

/// 工作内存接口
#[async_trait]
pub trait WorkingMemory: Base + Send + Sync {
    /// 获取工作内存内容
    async fn get(&self) -> Result<WorkingMemoryContent>;
    
    /// 更新工作内存内容
    async fn update(&self, content: WorkingMemoryContent) -> Result<()>;
    
    /// 清空工作内存
    async fn clear(&self) -> Result<()>;
    
    /// 获取特定键的值
    async fn get_value(&self, key: &str) -> Result<Option<Value>> {
        let content = self.get().await?;
        if let Value::Object(map) = &content.content {
            Ok(map.get(key).cloned())
        } else {
            Ok(None)
        }
    }
    
    /// 设置特定键的值
    async fn set_value(&self, key: &str, value: Value) -> Result<()> {
        let mut content = self.get().await?;
        if let Value::Object(map) = &mut content.content {
            map.insert(key.to_string(), value);
            content.updated_at = chrono::Utc::now();
            self.update(content).await
        } else {
            Err(Error::Parsing(format!("工作内存内容不是对象: {:?}", content.content)))
        }
    }
    
    /// 删除特定键
    async fn delete_value(&self, key: &str) -> Result<()> {
        let mut content = self.get().await?;
        if let Value::Object(map) = &mut content.content {
            map.remove(key);
            content.updated_at = chrono::Utc::now();
            self.update(content).await
        } else {
            Err(Error::Parsing(format!("工作内存内容不是对象: {:?}", content.content)))
        }
    }
}

/// 基本的工作内存实现
pub struct BasicWorkingMemory {
    /// 基础组件
    base: BaseComponent,
    /// 工作内存配置
    config: WorkingMemoryConfig,
    /// 工作内存内容
    content: RwLock<WorkingMemoryContent>,
}

impl BasicWorkingMemory {
    /// 创建一个新的基本工作内存
    pub fn new(config: WorkingMemoryConfig) -> Self {
        let component_config = ComponentConfig {
            name: Some("BasicWorkingMemory".to_string()),
            component: Component::Memory,
            log_level: None,
        };
        
        let mut content = WorkingMemoryContent::default();
        if let Some(content_type) = &config.content_type {
            content.content_type = content_type.clone();
        }
        
        Self {
            base: BaseComponent::new(component_config),
            config,
            content: RwLock::new(content),
        }
    }
    
    /// 从模板创建工作内存
    pub fn from_template(config: WorkingMemoryConfig, template: &str) -> Result<Self> {
        let memory = Self::new(config);
        
        // 尝试解析模板
        match serde_json::from_str::<Value>(template) {
            Ok(template_value) => {
                if let Value::Object(_) = &template_value {
                    // 在独立的作用域内使用锁，确保锁在返回memory前被释放
                    {
                        let mut content = memory.content.write().unwrap();
                        content.content = template_value;
                        content.updated_at = chrono::Utc::now();
                    } // 锁在这里被释放
                    
                    Ok(memory)
                } else {
                    Err(Error::Parsing("模板必须是有效的JSON对象".to_string()))
                }
            },
            Err(e) => Err(Error::Parsing(format!("无法解析模板: {}", e))),
        }
    }
}

#[async_trait]
impl WorkingMemory for BasicWorkingMemory {
    async fn get(&self) -> Result<WorkingMemoryContent> {
        let content = self.content.read().unwrap();
        Ok(content.clone())
    }
    
    async fn update(&self, content: WorkingMemoryContent) -> Result<()> {
        // 检查容量限制
        if let Some(max_capacity) = self.config.max_capacity {
            let size = serde_json::to_string(&content.content)
                .map_err(Error::Json)?
                .len();
            
            if size > max_capacity {
                return Err(Error::Constraint(format!(
                    "工作内存内容超过最大容量限制: {} > {}", size, max_capacity
                )));
            }
        }
        
        let mut current = self.content.write().unwrap();
        *current = content;
        Ok(())
    }
    
    async fn clear(&self) -> Result<()> {
        let mut content = self.content.write().unwrap();
        *content = WorkingMemoryContent::default();
        if let Some(content_type) = &self.config.content_type {
            content.content_type = content_type.clone();
        }
        Ok(())
    }
}

impl Base for BasicWorkingMemory {
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

/// 创建工作内存（工厂函数）
pub fn create_working_memory(config: &WorkingMemoryConfig) -> Result<Box<dyn WorkingMemory>> {
    if !config.enabled {
        return Err(Error::Configuration("工作内存未启用".to_string()));
    }
    
    if let Some(template) = &config.template {
        BasicWorkingMemory::from_template(config.clone(), template)
            .map(|mem| Box::new(mem) as Box<dyn WorkingMemory>)
    } else {
        Ok(Box::new(BasicWorkingMemory::new(config.clone())) as Box<dyn WorkingMemory>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_working_memory() {
        // 创建配置
        let config = WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("application/json".to_string()),
            max_capacity: Some(1024),
        };
        
        // 创建工作内存
        let memory = BasicWorkingMemory::new(config);
        
        // 测试初始状态
        let content = memory.get().await.unwrap();
        assert_eq!(content.content_type, "application/json");
        
        // 测试设置值
        memory.set_value("test_key", Value::String("test_value".to_string())).await.unwrap();
        
        // 测试获取值
        let value = memory.get_value("test_key").await.unwrap();
        assert_eq!(value, Some(Value::String("test_value".to_string())));
        
        // 测试更新值
        memory.set_value("test_key", Value::Number(serde_json::Number::from(42))).await.unwrap();
        let value = memory.get_value("test_key").await.unwrap();
        assert_eq!(value, Some(Value::Number(serde_json::Number::from(42))));
        
        // 测试删除值
        memory.delete_value("test_key").await.unwrap();
        let value = memory.get_value("test_key").await.unwrap();
        assert_eq!(value, None);
        
        // 测试清空
        memory.set_value("test_key", Value::String("test_value".to_string())).await.unwrap();
        memory.clear().await.unwrap();
        let value = memory.get_value("test_key").await.unwrap();
        assert_eq!(value, None);
    }
    
    #[tokio::test]
    async fn test_working_memory_from_template() {
        // 创建模板
        let template = r#"{"initial_key": "initial_value", "nested": {"key": "value"}}"#;
        
        // 创建配置
        let config = WorkingMemoryConfig {
            enabled: true,
            template: Some(template.to_string()),
            content_type: Some("application/json".to_string()),
            max_capacity: Some(1024),
        };
        
        // 创建工作内存
        let memory = create_working_memory(&config).unwrap();
        
        // 测试初始值
        let value = memory.get_value("initial_key").await.unwrap();
        assert_eq!(value, Some(Value::String("initial_value".to_string())));
        
        // 测试嵌套值
        let content = memory.get().await.unwrap();
        if let Value::Object(map) = &content.content {
            if let Some(Value::Object(nested)) = map.get("nested") {
                if let Some(value) = nested.get("key") {
                    assert_eq!(value, &Value::String("value".to_string()));
                } else {
                    panic!("嵌套键不存在");
                }
            } else {
                panic!("嵌套对象不存在");
            }
        } else {
            panic!("内容不是对象");
        }
    }
    
    #[tokio::test]
    async fn test_working_memory_capacity_limit() {
        // 创建小容量配置
        let config = WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("application/json".to_string()),
            max_capacity: Some(10), // 非常小的容量限制
        };
        
        // 创建工作内存
        let memory = BasicWorkingMemory::new(config);
        
        // 尝试设置超过容量限制的值
        let large_value = "a".repeat(100);
        let result = memory.set_value("key", Value::String(large_value)).await;
        
        // 应该返回错误
        assert!(result.is_err());
        if let Err(Error::Constraint(msg)) = result {
            assert!(msg.contains("工作内存内容超过最大容量限制"));
        } else {
            panic!("期望容量限制错误，但得到: {:?}", result);
        }
    }
} 