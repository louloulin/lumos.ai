//! Agent能力管理器 - 负责Agent的能力管理和描述
//!
//! 这个组件负责管理Agent的能力描述、能力注册和能力发现。

use crate::agent::modular::core::AgentCore;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Agent能力管理器
///
/// 负责管理Agent的能力信息，包括：
/// - 能力描述和分类
/// - 能力注册和发现
/// - 能力版本管理
/// - 能力依赖关系
pub struct AgentCapability {
    /// Agent核心引用
    core: AgentCore,
    /// 能力描述
    capabilities: HashMap<String, Capability>,
    /// 能力分类
    categories: HashMap<String, Vec<String>>,
    /// 能力依赖关系
    dependencies: HashMap<String, Vec<String>>,
    /// 能力版本
    versions: HashMap<String, String>,
    /// 能力元数据
    metadata: HashMap<String, serde_json::Value>,
}

/// 能力描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// 能力名称
    pub name: String,
    /// 能力描述
    pub description: String,
    /// 能力类型
    pub capability_type: CapabilityType,
    /// 能力分类
    pub category: String,
    /// 输入参数
    pub inputs: Vec<Parameter>,
    /// 输出参数
    pub outputs: Vec<Parameter>,
    /// 是否启用
    pub enabled: bool,
    /// 版本
    pub version: String,
    /// 标签
    pub tags: Vec<String>,
    /// 元数据
    pub metadata: serde_json::Value,
}

/// 能力类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityType {
    /// 对话能力
    Chat,
    /// 工具调用能力
    ToolCall,
    /// 数据处理能力
    DataProcessing,
    /// 分析能力
    Analysis,
    /// 生成能力
    Generation,
    /// 记忆能力
    Memory,
    /// 推理能力
    Reasoning,
    /// 自定义能力
    Custom(String),
}

/// 参数描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub param_type: ParameterType,
    /// 参数描述
    pub description: String,
    /// 是否必需
    pub required: bool,
    /// 默认值
    pub default_value: Option<serde_json::Value>,
    /// 验证规则
    pub validation: Option<ValidationRule>,
}

/// 参数类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    /// 字符串
    String,
    /// 数字
    Number,
    /// 布尔值
    Boolean,
    /// 数组
    Array,
    /// 对象
    Object,
    /// 自定义类型
    Custom(String),
}

/// 验证规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// 最小值（适用于数字）
    pub min: Option<f64>,
    /// 最大值（适用于数字）
    pub max: Option<f64>,
    /// 最小长度（适用于字符串和数组）
    pub min_length: Option<usize>,
    /// 最大长度（适用于字符串和数组）
    pub max_length: Option<usize>,
    /// 正则表达式（适用于字符串）
    pub pattern: Option<String>,
    /// 枚举值
    pub enum_values: Option<Vec<serde_json::Value>>,
    /// 自定义验证器
    pub custom_validator: Option<String>,
}

/// 能力查询过滤器
#[derive(Debug, Clone)]
pub struct CapabilityFilter {
    /// 能力类型过滤
    pub capability_type: Option<CapabilityType>,
    /// 分类过滤
    pub category: Option<String>,
    /// 标签过滤
    pub tags: Option<Vec<String>>,
    /// 启用状态过滤
    pub enabled: Option<bool>,
    /// 版本过滤
    pub version: Option<String>,
}

impl Default for CapabilityFilter {
    fn default() -> Self {
        Self {
            capability_type: None,
            category: None,
            tags: None,
            enabled: None,
            version: None,
        }
    }
}

impl AgentCapability {
    /// 创建新的Agent能力管理器
    pub fn new(core: AgentCore) -> Result<Self> {
        let mut manager = Self {
            core,
            capabilities: HashMap::new(),
            categories: HashMap::new(),
            dependencies: HashMap::new(),
            versions: HashMap::new(),
            metadata: HashMap::new(),
        };

        // 初始化默认能力
        manager.initialize_default_capabilities()?;

        Ok(manager)
    }

