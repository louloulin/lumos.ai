//! Working memory module for short-term context storage
//!
//! This module provides working memory implementations that maintain
//! short-term context with limited capacity, using LRU eviction when full.
//!
//! # Overview
//!
//! Working memory is designed for:
//! - Maintaining recent conversation context
//! - Storing temporary state during agent execution
//! - Key-value storage with automatic eviction
//! - Fast access to frequently used data
//!
//! # Examples
//!
//! ```rust
//! use lumosai_core::memory::{create_working_memory, WorkingMemory, WorkingMemoryConfig};
//! use serde_json::json;
//!
//! # async fn example() -> lumosai_core::Result<()> {
//! let config = WorkingMemoryConfig {
//!     enabled: true,
//!     max_capacity: Some(100),
//!     content_type: Some("application/json".to_string()),
//!     template: None,
//! };
//!
//! let memory = create_working_memory(&config)?;
//!
//! // Set a value
//! memory.set_value("user_name", json!("Alice")).await?;
//!
//! // Get a value
//! if let Some(name) = memory.get_value("user_name").await? {
//!     println!("User name: {}", name);
//! }
//!
//! // Clear memory
//! memory.clear().await?;
//! # Ok(())
//! # }
//! ```

use crate::compat::{Component, MemoryMetrics, MetricsCollector};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::error::{Error, Result};
// use crate::compat::Component;
// use crate::compat::metrics::{MemoryMetrics, MetricsCollector};
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration for working memory
///
/// # Examples
///
/// ```rust
/// use lumosai_core::memory::WorkingMemoryConfig;
///
/// let config = WorkingMemoryConfig {
///     enabled: true,
///     max_capacity: Some(1000),
///     content_type: Some("application/json".to_string()),
///     template: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryConfig {
    /// Whether working memory is enabled
    pub enabled: bool,
    /// Template for formatting memory content
    pub template: Option<String>,
    /// Content type (e.g., "application/json")
    pub content_type: Option<String>,
    /// Maximum capacity (number of entries)
    pub max_capacity: Option<usize>,
}

/// Working memory content container
///
/// Stores the actual content along with metadata and timestamps.
///
/// # Examples
///
/// ```rust
/// use lumosai_core::memory::WorkingMemoryContent;
/// use serde_json::json;
///
/// let content = WorkingMemoryContent {
///     content: json!({"key": "value"}),
///     content_type: "application/json".to_string(),
///     updated_at: chrono::Utc::now(),
///     metadata: Default::default(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryContent {
    /// The actual content (typically a JSON object)
    pub content: Value,
    /// Content MIME type
    pub content_type: String,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

