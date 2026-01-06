//! Tool registration and discovery mechanism
//!
//! Provides dynamic tool registration, discovery and management functionality, similar to Mastra's tool system

use crate::compat::Component;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::error::Result;
// use crate::compat::{Component, Logger};
use crate::tool::{Tool, ToolSchema};
use regex::Regex;

/// Tool category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ToolCategory {
    /// File operation tools
    FileSystem,
    /// Network request tools
    Network,
    /// Data processing tools
    DataProcessing,
    /// Math calculation tools
    Math,
    /// Text processing tools
    Text,
    /// Image processing tools
    Image,
    /// Audio processing tools
    Audio,
    /// Database operation tools
    Database,
    /// API integration tools
    ApiIntegration,
    /// Custom tools
    Custom(String),
}

/// Tool metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool version
    pub version: String,
    /// Tool author
    pub author: Option<String>,
    /// Tool category
    pub category: ToolCategory,
    /// Tool tags
    pub tags: Vec<String>,
    /// Requires authentication
    pub requires_auth: bool,
    /// Tool permission requirements
    pub permissions: Vec<String>,
    /// Tool dependencies
    pub dependencies: Vec<String>,
}

/// Tool registry
pub struct ToolRegistry {
    /// Base component
    base: BaseComponent,
    /// Registered tools
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    /// Tool metadata
    metadata: Arc<RwLock<HashMap<String, ToolMetadata>>>,
    /// Category index
    category_index: Arc<RwLock<HashMap<ToolCategory, Vec<String>>>>,
    /// Tag index
    tag_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl ToolRegistry {
    /// Create new tool registry
    pub fn new() -> Self {
        let component_config = ComponentConfig {
            name: Some("ToolRegistry".to_string()),
            component: Component::Tool,
            log_level: None,
        };

        Self {
            base: BaseComponent::new(component_config),
            tools: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            category_index: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register tool
    pub fn register_tool(&self, tool: Arc<dyn Tool>, metadata: ToolMetadata) -> Result<()> {
        let tool_name = metadata.name.clone();

        // Check if tool already exists
        {
            let tools = self.tools.read().map_err(|_| {
                crate::error::Error::Internal("Failed to acquire read lock".to_string())
            })?;
            if tools.contains_key(&tool_name) {
                return Err(crate::error::Error::Internal(format!(
                    "Tool '{tool_name}' is already registered"
                )));
            }
        }

        // Register tool
        {
            let mut tools = self.tools.write().map_err(|_| {
                crate::error::Error::Internal("Failed to acquire write lock".to_string())
            })?;
            tools.insert(tool_name.clone(), tool);
        }

        // Store metadata
        {
            let mut metadata_map = self.metadata.write().map_err(|_| {
                crate::error::Error::Internal("Failed to acquire write lock".to_string())
            })?;
            metadata_map.insert(tool_name.clone(), metadata.clone());
        }

        // Update category index
        {
            let mut category_index = self.category_index.write().map_err(|_| {
                crate::error::Error::Internal("Failed to acquire write lock".to_string())
            })?;
            category_index
                .entry(metadata.category.clone())
                .or_insert_with(Vec::new)
                .push(tool_name.clone());
        }

        // Update tag index
        {
            let mut tag_index = self.tag_index.write().map_err(|_| {
                crate::error::Error::Internal("Failed to acquire write lock".to_string())
            })?;
            for tag in &metadata.tags {
                tag_index
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(tool_name.clone());
            }
        }

        let _ = self
            .base
            .logger()
            .info(&format!("Tool '{tool_name}' registered successfully"));
        Ok(())
    }

    /// Unregister tool
    pub fn unregister_tool(&self, tool_name: &str) -> Result<()> {
        // Get metadata
        let metadata = {
            let metadata_map = self.metadata.read().map_err(|_| {
                crate::error::Error::Internal("Failed to acquire read lock".to_string())
            })?;
            metadata_map.get(tool_name).cloned()
        };

        let metadata = metadata.ok_or_else(|| {
            crate::error::Error::Internal(format!("Tool '{tool_name}' not found"))
        })?;

        // Remove tool
        {
            let mut tools = self.tools.write().map_err(|_| {
                crate::error::Error::Internal("Failed to acquire write lock".to_string())
            })?;
            tools.remove(tool_name);
        }

        // Remove metadata
        {
            let mut metadata_map = self.metadata.write().map_err(|_| {
                crate::error::Error::Internal("Failed to acquire write lock".to_string())
            })?;
            metadata_map.remove(tool_name);
        }

        // Update category index
        {
            let mut category_index = self.category_index.write().map_err(|_| {
                crate::error::Error::Internal("Failed to acquire write lock".to_string())
            })?;
            if let Some(tools_in_category) = category_index.get_mut(&metadata.category) {
                tools_in_category.retain(|name| name != tool_name);
                if tools_in_category.is_empty() {
                    category_index.remove(&metadata.category);
                }
            }
        }

        // Update tag index
        {
            let mut tag_index = self.tag_index.write().map_err(|_| {
                crate::error::Error::Internal("Failed to acquire write lock".to_string())
            })?;
            for tag in &metadata.tags {
                if let Some(tools_with_tag) = tag_index.get_mut(tag) {
                    tools_with_tag.retain(|name| name != tool_name);
                    if tools_with_tag.is_empty() {
                        tag_index.remove(tag);
                    }
                }
            }
        }

        let _ = self
            .base
            .logger()
            .info(&format!("Tool '{tool_name}' unregistered successfully"));
        Ok(())
    }

    /// Get tool
    pub fn get_tool(&self, tool_name: &str) -> Result<Option<Arc<dyn Tool>>> {
        let tools = self.tools.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;
        Ok(tools.get(tool_name).cloned())
    }

    /// Get all tool names
    pub fn list_tools(&self) -> Result<Vec<String>> {
        let tools = self.tools.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;
        Ok(tools.keys().cloned().collect())
    }

    /// Find tools by category
    pub fn find_tools_by_category(&self, category: &ToolCategory) -> Result<Vec<String>> {
        let category_index = self.category_index.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;
        Ok(category_index.get(category).cloned().unwrap_or_default())
    }

    /// Find tools by tag
    pub fn find_tools_by_tag(&self, tag: &str) -> Result<Vec<String>> {
        let tag_index = self.tag_index.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;
        Ok(tag_index.get(tag).cloned().unwrap_or_default())
    }

    /// Search tools
    pub fn search_tools(&self, query: &str) -> Result<Vec<String>> {
        let metadata_map = self.metadata.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;
        let query_lower = query.to_lowercase();

        let mut results = Vec::new();
        for (tool_name, metadata) in metadata_map.iter() {
            if metadata.name.to_lowercase().contains(&query_lower)
                || metadata.description.to_lowercase().contains(&query_lower)
                || metadata
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&query_lower))
            {
                results.push(tool_name.clone());
            }
        }

        Ok(results)
    }

    /// Get tool metadata
    pub fn get_metadata(&self, tool_name: &str) -> Result<Option<ToolMetadata>> {
        let metadata_map = self.metadata.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;
        Ok(metadata_map.get(tool_name).cloned())
    }

    /// Get all categories
    pub fn list_categories(&self) -> Result<Vec<ToolCategory>> {
        let category_index = self.category_index.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;
        Ok(category_index.keys().cloned().collect())
    }

    /// Get all tags
    pub fn list_tags(&self) -> Result<Vec<String>> {
        let tag_index = self.tag_index.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;
        Ok(tag_index.keys().cloned().collect())
    }

    /// Get tool statistics
    pub fn get_stats(&self) -> Result<ToolRegistryStats> {
        let tools = self.tools.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;
        let category_index = self.category_index.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;
        let tag_index = self.tag_index.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;

        Ok(ToolRegistryStats {
            total_tools: tools.len(),
            total_categories: category_index.len(),
            total_tags: tag_index.len(),
        })
    }

    /// 解析工具依赖 - 返回工具及其所有依赖
    ///
    /// 按照依赖顺序返回工具列表，确保依赖的工具在依赖它的工具之前
    pub fn resolve_dependencies(&self, tool_name: &str) -> Result<Vec<Arc<dyn Tool>>> {
        let metadata_map = self.metadata.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;
        let tools = self.tools.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;

        // 获取工具元数据
        let metadata = metadata_map.get(tool_name).ok_or_else(|| {
            crate::error::Error::NotFound(format!("Tool '{}' not found", tool_name))
        })?;

        let mut resolved = Vec::new();
        let mut visited = std::collections::HashSet::new();

        // 递归解析依赖
        self.resolve_dependencies_recursive(
            tool_name,
            &metadata_map,
            &tools,
            &mut resolved,
            &mut visited,
        )?;

        Ok(resolved)
    }

    /// 递归解析依赖（内部方法）
    fn resolve_dependencies_recursive(
        &self,
        tool_name: &str,
        metadata_map: &std::sync::RwLockReadGuard<HashMap<String, ToolMetadata>>,
        tools: &std::sync::RwLockReadGuard<HashMap<String, Arc<dyn Tool>>>,
        resolved: &mut Vec<Arc<dyn Tool>>,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        // 检查循环依赖
        if visited.contains(tool_name) {
            return Err(crate::error::Error::Internal(format!(
                "Circular dependency detected for tool '{}'",
                tool_name
            )));
        }

        visited.insert(tool_name.to_string());

        // 获取工具元数据
        let metadata = metadata_map.get(tool_name).ok_or_else(|| {
            crate::error::Error::NotFound(format!("Tool '{}' not found", tool_name))
        })?;

        // 先解析所有依赖
        for dep_name in &metadata.dependencies {
            if !resolved.iter().any(|t| t.id() == dep_name) {
                // 检查依赖是否存在
                if !tools.contains_key(dep_name) {
                    return Err(crate::error::Error::NotFound(format!(
                        "Dependency '{}' for tool '{}' not found",
                        dep_name, tool_name
                    )));
                }

                // 递归解析依赖
                self.resolve_dependencies_recursive(
                    dep_name,
                    metadata_map,
                    tools,
                    resolved,
                    visited,
                )?;
            }
        }

        // 添加当前工具
        if let Some(tool) = tools.get(tool_name) {
            resolved.push(tool.clone());
        }

        visited.remove(tool_name);

        Ok(())
    }

    /// 发现工具（支持模式匹配）
    ///
    /// 支持通配符模式，如 "calc*", "*search*" 等
    pub fn discover(&self, pattern: &str) -> Result<Vec<Arc<dyn Tool>>> {
        let tools = self.tools.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;

        // 简单的通配符匹配（支持 * 和 ?）
        let pattern_lower = pattern.to_lowercase();
        let regex_pattern = pattern_lower.replace("*", ".*").replace("?", ".");
        let regex = Regex::new(&format!("^{}$", regex_pattern)).map_err(|e| {
            crate::error::Error::Internal(format!("Invalid pattern '{}': {}", pattern, e))
        })?;

        let mut results = Vec::new();
        for (name, tool) in tools.iter() {
            if regex.is_match(&name.to_lowercase()) {
                results.push(tool.clone());
            }
        }

        Ok(results)
    }

    /// 获取工具的版本信息
    pub fn get_tool_version(&self, tool_name: &str) -> Result<Option<String>> {
        let metadata_map = self.metadata.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;
        Ok(metadata_map.get(tool_name).map(|m| m.version.clone()))
    }

    /// 检查工具版本兼容性
    pub fn check_version_compatibility(
        &self,
        tool_name: &str,
        required_version: &str,
    ) -> Result<bool> {
        let metadata = self.get_metadata(tool_name)?;
        let metadata = metadata.ok_or_else(|| {
            crate::error::Error::NotFound(format!("Tool '{}' not found", tool_name))
        })?;

        // 简单的版本比较（可以使用 semver 库进行更精确的比较）
        Ok(metadata.version == required_version || metadata.version.starts_with(required_version))
    }

    /// 获取所有工具及其依赖关系图
    pub fn get_dependency_graph(&self) -> Result<HashMap<String, Vec<String>>> {
        let metadata_map = self.metadata.read().map_err(|_| {
            crate::error::Error::Internal("Failed to acquire read lock".to_string())
        })?;

        let mut graph = HashMap::new();
        for (tool_name, metadata) in metadata_map.iter() {
            graph.insert(tool_name.clone(), metadata.dependencies.clone());
        }

        Ok(graph)
    }
}

/// Tool registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRegistryStats {
    /// Total number of tools
    pub total_tools: usize,
    /// Total number of categories
    pub total_categories: usize,
    /// Total number of tags
    pub total_tags: usize,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{FunctionTool, ParameterSchema, ToolSchema};