    /// 注册能力
    pub fn register_capability(&mut self, capability: Capability) -> Result<()> {
        let name = capability.name.clone();

        // 验证能力
        self.validate_capability(&capability)?;

        // 注册能力
        self.capabilities.insert(name.clone(), capability.clone());

        // 更新分类
        self.categories
            .entry(capability.category.clone())
            .or_insert_with(Vec::new)
            .push(name.clone());

        // 更新版本
        self.versions
            .insert(name.clone(), capability.version.clone());

        log::info!(
            "Capability '{}' registered for agent '{}'",
            name,
            self.core.id()
        );

        Ok(())
    }

    /// 注销能力
    pub fn unregister_capability(&mut self, name: &str) -> Result<()> {
        if let Some(capability) = self.capabilities.remove(name) {
            // 从分类中移除
            if let Some(category_capabilities) = self.categories.get_mut(&capability.category) {
                category_capabilities.retain(|cap_name| cap_name != name);
            }

            // 移除版本信息
            self.versions.remove(name);

            // 移除依赖关系
            self.dependencies.remove(name);

            log::info!(
                "Capability '{}' unregistered from agent '{}'",
                name,
                self.core.id()
            );

            Ok(())
        } else {
            Err(crate::error::Error::NotFound {
                message: format!("Capability '{}' not found", name),
            })
        }
    }

    /// 获取能力
    pub fn get_capability(&self, name: &str) -> Option<&Capability> {
        self.capabilities.get(name)
    }

    /// 获取所有能力
    pub fn get_capabilities(&self) -> Vec<&Capability> {
        self.capabilities.values().collect()
    }