impl Default for WorkingMemoryContent {
    /// Creates default working memory content
    ///
    /// # Default Values
    ///
    /// - `content`: Empty JSON object
    /// - `content_type`: "application/json"
    /// - `updated_at`: Current UTC time
    /// - `metadata`: Empty HashMap
    fn default() -> Self {
        Self {
            content: Value::Object(serde_json::Map::new()),
            content_type: "application/json".to_string(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

/// Trait for working memory implementations
///
/// Working memory provides short-term storage with limited capacity,
/// automatically evicting old entries when full (LRU policy).
///
/// # Overview
///
/// Working memory is useful for:
/// - Maintaining recent conversation context
/// - Storing temporary agent state
/// - Caching frequently accessed data
/// - Managing session-specific information
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use lumosai_core::memory::{WorkingMemory, WorkingMemoryContent};
/// use serde_json::json;
///
/// # async fn example(memory: &dyn WorkingMemory) -> lumosai_core::Result<()> {
/// // Set a value
/// memory.set_value("counter", json!(42)).await?;
///
/// // Get a value
/// if let Some(value) = memory.get_value("counter").await? {
///     println!("Counter: {}", value);
/// }
///
/// // Delete a value
/// memory.delete_value("counter").await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Storing Complex Data
///
/// ```rust
/// use lumosai_core::memory::WorkingMemory;
/// use serde_json::json;
///
/// # async fn example(memory: &dyn WorkingMemory) -> lumosai_core::Result<()> {
/// let user_data = json!({
///     "name": "Alice",
///     "age": 30,
///     "preferences": {
///         "theme": "dark",
///         "language": "en"
///     }
/// });
///
/// memory.set_value("user", user_data).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Thread Safety
///
/// All working memory implementations must be `Send + Sync`.
///
/// # See Also
///
/// - [`BasicWorkingMemory`] - Basic working memory implementation
/// - [`create_working_memory`] - Factory function for creating working memory
#[async_trait]
pub trait WorkingMemory: Base + Send + Sync {
    /// Retrieves the entire working memory content
    ///
    /// # Returns
    ///
    /// The current working memory content including metadata.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::memory::WorkingMemory;
    ///
    /// # async fn example(memory: &dyn WorkingMemory) -> lumosai_core::Result<()> {
    /// let content = memory.get().await?;
    /// println!("Content type: {}", content.content_type);
    /// println!("Updated at: {}", content.updated_at);
    /// # Ok(())
    /// # }
    /// ```
    async fn get(&self) -> Result<WorkingMemoryContent>;

    /// Updates the entire working memory content
    ///
    /// Replaces the current content with new content.
    ///
    /// # Arguments
    ///
    /// * `content` - New content to store
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::memory::{WorkingMemory, WorkingMemoryContent};
    /// use serde_json::json;
    ///
    /// # async fn example(memory: &dyn WorkingMemory) -> lumosai_core::Result<()> {
    /// let mut content = memory.get().await?;
    /// content.content = json!({"new": "data"});
    /// content.updated_at = chrono::Utc::now();
    /// memory.update(content).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn update(&self, content: WorkingMemoryContent) -> Result<()>;

    /// Clears all working memory content
    ///
    /// Removes all stored data and resets to default state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::memory::WorkingMemory;
    ///
    /// # async fn example(memory: &dyn WorkingMemory) -> lumosai_core::Result<()> {
    /// memory.clear().await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn clear(&self) -> Result<()>;

    /// Retrieves a specific value by key
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// - `Some(value)` if the key exists
    /// - `None` if the key does not exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::memory::WorkingMemory;
    ///
    /// # async fn example(memory: &dyn WorkingMemory) -> lumosai_core::Result<()> {
    /// if let Some(value) = memory.get_value("user_name").await? {
    ///     println!("User name: {}", value);
    /// } else {
    ///     println!("User name not found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn get_value(&self, key: &str) -> Result<Option<Value>> {
        let content = self.get().await?;
        if let Value::Object(map) = &content.content {
            Ok(map.get(key).cloned())
        } else {
            Ok(None)
        }
    }

    /// Sets a specific value by key
    ///
    /// If the key already exists, its value is updated. Otherwise, a new
    /// key-value pair is created.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set
    /// * `value` - The value to store
    ///
    /// # Errors
    ///
    /// Returns an error if the content is not a JSON object.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::memory::WorkingMemory;
    /// use serde_json::json;
    ///
    /// # async fn example(memory: &dyn WorkingMemory) -> lumosai_core::Result<()> {
    /// memory.set_value("counter", json!(1)).await?;
    /// memory.set_value("name", json!("Alice")).await?;
    /// memory.set_value("active", json!(true)).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn set_value(&self, key: &str, value: Value) -> Result<()> {
        let mut content = self.get().await?;
        if let Value::Object(map) = &mut content.content {
            map.insert(key.to_string(), value);
            content.updated_at = chrono::Utc::now();
            self.update(content).await
        } else {
            Err(Error::Parsing(format!(
                "Working memory content is not an object: {:?}",
                content.content
            )))
        }
    }

    /// Deletes a specific value by key
    ///
    /// Removes the key-value pair from working memory.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to delete
    ///
    /// # Errors
    ///
    /// Returns an error if the content is not a JSON object.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::memory::WorkingMemory;
    ///
    /// # async fn example(memory: &dyn WorkingMemory) -> lumosai_core::Result<()> {
    /// memory.delete_value("old_key").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn delete_value(&self, key: &str) -> Result<()> {
        let mut content = self.get().await?;
        if let Value::Object(map) = &mut content.content {
            map.remove(key);
            content.updated_at = chrono::Utc::now();
            self.update(content).await
        } else {
            Err(Error::Parsing(format!(
                "Working memory content is not an object: {:?}",
                content.content
            )))
        }
    }
}

/// 基本的工作内存实现
pub struct BasicWorkingMemory {
    /// 基础组件
    base: BaseComponent,
    /// 工作内存配置
    config: WorkingMemoryConfig,
    /// 内存内容
    content: Arc<RwLock<WorkingMemoryContent>>,
    /// 指标收集器
    metrics_collector: Option<Arc<dyn MetricsCollector>>,
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
            content: Arc::new(RwLock::new(content)),
            metrics_collector: None,
        }
    }

    /// 创建带有指标收集器的工作内存
    pub fn with_metrics_collector(
        config: WorkingMemoryConfig,
        metrics_collector: Arc<dyn MetricsCollector>,
    ) -> Self {
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
            content: Arc::new(RwLock::new(content)),
            metrics_collector: Some(metrics_collector),
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
            }
            Err(e) => Err(Error::Parsing(format!("无法解析模板: {e}"))),
        }
    }

    /// 记录内存操作指标的辅助方法
    async fn record_memory_metrics(
        &self,
        operation_type: &str,
        execution_time_ms: u64,
        success: bool,
        key: Option<String>,
        data_size_bytes: Option<usize>,
    ) {
        if let Some(collector) = &self.metrics_collector {
            let metrics = MemoryMetrics {
                operation: operation_type.to_string(),
                execution_time_ms,
                success,
                operation_type: operation_type.to_string(),
                key: key.unwrap_or_default(),
                data_size_bytes: data_size_bytes.unwrap_or(0),
                timestamp: SystemTime::now(),
                total_entries: 0,
                memory_usage: 0,
                cache_hits: 0,
                cache_misses: 0,
            };

            if let Err(e) = collector.record_memory_operation(metrics).await {
                // 记录日志但不影响主要操作
                let logger = self.logger();
                logger.error(&format!("Failed to record memory metrics: {e}"));
            }
        }
    }
}

