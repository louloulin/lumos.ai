//! Agent executor implementation

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use futures::stream::{self, BoxStream, StreamExt};
use regex::Regex;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::error::{Error, Result};
use crate::logger::{Component, Logger};
use crate::llm::{LlmProvider, LlmOptions, Message, Role};
use crate::memory::Memory;
use crate::telemetry::TelemetrySink;
use crate::tool::{Tool, ToolExecutionOptions};
use crate::agent::{
    AgentConfig,
    AgentGenerateOptions,
    AgentGenerateResult,
    AgentStreamOptions,
    AgentStep,
    StepType,
    ToolCall,
    ToolResult,
    ToolResultStatus,
    TokenUsage,
    system_message,
    tool_message,
};
use crate::agent::trait_def::Agent;
use crate::agent::types::*;
use crate::logger::LogEntry;

/// Basic agent implementation
pub struct BasicAgent {
    /// Base component for logging and telemetry
    base: BaseComponent,
    /// Agent name
    name: String,
    /// Agent instructions
    instructions: String,
    /// LLM provider
    llm: Arc<dyn LlmProvider>,
    /// Memory
    memory: Option<Arc<dyn Memory>>,
    /// Tools available to the agent
    tools: Mutex<HashMap<String, Box<dyn Tool>>>,
}

impl BasicAgent {
    /// Create a new basic agent
    pub fn new(config: AgentConfig, llm: Arc<dyn LlmProvider>) -> Self {
        let component_config = ComponentConfig {
            name: Some(config.name.clone()),
            component: Component::Agent,
            log_level: None,
        };
        
        Self {
            base: BaseComponent::new(component_config),
            name: config.name,
            instructions: config.instructions,
            llm,
            memory: config.memory_config.and_then(|_| None), // This will be implemented later
            tools: Mutex::new(HashMap::new()),
        }
    }
    
    /// Build tool descriptions for the system message
    fn build_tool_descriptions(&self, options: &AgentGenerateOptions) -> String {
        let mut tool_descriptions = String::new();
        
        if let Ok(tools) = self.tools.lock() {
            for (_, tool) in tools.iter() {
                tool_descriptions.push_str(&format!(
                    "Tool: {}\nDescription: {}\n\n",
                    tool.name(), 
                    tool.description()
                ));
            }
        }
        
        tool_descriptions
    }

    /// Create a system message with agent instructions and tool descriptions
    fn create_system_message(&self, options: &AgentGenerateOptions) -> Message {
        // Get custom instructions or default instructions
        let instructions = options.instructions.as_ref().unwrap_or(&self.instructions);
        
        // Build tool descriptions
        let tool_descriptions = self.build_tool_descriptions(options);
        
        // Format the final system message
        let system_content = format!(
            "{}\n\nYou have access to the following tools:\n\n{}",
            instructions,
            tool_descriptions
        );
        
        Message {
            role: Role::System,
            content: system_content,
            metadata: None,
            name: None,
        }
    }
}

#[async_trait]
impl Agent for BasicAgent {
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn get_instructions(&self) -> &str {
        &self.instructions
    }
    
    fn set_instructions(&mut self, instructions: String) {
        self.instructions = instructions;
        self.logger().debug(&format!("Instructions updated for agent '{}'", self.name), None);
    }
    
    fn get_llm(&self) -> Arc<dyn LlmProvider> {
        self.llm.clone()
    }
    
    fn get_memory(&self) -> Option<Arc<dyn Memory>> {
        self.memory.clone()
    }
    
    fn has_own_memory(&self) -> bool {
        self.memory.is_some()
    }
    
    fn get_tools(&self) -> HashMap<String, Box<dyn Tool>> {
        match self.tools.lock() {
            Ok(guard) => {
                let mut tools_copy = HashMap::new();
                // Create a new hashmap with cloned tools
                for (name, tool) in guard.iter() {
                    tools_copy.insert(name.clone(), tool.clone());
                }
                tools_copy
            },
            Err(_) => HashMap::new(), // Return empty HashMap if lock failed
        }
    }
    
    fn add_tool(&mut self, tool: Box<dyn Tool>) -> Result<()> {
        let tool_name = tool.name().to_string();
        
        let mut tools = match self.tools.lock() {
            Ok(guard) => guard,
            Err(_) => return Err(Error::Lock("Could not lock tools".to_string())),
        };
        
        if tools.contains_key(&tool_name) {
            return Err(Error::AlreadyExists(format!("Tool '{}' already exists", tool_name)));
        }
        
        tools.insert(tool_name.clone(), tool);
        self.logger().debug(&format!("Tool '{}' added to agent '{}'", tool_name, self.name), None);
        
        Ok(())
    }
    