    /// 查询能力
    pub fn query_capabilities(&self, filter: &CapabilityFilter) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|cap| {
                if let Some(ref capability_type) = filter.capability_type {
                    if cap.capability_type != *capability_type {
                        return false;
                    }
                }

                if let Some(ref category) = filter.category {
                    if cap.category != *category {
                        return false;
                    }
                }

                if let Some(ref tags) = filter.tags {
                    if !tags.iter().any(|tag| cap.tags.contains(tag)) {
                        return false;
                    }
                }

                if let Some(enabled) = filter.enabled {
                    if cap.enabled != enabled {
                        return false;
                    }
                }

                if let Some(ref version) = filter.version {
                    if cap.version != *version {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// 检查是否具有能力
    pub fn has_capability(&self, name: &str) -> bool {
        self.capabilities.contains_key(name)
    }

    /// 启用能力
    pub fn enable_capability(&mut self, name: &str) -> Result<()> {
        if let Some(capability) = self.capabilities.get_mut(name) {
            capability.enabled = true;
            log::info!(
                "Capability '{}' enabled for agent '{}'",
                name,
                self.core.id()
            );
            Ok(())
        } else {
            Err(crate::error::Error::NotFound {
                message: format!("Capability '{}' not found", name),
            })
        }
    }

    /// 禁用能力
    pub fn disable_capability(&mut self, name: &str) -> Result<()> {
        if let Some(capability) = self.capabilities.get_mut(name) {
            capability.enabled = false;
            log::info!(
                "Capability '{}' disabled for agent '{}'",
                name,
                self.core.id()
            );
            Ok(())
        } else {
            Err(crate::error::Error::NotFound {
                message: format!("Capability '{}' not found", name),
            })
        }
    }

    /// 添加能力依赖
    pub fn add_dependency(&mut self, capability: &str, dependency: &str) -> Result<()> {
        if !self.capabilities.contains_key(capability) {
            return Err(crate::error::Error::NotFound {
                message: format!("Capability '{}' not found", capability),
            });
        }

        if !self.capabilities.contains_key(dependency) {
            return Err(crate::error::Error::NotFound {
                message: format!("Dependency '{}' not found", dependency),
            });
        }

        self.dependencies
            .entry(capability.to_string())
            .or_insert_with(Vec::new)
            .push(dependency.to_string());

        Ok(())
    }

    /// 移除能力依赖
    pub fn remove_dependency(&mut self, capability: &str, dependency: &str) -> Result<()> {
        if let Some(deps) = self.dependencies.get_mut(capability) {
            deps.retain(|dep| dep != dependency);
        }
        Ok(())
    }

    /// 检查依赖关系
    pub fn check_dependencies(&self, capability: &str) -> Result<Vec<String>> {
        let mut missing_deps = Vec::new();

        if let Some(deps) = self.dependencies.get(capability) {
            for dep in deps {
                if !self.capabilities.contains_key(dep) {
                    missing_deps.push(dep.clone());
                }
            }
        }

        if missing_deps.is_empty() {
            Ok(vec![])
        } else {
            Ok(missing_deps)
        }
    }

    /// 获取能力分类
    pub fn get_categories(&self) -> Vec<&String> {
        self.categories.keys().collect()
    }

    /// 获取分类下的能力
    pub fn get_capabilities_by_category(&self, category: &str) -> Vec<&Capability> {
        if let Some(capability_names) = self.categories.get(category) {
            capability_names
                .iter()
                .filter_map(|name| self.capabilities.get(name))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 获取能力版本
    pub fn get_capability_version(&self, name: &str) -> Option<&str> {
        self.versions.get(name).map(|s| s.as_str())
    }

    /// 更新能力版本
    pub fn update_capability_version(&mut self, name: &str, version: String) -> Result<()> {
        if let Some(capability) = self.capabilities.get_mut(name) {
            capability.version = version.clone();
            self.versions.insert(name.to_string(), version);
            Ok(())
        } else {
            Err(crate::error::Error::NotFound {
                message: format!("Capability '{}' not found", name),
            })
        }
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// 获取能力统计信息
    pub fn get_capability_stats(&self) -> CapabilityStats {
        let total_capabilities = self.capabilities.len();
        let enabled_capabilities = self.capabilities.values().filter(|cap| cap.enabled).count();
        let total_categories = self.categories.len();

        let mut capability_type_counts = HashMap::new();
        for capability in self.capabilities.values() {
            *capability_type_counts
                .entry(capability.capability_type.clone())
                .or_insert(0) += 1;
        }

        CapabilityStats {
            total_capabilities,
            enabled_capabilities,
            total_categories,
            capability_type_counts,
        }
    }

    /// 验证能力
    fn validate_capability(&self, capability: &Capability) -> Result<()> {
        if capability.name.is_empty() {
            return Err(crate::error::Error::Validation {
                field: "name".to_string(),
                message: "Capability name cannot be empty".to_string(),
            });
        }

        if capability.description.is_empty() {
            return Err(crate::error::Error::Validation {
                field: "description".to_string(),
                message: "Capability description cannot be empty".to_string(),
            });
        }

        if capability.category.is_empty() {
            return Err(crate::error::Error::Validation {
                field: "category".to_string(),
                message: "Capability category cannot be empty".to_string(),
            });
        }

        // 验证参数
        for input in &capability.inputs {
            self.validate_parameter(input)?;
        }

        for output in &capability.outputs {
            self.validate_parameter(output)?;
        }

        Ok(())
    }

    /// 验证参数
    fn validate_parameter(&self, parameter: &Parameter) -> Result<()> {
        if parameter.name.is_empty() {
            return Err(crate::error::Error::Validation {
                field: "parameter.name".to_string(),
                message: "Parameter name cannot be empty".to_string(),
            });
        }

        if parameter.description.is_empty() {
            return Err(crate::error::Error::Validation {
                field: "parameter.description".to_string(),
                message: "Parameter description cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    /// 初始化默认能力
    fn initialize_default_capabilities(&mut self) -> Result<()> {
        // 注册对话能力
        self.register_capability(Capability {
            name: "chat".to_string(),
            description: "基本的对话能力，能够理解和回应文本消息".to_string(),
            capability_type: CapabilityType::Chat,
            category: "conversation".to_string(),
            inputs: vec![Parameter {
                name: "message".to_string(),
                param_type: ParameterType::String,
                description: "用户输入的消息".to_string(),
                required: true,
                default_value: None,
                validation: None,
            }],
            outputs: vec![Parameter {
                name: "response".to_string(),
                param_type: ParameterType::String,
                description: "AI助手的回应".to_string(),
                required: true,
                default_value: None,
                validation: None,
            }],
            enabled: true,
            version: "1.0.0".to_string(),
            tags: vec!["basic".to_string(), "conversation".to_string()],
            metadata: serde_json::json!({"priority": "high"}),
        })?;

        // 注册工具调用能力
        self.register_capability(Capability {
            name: "tool_call".to_string(),
            description: "调用外部工具的能力，能够执行复杂的任务".to_string(),
            capability_type: CapabilityType::ToolCall,
            category: "execution".to_string(),
            inputs: vec![
                Parameter {
                    name: "tool_name".to_string(),
                    param_type: ParameterType::String,
                    description: "要调用的工具名称".to_string(),
                    required: true,
                    default_value: None,
                    validation: None,
                },
                Parameter {
                    name: "arguments".to_string(),
                    param_type: ParameterType::Object,
                    description: "工具参数".to_string(),
                    required: true,
                    default_value: None,
                    validation: None,
                },
            ],
            outputs: vec![Parameter {
                name: "result".to_string(),
                param_type: ParameterType::Object,
                description: "工具执行结果".to_string(),
                required: true,
                default_value: None,
                validation: None,
            }],
            enabled: true,
            version: "1.0.0".to_string(),
            tags: vec!["advanced".to_string(), "execution".to_string()],
            metadata: serde_json::json!({"priority": "high"}),
        })?;

        // 注册记忆能力
        self.register_capability(Capability {
            name: "memory".to_string(),
            description: "记忆和检索信息的能力".to_string(),
            capability_type: CapabilityType::Memory,
            category: "cognition".to_string(),
            inputs: vec![Parameter {
                name: "query".to_string(),
                param_type: ParameterType::String,
                description: "要查询的信息".to_string(),
                required: true,
                default_value: None,
                validation: None,
            }],
            outputs: vec![Parameter {
                name: "memories".to_string(),
                param_type: ParameterType::Array,
                description: "检索到的记忆".to_string(),
                required: true,
                default_value: None,
                validation: None,
            }],
            enabled: true,
            version: "1.0.0".to_string(),
            tags: vec!["basic".to_string(), "cognition".to_string()],
            metadata: serde_json::json!({"priority": "medium"}),
        })?;

        Ok(())
    }
}

/// 能力统计信息
#[derive(Debug, Clone)]
pub struct CapabilityStats {
    /// 总能力数量
    pub total_capabilities: usize,
    /// 启用的能力数量
    pub enabled_capabilities: usize,
    /// 总分类数量
    pub total_categories: usize,
    /// 能力类型统计
    pub capability_type_counts: HashMap<CapabilityType, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_manager_creation() {
        let config = crate::agent::AgentConfig::default();
        let core = AgentCore::new(config).unwrap();
        let manager = AgentCapability::new(core).unwrap();

        let stats = manager.get_capability_stats();
        assert!(stats.total_capabilities > 0);
        assert!(stats.total_categories > 0);
    }

    #[test]
    fn test_capability_registration() {
        let config = crate::agent::AgentConfig::default();
        let core = AgentCore::new(config).unwrap();
        let mut manager = AgentCapability::new(core).unwrap();

        let capability = Capability {
            name: "test_capability".to_string(),
            description: "Test capability".to_string(),
            capability_type: CapabilityType::Custom("test".to_string()),
            category: "test".to_string(),
            inputs: vec![],
            outputs: vec![],
            enabled: true,
            version: "1.0.0".to_string(),
            tags: vec![],
            metadata: serde_json::json!({}),
        };

        assert!(manager.register_capability(capability).is_ok());
        assert!(manager.has_capability("test_capability"));
    }

    #[test]
    fn test_capability_query() {
        let config = crate::agent::AgentConfig::default();
        let core = AgentCore::new(config).unwrap();
        let manager = AgentCapability::new(core).unwrap();

        let filter = CapabilityFilter {
            category: Some("conversation".to_string()),
            enabled: Some(true),
            ..Default::default()
        };

        let capabilities = manager.query_capabilities(&filter);
        assert!(!capabilities.is_empty());
    }

    #[test]
    fn test_capability_dependencies() {
        let config = crate::agent::AgentConfig::default();
        let core = AgentCore::new(config).unwrap();
        let mut manager = AgentCapability::new(core).unwrap();

        // 添加依赖
        assert!(manager.add_dependency("chat", "tool_call").is_ok());

        // 检查依赖
        let missing_deps = manager.check_dependencies("chat").unwrap();
        assert!(missing_deps.is_empty());
    }
}
