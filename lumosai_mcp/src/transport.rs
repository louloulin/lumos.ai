use async_trait::async_trait;
use std::process::{Stdio, Command};
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::sync::{Mutex, mpsc, broadcast};
use eventsource_stream::EventStream;
use futures::StreamExt;
use reqwest::Client;
use std::sync::Arc;
use tokio_util::codec::{FramedRead, LinesCodec};
use futures::stream::BoxStream;
use futures::TryStreamExt;

use crate::error::{MCPError, Result};
use crate::types::{MCPMessage, ServerParameters, StdioServerParameters, SSEServerParameters};

/// Trait representing a transport mechanism for MCP communication
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect to the MCP server
    async fn connect(&mut self) -> Result<()>;
    
    /// Disconnect from the MCP server
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Send a message to the server
    async fn send_message(&mut self, message: &MCPMessage) -> Result<()>;
    
    /// Receive a message from the server
    async fn receive_message(&mut self) -> Result<MCPMessage>;
    
    /// Returns a Stream of messages from the server
    fn message_stream(&self) -> Result<mpsc::Receiver<Result<MCPMessage>>>;
}

/// Transport implementation for stdio-based MCP servers
pub struct StdioTransport {
    params: StdioServerParameters,
    process: Option<Child>,
    stdin: Option<tokio::process::ChildStdin>,
    is_connected: bool,
    broadcast_tx: Arc<Mutex<Option<broadcast::Sender<Result<MCPMessage>>>>>,
}

