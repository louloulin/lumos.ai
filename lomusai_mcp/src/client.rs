use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use futures::Stream;
use tokio::sync::{mpsc, Mutex};
use tokio::time::timeout;
use tokio_stream::wrappers::ReceiverStream;
use url::Url;
use async_trait::async_trait;
use std::fmt;

use lomusai_core::tool::{Tool, ToolExecutionOptions, ToolSchema};
use lomusai_core::{Error as CoreError, Result as CoreResult};
use serde_json::Value;

use crate::error::{MCPError, Result};
use crate::types::{
    ClientCapabilities, ExecuteToolRequest, ListResourcesResult, MCPMessage,
    ServerParameters, StdioServerParameters, SSEServerParameters, ToolDefinition,
};
use crate::transport::{Transport, create_transport};

/// MCP client for interacting with an MCP server
pub struct MCPClient {
    name: String,
    version: String,
    capabilities: ClientCapabilities,
    #[allow(dead_code)]
    pub transport: Arc<Mutex<Box<dyn Transport>>>,
    timeout_ms: u64,
    resources: Arc<Mutex<Option<ListResourcesResult>>>,
    connected: Arc<Mutex<bool>>,
}

// Custom Debug implementation to handle non-Debug Transport
impl fmt::Debug for MCPClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MCPClient")
            .field("name", &self.name)
            .field("version", &self.version)
            .field("capabilities", &self.capabilities)
            .field("timeout_ms", &self.timeout_ms)
            .field("connected", &self.connected)
            .finish_non_exhaustive()
    }
}

