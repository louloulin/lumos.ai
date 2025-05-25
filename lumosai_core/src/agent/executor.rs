//! Agent executor implementation

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use futures::stream::{self, BoxStream, StreamExt};
use regex::Regex;
use serde_json::Value;
use uuid::Uuid;
use serde::de::DeserializeOwned;
use tokio::sync::watch;
use tokio::io::AsyncRead;

use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::error::{Error, Result};
use crate::logger::{Component, Logger};
use crate::llm::{LlmProvider, LlmOptions, Message, Role, FunctionDefinition, ToolChoice as LlmToolChoice};
use crate::memory::Memory;
use crate::telemetry::TelemetrySink;
use crate::tool::{Tool, ToolExecutionOptions, ToolExecutionContext};
use crate::llm::function_calling_utils;
use crate::agent::types::{
    AgentGenerateResult, 
    AgentGenerateOptions, 
    AgentStreamOptions,
    StepType,
    ToolCall,
    ToolResult,
    ToolResultStatus,
    TokenUsage,
    AgentStep,
};
use crate::agent::trait_def::{Agent, AgentStructuredOutput, AgentVoiceListener, AgentVoiceSender};
use crate::voice::{VoiceProvider, VoiceOptions, ListenOptions};
use crate::memory::{WorkingMemory, create_working_memory};
use crate::agent::AgentConfig;
use crate::agent::types::{system_message, tool_message};

/// Basic agent implementation
#[allow(dead_code, clippy::borrowed_box)]
pub struct BasicAgent {
    /// Base component for logging and telemetry
    base: BaseComponent,
    /// Agent name
    name: String,
    /// Agent instructions
    instructions: String,
    /// LLM provider
    llm: Arc<dyn LlmProvider>,
    /// Tools available to the agent
    tools: Arc<Mutex<HashMap<String, Box<dyn Tool>>>>,
    /// Memory
    memory: Option<Arc<dyn Memory>>,
    /// Working memory
    working_memory: Option<Box<dyn WorkingMemory>>,
    /// 语音提供者
    voice: Option<Arc<dyn VoiceProvider>>,
    /// Temperature for LLM calls
    temperature: Option<f32>,
    /// Abort signal
    abort_signal: Option<watch::Receiver<bool>>,
    /// Output schema for structured outputs
    output_schema: Option<Value>,
    /// Experimental features flag
    experimental_output: bool,
    /// Enable function calling (if provider supports it)
    enable_function_calling: bool,
    /// Telemetry settings
    telemetry: Option<Box<dyn TelemetrySink>>,
}

