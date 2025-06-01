//! Enhanced application class
//! 
//! Provides Mastra-like unified application management functionality

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::error::Result;
use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::agent::{Agent, AgentConfig};
use crate::tool::{Tool, ToolRegistry, ToolMetadata, ToolCategory};
use crate::memory::{Memory, MemoryConfig};
use crate::llm::{LlmProvider, Message};
use crate::workflow::basic::Workflow;
use crate::vector::VectorStorage;
use crate::rag::RagPipeline;
use crate::logger::{Component, Logger};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAppConfig {
    /// Application name
    pub name: String,
    /// Application description
    pub description: Option<String>,
    /// Application version
    pub version: Option<String>,
    /// Default LLM configuration
    pub default_llm: Option<String>,
    /// Memory configuration
    pub memory: Option<MemoryConfig>,
    /// Tools configuration
    pub tools: Option<ToolsConfig>,
    /// Agents configuration
    pub agents: Option<HashMap<String, AgentConfig>>,
    /// Workflows configuration
    pub workflows: Option<HashMap<String, Value>>,
    /// RAG configuration
    pub rag: Option<RagConfig>,
    /// Environment variables
    pub env: Option<HashMap<String, String>>,
}

/// Tools configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    /// Enable tool registry
    pub enable_registry: bool,
    /// Auto discover tools
    pub auto_discover: bool,
    /// Tool directories
    pub tool_directories: Vec<String>,
    /// Preloaded tools
    pub preload: Vec<String>,
}

/// RAG configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    /// Vector store configuration
    pub vector_store: String,
    /// Embedding model configuration
    pub embedding_model: String,
    /// Chunking configuration
    pub chunking: Option<ChunkingConfig>,
}

/// Chunking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Chunk size
    pub chunk_size: usize,
    /// Overlap size
    pub overlap: usize,
    /// Separators
    pub separators: Vec<String>,
}

/// Enhanced application class
pub struct EnhancedApp {
    /// Base component
    base: BaseComponent,
    /// Application configuration
    config: EnhancedAppConfig,
    /// LLM providers
    llm_providers: Arc<RwLock<HashMap<String, Arc<dyn LlmProvider>>>>,
    /// Agents
    agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
    /// Tool registry
    tool_registry: Arc<ToolRegistry>,
    /// Memory manager
    memory: Option<Arc<dyn Memory>>,
    /// Workflows
    workflows: Arc<RwLock<HashMap<String, Arc<dyn Workflow>>>>,
    /// RAG pipelines
    rag_pipelines: Arc<RwLock<HashMap<String, Arc<dyn RagPipeline>>>>,
    /// Vector storage
    vector_storage: Option<Arc<dyn VectorStorage>>,
    /// Runtime context
    context: Arc<RwLock<HashMap<String, Value>>>,
}