impl MCPClient {
    /// Create a new MCP client
    pub fn new(
        name: &str,
        server: ServerParameters,
        capabilities: Option<ClientCapabilities>,
        version: Option<&str>,
        timeout_ms: Option<u64>,
    ) -> Self {
        Self {
            name: name.to_string(),
            version: version.unwrap_or("1.0.0").to_string(),
            capabilities: capabilities.unwrap_or_default(),
            transport: Arc::new(Mutex::new(create_transport(server))),
            timeout_ms: timeout_ms.unwrap_or(60000),
            resources: Arc::new(Mutex::new(None)),
            connected: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Create a new MCP client with a stdio server
    pub fn with_stdio(
        name: &str,
        command: &str,
        args: Vec<&str>,
        env: Option<HashMap<String, String>>,
    ) -> Self {
        let server_params = StdioServerParameters {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            env: env.unwrap_or_default(),
        };
        
        Self::new(
            name,
            ServerParameters::Stdio(server_params),
            None,
            None,
            None,
        )
    }
    
    /// Create a new MCP client with an SSE server
    pub fn with_sse(
        name: &str,
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Self> {
        let url = Url::parse(url)
            .map_err(|e| MCPError::ConfigurationError(format!("Invalid URL: {}", e)))?;
            
        let server_params = SSEServerParameters {
            url,
            request_init: headers,
        };
        
        Ok(Self::new(
            name,
            ServerParameters::SSE(server_params),
            None,
            None,
            None,
        ))
    }
    
    /// Connect to the MCP server
    pub async fn connect(&self) -> Result<()> {
        let mut connected = self.connected.lock().await;
        if *connected {
            return Ok(());
        }
        
        let mut transport = self.transport.lock().await;
        transport.connect().await?;
        
        // Send initialization message
        let init_message = MCPMessage::Initialize {
            name: self.name.clone(),
            version: self.version.clone(),
            capabilities: self.capabilities.clone(),
        };
        
        transport.send_message(&init_message).await?;
        
        // Wait for initialization response
        let response = match timeout(
            Duration::from_millis(self.timeout_ms),
            transport.receive_message()
        ).await {
            Ok(result) => result?,
            Err(_) => return Err(MCPError::TimeoutError(self.timeout_ms)),
        };
        
        match response {
            MCPMessage::InitializeResult { status, error } => {
                if status != "success" {
                    return Err(MCPError::ServerError(
                        error.unwrap_or_else(|| "Unknown server error".to_string())
                    ));
                }
            },
            _ => return Err(MCPError::ProtocolError(
                format!("Expected InitializeResult, got {:?}", response)
            )),
        }
        
        *connected = true;
        Ok(())
    }
    
    /// Disconnect from the MCP server
    pub async fn disconnect(&self) -> Result<()> {
        let mut connected = self.connected.lock().await;
        if !*connected {
            return Ok(());
        }
        
        let mut transport = self.transport.lock().await;
        transport.disconnect().await?;
        
        *connected = false;
        Ok(())
    }
    
    /// Retrieve available resources from the server
    pub async fn resources(&self) -> Result<ListResourcesResult> {
        // Check if resources are already cached
        {
            let resources = self.resources.lock().await;
            if let Some(ref cached) = *resources {
                return Ok(cached.clone());
            }
        }
        
        // Ensure we're connected
        self.connect().await?;
        
        // Send list resources message
        let mut transport = self.transport.lock().await;
        transport.send_message(&MCPMessage::ListResources {}).await?;
        
        // Wait for response
        let response = match timeout(
            Duration::from_millis(self.timeout_ms),
            transport.receive_message()
        ).await {
            Ok(result) => result?,
            Err(_) => return Err(MCPError::TimeoutError(self.timeout_ms)),
        };
        
        match response {
            MCPMessage::ListResourcesResult { resources } => {
                let result = ListResourcesResult { resources };
                
                // Cache the result
                let mut cache = self.resources.lock().await;
                *cache = Some(result.clone());
                
                Ok(result)
            },
            MCPMessage::Error { error } => {
                Err(MCPError::ServerError(error))
            },
            _ => Err(MCPError::ProtocolError(
                format!("Expected ListResourcesResult, got {:?}", response)
            )),
        }
    }
    
    /// Execute a tool on the server
    pub async fn execute_tool(
        &self,
        resource_name: &str,
        tool_name: &str,
        parameters: HashMap<String, serde_json::Value>,
        stream: bool,
    ) -> Result<String> {
        // Ensure we're connected
        self.connect().await?;
        
        // Prepare the execute request
        let request = ExecuteToolRequest {
            resource: resource_name.to_string(),
            tool: tool_name.to_string(),
            parameters,
            stream: Some(stream),
        };
        
        let message = MCPMessage::ExecuteTool(request);
        
        // Send the message
        let mut transport = self.transport.lock().await;
        transport.send_message(&message).await?;
        
        // Wait for a response
        let response = match timeout(
            Duration::from_millis(self.timeout_ms),
            transport.receive_message()
        ).await {
            Ok(result) => result?,
            Err(_) => return Err(MCPError::TimeoutError(self.timeout_ms)),
        };
        
        match response {
            MCPMessage::ExecuteToolResult { result } => {
                Ok(result)
            },
            MCPMessage::ExecuteToolError { error } => {
                Err(MCPError::ToolExecutionError(error))
            },
            MCPMessage::Error { error } => {
                Err(MCPError::ServerError(error))
            },
            _ => Err(MCPError::ProtocolError(
                format!("Expected ExecuteToolResult, got {:?}", response)
            )),
        }
    }
    
    /// Execute a tool and receive streaming results
    pub async fn execute_tool_stream(
        &self,
        resource_name: &str,
        tool_name: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send + Sync>>> {
        // Ensure we're connected
        self.connect().await?;
        
        // Prepare the execute request
        let request = ExecuteToolRequest {
            resource: resource_name.to_string(),
            tool: tool_name.to_string(),
            parameters,
            stream: Some(true),
        };
        
        let message = MCPMessage::ExecuteTool(request);
        
        // Send the message
        let mut transport = self.transport.lock().await;
        transport.send_message(&message).await?;
        
        // Get the message stream - note: message_stream is not async
        let message_rx = transport.message_stream()?;
        
        // Release the transport lock
        drop(transport);
        
        // Create a new channel for the result stream
        let (tx, rx) = mpsc::channel(100);
        
        // Spawn a task to transform messages and forward them to the result stream
        tokio::spawn(async move {
            let mut message_rx = message_rx;
            
            while let Some(message_result) = message_rx.recv().await {
                match message_result {
                    Ok(MCPMessage::ExecuteToolStreamResult { data }) => {
                        if tx.send(Ok(data)).await.is_err() {
                            break;
                        }
                    },
                    Ok(MCPMessage::ExecuteToolStreamEnd { error }) => {
                        if let Some(err) = error {
                            let _ = tx.send(Err(MCPError::ToolExecutionError(err))).await;
                        }
                        break;
                    },
                    Ok(MCPMessage::ExecuteToolError { error }) => {
                        let _ = tx.send(Err(MCPError::ToolExecutionError(error))).await;
                        break;
                    },
                    Ok(MCPMessage::Error { error }) => {
                        let _ = tx.send(Err(MCPError::ServerError(error))).await;
                        break;
                    },
                    Ok(_) => continue,
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    },
                }
            }
        });
        
        // Convert the channel receiver to a stream
        let stream = ReceiverStream::new(rx);
        Ok(Box::pin(stream))
    }
    
    /// Convert MCP tool definitions to Lomusai tools
    pub async fn tools(&self) -> Result<HashMap<String, Box<dyn Tool>>> {
        // Get resources first
        let resources = self.resources().await?;
        let mut tools = HashMap::new();
        
        for resource in resources.resources {
            let resource_name = resource.metadata.name.clone();
            for tool_def in resource.tools {
                let tool_name = format!("{}_{}", resource_name, tool_def.name);
                let wrapper = MCPToolWrapper::new(
                    tool_def.name,
                    tool_def.description,
                    Arc::new(self.clone()),
                    resource_name.clone(),
                );
                tools.insert(tool_name, Box::new(wrapper) as Box<dyn Tool>);
            }
        }
        
        Ok(tools)
    }
}

/// A wrapper tool implementation for MCP tools
#[derive(Debug)]
struct MCPToolWrapper {
    name: String,
    description: String,
    client: Arc<MCPClient>,
    resource_name: String,
}

impl MCPToolWrapper {
    fn new(
        name: String, 
        description: String,
        client: Arc<MCPClient>,
        resource_name: String,
    ) -> Self {
        Self {
            name,
            description,
            client,
            resource_name,
        }
    }
}

#[async_trait]
impl Tool for MCPToolWrapper {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    // Create an empty schema since we cannot rely on default
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            parameters: Vec::new(),
        }
    }
    
    async fn execute(&self, params: HashMap<String, Value>, _options: &ToolExecutionOptions) -> CoreResult<Value> {
        match self.client.execute_tool(
            &self.resource_name,
            &self.name,
            params,
            false,
        ).await {
            Ok(output) => {
                // Try to parse the output as JSON
                match serde_json::from_str(&output) {
                    Ok(value) => Ok(value),
                    Err(_) => {
                        // If parsing fails, return the raw string as a JSON string
                        Ok(Value::String(output))
                    }
                }
            },
            Err(e) => Err(CoreError::Tool(format!("Tool execution error: {:?}", e))),
        }
    }
    
    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(Self {
            name: self.name.clone(),
            description: self.description.clone(),
            client: self.client.clone(),
            resource_name: self.resource_name.clone(),
        })
    }
}

impl Clone for MCPClient {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            version: self.version.clone(),
            capabilities: self.capabilities.clone(),
            transport: self.transport.clone(),
            timeout_ms: self.timeout_ms,
            resources: self.resources.clone(),
            connected: self.connected.clone(),
        }
    }
} 