//! Tests for WebSocket streaming functionality

use std::sync::Arc;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

use lumosai_core::agent::{
    AgentConfig, BasicAgent, AgentGenerateOptions,
    StreamingConfig, WebSocketConfig, 
    IntoWebSocketStreaming, WebSocketMessage, AgentEvent
};
use lumosai_core::llm::{MockLlmProvider, LlmOptions};
use lumosai_core::memory::WorkingMemoryConfig;
use lumosai_core::agent::message_utils::user_message;

#[tokio::test]
async fn test_websocket_manager_basic_operations() {
    let config = WebSocketConfig::default();
    let manager = lumosai_core::agent::websocket::WebSocketManager::new(config);
    
    // Initially empty
    assert_eq!(manager.connection_count().await, 0);
    assert_eq!(manager.session_count().await, 0);
    
    // Add connection
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let result = manager.add_connection(
        "test_client".to_string(),
        "test_session".to_string(),
        tx,
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(manager.connection_count().await, 1);
    assert_eq!(manager.session_count().await, 1);
    
    // Remove connection
    manager.remove_connection("test_client").await;
    assert_eq!(manager.connection_count().await, 0);
    assert_eq!(manager.session_count().await, 0);
}

#[tokio::test]
async fn test_websocket_manager_multiple_clients_same_session() {
    let config = WebSocketConfig::default();
    let manager = lumosai_core::agent::websocket::WebSocketManager::new(config);
    
    let (tx1, _rx1) = tokio::sync::mpsc::unbounded_channel();
    let (tx2, _rx2) = tokio::sync::mpsc::unbounded_channel();
    
    // Add two clients to same session
    let _ = manager.add_connection(
        "client1".to_string(),
        "shared_session".to_string(),
        tx1,
    ).await;
    
    let _ = manager.add_connection(
        "client2".to_string(),
        "shared_session".to_string(),
        tx2,
    ).await;
    
    assert_eq!(manager.connection_count().await, 2);
    assert_eq!(manager.session_count().await, 1);
    
    // Remove one client
    manager.remove_connection("client1").await;
    assert_eq!(manager.connection_count().await, 1);
    assert_eq!(manager.session_count().await, 1);
    
    // Remove second client
    manager.remove_connection("client2").await;
    assert_eq!(manager.connection_count().await, 0);
    assert_eq!(manager.session_count().await, 0);
}

#[tokio::test]
async fn test_websocket_manager_connection_limit() {
    let mut config = WebSocketConfig::default();
    config.max_connections = 2; // Set low limit for testing
    
    let manager = lumosai_core::agent::websocket::WebSocketManager::new(config);
    
    let (tx1, _rx1) = tokio::sync::mpsc::unbounded_channel();
    let (tx2, _rx2) = tokio::sync::mpsc::unbounded_channel();
    let (tx3, _rx3) = tokio::sync::mpsc::unbounded_channel();
    
    // Add two connections (should succeed)
    assert!(manager.add_connection("client1".to_string(), "session1".to_string(), tx1).await.is_ok());
    assert!(manager.add_connection("client2".to_string(), "session2".to_string(), tx2).await.is_ok());
    
    // Third connection should fail
    assert!(manager.add_connection("client3".to_string(), "session3".to_string(), tx3).await.is_err());
    
    assert_eq!(manager.connection_count().await, 2);
}

#[tokio::test]
async fn test_websocket_manager_cleanup_stale_connections() {
    let mut config = WebSocketConfig::default();
    config.connection_timeout_ms = 1; // Very short timeout for testing
    
    let manager = lumosai_core::agent::websocket::WebSocketManager::new(config);
    
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    
    // Add connection
    let _ = manager.add_connection(
        "stale_client".to_string(),
        "stale_session".to_string(),
        tx,
    ).await;
    
    assert_eq!(manager.connection_count().await, 1);
    
    // Wait for connection to become stale
    sleep(Duration::from_millis(10)).await;
    
    // Cleanup should remove stale connection
    manager.cleanup_stale_connections().await;
    assert_eq!(manager.connection_count().await, 0);
}

#[tokio::test]
async fn test_websocket_streaming_agent_creation() {
    let streaming_config = StreamingConfig::default();
    let websocket_config = WebSocketConfig::default();
    
    let wm_config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1024),
    };
    
    let agent_config = AgentConfig {
        name: "test_agent".to_string(),
        instructions: "Test agent".to_string(),
        working_memory: Some(wm_config),
        ..Default::default()
    };
    
    let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
    let agent = BasicAgent::new(agent_config, llm);
    
    let ws_agent = agent.into_websocket_streaming(streaming_config, websocket_config);
    
    // Should start with no connections
    assert_eq!(ws_agent.websocket_manager().connection_count().await, 0);
    assert_eq!(ws_agent.websocket_manager().session_count().await, 0);
}

