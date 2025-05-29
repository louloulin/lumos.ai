//! WebSocket Streaming Demo
//!
//! This example demonstrates the new WebSocket streaming capabilities
//! for Lumosai agents, showing real-time bidirectional communication.

use std::sync::Arc;
use futures::StreamExt;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use lumosai_core::agent::{
    AgentConfig, BasicAgent, AgentGenerateOptions,
    StreamingConfig, WebSocketConfig, 
    IntoWebSocketStreaming, WebSocketMessage
};
use lumosai_core::llm::{MockLlmProvider, LlmOptions};
use lumosai_core::memory::WorkingMemoryConfig;
use lumosai_core::agent::message_utils::user_message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 WebSocket Streaming Demo Starting...");
    
    // Configure streaming behavior
    let streaming_config = StreamingConfig {
        text_buffer_size: 5, // Stream in 5-character chunks for demo
        emit_metadata: true,
        emit_memory_updates: true,
        text_delta_delay_ms: Some(100), // 100ms delay between chunks for demo
    };
    
    // Configure WebSocket behavior
    let websocket_config = WebSocketConfig {
        max_connections: 100,
        heartbeat_interval_ms: 10000, // 10 seconds for demo
        connection_timeout_ms: 30000, // 30 seconds for demo
        message_buffer_size: 500,
        broadcast_events: true,
        session_isolation: true,
    };
    
    // Create working memory config
    let wm_config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1024),
    };
    
    // Create agent config
    let agent_config = AgentConfig {
        name: "websocket_demo_agent".to_string(),
        instructions: "You are a helpful AI assistant that demonstrates WebSocket streaming capabilities. Provide detailed, engaging responses.".to_string(),
        working_memory: Some(wm_config),
        ..Default::default()
    };
    
    // Create mock LLM with realistic response
    let mock_responses = vec![
        "Hello! I'm excited to demonstrate WebSocket streaming capabilities. ".to_string(),
        "This is a real-time conversation where you can see my thoughts unfold as I generate them. ".to_string(),
        "Each chunk of text is being streamed live through WebSocket connections, ".to_string(),
        "allowing for truly interactive AI experiences. ".to_string(),
        "You could easily integrate this into web applications, chat interfaces, or any real-time system!".to_string(),
    ];
    
    let llm = Arc::new(MockLlmProvider::new(mock_responses));
    let agent = BasicAgent::new(agent_config, llm);
    
    // Create WebSocket streaming agent
    let ws_agent = agent.into_websocket_streaming(streaming_config, websocket_config);
    
    println!("✅ WebSocket Streaming Agent Created");
    
    // Get WebSocket manager for connection simulation
    let websocket_manager = ws_agent.websocket_manager();
    
    println!("📊 Initial Stats:");
    println!("   Connections: {}", websocket_manager.connection_count().await);
    println!("   Sessions: {}", websocket_manager.session_count().await);
    
    // Simulate WebSocket connections
    println!("\n🔌 Simulating WebSocket Connections...");
    
    let (tx1, mut rx1) = tokio::sync::mpsc::unbounded_channel();
    let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();
    
    // Add two simulated connections
    websocket_manager.add_connection(
        "client_1".to_string(),
        "session_demo".to_string(),
        tx1,
    ).await?;
    
    websocket_manager.add_connection(
        "client_2".to_string(),
        "session_demo".to_string(),
        tx2,
    ).await?;
    
    println!("✅ Added 2 simulated WebSocket connections");
    println!("📊 Updated Stats:");
    println!("   Connections: {}", websocket_manager.connection_count().await);
    println!("   Sessions: {}", websocket_manager.session_count().await);
    
    // Start heartbeat monitoring
    let _heartbeat_task = ws_agent.start_heartbeat_monitoring();
    println!("💓 Heartbeat monitoring started");
    
    // Create message for the agent
    let messages = vec![user_message("Hello! Can you demonstrate your WebSocket streaming capabilities?")];
    
    let options = AgentGenerateOptions {
        llm_options: LlmOptions::default(),
        max_steps: Some(3),
        tools_config: None,
    };
    
    // Create background tasks to listen to WebSocket messages
    let websocket_manager_clone1 = websocket_manager.clone();
    let websocket_manager_clone2 = websocket_manager.clone();
    
    let client1_task = tokio::spawn(async move {
        println!("\n🔄 Client 1 WebSocket Message Stream:");
        while let Some(message) = rx1.recv().await {
            match message {
                WebSocketMessage::Connected { client_id, session_id, timestamp } => {
                    println!("  📱 Client 1 Connected: {} in session {} at {}", client_id, session_id, timestamp);
                },
                WebSocketMessage::SessionStart { session_id, timestamp } => {
                    println!("  🎬 Client 1 Session Started: {} at {}", session_id, timestamp);
                },
                WebSocketMessage::AgentEvent { event, session_id, timestamp } => {
                    println!("  🤖 Client 1 Agent Event in {}: {:?} at {}", session_id, event, timestamp);
                },
                WebSocketMessage::SessionEnd { session_id, timestamp } => {
                    println!("  🏁 Client 1 Session Ended: {} at {}", session_id, timestamp);
                    break;
                },
                WebSocketMessage::Ping { timestamp } => {
                    println!("  💓 Client 1 Ping at {}", timestamp);
                    // Update ping time
                    websocket_manager_clone1.update_ping("client_1").await;
                },
                _ => {
                    println!("  📨 Client 1 Other message: {:?}", message);
                }
            }
        }
        println!("  🔚 Client 1 WebSocket stream ended");
    });
    
    let client2_task = tokio::spawn(async move {
        println!("\n🔄 Client 2 WebSocket Message Stream:");
        while let Some(message) = rx2.recv().await {
            match message {
                WebSocketMessage::Connected { client_id, session_id, timestamp } => {
                    println!("  📱 Client 2 Connected: {} in session {} at {}", client_id, session_id, timestamp);
                },
                WebSocketMessage::SessionStart { session_id, timestamp } => {
                    println!("  🎬 Client 2 Session Started: {} at {}", session_id, timestamp);
                },
                WebSocketMessage::AgentEvent { event, session_id, timestamp } => {
                    println!("  🤖 Client 2 Agent Event in {}: {:?} at {}", session_id, event, timestamp);
                },
                WebSocketMessage::SessionEnd { session_id, timestamp } => {
                    println!("  🏁 Client 2 Session Ended: {} at {}", session_id, timestamp);
                    break;
                },
                WebSocketMessage::Ping { timestamp } => {
                    println!("  💓 Client 2 Ping at {}", timestamp);
                    // Update ping time
                    websocket_manager_clone2.update_ping("client_2").await;
                },
                _ => {
                    println!("  📨 Client 2 Other message: {:?}", message);
                }
            }
        }
        println!("  🔚 Client 2 WebSocket stream ended");
    });
    
    // Execute streaming with WebSocket broadcasting
    println!("\n🎯 Starting WebSocket Streaming Execution...");
    
    let mut stream = ws_agent.execute_with_websocket(
        &messages,
        &options,
        "session_demo",
    ).await;
    
    println!("\n🔥 Agent Response Stream:");
    let mut response_text = String::new();
    
    while let Some(event_result) = stream.next().await {
        match event_result {
            Ok(event) => {
                match &event {
                    lumosai_core::agent::AgentEvent::TextDelta { delta, step_id } => {
                        print!("{}", delta);
                        response_text.push_str(delta);
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    },
                    lumosai_core::agent::AgentEvent::GenerationComplete { final_response, total_steps } => {
                        println!("\n\n✅ Generation Complete!");
                        println!("   Final Response: {}", final_response);
                        println!("   Total Steps: {}", total_steps);
                    },
                    lumosai_core::agent::AgentEvent::Metadata { key, value } => {
                        println!("\n📋 Metadata: {} = {:?}", key, value);
                    },
                    _ => {
                        println!("\n🔔 Event: {:?}", event);
                    }
                }
            },
            Err(e) => {
                println!("\n❌ Error: {}", e);
                break;
            }
        }
    }
    
    println!("\n\n📊 Final Stats:");
    println!("   Connections: {}", websocket_manager.connection_count().await);
    println!("   Sessions: {}", websocket_manager.session_count().await);
    
    // Wait a bit for WebSocket messages to be processed
    sleep(Duration::from_millis(500)).await;
    
    // Clean up connections
    websocket_manager.remove_connection("client_1").await;
    websocket_manager.remove_connection("client_2").await;
    
    println!("\n🧹 Cleaned up connections");
    println!("📊 Final Stats:");
    println!("   Connections: {}", websocket_manager.connection_count().await);
    println!("   Sessions: {}", websocket_manager.session_count().await);
    
    // Wait for client tasks to complete
    let _ = tokio::join!(client1_task, client2_task);
    
    println!("\n🎉 WebSocket Streaming Demo Complete!");
    println!("\n📝 Summary:");
    println!("   ✅ Created WebSocket streaming agent");
    println!("   ✅ Simulated multiple client connections");
    println!("   ✅ Demonstrated real-time event broadcasting");
    println!("   ✅ Showed session management");
    println!("   ✅ Implemented heartbeat monitoring");
    println!("   ✅ Streamed agent responses with text deltas");
    println!("   ✅ Managed connection lifecycle");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_websocket_streaming_demo() {
        // This test verifies that the demo setup works correctly
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
        
        assert_eq!(ws_agent.websocket_manager().connection_count().await, 0);
    }
}
