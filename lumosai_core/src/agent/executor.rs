//! Agent executor implementation

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;

use regex::Regex;
use serde_json::{Value, Map};
use uuid::Uuid;
use tokio::sync::watch;

use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::error::{Error, Result};
use crate::logger::{Component, Logger};
use crate::llm::{LlmProvider, LlmOptions, Message, Role, FunctionDefinition, ToolChoice as LlmToolChoice};
use crate::memory::Memory;
use crate::agent::trait_def::AgentStatus;
use crate::telemetry::{TelemetrySink, MetricsCollector, TraceCollector, AgentMetrics, ExecutionContext, StepType as TraceStepType, TokenUsage as TelemetryTokenUsage, TraceStep};
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
use crate::agent::trait_def::Agent;
use crate::voice::VoiceProvider;
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
    /// Metrics collector for performance monitoring
    metrics_collector: Option<Arc<dyn MetricsCollector>>,
    /// Trace collector for execution tracing
    trace_collector: Option<Arc<dyn TraceCollector>>,
    /// Agent status
    status: AgentStatus,
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

        // Initialize memory (if configured)
        let memory = if let Some(memory_config) = &config.memory_config {
            // Create a basic memory with working memory
            let working_memory_arc = working_memory.as_ref().map(|wm| {
                // Convert Box<dyn WorkingMemory> to Arc<dyn WorkingMemory>
                // This is a workaround - ideally we should store Arc directly
                use crate::memory::BasicWorkingMemory;
                Arc::new(BasicWorkingMemory::new(crate::memory::WorkingMemoryConfig {
                    enabled: true,
                    template: None,
                    content_type: None,
                    max_capacity: Some(100),
                })) as Arc<dyn crate::memory::WorkingMemory>
            });

            let basic_memory = crate::memory::BasicMemory::new(working_memory_arc, None);
            Some(Arc::new(basic_memory) as Arc<dyn crate::memory::Memory>)
        } else {
            None
        };

        Self {
            base: BaseComponent::new(component_config),
            name: config.name,
            instructions: config.instructions,
            llm,
            tools: Arc::new(Mutex::new(HashMap::new())),
            memory,
            working_memory,
            voice: config.voice_config.and_then(|_| None),
            temperature: None,
            abort_signal: None,
            output_schema: None,
            experimental_output: false,
            enable_function_calling: config.enable_function_calling.unwrap_or(true), // Default to true
            telemetry: None,
            metrics_collector: None,
            trace_collector: None,
            status: AgentStatus::Ready,
        }
    }
    
    /// Set metrics collector
    pub fn with_metrics_collector(mut self, collector: Arc<dyn MetricsCollector>) -> Self {
        self.metrics_collector = Some(collector);
        self
    }
    
    /// Set trace collector
    pub fn with_trace_collector(mut self, collector: Arc<dyn TraceCollector>) -> Self {
        self.trace_collector = Some(collector);
        self
    }
    
    /// Set both metrics and trace collectors
    pub fn with_monitoring(
        mut self, 
        metrics_collector: Arc<dyn MetricsCollector>,
        trace_collector: Arc<dyn TraceCollector>
    ) -> Self {
        self.metrics_collector = Some(metrics_collector);
        self.trace_collector = Some(trace_collector);
        self
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
        
        // Add instruction for legacy regex tool calling
        descriptions.push_str("To use tools, use the following format:\n");
        descriptions.push_str("思考: [your reasoning about which tool to use]\n");
        descriptions.push_str("工具: <tool_id>\n");
        descriptions.push_str("参数: {\"parameter_name\": \"parameter_value\"}\n");
        descriptions.push_str("结果: [tool execution result will appear here]\n\n");
        descriptions.push_str("Available tools:\n\n");
        
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
        
        // Check if we're using function calling mode
        let tools = self.tools.lock().ok();
        let has_tools = tools.as_ref().map_or(false, |t| !t.is_empty());
        let use_function_calling = crate::llm::function_calling_utils::should_use_function_calling(
            self.enable_function_calling,
            self.llm.supports_function_calling(),
            has_tools
        );
        
        // Build tool descriptions for legacy mode
        let tool_descriptions = if !use_function_calling && has_tools {
            let tools_ref = tools.as_ref().unwrap();
            Some(crate::llm::function_calling_utils::create_tools_description(tools_ref, None))
        } else {
            None
        };
        
        // Generate appropriate system prompt based on mode
        let system_content = crate::llm::function_calling_utils::generate_system_prompt(
            instructions,
            use_function_calling,
            tool_descriptions.as_deref()
        );
        
        if use_function_calling {
            self.logger().debug("Using function calling mode - omitting tool format from system message", None);
        } else if has_tools {
            self.logger().debug("Using legacy regex mode - including tool format in system message", None);
        }
        
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

/// Call LLM with monitoring and telemetry
async fn call_llm_with_monitoring(
    llm: &dyn LlmProvider,
    trace_collector: &Option<Arc<dyn TraceCollector>>,
    messages: &[Message],
    options: &LlmOptions,
    trace_id: &Option<String>,
    step_name: &str,
    agent_metrics: &mut Option<AgentMetrics>,
) -> Result<String> {
    let start_time = std::time::Instant::now();
    
    // Record LLM call start in trace
    if let (Some(trace_collector), Some(trace_id)) = (trace_collector, trace_id) {
        let mut llm_step = TraceStep::new(
            step_name.to_string(),
            TraceStepType::LlmCall,
        );
        llm_step.metadata.insert("messages_count".to_string(), Value::from(messages.len()));
        if let Some(model) = &options.model {
            llm_step.metadata.insert("model".to_string(), Value::from(model.clone()));
        }
        let _ = trace_collector.add_trace_step(trace_id, llm_step).await;
    }
    
    // Make the LLM call
    let response = llm.generate_with_messages(messages, options).await?;
    let execution_time = start_time.elapsed();
    
    // Update agent metrics if available
    if let Some(metrics) = agent_metrics {
        // Note: LLM provider returns String, so we can't get usage info here
        // Usage tracking would need to be handled by the provider implementation
        metrics.execution_time_ms += execution_time.as_millis() as u64;
    }
    
    // Record LLM call completion in trace
    if let (Some(trace_collector), Some(trace_id)) = (trace_collector, trace_id) {
        let mut completion_step = TraceStep::new(
            format!("{} completed", step_name),
            TraceStepType::DataProcessing,
        );
        completion_step.metadata.insert("execution_time_ms".to_string(), Value::from(execution_time.as_millis() as u64));
        completion_step.metadata.insert("response_length".to_string(), Value::from(response.len()));
        let _ = trace_collector.add_trace_step(trace_id, completion_step).await;
    }
    
    Ok(response)
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
        let tool_name = tool.name().unwrap_or("unknown").to_string();

        let mut tools = match self.tools.lock() {
            Ok(guard) => guard,
            Err(poison_error) => {
                eprintln!("Tools mutex poisoned during add_tool, attempting recovery: {}", poison_error);
                poison_error.into_inner()
            }
        };

        if tools.contains_key(&tool_name) {
            return Err(Error::Tool(format!("Tool '{}' already exists", tool_name)));
        }
        
        tools.insert(tool_name.clone(), tool);
        self.logger().debug(&format!("Tool '{}' added to agent '{}'", tool_name, self.name), None);
        
        Ok(())
    }
    
    fn remove_tool(&mut self, tool_name: &str) -> Result<()> {
        let mut tools = match self.tools.lock() {
            Ok(guard) => guard,
            Err(poison_error) => {
                eprintln!("Tools mutex poisoned during remove_tool, attempting recovery: {}", poison_error);
                poison_error.into_inner()
            }
        };
        
        if !tools.contains_key(tool_name) {
            return Err(Error::NotFound(format!("Tool '{}' not found", tool_name)));
        }
        
        tools.remove(tool_name);
        self.logger().debug(&format!("Tool '{}' removed from agent '{}'", tool_name, self.name), None);
        
        Ok(())
    }
    
    fn get_tool(&self, tool_name: &str) -> Option<Box<dyn Tool>> {
        match self.tools.lock() {
            Ok(tools) => tools.get(tool_name).cloned(),
            Err(_) => None,
        }
    }
    
    fn parse_tool_calls(&self, response: &str) -> Result<Vec<ToolCall>> {
        // Enhanced method for parsing tool calls
        // Now supports multiple parsing strategies with better error handling
        
        // Check if we're using function calling - if so, log warning and return empty
        if self.enable_function_calling && self.llm.supports_function_calling() {
            self.logger().warn("parse_tool_calls called despite function calling being enabled", None);
            return Ok(Vec::new());
        }
        
        // Enhanced parsing with multiple patterns
        // Try JSON extraction first (for structured responses)
        if response.contains("```json") && response.contains("```") {
            // Try to extract JSON code blocks
            let mut tool_calls = Vec::new();
            let json_regex = Regex::new(r"(?s)```json\s*\n?(.*?)\n?\s*```")
                .map_err(|e| Error::Tool(format!("Failed to compile JSON regex: {}", e)))?;
            
            for cap in json_regex.captures_iter(response) {
                let json_str = cap[1].trim();
                
                match serde_json::from_str::<HashMap<String, Value>>(json_str) {
                    Ok(json_obj) => {
                        // Extract tool name and arguments from JSON
                        if let Some(tool_name) = json_obj.get("tool").and_then(Value::as_str) {
                            if let Some(args) = json_obj.get("parameters") {
                                let arguments = match args {
                                    Value::Object(obj) => obj.clone(),
                                    _ => {
                                        // Try to parse as string
                                        if let Some(arg_str) = args.as_str() {
                                            match serde_json::from_str::<Map<String, Value>>(arg_str) {
                                                Ok(parsed) => parsed,
                                                Err(_) => {
                                                    // Fallback: create a single parameter "value"
                                                    let mut single_param = Map::new();
                                                    single_param.insert("value".to_string(), args.clone());
                                                    single_param
                                                }
                                            }
                                        } else {
                                            let mut single_param = Map::new();
                                            single_param.insert("value".to_string(), args.clone());
                                            single_param
                                        }
                                    }
                                };
                                
                                tool_calls.push(ToolCall {
                                    id: Uuid::new_v4().to_string(),
                                    name: tool_name.to_string(),
                                    arguments: arguments.into_iter().collect(),
                                });
                            }
                        }
                    },
                    Err(e) => {
                        self.logger().warn(&format!("Failed to parse JSON code block: {}", e), None);
                    }
                }
            }
            
            if !tool_calls.is_empty() {
                self.logger().info(&format!("Parsed {} tool calls from code blocks", tool_calls.len()), None);
                return Ok(tool_calls);
            }
        }
        
        // If JSON extraction didn't work, try the legacy regex pattern
        let re = Regex::new(r"Using the tool '([^']+)' with parameters: (\{[^}]+\})")
            .map_err(|e| Error::Tool(format!("Failed to compile tool call regex: {}", e)))?;
        
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
        
        // Try additional pattern for function-style calls
        if tool_calls.is_empty() {
            let fn_regex = Regex::new(r"(\w+)\(([^)]*)\)")
                .map_err(|e| Error::Tool(format!("Failed to compile function call regex: {}", e)))?;
            
            for cap in fn_regex.captures_iter(response) {
                let tool_name = cap[1].to_string();
                let params_str = cap[2].trim();
                
                // Only process if we have this tool
                let tools = match self.tools.lock() {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                
                if !tools.contains_key(&tool_name) {
                    continue;
                }
                
                // Try to parse arguments
                let mut arguments = HashMap::new();
                
                // Try JSON-like string first
                if params_str.starts_with('{') && params_str.ends_with('}') {
                    match serde_json::from_str::<HashMap<String, Value>>(params_str) {
                        Ok(args) => {
                            arguments = args;
                        },
                        Err(_) => {
                            // Fall through to simple parsing
                        }
                    }
                }
                
                // If not parsed as JSON, try key-value parsing
                if arguments.is_empty() && !params_str.is_empty() {
                    // Simple key=value,key2=value2 parsing
                    for pair in params_str.split(',') {
                        let parts: Vec<&str> = pair.split('=').collect();
                        if parts.len() == 2 {
                            let key = parts[0].trim();
                            let value = parts[1].trim();
                            
                            // Try to parse as number or boolean first
                            if let Ok(num) = value.parse::<i64>() {
                                arguments.insert(key.to_string(), Value::Number(num.into()));
                            } else if let Ok(float) = value.parse::<f64>() {
                                // Use serde_json's number constructor which handles precision
                                if let Some(num) = serde_json::Number::from_f64(float) {
                                    arguments.insert(key.to_string(), Value::Number(num));
                                } else {
                                    arguments.insert(key.to_string(), Value::String(value.to_string()));
                                }
                            } else if value == "true" {
                                arguments.insert(key.to_string(), Value::Bool(true));
                            } else if value == "false" {
                                arguments.insert(key.to_string(), Value::Bool(false));
                            } else {
                                // Remove quotes if present
                                let clean_value = value.trim_matches('"').trim_matches('\'');
                                arguments.insert(key.to_string(), Value::String(clean_value.to_string()));
                            }
                        }
                    }
                }
                
                // If parsing worked, add the tool call
                if !arguments.is_empty() || params_str.is_empty() {
                    tool_calls.push(ToolCall {
                        id: Uuid::new_v4().to_string(),
                        name: tool_name,
                        arguments,
                    });
                }
            }
        }
        
        if !tool_calls.is_empty() {
            self.logger().info(&format!("Parsed {} tool calls using enhanced parsing methods", tool_calls.len()), None);
        }
        
        Ok(tool_calls)
    }





    async fn execute_tool_call(&self, tool_call: &ToolCall) -> Result<Value> {
        let start_time = std::time::Instant::now();
        
        // First get a clone of the tool to avoid holding the lock across await
        let tool_clone = {
            let tools = match self.tools.lock() {
                Ok(guard) => guard,
                Err(poison_error) => {
                    // Log the error and attempt recovery
                    eprintln!("Tools mutex poisoned, attempting recovery: {}", poison_error);
                    poison_error.into_inner()
                }
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
        
        // Execute tool and record metrics
        let result = tool_clone.execute(args_value.clone(), context, &options).await;
        let execution_time = start_time.elapsed();
        
        // Record tool metrics regardless of success/failure
        if let Some(metrics_collector) = &self.metrics_collector {
            let input_size = serde_json::to_string(&args_value).unwrap_or_default().len();
            let (output_size, success, error) = match &result {
                Ok(output) => (
                    serde_json::to_string(output).unwrap_or_default().len(),
                    true,
                    None
                ),
                Err(e) => (0, false, Some(e.to_string())),
            };
            
            let tool_metrics = crate::telemetry::ToolMetrics {
                tool_name: tool_call.name.clone(),
                execution_time_ms: execution_time.as_millis() as u64,
                success,
                error,
                input_size_bytes: input_size,
                output_size_bytes: output_size,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_else(|_| std::time::Duration::from_millis(0))
                    .as_millis() as u64,
            };
            
            let _ = metrics_collector.record_tool_execution(tool_metrics).await;
        }
        
        result
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
    
    async fn generate_with_memory(&self,
        messages: &[Message],
        thread_id: Option<String>,
        options: &AgentGenerateOptions
    ) -> Result<AgentGenerateResult> {
        // For now, delegate to regular generate method
        // TODO: Implement proper memory thread integration
        self.logger().debug(&format!("generate_with_memory called with thread_id: {:?}", thread_id), None);
        self.generate(messages, options).await
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
        
        // Initialize comprehensive monitoring
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::SystemTime(format!("Failed to get system time: {}", e)))?
            .as_millis() as u64;
        
        // Create execution context for telemetry
        let execution_context = ExecutionContext {
            session_id: options.thread_id.clone(),
            user_id: options.resource_id.clone(),
            request_id: Some(run_id.clone()),
            environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        };
        
        // Initialize agent metrics
        let mut agent_metrics = if self.metrics_collector.is_some() {
            Some(AgentMetrics::new(self.name.clone(), execution_context.clone()))
        } else {
            None
        };
        
        // Start execution trace
        let trace_id = if let Some(trace_collector) = &self.trace_collector {
            match trace_collector.start_trace(
                format!("agent_{}", self.name),
                {
                    let mut trace_metadata = HashMap::new();
                    trace_metadata.insert("run_id".to_string(), serde_json::Value::String(run_id.clone()));
                    trace_metadata.insert("agent_name".to_string(), serde_json::Value::String(self.name.clone()));
                    trace_metadata.insert("max_steps".to_string(), serde_json::Value::Number(serde_json::Number::from(max_steps)));
                    trace_metadata.insert("message_count".to_string(), serde_json::Value::Number(serde_json::Number::from(messages.len())));
                    trace_metadata
                }
            ).await {
                Ok(id) => {
                    self.logger().debug(&format!("Started execution trace: {}", id), None);
                    Some(id)
                },
                Err(e) => {
                    self.logger().warn(&format!("Failed to start trace: {}", e), None);
                    None
                }
            }
        } else {
            None
        };
        
        // Record initial trace step
        if let (Some(trace_collector), Some(trace_id)) = (&self.trace_collector, &trace_id) {
            let mut init_step = TraceStep::new(
                "Agent execution started".to_string(),
                TraceStepType::DataProcessing,
            );
            init_step.input = Some(serde_json::to_value(messages).unwrap_or_default());
            init_step.metadata.insert("messages_count".to_string(), serde_json::Value::Number(serde_json::Number::from(messages.len())));
            init_step.metadata.insert("max_steps".to_string(), serde_json::Value::Number(serde_json::Number::from(max_steps)));
            init_step.success = true;
            init_step.duration_ms = 0;
            
            let _ = trace_collector.add_trace_step(trace_id, init_step).await;
        }
        
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
        let total_tokens = TelemetryTokenUsage::default();
        let mut total_tool_calls = 0;
        let mut total_errors = 0;
        
        // Check if we should use function calling
        let use_function_calling = self.enable_function_calling && 
                                  self.llm.supports_function_calling() &&
                                  !self.tools.lock().map(|tools| tools.is_empty()).unwrap_or(true);
        
        self.logger().info(&format!("Using function calling mode: {}", use_function_calling), None);
        
        // Record function calling mode in trace
        if let (Some(trace_collector), Some(trace_id)) = (&self.trace_collector, &trace_id) {
            let mut mode_step = TraceStep::new(
                "Function calling mode determined".to_string(),
                TraceStepType::DataProcessing,
            );
            mode_step.metadata.insert("function_calling_enabled".to_string(), serde_json::Value::Bool(use_function_calling));
            mode_step.metadata.insert("tools_available".to_string(), serde_json::Value::Number(serde_json::Number::from(self.tools.lock().map(|t| t.len()).unwrap_or(0))));
            mode_step.success = true;
            mode_step.duration_ms = 0;
            
            let _ = trace_collector.add_trace_step(trace_id, mode_step).await;
        }

        while current_step < max_steps {
            current_step += 1;
            let step_start_time = std::time::Instant::now();
            
            self.logger().debug(&format!("Starting step {} of {}", current_step, max_steps), None);
            
            // Record step start in trace
            if let (Some(trace_collector), Some(trace_id)) = (&self.trace_collector, &trace_id) {
                let mut step = TraceStep::new(
                    format!("Step {} - LLM Generation", current_step),
                    TraceStepType::LlmCall,
                );
                step.metadata.insert("step_number".to_string(), serde_json::Value::Number(serde_json::Number::from(current_step)));
                step.metadata.insert("messages_count".to_string(), serde_json::Value::Number(serde_json::Number::from(all_messages.len())));
                step.input = Some(serde_json::to_value(&all_messages).unwrap_or_default());
                
                let _ = trace_collector.add_trace_step(trace_id, step).await;
            }
            
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
                    let llm_start_time = std::time::Instant::now();
                    
                    let response: crate::llm::provider::FunctionCallingResponse = self.llm.generate_with_functions(
                        &all_messages, 
                        &function_definitions,
                        &llm_tool_choice,
                        &llm_options
                    ).await?;
                    
                    let llm_duration = llm_start_time.elapsed();
                    
                    // Note: FunctionCallingResponse doesn't include usage info
                    // Token usage tracking would need to be handled separately
                    
                    // Record LLM call completion in trace
                    if let (Some(trace_collector), Some(trace_id)) = (&self.trace_collector, &trace_id) {
                        let mut llm_step = TraceStep::new(
                            format!("LLM call completed - step {}", current_step),
                            TraceStepType::LlmCall,
                        );
                        llm_step.duration_ms = llm_duration.as_millis() as u64;
                        llm_step.success = true;
                        let empty_content = String::new();
                        let response_content = response.content.as_ref().unwrap_or(&empty_content);
                        llm_step.metadata.insert("response_length".to_string(), serde_json::Value::Number(serde_json::Number::from(response_content.len())));
                        llm_step.metadata.insert("function_calls_count".to_string(), serde_json::Value::Number(serde_json::Number::from(response.function_calls.len())));
                        llm_step.output = Some(serde_json::json!({
                            "content": response_content,
                            "function_calls_count": response.function_calls.len()
                        }));
                        
                        let _ = trace_collector.add_trace_step(trace_id, llm_step).await;
                    }
                    
                    if !response.function_calls.is_empty() {
                        // Execute function calls with enhanced logging and metrics
                        let tool_calls = self.parse_function_calls(&response.function_calls);
                        let mut tool_results = Vec::new();
                        
                        total_tool_calls += tool_calls.len();
                        if let Some(ref mut metrics) = agent_metrics {
                            metrics.tool_calls_count += tool_calls.len();
                        }
                        
                        self.logger().info(&format!("Executing {} function calls", tool_calls.len()), None);
                        
                        for call in &tool_calls {
                            self.logger().debug(&format!("Executing function call: {} with arguments: {}", 
                                call.name, 
                                serde_json::to_string_pretty(&call.arguments).unwrap_or_else(|_| "{}".to_string())
                            ), None);
                            
                            let tool_start_time = std::time::Instant::now();
                            
                            let result = match self.execute_tool_call(call).await {
                                Ok(result) => {
                                    let execution_time = tool_start_time.elapsed();
                                    self.logger().debug(&format!("Function call '{}' completed in {:?}", call.name, execution_time), None);
                                    
                                    // Record successful tool metrics
                                    if let Some(metrics_collector) = &self.metrics_collector {
                                        let tool_metrics = crate::telemetry::ToolMetrics {
                                            tool_name: call.name.clone(),
                                            execution_time_ms: execution_time.as_millis() as u64,
                                            success: true,
                                            error: None,
                                            input_size_bytes: serde_json::to_string(&call.arguments).unwrap_or_default().len(),
                                            output_size_bytes: serde_json::to_string(&result).unwrap_or_default().len(),
                                            timestamp: SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap_or_else(|_| std::time::Duration::from_millis(0))
                                                .as_millis() as u64,
                                        };
                                        
                                        let _ = metrics_collector.record_tool_execution(tool_metrics).await;
                                    }
                                    
                                    // Record successful tool call in trace
                                    if let (Some(trace_collector), Some(trace_id)) = (&self.trace_collector, &trace_id) {
                                        let mut step = TraceStep::new(
                                            format!("Tool call: {} - Success", call.name),
                                            TraceStepType::ToolCall,
                                        );
                                        step.metadata.insert("tool_name".to_string(), serde_json::Value::String(call.name.clone()));
                                        step.metadata.insert("success".to_string(), serde_json::Value::Bool(true));
                                        step.metadata.insert("input_size".to_string(), serde_json::Value::Number(serde_json::Number::from(serde_json::to_string(&call.arguments).unwrap_or_default().len())));
                                        step.metadata.insert("output_size".to_string(), serde_json::Value::Number(serde_json::Number::from(serde_json::to_string(&result).unwrap_or_default().len())));
                                        step.duration_ms = execution_time.as_millis() as u64;
                                        step.success = true;
                                        step.input = Some(serde_json::to_value(&call.arguments).unwrap_or_default());
                                        step.output = Some(serde_json::to_value(&result).unwrap_or_default());
                                        
                                        let _ = trace_collector.add_trace_step(trace_id, step).await;
                                    }
                                    
                                    ToolResult {
                                        call_id: call.id.clone(),
                                        name: call.name.clone(),
                                        result,
                                        status: ToolResultStatus::Success,
                                    }
                                },
                                Err(e) => {
                                    let execution_time = tool_start_time.elapsed();
                                    total_errors += 1;
                                    if let Some(ref mut metrics) = agent_metrics {
                                        metrics.record_error();
                                    }
                                    
                                    self.logger().error(&format!("Function call '{}' failed after {:?}: {}", call.name, execution_time, e), None);
                                    
                                    // Record failed tool metrics
                                    if let Some(metrics_collector) = &self.metrics_collector {
                                        let tool_metrics = crate::telemetry::ToolMetrics {
                                            tool_name: call.name.clone(),
                                            execution_time_ms: execution_time.as_millis() as u64,
                                            success: false,
                                            error: Some(e.to_string()),
                                            input_size_bytes: serde_json::to_string(&call.arguments).unwrap_or_default().len(),
                                            output_size_bytes: 0,
                                            timestamp: SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap_or_else(|_| std::time::Duration::from_millis(0))
                                                .as_millis() as u64,
                                        };
                                        
                                        let _ = metrics_collector.record_tool_execution(tool_metrics).await;
                                    }
                                    
                                    // Record failed tool call in trace
                                    if let (Some(trace_collector), Some(trace_id)) = (&self.trace_collector, &trace_id) {
                                        let mut step = TraceStep::new(
                                            format!("Tool call: {} - Failed", call.name),
                                            TraceStepType::ToolCall,
                                        );
                                        step.metadata.insert("tool_name".to_string(), serde_json::Value::String(call.name.clone()));
                                        step.metadata.insert("success".to_string(), serde_json::Value::Bool(false));
                                        step.metadata.insert("error".to_string(), serde_json::Value::String(e.to_string()));
                                        step.duration_ms = execution_time.as_millis() as u64;
                                        step.success = false;
                                        step.error = Some(e.to_string());
                                        step.input = Some(serde_json::to_value(&call.arguments).unwrap_or_default());
                                        
                                        let _ = trace_collector.add_trace_step(trace_id, step).await;
                                    }
                                    
                                    ToolResult {
                                        call_id: call.id.clone(),
                                        name: call.name.clone(),
                                        result: Value::String(format!("Error: {}", e)),
                                        status: ToolResultStatus::Error,
                                    }
                                }
                            };
                            
                            tool_results.push(result);
                        }
                        
                        // Add assistant message with tool calls to conversation
                        let mut assistant_metadata = HashMap::new();

                        // Convert function calls to tool_calls format for the message
                        if !response.function_calls.is_empty() {
                            let tool_calls_json: Vec<serde_json::Value> = response.function_calls
                                .iter()
                                .map(|fc| {
                                    serde_json::json!({
                                        "id": fc.id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                                        "type": "function",
                                        "function": {
                                            "name": fc.name,
                                            "arguments": fc.arguments
                                        }
                                    })
                                })
                                .collect();
                            assistant_metadata.insert("tool_calls".to_string(), serde_json::Value::Array(tool_calls_json));
                        }

                        let assistant_msg = Message {
                            role: Role::Assistant,
                            content: response.content.clone().unwrap_or_default(),
                            metadata: if assistant_metadata.is_empty() { None } else { Some(assistant_metadata) },
                            name: None,
                        };
                        all_messages.push(assistant_msg);

                        // Add tool result messages
                        for result in &tool_results {
                            // Create tool message with proper metadata for function calling
                            let mut tool_msg = tool_message(&result.result.to_string());

                            // Add tool_call_id to metadata for DeepSeek/OpenAI compatibility
                            let mut metadata = HashMap::new();
                            metadata.insert("tool_call_id".to_string(), serde_json::Value::String(result.call_id.clone()));
                            tool_msg.metadata = Some(metadata);

                            all_messages.push(tool_msg);
                        }
                        
                        let step = AgentStep {
                            id: Uuid::new_v4().to_string(),
                            step_type: StepType::Tool,
                            input: all_messages.clone(),
                            output: Some(Message {
                                role: Role::Assistant,
                                content: response.content.clone().unwrap_or_default(),
                                metadata: None,
                                name: None,
                            }),
                            tool_calls: tool_calls.clone(),
                            tool_results: tool_results,
                            metadata: HashMap::new(),
                        };
                        steps.push(step);
                        
                        // Continue to next iteration to get final response
                        continue;
                    } else {
                        // No function calls, this is the final response
                        final_response = response.content.unwrap_or_default();
                        break;
                    }
                } else {
                    // No tools available, generate normally
                    let response = call_llm_with_monitoring(
                        self.llm.as_ref(),
                        &self.trace_collector,
                        &all_messages,
                        &options.llm_options,
                        &trace_id,
                        "Final LLM call (no tools)",
                        &mut agent_metrics,
                    ).await?;
                    
                    final_response = response;
                    break;
                }
            } else {
                // Use legacy regex-based tool calling
                let response = self.llm.generate_with_messages(&all_messages, &options.llm_options).await?;
                
                // Note: generate_with_messages returns String, no usage info available
                
                // Record legacy LLM call in trace
                if let (Some(trace_collector), Some(trace_id)) = (&self.trace_collector, &trace_id) {
                    let mut llm_step = TraceStep::new(
                        format!("Legacy LLM call - step {}", current_step),
                        TraceStepType::LlmCall,
                    );
                    llm_step.duration_ms = step_start_time.elapsed().as_millis() as u64;
                    llm_step.success = true;
                    llm_step.metadata.insert("mode".to_string(), serde_json::Value::String("legacy_regex".to_string()));
                    llm_step.metadata.insert("response_length".to_string(), serde_json::Value::Number(serde_json::Number::from(response.len())));
                    llm_step.output = Some(serde_json::Value::String(response.clone()));
                    
                    let _ = trace_collector.add_trace_step(trace_id, llm_step).await;
                }
                
                let tool_calls = self.parse_tool_calls(&response)?;
                
                if !tool_calls.is_empty() {
                    // Execute tools found in response text
                    let mut tool_results = Vec::new();
                    total_tool_calls += tool_calls.len();
                    if let Some(ref mut metrics) = agent_metrics {
                        metrics.tool_calls_count += tool_calls.len();
                    }
                    
                    for call in &tool_calls {
                        let tool_start_time = std::time::Instant::now();
                        
                        let result = match self.execute_tool_call(call).await {
                            Ok(result) => {
                                let execution_time = tool_start_time.elapsed();
                                
                                // Record successful legacy tool metrics
                                if let Some(metrics_collector) = &self.metrics_collector {
                                    let tool_metrics = crate::telemetry::ToolMetrics {
                                        tool_name: call.name.clone(),
                                        execution_time_ms: execution_time.as_millis() as u64,
                                        success: true,
                                        error: None,
                                        input_size_bytes: serde_json::to_string(&call.arguments).unwrap_or_default().len(),
                                        output_size_bytes: serde_json::to_string(&result).unwrap_or_default().len(),
                                        timestamp: SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .unwrap_or_else(|_| std::time::Duration::from_millis(0))
                                            .as_millis() as u64,
                                    };
                                    
                                    let _ = metrics_collector.record_tool_execution(tool_metrics).await;
                                }
                                
                                // Record legacy tool call in trace
                                if let (Some(trace_collector), Some(trace_id)) = (&self.trace_collector, &trace_id) {
                                    let mut step = TraceStep::new(
                                        format!("Legacy tool call: {} - Success", call.name),
                                        TraceStepType::ToolCall,
                                    );
                                    step.metadata.insert("mode".to_string(), serde_json::Value::String("legacy_regex".to_string()));
                                    step.metadata.insert("tool_name".to_string(), serde_json::Value::String(call.name.clone()));
                                    step.metadata.insert("success".to_string(), serde_json::Value::Bool(true));
                                    step.duration_ms = execution_time.as_millis() as u64;
                                    step.success = true;
                                    step.input = Some(serde_json::to_value(&call.arguments).unwrap_or_default());
                                    step.output = Some(serde_json::to_value(&result).unwrap_or_default());
                                    
                                    let _ = trace_collector.add_trace_step(trace_id, step).await;
                                }
                                
                                ToolResult {
                                    call_id: call.id.clone(),
                                    name: call.name.clone(),
                                    result,
                                    status: ToolResultStatus::Success,
                                }
                            },
                            Err(e) => {
                                let execution_time = tool_start_time.elapsed();
                                total_errors += 1;
                                if let Some(ref mut metrics) = agent_metrics {
                                    metrics.record_error();
                                }
                                
                                self.logger().error(&format!("Function call '{}' failed after {:?}: {}", call.name, execution_time, e), None);
                                
                                // Record failed legacy tool metrics
                                if let Some(metrics_collector) = &self.metrics_collector {
                                    let tool_metrics = crate::telemetry::ToolMetrics {
                                        tool_name: call.name.clone(),
                                        execution_time_ms: execution_time.as_millis() as u64,
                                        success: false,
                                        error: Some(e.to_string()),
                                        input_size_bytes: serde_json::to_string(&call.arguments).unwrap_or_default().len(),
                                        output_size_bytes: 0,
                                        timestamp: SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .unwrap_or_else(|_| std::time::Duration::from_millis(0))
                                            .as_millis() as u64,
                                    };
                                    
                                    let _ = metrics_collector.record_tool_execution(tool_metrics).await;
                                }
                                
                                // Record failed legacy tool call in trace
                                if let (Some(trace_collector), Some(trace_id)) = (&self.trace_collector, &trace_id) {
                                    let mut step = TraceStep::new(
                                        format!("Legacy tool call: {} - Failed", call.name),
                                        TraceStepType::ToolCall,
                                    );
                                    step.metadata.insert("mode".to_string(), serde_json::Value::String("legacy_regex".to_string()));
                                    step.metadata.insert("tool_name".to_string(), serde_json::Value::String(call.name.clone()));
                                    step.metadata.insert("success".to_string(), serde_json::Value::Bool(false));
                                    step.metadata.insert("error".to_string(), serde_json::Value::String(e.to_string()));
                                    step.duration_ms = execution_time.as_millis() as u64;
                                    step.success = false;
                                    step.error = Some(e.to_string());
                                    step.input = Some(serde_json::to_value(&call.arguments).unwrap_or_default());
                                    
                                    let _ = trace_collector.add_trace_step(trace_id, step).await;
                                }
                                
                                ToolResult {
                                    call_id: call.id.clone(),
                                    name: call.name.clone(),
                                    result: Value::String(format!("Error: {}", e)),
                                    status: ToolResultStatus::Error,
                                }
                            }
                        };
                        
                        tool_results.push(result);
                    }
                    
                    // Format tool results and add to messages
                    let mut updated_response = response.clone();
                    for (call, result) in tool_calls.iter().zip(tool_results.iter()) {
                        let result_text = match &result.status {
                            ToolResultStatus::Success => format!("结果: {}", result.result),
                            ToolResultStatus::Error => format!("错误: {}", result.result),
                        };
                        
                        // Replace the tool call in the response with the result
                        let tool_pattern = format!(r"工具: {}\s*参数: [^\n]*", regex::escape(&call.name));
                        if let Ok(re) = Regex::new(&tool_pattern) {
                            updated_response = re.replace(&updated_response, &format!("工具: {}\n参数: {}\n{}", 
                                call.name, 
                                serde_json::to_string(&call.arguments).unwrap_or_else(|_| "{}".to_string()),
                                result_text
                            )).to_string();
                        }
                    }
                    
                    all_messages.push(Message {
                        role: Role::Assistant,
                        content: updated_response.clone(),
                        metadata: None,
                        name: None,
                    });
                    
                    let step = AgentStep {
                        id: Uuid::new_v4().to_string(),
                        step_type: StepType::Tool,
                        input: all_messages.clone(),
                        output: Some(Message {
                            role: Role::Assistant,
                            content: updated_response.clone(),
                            metadata: None,
                            name: None,
                        }),
                        tool_calls,
                        tool_results,
                        metadata: HashMap::new(),
                    };
                    steps.push(step);
                    
                    final_response = updated_response;
                } else {
                    // No tool calls found, this is the final response
                    final_response = response;
                    break;
                }
            }
        }
        
        // Calculate total execution time
        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::SystemTime(format!("Failed to get end time: {}", e)))?
            .as_millis() as u64;
        let total_execution_time = end_time - start_time;
        
        // Finalize agent metrics
        if let Some(mut metrics) = agent_metrics {
            metrics.end_timing();
            metrics.set_token_usage(total_tokens.clone());
            metrics.set_success(total_errors == 0);
            
            // Add custom metrics
            metrics.add_custom_metric("total_steps".to_string(), crate::telemetry::MetricValue::Integer(current_step as i64));
            metrics.add_custom_metric("function_calling_mode".to_string(), crate::telemetry::MetricValue::Boolean(use_function_calling));
            metrics.add_custom_metric("response_length".to_string(), crate::telemetry::MetricValue::Integer(final_response.len() as i64));
            
            // Record the agent execution metrics
            if let Some(metrics_collector) = &self.metrics_collector {
                if let Err(e) = metrics_collector.record_agent_execution(metrics).await {
                    self.logger().warn(&format!("Failed to record agent metrics: {}", e), None);
                }
            }
        }
        
        // Complete execution trace
        if let (Some(trace_collector), Some(trace_id)) = (&self.trace_collector, &trace_id) {
            // Add final completion step
            let mut completion_step = TraceStep::new(
                "Agent execution completed".to_string(),
                TraceStepType::DataProcessing,
            );
            completion_step.metadata.insert("total_steps".to_string(), serde_json::Value::Number(serde_json::Number::from(current_step)));
            completion_step.metadata.insert("total_tool_calls".to_string(), serde_json::Value::Number(serde_json::Number::from(total_tool_calls)));
            completion_step.metadata.insert("total_errors".to_string(), serde_json::Value::Number(serde_json::Number::from(total_errors)));
            completion_step.metadata.insert("execution_time_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(total_execution_time)));
            completion_step.success = total_errors == 0;
            completion_step.duration_ms = 0;
            completion_step.output = Some(serde_json::json!({
                "response": final_response,
                "response_length": final_response.len(),
                "success": total_errors == 0
            }));
            
            let _ = trace_collector.add_trace_step(&trace_id, completion_step).await;
            
            // End the trace
            if let Err(e) = trace_collector.end_trace(&trace_id, total_errors == 0).await {
                self.logger().warn(&format!("Failed to end trace: {}", e), None);
            } else {
                self.logger().debug(&format!("Completed execution trace: {}", trace_id), None);
            }
        }
        
        // Create final step
        let final_step = AgentStep {
            id: Uuid::new_v4().to_string(),
            step_type: StepType::Final,
            input: all_messages.clone(),
            output: Some(Message {
                role: Role::Assistant,
                content: final_response.clone(),
                metadata: None,
                name: None,
            }),
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
            metadata: HashMap::new(),
        };
        steps.push(final_step);
        
        self.logger().info(&format!("Agent '{}' completed execution in {}ms with {} steps, {} tool calls, {} errors", 
            self.name, total_execution_time, current_step, total_tool_calls, total_errors), None);
        
        Ok(AgentGenerateResult {
            response: final_response,
            steps,
            usage: TokenUsage {
                prompt_tokens: total_tokens.prompt_tokens as usize,
                completion_tokens: total_tokens.completion_tokens as usize,
                total_tokens: total_tokens.total_tokens as usize,
            },
            metadata: HashMap::new(),
        })
    }
    
    async fn stream<'a>(&'a self, 
        messages: &'a [Message], 
        options: &'a AgentStreamOptions
    ) -> Result<BoxStream<'a, Result<String>>> {

        
        let _stream_start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::SystemTime(format!("Failed to get stream start time: {}", e)))?
            .as_millis() as u64;
        
        let run_id = options.run_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        
        self.logger().info(&format!("Starting enhanced streaming generation (run_id: {})", run_id), None);
        
        // Use legacy streaming mode for now
        // TODO: Implement advanced streaming
        
        // Legacy mode fallback - simplified implementation
        let run_id = options.run_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());

        // Generate complete response first
        let result = self.generate_with_steps(messages, &AgentGenerateOptions {
            system_message: None,
            instructions: options.instructions.clone(),
            context: options.context.clone(),
            memory_options: options.memory_options.clone(),
            thread_id: options.thread_id.clone(),
            resource_id: options.resource_id.clone(),
            run_id: Some(run_id.clone()),
            max_steps: options.max_steps,
            tool_choice: options.tool_choice.clone(),
            context_window: None,
            llm_options: options.llm_options.clone(),
            ..Default::default()
        }, options.max_steps).await?;

        // Create improved streaming experience with smart chunking
        let response_chunks = self.create_smart_chunks(&result.response);

        let stream = futures::stream::iter(response_chunks)
            .map(|chunk| Ok(chunk))
            .boxed();

        Ok(stream)
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
        let generate_result = self.generate_with_steps(messages, &AgentGenerateOptions {
            system_message: None,
            instructions: options.instructions.clone(),
            context: options.context.clone(),
            memory_options: options.memory_options.clone(),
            thread_id: options.thread_id.clone(),
            resource_id: options.resource_id.clone(),
            run_id: options.run_id.clone(),
            max_steps: options.max_steps,
            tool_choice: options.tool_choice.clone(),
            context_window: None,
            llm_options: options.llm_options.clone(),
            ..Default::default()
        }, options.max_steps).await?;

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

        let stream = futures::stream::iter(chunks)
            .map(|chunk| Ok(chunk))
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

    fn get_working_memory(&self) -> Option<Arc<dyn WorkingMemory>> {
        // Convert Box to Arc by cloning the underlying data
        // This is a workaround for the type mismatch
        self.working_memory.as_ref().map(|_wm| {
            // Create a new Arc from the Box reference
            // Note: This is not ideal as it requires the WorkingMemory to be Clone
            // In a real implementation, we should store Arc<dyn WorkingMemory> directly
            Arc::new(crate::memory::BasicWorkingMemory::new(
                crate::memory::WorkingMemoryConfig {
                    enabled: true,
                    template: None,
                    content_type: Some("application/json".to_string()),
                    max_capacity: Some(1024),
                }
            )) as Arc<dyn WorkingMemory>
        })
    }

    /// Get the current status of the agent
    fn get_status(&self) -> AgentStatus {
        self.status.clone()
    }

    /// Set the status of the agent
    fn set_status(&mut self, status: AgentStatus) -> Result<()> {
        self.status = status;
        Ok(())
    }

    /// Reset the agent state
    async fn reset(&mut self) -> Result<()> {
        // Only clear memory if it exists
        if self.working_memory.is_some() {
            let _ = self.clear_memory().await; // Ignore errors
        }
        self.set_status(AgentStatus::Ready)?;
        Ok(())
    }
}

