use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use uuid::Uuid;

use crate::Result;
use crate::llm::{LlmProvider, Message};
use crate::memory::Memory;
use crate::tool::{Tool, ToolExecutionOptions};
use super::config::{AgentConfig, AgentGenerateOptions};

/// A tool call parsed from a response
#[derive(Debug, Clone)]
pub struct ToolCall {
    /// The name of the tool to call
    pub name: String,
    /// The parameters for the tool call
    pub params: HashMap<String, Value>,
}

/// An agent that can generate responses and execute tools
pub struct Agent {
    /// Configuration for the agent
    config: AgentConfig,
    /// Tools available to the agent
    tools: HashMap<String, Arc<dyn Tool>>,
    /// LLM provider for the agent
    llm: Arc<dyn LlmProvider>,
    /// Memory for the agent
    memory: Option<Arc<dyn Memory>>,
}

impl Agent {
    /// Create a new agent
    pub fn new(
        config: AgentConfig,
        llm: Arc<dyn LlmProvider>,
    ) -> Self {
        Self {
            config,
            tools: HashMap::new(),
            llm,
            memory: None,
        }
    }
    
    /// Add a tool to the agent
    pub fn add_tool(&mut self, tool: Arc<dyn Tool>) -> &mut Self {
        self.tools.insert(tool.name().to_string(), tool);
        self
    }
    
    /// Add multiple tools to the agent
    pub fn add_tools(&mut self, tools: Vec<Arc<dyn Tool>>) -> &mut Self {
        for tool in tools {
            self.add_tool(tool);
        }
        self
    }
    
    /// Set the memory for the agent
    pub fn with_memory(&mut self, memory: Arc<dyn Memory>) -> &mut Self {
        self.memory = Some(memory);
        self
    }
    
    /// Generate a response with the agent
    pub async fn generate(&self, user_input: &str, options: &AgentGenerateOptions) -> Result<String> {
        // Create a run ID if not provided
        let run_id = options.run_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        
        // Prepare the context for the LLM
        let mut messages = vec![
            Message {
                role: "system".to_string(),
                content: options.instructions.clone().unwrap_or_else(|| self.config.instructions.clone()),
                metadata: None,
            },
        ];
        
        // Add additional context if provided
        if let Some(context) = &options.context {
            messages.extend(context.clone());
        }
        
        // Add the user input
        messages.push(Message {
            role: "user".to_string(),
            content: user_input.to_string(),
            metadata: None,
        });
        
        // Retrieve memory if available
        if let Some(memory) = &self.memory {
            let memory_config = options.memory_options.clone()
                .or_else(|| self.config.memory_config.clone())
                .unwrap_or_default();
            
            if let Ok(memory_items) = memory.retrieve(&memory_config).await {
                // Add memory items as context
                for item in memory_items {
                    messages.push(Message {
                        role: "system".to_string(),
                        content: format!("Memory: {}", item.content),
                        metadata: None,
                    });
                }
            }
        }
        
        // Create the LLM options with the messages
        let mut llm_options = options.llm_options.clone();
        llm_options.messages = Some(messages.clone());
        
        // Generate a response
        let mut response = self.llm.generate(user_input, &llm_options).await?;
        
        // Parse the response to see if it contains a tool call
        let max_steps = options.max_steps.unwrap_or(5);
        let mut current_step = 0;
        
        while current_step < max_steps {
            // See if the response contains a tool call
            if let Some(tool_call) = self.parse_tool_call(&response) {
                // Execute the tool
                if let Some(tool) = self.tools.get(&tool_call.name) {
                    messages.push(Message {
                        role: "assistant".to_string(),
                        content: response.clone(),
                        metadata: None,
                    });
                    
                    let tool_result = tool.execute(
                        tool_call.params,
                        &ToolExecutionOptions::default(),
                    ).await;
                    
                    let tool_response = match tool_result {
                        Ok(result) => format!(
                            "Tool '{}' executed successfully. Result: {}",
                            tool_call.name,
                            serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                        ),
                        Err(e) => format!("Tool '{}' execution failed: {}", tool_call.name, e),
                    };
                    
                    messages.push(Message {
                        role: "system".to_string(),
                        content: tool_response.clone(),
                        metadata: None,
                    });
                    
                    // Update LLM options with new messages
                    llm_options.messages = Some(messages.clone());
                    
                    // Generate a new response
                    response = self.llm.generate("", &llm_options).await?;
                    current_step += 1;
                } else {
                    // Tool not found
                    let error_message = format!("Tool '{}' not found", tool_call.name);
                    messages.push(Message {
                        role: "system".to_string(),
                        content: error_message,
                        metadata: None,
                    });
                    
                    // Update LLM options with new messages
                    llm_options.messages = Some(messages.clone());
                    
                    // Generate a new response
                    response = self.llm.generate("", &llm_options).await?;
                    current_step += 1;
                }
            } else {
                // No tool call, we're done
                break;
            }
        }
        
        // Store the final exchange in memory if available
        if let Some(memory) = &self.memory {
            let memory_config = options.memory_options.clone()
                .or_else(|| self.config.memory_config.clone())
                .unwrap_or_default();
            
            // Store the user message
            let user_message = Message {
                role: "user".to_string(),
                content: user_input.to_string(),
                metadata: Some(HashMap::from([
                    ("run_id".to_string(), Value::String(run_id.clone()))
                ])),
            };
            let _ = memory.store_message(&user_message, &memory_config).await;
            
            // Store the assistant message
            let assistant_message = Message {
                role: "assistant".to_string(),
                content: response.clone(),
                metadata: Some(HashMap::from([
                    ("run_id".to_string(), Value::String(run_id))
                ])),
            };
            let _ = memory.store_message(&assistant_message, &memory_config).await;
        }
        
        Ok(response)
    }
    
    /// Parse a tool call from a response
    fn parse_tool_call(&self, response: &str) -> Option<ToolCall> {
        // This is a simple implementation that looks for patterns like:
        // Using the tool 'tool_name' with parameters: { "param1": "value1" }
        
        // In a real implementation, we would parse more complex responses
        // and handle different formats
        
        if let Some(start) = response.find("Using the tool '") {
            if let Some(end_of_tool_name) = response[start + 16..].find("'") {
                let tool_name = &response[start + 16..start + 16 + end_of_tool_name];
                
                if let Some(params_start) = response[start..].find("parameters: ") {
                    let params_str = &response[start + params_start + 12..];
                    
                    if let Some(open_brace) = params_str.find('{') {
                        let params_json = &params_str[open_brace..];
                        
                        // Find the matching closing brace
                        let mut brace_count = 0;
                        let mut close_brace_pos = 0;
                        
                        for (i, c) in params_json.char_indices() {
                            if c == '{' {
                                brace_count += 1;
                            } else if c == '}' {
                                brace_count -= 1;
                                if brace_count == 0 {
                                    close_brace_pos = i;
                                    break;
                                }
                            }
                        }
                        
                        if close_brace_pos > 0 {
                            let json_str = &params_json[0..=close_brace_pos];
                            
                            if let Ok(params) = serde_json::from_str::<HashMap<String, Value>>(json_str) {
                                return Some(ToolCall {
                                    name: tool_name.to_string(),
                                    params,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        None
    }
} 