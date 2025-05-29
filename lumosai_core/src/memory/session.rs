//! Session Management module for managing agent conversations and context
//! 
//! This module provides session-based conversation management, building on top of Memory Threads
//! to provide higher-level abstractions for conversation flow and context management.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::llm::Message;
use crate::Result;
use crate::error::LumosError;
use super::thread::{MemoryThread, MemoryThreadManager, MemoryThreadStorage, CreateThreadParams, UpdateThreadParams};
use super::MemoryConfig;

/// Session state for tracking conversation progress
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    /// Session is active and accepting new messages
    Active,
    /// Session is paused (can be resumed)
    Paused,
    /// Session has ended normally
    Ended,
    /// Session was terminated due to error or timeout
    Terminated,
    /// Session is being archived
    Archived,
}

/// Session context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Current topic or focus of the conversation
    pub current_topic: Option<String>,
    /// Important facts extracted from conversation
    pub facts: Vec<String>,
    /// Open questions that need follow-up
    pub open_questions: Vec<String>,
    /// Action items or tasks identified
    pub action_items: Vec<ActionItem>,
    /// User preferences discovered during session
    pub preferences: HashMap<String, Value>,
    /// Session tags for categorization
    pub tags: Vec<String>,
}

/// Action item extracted from conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    /// Unique identifier for the action item
    pub id: String,
    /// Description of the action
    pub description: String,
    /// Whether the action is completed
    #[serde(default)]
    pub completed: bool,
    /// Due date if specified
    pub due_date: Option<DateTime<Utc>>,
    /// Priority level
    pub priority: Option<Priority>,
    /// Who is responsible for the action
    pub assignee: Option<String>,
}

/// Priority level for action items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum session duration in seconds
    pub max_duration_seconds: Option<u64>,
    /// Session timeout in seconds (auto-pause if inactive)
    pub timeout_seconds: Option<u64>,
    /// Maximum number of messages per session
    pub max_messages: Option<usize>,
    /// Whether to auto-archive completed sessions
    #[serde(default)]
    pub auto_archive: bool,
    /// Whether to extract action items automatically
    #[serde(default = "default_true")]
    pub extract_action_items: bool,
    /// Whether to track topics automatically
    #[serde(default = "default_true")]
    pub track_topics: bool,
    /// Memory configuration for the session
    pub memory_config: Option<MemoryConfig>,
}

/// Session management wrapper around memory threads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: String,
    /// Associated memory thread
    pub thread_id: String,
    /// Session title
    pub title: String,
    /// Agent ID managing this session
    pub agent_id: Option<String>,
    /// Resource ID (user) this session belongs to
    pub resource_id: Option<String>,
    /// Current session state
    pub state: SessionState,
    /// Session context information
    pub context: SessionContext,
    /// Session configuration
    pub config: SessionConfig,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session was last updated
    pub updated_at: DateTime<Utc>,
    /// When the session started (first message)
    pub started_at: Option<DateTime<Utc>>,
    /// When the session ended
    pub ended_at: Option<DateTime<Utc>>,
    /// Session metadata
    pub metadata: HashMap<String, Value>,
}

/// Parameters for creating a new session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionParams {
    /// Optional session ID
    pub id: Option<String>,
    /// Session title
    pub title: String,
    /// Agent ID
    pub agent_id: Option<String>,
    /// Resource ID (user ID)
    pub resource_id: Option<String>,
    /// Initial session configuration
    pub config: Option<SessionConfig>,
    /// Initial metadata
    pub metadata: Option<HashMap<String, Value>>,
}

/// Parameters for updating a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSessionParams {
    /// New title
    pub title: Option<String>,
    /// New state
    pub state: Option<SessionState>,
    /// Context updates
    pub context: Option<SessionContext>,
    /// Configuration updates
    pub config: Option<SessionConfig>,
    /// Metadata updates
    pub metadata: Option<HashMap<String, Value>>,
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    /// Total duration of session in seconds
    pub duration_seconds: u64,
    /// Number of messages exchanged
    pub message_count: usize,
    /// Number of user messages
    pub user_message_count: usize,
    /// Number of assistant messages
    pub assistant_message_count: usize,
    /// Number of action items created
    pub action_item_count: usize,
    /// Number of topics discussed
    pub topic_count: usize,
    /// Average response time in milliseconds
    pub avg_response_time_ms: Option<f64>,
}

fn default_true() -> bool {
    true
}

