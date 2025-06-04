//! Debug tools and utilities inspired by Mastra's debugging capabilities
//! 
//! This module provides comprehensive debugging tools for agents, tools, and workflows

// pub mod inspector;
// pub mod tracer;
// pub mod profiler;
// pub mod diagnostics;

// use crate::agent::BasicAgent;
// use crate::tool::Tool;
use crate::llm::Message;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Debug session for tracking execution
#[derive(Debug, Clone)]
pub struct DebugSession {
    pub id: String,
    pub started_at: Instant,
    pub events: Vec<DebugEvent>,
    pub metadata: HashMap<String, Value>,
}

/// Debug event types
#[derive(Debug, Clone)]
pub enum DebugEvent {
    AgentCreated {
        agent_name: String,
        timestamp: Instant,
        config: Value,
    },
    ToolExecuted {
        tool_name: String,
        timestamp: Instant,
        duration: Duration,
        input: Value,
        output: Result<Value, String>,
    },
    MessageProcessed {
        timestamp: Instant,
        duration: Duration,
        input_messages: Vec<Message>,
        output_message: String,
        token_usage: Option<TokenUsage>,
    },
    ErrorOccurred {
        timestamp: Instant,
        error: String,
        context: HashMap<String, Value>,
    },
    MemoryAccessed {
        timestamp: Instant,
        operation: String,
        key: Option<String>,
        value: Option<Value>,
    },
}

/// Token usage information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl DebugSession {
    /// Create a new debug session
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            started_at: Instant::now(),
            events: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add an event to the session
    pub fn add_event(&mut self, event: DebugEvent) {
        self.events.push(event);
    }

    /// Add metadata
    pub fn add_metadata<K: Into<String>, V: Into<Value>>(&mut self, key: K, value: V) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Get session duration
    pub fn duration(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Get events by type
    pub fn get_events_by_type<F>(&self, filter: F) -> Vec<&DebugEvent>
    where
        F: Fn(&DebugEvent) -> bool,
    {
        self.events.iter().filter(|e| filter(e)).collect()
    }

    /// Generate summary report
    pub fn generate_summary(&self) -> DebugSummary {
        let mut summary = DebugSummary::new(self.id.clone(), self.duration());

        for event in &self.events {
            match event {
                DebugEvent::ToolExecuted { tool_name, duration, output, .. } => {
                    summary.tool_executions += 1;
                    summary.total_tool_time += *duration;
                    if output.is_err() {
                        summary.tool_errors += 1;
                    }
                    *summary.tool_usage.entry(tool_name.clone()).or_insert(0) += 1;
                },
                DebugEvent::MessageProcessed { duration, token_usage, .. } => {
                    summary.message_count += 1;
                    summary.total_message_time += *duration;
                    if let Some(usage) = token_usage {
                        summary.total_tokens += usage.total_tokens;
                    }
                },
                DebugEvent::ErrorOccurred { .. } => {
                    summary.error_count += 1;
                },
                _ => {}
            }
        }

        summary
    }

    /// Export session data
    pub fn export_json(&self) -> Value {
        json!({
            "session_id": self.id,
            "started_at": self.started_at.elapsed().as_secs(),
            "duration_ms": self.duration().as_millis(),
            "event_count": self.events.len(),
            "metadata": self.metadata,
            "events": self.events.iter().map(|e| self.event_to_json(e)).collect::<Vec<_>>()
        })
    }

    fn event_to_json(&self, event: &DebugEvent) -> Value {
        match event {
            DebugEvent::AgentCreated { agent_name, timestamp, config } => {
                json!({
                    "type": "agent_created",
                    "timestamp_ms": timestamp.elapsed().as_millis(),
                    "agent_name": agent_name,
                    "config": config
                })
            },
            DebugEvent::ToolExecuted { tool_name, timestamp, duration, input, output } => {
                json!({
                    "type": "tool_executed",
                    "timestamp_ms": timestamp.elapsed().as_millis(),
                    "duration_ms": duration.as_millis(),
                    "tool_name": tool_name,
                    "input": input,
                    "output": match output {
                        Ok(val) => json!({"success": true, "value": val}),
                        Err(err) => json!({"success": false, "error": err})
                    }
                })
            },
            DebugEvent::MessageProcessed { timestamp, duration, input_messages, output_message, token_usage } => {
                json!({
                    "type": "message_processed",
                    "timestamp_ms": timestamp.elapsed().as_millis(),
                    "duration_ms": duration.as_millis(),
                    "input_count": input_messages.len(),
                    "output_length": output_message.len(),
                    "token_usage": token_usage
                })
            },
            DebugEvent::ErrorOccurred { timestamp, error, context } => {
                json!({
                    "type": "error_occurred",
                    "timestamp_ms": timestamp.elapsed().as_millis(),
                    "error": error,
                    "context": context
                })
            },
            DebugEvent::MemoryAccessed { timestamp, operation, key, value } => {
                json!({
                    "type": "memory_accessed",
                    "timestamp_ms": timestamp.elapsed().as_millis(),
                    "operation": operation,
                    "key": key,
                    "has_value": value.is_some()
                })
            }
        }
    }
}

