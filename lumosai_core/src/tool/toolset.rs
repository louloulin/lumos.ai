//! Tool set management system
//! 
//! Provides collection and management of tools for agents and workflows

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Result, Error};
use crate::agent::types::RuntimeContext;
use super::enhanced::{EnhancedTool, ToolCategory, ToolCapability};

/// Tool definition for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Parameter schema (JSON Schema)
    pub parameters: Value,
}

/// Tool set error types
#[derive(Debug, thiserror::Error)]
pub enum ToolSetError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Tool execution error: {0}")]
    ExecutionError(String),
    
    #[error("Duplicate tool name: {0}")]
    DuplicateName(String),
    
    #[error("Invalid tool configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl From<ToolSetError> for Error {
    fn from(e: ToolSetError) -> Self {
        Error::Tool(e.to_string())
    }
}

/// Tool type enumeration for dynamic dispatch
pub enum ToolType {
    /// Simple tool
    Simple(Box<dyn crate::tool::tool::Tool>),
    /// Enhanced tool
    Enhanced(Box<dyn EnhancedTool>),
}

impl ToolType {
    /// Get tool name
    pub fn name(&self) -> String {
        match self {
            ToolType::Simple(tool) => {
                use crate::base::Base;
                tool.name().unwrap_or("unknown").to_string()
            }
            ToolType::Enhanced(_tool) => {
                // For enhanced tools, we would need to implement a name method
                "enhanced_tool".to_string()
            }
        }
    }
    
    /// Get tool definition (simplified for compatibility)
    pub async fn definition(&self, _prompt: Option<&str>) -> Result<ToolDefinition> {
        match self {
            ToolType::Simple(_tool) => {
                // For simple tools, create a basic definition
                Ok(ToolDefinition {
                    name: self.name(),
                    description: "Tool description".to_string(),
                    parameters: serde_json::json!({}),
                })
            }
            ToolType::Enhanced(_tool) => {
                // For enhanced tools, create a basic definition
                Ok(ToolDefinition {
                    name: self.name(),
                    description: "Enhanced tool description".to_string(),
                    parameters: serde_json::json!({}),
                })
            }
        }
    }

    /// Execute tool (simplified for compatibility)
    pub async fn execute(&self, args: Value, _context: &RuntimeContext) -> Result<Value> {
        match self {
            ToolType::Simple(tool) => {
                let context = crate::tool::context::ToolExecutionContext::default();
                let options = crate::tool::schema::ToolExecutionOptions::default();
                tool.execute(args, context, &options).await
            }
            ToolType::Enhanced(_tool) => {
                // For enhanced tools, return a placeholder result
                Ok(serde_json::json!({"result": "enhanced tool executed"}))
            }
        }
    }
    
    /// Get tool category
    pub fn category(&self) -> ToolCategory {
        match self {
            ToolType::Simple(_) => ToolCategory::General,
            ToolType::Enhanced(_tool) => ToolCategory::General,
        }
    }
    
    /// Get tool capabilities
    pub fn capabilities(&self) -> Vec<ToolCapability> {
        match self {
            ToolType::Simple(_) => vec![ToolCapability::Basic],
            ToolType::Enhanced(_tool) => vec![ToolCapability::Basic],
        }
    }
}

/// Tool set for managing collections of tools
pub struct ToolSet {
    /// Tools in the set
    tools: HashMap<String, ToolType>,
    /// Tool metadata
    metadata: ToolSetMetadata,
}