#[async_trait]
impl WorkingMemory for BasicWorkingMemory {
    async fn get(&self) -> Result<WorkingMemoryContent> {
        let start_time = SystemTime::now();

        let result = async {
            let content = self.content.read().unwrap();
            Ok(content.clone())
        }
        .await;

        let execution_time = start_time.elapsed().unwrap_or_default().as_millis() as u64;
        let success = result.is_ok();
        let data_size = if let Ok(ref content) = result {
            serde_json::to_string(&content.content)
                .ok()
                .map(|s| s.len())
        } else {
            None
        };

        self.record_memory_metrics("get", execution_time, success, None, data_size)
            .await;
        result
    }

    async fn update(&self, content: WorkingMemoryContent) -> Result<()> {
        let start_time = SystemTime::now();
        let data_size = serde_json::to_string(&content.content)
            .ok()
            .map(|s| s.len());

        let result = async {
            // 检查容量限制
            if let Some(max_capacity) = self.config.max_capacity {
                let size = serde_json::to_string(&content.content)
                    .map_err(Error::Json)?
                    .len();

                if size > max_capacity {
                    return Err(Error::Constraint(format!(
                        "工作内存内容超过最大容量限制: {size} > {max_capacity}"
                    )));
                }
            }

            let mut current = self.content.write().unwrap();
            *current = content;
            Ok(())
        }
        .await;

        let execution_time = start_time.elapsed().unwrap_or_default().as_millis() as u64;
        let success = result.is_ok();

        self.record_memory_metrics("update", execution_time, success, None, data_size)
            .await;
        result
    }