impl Default for SessionState {
    fn default() -> Self {
        Self::Active
    }
}

impl Default for SessionContext {
    fn default() -> Self {
        Self {
            current_topic: None,
            facts: Vec::new(),
            open_questions: Vec::new(),
            action_items: Vec::new(),
            preferences: HashMap::new(),
            tags: Vec::new(),
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_duration_seconds: Some(3600), // 1 hour
            timeout_seconds: Some(1800),     // 30 minutes
            max_messages: Some(100),
            auto_archive: false,
            extract_action_items: true,
            track_topics: true,
            memory_config: None,
        }
    }
}

impl Session {
    /// Create a new session
    pub fn new(params: CreateSessionParams) -> Self {
        let now = Utc::now();
        let session_id = params.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let thread_id = format!("thread_{}", session_id);

        Self {
            id: session_id,
            thread_id,
            title: params.title,
            agent_id: params.agent_id,
            resource_id: params.resource_id,
            state: SessionState::Active,
            context: SessionContext::default(),
            config: params.config.unwrap_or_default(),
            created_at: now,
            updated_at: now,
            started_at: None,
            ended_at: None,
            metadata: params.metadata.unwrap_or_default(),
        }
    }

    /// Update session state
    pub fn update(&mut self, params: UpdateSessionParams) -> Result<()> {
        if let Some(title) = params.title {
            self.title = title;
        }

        if let Some(state) = params.state {
            self.set_state(state)?;
        }

        if let Some(context) = params.context {
            self.context = context;
        }

        if let Some(config) = params.config {
            self.config = config;
        }

        if let Some(metadata) = params.metadata {
            self.metadata.extend(metadata);
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Set session state with validation
    pub fn set_state(&mut self, new_state: SessionState) -> Result<()> {
        match (&self.state, &new_state) {
            // Valid transitions
            (SessionState::Active, SessionState::Paused) => {},
            (SessionState::Active, SessionState::Ended) => {
                self.ended_at = Some(Utc::now());
            },
            (SessionState::Active, SessionState::Terminated) => {
                self.ended_at = Some(Utc::now());
            },
            (SessionState::Paused, SessionState::Active) => {},
            (SessionState::Paused, SessionState::Ended) => {
                self.ended_at = Some(Utc::now());
            },
            (SessionState::Paused, SessionState::Terminated) => {
                self.ended_at = Some(Utc::now());
            },
            (SessionState::Ended, SessionState::Archived) => {},
            (SessionState::Terminated, SessionState::Archived) => {},
            // Invalid transitions
            _ => {
                return Err(LumosError::InvalidOperation(format!(
                    "Cannot transition from {:?} to {:?}",
                    self.state, new_state
                )));
            }
        }

        self.state = new_state;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Add an action item to the session
    pub fn add_action_item(&mut self, description: String, priority: Option<Priority>) -> String {
        let action_item = ActionItem {
            id: Uuid::new_v4().to_string(),
            description,
            completed: false,
            due_date: None,
            priority,
            assignee: None,
        };

        let id = action_item.id.clone();
        self.context.action_items.push(action_item);
        self.updated_at = Utc::now();
        id
    }

    /// Mark an action item as completed
    pub fn complete_action_item(&mut self, action_id: &str) -> Result<()> {
        if let Some(action) = self.context.action_items.iter_mut().find(|a| a.id == action_id) {
            action.completed = true;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(LumosError::NotFound(format!(
                "Action item {} not found",
                action_id
            )))
        }
    }

    /// Add a fact to the session context
    pub fn add_fact(&mut self, fact: String) {
        if !self.context.facts.contains(&fact) {
            self.context.facts.push(fact);
            self.updated_at = Utc::now();
        }
    }

    /// Set current topic
    pub fn set_topic(&mut self, topic: String) {
        self.context.current_topic = Some(topic);
        self.updated_at = Utc::now();
    }

    /// Add a tag to the session
    pub fn add_tag(&mut self, tag: String) {
        if !self.context.tags.contains(&tag) {
            self.context.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Check if session is active and accepting messages
    pub fn is_active(&self) -> bool {
        matches!(self.state, SessionState::Active)
    }

    /// Check if session has been started (has messages)
    pub fn is_started(&self) -> bool {
        self.started_at.is_some()
    }

    /// Check if session has ended
    pub fn is_ended(&self) -> bool {
        matches!(self.state, SessionState::Ended | SessionState::Terminated | SessionState::Archived)
    }

    /// Mark session as started
    pub fn mark_started(&mut self) {
        if self.started_at.is_none() {
            self.started_at = Some(Utc::now());
            self.updated_at = Utc::now();
        }
    }

    /// Calculate session duration
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.ended_at) {
            (Some(start), Some(end)) => Some(end - start),
            (Some(start), None) => Some(Utc::now() - start),
            _ => None,
        }
    }
}

/// Session manager for high-level session operations
#[derive(Debug)]
pub struct SessionManager<S: MemoryThreadStorage> {
    thread_manager: MemoryThreadManager<S>,
    sessions: HashMap<String, Session>, // In-memory cache
}

impl<S: MemoryThreadStorage> SessionManager<S> {
    /// Create a new session manager
    pub fn new(storage: S) -> Self {
        Self {
            thread_manager: MemoryThreadManager::new(storage),
            sessions: HashMap::new(),
        }
    }

    /// Create a new session with associated memory thread
    pub async fn create_session(&mut self, params: CreateSessionParams) -> Result<Session> {
        let session = Session::new(params.clone());

        // Create the associated memory thread
        let thread_params = CreateThreadParams {
            id: Some(session.thread_id.clone()),
            title: session.title.clone(),
            agent_id: session.agent_id.clone(),
            resource_id: session.resource_id.clone(),
            metadata: Some({
                let mut metadata = HashMap::new();
                metadata.insert("session_id".to_string(), serde_json::Value::String(session.id.clone()));
                metadata.insert("session_type".to_string(), serde_json::Value::String("conversation".to_string()));
                metadata
            }),
        };

        self.thread_manager.create_thread(thread_params).await?;

        // Cache the session
        self.sessions.insert(session.id.clone(), session.clone());

        Ok(session)
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
        // Check cache first
        if let Some(session) = self.sessions.get(session_id) {
            return Ok(Some(session.clone()));
        }

        // TODO: Load from persistent storage if not in cache
        // For now, return None if not in cache
        Ok(None)
    }

    /// Update a session
    pub async fn update_session(&mut self, session_id: &str, params: UpdateSessionParams) -> Result<Session> {
        let mut session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| LumosError::NotFound(format!("Session {} not found", session_id)))?;

        session.update(params)?;

        // Update cache
        self.sessions.insert(session_id.to_string(), session.clone());

        // TODO: Persist to storage

        Ok(session)
    }

    /// Delete a session and its associated thread
    pub async fn delete_session(&mut self, session_id: &str, resource_id: Option<&str>) -> Result<()> {
        if let Some(session) = self.get_session(session_id).await? {
            // Delete the associated memory thread
            self.thread_manager.delete_thread(&session.thread_id, resource_id).await?;
            
            // Remove from cache
            self.sessions.remove(session_id);
        }

        Ok(())
    }

    /// Add a message to a session
    pub async fn add_message(
        &mut self,
        session_id: &str,
        message: &Message,
        resource_id: Option<&str>,
    ) -> Result<()> {
        let mut session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| LumosError::NotFound(format!("Session {} not found", session_id)))?;

        if !session.is_active() {
            return Err(LumosError::InvalidOperation(
                "Cannot add message to inactive session".to_string(),
            ));
        }

        // Mark session as started if this is the first message
        if !session.is_started() {
            session.mark_started();
            self.sessions.insert(session_id.to_string(), session.clone());
        }

        // Add message to the associated thread
        self.thread_manager
            .add_message(&session.thread_id, message, resource_id)
            .await?;

        Ok(())
    }

    /// Get messages from a session
    pub async fn get_messages(
        &self,
        session_id: &str,
        limit: Option<usize>,
        resource_id: Option<&str>,
    ) -> Result<Vec<Message>> {
        let session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| LumosError::NotFound(format!("Session {} not found", session_id)))?;

        let params = super::thread::GetMessagesParams {
            limit,
            cursor: None,
            filter: None,
            include_content: true,
            reverse_order: false,
        };

        self.thread_manager
            .get_messages(&session.thread_id, &params, resource_id)
            .await
    }

