//! WebSocket streaming support for Lumosai agents
//! 
//! This module provides WebSocket-based real-time streaming capabilities
//! for agent interactions, enabling bidirectional communication and 
//! live event broadcasting to connected clients.

use std::collections::HashMap;
use std::sync::Arc;
use std::pin::Pin;
use async_stream::stream;
use futures::{Stream, StreamExt, SinkExt};
use serde::{Serialize, Deserialize};
use tokio::sync::{broadcast, RwLock, mpsc};
use uuid::Uuid;

use crate::agent::streaming::{AgentEvent, StreamingAgent, StreamingConfig};
use crate::agent::trait_def::Agent;
use crate::agent::types::AgentGenerateOptions;
use crate::llm::Message;

/// WebSocket message types for agent streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    /// Agent event forwarded over WebSocket
    AgentEvent {
        event: AgentEvent,
        session_id: String,
        timestamp: u64,
    },
    
    /// Client connection established
    Connected {
        client_id: String,
        session_id: String,
        timestamp: u64,
    },
    
    /// Client disconnected
    Disconnected {
        client_id: String,
        session_id: String,
        timestamp: u64,
    },
    
    /// Heartbeat ping
    Ping {
        timestamp: u64,
    },
    
    /// Heartbeat pong
    Pong {
        timestamp: u64,
    },
    
    /// Error message
    Error {
        error: String,
        session_id: Option<String>,
        timestamp: u64,
    },
    
    /// Session management
    SessionStart {
        session_id: String,
        timestamp: u64,
    },
    
    SessionEnd {
        session_id: String,
        timestamp: u64,
    },
}

/// WebSocket connection information
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub client_id: String,
    pub session_id: String,
    pub connected_at: u64,
    pub last_ping: u64,
    pub sender: mpsc::UnboundedSender<WebSocketMessage>,
}

/// Configuration for WebSocket streaming
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,
    
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    
    /// Buffer size for message queues
    pub message_buffer_size: usize,
    
    /// Whether to broadcast agent events to all connections
    pub broadcast_events: bool,
    
    /// Whether to include session isolation
    pub session_isolation: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            heartbeat_interval_ms: 30000, // 30 seconds
            connection_timeout_ms: 90000, // 90 seconds
            message_buffer_size: 1000,
            broadcast_events: true,
            session_isolation: true,
        }
    }
}

/// WebSocket manager for handling multiple client connections
pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    sessions: Arc<RwLock<HashMap<String, Vec<String>>>>, // session_id -> client_ids
    config: WebSocketConfig,
    broadcast_tx: broadcast::Sender<WebSocketMessage>,
}

