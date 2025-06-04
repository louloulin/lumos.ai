use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{Result, Error};
use crate::agent::{config::AgentConfig, executor::BasicAgent};
use crate::tool::{FunctionTool, ToolSchema, ParameterSchema};
use crate::tool::registry::{ToolRegistry, ToolMetadata, ToolCategory};

/// TypeScript-compatible agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSAgentConfig {
    pub name: String,
    pub instructions: String,
    pub model: String,
    pub tools: Vec<String>,
    pub memory_config: Option<TSMemoryConfig>,
    pub metadata: HashMap<String, Value>,
}

/// TypeScript-compatible memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSMemoryConfig {
    pub max_tokens: Option<usize>,
    pub strategy: String,
    pub persistence: bool,
}

/// TypeScript-compatible tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: TSParameterSchema,
    pub handler: String, // JavaScript function as string
}

/// TypeScript-compatible parameter schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSParameterSchema {
    pub r#type: String,
    pub properties: HashMap<String, TSPropertySchema>,
    pub required: Vec<String>,
}

/// TypeScript-compatible property schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSPropertySchema {
    pub r#type: String,
    pub description: String,
    pub default: Option<Value>,
    pub enum_values: Option<Vec<Value>>,
}

/// TypeScript-compatible agent response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSAgentResponse {
    pub content: String,
    pub tool_calls: Vec<TSToolCall>,
    pub metadata: HashMap<String, Value>,
    pub usage: Option<TSUsageStats>,
}

/// TypeScript-compatible tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
    pub result: Option<Value>,
}

/// TypeScript-compatible usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TSUsageStats {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// TypeScript bindings for Lumos.ai
pub struct TypeScriptBindings;

impl TypeScriptBindings {
    /// Create a new agent from TypeScript configuration
    pub async fn create_agent(config: TSAgentConfig) -> Result<String> {
        let agent_config = AgentConfig {
            name: config.name.clone(),
            instructions: config.instructions.clone(),
            model_id: Some(config.model),
            voice_config: None,
            telemetry: None,
            working_memory: None,
            enable_function_calling: Some(true),
            context: None,
            metadata: Some({
                let mut meta = HashMap::new();
                for (k, v) in config.metadata {
                    meta.insert(k, v.to_string());
                }
                meta
            }),
            max_tool_calls: None,
            tool_timeout: None,
            memory_config: None,
        };

        // For now, just return the agent name as ID
        // In a real implementation, we would create and store the agent

        // Return agent ID for JavaScript reference
        Ok(config.name)
    }

    /// Execute agent with TypeScript-compatible interface
    pub async fn execute_agent(
        agent_id: &str,
        message: &str,
        context: Option<HashMap<String, Value>>
    ) -> Result<TSAgentResponse> {
        // In a real implementation, we would maintain a registry of agents
        // For now, return a mock response
        Ok(TSAgentResponse {
            content: format!("Response from agent {} to: {}", agent_id, message),
            tool_calls: vec![],
            metadata: context.unwrap_or_default(),
            usage: Some(TSUsageStats {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }),
        })
    }

    /// Register a tool from TypeScript
    pub async fn register_tool(tool_def: TSToolDefinition) -> Result<()> {
        let parameter_schema = ParameterSchema {
            name: "parameters".to_string(),
            description: "Tool parameters".to_string(),
            r#type: tool_def.parameters.r#type,
            required: !tool_def.parameters.required.is_empty(),
            properties: None,
            default: None,
        };

        let tool_schema = ToolSchema::new(vec![parameter_schema]);

        // Create a function tool with JavaScript handler
        let tool = FunctionTool::new(
            tool_def.name.clone(),
            tool_def.description.clone(),
            tool_schema,
            move |_params| {
                // In a real implementation, we would execute the JavaScript handler
                Ok(serde_json::json!({"result": "Tool executed successfully"}))
            }
        );

        // Create tool metadata
        let metadata = ToolMetadata {
            name: tool_def.name,
            description: tool_def.description,
            version: "1.0.0".to_string(),
            author: None,
            category: ToolCategory::Network, // Use an existing category
            tags: vec!["typescript".to_string()],
            requires_auth: false,
            permissions: vec![],
            dependencies: vec![],
        };

        // Register tool in registry
        let registry = ToolRegistry::new();
        registry.register_tool(std::sync::Arc::new(tool), metadata)?;

        Ok(())
    }