#[tokio::test]
async fn test_websocket_streaming_execution() {
    let streaming_config = StreamingConfig {
        text_buffer_size: 5,
        emit_metadata: true,
        emit_memory_updates: false,
        text_delta_delay_ms: None, // No delay for testing
    };
    
    let websocket_config = WebSocketConfig {
        session_isolation: true,
        broadcast_events: false,
        ..Default::default()
    };
    
    let wm_config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1024),
    };
    
    let agent_config = AgentConfig {
        name: "test_agent".to_string(),
        instructions: "Test streaming agent".to_string(),
        working_memory: Some(wm_config),
        ..Default::default()
    };
    
    let llm = Arc::new(MockLlmProvider::new(vec![
        "Hello world this is a test response".to_string()
    ]));
    let agent = BasicAgent::new(agent_config, llm);
    
    let ws_agent = agent.into_websocket_streaming(streaming_config, websocket_config);
    
    // Set up a mock WebSocket connection
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let websocket_manager = ws_agent.websocket_manager();
    
    let _ = websocket_manager.add_connection(
        "test_client".to_string(),
        "test_session".to_string(),
        tx,
    ).await;
    
    // Start execution
    let messages = vec![user_message("Test message")];
    let options = AgentGenerateOptions {
        llm_options: LlmOptions::default(),
        max_steps: Some(1),
        tools_config: None,
    };
    
    let mut stream = ws_agent.execute_with_websocket(
        &messages,
        &options,
        "test_session",
    ).await;
    
    // Start a task to collect WebSocket messages
    let ws_messages_task = tokio::spawn(async move {
        let mut messages = Vec::new();
        
        // Collect messages for a short time
        let timeout = tokio::time::timeout(Duration::from_millis(500), async {
            while let Some(message) = rx.recv().await {
                messages.push(message);
                
                // Break on session end
                if matches!(message, WebSocketMessage::SessionEnd { .. }) {
                    break;
                }
            }
        });
        
        let _ = timeout.await;
        messages
    });
    
    // Consume the stream
    let mut events = Vec::new();
    
    let stream_timeout = tokio::time::timeout(Duration::from_millis(1000), async {
        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(event) => {
                    events.push(event);
                    
                    // Break on generation complete
                    if matches!(events.last(), Some(AgentEvent::GenerationComplete { .. })) {
                        break;
                    }
                },
                Err(_) => break,
            }
        }
    });
    
    let _ = stream_timeout.await;
    
    // Wait for WebSocket messages
    let ws_messages = ws_messages_task.await.unwrap();
    
    // Verify we got events
    assert!(!events.is_empty(), "Should have received at least one event");
    
    // Verify we got WebSocket messages
    assert!(!ws_messages.is_empty(), "Should have received at least one WebSocket message");
    
    // Check for session start and end messages
    let has_session_start = ws_messages.iter().any(|msg| {
        matches!(msg, WebSocketMessage::SessionStart { .. })
    });
    
    let has_session_end = ws_messages.iter().any(|msg| {
        matches!(msg, WebSocketMessage::SessionEnd { .. })
    });
    
    assert!(has_session_start, "Should have received session start message");
    assert!(has_session_end, "Should have received session end message");
}

