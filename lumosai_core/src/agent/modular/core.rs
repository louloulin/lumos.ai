//! Agent核心组件 - 负责Agent的基本配置和元数据管理
//!
//! 这个组件是Agent的核心，负责管理Agent的配置、基本信息和核心功能。

use crate::agent::types::{AgentCapabilities, AgentState, AgentStatus};
use crate::agent::AgentConfig;
use crate::error::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Agent核心组件
///
/// 负责管理Agent的基本信息、配置和核心功能
/// 这是Agent的"身份"组件，定义了Agent是谁，能做什么
#[derive(Clone)]
pub struct AgentCore {
    /// Agent唯一标识符
    id: String,
    /// Agent名称
    name: String,
    /// Agent描述
    description: String,
    /// Agent配置
    config: Arc<AgentConfig>,
    /// Agent创建时间
    created_at: DateTime<Utc>,
    /// Agent最后更新时间
    updated_at: DateTime<Utc>,
    /// Agent版本
    version: String,
    /// Agent标签
    tags: Vec<String>,
    /// Agent元数据
    metadata: serde_json::Value,
}

impl AgentCore {
    /// 创建新的Agent核心
    pub fn new(config: AgentConfig) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        Ok(Self {
            id,
            name: config
                .name
                .clone()
                .unwrap_or_else(|| "Unnamed Agent".to_string()),
            description: config.description.clone().unwrap_or_default(),
            config: Arc::new(config),
            created_at: now,
            updated_at: now,
            version: "1.0.0".to_string(),
            tags: vec![],
            metadata: serde_json::json!({}),
        })
    }

    /// 从现有配置创建Agent核心（用于重建）
    pub fn with_id(id: String, config: AgentConfig) -> Result<Self> {
        let now = Utc::now();

        Ok(Self {
            id,
            name: config
                .name
                .clone()
                .unwrap_or_else(|| "Unnamed Agent".to_string()),
            description: config.description.clone().unwrap_or_default(),
            config: Arc::new(config),
            created_at: now,
            updated_at: now,
            version: "1.0.0".to_string(),
            tags: vec![],
            metadata: serde_json::json!({}),
        })
    }

    /// 获取Agent ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 获取Agent名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取Agent描述
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 获取Agent配置
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// 获取Agent创建时间
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// 获取Agent最后更新时间
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    /// 获取Agent版本
    pub fn version(&self) -> &str {
        &self.version
    }

    /// 获取Agent标签
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// 获取Agent元数据
    pub fn metadata(&self) -> &serde_json::Value {
        &self.metadata
    }

    /// 设置Agent名称
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    /// 设置Agent描述
    pub fn set_description(&mut self, description: String) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// 移除标签
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, metadata: serde_json::Value) {
        self.metadata = metadata;
        self.updated_at = Utc::now();
    }

    /// 更新元数据中的指定字段
    pub fn update_metadata(&mut self, key: &str, value: serde_json::Value) {
        if let Some(obj) = self.metadata.as_object_mut() {
            obj.insert(key.to_string(), value);
            self.updated_at = Utc::now();
        }
    }

    /// 获取元数据中的指定字段
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// 检查是否具有指定能力
    pub fn has_capability(&self, capability: &str) -> bool {
        self.config.capabilities.iter().any(|cap| cap == capability)
    }

    /// 获取所有能力
    pub fn capabilities(&self) -> &[String] {
        &self.config.capabilities
    }

    /// 检查是否处于活跃状态
    pub fn is_active(&self) -> bool {
        matches!(self.config.status, AgentStatus::Active)
    }

    /// 获取Agent类型
    pub fn agent_type(&self) -> &str {
        &self.config.agent_type
    }

    /// 获取Agent模型
    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// 获取租户ID
    pub fn tenant_id(&self) -> Option<&str> {
        self.config.tenant_id.as_deref()
    }

    /// 获取隔离级别
    pub fn isolation_level(&self) -> &crate::agent::types::IsolationLevel {
        &self.config.isolation_level
    }

    /// 验证Agent配置
    pub fn validate(&self) -> Result<()> {
        // 验证必需字段
        if self.name.is_empty() {
            return Err(crate::error::Error::Validation {
                field: "name".to_string(),
                message: "Agent name cannot be empty".to_string(),
            });
        }

        if self.config.model.is_empty() {
            return Err(crate::error::Error::Validation {
                field: "model".to_string(),
                message: "Agent model cannot be empty".to_string(),
            });
        }

        if self.config.llm_provider.is_empty() {
            return Err(crate::error::Error::Validation {
                field: "llm_provider".to_string(),
                message: "LLM provider cannot be empty".to_string(),
            });
        }

        // 验证能力配置
        for capability in &self.config.capabilities {
            if capability.is_empty() {
                return Err(crate::error::Error::Validation {
                    field: "capabilities".to_string(),
                    message: "Capability cannot be empty".to_string(),
                });
            }
        }

        Ok(())
    }

    /// 获取Agent的字符串表示
    pub fn to_string(&self) -> String {
        format!(
            "Agent(id={}, name={}, type={}, model={})",
            self.id,
            self.name,
            self.agent_type(),
            self.model()
        )
    }

    /// 获取Agent的详细信息
    pub fn to_details(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "name": self.name,
            "description": self.description,
            "agent_type": self.agent_type(),
            "model": self.model(),
            "status": self.config.status,
            "capabilities": self.capabilities(),
            "tenant_id": self.tenant_id(),
            "isolation_level": self.isolation_level(),
            "version": self.version,
            "tags": self.tags,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
            "metadata": self.metadata
        })
    }

    /// 克隆Agent核心（用于创建实例）
    pub fn clone_with_id(&self, new_id: String) -> Self {
        Self {
            id: new_id,
            name: self.name.clone(),
            description: self.description.clone(),
            config: self.config.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: self.version.clone(),
            tags: self.tags.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl std::fmt::Debug for AgentCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentCore")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("agent_type", &self.agent_type())
            .field("model", &self.model())
            .field("status", &self.config.status)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

impl PartialEq for AgentCore {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for AgentCore {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::{AgentStatus, IsolationLevel};

    #[test]
    fn test_agent_core_creation() {
        let config = AgentConfig {
            name: Some("Test Agent".to_string()),
            agent_type: "test".to_string(),
            model: "gpt-4".to_string(),
            llm_provider: "openai".to_string(),
            status: AgentStatus::Active,
            capabilities: vec!["chat".to_string()],
            ..Default::default()
        };

        let core = AgentCore::new(config).unwrap();

        assert!(!core.id().is_empty());
        assert_eq!(core.name(), "Test Agent");
        assert_eq!(core.agent_type(), "test");
        assert_eq!(core.model(), "gpt-4");
        assert!(core.is_active());
        assert!(core.has_capability("chat"));
    }

    #[test]
    fn test_agent_core_validation() {
        let mut config = AgentConfig::default();
        config.name = Some("".to_string());

        let core = AgentCore::new(config);
        assert!(core.is_err());
    }

    #[test]
    fn test_agent_core_tags() {
        let config = AgentConfig::default();
        let mut core = AgentCore::new(config).unwrap();

        core.add_tag("important".to_string());
        assert!(core.tags().contains(&"important".to_string()));

        core.remove_tag("important");
        assert!(!core.tags().contains(&"important".to_string()));
    }

    #[test]
    fn test_agent_core_metadata() {
        let config = AgentConfig::default();
        let mut core = AgentCore::new(config).unwrap();

        let metadata = serde_json::json!({"key": "value"});
        core.set_metadata(metadata);

        assert_eq!(core.get_metadata("key"), Some(&serde_json::json!("value")));
    }

    #[test]
    fn test_agent_core_clone() {
        let config = AgentConfig {
            name: Some("Original".to_string()),
            ..Default::default()
        };

        let core = AgentCore::new(config).unwrap();
        let cloned = core.clone_with_id("new-id".to_string());

        assert_eq!(cloned.id(), "new-id");
        assert_eq!(cloned.name(), "Original");
        assert_ne!(cloned.id(), core.id());
    }
}
