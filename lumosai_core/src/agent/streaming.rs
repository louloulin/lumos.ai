//! True streaming processing architecture for AI agents
//! 
//! This module implements event-driven, real-time streaming processing
//! to replace the current post-generation chunking approach with genuine
//! asynchronous streaming during LLM generation.

use std::collections::HashMap;
use std::sync::Arc;
use std::pin::Pin;
use async_stream::stream;
use futures::Stream;
use futures::StreamExt;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::agent::trait_def::Agent;
use crate::agent::types::{AgentGenerateOptions, AgentStep, ToolCall, ToolResult};
use crate::llm::Message;
use crate::telemetry::TraceCollector;

/// Events emitted during streaming agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AgentEvent {
    /// Text delta from LLM streaming
    TextDelta { 
        delta: String,
        step_id: Option<String>,
    },
    
    /// Tool call is starting
    ToolCallStart { 
        tool_call: ToolCall,
        step_id: String,
    },
    
    /// Tool call has completed
    ToolCallComplete { 
        tool_result: ToolResult,
        step_id: String,
    },
    
    /// Agent step has completed
    StepComplete { 
        step: AgentStep,
        step_id: String,
    },
    
    /// Generation phase has completed
    GenerationComplete { 
        final_response: String,
        total_steps: usize,
    },
    
    /// Memory update occurred
    MemoryUpdate {
        key: String,
        operation: MemoryOperation,
    },
    
    /// Error occurred during processing
    Error { 
        error: String,
        step_id: Option<String>,
    },
    
    /// Metadata/debug information
    Metadata {
        key: String,
        value: serde_json::Value,
    },
}

/// Memory operations for streaming events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryOperation {
    Set { value: serde_json::Value },
    Delete,
    Clear,
}

/// Configuration for streaming behavior
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Buffer size for text deltas (in characters)
    pub text_buffer_size: usize,
    
    /// Whether to emit metadata events
    pub emit_metadata: bool,
    
    /// Whether to emit memory update events
    pub emit_memory_updates: bool,
    
    /// Delay between text deltas (for simulation)
    pub text_delta_delay_ms: Option<u64>,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            text_buffer_size: 1, // Character-by-character for true streaming
            emit_metadata: false,
            emit_memory_updates: true,
            text_delta_delay_ms: None,
        }
    }
}

/// Wrapper that adds streaming capabilities to any Agent
pub struct StreamingAgent<T: Agent> {
    base_agent: T,
    config: StreamingConfig,
    trace_collector: Option<Arc<dyn TraceCollector>>,
}

impl<T: Agent> StreamingAgent<T> {
    /// Create a new streaming agent wrapper
    pub fn new(base_agent: T) -> Self {
        Self {
            base_agent,
            config: StreamingConfig::default(),
            trace_collector: None,
        }
    }
    
    /// Create with custom streaming configuration
    pub fn with_config(base_agent: T, config: StreamingConfig) -> Self {
        Self {
            base_agent,
            config,
            trace_collector: None,
        }
    }
    
    /// Set trace collector for telemetry
    pub fn with_trace_collector(mut self, trace_collector: Arc<dyn TraceCollector>) -> Self {
        self.trace_collector = Some(trace_collector);
        self
    }
    