    fn create_test_tool(name: &str, dependencies: Vec<String>) -> (Arc<dyn Tool>, ToolMetadata) {
        let schema = ToolSchema::new(vec![ParameterSchema {
            name: "input".to_string(),
            description: "Input parameter".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        }]);

        let tool = Arc::new(FunctionTool::new(name, "Test tool", schema, |params| {
            let input = params
                .get("input")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            Ok(serde_json::json!({ "result": format!("Processed: {}", input) }))
        }));

        let metadata = ToolMetadata {
            name: name.to_string(),
            description: format!("Test tool {}", name),
            version: "1.0.0".to_string(),
            author: Some("Test".to_string()),
            category: ToolCategory::Custom("test".to_string()),
            tags: vec!["test".to_string()],
            requires_auth: false,
            permissions: vec![],
            dependencies,
        };

        (tool, metadata)
    }

    #[tokio::test]
    async fn test_resolve_dependencies() {
        let registry = ToolRegistry::new();

        // 创建依赖工具
        let (dep_tool, dep_metadata) = create_test_tool("dependency_tool", vec![]);
        registry.register_tool(dep_tool, dep_metadata).unwrap();

        // 创建依赖 dependency_tool 的工具
        let (tool, metadata) = create_test_tool("main_tool", vec!["dependency_tool".to_string()]);
        registry.register_tool(tool, metadata).unwrap();

        // 解析依赖
        let resolved = registry.resolve_dependencies("main_tool").unwrap();

        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].id(), "dependency_tool");
        assert_eq!(resolved[1].id(), "main_tool");
    }

    #[tokio::test]
    async fn test_resolve_dependencies_circular() {
        let registry = ToolRegistry::new();

        // 创建循环依赖
        let (tool1, metadata1) = create_test_tool("tool1", vec!["tool2".to_string()]);
        let (tool2, metadata2) = create_test_tool("tool2", vec!["tool1".to_string()]);

        registry.register_tool(tool1, metadata1).unwrap();
        registry.register_tool(tool2, metadata2).unwrap();

        // 应该检测到循环依赖
        let result = registry.resolve_dependencies("tool1");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency"));
    }

    #[tokio::test]
    async fn test_discover_tools() {
        let registry = ToolRegistry::new();

        // 注册多个工具
        let (tool1, metadata1) = create_test_tool("calculator", vec![]);
        let (tool2, metadata2) = create_test_tool("web_search", vec![]);
        let (tool3, metadata3) = create_test_tool("file_manager", vec![]);

        registry.register_tool(tool1, metadata1).unwrap();
        registry.register_tool(tool2, metadata2).unwrap();
        registry.register_tool(tool3, metadata3).unwrap();

        // 测试通配符匹配
        let results = registry.discover("calc*").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id(), "calculator");

        let results = registry.discover("*search*").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id(), "web_search");
    }

    #[tokio::test]
    async fn test_get_tool_version() {
        let registry = ToolRegistry::new();

        let (tool, metadata) = create_test_tool("test_tool", vec![]);
        registry.register_tool(tool, metadata).unwrap();

        let version = registry.get_tool_version("test_tool").unwrap();
        assert_eq!(version, Some("1.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_check_version_compatibility() {
        let registry = ToolRegistry::new();

        let (tool, metadata) = create_test_tool("test_tool", vec![]);
        registry.register_tool(tool, metadata).unwrap();

        assert!(registry
            .check_version_compatibility("test_tool", "1.0.0")
            .unwrap());
        assert!(registry
            .check_version_compatibility("test_tool", "1.0")
            .unwrap());
        assert!(!registry
            .check_version_compatibility("test_tool", "2.0.0")
            .unwrap());
    }

    #[tokio::test]
    async fn test_get_dependency_graph() {
        let registry = ToolRegistry::new();

        let (tool1, metadata1) = create_test_tool("tool1", vec!["tool2".to_string()]);
        let (tool2, metadata2) = create_test_tool("tool2", vec![]);

        registry.register_tool(tool1, metadata1).unwrap();
        registry.register_tool(tool2, metadata2).unwrap();

        let graph = registry.get_dependency_graph().unwrap();
        assert_eq!(graph.get("tool1"), Some(&vec!["tool2".to_string()]));
        assert_eq!(graph.get("tool2"), Some(&vec![]));
    }
}
