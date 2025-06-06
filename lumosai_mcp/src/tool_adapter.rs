//! MCP Tool Adapter - Converts MCP tools to Lumos tools
//! 
//! This module provides seamless integration between MCP tools and the Lumos tool system,
//! allowing MCP tools to be used as native Lumos tools with full type safety and validation.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;

use lumosai_core::tool::{Tool as LumosTool, ToolExecutionContext, ToolExecutionOptions, ToolSchema, ParameterSchema, SchemaFormat};
use lumosai_core::error::{Result as CoreResult, Error as CoreError};
use lumosai_core::base::Base;
use lumosai_core::logger::{Logger, Component as LogComponent, ConsoleLogger, LogLevel};
use lumosai_core::telemetry::TelemetrySink;

use crate::{EnhancedMCPManager, Result as MCPResult, MCPError};
use crate::types::{ToolDefinition, ParameterSchema as MCPParameterSchema};

/// Adapter that wraps MCP tools to work with the Lumos tool system
#[derive(Debug, Clone)]
pub struct MCPToolAdapter {
    /// Name of the MCP tool
    tool_name: String,
    /// Description of the tool
    description: String,
    /// MCP tool definition
    mcp_definition: ToolDefinition,
    /// Reference to the MCP manager
    mcp_manager: Arc<EnhancedMCPManager>,
    /// Server name where this tool is available
    server_name: String,
}

impl MCPToolAdapter {
    /// Create a new MCP tool adapter
    pub fn new(
        tool_name: String,
        description: String,
        mcp_definition: ToolDefinition,
        mcp_manager: Arc<EnhancedMCPManager>,
        server_name: String,
    ) -> Self {
        Self {
            tool_name,
            description,
            mcp_definition,
            mcp_manager,
            server_name,
        }
    }

    /// Convert MCP parameter schema to Lumos parameter schema
    fn convert_parameter_schema(mcp_param: &MCPParameterSchema) -> ParameterSchema {
        ParameterSchema {
            name: mcp_param.name.clone(),
            description: mcp_param.description.clone(),
            r#type: mcp_param.r#type.clone(),
            required: mcp_param.required.unwrap_or(false),
            properties: None,
            default: None,
        }
    }