/// Tool set metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSetMetadata {
    /// Set name
    pub name: String,
    /// Set description
    pub description: Option<String>,
    /// Set version
    pub version: String,
    /// Set tags
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for ToolSetMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            name: "default".to_string(),
            description: None,
            version: "1.0.0".to_string(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl ToolSet {
    /// Create a new tool set
    pub fn new(name: String) -> Self {
        Self {
            tools: HashMap::new(),
            metadata: ToolSetMetadata {
                name,
                ..Default::default()
            },
        }
    }
    
    /// Create a tool set builder
    pub fn builder() -> ToolSetBuilder {
        ToolSetBuilder::new()
    }
    
    /// Add a simple tool to the set
    pub fn add_tool(&mut self, tool: Box<dyn crate::tool::tool::Tool>) -> Result<()> {
        use crate::base::Base;
        let name = tool.name().unwrap_or("unknown").to_string();
        if self.tools.contains_key(&name) {
            return Err(ToolSetError::DuplicateName(name).into());
        }

        self.tools.insert(name, ToolType::Simple(tool));
        self.metadata.updated_at = chrono::Utc::now();
        Ok(())
    }
    
    /// Add an enhanced tool to the set
    pub fn add_enhanced_tool(&mut self, tool: Box<dyn EnhancedTool>) -> Result<()> {
        let name = "enhanced_tool".to_string(); // Simplified for now
        if self.tools.contains_key(&name) {
            return Err(ToolSetError::DuplicateName(name).into());
        }

        self.tools.insert(name, ToolType::Enhanced(tool));
        self.metadata.updated_at = chrono::Utc::now();
        Ok(())
    }
    
    /// Remove a tool from the set
    pub fn remove_tool(&mut self, name: &str) -> Result<()> {
        if self.tools.remove(name).is_some() {
            self.metadata.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(ToolSetError::ToolNotFound(name.to_string()).into())
        }
    }
    
    /// Check if tool exists
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
    
    /// Get tool by name
    pub fn get(&self, name: &str) -> Option<&ToolType> {
        self.tools.get(name)
    }
    
    /// Get all tool names
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.values().map(|tool| tool.name()).collect()
    }
    
    /// Get tools by category
    pub fn tools_by_category(&self, category: &ToolCategory) -> Vec<&ToolType> {
        self.tools.values()
            .filter(|tool| &tool.category() == category)
            .collect()
    }
    
    /// Get tools by capability
    pub fn tools_by_capability(&self, capability: &ToolCapability) -> Vec<&ToolType> {
        self.tools.values()
            .filter(|tool| tool.capabilities().contains(capability))
            .collect()
    }
    
    /// Execute a tool by name
    pub async fn execute(&self, name: &str, args: Value, context: &RuntimeContext) -> Result<Value> {
        if let Some(tool) = self.tools.get(name) {
            tool.execute(args, context).await
        } else {
            Err(ToolSetError::ToolNotFound(name.to_string()).into())
        }
    }
    
    /// Get all tool definitions
    pub async fn definitions(&self, prompt: Option<&str>) -> Result<Vec<ToolDefinition>> {
        let mut definitions = Vec::new();
        
        for tool in self.tools.values() {
            definitions.push(tool.definition(prompt).await?);
        }
        
        Ok(definitions)
    }
    
    /// Get tool set metadata
    pub fn metadata(&self) -> &ToolSetMetadata {
        &self.metadata
    }
    
    /// Update tool set metadata
    pub fn update_metadata(&mut self, metadata: ToolSetMetadata) {
        self.metadata = metadata;
        self.metadata.updated_at = chrono::Utc::now();
    }
    
    /// Merge another tool set into this one
    pub fn merge(&mut self, other: ToolSet) -> Result<()> {
        for (name, tool) in other.tools {
            if self.tools.contains_key(&name) {
                return Err(ToolSetError::DuplicateName(name).into());
            }
            self.tools.insert(name, tool);
        }
        
        self.metadata.updated_at = chrono::Utc::now();
        Ok(())
    }
    
    /// Get tool set statistics
    pub fn stats(&self) -> ToolSetStats {
        let mut stats = ToolSetStats::default();
        
        for tool in self.tools.values() {
            stats.total_tools += 1;
            
            match tool.category() {
                ToolCategory::General => stats.categories.insert("general".to_string(), stats.categories.get("general").unwrap_or(&0) + 1),
                ToolCategory::Web => stats.categories.insert("web".to_string(), stats.categories.get("web").unwrap_or(&0) + 1),
                ToolCategory::FileSystem => stats.categories.insert("filesystem".to_string(), stats.categories.get("filesystem").unwrap_or(&0) + 1),
                ToolCategory::Database => stats.categories.insert("database".to_string(), stats.categories.get("database").unwrap_or(&0) + 1),
                ToolCategory::AI => stats.categories.insert("ai".to_string(), stats.categories.get("ai").unwrap_or(&0) + 1),
                ToolCategory::Communication => stats.categories.insert("communication".to_string(), stats.categories.get("communication").unwrap_or(&0) + 1),
                ToolCategory::DataProcessing => stats.categories.insert("dataprocessing".to_string(), stats.categories.get("dataprocessing").unwrap_or(&0) + 1),
                ToolCategory::System => stats.categories.insert("system".to_string(), stats.categories.get("system").unwrap_or(&0) + 1),
                ToolCategory::Math => stats.categories.insert("math".to_string(), stats.categories.get("math").unwrap_or(&0) + 1),
                ToolCategory::Custom(name) => stats.categories.insert(name.clone(), stats.categories.get(&name).unwrap_or(&0) + 1),
            };
            
            for capability in tool.capabilities() {
                match capability {
                    ToolCapability::Basic => stats.capabilities.insert("basic".to_string(), stats.capabilities.get("basic").unwrap_or(&0) + 1),
                    ToolCapability::Streaming => stats.capabilities.insert("streaming".to_string(), stats.capabilities.get("streaming").unwrap_or(&0) + 1),
                    ToolCapability::Async => stats.capabilities.insert("async".to_string(), stats.capabilities.get("async").unwrap_or(&0) + 1),
                    ToolCapability::Batch => stats.capabilities.insert("batch".to_string(), stats.capabilities.get("batch").unwrap_or(&0) + 1),
                    ToolCapability::Caching => stats.capabilities.insert("caching".to_string(), stats.capabilities.get("caching").unwrap_or(&0) + 1),
                    ToolCapability::RateLimit => stats.capabilities.insert("ratelimit".to_string(), stats.capabilities.get("ratelimit").unwrap_or(&0) + 1),
                    ToolCapability::Auth => stats.capabilities.insert("auth".to_string(), stats.capabilities.get("auth").unwrap_or(&0) + 1),
                    ToolCapability::Encryption => stats.capabilities.insert("encryption".to_string(), stats.capabilities.get("encryption").unwrap_or(&0) + 1),
                    ToolCapability::Monitoring => stats.capabilities.insert("monitoring".to_string(), stats.capabilities.get("monitoring").unwrap_or(&0) + 1),
                    ToolCapability::Custom(name) => stats.capabilities.insert(name.clone(), stats.capabilities.get(&name).unwrap_or(&0) + 1),
                };
            }
        }
        
        stats
    }
}