    /// Execute agent with true streaming, emitting events as they occur
    pub fn execute_streaming<'a>(
        &'a self,
        messages: &'a [Message],
        options: &'a AgentGenerateOptions,
    ) -> Pin<Box<dyn Stream<Item = std::result::Result<AgentEvent, Box<dyn std::error::Error + Send + Sync>>> + Send + 'a>> {
        Box::pin(stream! {
            let run_id = Uuid::new_v4().to_string();
            let trace_id = self.start_streaming_trace(&run_id).await;
            
            // Emit metadata event for run start
            if self.config.emit_metadata {
                yield Ok(AgentEvent::Metadata {
                    key: "run_id".to_string(),
                    value: serde_json::Value::String(run_id.clone()),
                });
            }
            
            // Check if we should use function calling
            let supports_fc = self.base_agent.get_llm().supports_function_calling();
            let tools = self.base_agent.get_tools();
            let has_tools = !tools.is_empty();
            let use_function_calling = supports_fc && has_tools;
            
            if self.config.emit_metadata {
                yield Ok(AgentEvent::Metadata {
                    key: "streaming_mode".to_string(),
                    value: serde_json::Value::String(
                        if use_function_calling { "function_calling" } else { "direct" }.to_string()
                    ),
                });
            }
            
            if use_function_calling {
                // Function calling mode: stream through tool calls and responses
                match self.execute_function_calling_streaming(messages, options, &run_id).await {
                    Ok(mut events) => {
                        while let Some(event) = events.next().await {
                            yield event;
                        }
                    },
                    Err(e) => {
                        yield Err(e);
                    }
                }
            } else {
                // Direct streaming mode: stream LLM responses directly
                match self.execute_direct_streaming(messages, options, &run_id).await {
                    Ok(mut events) => {
                        while let Some(event) = events.next().await {
                            yield event;
                        }
                    },
                    Err(e) => {
                        yield Err(e);
                    }
                }
            }
            
            // End trace
            if let (Some(trace_collector), Some(trace_id)) = (&self.trace_collector, &trace_id) {
                let _ = trace_collector.end_trace(&trace_id, true).await;
            }
        })
    }
    
    /// Start streaming trace and return trace ID
    async fn start_streaming_trace(&self, run_id: &str) -> Option<String> {
        if let Some(trace_collector) = &self.trace_collector {
            let mut metadata = HashMap::new();
            metadata.insert("run_id".to_string(), serde_json::Value::String(run_id.to_string()));
            metadata.insert("streaming_mode".to_string(), serde_json::Value::Bool(true));
            
            match trace_collector.start_trace(
                "agent_streaming_execution".to_string(),
                metadata
            ).await {
                Ok(trace_id) => Some(trace_id),
                Err(_) => None,
            }
        } else {
            None
        }
    }
    
    /// Execute streaming with function calling support
    async fn execute_function_calling_streaming(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
        _run_id: &str,
    ) -> std::result::Result<Pin<Box<dyn Stream<Item = std::result::Result<AgentEvent, Box<dyn std::error::Error + Send + Sync>>> + Send>>, Box<dyn std::error::Error + Send + Sync>> {
        // For function calling, we need to:
        // 1. Stream initial LLM response
        // 2. Parse function calls from the response
        // 3. Execute tools and emit tool events
        // 4. Continue with follow-up generation if needed
        
        Ok(Box::pin(stream! {
            let step_id = Uuid::new_v4().to_string();
            
            // Stream initial LLM generation
            match self.stream_llm_generation(messages, options, &step_id).await {
                Ok(mut llm_stream) => {
                    let mut accumulated_response = String::new();
                    
                    while let Some(event_result) = llm_stream.next().await {
                        match event_result {
                            Ok(AgentEvent::TextDelta { delta, .. }) => {
                                accumulated_response.push_str(&delta);
                                yield Ok(AgentEvent::TextDelta { 
                                    delta, 
                                    step_id: Some(step_id.clone()) 
                                });
                            },
                            Ok(other_event) => yield Ok(other_event),
                            Err(e) => {
                                yield Err(e);
                                return;
                            }
                        }
                    }
                    
                    // TODO: Parse function calls from accumulated_response
                    // TODO: Execute tools and emit tool events
                    // TODO: Continue generation if more steps needed
                    
                    yield Ok(AgentEvent::GenerationComplete {
                        final_response: accumulated_response,
                        total_steps: 1,
                    });
                },
                Err(e) => {
                    yield Err(e);
                }
            }
        }))
    }
    
    /// Execute direct streaming without function calls
    async fn execute_direct_streaming(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
        _run_id: &str,
    ) -> std::result::Result<Pin<Box<dyn Stream<Item = std::result::Result<AgentEvent, Box<dyn std::error::Error + Send + Sync>>> + Send>>, Box<dyn std::error::Error + Send + Sync>> {
        let step_id = Uuid::new_v4().to_string();
        
        Ok(Box::pin(stream! {
            match self.stream_llm_generation(messages, options, &step_id).await {
                Ok(mut llm_stream) => {
                    let mut accumulated_response = String::new();
                    
                    while let Some(event_result) = llm_stream.next().await {
                        match event_result {
                            Ok(AgentEvent::TextDelta { delta, .. }) => {
                                accumulated_response.push_str(&delta);
                                yield Ok(AgentEvent::TextDelta { 
                                    delta, 
                                    step_id: Some(step_id.clone()) 
                                });
                            },
                            Ok(other_event) => yield Ok(other_event),
                            Err(e) => {
                                yield Err(e);
                                return;
                            }
                        }
                    }
                    
                    yield Ok(AgentEvent::GenerationComplete {
                        final_response: accumulated_response,
                        total_steps: 1,
                    });
                },
                Err(e) => {
                    yield Err(e);
                }
            }
        }))
    }
    
    /// Stream LLM generation with real-time text deltas
    async fn stream_llm_generation(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
        step_id: &str,
    ) -> std::result::Result<Pin<Box<dyn Stream<Item = std::result::Result<AgentEvent, Box<dyn std::error::Error + Send + Sync>>> + Send>>, Box<dyn std::error::Error + Send + Sync>> {
        // Get LLM streaming response
        let llm_options = &options.llm_options;
        
        // Convert messages to a single prompt for now 
        // TODO: Add generate_stream_with_messages to LlmProvider trait
        let prompt = messages.last()
            .map(|msg| &msg.content)
            .unwrap_or(&"".to_string())
            .clone();
            
        let mut llm_stream = self.base_agent.get_llm()
            .generate_stream(&prompt, llm_options)
            .await?;
        
        Ok(Box::pin(stream! {
            let mut text_buffer = String::new();
            
            while let Some(chunk_result) = llm_stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        text_buffer.push_str(&chunk);
                        
                        // Emit text deltas based on buffer size configuration
                        while text_buffer.len() >= self.config.text_buffer_size {
                            let delta = text_buffer.chars()
                                .take(self.config.text_buffer_size)
                                .collect::<String>();
                            
                            text_buffer = text_buffer.chars()
                                .skip(self.config.text_buffer_size)
                                .collect();
                            
                            yield Ok(AgentEvent::TextDelta {
                                delta,
                                step_id: Some(step_id.to_string()),
                            });
                            
                            // Optional delay for demonstration
                            if let Some(delay_ms) = self.config.text_delta_delay_ms {
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                            }
                        }
                    },
                    Err(e) => {
                        yield Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
                        return;
                    }
                }
            }
            
            // Emit any remaining text in buffer
            if !text_buffer.is_empty() {
                yield Ok(AgentEvent::TextDelta {
                    delta: text_buffer,
                    step_id: Some(step_id.to_string()),
                });
            }
        }))
    }
}