    /// Convert MCP tool definition to Lumos tool schema
    fn create_tool_schema(&self) -> ToolSchema {
        let parameters: Vec<ParameterSchema> = self.mcp_definition.parameters
            .iter()
            .map(Self::convert_parameter_schema)
            .collect();

        // Create a simple JSON schema for the tool
        let json_schema = serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        });

        ToolSchema {
            parameters,
            json_schema: Some(json_schema),
            format: SchemaFormat::JsonSchema,
            output_schema: None,
        }
    }

    /// Validate parameters against MCP schema
    fn validate_parameters(&self, params: &Value) -> Result<HashMap<String, Value>, CoreError> {
        let mut validated_params = HashMap::new();

        if let Value::Object(param_map) = params {
            // Check required parameters
            for mcp_param in &self.mcp_definition.parameters {
                let param_name = &mcp_param.name;
                let is_required = mcp_param.required.unwrap_or(false);

                if is_required && !param_map.contains_key(param_name) {
                    return Err(CoreError::Tool(format!("Required parameter '{}' is missing", param_name)));
                }

                if let Some(value) = param_map.get(param_name) {
                    // Basic type validation
                    if !self.validate_parameter_type(value, &mcp_param.r#type) {
                        return Err(CoreError::Tool(format!("Parameter '{}' has invalid type. Expected: {}",
                                   param_name, mcp_param.r#type)));
                    }
                    validated_params.insert(param_name.clone(), value.clone());
                }
            }
        } else {
            return Err(CoreError::Tool("Parameters must be a JSON object".to_string()));
        }

        Ok(validated_params)
    }

    /// Validate parameter type
    fn validate_parameter_type(&self, value: &Value, expected_type: &str) -> bool {
        match expected_type {
            "string" => value.is_string(),
            "number" | "integer" => value.is_number(),
            "boolean" => value.is_boolean(),
            "array" => value.is_array(),
            "object" => value.is_object(),
            _ => true, // Unknown types are accepted
        }
    }
}

impl Base for MCPToolAdapter {
    fn name(&self) -> Option<&str> {
        Some(&self.tool_name)
    }

    fn component(&self) -> LogComponent {
        LogComponent::Tool
    }

    fn logger(&self) -> Arc<dyn Logger> {
        // Return a default logger for now
        Arc::new(ConsoleLogger::new(&self.tool_name, LogComponent::Tool, LogLevel::Info))
    }

    fn set_logger(&mut self, _logger: Arc<dyn Logger>) {
        // MCP tools don't support setting loggers directly
    }

    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        None
    }

    fn set_telemetry(&mut self, _telemetry: Arc<dyn TelemetrySink>) {
        // MCP tools don't support setting telemetry directly
    }
}

#[async_trait]
impl LumosTool for MCPToolAdapter {
    fn id(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn clone_box(&self) -> Box<dyn LumosTool> {
        Box::new(self.clone())
    }

    fn schema(&self) -> ToolSchema {
        self.create_tool_schema()
    }

    async fn execute(
        &self,
        parameters: Value,
        _context: ToolExecutionContext,
        _options: &ToolExecutionOptions,
    ) -> Result<Value, CoreError> {
        // Validate parameters
        let validated_params = self.validate_parameters(&parameters)?;

        // Execute via MCP manager
        match self.mcp_manager.execute_mcp_tool(&self.tool_name, validated_params).await {
            Ok(result) => Ok(result),
            Err(mcp_error) => {
                // Convert MCP error to Core error
                Err(CoreError::Tool(format!("MCP tool execution failed: {}", mcp_error)))
            }
        }
    }
}

/// Factory for creating MCP tool adapters
pub struct MCPToolFactory {
    mcp_manager: Arc<EnhancedMCPManager>,
}

impl MCPToolFactory {
    /// Create a new MCP tool factory
    pub fn new(mcp_manager: Arc<EnhancedMCPManager>) -> Self {
        Self { mcp_manager }
    }

    /// Create Lumos tools from all discovered MCP tools
    pub async fn create_all_tools(&self) -> MCPResult<Vec<Arc<dyn LumosTool>>> {
        let mut lumos_tools = Vec::new();
        
        // Discover all tools from MCP servers
        let discovered_tools = self.mcp_manager.auto_discover_tools().await?;
        
        for (server_name, tool_definitions) in discovered_tools {
            for tool_def in tool_definitions {
                let adapter = MCPToolAdapter::new(
                    tool_def.name.clone(),
                    tool_def.description.clone(),
                    tool_def,
                    self.mcp_manager.clone(),
                    server_name.clone(),
                );
                
                lumos_tools.push(Arc::new(adapter) as Arc<dyn LumosTool>);
            }
        }
        
        println!("🔧 Created {} Lumos tools from MCP servers", lumos_tools.len());
        Ok(lumos_tools)
    }

    /// Create a specific tool by name
    pub async fn create_tool(&self, tool_name: &str) -> MCPResult<Option<Arc<dyn LumosTool>>> {
        let discovered_tools = self.mcp_manager.auto_discover_tools().await?;
        
        for (server_name, tool_definitions) in discovered_tools {
            for tool_def in tool_definitions {
                if tool_def.name == tool_name {
                    let adapter = MCPToolAdapter::new(
                        tool_def.name.clone(),
                        tool_def.description.clone(),
                        tool_def,
                        self.mcp_manager.clone(),
                        server_name.clone(),
                    );
                    
                    return Ok(Some(Arc::new(adapter) as Arc<dyn LumosTool>));
                }
            }
        }
        
        Ok(None)
    }

    /// Get tools from a specific MCP server
    pub async fn create_tools_from_server(&self, server_name: &str) -> MCPResult<Vec<Arc<dyn LumosTool>>> {
        let mut lumos_tools = Vec::new();
        let discovered_tools = self.mcp_manager.auto_discover_tools().await?;
        
        if let Some(tool_definitions) = discovered_tools.get(server_name) {
            for tool_def in tool_definitions {
                let adapter = MCPToolAdapter::new(
                    tool_def.name.clone(),
                    tool_def.description.clone(),
                    tool_def.clone(),
                    self.mcp_manager.clone(),
                    server_name.to_string(),
                );
                
                lumos_tools.push(Arc::new(adapter) as Arc<dyn LumosTool>);
            }
        }
        
        Ok(lumos_tools)
    }
}

/// MCP integration helper for easy setup
pub struct MCPIntegration {
    manager: Arc<EnhancedMCPManager>,
    factory: MCPToolFactory,
}

impl MCPIntegration {
    /// Create a new MCP integration
    pub fn new() -> Self {
        let manager = Arc::new(EnhancedMCPManager::new(Default::default()));
        let factory = MCPToolFactory::new(manager.clone());
        
        Self { manager, factory }
    }

    /// Get the MCP manager
    pub fn manager(&self) -> &Arc<EnhancedMCPManager> {
        &self.manager
    }

    /// Get the tool factory
    pub fn factory(&self) -> &MCPToolFactory {
        &self.factory
    }

    /// Quick setup with common MCP servers
    pub async fn quick_setup(&self) -> MCPResult<()> {
        // Start background tasks
        self.manager.start_background_tasks().await;
        
        println!("🚀 MCP integration initialized successfully");
        Ok(())
    }

    /// Get all available tools as Lumos tools
    pub async fn get_all_tools(&self) -> MCPResult<Vec<Arc<dyn LumosTool>>> {
        self.factory.create_all_tools().await
    }
}

impl Default for MCPIntegration {
    fn default() -> Self {
        Self::new()
    }
}