#[tokio::test]
async fn test_websocket_manager_broadcast() {
    let config = WebSocketConfig::default();
    let manager = lumosai_core::agent::websocket::WebSocketManager::new(config);
    
    let (tx1, mut rx1) = tokio::sync::mpsc::unbounded_channel();
    let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();
    
    // Add two connections
    let _ = manager.add_connection("client1".to_string(), "session1".to_string(), tx1).await;
    let _ = manager.add_connection("client2".to_string(), "session2".to_string(), tx2).await;
    
    // Subscribe to broadcast
    let mut broadcast_rx = manager.subscribe();
    
    // Send a broadcast message
    let test_message = WebSocketMessage::Ping { timestamp: 123456789 };
    manager.broadcast(test_message.clone()).await;
    
    // Should receive the broadcast
    let received = tokio::time::timeout(
        Duration::from_millis(100),
        broadcast_rx.recv()
    ).await;
    
    assert!(received.is_ok());
    let received_message = received.unwrap().unwrap();
    
    match (&test_message, &received_message) {
        (WebSocketMessage::Ping { timestamp: t1 }, WebSocketMessage::Ping { timestamp: t2 }) => {
            assert_eq!(t1, t2);
        },
        _ => panic!("Message types don't match"),
    }
}

#[tokio::test]
async fn test_websocket_manager_send_to_session() {
    let config = WebSocketConfig::default();
    let manager = lumosai_core::agent::websocket::WebSocketManager::new(config);
    
    let (tx1, mut rx1) = tokio::sync::mpsc::unbounded_channel();
    let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();
    
    // Add connections to different sessions
    let _ = manager.add_connection("client1".to_string(), "session1".to_string(), tx1).await;
    let _ = manager.add_connection("client2".to_string(), "session2".to_string(), tx2).await;
    
    // Send message to specific session
    let test_message = WebSocketMessage::Ping { timestamp: 123456789 };
    let _ = manager.send_to_session("session1", test_message.clone()).await;
    
    // Client 1 should receive the message
    let received1 = tokio::time::timeout(
        Duration::from_millis(100),
        rx1.recv()
    ).await;
    
    assert!(received1.is_ok());
    
    // Client 2 should not receive the message (different session)
    let received2 = tokio::time::timeout(
        Duration::from_millis(50),
        rx2.recv()
    ).await;
    
    // This should timeout since no message should be sent to session2
    assert!(received2.is_err());
}

#[tokio::test]
async fn test_heartbeat_monitoring() {
    let mut config = WebSocketConfig::default();
    config.heartbeat_interval_ms = 50; // Very fast for testing
    config.connection_timeout_ms = 100;
    
    let streaming_config = StreamingConfig::default();
    
    let wm_config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1024),
    };
    
    let agent_config = AgentConfig {
        name: "test_agent".to_string(),
        instructions: "Test agent".to_string(),
        working_memory: Some(wm_config),
        ..Default::default()
    };
    
    let llm = Arc::new(MockLlmProvider::new(vec!["Test".to_string()]));
    let agent = BasicAgent::new(agent_config, llm);
    
    let ws_agent = agent.into_websocket_streaming(streaming_config, config);
    
    // Start heartbeat monitoring
    let heartbeat_task = ws_agent.start_heartbeat_monitoring();
    
    // Add a connection
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let _ = ws_agent.websocket_manager().add_connection(
        "test_client".to_string(),
        "test_session".to_string(),
        tx,
    ).await;
    
    // Should receive ping messages
    let ping_received = tokio::time::timeout(
        Duration::from_millis(200),
        async {
            while let Some(message) = rx.recv().await {
                if matches!(message, WebSocketMessage::Ping { .. }) {
                    return true;
                }
            }
            false
        }
    ).await;
    
    assert!(ping_received.is_ok() && ping_received.unwrap(), "Should have received ping message");
    
    // Cleanup
    heartbeat_task.abort();
}
