//! Tool registration and discovery mechanism
//! 
//! Provides dynamic tool registration, discovery and management functionality, similar to Mastra's tool system

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::error::Result;
use crate::tool::{Tool, ToolSchema};
use crate::base::{BaseComponent, ComponentConfig, Base};
use crate::logger::{Component, Logger};

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
            let tools = self.tools.read().map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
            if tools.contains_key(&tool_name) {
                return Err(crate::error::Error::Internal(format!("Tool '{}' is already registered", tool_name)));
            }
        }

        // Register tool
        {
            let mut tools = self.tools.write().map_err(|_| crate::error::Error::Internal("Failed to acquire write lock".to_string()))?;
            tools.insert(tool_name.clone(), tool);
        }

        // Store metadata
        {
            let mut metadata_map = self.metadata.write().map_err(|_| crate::error::Error::Internal("Failed to acquire write lock".to_string()))?;
            metadata_map.insert(tool_name.clone(), metadata.clone());
        }

        // Update category index
        {
            let mut category_index = self.category_index.write().map_err(|_| crate::error::Error::Internal("Failed to acquire write lock".to_string()))?;
            category_index.entry(metadata.category.clone()).or_insert_with(Vec::new).push(tool_name.clone());
        }

        // Update tag index
        {
            let mut tag_index = self.tag_index.write().map_err(|_| crate::error::Error::Internal("Failed to acquire write lock".to_string()))?;
            for tag in &metadata.tags {
                tag_index.entry(tag.clone()).or_insert_with(Vec::new).push(tool_name.clone());
            }
        }

        self.base.logger().info(&format!("Tool '{}' registered successfully", tool_name), None);
        Ok(())
    }

    /// Unregister tool
    pub fn unregister_tool(&self, tool_name: &str) -> Result<()> {
        // Get metadata
        let metadata = {
            let metadata_map = self.metadata.read().map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
            metadata_map.get(tool_name).cloned()
        };

        let metadata = metadata.ok_or_else(|| crate::error::Error::Internal(format!("Tool '{}' not found", tool_name)))?;

        // Remove tool
        {
            let mut tools = self.tools.write().map_err(|_| crate::error::Error::Internal("Failed to acquire write lock".to_string()))?;
            tools.remove(tool_name);
        }

        // Remove metadata
        {
            let mut metadata_map = self.metadata.write().map_err(|_| crate::error::Error::Internal("Failed to acquire write lock".to_string()))?;
            metadata_map.remove(tool_name);
        }

        // Update category index
        {
            let mut category_index = self.category_index.write().map_err(|_| crate::error::Error::Internal("Failed to acquire write lock".to_string()))?;
            if let Some(tools_in_category) = category_index.get_mut(&metadata.category) {
                tools_in_category.retain(|name| name != tool_name);
                if tools_in_category.is_empty() {
                    category_index.remove(&metadata.category);
                }
            }
        }

        // Update tag index
        {
            let mut tag_index = self.tag_index.write().map_err(|_| crate::error::Error::Internal("Failed to acquire write lock".to_string()))?;
            for tag in &metadata.tags {
                if let Some(tools_with_tag) = tag_index.get_mut(tag) {
                    tools_with_tag.retain(|name| name != tool_name);
                    if tools_with_tag.is_empty() {
                        tag_index.remove(tag);
                    }
                }
            }
        }

        self.base.logger().info(&format!("Tool '{}' unregistered successfully", tool_name), None);
        Ok(())
    }

    /// Get tool
    pub fn get_tool(&self, tool_name: &str) -> Result<Option<Arc<dyn Tool>>> {
        let tools = self.tools.read().map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        Ok(tools.get(tool_name).cloned())
    }

    /// Get all tool names
    pub fn list_tools(&self) -> Result<Vec<String>> {
        let tools = self.tools.read().map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        Ok(tools.keys().cloned().collect())
    }

    /// Find tools by category
    pub fn find_tools_by_category(&self, category: &ToolCategory) -> Result<Vec<String>> {
        let category_index = self.category_index.read().map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        Ok(category_index.get(category).cloned().unwrap_or_default())
    }

    /// Find tools by tag
    pub fn find_tools_by_tag(&self, tag: &str) -> Result<Vec<String>> {
        let tag_index = self.tag_index.read().map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        Ok(tag_index.get(tag).cloned().unwrap_or_default())
    }

    /// Search tools
    pub fn search_tools(&self, query: &str) -> Result<Vec<String>> {
        let metadata_map = self.metadata.read().map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        let query_lower = query.to_lowercase();
        
        let mut results = Vec::new();
        for (tool_name, metadata) in metadata_map.iter() {
            if metadata.name.to_lowercase().contains(&query_lower) ||
               metadata.description.to_lowercase().contains(&query_lower) ||
               metadata.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) {
                results.push(tool_name.clone());
            }
        }
        
        Ok(results)
    }

    /// Get tool metadata
    pub fn get_metadata(&self, tool_name: &str) -> Result<Option<ToolMetadata>> {
        let metadata_map = self.metadata.read().map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        Ok(metadata_map.get(tool_name).cloned())
    }

    /// Get all categories
    pub fn list_categories(&self) -> Result<Vec<ToolCategory>> {
        let category_index = self.category_index.read().map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        Ok(category_index.keys().cloned().collect())
    }

    /// Get all tags
    pub fn list_tags(&self) -> Result<Vec<String>> {
        let tag_index = self.tag_index.read().map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        Ok(tag_index.keys().cloned().collect())
    }

    /// Get tool statistics
    pub fn get_stats(&self) -> Result<ToolRegistryStats> {
        let tools = self.tools.read().map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        let category_index = self.category_index.read().map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        let tag_index = self.tag_index.read().map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;

        Ok(ToolRegistryStats {
            total_tools: tools.len(),
            total_categories: category_index.len(),
            total_tags: tag_index.len(),
        })
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