    /// List available tools for TypeScript
    pub async fn list_tools() -> Result<Vec<TSToolDefinition>> {
        let registry = ToolRegistry::new();
        let tools = registry.list_tools()?;

        let mut ts_tools = Vec::new();
        for tool_name in tools {
            if let Some(tool) = registry.get_tool(&tool_name)? {
                let ts_tool = TSToolDefinition {
                    name: tool_name.clone(),
                    description: tool.description().to_string(),
                    parameters: TSParameterSchema {
                        r#type: "object".to_string(),
                        properties: HashMap::new(), // Simplified for now
                        required: vec![],
                    },
                    handler: "// JavaScript handler".to_string(),
                };
                ts_tools.push(ts_tool);
            }
        }

        Ok(ts_tools)
    }

    /// Convert Rust error to TypeScript-compatible format
    pub fn format_error(error: &Error) -> Value {
        serde_json::json!({
            "type": "LumosError",
            "message": error.to_string(),
            "code": match error {
                Error::Agent(_) => "AGENT_ERROR",
                Error::Tool(_) => "TOOL_ERROR",
                Error::Memory(_) => "MEMORY_ERROR",
                Error::Network(_) => "NETWORK_ERROR",
                Error::Io(_) => "IO_ERROR",
                Error::Json(_) => "SERIALIZATION_ERROR",
                Error::Internal(_) => "INTERNAL_ERROR",
                Error::Other(_) => "UNKNOWN_ERROR",
                Error::Llm(_) => "LLM_ERROR",
                Error::Storage(_) => "STORAGE_ERROR",
                Error::Workflow(_) => "WORKFLOW_ERROR",
                Error::Http(_) => "HTTP_ERROR",
                Error::Lock(_) => "LOCK_ERROR",
                _ => "UNKNOWN_ERROR",
            }
        })
    }

    /// Generate TypeScript type definitions
    pub fn generate_type_definitions() -> String {
        r#"
// Lumos.ai TypeScript Bindings
export interface AgentConfig {
  name: string;
  instructions: string;
  model: string;
  tools: string[];
  memoryConfig?: MemoryConfig;
  metadata?: Record<string, any>;
}

export interface MemoryConfig {
  maxTokens?: number;
  strategy: 'sliding_window' | 'summarization';
  persistence: boolean;
}

export interface ToolDefinition {
  name: string;
  description: string;
  parameters: ParameterSchema;
  handler: string;
}

export interface ParameterSchema {
  type: string;
  properties: Record<string, PropertySchema>;
  required: string[];
}

export interface PropertySchema {
  type: string;
  description: string;
  default?: any;
  enumValues?: any[];
}

export interface AgentResponse {
  content: string;
  toolCalls: ToolCall[];
  metadata: Record<string, any>;
  usage?: UsageStats;
}

export interface ToolCall {
  id: string;
  name: string;
  arguments: any;
  result?: any;
}

export interface UsageStats {
  promptTokens: number;
  completionTokens: number;
  totalTokens: number;
}

export interface LumosError {
  type: 'LumosError';
  message: string;
  code: string;
}

// Main Lumos client class
export class LumosClient {
  static async createAgent(config: AgentConfig): Promise<string>;
  static async executeAgent(
    agentId: string, 
    message: string, 
    context?: Record<string, any>
  ): Promise<AgentResponse>;
  static async registerTool(tool: ToolDefinition): Promise<void>;
  static async listTools(): Promise<ToolDefinition[]>;
}
"#.to_string()
    }
}
