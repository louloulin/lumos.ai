//! Agent trait definition

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use futures::stream::BoxStream;
use serde_json::Value;

use crate::base::Base;
use crate::error::Result;
use crate::llm::{LlmProvider, Message};
use crate::memory::Memory;
use crate::tool::Tool;
use crate::agent::types::{
    AgentGenerateResult, 
    AgentGenerateOptions, 
    AgentStreamOptions,
    ToolCall,
};

/// Trait defining the core functionality of an agent
#[async_trait]
pub trait Agent: Base + Send + Sync {
    /// Get the name of the agent
    fn get_name(&self) -> &str;
    
    /// Get the instructions for the agent
    fn get_instructions(&self) -> &str;
    
    /// Set new instructions for the agent
    fn set_instructions(&mut self, instructions: String);
    
    /// Get the LLM provider for the agent
    fn get_llm(&self) -> Arc<dyn LlmProvider>;
    
    /// Get the memory for the agent
    fn get_memory(&self) -> Option<Arc<dyn Memory>>;
    
    /// Check if the agent has its own memory
    fn has_own_memory(&self) -> bool;
    
    /// Get all tools available to the agent
    fn get_tools(&self) -> HashMap<String, Box<dyn Tool>>;
    
    /// Add a tool to the agent
    fn add_tool(&mut self, tool: Box<dyn Tool>) -> Result<()>;
    
    /// Remove a tool from the agent
    fn remove_tool(&mut self, tool_name: &str) -> Result<()>;
    
    /// Get a specific tool by name
    fn get_tool(&self, tool_name: &str) -> Option<&Box<dyn Tool>>;
    
    /// Parse the LLM response to extract tool calls
    fn parse_tool_calls(&self, response: &str) -> Result<Vec<ToolCall>>;
    
    /// Execute a tool call and return the result
    async fn execute_tool_call(&self, tool_call: &ToolCall) -> Result<Value>;
    
    /// Format messages for the LLM provider
    fn format_messages(&self, messages: &[Message], options: &AgentGenerateOptions) -> Vec<Message>;
    
    /// Generate a title for a conversation
    async fn generate_title(&self, user_message: &Message) -> Result<String>;
    
    /// Generate a response given a set of messages
    async fn generate(&self, 
        messages: &[Message], 
        options: &AgentGenerateOptions
    ) -> Result<AgentGenerateResult>;
    
    /// Stream a response given a set of messages
    async fn stream<'a>(&'a self, 
        messages: &'a [Message], 
        options: &'a AgentStreamOptions
    ) -> Result<BoxStream<'a, Result<String>>>;
} 