impl BasicAgent {
    /// Create a new basic agent
    pub fn new(config: AgentConfig, llm: Arc<dyn LlmProvider>) -> Self {
        let component_config = ComponentConfig {
            name: Some(config.name.clone()),
            component: Component::Agent,
            log_level: None,
        };
        
        // Initialize working memory (if configured)
        let working_memory = if let Some(wm_config) = &config.working_memory {
            match create_working_memory(wm_config) {
                Ok(wm) => Some(wm),
                Err(e) => {
                    eprintln!("Failed to initialize working memory: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        Self {
            base: BaseComponent::new(component_config),
            name: config.name,
            instructions: config.instructions,
            llm,
            tools: Arc::new(Mutex::new(HashMap::new())),
            memory: config.memory_config.and_then(|_| None), // This will be implemented later
            working_memory,
            voice: config.voice_config.and_then(|_| None),
            temperature: None,
            abort_signal: None,
            output_schema: None,
            experimental_output: false,
            enable_function_calling: config.enable_function_calling.unwrap_or(true), // Default to true
            telemetry: None,
        }
    }
    
    /// Build tool descriptions for the system message
    #[allow(unused_variables)]
    fn build_tool_descriptions(&self, _options: &AgentGenerateOptions) -> String {
        let tools = match self.tools.lock() {
            Ok(tools) => tools,
            Err(_) => return "".to_string(),
        };

        if tools.is_empty() {
            return "".to_string();
        }

        let mut descriptions = String::new();
        
        for tool in tools.values() {
            descriptions.push_str(&format!("工具ID: {}\n", tool.id()));
            descriptions.push_str(&format!("描述: {}\n", tool.description()));
            
            // 添加工具参数描述
            let schema = tool.schema();
            if !schema.parameters.is_empty() {
                descriptions.push_str("参数:\n");
                for param in &schema.parameters {
                    descriptions.push_str(&format!("  - 名称: {}\n", param.name));
                    descriptions.push_str(&format!("    描述: {}\n", param.description));
                    descriptions.push_str(&format!("    类型: {}\n", param.r#type));
                    descriptions.push_str(&format!("    必须: {}\n", param.required));
                }
            }
            
            descriptions.push('\n');
        }
        
        descriptions
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
    
    /// Convert tools to function definitions for OpenAI function calling
    fn build_function_definitions(&self) -> Vec<FunctionDefinition> {
        let tools = match self.tools.lock() {
            Ok(tools) => tools,
            Err(_) => return Vec::new(),
        };

        if tools.is_empty() {
            return Vec::new();
        }

        // Use the utility function from llm module
        function_calling_utils::tools_to_function_definitions(&*tools)
    }
    
    /// Parse function calls from OpenAI function calling response
    fn parse_function_calls(&self, function_calls: &[crate::llm::FunctionCall]) -> Vec<ToolCall> {
        let mut tool_calls = Vec::new();
        
        // Get function definitions for validation
        let function_definitions = self.build_function_definitions();
        
        for func_call in function_calls {
            match serde_json::from_str::<HashMap<String, Value>>(&func_call.arguments) {
                Ok(arguments) => {
                    // Validate arguments against function schema
                    let validation_result = function_calling_utils::validate_against_schema(
                        &serde_json::to_value(&arguments).unwrap_or(Value::Null),
                        &function_definitions.iter()
                            .find(|def| def.name == func_call.name)
                            .map(|def| &def.parameters)
                            .unwrap_or(&Value::Null)
                    );
                    
                    match validation_result {
                        Ok(_) => {
                            tool_calls.push(ToolCall {
                                id: func_call.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
                                name: func_call.name.clone(),
                                arguments,
                            });
                            self.logger().debug(&format!("Function call '{}' validated successfully", func_call.name), None);
                        },
                        Err(e) => {
                            self.logger().warn(&format!("Function call '{}' failed validation: {}", func_call.name, e), None);
                            // Still add the call but log the validation failure
                            tool_calls.push(ToolCall {
                                id: func_call.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
                                name: func_call.name.clone(),
                                arguments,
                            });
                        }
                    }
                },
                Err(e) => {
                    self.logger().warn(&format!("Failed to parse function call arguments: {}", e), None);
                }
            }
        }
        
        tool_calls
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
        let tool_id = tool.id().to_string();
        
        let mut tools = match self.tools.lock() {
            Ok(guard) => guard,
            Err(_) => return Err(Error::Lock("Could not lock tools".to_string())),
        };
        
        if tools.contains_key(&tool_id) {
            return Err(Error::AlreadyExists(format!("Tool '{}' already exists", tool_id)));
        }
        
        tools.insert(tool_id.clone(), tool);
        self.logger().debug(&format!("Tool '{}' added to agent '{}'", tool_id, self.name), None);
        
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
    
    #[allow(unused_variables)]
    fn get_tool(&self, _tool_name: &str) -> Option<&Box<dyn Tool>> {
        None // We can't return a reference to a locked Mutex, so we'll return None
        // This is a limitation of the current design and would need refactoring
    }
    
    fn parse_tool_calls(&self, response: &str) -> Result<Vec<ToolCall>> {
        // First try to detect if this is a function calling response
        // (This would be handled differently in the generate method, but kept for compatibility)
        
        // Parse regex-based tool calls (existing functionality)
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
        
        // Convert HashMap to JSON Value
        let args_value = serde_json::to_value(&tool_call.arguments)
            .map_err(|e| Error::Json(e))?;
        
        // Create execution context and options
        let context = ToolExecutionContext::new()
            .with_tool_call_id(tool_call.id.clone());
        
        let options = ToolExecutionOptions::default();
        
        // Now we can safely await without holding the lock
        tool_clone.execute(args_value, context, &options).await
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
        let metadata = HashMap::new();
        
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
        
        // Check if we should use function calling
        let use_function_calling = self.enable_function_calling && 
                                  self.llm.supports_function_calling() &&
                                  !self.tools.lock().map(|tools| tools.is_empty()).unwrap_or(true);
        
        while current_step < max_steps {
            current_step += 1;
            
            if use_function_calling {
                // Use OpenAI function calling
                let function_definitions = self.build_function_definitions();
                
                if !function_definitions.is_empty() {
                    // Convert tool choice from agent options to LLM tool choice
                    let llm_tool_choice = match &options.tool_choice {
                        Some(crate::agent::types::ToolChoice::Auto) => LlmToolChoice::Auto,
                        Some(crate::agent::types::ToolChoice::None) => LlmToolChoice::None,
                        Some(crate::agent::types::ToolChoice::Required) => LlmToolChoice::Required,
                        Some(crate::agent::types::ToolChoice::Tool { tool_name }) => {
                            LlmToolChoice::Function { name: tool_name.clone() }
                        },
                        None => LlmToolChoice::Auto,
                    };
                    
                    let llm_options = options.llm_options.clone();
                    let response: crate::llm::provider::FunctionCallingResponse = self.llm.generate_with_functions(
                        &all_messages, 
                        &function_definitions,
                        &llm_tool_choice,
                        &llm_options
                    ).await?;
                    
                    if !response.function_calls.is_empty() {
                        // Execute function calls with enhanced logging
                        let tool_calls = self.parse_function_calls(&response.function_calls);
                        let mut tool_results = Vec::new();
                        
                        self.logger().info(&format!("Executing {} function calls", tool_calls.len()), None);
                        
                        for call in &tool_calls {
                            self.logger().debug(&format!("Executing function call: {} with arguments: {}", 
                                call.name, 
                                serde_json::to_string_pretty(&call.arguments).unwrap_or_else(|_| "{}".to_string())
                            ), None);
                            
                            let start_time = std::time::Instant::now();
                            let result = match self.execute_tool_call(call).await {
                                Ok(result) => {
                                    let execution_time = start_time.elapsed();
                                    self.logger().debug(&format!("Function call '{}' completed in {:?}", call.name, execution_time), None);
                                    
                                    ToolResult {
                                        call_id: call.id.clone(),
                                        name: call.name.clone(),
                                        result,
                                        status: ToolResultStatus::Success,
                                    }
                                },
                                Err(e) => {
                                    let execution_time = start_time.elapsed();
                                    self.logger().error(&format!("Function call '{}' failed after {:?}: {}", call.name, execution_time, e), None);
                                    
                                    ToolResult {
                                        call_id: call.id.clone(),
                                        name: call.name.clone(),
                                        result: Value::String(format!("Error: {}", e)),
                                        status: ToolResultStatus::Error,
                                    }
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
                                content: response.content.clone().unwrap_or_default(),
                                metadata: None,
                                name: None,
                            }),
                            tool_calls,
                            tool_results: tool_results.clone(),
                            metadata: HashMap::new(),
                        };
                        
                        steps.push(tool_step);
                        
                        // Add assistant message with function calls
                        all_messages.push(Message {
                            role: Role::Assistant,
                            content: response.content.unwrap_or_default(),
                            metadata: None,
                            name: None,
                        });
                        
                        // Add tool results to messages
                        for result in tool_results {
                            let content = format!("Tool result from {}: {}", 
                                result.name, 
                                serde_json::to_string_pretty(&result.result).unwrap_or_else(|_| "Error serializing result".to_string())
                            );
                            
                            all_messages.push(tool_message(content));
                        }
                        
                        continue; // Continue the loop for the next step
                    }
                    
                    // No function calls, this is the final response
                    let content = response.content.unwrap_or_default();
                    let final_step = AgentStep {
                        id: Uuid::new_v4().to_string(),
                        step_type: StepType::Final,
                        input: all_messages.clone(),
                        output: Some(Message {
                            role: Role::Assistant,
                            content: content.clone(),
                            metadata: None,
                            name: None,
                        }),
                        tool_calls: Vec::new(),
                        tool_results: Vec::new(),
                        metadata: HashMap::new(),
                    };
                    
                    steps.push(final_step);
                    final_response = content;
                    break;
                } else {
                    // No tools available, fall back to regular generation
                    let llm_options = options.llm_options.clone();
                    let response = self.llm.generate_with_messages(&all_messages, &llm_options).await?;
                    
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
            } else {
                // Use traditional regex-based tool calling
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
        // Check if we should use function calling for streaming
        let use_function_calling = self.enable_function_calling && 
                                  self.llm.supports_function_calling() &&
                                  !self.tools.lock().map(|tools| tools.is_empty()).unwrap_or(true);
        
        if use_function_calling {
            // For function calling, we need to process the complete response first
            // then stream the results (since function calls need to be executed)
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
            
            // Stream the function call steps and final response
            let mut stream_items = Vec::new();
            
            // Add step-by-step streaming for better UX
            for step in &result.steps {
                match step.step_type {
                    StepType::Tool => {
                        stream_items.push(format!("🔧 Executing {} tool call(s)...\n", step.tool_calls.len()));
                        for tool_call in &step.tool_calls {
                            stream_items.push(format!("  • Calling {}\n", tool_call.name));
                        }
                        for tool_result in &step.tool_results {
                            match tool_result.status {
                                ToolResultStatus::Success => {
                                    stream_items.push(format!("  ✅ {} completed\n", tool_result.name));
                                },
                                ToolResultStatus::Error => {
                                    stream_items.push(format!("  ❌ {} failed\n", tool_result.name));
                                }
                            }
                        }
                    },
                    StepType::Final => {
                        // Stream the final response in chunks
                        let response_chunks = result.response
                            .chars()
                            .collect::<Vec<_>>()
                            .chunks(20)
                            .map(|c| c.iter().collect::<String>())
                            .collect::<Vec<_>>();
                        
                        stream_items.extend(response_chunks);
                    },
                    _ => {
                        // Handle other step types
                        if let Some(output) = &step.output {
                            stream_items.push(output.content.clone());
                        }
                    }
                }
            }
            
            let stream = stream::iter(stream_items)
                .map(Ok)
                .boxed();
            
            Ok(stream)
        } else {
            // Fallback to regular streaming without function calling
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
                .map(Ok)
                .boxed();
            
            Ok(stream)
        }
    }

    /// 流式输出带回调
    async fn stream_with_callbacks<'a>(
        &'a self, 
        messages: &'a [Message], 
        options: &'a AgentStreamOptions,
        on_step_finish: Option<Box<dyn FnMut(AgentStep) + Send + 'a>>,
        on_finish: Option<Box<dyn FnOnce(AgentGenerateResult) + Send + 'a>>
    ) -> Result<BoxStream<'a, Result<String>>> {
        // 直接生成结果，而不是在后台任务中
        let generate_result = self.generate(messages, &AgentGenerateOptions {
            instructions: options.instructions.clone(),
            context: options.context.clone(),
            memory_options: options.memory_options.clone(),
            thread_id: options.thread_id.clone(),
            resource_id: options.resource_id.clone(),
            run_id: options.run_id.clone(),
            max_steps: options.max_steps,
            tool_choice: options.tool_choice.clone(),
            llm_options: options.llm_options.clone(),
            ..Default::default()
        }).await?;
        
        // 为每个步骤触发回调
        if let Some(mut on_step) = on_step_finish {
            for step in &generate_result.steps {
                on_step(step.clone());
            }
        }
        
        // 触发完成回调
        if let Some(on_finish_cb) = on_finish {
            on_finish_cb(generate_result.clone());
        }
        
        // 将回复分成块返回
        let response = generate_result.response;
        let chunks = response
            .chars()
            .collect::<Vec<_>>()
            .chunks(20)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<_>>();
        
        let stream = stream::iter(chunks)
            .map(Ok)
            .boxed();
        
        Ok(stream)
    }

    /// 获取语音提供者
    fn get_voice(&self) -> Option<Arc<dyn VoiceProvider>> {
        self.voice.clone()
    }

    /// 设置语音提供者
    fn set_voice(&mut self, voice: Arc<dyn VoiceProvider>) {
        self.voice = Some(voice);
    }

    async fn get_memory_value(&self, key: &str) -> Result<Option<Value>> {
        match &self.working_memory {
            Some(wm) => wm.get_value(key).await,
            None => Err(Error::Memory("Working memory not initialized".to_string())),
        }
    }
    
    async fn set_memory_value(&self, key: &str, value: Value) -> Result<()> {
        match &self.working_memory {
            Some(wm) => wm.set_value(key, value).await,
            None => Err(Error::Memory("Working memory not initialized".to_string())),
        }
    }
    
    async fn delete_memory_value(&self, key: &str) -> Result<()> {
        match &self.working_memory {
            Some(wm) => wm.delete_value(key).await,
            None => Err(Error::Memory("Working memory not initialized".to_string())),
        }
    }
    
    async fn clear_memory(&self) -> Result<()> {
        match &self.working_memory {
            Some(wm) => wm.clear().await,
            None => Err(Error::Memory("Working memory not initialized".to_string())),
        }
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

#[async_trait]
impl AgentVoiceListener for BasicAgent {
    async fn listen(&self, audio: impl AsyncRead + Send + Unpin + 'static, options: &ListenOptions) -> Result<String> {
        match self.get_voice() {
            Some(voice) => {
                if let Some(listener) = voice.as_listener() {
                    // 使用非泛型方法
                    // 先将 AsyncRead 转换为 Vec<u8>
                    use tokio::io::AsyncReadExt;
                    let mut buffer = Vec::new();
                    let mut reader = tokio::io::BufReader::new(audio);
                    reader.read_to_end(&mut buffer).await?;
                    
                    // 然后调用非泛型的方法
                    listener.listen(buffer, options).await
                } else {
                    Err(Error::Unsupported("The voice provider does not support speech recognition".to_string()))
                }
            },
            None => Err(Error::Unsupported("No voice provider configured for this agent".to_string()))
        }
    }
}

#[async_trait]
impl AgentStructuredOutput for BasicAgent {
    async fn generate_structured<T: DeserializeOwned + Send + 'static>(
        &self, 
        messages: &[Message], 
        options: &AgentGenerateOptions
    ) -> Result<T> {
        // Use the schema to generate structured output
        let formatted_messages = self.format_messages(messages, options);
        
        // Add schema instruction if schema is available
        let schema_messages = if let Some(schema) = &self.output_schema {
            let schema_str = serde_json::to_string_pretty(schema)
                .map_err(|e| Error::Parsing(format!("Failed to serialize schema: {}", e)))?;
            
            let mut new_messages = formatted_messages.clone();
            let schema_instruction = format!(
                "Your response must be valid JSON that conforms to this schema:\n\n```json\n{}\n```\n\nDo not include any explanation or additional text, just the JSON object.",
                schema_str
            );
            
            // Add schema instruction at the end of system message or create a new one
            if let Some(pos) = new_messages.iter().position(|m| m.role == Role::System) {
                let mut system_msg = new_messages[pos].clone();
                system_msg.content = format!("{}\n\n{}", system_msg.content, schema_instruction);
                new_messages[pos] = system_msg;
            } else {
                new_messages.insert(0, system_message(schema_instruction));
            }
            
            new_messages
        } else {
            formatted_messages
        };
        
        let json_response = self.llm.generate_with_messages(&schema_messages, &options.llm_options).await?;
        
        // Parse the JSON response
        serde_json::from_str::<T>(&json_response)
            .map_err(|e| Error::Parsing(format!("Failed to parse structured output: {}", e)))
    }
}

#[async_trait]
impl AgentVoiceSender for BasicAgent {
    async fn speak(&self, text: &str, options: &VoiceOptions) -> Result<BoxStream<'_, Result<Vec<u8>>>> {
        // 获取声音提供者
        let voice_provider = self.get_voice().ok_or_else(|| {
            Error::Unsupported("No voice provider configured for this agent".to_string())
        })?;
        
        // 使用声音提供者生成语音数据
        let mut buffer = Vec::new();
        
        // 先生成完整的语音数据
        let stream = voice_provider.speak(text, options).await?;
        futures::pin_mut!(stream);
        while let Some(chunk) = stream.next().await {
            if let Ok(data) = chunk {
                buffer.push(data);
            }
        }
        
        // 创建一个新的流，包含所有语音数据
        // 这个流不依赖于voice_provider的生命周期
        let output_stream = stream::iter(buffer)
            .map(Ok)
            .boxed();
        
        Ok(output_stream)
    }
}

// 为未使用的结构体添加告警注解
#[allow(dead_code)]
struct AgentRef<'a>(&'a BasicAgent);

// 实现Send和Sync，使AgentRef可以跨线程传递
unsafe impl Send for AgentRef<'_> {}
unsafe impl Sync for AgentRef<'_> {}

#[cfg(test)]
mod voice_tests {
    use std::sync::Arc;
    use super::*;
    use crate::voice::{MockVoice, VoiceOptions};
    use serde::{Deserialize, Serialize};
    
    /// 模拟的LLM提供者，用于测试
    struct MockLlmProvider {
        responses: Vec<String>,
        index: std::sync::atomic::AtomicUsize,
    }
    
    impl MockLlmProvider {
        fn new(responses: Vec<String>) -> Self {
            Self {
                responses,
                index: std::sync::atomic::AtomicUsize::new(0),
            }
        }
    }
    
    #[async_trait]
    impl LlmProvider for MockLlmProvider {
        async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> Result<String> {
            let index = self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst) % self.responses.len();
            Ok(self.responses[index].clone())
        }
        
        async fn generate_with_messages(&self, _messages: &[Message], _options: &LlmOptions) -> Result<String> {
            let index = self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst) % self.responses.len();
            Ok(self.responses[index].clone())
        }
        
        async fn generate_stream<'a>(
            &'a self,
            _prompt: &'a str,
            _options: &'a LlmOptions,
        ) -> Result<futures::stream::BoxStream<'a, Result<String>>> {
            unimplemented!("Streaming not implemented for mock provider")
        }
        
        async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>> {
            unimplemented!("Embeddings not implemented for mock provider")
        }
    }
    
    // 用于结构化输出的测试结构
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestOutput {
        message: String,
        value: i32,
    }
    
    // 扩展MockLlmProvider以返回结构化输出
    struct StructuredMockLlm;
    
    #[async_trait]
    impl LlmProvider for StructuredMockLlm {
        async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> Result<String> {
            // 返回JSON格式的结构化输出
            Ok(r#"{"message": "Hello, world!", "value": 42}"#.to_string())
        }
        
        async fn generate_with_messages(&self, _messages: &[Message], _options: &LlmOptions) -> Result<String> {
            // 返回JSON格式的结构化输出
            Ok(r#"{"message": "Hello, world!", "value": 42}"#.to_string())
        }
        
        async fn generate_stream<'a>(
            &'a self,
            _prompt: &'a str,
            _options: &'a LlmOptions,
        ) -> Result<futures::stream::BoxStream<'a, Result<String>>> {
            unimplemented!("Streaming not implemented for mock provider")
        }
        
        async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>> {
            unimplemented!("Embeddings not implemented for mock provider")
        }
    }
    
    #[tokio::test]
    async fn test_agent_structured_output() {
        // 创建一个结构化输出的LLM提供者
        let mock_llm = Arc::new(StructuredMockLlm);
        
        // 创建Agent
        let config = AgentConfig {
            name: "TestAgent".to_string(),
            instructions: "You are a test agent.".to_string(),
            memory_config: None,
            ..Default::default()
        };
        
        let agent = BasicAgent::new(config, mock_llm);
        
        // 测试结构化输出
        let user_message = Message {
            role: Role::User,
            content: "Hello".to_string(),
            metadata: None,
            name: None,
        };
        
        let result: TestOutput = agent.generate_structured(&[user_message], &AgentGenerateOptions::default()).await.unwrap();
        
        assert_eq!(result.message, "Hello, world!");
        assert_eq!(result.value, 42);
    }
    
    #[tokio::test]
    async fn test_agent_voice() {
        // 创建一个模拟LLM提供者
        let mock_llm = Arc::new(MockLlmProvider::new(vec![
            "Hello, this is a voice test".to_string(),
        ]));
        
        // 创建Agent
        let config = AgentConfig {
            name: "TestAgent".to_string(),
            instructions: "You are a test agent.".to_string(),
            memory_config: None,
            ..Default::default()
        };
        
        let mut agent = BasicAgent::new(config, mock_llm);
        
        // 添加语音提供者
        let voice = Arc::new(MockVoice::new());
        agent.set_voice(voice);
        
        // 测试语音功能 - 使用AgentVoiceSender trait
        let agent_voice_sender: &dyn AgentVoiceSender = &agent;
        let audio_stream = agent_voice_sender.speak(
            "Test voice functionality", 
            &VoiceOptions::default()
        ).await.unwrap();
        
        // 收集音频数据
        let mut audio_data = Vec::new();
        futures::pin_mut!(audio_stream);
        while let Some(result) = futures::StreamExt::next(&mut audio_stream).await {
            audio_data.push(result.unwrap());
        }
        
        // 验证有数据返回
        assert!(!audio_data.is_empty());
    }
    
    #[tokio::test]
    async fn test_agent_voice_sender_speak() {
        // 创建一个模拟LLM提供者
        let mock_llm = Arc::new(MockLlmProvider::new(vec![
            "Hello, this is a voice test".to_string(),
        ]));
        
        // 创建Agent
        let config = AgentConfig {
            name: "TestAgent".to_string(),
            instructions: "You are a test agent.".to_string(),
            memory_config: None,
            ..Default::default()
        };
        
        let mut agent = BasicAgent::new(config, mock_llm);
        
        // 添加语音提供者
        let voice = Arc::new(MockVoice::new());
        agent.set_voice(voice);
        
        // 直接调用speak方法
        let agent_voice_sender = &agent as &dyn AgentVoiceSender;
        let result = agent_voice_sender.speak("Test direct voice call", &VoiceOptions::default()).await;
        
        // 验证返回结果
        assert!(result.is_ok(), "speak方法应该返回Ok结果");
        
        if let Ok(stream) = result {
            // 收集所有音频数据
            let mut audio_data = Vec::new();
            futures::pin_mut!(stream);
            
            while let Some(chunk_result) = stream.next().await {
                if let Ok(chunk) = chunk_result {
                    audio_data.push(chunk);
                }
            }
            
            // 验证有音频数据返回
            assert!(!audio_data.is_empty(), "应该有音频数据返回");
            println!("成功收集了 {} 段音频数据", audio_data.len());
        }
    }
}