/// Debug summary statistics
#[derive(Debug, Clone)]
pub struct DebugSummary {
    pub session_id: String,
    pub total_duration: Duration,
    pub tool_executions: u32,
    pub tool_errors: u32,
    pub total_tool_time: Duration,
    pub message_count: u32,
    pub total_message_time: Duration,
    pub total_tokens: u32,
    pub error_count: u32,
    pub tool_usage: HashMap<String, u32>,
}

impl DebugSummary {
    fn new(session_id: String, total_duration: Duration) -> Self {
        Self {
            session_id,
            total_duration,
            tool_executions: 0,
            tool_errors: 0,
            total_tool_time: Duration::ZERO,
            message_count: 0,
            total_message_time: Duration::ZERO,
            total_tokens: 0,
            error_count: 0,
            tool_usage: HashMap::new(),
        }
    }

    /// Format summary for display
    pub fn format_display(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("🔍 Debug Session Summary ({})\n", self.session_id));
        output.push_str(&format!("⏱️  Total Duration: {:?}\n", self.total_duration));
        output.push_str(&format!("🔧 Tool Executions: {} (errors: {})\n", self.tool_executions, self.tool_errors));
        output.push_str(&format!("💬 Messages Processed: {}\n", self.message_count));
        output.push_str(&format!("🎯 Total Tokens: {}\n", self.total_tokens));
        output.push_str(&format!("❌ Errors: {}\n", self.error_count));
        
        if !self.tool_usage.is_empty() {
            output.push_str("\n📊 Tool Usage:\n");
            for (tool, count) in &self.tool_usage {
                output.push_str(&format!("  • {}: {} times\n", tool, count));
            }
        }
        
        // Performance metrics
        if self.tool_executions > 0 {
            let avg_tool_time = self.total_tool_time.as_millis() / self.tool_executions as u128;
            output.push_str(&format!("\n⚡ Performance:\n"));
            output.push_str(&format!("  • Avg Tool Execution: {}ms\n", avg_tool_time));
        }
        
        if self.message_count > 0 {
            let avg_message_time = self.total_message_time.as_millis() / self.message_count as u128;
            output.push_str(&format!("  • Avg Message Processing: {}ms\n", avg_message_time));
        }
        
        output
    }
}

/// Global debug manager
pub struct DebugManager {
    current_session: Option<DebugSession>,
    enabled: bool,
}

impl DebugManager {
    /// Create a new debug manager
    pub fn new() -> Self {
        Self {
            current_session: None,
            enabled: false,
        }
    }

    /// Enable debugging
    pub fn enable(&mut self) {
        self.enabled = true;
        if self.current_session.is_none() {
            self.current_session = Some(DebugSession::new());
        }
    }

    /// Disable debugging
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if debugging is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Start a new debug session
    pub fn start_session(&mut self) -> String {
        let session = DebugSession::new();
        let id = session.id.clone();
        self.current_session = Some(session);
        self.enabled = true;
        id
    }

    /// End current session and return summary
    pub fn end_session(&mut self) -> Option<DebugSummary> {
        if let Some(session) = self.current_session.take() {
            self.enabled = false;
            Some(session.generate_summary())
        } else {
            None
        }
    }

    /// Add event to current session
    pub fn add_event(&mut self, event: DebugEvent) {
        if self.enabled {
            if let Some(session) = &mut self.current_session {
                session.add_event(event);
            }
        }
    }

    /// Get current session
    pub fn current_session(&self) -> Option<&DebugSession> {
        self.current_session.as_ref()
    }

    /// Export current session
    pub fn export_current_session(&self) -> Option<Value> {
        self.current_session.as_ref().map(|s| s.export_json())
    }
}

impl Default for DebugManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_debug_session_creation() {
        let session = DebugSession::new();
        assert!(!session.id.is_empty());
        assert_eq!(session.events.len(), 0);
    }

    #[test]
    fn test_event_addition() {
        let mut session = DebugSession::new();
        
        let event = DebugEvent::AgentCreated {
            agent_name: "test_agent".to_string(),
            timestamp: Instant::now(),
            config: json!({"name": "test"}),
        };
        
        session.add_event(event);
        assert_eq!(session.events.len(), 1);
    }

    #[test]
    fn test_summary_generation() {
        let mut session = DebugSession::new();
        
        // Add some events
        session.add_event(DebugEvent::ToolExecuted {
            tool_name: "calculator".to_string(),
            timestamp: Instant::now(),
            duration: Duration::from_millis(100),
            input: json!({"operation": "add"}),
            output: Ok(json!({"result": 42})),
        });
        
        session.add_event(DebugEvent::ErrorOccurred {
            timestamp: Instant::now(),
            error: "Test error".to_string(),
            context: HashMap::new(),
        });
        
        let summary = session.generate_summary();
        assert_eq!(summary.tool_executions, 1);
        assert_eq!(summary.error_count, 1);
        assert!(summary.tool_usage.contains_key("calculator"));
    }

    #[test]
    fn test_debug_manager() {
        let mut manager = DebugManager::new();
        assert!(!manager.enabled);
        
        let session_id = manager.start_session();
        assert!(manager.enabled);
        assert!(!session_id.is_empty());
        
        manager.add_event(DebugEvent::AgentCreated {
            agent_name: "test".to_string(),
            timestamp: Instant::now(),
            config: json!({}),
        });
        
        let summary = manager.end_session();
        assert!(summary.is_some());
        assert!(!manager.enabled);
    }
}