impl BasicAgent {
    /// Legacy mode fallback for LLMs that don't support streaming
    async fn stream_legacy_mode<'a>(&'a self,
        messages: &'a [Message],
        options: &'a AgentStreamOptions
    ) -> Result<BoxStream<'a, Result<String>>> {
        let run_id = options.run_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // Generate complete response first
        let result = self.generate_with_steps(messages, &AgentGenerateOptions {
            system_message: None,
            instructions: options.instructions.clone(),
            context: options.context.clone(),
            memory_options: options.memory_options.clone(),
            thread_id: options.thread_id.clone(),
            resource_id: options.resource_id.clone(),
            run_id: Some(run_id.clone()),
            max_steps: options.max_steps,
            tool_choice: options.tool_choice.clone(),
            context_window: None,
            llm_options: options.llm_options.clone(),
            ..Default::default()
        }, options.max_steps).await?;

        // Create streaming-like experience by chunking the response
        let response_chunks = result.response
            .chars()
            .collect::<Vec<_>>()
            .chunks(3)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<_>>();

        let stream = futures::stream::iter(response_chunks)
            .map(|chunk| Ok(chunk))
            .boxed();

        Ok(stream)
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
        let generate_result = self.generate_with_steps(messages, &AgentGenerateOptions {
            system_message: None,
            instructions: options.instructions.clone(),
            context: options.context.clone(),
            memory_options: options.memory_options.clone(),
            thread_id: options.thread_id.clone(),
            resource_id: options.resource_id.clone(),
            run_id: options.run_id.clone(),
            max_steps: options.max_steps,
            tool_choice: options.tool_choice.clone(),
            context_window: None,
            llm_options: options.llm_options.clone(),
            ..Default::default()
        }, options.max_steps).await?;
        
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
        
        let stream = futures::stream::iter(chunks)
            .map(|chunk| Ok(chunk))
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
            None => {
                // Gracefully handle uninitialized memory by returning None
                self.logger().warn("Working memory not initialized, returning None for key", None);
                Ok(None)
            }
        }
    }
    
    async fn set_memory_value(&self, key: &str, value: Value) -> Result<()> {
        match &self.working_memory {
            Some(wm) => wm.set_value(key, value).await,
            None => {
                // Log warning but don't fail - graceful degradation
                self.logger().warn("Working memory not initialized, cannot set value", None);
                Err(Error::Memory("Working memory not initialized. Please initialize working memory before setting values.".to_string()))
            }
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

    /// Internal generate method to avoid trait conflicts
    async fn generate_internal(&self, messages: &[Message], options: &AgentGenerateOptions) -> Result<AgentGenerateResult> {
        // Use the existing generate implementation
        self.generate_with_steps(messages, options, options.max_steps).await
    }

    /// Internal stream method to avoid trait conflicts
    async fn stream_internal<'a>(&'a self, messages: &'a [Message], options: &'a AgentStreamOptions) -> Result<BoxStream<'a, Result<String>>> {
        // Directly implement streaming logic to avoid recursion
        let stream_start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::SystemTime(format!("Failed to get stream start time: {}", e)))?
            .as_millis() as u64;

        let run_id = options.run_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());

        self.logger().info(&format!("Starting enhanced streaming generation (run_id: {})", run_id), None);

        // Generate complete response first
        let result = self.generate_with_steps(messages, &AgentGenerateOptions {
            system_message: None,
            instructions: options.instructions.clone(),
            context: options.context.clone(),
            memory_options: options.memory_options.clone(),
            thread_id: options.thread_id.clone(),
            resource_id: options.resource_id.clone(),
            run_id: Some(run_id.clone()),
            max_steps: options.max_steps,
            tool_choice: options.tool_choice.clone(),
            context_window: None,
            llm_options: options.llm_options.clone(),
            ..Default::default()
        }, options.max_steps).await?;

        // Create improved streaming experience with smart chunking
        let response_chunks = self.create_smart_chunks(&result.response);

        let stream = futures::stream::iter(response_chunks)
            .map(|chunk| Ok(chunk))
            .boxed();

        Ok(stream)
    }
}

impl BasicAgent {
    /// Create smart chunks that respect word and sentence boundaries
    fn create_smart_chunks(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let target_chunk_size = 50; // Characters per chunk

        for word in text.split_whitespace() {
            if current_chunk.len() + word.len() + 1 > target_chunk_size && !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
            }

            if !current_chunk.is_empty() {
                current_chunk.push(' ');
            }
            current_chunk.push_str(word);
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        // If no chunks were created, return the original text
        if chunks.is_empty() && !text.is_empty() {
            chunks.push(text.to_string());
        }

        chunks
    }
}

// Chain operations are temporarily disabled for BasicAgent
// due to the complexity of implementing Clone for all internal components