    /// Pause a session
    pub async fn pause_session(&mut self, session_id: &str) -> Result<Session> {
        self.update_session(session_id, UpdateSessionParams {
            title: None,
            state: Some(SessionState::Paused),
            context: None,
            config: None,
            metadata: None,
        }).await
    }

    /// Resume a paused session
    pub async fn resume_session(&mut self, session_id: &str) -> Result<Session> {
        self.update_session(session_id, UpdateSessionParams {
            title: None,
            state: Some(SessionState::Active),
            context: None,
            config: None,
            metadata: None,
        }).await
    }

    /// End a session
    pub async fn end_session(&mut self, session_id: &str) -> Result<Session> {
        self.update_session(session_id, UpdateSessionParams {
            title: None,
            state: Some(SessionState::Ended),
            context: None,
            config: None,
            metadata: None,
        }).await
    }

    /// Get session statistics
    pub async fn get_session_stats(&self, session_id: &str) -> Result<SessionStats> {
        let session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| LumosError::NotFound(format!("Session {} not found", session_id)))?;

        let thread_stats = self
            .thread_manager
            .get_thread_stats(&session.thread_id, session.resource_id.as_deref())
            .await?;

        let duration_seconds = session
            .duration()
            .map(|d| d.num_seconds() as u64)
            .unwrap_or(0);

