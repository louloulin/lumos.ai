use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// Capabilities that a client can advertise to a server
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,
    
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// Definition of an MCP server using stdio transport
#[derive(Debug, Clone)]
pub struct StdioServerParameters {
    /// Command to execute
    pub command: String,
    
    /// Command arguments
    pub args: Vec<String>,
    
    /// Environment variables
    pub env: HashMap<String, String>,
}

/// Definition of an MCP server using SSE transport
#[derive(Debug, Clone)]
pub struct SSEServerParameters {
    /// URL of the SSE endpoint
    pub url: Url,
    
    /// Request initialization parameters
    pub request_init: Option<HashMap<String, String>>,
}

/// Combined server parameters
#[derive(Debug, Clone)]
pub enum ServerParameters {
    Stdio(StdioServerParameters),
    SSE(SSEServerParameters),
}

/// Resource metadata returned by the server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetadata {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub properties: HashMap<String, serde_json::Value>,
}

/// Tool parameter schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterSchema {
    pub name: String,
    pub description: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}

/// Tool definition returned by the server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_schema: Option<serde_json::Value>,
}

/// Resource with its tools
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub metadata: ResourceMetadata,
    pub tools: Vec<ToolDefinition>,
}

/// Result of listing resources
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesResult {
    pub resources: Vec<Resource>,
}

/// Request to execute a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteToolRequest {
    pub resource: String,
    pub tool: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub stream: Option<bool>,
}

/// MCP message types for client-server communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MCPMessage {
    Initialize {
        name: String,
        version: String,
        capabilities: ClientCapabilities,
    },
    InitializeResult {
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    ListResources {},
    ListResourcesResult {
        resources: Vec<Resource>,
    },
    ListTools {},
    ListToolsResult {
        tools: Vec<Tool>,
    },
    GetCapabilities {},
    GetCapabilitiesResult {
        capabilities: ServerCapabilities,
    },
    ExecuteTool(ExecuteToolRequest),
    ExecuteToolResult {
        result: String,
    },
    ExecuteToolError {
        error: String,
    },
    ExecuteToolStreamResult {
        data: String,
    },
    ExecuteToolStreamEnd {
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    ReadResource {
        uri: String,
    },
    ReadResourceResult {
        contents: Vec<ResourceContent>,
    },
    SubscribeResource {
        uri: String,
    },
    UnsubscribeResource {
        uri: String,
    },
    ResourceUpdated {
        uri: String,
        contents: Vec<ResourceContent>,
    },
    Ping {
        id: String,
    },
    Pong {
        id: String,
    },
    Error {
        error: String,
    },
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingCapability>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// Tools capability
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Resources capability
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Logging capability
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LoggingCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// Tool definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
}

/// Resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub text: Option<String>,
    pub blob: Option<String>,
}