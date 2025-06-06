//! MCP Server Discovery and Registry
//! 
//! This module provides automatic discovery and registration of MCP servers,
//! including support for common MCP server patterns and configurations.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{MCPConfiguration, ServerDefinition, EnhancedMCPManager, Result, MCPError};

/// MCP server registry for discovering and managing available servers
pub struct MCPServerRegistry {
    /// Known server configurations
    servers: HashMap<String, ServerConfig>,
    /// MCP manager instance
    manager: Arc<EnhancedMCPManager>,
}

/// Configuration for an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server name
    pub name: String,
    /// Server description
    pub description: String,
    /// Server type (stdio, sse, http)
    pub server_type: ServerType,
    /// Connection parameters
    pub connection: ConnectionConfig,
    /// Capabilities this server provides
    pub capabilities: Vec<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Whether this server is enabled by default
    pub enabled: bool,
    /// Priority for load balancing (higher = preferred)
    pub priority: u32,
}

/// Type of MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerType {
    Stdio,
    SSE,
    HTTP,
}

/// Connection configuration for different server types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ConnectionConfig {
    Stdio {
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        working_dir: Option<String>,
    },
    SSE {
        url: String,
        headers: HashMap<String, String>,
        timeout_ms: Option<u64>,
    },
    HTTP {
        base_url: String,
        headers: HashMap<String, String>,
        timeout_ms: Option<u64>,
    },
}

impl MCPServerRegistry {
    /// Create a new server registry
    pub fn new(manager: Arc<EnhancedMCPManager>) -> Self {
        Self {
            servers: HashMap::new(),
            manager,
        }
    }

    /// Register a server configuration
    pub fn register_server(&mut self, config: ServerConfig) -> Result<()> {
        if self.servers.contains_key(&config.name) {
            return Err(MCPError::DuplicateConfiguration(config.name.clone()));
        }

        self.servers.insert(config.name.clone(), config);
        Ok(())
    }

    /// Load server configurations from a directory
    pub async fn load_from_directory(&mut self, dir_path: PathBuf) -> Result<usize> {
        let mut loaded_count = 0;

        if !dir_path.exists() {
            return Ok(0);
        }

        let mut entries = fs::read_dir(&dir_path).await
            .map_err(|e| MCPError::IOError(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| MCPError::IOError(format!("Failed to read directory entry: {}", e)))? {
            
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_server_config(&path).await {
                    Ok(config) => {
                        self.register_server(config)?;
                        loaded_count += 1;
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to load server config from {:?}: {}", path, e);
                    }
                }
            }
        }

        println!("📁 Loaded {} MCP server configurations from {:?}", loaded_count, dir_path);
        Ok(loaded_count)
    }

    /// Load a single server configuration file
    async fn load_server_config(&self, path: &PathBuf) -> Result<ServerConfig> {
        let content = fs::read_to_string(path).await
            .map_err(|e| MCPError::IOError(format!("Failed to read config file: {}", e)))?;

        let config: ServerConfig = serde_json::from_str(&content)
            .map_err(|e| MCPError::DeserializationError(format!("Invalid config format: {}", e)))?;

        Ok(config)
    }

    /// Auto-discover common MCP servers
    pub async fn auto_discover(&mut self) -> Result<usize> {
        let mut discovered_count = 0;

        // Discover common MCP servers
        discovered_count += self.discover_mastra_servers().await?;
        discovered_count += self.discover_local_servers().await?;
        discovered_count += self.discover_npm_servers().await?;

        println!("🔍 Auto-discovered {} MCP servers", discovered_count);
        Ok(discovered_count)
    }

    /// Discover Mastra-compatible servers
    async fn discover_mastra_servers(&mut self) -> Result<usize> {
        let mut count = 0;

        // Common Mastra MCP servers
        let mastra_servers = vec![
            ServerConfig {
                name: "mastra-web-search".to_string(),
                description: "Web search capabilities via Mastra".to_string(),
                server_type: ServerType::Stdio,
                connection: ConnectionConfig::Stdio {
                    command: "npx".to_string(),
                    args: vec!["@mastra/web-search-mcp".to_string()],
                    env: HashMap::new(),
                    working_dir: None,
                },
                capabilities: vec!["web_search".to_string(), "search_results".to_string()],
                tags: vec!["mastra".to_string(), "web".to_string(), "search".to_string()],
                enabled: true,
                priority: 80,
            },
            ServerConfig {
                name: "mastra-file-system".to_string(),
                description: "File system operations via Mastra".to_string(),
                server_type: ServerType::Stdio,
                connection: ConnectionConfig::Stdio {
                    command: "npx".to_string(),
                    args: vec!["@mastra/filesystem-mcp".to_string()],
                    env: HashMap::new(),
                    working_dir: None,
                },
                capabilities: vec!["file_read".to_string(), "file_write".to_string(), "directory_list".to_string()],
                tags: vec!["mastra".to_string(), "filesystem".to_string()],
                enabled: true,
                priority: 75,
            },
        ];

        for server in mastra_servers {
            if !self.servers.contains_key(&server.name) {
                self.register_server(server)?;
                count += 1;
            }
        }

        Ok(count)
    }

