#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    // use futures::StreamExt; // 暂时未使用
    use mockall::predicate::*;
    use mockall::mock;
    use tokio::sync::mpsc;
    
    
    use crate::client::MCPClient;
    use crate::error::Result;
    use crate::types::{MCPMessage, Resource, ResourceMetadata, ToolDefinition, ParameterSchema};
    use crate::transport::Transport;
    
    // Create a mock for the Transport trait
    mock! {
        pub Transport {}
        
        #[async_trait::async_trait]
        impl Transport for Transport {
            async fn connect(&mut self) -> Result<()>;
            async fn disconnect(&mut self) -> Result<()>;
            async fn send_message(&mut self, message: &MCPMessage) -> Result<()>;
            async fn receive_message(&mut self) -> Result<MCPMessage>;
            fn message_stream(&self) -> Result<mpsc::Receiver<Result<MCPMessage>>>;
        }
    }
    
    #[tokio::test]
    async fn test_client_connect() {
        let mut mock_transport = MockTransport::new();
        mock_transport.expect_connect()
            .times(1)
            .returning(|| Ok(()));
            
        mock_transport.expect_send_message()
            .with(function(|msg| matches!(msg, MCPMessage::Initialize { .. })))
            .times(1)
            .returning(|_| Ok(()));
            
        mock_transport.expect_receive_message()
            .times(1)
            .returning(|| Ok(MCPMessage::InitializeResult {
                status: "success".to_string(),
                error: None,
            }));
            
        let client = MCPClient::new(
            "test",
            crate::types::ServerParameters::Stdio(crate::types::StdioServerParameters {
                command: "test".to_string(),
                args: vec!["arg1".to_string(), "arg2".to_string()],
                env: HashMap::new(),
            }),
            None,
            None,
            None,
        );
        
        // Replace the transport with our mock
        let transport_field = client.transport.clone();
        *transport_field.lock().await = Box::new(mock_transport);
        
        // Test connect
        let result = client.connect().await;
        assert!(result.is_ok(), "Failed to connect: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_client_resources() {
        let mut mock_transport = MockTransport::new();
        mock_transport.expect_connect()
            .times(1)
            .returning(|| Ok(()));
            
        mock_transport.expect_send_message()
            .times(2)  // Initialize + ListResources
            .returning(|_| Ok(()));
            
        // First message is the initialization result
        mock_transport.expect_receive_message()
            .times(1)
            .returning(|| Ok(MCPMessage::InitializeResult {
                status: "success".to_string(),
                error: None,
            }));
            
        // Second message is the resources list
        mock_transport.expect_receive_message()
            .times(1)
            .returning(|| {
                let metadata = ResourceMetadata {
                    name: "test_resource".to_string(),
                    version: "1.0".to_string(),
                    description: Some("Test resource".to_string()),
                    properties: HashMap::new(),
                };
                
                let tool_def = ToolDefinition {
                    name: "test_tool".to_string(),
                    description: "A test tool".to_string(),
                    parameters: vec![
                        ParameterSchema {
                            name: "param1".to_string(),
                            description: "A test parameter".to_string(),
                            r#type: "string".to_string(),
                            required: Some(true),
                            schema: None,
                        }
                    ],
                    return_schema: None,
                };
                
                let resource = Resource {
                    metadata,
                    tools: vec![tool_def],
                };
                
                Ok(MCPMessage::ListResourcesResult {
                    resources: vec![resource],
                })
            });
            
        let client = MCPClient::new(
            "test",
            crate::types::ServerParameters::Stdio(crate::types::StdioServerParameters {
                command: "test".to_string(),
                args: vec!["arg1".to_string(), "arg2".to_string()],
                env: HashMap::new(),
            }),
            None,
            None,
            None,
        );
        
        // Replace the transport with our mock
        let transport_field = client.transport.clone();
        *transport_field.lock().await = Box::new(mock_transport);
        
        // Test resources
        let result = client.resources().await;
        assert!(result.is_ok(), "Failed to get resources: {:?}", result);
        
        let resources = result.unwrap();
        assert_eq!(resources.resources.len(), 1, "Expected 1 resource");
        assert_eq!(resources.resources[0].metadata.name, "test_resource");
        assert_eq!(resources.resources[0].tools.len(), 1, "Expected 1 tool");
        assert_eq!(resources.resources[0].tools[0].name, "test_tool");
    }
    
    #[tokio::test]
    async fn test_client_tools() {
        let mut mock_transport = MockTransport::new();
        mock_transport.expect_connect()
            .times(1)
            .returning(|| Ok(()));
            
        mock_transport.expect_send_message()
            .times(2)  // Initialize + ListResources
            .returning(|_| Ok(()));
            
        // First message is the initialization result
        mock_transport.expect_receive_message()
            .times(1)
            .returning(|| Ok(MCPMessage::InitializeResult {
                status: "success".to_string(),
                error: None,
            }));
            
        // Second message is the resources list
        mock_transport.expect_receive_message()
            .times(1)
            .returning(|| {
                let metadata = ResourceMetadata {
                    name: "test_resource".to_string(),
                    version: "1.0".to_string(),
                    description: Some("Test resource".to_string()),
                    properties: HashMap::new(),
                };
                
                let tool_def = ToolDefinition {
                    name: "test_tool".to_string(),
                    description: "A test tool".to_string(),
                    parameters: vec![
                        ParameterSchema {
                            name: "param1".to_string(),
                            description: "A test parameter".to_string(),
                            r#type: "string".to_string(),
                            required: Some(true),
                            schema: None,
                        }
                    ],
                    return_schema: None,
                };
                
                let resource = Resource {
                    metadata,
                    tools: vec![tool_def],
                };
                
                Ok(MCPMessage::ListResourcesResult {
                    resources: vec![resource],
                })
            });
            
        let client = MCPClient::new(
            "test",
            crate::types::ServerParameters::Stdio(crate::types::StdioServerParameters {
                command: "test".to_string(),
                args: vec!["arg1".to_string(), "arg2".to_string()],
                env: HashMap::new(),
            }),
            None,
            None,
            None,
        );
        
        // Replace the transport with our mock
        let transport_field = client.transport.clone();
        *transport_field.lock().await = Box::new(mock_transport);
        
        // Test tools
        let result = client.tools().await;
        assert!(result.is_ok(), "Failed to get tools: {:?}", result);
        
        let tools = result.unwrap();
        assert_eq!(tools.len(), 1, "Expected 1 tool");
        assert!(tools.contains_key("test_resource_test_tool"), "Expected tool with correct name");
    }
    
    #[tokio::test]
    async fn test_configuration() {
        let mut servers = HashMap::new();
        servers.insert(
            "test_server".to_string(),
            crate::configuration::ServerDefinition::Stdio {
                command: "test".to_string(),
                args: vec!["arg1".to_string(), "arg2".to_string()],
                env: None,
            },
        );

        let config = crate::configuration::MCPConfiguration::new(servers, Some("test_config".to_string()));
        assert_eq!(config.id, "test_config");
    }

    #[tokio::test]
    async fn test_enhanced_mcp_manager() {
        use crate::enhanced::{EnhancedMCPManager, ManagerConfig};
        use std::time::Duration;

        let config = ManagerConfig {
            health_check_interval: Duration::from_secs(10),
            max_consecutive_failures: 2,
            connection_timeout: Duration::from_secs(5),
            tool_cache_ttl: Duration::from_secs(60),
            auto_reconnect: true,
            max_retry_attempts: 2,
        };

        let manager = EnhancedMCPManager::new(config);

        // Test adding a client
        let mut servers = HashMap::new();
        servers.insert(
            "test_server".to_string(),
            crate::configuration::ServerDefinition::Stdio {
                command: "echo".to_string(),
                args: vec!["test".to_string()],
                env: None,
            },
        );

        let mcp_config = crate::configuration::MCPConfiguration::new(servers, Some("test".to_string()));
        let result = manager.add_client("test_server".to_string(), mcp_config).await;
        assert!(result.is_ok(), "Failed to add client: {:?}", result);

        // Test getting health status
        let health_status = manager.get_health_status().await;
        assert!(health_status.contains_key("test_server"), "Health status should contain test_server");

        // Test getting metrics
        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.total_requests, 0, "Initial metrics should be zero");
    }

    #[tokio::test]
    async fn test_mcp_tool_adapter() {
        use crate::tool_adapter::MCPToolAdapter;
        use crate::enhanced::{EnhancedMCPManager, ManagerConfig};
        use crate::types::{ToolDefinition, ParameterSchema};
        use lumosai_core::tool::{Tool as LumosTool, ToolExecutionContext, ToolExecutionOptions};
        use lumosai_core::base::Base;
        use serde_json::json;

        let manager = Arc::new(EnhancedMCPManager::new(ManagerConfig::default()));

        let tool_def = ToolDefinition {
            name: "test_tool".to_string(),
            description: "A test tool for MCP adapter".to_string(),
            parameters: vec![
                ParameterSchema {
                    name: "input".to_string(),
                    description: "Test input parameter".to_string(),
                    r#type: "string".to_string(),
                    required: Some(true),
                    schema: None,
                }
            ],
            return_schema: None,
        };

        let adapter = MCPToolAdapter::new(
            "test_tool".to_string(),
            "Test tool description".to_string(),
            tool_def,
            manager.clone(),
            "test_server".to_string(),
        );

        // Test tool metadata
        assert_eq!(adapter.name(), Some("test_tool"));
        assert_eq!(adapter.description(), "Test tool description");

        let schema = adapter.schema();
        assert_eq!(schema.parameters.len(), 1);
        assert_eq!(schema.parameters[0].name, "input");

        // Test parameter validation (this will fail execution but validate parameters)
        let params = json!({"input": "test_value"});
        let context = ToolExecutionContext::new();
        let options = ToolExecutionOptions::new();

        // This will fail because we don't have a real MCP server, but it tests the validation
        let result = adapter.execute(params, context, &options).await;
        assert!(result.is_err(), "Expected execution to fail without real MCP server");
    }

    #[tokio::test]
    async fn test_mcp_server_registry() {
        use crate::discovery::{MCPServerRegistry, ServerConfig, ServerType, ConnectionConfig};
        use crate::enhanced::{EnhancedMCPManager, ManagerConfig};

        let manager = Arc::new(EnhancedMCPManager::new(ManagerConfig::default()));
        let mut registry = MCPServerRegistry::new(manager);

        // Test registering a server
        let server_config = ServerConfig {
            name: "test_calculator".to_string(),
            description: "Test calculator server".to_string(),
            server_type: ServerType::Stdio,
            connection: ConnectionConfig::Stdio {
                command: "npx".to_string(),
                args: vec!["@modelcontextprotocol/calculator".to_string()],
                env: HashMap::new(),
                working_dir: None,
            },
            capabilities: vec!["math".to_string(), "calculator".to_string()],
            tags: vec!["utility".to_string(), "math".to_string()],
            enabled: true,
            priority: 80,
        };

        let result = registry.register_server(server_config.clone());
        assert!(result.is_ok(), "Failed to register server: {:?}", result);

        // Test getting servers
        let servers = registry.get_servers();
        assert!(servers.contains_key("test_calculator"), "Registry should contain test_calculator");

        // Test getting servers by capability
        let math_servers = registry.get_servers_by_capability("math");
        assert_eq!(math_servers.len(), 1, "Should find 1 server with math capability");
        assert_eq!(math_servers[0].name, "test_calculator");

        // Test getting servers by tag
        let utility_servers = registry.get_servers_by_tag("utility");
        assert_eq!(utility_servers.len(), 1, "Should find 1 server with utility tag");

        // Test enabling/disabling servers
        let result = registry.disable_server("test_calculator");
        assert!(result.is_ok(), "Failed to disable server");

        let servers = registry.get_servers();
        assert!(!servers.get("test_calculator").unwrap().enabled, "Server should be disabled");

        let result = registry.enable_server("test_calculator");
        assert!(result.is_ok(), "Failed to enable server");

        let servers = registry.get_servers();
        assert!(servers.get("test_calculator").unwrap().enabled, "Server should be enabled");
    }

    #[tokio::test]
    async fn test_mcp_integration() {
        use crate::tool_adapter::MCPIntegration;

        let integration = MCPIntegration::new();

        // Test quick setup
        let result = integration.quick_setup().await;
        assert!(result.is_ok(), "Failed to setup MCP integration: {:?}", result);

        // Test getting manager and factory
        let _manager = integration.manager();
        let _factory = integration.factory();

        // Test getting all tools (will be empty without real servers)
        let tools = integration.get_all_tools().await;
        assert!(tools.is_ok(), "Failed to get tools: {:?}", tools);

        let tool_list = tools.unwrap();
        // Should be empty since we don't have real MCP servers connected
        assert_eq!(tool_list.len(), 0, "Should have no tools without real servers");
    }

    #[tokio::test]
    async fn test_server_status_report() {
        use crate::enhanced::{EnhancedMCPManager, ManagerConfig};

        let manager = EnhancedMCPManager::new(ManagerConfig::default());

        // Add a test client
        let mut servers = HashMap::new();
        servers.insert(
            "status_test_server".to_string(),
            crate::configuration::ServerDefinition::Stdio {
                command: "echo".to_string(),
                args: vec!["test".to_string()],
                env: None,
            },
        );

        let mcp_config = crate::configuration::MCPConfiguration::new(servers, Some("status_test".to_string()));
        let result = manager.add_client("status_test_server".to_string(), mcp_config).await;
        assert!(result.is_ok(), "Failed to add client for status test");

        // Get server status report
        let report = manager.get_server_status_report().await;
        assert!(report.contains_key("status_test_server"), "Report should contain status_test_server");

        let server_status = &report["status_test_server"];
        assert_eq!(server_status.name, "status_test_server");
        assert_eq!(server_status.tool_count, 0); // No tools discovered yet
        assert_eq!(server_status.subscription_count, 0); // No subscriptions yet
    }
}