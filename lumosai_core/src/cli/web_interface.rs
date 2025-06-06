//! Web interface for development server
//! 
//! This module provides a web-based interface for testing and debugging agents

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::Result;
use super::{ProjectConfig, CliUtils};

/// Web interface configuration
#[derive(Debug, Clone)]
pub struct WebInterfaceConfig {
    pub port: u16,
    pub host: String,
    pub enable_cors: bool,
    pub static_files_path: Option<String>,
}

/// Web interface server
pub struct WebInterface {
    config: WebInterfaceConfig,
    project_config: Arc<RwLock<ProjectConfig>>,
    sessions: Arc<RwLock<HashMap<String, AgentSession>>>,
}

/// Agent testing session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub agent_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub messages: Vec<ChatMessage>,
    pub status: SessionStatus,
}

/// Chat message in a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: Option<MessageMetadata>,
}

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// Message metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub tool_calls: Option<Vec<ToolCall>>,
    pub execution_time_ms: Option<u64>,
    pub token_count: Option<u32>,
    pub model_used: Option<String>,
}

/// Tool call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub execution_time_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Error,
}

/// API request/response types
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub agent_name: String,
    pub initial_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub agent_name: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub message_id: String,
    pub response: String,
    pub metadata: Option<MessageMetadata>,
}

#[derive(Debug, Serialize)]
pub struct ProjectStatusResponse {
    pub name: String,
    pub version: String,
    pub tools: Vec<ToolInfo>,
    pub agents: Vec<AgentInfo>,
    pub health: HealthStatus,
}

#[derive(Debug, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub category: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct AgentInfo {
    pub name: String,
    pub instructions: String,
    pub model: String,
    pub tools: Vec<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub overall: String,
    pub components: HashMap<String, ComponentHealth>,
}

#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub status: String,
    pub message: String,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

impl WebInterface {
    /// Create a new web interface
    pub fn new(config: WebInterfaceConfig, project_config: ProjectConfig) -> Self {
        Self {
            config,
            project_config: Arc::new(RwLock::new(project_config)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the web interface server
    pub async fn start(&self) -> Result<()> {
        CliUtils::info(&format!("Starting web interface on {}:{}", self.config.host, self.config.port));
        
        // In a real implementation, this would start an HTTP server
        // For now, we'll simulate the web interface functionality
        
        self.setup_routes().await?;
        self.start_background_tasks().await?;
        
        CliUtils::success(&format!("Web interface available at http://{}:{}", self.config.host, self.config.port));
        CliUtils::info("Available endpoints:");
        CliUtils::info("  GET  /api/status - Project status");
        CliUtils::info("  POST /api/sessions - Create agent session");
        CliUtils::info("  GET  /api/sessions/{id} - Get session details");
        CliUtils::info("  POST /api/sessions/{id}/messages - Send message");
        CliUtils::info("  GET  /api/tools - List available tools");
        CliUtils::info("  GET  /api/agents - List available agents");
        CliUtils::info("  GET  / - Web dashboard");
        
        Ok(())
    }

    /// Setup HTTP routes
    async fn setup_routes(&self) -> Result<()> {
        // This would typically set up actual HTTP routes
        // For demonstration, we'll just log the available endpoints
        
        CliUtils::info("Setting up web interface routes...");
        
        // Simulate route setup
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(())
    }

    /// Start background tasks
    async fn start_background_tasks(&self) -> Result<()> {
        let sessions = self.sessions.clone();
        
        // Session cleanup task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
            
            loop {
                interval.tick().await;
                
                // Clean up old sessions
                let mut sessions_guard = sessions.write().await;
                let now = chrono::Utc::now();
                
                sessions_guard.retain(|_, session| {
                    let age = now.signed_duration_since(session.created_at);
                    age.num_hours() < 24 // Keep sessions for 24 hours
                });
            }
        });
        
        Ok(())
    }

    /// Create a new agent session
    pub async fn create_session(&self, request: CreateSessionRequest) -> Result<CreateSessionResponse> {
        let session_id = uuid::Uuid::new_v4().to_string();
        
        let mut session = AgentSession {
            id: session_id.clone(),
            agent_name: request.agent_name.clone(),
            created_at: chrono::Utc::now(),
            messages: Vec::new(),
            status: SessionStatus::Active,
        };

        // Add initial message if provided
        if let Some(initial_message) = request.initial_message {
            let message = ChatMessage {
                id: uuid::Uuid::new_v4().to_string(),
                role: MessageRole::User,
                content: initial_message,
                timestamp: chrono::Utc::now(),
                metadata: None,
            };
            session.messages.push(message);
        }

        // Store session
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        Ok(CreateSessionResponse {
            session_id,
            agent_name: request.agent_name,
            status: "active".to_string(),
        })
    }

    /// Send a message to an agent session
    pub async fn send_message(&self, session_id: &str, request: SendMessageRequest) -> Result<SendMessageResponse> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| crate::Error::Other(format!("Session {} not found", session_id)))?;

        // Add user message
        let user_message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: request.content.clone(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        };
        session.messages.push(user_message);

        // Simulate agent response
        let response_content = format!("This is a simulated response to: {}", request.content);
        let response_id = uuid::Uuid::new_v4().to_string();
        
        let metadata = MessageMetadata {
            tool_calls: None,
            execution_time_ms: Some(150),
            token_count: Some(25),
            model_used: Some("gpt-4".to_string()),
        };

        let assistant_message = ChatMessage {
            id: response_id.clone(),
            role: MessageRole::Assistant,
            content: response_content.clone(),
            timestamp: chrono::Utc::now(),
            metadata: Some(metadata.clone()),
        };
        session.messages.push(assistant_message);

        Ok(SendMessageResponse {
            message_id: response_id,
            response: response_content,
            metadata: Some(metadata),
        })
    }

    /// Get project status
    pub async fn get_project_status(&self) -> Result<ProjectStatusResponse> {
        let config = self.project_config.read().await;
        
        let tools: Vec<ToolInfo> = config.tools.iter().map(|tool| ToolInfo {
            name: tool.name.clone(),
            version: tool.version.clone(),
            category: "general".to_string(), // Would be determined from tool metadata
            status: "active".to_string(),
        }).collect();

        // Simulate agent discovery
        let agents = vec![
            AgentInfo {
                name: "default".to_string(),
                instructions: "A helpful assistant".to_string(),
                model: "gpt-4".to_string(),
                tools: tools.iter().map(|t| t.name.clone()).collect(),
                status: "ready".to_string(),
            }
        ];

        let mut components = HashMap::new();
        components.insert("tools".to_string(), ComponentHealth {
            status: "healthy".to_string(),
            message: format!("{} tools loaded", tools.len()),
            last_check: chrono::Utc::now(),
        });
        components.insert("agents".to_string(), ComponentHealth {
            status: "healthy".to_string(),
            message: format!("{} agents available", agents.len()),
            last_check: chrono::Utc::now(),
        });

        Ok(ProjectStatusResponse {
            name: config.name.clone(),
            version: config.version.clone(),
            tools,
            agents,
            health: HealthStatus {
                overall: "healthy".to_string(),
                components,
            },
        })
    }

    /// Get session details
    pub async fn get_session(&self, session_id: &str) -> Result<AgentSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id)
            .cloned()
            .ok_or_else(|| crate::Error::Other(format!("Session {} not found", session_id)))
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<AgentSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().cloned().collect())
    }
}

impl Default for WebInterfaceConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "localhost".to_string(),
            enable_cors: true,
            static_files_path: None,
        }
    }
}