    fn remove_tool(&mut self, tool_name: &str) -> Result<()> {
        let mut tools = match self.tools.lock() {
            Ok(guard) => guard,
            Err(_) => return Err(Error::Lock("Could not lock tools".to_string())),
        };
        
        if !tools.contains_key(tool_name) {
            return Err(Error::NotFound(format!("Tool '{}' not found", tool_name)));
        }
        
        tools.remove(tool_name);
        self.logger().debug(&format!("Tool '{}' removed from agent '{}'", tool_name, self.name), None);
        
        Ok(())
    }
    
    fn get_tool(&self, tool_name: &str) -> Option<&Box<dyn Tool>> {
        None // We can't return a reference to a locked Mutex, so we'll return None
        // This is a limitation of the current design and would need refactoring
    }
    
    fn parse_tool_calls(&self, response: &str) -> Result<Vec<ToolCall>> {
        let re = Regex::new(r"Using the tool '([^']+)' with parameters: (\{[^}]+\})").unwrap();
        
        let mut tool_calls = Vec::new();
        
        for cap in re.captures_iter(response) {
            let tool_name = cap[1].to_string();
            let params_str = cap[2].to_string();
            
            match serde_json::from_str::<HashMap<String, Value>>(&params_str) {
                Ok(arguments) => {
                    tool_calls.push(ToolCall {
                        id: Uuid::new_v4().to_string(),
                        name: tool_name,
                        arguments,
                    });
                },
                Err(e) => {
                    self.logger().warn(&format!("Failed to parse tool parameters: {}", e), None);
                }
            }
        }
        
        Ok(tool_calls)
    }
    
    async fn execute_tool_call(&self, tool_call: &ToolCall) -> Result<Value> {
        // First get a clone of the tool to avoid holding the lock across await
        let tool_clone = {
            let tools = match self.tools.lock() {
                Ok(guard) => guard,
                Err(_) => return Err(Error::Lock("Could not lock tools".to_string())),
            };
            
            let tool = match tools.get(&tool_call.name) {
                Some(t) => t.clone(),
                None => return Err(Error::NotFound(format!("Tool '{}' not found", tool_call.name))),
            };
            
            tool // This will be moved out as tools guard is dropped at the end of this block
        }; // MutexGuard is dropped here
        
        let options = ToolExecutionOptions::default();
        
        // Now we can safely await without holding the lock
        tool_clone.execute(tool_call.arguments.clone(), &options).await
    }
    
    fn format_messages(&self, messages: &[Message], options: &AgentGenerateOptions) -> Vec<Message> {
        let mut formatted_messages = Vec::new();
        
        // Add system message
        formatted_messages.push(self.create_system_message(options));
        
        // Add context messages if any
        if let Some(context) = &options.context {
            formatted_messages.extend_from_slice(context);
        }
        
        // Add user messages
        formatted_messages.extend_from_slice(messages);
        
        formatted_messages
    }
    
    async fn generate_title(&self, user_message: &Message) -> Result<String> {
        if user_message.role != Role::User {
            return Err(Error::InvalidInput("Expected a user message".to_string()));
        }
        
        // Generate a title based on the user message
        let system_msg = system_message("You will generate a short title based on the first message a user begins a conversation with. \
            The title should be a summary of the user's message. Keep it under 80 characters. \
            Do not use quotes or colons. The entire text you return will be used as the title.");
        
        let messages = vec![
            system_msg,
            user_message.clone(),
        ];
        
        let options = LlmOptions::default();
        let title = self.llm.generate_with_messages(&messages, &options).await?;
        
        // Trim the title and limit to 80 chars
        let title = title.trim().chars().take(80).collect::<String>();
        
        Ok(title)
    }
    