impl EnhancedApp {
    /// Create new enhanced application
    pub fn new(config: EnhancedAppConfig) -> Self {
        let component_config = ComponentConfig {
            name: Some(config.name.clone()),
            component: Component::Agent, // Use Agent component type
            log_level: None,
        };

        Self {
            base: BaseComponent::new(component_config),
            config,
            llm_providers: Arc::new(RwLock::new(HashMap::new())),
            agents: Arc::new(RwLock::new(HashMap::new())),
            tool_registry: Arc::new(ToolRegistry::new()),
            memory: None,
            workflows: Arc::new(RwLock::new(HashMap::new())),
            rag_pipelines: Arc::new(RwLock::new(HashMap::new())),
            vector_storage: None,
            context: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add LLM provider
    pub fn add_llm_provider(&self, name: String, provider: Arc<dyn LlmProvider>) -> Result<()> {
        let mut providers = self.llm_providers.write()
            .map_err(|_| crate::error::Error::Internal("Failed to acquire write lock".to_string()))?;
        providers.insert(name.clone(), provider);
        
        self.base.logger().info(&format!("LLM provider '{}' added", name), None);
        Ok(())
    }

    /// Get LLM provider
    pub fn get_llm_provider(&self, name: &str) -> Result<Option<Arc<dyn LlmProvider>>> {
        let providers = self.llm_providers.read()
            .map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        Ok(providers.get(name).cloned())
    }

    /// Add agent
    pub fn add_agent(&self, name: String, agent: Arc<dyn Agent>) -> Result<()> {
        let mut agents = self.agents.write()
            .map_err(|_| crate::error::Error::Internal("Failed to acquire write lock".to_string()))?;
        agents.insert(name.clone(), agent);
        
        self.base.logger().info(&format!("Agent '{}' added", name), None);
        Ok(())
    }

    /// Get agent
    pub fn get_agent(&self, name: &str) -> Result<Option<Arc<dyn Agent>>> {
        let agents = self.agents.read()
            .map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        Ok(agents.get(name).cloned())
    }

    /// List all agents
    pub fn list_agents(&self) -> Result<Vec<String>> {
        let agents = self.agents.read()
            .map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        Ok(agents.keys().cloned().collect())
    }

    /// Add tool
    pub fn add_tool(&self, tool: Arc<dyn Tool>, metadata: ToolMetadata) -> Result<()> {
        self.tool_registry.register_tool(tool, metadata)?;
        Ok(())
    }

    /// Get tool
    pub fn get_tool(&self, name: &str) -> Result<Option<Arc<dyn Tool>>> {
        self.tool_registry.get_tool(name)
    }

    /// Search tools
    pub fn search_tools(&self, query: &str) -> Result<Vec<String>> {
        self.tool_registry.search_tools(query)
    }

    /// Find tools by category
    pub fn find_tools_by_category(&self, category: &ToolCategory) -> Result<Vec<String>> {
        self.tool_registry.find_tools_by_category(category)
    }

    /// Set memory manager
    pub fn set_memory(&mut self, memory: Arc<dyn Memory>) {
        self.memory = Some(memory);
        self.base.logger().info("Memory manager set", None);
    }

    /// Get memory manager
    pub fn get_memory(&self) -> Option<Arc<dyn Memory>> {
        self.memory.clone()
    }

    /// Set vector storage
    pub fn set_vector_storage(&mut self, storage: Arc<dyn VectorStorage>) {
        self.vector_storage = Some(storage);
        self.base.logger().info("Vector storage set", None);
    }

    /// Get vector storage
    pub fn get_vector_storage(&self) -> Option<Arc<dyn VectorStorage>> {
        self.vector_storage.clone()
    }

    /// Set context variable
    pub fn set_context(&self, key: String, value: Value) -> Result<()> {
        let mut context = self.context.write()
            .map_err(|_| crate::error::Error::Internal("Failed to acquire write lock".to_string()))?;
        context.insert(key, value);
        Ok(())
    }

    /// Get context variable
    pub fn get_context(&self, key: &str) -> Result<Option<Value>> {
        let context = self.context.read()
            .map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        Ok(context.get(key).cloned())
    }

    /// Run agent
    pub async fn run_agent(&self, agent_name: &str, messages: &[Message]) -> Result<String> {
        let agent = self.get_agent(agent_name)?
            .ok_or_else(|| crate::error::Error::Internal(format!("Agent '{}' not found", agent_name)))?;

        let options = crate::agent::types::AgentGenerateOptions::default();
        let result = agent.generate(messages, &options).await?;
        
        Ok(result.response)
    }

    /// Get application statistics
    pub fn get_stats(&self) -> Result<AppStats> {
        let agents = self.agents.read()
            .map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        let workflows = self.workflows.read()
            .map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        let rag_pipelines = self.rag_pipelines.read()
            .map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;
        let llm_providers = self.llm_providers.read()
            .map_err(|_| crate::error::Error::Internal("Failed to acquire read lock".to_string()))?;

        let tool_stats = self.tool_registry.get_stats()?;

        Ok(AppStats {
            agents_count: agents.len(),
            workflows_count: workflows.len(),
            rag_pipelines_count: rag_pipelines.len(),
            llm_providers_count: llm_providers.len(),
            tools_count: tool_stats.total_tools,
            tool_categories_count: tool_stats.total_categories,
        })
    }

    /// Start application
    pub async fn start(&self) -> Result<()> {
        self.base.logger().info(&format!("Starting application '{}'", self.config.name), None);
        
        // Here you can add startup logic, such as initializing connections, warming up models, etc.
        
        self.base.logger().info("Application started successfully", None);
        Ok(())
    }

    /// Stop application
    pub async fn stop(&self) -> Result<()> {
        self.base.logger().info("Stopping application", None);
        
        // Here you can add cleanup logic
        
        self.base.logger().info("Application stopped", None);
        Ok(())
    }
}

/// Application statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStats {
    /// Number of agents
    pub agents_count: usize,
    /// Number of workflows
    pub workflows_count: usize,
    /// Number of RAG pipelines
    pub rag_pipelines_count: usize,
    /// Number of LLM providers
    pub llm_providers_count: usize,
    /// Number of tools
    pub tools_count: usize,
    /// Number of tool categories
    pub tool_categories_count: usize,
}

impl Default for EnhancedAppConfig {
    fn default() -> Self {
        Self {
            name: "LumosApp".to_string(),
            description: Some("Enhanced Lumosai Application".to_string()),
            version: Some("1.0.0".to_string()),
            default_llm: None,
            memory: None,
            tools: None,
            agents: None,
            workflows: None,
            rag: None,
            env: None,
        }
    }
}