/// Helper trait to add streaming capabilities to existing agents
pub trait IntoStreaming<T: Agent> {
    fn into_streaming(self) -> StreamingAgent<T>;
    fn into_streaming_with_config(self, config: StreamingConfig) -> StreamingAgent<T>;
}

impl<T: Agent> IntoStreaming<T> for T {
    fn into_streaming(self) -> StreamingAgent<T> {
        StreamingAgent::new(self)
    }
    
    fn into_streaming_with_config(self, config: StreamingConfig) -> StreamingAgent<T> {
        StreamingAgent::with_config(self, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{AgentConfig, BasicAgent};
    use crate::memory::WorkingMemoryConfig;
    use crate::llm::MockLlmProvider;

    #[tokio::test]
    async fn test_streaming_agent_creation() {
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
        
        let streaming_agent = agent.into_streaming();
        assert_eq!(streaming_agent.config.text_buffer_size, 1);
    }
    
    #[tokio::test]
    async fn test_streaming_config() {
        let config = StreamingConfig {
            text_buffer_size: 5,
            emit_metadata: true,
            emit_memory_updates: false,
            text_delta_delay_ms: Some(10),
        };
        
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
        
        let streaming_agent = agent.into_streaming_with_config(config);
        assert_eq!(streaming_agent.config.text_buffer_size, 5);
        assert!(streaming_agent.config.emit_metadata);
        assert!(!streaming_agent.config.emit_memory_updates);
        assert_eq!(streaming_agent.config.text_delta_delay_ms, Some(10));
    }
}