/// Tool set statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSetStats {
    /// Total number of tools
    pub total_tools: u32,
    /// Tools by category
    pub categories: HashMap<String, u32>,
    /// Tools by capability
    pub capabilities: HashMap<String, u32>,
}

impl Default for ToolSetStats {
    fn default() -> Self {
        Self {
            total_tools: 0,
            categories: HashMap::new(),
            capabilities: HashMap::new(),
        }
    }
}

/// Tool set builder for fluent construction
pub struct ToolSetBuilder {
    /// Tool set being built
    toolset: ToolSet,
}

impl ToolSetBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            toolset: ToolSet::new("builder".to_string()),
        }
    }
    
    /// Set tool set name
    pub fn name(mut self, name: String) -> Self {
        self.toolset.metadata.name = name;
        self
    }
    
    /// Set tool set description
    pub fn description(mut self, description: String) -> Self {
        self.toolset.metadata.description = Some(description);
        self
    }
    
    /// Set tool set version
    pub fn version(mut self, version: String) -> Self {
        self.toolset.metadata.version = version;
        self
    }
    
    /// Add a tag
    pub fn tag(mut self, tag: String) -> Self {
        self.toolset.metadata.tags.push(tag);
        self
    }
    
    /// Add a simple tool
    pub fn tool(mut self, tool: Box<dyn crate::tool::tool::Tool>) -> Result<Self> {
        self.toolset.add_tool(tool)?;
        Ok(self)
    }

    /// Add an enhanced tool
    pub fn enhanced_tool(mut self, tool: Box<dyn EnhancedTool>) -> Result<Self> {
        self.toolset.add_enhanced_tool(tool)?;
        Ok(self)
    }

    /// Add multiple tools
    pub fn tools(mut self, tools: Vec<Box<dyn crate::tool::tool::Tool>>) -> Result<Self> {
        for tool in tools {
            self.toolset.add_tool(tool)?;
        }
        Ok(self)
    }
    
    /// Build the tool set
    pub fn build(self) -> ToolSet {
        self.toolset
    }
}

impl Default for ToolSetBuilder {
    fn default() -> Self {
        Self::new()
    }
}