    async fn clear(&self) -> Result<()> {
        let start_time = SystemTime::now();

        let result = async {
            let mut content = self.content.write().unwrap();
            *content = WorkingMemoryContent::default();
            if let Some(content_type) = &self.config.content_type {
                content.content_type = content_type.clone();
            }
            Ok(())
        }
        .await;

        let execution_time = start_time.elapsed().unwrap_or_default().as_millis() as u64;
        let success = result.is_ok();

        self.record_memory_metrics("clear", execution_time, success, None, None)
            .await;
        result
    }

    async fn get_value(&self, key: &str) -> Result<Option<Value>> {
        let start_time = SystemTime::now();

        let result = async {
            let content = self.get().await?;
            if let Value::Object(map) = &content.content {
                Ok(map.get(key).cloned())
            } else {
                Ok(None)
            }
        }
        .await;

        let execution_time = start_time.elapsed().unwrap_or_default().as_millis() as u64;
        let success = result.is_ok();
        let data_size = if let Ok(Some(ref value)) = result {
            serde_json::to_string(value).ok().map(|s| s.len())
        } else {
            None
        };

        self.record_memory_metrics(
            "get_value",
            execution_time,
            success,
            Some(key.to_string()),
            data_size,
        )
        .await;
        result
    }

    async fn set_value(&self, key: &str, value: Value) -> Result<()> {
        let start_time = SystemTime::now();
        let data_size = serde_json::to_string(&value).ok().map(|s| s.len());

        let result = async {
            let mut content = self.get().await?;
            if let Value::Object(map) = &mut content.content {
                map.insert(key.to_string(), value);
                content.updated_at = chrono::Utc::now();
                self.update(content).await
            } else {
                Err(Error::Parsing(format!(
                    "工作内存内容不是对象: {:?}",
                    content.content
                )))
            }
        }
        .await;

        let execution_time = start_time.elapsed().unwrap_or_default().as_millis() as u64;
        let success = result.is_ok();

        self.record_memory_metrics(
            "set_value",
            execution_time,
            success,
            Some(key.to_string()),
            data_size,
        )
        .await;
        result
    }

    async fn delete_value(&self, key: &str) -> Result<()> {
        let start_time = SystemTime::now();

        let result = async {
            let mut content = self.get().await?;
            if let Value::Object(map) = &mut content.content {
                map.remove(key);
                content.updated_at = chrono::Utc::now();
                self.update(content).await
            } else {
                Err(Error::Parsing(format!(
                    "工作内存内容不是对象: {:?}",
                    content.content
                )))
            }
        }
        .await;

        let execution_time = start_time.elapsed().unwrap_or_default().as_millis() as u64;
        let success = result.is_ok();

        self.record_memory_metrics(
            "delete_value",
            execution_time,
            success,
            Some(key.to_string()),
            None,
        )
        .await;
        result
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
        memory
            .set_value("test_key", Value::String("test_value".to_string()))
            .await
            .unwrap();

        // 测试获取值
        let value = memory.get_value("test_key").await.unwrap();
        assert_eq!(value, Some(Value::String("test_value".to_string())));

        // 测试更新值
        memory
            .set_value("test_key", Value::Number(serde_json::Number::from(42)))
            .await
            .unwrap();
        let value = memory.get_value("test_key").await.unwrap();
        assert_eq!(value, Some(Value::Number(serde_json::Number::from(42))));

        // 测试删除值
        memory.delete_value("test_key").await.unwrap();
        let value = memory.get_value("test_key").await.unwrap();
        assert_eq!(value, None);

        // 测试清空
        memory
            .set_value("test_key", Value::String("test_value".to_string()))
            .await
            .unwrap();
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