    async fn generate(&self, 
        messages: &[Message], 
        options: &AgentGenerateOptions
    ) -> Result<AgentGenerateResult> {
        let mut steps = Vec::new();
        let mut all_messages = self.format_messages(messages, options);
        let run_id = options.run_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        let max_steps = options.max_steps.unwrap_or(5);
        let mut current_step = 0;
        let mut metadata = HashMap::new();
        
        // Log the generation start
        self.logger().debug(&format!("Starting generation for agent '{}' (run_id: {})", self.name, run_id), None);
        
        // Create initial step
        let initial_step = AgentStep {
            id: Uuid::new_v4().to_string(),
            step_type: StepType::Initial,
            input: all_messages.clone(),
            output: None,
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
            metadata: HashMap::new(),
        };
        steps.push(initial_step);
        
        let mut final_response = String::new();
        
        while current_step < max_steps {
            current_step += 1;
            
            // Generate a response
            let llm_options = options.llm_options.clone();
            let response = self.llm.generate_with_messages(&all_messages, &llm_options).await?;
            
            // Parse the response to see if it contains tool calls
            let tool_calls = self.parse_tool_calls(&response);
            
            if let Ok(calls) = tool_calls {
                if calls.is_empty() {
                    // No tool calls, this is the final response
                    let final_step = AgentStep {
                        id: Uuid::new_v4().to_string(),
                        step_type: StepType::Final,
                        input: all_messages.clone(),
                        output: Some(Message {
                            role: Role::Assistant,
                            content: response.clone(),
                            metadata: None,
                            name: None,
                        }),
                        tool_calls: Vec::new(),
                        tool_results: Vec::new(),
                        metadata: HashMap::new(),
                    };
                    
                    steps.push(final_step);
                    final_response = response;
                    break;
                } else {
                    // We have tool calls, execute them
                    let mut tool_results = Vec::new();
                    
                    for call in &calls {
                        let result = match self.execute_tool_call(call).await {
                            Ok(result) => ToolResult {
                                call_id: call.id.clone(),
                                name: call.name.clone(),
                                result,
                                status: ToolResultStatus::Success,
                            },
                            Err(e) => ToolResult {
                                call_id: call.id.clone(),
                                name: call.name.clone(),
                                result: Value::String(format!("Error: {}", e)),
                                status: ToolResultStatus::Error,
                            },
                        };
                        
                        tool_results.push(result);
                    }
                    
                    // Create a tool step
                    let tool_step = AgentStep {
                        id: Uuid::new_v4().to_string(),
                        step_type: StepType::Tool,
                        input: all_messages.clone(),
                        output: Some(Message {
                            role: Role::Assistant,
                            content: response,
                            metadata: None,
                            name: None,
                        }),
                        tool_calls: calls,
                        tool_results: tool_results.clone(),
                        metadata: HashMap::new(),
                    };
                    
                    steps.push(tool_step);
                    
                    // Add tool results to messages
                    for result in tool_results {
                        let content = format!("Tool result from {}: {}", 
                            result.name, 
                            serde_json::to_string_pretty(&result.result).unwrap_or_else(|_| "Error serializing result".to_string())
                        );
                        
                        all_messages.push(tool_message(content));
                    }
                }
            } else {
                // Error parsing tool calls, treat as final response
                let final_step = AgentStep {
                    id: Uuid::new_v4().to_string(),
                    step_type: StepType::Final,
                    input: all_messages.clone(),
                    output: Some(Message {
                        role: Role::Assistant,
                        content: response.clone(),
                        metadata: None,
                        name: None,
                    }),
                    tool_calls: Vec::new(),
                    tool_results: Vec::new(),
                    metadata: HashMap::new(),
                };
                
                steps.push(final_step);
                final_response = response;
                break;
            }
        }
        
        // If we've reached the maximum number of steps without a final response, use the last response
        if current_step >= max_steps && final_response.is_empty() {
            if let Some(last_step) = steps.last() {
                if let Some(output) = &last_step.output {
                    final_response = output.content.clone();
                }
            }
        }
        
        // Create a token usage estimate (this would be provided by the actual LLM in a real implementation)
        let usage = TokenUsage {
            prompt_tokens: all_messages.iter().map(|m| m.content.len() / 4).sum(),
            completion_tokens: final_response.len() / 4,
            total_tokens: all_messages.iter().map(|m| m.content.len() / 4).sum::<usize>() + final_response.len() / 4,
        };
        
        Ok(AgentGenerateResult {
            response: final_response,
            steps,
            usage,
            metadata,
        })
    }
    
    async fn stream<'a>(&'a self, 
        messages: &'a [Message], 
        options: &'a AgentStreamOptions
    ) -> Result<BoxStream<'a, Result<String>>> {
        // For now, we'll just convert the generate method to a stream
        // In a real implementation, this would use the streaming capabilities of the LLM
        
        let result = self.generate(messages, &AgentGenerateOptions {
            instructions: options.instructions.clone(),
            context: options.context.clone(),
            memory_options: options.memory_options.clone(),
            thread_id: options.thread_id.clone(),
            resource_id: options.resource_id.clone(),
            run_id: options.run_id.clone(),
            max_steps: options.max_steps,
            tool_choice: options.tool_choice.clone(),
            llm_options: options.llm_options.clone(),
        }).await?;
        
        // Split the response into chunks for streaming simulation
        let chunks = result.response
            .chars()
            .collect::<Vec<_>>()
            .chunks(20)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<_>>();
        
        let stream = stream::iter(chunks)
            .map(|chunk| Ok(chunk))
            .boxed();
        
        Ok(stream)
    }
}

impl Base for BasicAgent {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }
    
    fn component(&self) -> Component {
        self.base.component()
    }
    
    fn logger(&self) -> Arc<dyn Logger> {
        self.base.logger()
    }
    
    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.base.set_logger(logger);
    }
    
    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        self.base.telemetry()
    }
    
    fn set_telemetry(&mut self, telemetry: Arc<dyn TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
} 