    /// Discover local MCP servers
    async fn discover_local_servers(&mut self) -> Result<usize> {
        let mut count = 0;

        // Check for common local MCP server patterns
        let local_patterns = vec![
            ("local-docs", "Local documentation server", "mcp-docs-server"),
            ("local-git", "Local Git operations", "mcp-git-server"),
            ("local-database", "Local database access", "mcp-db-server"),
        ];

        for (name, description, command) in local_patterns {
            // Check if command exists (simplified check)
            if self.command_exists(command).await {
                let server = ServerConfig {
                    name: name.to_string(),
                    description: description.to_string(),
                    server_type: ServerType::Stdio,
                    connection: ConnectionConfig::Stdio {
                        command: command.to_string(),
                        args: vec![],
                        env: HashMap::new(),
                        working_dir: None,
                    },
                    capabilities: vec!["local".to_string()],
                    tags: vec!["local".to_string()],
                    enabled: false, // Disabled by default for security
                    priority: 50,
                };

                if !self.servers.contains_key(&server.name) {
                    self.register_server(server)?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Discover NPM-based MCP servers
    async fn discover_npm_servers(&mut self) -> Result<usize> {
        let mut count = 0;

        // Common NPM MCP packages
        let npm_servers = vec![
            ("calculator-mcp", "Basic calculator operations", "@modelcontextprotocol/calculator"),
            ("weather-mcp", "Weather information", "@modelcontextprotocol/weather"),
            ("time-mcp", "Time and date utilities", "@modelcontextprotocol/time"),
        ];

        for (name, description, package) in npm_servers {
            let server = ServerConfig {
                name: name.to_string(),
                description: description.to_string(),
                server_type: ServerType::Stdio,
                connection: ConnectionConfig::Stdio {
                    command: "npx".to_string(),
                    args: vec![package.to_string()],
                    env: HashMap::new(),
                    working_dir: None,
                },
                capabilities: vec!["utility".to_string()],
                tags: vec!["npm".to_string(), "utility".to_string()],
                enabled: false, // Disabled by default, user can enable
                priority: 60,
            };

            if !self.servers.contains_key(&server.name) {
                self.register_server(server)?;
                count += 1;
            }
        }

        Ok(count)
    }

    /// Check if a command exists (simplified implementation)
    async fn command_exists(&self, _command: &str) -> bool {
        // In a real implementation, this would check if the command is available
        // For now, we'll return false to avoid false positives
        false
    }

    /// Connect to all enabled servers
    pub async fn connect_enabled_servers(&self) -> Result<usize> {
        let mut connected_count = 0;

        for (name, config) in &self.servers {
            if config.enabled {
                match self.connect_server(name, config).await {
                    Ok(()) => {
                        connected_count += 1;
                        println!("✅ Connected to MCP server '{}'", name);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to connect to MCP server '{}': {}", name, e);
                    }
                }
            }
        }

        println!("🔗 Connected to {} MCP servers", connected_count);
        Ok(connected_count)
    }

    /// Connect to a specific server
    async fn connect_server(&self, name: &str, config: &ServerConfig) -> Result<()> {
        let server_def = self.convert_to_server_definition(config)?;
        let mcp_config = MCPConfiguration::new(
            [(name.to_string(), server_def)].into_iter().collect(),
            Some(format!("mcp-{}", name)),
        );

        self.manager.register_mcp_server(name.to_string(), mcp_config).await
    }

    /// Convert ServerConfig to ServerDefinition
    fn convert_to_server_definition(&self, config: &ServerConfig) -> Result<ServerDefinition> {
        match &config.connection {
            ConnectionConfig::Stdio { command, args, env, .. } => {
                Ok(ServerDefinition::Stdio {
                    command: command.clone(),
                    args: args.clone(),
                    env: Some(env.clone()),
                })
            }
            ConnectionConfig::SSE { url, .. } => {
                Ok(ServerDefinition::SSE {
                    url: url.clone(),
                    request_init: None,
                })
            }
            ConnectionConfig::HTTP { base_url, .. } => {
                // For now, treat HTTP as SSE
                Ok(ServerDefinition::SSE {
                    url: base_url.clone(),
                    request_init: None,
                })
            }
        }
    }

    /// Get all registered servers
    pub fn get_servers(&self) -> &HashMap<String, ServerConfig> {
        &self.servers
    }

    /// Get servers by capability
    pub fn get_servers_by_capability(&self, capability: &str) -> Vec<&ServerConfig> {
        self.servers.values()
            .filter(|config| config.capabilities.contains(&capability.to_string()))
            .collect()
    }

    /// Get servers by tag
    pub fn get_servers_by_tag(&self, tag: &str) -> Vec<&ServerConfig> {
        self.servers.values()
            .filter(|config| config.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Enable a server
    pub fn enable_server(&mut self, name: &str) -> Result<()> {
        if let Some(config) = self.servers.get_mut(name) {
            config.enabled = true;
            Ok(())
        } else {
            Err(MCPError::ClientNotFound(name.to_string()))
        }
    }

    /// Disable a server
    pub fn disable_server(&mut self, name: &str) -> Result<()> {
        if let Some(config) = self.servers.get_mut(name) {
            config.enabled = false;
            Ok(())
        } else {
            Err(MCPError::ClientNotFound(name.to_string()))
        }
    }

    /// Save registry to file
    pub async fn save_to_file(&self, path: PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.servers)
            .map_err(|e| MCPError::DeserializationError(format!("Failed to serialize: {}", e)))?;

        fs::write(&path, json).await
            .map_err(|e| MCPError::IOError(format!("Failed to write file: {}", e)))?;

        println!("💾 Saved MCP server registry to {:?}", path);
        Ok(())
    }
}
