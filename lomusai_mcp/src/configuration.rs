use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

use lomusai_core::tool::Tool;

use crate::client::MCPClient;
use crate::error::{MCPError, Result};
use crate::types::{ServerParameters, StdioServerParameters, SSEServerParameters};

/// Server definitions for different transport types
#[derive(Debug, Clone)]
pub enum ServerDefinition {
    /// Standard I/O-based server definition
    Stdio {
        command: String,
        args: Vec<String>,
        env: Option<HashMap<String, String>>,
    },
    
    /// Server-Sent Events-based server definition
    SSE {
        url: String,
        request_init: Option<HashMap<String, String>>,
    },
}

/// Configuration for MCP servers and tools
#[derive(Debug, Clone)]
pub struct MCPConfiguration {
    /// Unique identifier for this configuration
    pub id: String,
    /// The set of defined servers
    servers: HashMap<String, ServerDefinition>,
    /// The default server to use if none is specified
    default_server: Option<String>,
    /// Clients created for each server
    clients: Arc<Mutex<HashMap<String, Arc<MCPClient>>>>,
}

impl MCPConfiguration {
    /// Create a new MCP configuration with multiple servers
    pub fn new(
        servers: HashMap<String, ServerDefinition>,
        id: Option<String>,
    ) -> Self {
        Self {
            id: id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            servers,
            default_server: None,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Get a client for a specific server
    async fn get_client(&self, server_name: &str) -> Result<Arc<MCPClient>> {
        let mut clients = self.clients.lock().await;
        
        // Return cached client if available
        if let Some(client) = clients.get(server_name) {
            return Ok(client.clone());
        }
        
        // Get server configuration
        let server_def = self.servers.get(server_name)
            .ok_or_else(|| MCPError::ResourceNotFoundError(
                format!("Server '{}' not found in configuration", server_name)
            ))?;
            
        // Create a client based on server definition
        let client = match server_def {
            ServerDefinition::Stdio { command, args, env } => {
                let params = StdioServerParameters {
                    command: command.clone(),
                    args: args.clone(),
                    env: env.clone().unwrap_or_default(),
                };
                
                Arc::new(MCPClient::new(
                    server_name,
                    ServerParameters::Stdio(params),
                    None,
                    None,
                    None,
                ))
            },
            ServerDefinition::SSE { url, request_init } => {
                let url_parsed = Url::parse(url)
                    .map_err(|e| MCPError::ConfigurationError(
                        format!("Invalid URL for server '{}': {}", server_name, e)
                    ))?;
                    
                let params = SSEServerParameters {
                    url: url_parsed,
                    request_init: request_init.clone(),
                };
                
                Arc::new(MCPClient::new(
                    server_name,
                    ServerParameters::SSE(params),
                    None,
                    None,
                    None,
                ))
            },
        };
        
        // Connect to the server
        client.connect().await?;
        
        // Cache the client
        clients.insert(server_name.to_string(), client.clone());
        
        Ok(client)
    }
    
    /// Get all tools from all servers as a flat map
    pub async fn get_tools(&self) -> Result<HashMap<String, Box<dyn Tool>>> {
        let mut all_tools = HashMap::new();
        
        for server_name in self.servers.keys() {
            let client = self.get_client(server_name).await?;
            let server_tools = client.tools().await?;
            
            // Add all tools with namespaced names
            for (tool_name, tool) in server_tools {
                all_tools.insert(tool_name, tool);
            }
        }
        
        Ok(all_tools)
    }
    
    /// Get tools organized by server
    pub async fn get_toolsets(&self) -> Result<HashMap<String, HashMap<String, Box<dyn Tool>>>> {
        let mut toolsets = HashMap::new();
        
        for server_name in self.servers.keys() {
            let client = self.get_client(server_name).await?;
            let server_tools = client.tools().await?;
            
            toolsets.insert(server_name.clone(), server_tools);
        }
        
        Ok(toolsets)
    }
    
    /// Disconnect from all servers
    pub async fn disconnect(&self) -> Result<()> {
        let mut clients = self.clients.lock().await;
        
        for client in clients.values() {
            client.disconnect().await?;
        }
        
        clients.clear();
        
        Ok(())
    }
} 