        Ok(SessionStats {
            duration_seconds,
            message_count: thread_stats.message_count,
            user_message_count: thread_stats.user_message_count,
            assistant_message_count: thread_stats.assistant_message_count,
            action_item_count: session.context.action_items.len(),
            topic_count: if session.context.current_topic.is_some() { 1 } else { 0 } + session.context.tags.len(),
            avg_response_time_ms: None, // TODO: Calculate from message timestamps
        })
    }

    /// List active sessions for a resource
    pub async fn list_active_sessions(&self, resource_id: &str) -> Result<Vec<Session>> {
        Ok(self
            .sessions
            .values()
            .filter(|session| {
                session.resource_id.as_ref().map_or(false, |rid| rid == resource_id)
                    && session.is_active()
            })
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let params = CreateSessionParams {
            id: Some("test-session".to_string()),
            title: "Test Session".to_string(),
            agent_id: Some("agent-1".to_string()),
            resource_id: Some("user-123".to_string()),
            config: None,
            metadata: None,
        };

        let session = Session::new(params);
        assert_eq!(session.id, "test-session");
        assert_eq!(session.title, "Test Session");
        assert_eq!(session.state, SessionState::Active);
        assert!(!session.is_started());
        assert!(session.is_active());
    }

    #[test]
    fn test_session_state_transitions() {
        let params = CreateSessionParams {
            id: None,
            title: "Test Session".to_string(),
            agent_id: None,
            resource_id: None,
            config: None,
            metadata: None,
        };

        let mut session = Session::new(params);

        // Valid transitions
        assert!(session.set_state(SessionState::Paused).is_ok());
        assert!(session.set_state(SessionState::Active).is_ok());
        assert!(session.set_state(SessionState::Ended).is_ok());
        assert!(session.set_state(SessionState::Archived).is_ok());

        // Invalid transition
        session.state = SessionState::Active;
        assert!(session.set_state(SessionState::Archived).is_err());
    }

    #[test]
    fn test_action_items() {
        let params = CreateSessionParams {
            id: None,
            title: "Test Session".to_string(),
            agent_id: None,
            resource_id: None,
            config: None,
            metadata: None,
        };

        let mut session = Session::new(params);

        let action_id = session.add_action_item(
            "Complete task".to_string(),
            Some(Priority::High),
        );

        assert_eq!(session.context.action_items.len(), 1);
        assert_eq!(session.context.action_items[0].description, "Complete task");
        assert_eq!(session.context.action_items[0].priority, Some(Priority::High));
        assert!(!session.context.action_items[0].completed);

        assert!(session.complete_action_item(&action_id).is_ok());
        assert!(session.context.action_items[0].completed);
    }

    #[test]
    fn test_session_context() {
        let params = CreateSessionParams {
            id: None,
            title: "Test Session".to_string(),
            agent_id: None,
            resource_id: None,
            config: None,
            metadata: None,
        };

        let mut session = Session::new(params);

        session.add_fact("User prefers email communication".to_string());
        session.add_fact("User is in PST timezone".to_string());
        session.set_topic("Project planning".to_string());
        session.add_tag("important".to_string());
        session.add_tag("follow-up".to_string());

        assert_eq!(session.context.facts.len(), 2);
        assert_eq!(session.context.current_topic, Some("Project planning".to_string()));
        assert_eq!(session.context.tags.len(), 2);
        assert!(session.context.tags.contains(&"important".to_string()));
    }
}