impl StdioTransport {
    pub fn new(params: StdioServerParameters) -> Self {
        Self {
            params,
            process: None,
            stdin: None,
            is_connected: false,
            broadcast_tx: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn connect(&mut self) -> Result<()> {
        if self.is_connected {
            return Ok(());
        }
        
        // Create command with specified parameters
        let mut cmd = Command::new(&self.params.command);
        cmd.args(&self.params.args)
           .stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::inherit());
        
        // Add environment variables
        for (key, value) in &self.params.env {
            cmd.env(key, value);
        }
        
        // Spawn the process
        let mut child = tokio::process::Command::from(cmd)
            .spawn()
            .map_err(|e| MCPError::ConnectionError(format!("Failed to spawn process: {}", e)))?;
        
        // Get stdio handles
        let stdin = child.stdin.take()
            .ok_or_else(|| MCPError::ConnectionError("Failed to get stdin handle".to_string()))?;
        
        let stdout = child.stdout.take()
            .ok_or_else(|| MCPError::ConnectionError("Failed to get stdout handle".to_string()))?;
        
        // Create a lines stream from stdout
        let reader = BufReader::new(stdout);
        let lines_stream = FramedRead::new(reader, LinesCodec::new());
        
        // Create a broadcast channel for the messages
        let (tx, _rx) = broadcast::channel(100);
        
        // Store the sender
        {
            let mut tx_guard = self.broadcast_tx.lock().await;
            *tx_guard = Some(tx.clone());
        }
        
        // Spawn a task to read from the stream and send to the channel
        tokio::spawn(async move {
            let stream = lines_stream
                .map_err(|e| MCPError::ConnectionError(format!("Failed to read line: {}", e)))
                .and_then(|line| async move {
                    serde_json::from_str::<MCPMessage>(&line)
                        .map_err(|e| MCPError::DeserializationError(e.to_string()))
                });
            
            let mut pinned_stream = Box::pin(stream);
            
            while let Some(message_result) = pinned_stream.next().await {
                // Even if there are no subscribers, we still process the stream
                let _ = tx.send(message_result);
            }
        });
        
        // Store the process and IO handles
        self.process = Some(child);
        self.stdin = Some(stdin);
        self.is_connected = true;
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        if !self.is_connected {
            return Ok(());
        }
        
        // Clear broadcast sender
        {
            let mut tx_guard = self.broadcast_tx.lock().await;
            *tx_guard = None;
        }
        
        // Release stdin to ensure process can finish
        self.stdin = None;
        
        // Kill the process if it's still running
        if let Some(mut child) = self.process.take() {
            let _ = child.kill().await;
        }
        
        self.is_connected = false;
        
        Ok(())
    }
    
    async fn send_message(&mut self, message: &MCPMessage) -> Result<()> {
        if !self.is_connected {
            return Err(MCPError::ConnectionError("Not connected to server".to_string()));
        }
        
        if let Some(stdin) = &mut self.stdin {
            let message_json = serde_json::to_string(message)
                .map_err(|e| MCPError::DeserializationError(e.to_string()))?;
                
            stdin.write_all(message_json.as_bytes()).await
                .map_err(|e| MCPError::ConnectionError(format!("Failed to write to stdin: {}", e)))?;
            stdin.write_all(b"\n").await
                .map_err(|e| MCPError::ConnectionError(format!("Failed to write newline to stdin: {}", e)))?;
            stdin.flush().await
                .map_err(|e| MCPError::ConnectionError(format!("Failed to flush stdin: {}", e)))?;
                
            Ok(())
        } else {
            Err(MCPError::ConnectionError("Not connected to server".to_string()))
        }
    }
    
    async fn receive_message(&mut self) -> Result<MCPMessage> {
        if !self.is_connected {
            return Err(MCPError::ConnectionError("Not connected to server".to_string()));
        }
        
        let tx_guard = self.broadcast_tx.lock().await;
        if let Some(tx) = &*tx_guard {
            // Create a new receiver from the broadcast channel
            let mut rx = tx.subscribe();
            drop(tx_guard);
            
            // Wait for a message
            match rx.recv().await {
                Ok(result) => result,
                Err(_) => Err(MCPError::ConnectionError("Failed to receive message".to_string())),
            }
        } else {
            Err(MCPError::ConnectionError("Not connected to server".to_string()))
        }
    }
    
    fn message_stream(&self) -> Result<mpsc::Receiver<Result<MCPMessage>>> {
        if !self.is_connected {
            return Err(MCPError::ConnectionError("Not connected to server".to_string()));
        }
        
        // Create a new mpsc channel
        let (tx, rx) = mpsc::channel(100);
        
        // Create a clone of the broadcast sender
        let broadcast_tx = self.broadcast_tx.clone();
        
        // Spawn a task to forward messages from the broadcast to the mpsc channel
        tokio::spawn(async move {
            // Get a lock on the broadcast sender
            let guard = broadcast_tx.lock().await;
            if let Some(sender) = &*guard {
                // Create a new receiver from the broadcast channel
                let mut broadcast_rx = sender.subscribe();
                drop(guard);
                
                // Forward messages from the broadcast to the mpsc channel
                while let Ok(msg) = broadcast_rx.recv().await {
                    if tx.send(msg).await.is_err() {
                        break;
                    }
                }
            }
        });
        
        Ok(rx)
    }
}

/// Transport implementation for SSE-based MCP servers
pub struct SSETransport {
    params: SSEServerParameters,
    client: Client,
    is_connected: bool,
    broadcast_tx: Arc<Mutex<Option<broadcast::Sender<Result<MCPMessage>>>>>,
    outbound_messages: Arc<Mutex<Vec<MCPMessage>>>,
}

impl SSETransport {
    pub fn new(params: SSEServerParameters) -> Self {
        Self {
            params,
            client: Client::new(),
            is_connected: false,
            broadcast_tx: Arc::new(Mutex::new(None)),
            outbound_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl Transport for SSETransport {
    async fn connect(&mut self) -> Result<()> {
        if self.is_connected {
            return Ok(());
        }
        
        // Prepare headers for SSE connection
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("text/event-stream"),
        );
        
        // Add custom headers from request_init if available
        if let Some(request_init) = &self.params.request_init {
            for (key, value) in request_init {
                if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                    if let Ok(header_value) = reqwest::header::HeaderValue::from_str(value) {
                        headers.insert(header_name, header_value);
                    }
                }
            }
        }
        
        // Make the request to the SSE endpoint
        let response = self.client.get(self.params.url.as_str())
            .headers(headers)
            .send()
            .await
            .map_err(|e| MCPError::HttpError(e.to_string()))?;
            
        if !response.status().is_success() {
            return Err(MCPError::ConnectionError(format!(
                "Server returned non-success status: {}", 
                response.status()
            )));
        }
        
        // Create a broadcast channel for the messages
        let (tx, _rx) = broadcast::channel(100);
        
        // Store the sender
        {
            let mut tx_guard = self.broadcast_tx.lock().await;
            *tx_guard = Some(tx.clone());
        }
        
        // Get the response bytes stream and create an event stream
        let bytes_stream = response.bytes_stream();
        let event_stream = EventStream::new(bytes_stream);
        
        // Create a stream that maps events to MCPMessages
        let message_stream = event_stream
            .filter_map(|event_result| async move {
                match event_result {
                    Ok(event) => {
                        if event.data.is_empty() {
                            return None;
                        }
                        
                        match serde_json::from_str::<MCPMessage>(&event.data) {
                            Ok(message) => Some(Ok(message)),
                            Err(e) => Some(Err(MCPError::DeserializationError(e.to_string()))),
                        }
                    },
                    Err(e) => Some(Err(MCPError::Other(format!("SSE error: {}", e)))),
                }
            });
        
        // Pin the stream and forward messages to the channel
        let mut pinned_stream = Box::pin(message_stream) as BoxStream<'static, Result<MCPMessage>>;
        
        // Spawn a task to read from the event stream and send to the channel
        tokio::spawn(async move {
            while let Some(message_result) = pinned_stream.next().await {
                // Even if there are no subscribers, we still process the stream
                let _ = tx.send(message_result);
            }
        });
        
        self.is_connected = true;
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        if !self.is_connected {
            return Ok(());
        }
        
        // Clear broadcast sender
        {
            let mut tx_guard = self.broadcast_tx.lock().await;
            *tx_guard = None;
        }
        
        self.is_connected = false;
        
        Ok(())
    }
    
    async fn send_message(&mut self, message: &MCPMessage) -> Result<()> {
        if !self.is_connected {
            return Err(MCPError::ConnectionError("Not connected to server".to_string()));
        }
        
        // For SSE, we need to queue outbound messages and handle them separately
        // as SSE is a one-way communication protocol from server to client
        let mut messages = self.outbound_messages.lock().await;
        messages.push(message.clone());
        
        // In a real implementation, we would need to use a separate HTTP request
        // to send messages to the server
        
        Ok(())
    }
    
    async fn receive_message(&mut self) -> Result<MCPMessage> {
        if !self.is_connected {
            return Err(MCPError::ConnectionError("Not connected to server".to_string()));
        }
        
        let tx_guard = self.broadcast_tx.lock().await;
        if let Some(tx) = &*tx_guard {
            // Create a new receiver from the broadcast channel
            let mut rx = tx.subscribe();
            drop(tx_guard);
            
            // Wait for a message
            match rx.recv().await {
                Ok(result) => result,
                Err(_) => Err(MCPError::ConnectionError("Failed to receive message".to_string())),
            }
        } else {
            Err(MCPError::ConnectionError("Not connected to server".to_string()))
        }
    }
    
    fn message_stream(&self) -> Result<mpsc::Receiver<Result<MCPMessage>>> {
        if !self.is_connected {
            return Err(MCPError::ConnectionError("Not connected to server".to_string()));
        }
        
        // Create a new mpsc channel
        let (tx, rx) = mpsc::channel(100);
        
        // Create a clone of the broadcast sender
        let broadcast_tx = self.broadcast_tx.clone();
        
        // Spawn a task to forward messages from the broadcast to the mpsc channel
        tokio::spawn(async move {
            // Get a lock on the broadcast sender
            let guard = broadcast_tx.lock().await;
            if let Some(sender) = &*guard {
                // Create a new receiver from the broadcast channel
                let mut broadcast_rx = sender.subscribe();
                drop(guard);
                
                // Forward messages from the broadcast to the mpsc channel
                while let Ok(msg) = broadcast_rx.recv().await {
                    if tx.send(msg).await.is_err() {
                        break;
                    }
                }
            }
        });
        
        Ok(rx)
    }
}

/// Create a Transport instance based on server parameters
pub fn create_transport(params: ServerParameters) -> Box<dyn Transport> {
    match params {
        ServerParameters::Stdio(stdio_params) => Box::new(StdioTransport::new(stdio_params)),
        ServerParameters::SSE(sse_params) => Box::new(SSETransport::new(sse_params)),
    }
} 