impl WebSocketManager {
    /// Create a new WebSocket manager
    pub fn new(config: WebSocketConfig) -> Self {
        let (broadcast_tx, _) = broadcast::channel(config.message_buffer_size);
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
            broadcast_tx,
        }
    }
    
    /// Add a new WebSocket connection
    pub async fn add_connection(
        &self,
        client_id: String,
        session_id: String,
        sender: mpsc::UnboundedSender<WebSocketMessage>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut connections = self.connections.write().await;
        
        // Check connection limit
        if connections.len() >= self.config.max_connections {
            return Err("Maximum connections reached".into());
        }
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let connection = WebSocketConnection {
            client_id: client_id.clone(),
            session_id: session_id.clone(),
            connected_at: now,
            last_ping: now,
            sender,
        };
        
        connections.insert(client_id.clone(), connection);
        
        // Update session mapping
        let mut sessions = self.sessions.write().await;
        sessions
            .entry(session_id.clone())
            .or_insert_with(Vec::new)
            .push(client_id.clone());
        
        // Send connection confirmation
        let connected_message = WebSocketMessage::Connected {
            client_id: client_id.clone(),
            session_id: session_id.clone(),
            timestamp: now,
        };
        
        if let Some(conn) = connections.get(&client_id) {
            let _ = conn.sender.send(connected_message.clone());
        }
        
        // Broadcast to other connections if enabled
        if self.config.broadcast_events {
            let _ = self.broadcast_tx.send(connected_message);
        }
        
        Ok(())
    }
    
    /// Remove a WebSocket connection
    pub async fn remove_connection(&self, client_id: &str) {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.remove(client_id) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            
            // Update session mapping
            let mut sessions = self.sessions.write().await;
            if let Some(client_ids) = sessions.get_mut(&connection.session_id) {
                client_ids.retain(|id| id != client_id);
                if client_ids.is_empty() {
                    sessions.remove(&connection.session_id);
                }
            }
            
            // Send disconnection message
            let disconnected_message = WebSocketMessage::Disconnected {
                client_id: client_id.to_string(),
                session_id: connection.session_id.clone(),
                timestamp: now,
            };
            
            // Broadcast to other connections if enabled
            if self.config.broadcast_events {
                let _ = self.broadcast_tx.send(disconnected_message);
            }
        }
    }
    
    /// Broadcast a message to all connections
    pub async fn broadcast(&self, message: WebSocketMessage) {
        let _ = self.broadcast_tx.send(message);
    }
    
    /// Send a message to a specific session
    pub async fn send_to_session(
        &self,
        session_id: &str,
        message: WebSocketMessage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connections = self.connections.read().await;
        let sessions = self.sessions.read().await;
        
        if let Some(client_ids) = sessions.get(session_id) {
            for client_id in client_ids {
                if let Some(connection) = connections.get(client_id) {
                    let _ = connection.sender.send(message.clone());
                }
            }
        }
        
        Ok(())
    }
    
    /// Send a message to a specific client
    pub async fn send_to_client(
        &self,
        client_id: &str,
        message: WebSocketMessage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connections = self.connections.read().await;
        
        if let Some(connection) = connections.get(client_id) {
            connection.sender.send(message)?;
        }
        
        Ok(())
    }
    
    /// Get connection count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
    
    /// Get session count
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
    
    /// Update last ping time for a connection
    pub async fn update_ping(&self, client_id: &str) {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.get_mut(client_id) {
            connection.last_ping = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
        }
    }
    
    /// Clean up stale connections
    pub async fn cleanup_stale_connections(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let mut connections = self.connections.write().await;
        let mut sessions = self.sessions.write().await;
        
        let stale_clients: Vec<String> = connections
            .iter()
            .filter(|(_, conn)| {
                now - conn.last_ping > self.config.connection_timeout_ms
            })
            .map(|(client_id, _)| client_id.clone())
            .collect();
        
        for client_id in stale_clients {
            if let Some(connection) = connections.remove(&client_id) {
                // Update session mapping
                if let Some(client_ids) = sessions.get_mut(&connection.session_id) {
                    client_ids.retain(|id| id != &client_id);
                    if client_ids.is_empty() {
                        sessions.remove(&connection.session_id);
                    }
                }
            }
        }
    }
    
    /// Subscribe to broadcast messages
    pub fn subscribe(&self) -> broadcast::Receiver<WebSocketMessage> {
        self.broadcast_tx.subscribe()
    }
}

/// WebSocket-enabled streaming agent
pub struct WebSocketStreamingAgent<T: Agent> {
    streaming_agent: StreamingAgent<T>,
    websocket_manager: Arc<WebSocketManager>,
    config: WebSocketConfig,
}

impl<T: Agent> WebSocketStreamingAgent<T> {
    /// Create a new WebSocket streaming agent
    pub fn new(
        base_agent: T,
        streaming_config: StreamingConfig,
        websocket_config: WebSocketConfig,
    ) -> Self {
        let streaming_agent = StreamingAgent::with_config(base_agent, streaming_config);
        let websocket_manager = Arc::new(WebSocketManager::new(websocket_config.clone()));
        
        Self {
            streaming_agent,
            websocket_manager,
            config: websocket_config,
        }
    }
    
    /// Get the WebSocket manager
    pub fn websocket_manager(&self) -> Arc<WebSocketManager> {
        self.websocket_manager.clone()
    }
    
