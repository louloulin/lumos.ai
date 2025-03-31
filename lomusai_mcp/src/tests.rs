#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    
    
    
    use futures::StreamExt;
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
} 