    /// Execute agent with WebSocket streaming
    pub async fn execute_with_websocket<'a>(
        &'a self,
        messages: &'a [Message],
        options: &'a AgentGenerateOptions,
        session_id: &str,
    ) -> Pin<Box<dyn Stream<Item = std::result::Result<AgentEvent, Box<dyn std::error::Error + Send + Sync>>> + Send + 'a>> {
        let session_id = session_id.to_string();
        let websocket_manager = self.websocket_manager.clone();
        let config = self.config.clone();
        
        Box::pin(stream! {
            // Start session
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            
            let session_start_msg = WebSocketMessage::SessionStart {
                session_id: session_id.clone(),
                timestamp: now,
            };
            
            if config.session_isolation {
                let _ = websocket_manager.send_to_session(&session_id, session_start_msg.clone()).await;
            } else {
                websocket_manager.broadcast(session_start_msg).await;
            }
            
            // Execute streaming and forward events via WebSocket
            let mut agent_stream = self.streaming_agent.execute_streaming(messages, options);
            
            while let Some(event_result) = agent_stream.next().await {
                match event_result {
                    Ok(event) => {
                        // Forward the event directly to the stream consumer
                        yield Ok(event.clone());
                        
                        // Also send via WebSocket
                        let ws_message = WebSocketMessage::AgentEvent {
                            event,
                            session_id: session_id.clone(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64,
                        };
                        
                        if config.session_isolation {
                            let _ = websocket_manager.send_to_session(&session_id, ws_message).await;
                        } else {
                            websocket_manager.broadcast(ws_message).await;
                        }
                    },
                    Err(e) => {
                        // Create error string for forwarding
                        let error_msg = e.to_string();
                        
                        // Forward the error by creating a new Error from the string
                        yield Err(crate::Error::InvalidOperation(error_msg.clone()).into());
                        
                        // Send error via WebSocket
                        let error_message = WebSocketMessage::Error {
                            error: error_msg,
                            session_id: Some(session_id.clone()),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64,
                        };
                        
                        if config.session_isolation {
                            let _ = websocket_manager.send_to_session(&session_id, error_message).await;
                        } else {
                            websocket_manager.broadcast(error_message).await;
                        }
                    }
                }
            }
            
            // End session
            let session_end_msg = WebSocketMessage::SessionEnd {
                session_id: session_id.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            };
            
            if config.session_isolation {
                let _ = websocket_manager.send_to_session(&session_id, session_end_msg).await;
            } else {
                websocket_manager.broadcast(session_end_msg).await;
            }
        })
    }
    
    /// Start heartbeat monitoring task
    pub fn start_heartbeat_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let websocket_manager = self.websocket_manager.clone();
        let interval_ms = self.config.heartbeat_interval_ms;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_millis(interval_ms)
            );
            
            loop {
                interval.tick().await;
                websocket_manager.cleanup_stale_connections().await;
                
                // Send ping to all connections
                let ping_message = WebSocketMessage::Ping {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                };
                
                websocket_manager.broadcast(ping_message).await;
            }
        })
    }
}

/// Helper trait to add WebSocket streaming capabilities
pub trait IntoWebSocketStreaming<T: Agent> {
    fn into_websocket_streaming(
        self,
        streaming_config: StreamingConfig,
        websocket_config: WebSocketConfig,
    ) -> WebSocketStreamingAgent<T>;
}

impl<T: Agent> IntoWebSocketStreaming<T> for T {
    fn into_websocket_streaming(
        self,
        streaming_config: StreamingConfig,
        websocket_config: WebSocketConfig,
    ) -> WebSocketStreamingAgent<T> {
        WebSocketStreamingAgent::new(self, streaming_config, websocket_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{AgentConfig, BasicAgent};
    use crate::memory::WorkingMemoryConfig;
    use crate::llm::MockLlmProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_websocket_manager_connection() {
        let config = WebSocketConfig::default();
        let manager = WebSocketManager::new(config);
        
        let (tx, _rx) = mpsc::unbounded_channel();
        
        let result = manager.add_connection(
            "client1".to_string(),
            "session1".to_string(),
            tx,
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(manager.connection_count().await, 1);
        assert_eq!(manager.session_count().await, 1);
    }
    
    #[tokio::test]
    async fn test_websocket_manager_broadcast() {
        let config = WebSocketConfig::default();
        let manager = WebSocketManager::new(config);
        
        let message = WebSocketMessage::Ping {
            timestamp: 123456789,
        };
        
        // Should not fail even with no connections
        manager.broadcast(message).await;
    }
    
    #[tokio::test]
    async fn test_websocket_streaming_agent_creation() {
        // Create a working memory config
        let wm_config = WorkingMemoryConfig {
            enabled: true,
            template: None,
            content_type: Some("application/json".to_string()),
            max_capacity: Some(1024),
        };
        
        // Create agent config with working memory
        let agent_config = AgentConfig {
            name: "test_agent".to_string(),
            instructions: "Test agent".to_string(),
            working_memory: Some(wm_config),
            ..Default::default()
        };
        
        let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
        let agent = BasicAgent::new(agent_config, llm);
        
        let streaming_config = StreamingConfig::default();
        let websocket_config = WebSocketConfig::default();
        
        let ws_agent = agent.into_websocket_streaming(streaming_config, websocket_config);
        
        assert_eq!(ws_agent.websocket_manager().connection_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_websocket_manager_cleanup() {
        let mut config = WebSocketConfig::default();
        config.connection_timeout_ms = 1; // Very short timeout for testing
        
        let manager = WebSocketManager::new(config);
        
        let (tx, _rx) = mpsc::unbounded_channel();
        
        let _ = manager.add_connection(
            "client1".to_string(),
            "session1".to_string(),
            tx,
        ).await;
        
        assert_eq!(manager.connection_count().await, 1);
        
        // Wait a bit to let the connection become stale
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        manager.cleanup_stale_connections().await;
        assert_eq!(manager.connection_count().await, 0);
    }
}
