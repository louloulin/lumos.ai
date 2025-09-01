//! Agent trait definition

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use futures::stream::BoxStream;
use serde_json::Value;
use serde::de::DeserializeOwned;

use crate::base::Base;
use crate::error::{Error, Result};
use crate::llm::{LlmProvider, Message};
use crate::memory::Memory;
use crate::memory::working::WorkingMemory;
use crate::tool::Tool;
use crate::agent::types::{
    AgentGenerateResult,
    AgentGenerateOptions,
    AgentStreamOptions,
    AgentStep,
    ToolCall,
    RuntimeContext,
};
use crate::voice::{VoiceProvider, VoiceOptions, ListenOptions};
use crate::workflow::Workflow;
use crate::agent::config::AgentConfig;
use tokio::io::AsyncRead;
use serde::{Serialize, Deserialize};

/// Agent状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// 初始化中
    Initializing,
    /// 就绪状态
    Ready,
    /// 运行中
    Running,
    /// 暂停状态
    Paused,
    /// 错误状态
    Error(String),
    /// 停止状态
    Stopped,
}

impl Default for AgentStatus {
    fn default() -> Self {
        Self::Initializing
    }
}

/// Trait for agents that support structured output generation
#[async_trait]
pub trait AgentStructuredOutput: Send + Sync {
    /// Generate structured output based on a schema
    async fn generate_structured<T: DeserializeOwned + Send + 'static>(
        &self, 
        messages: &[Message], 
        options: &AgentGenerateOptions
    ) -> Result<T>;
}

/// Trait for agents that support voice input (speech-to-text)
#[async_trait]
pub trait AgentVoiceListener: Send + Sync {
    /// Convert speech to text using the agent's voice provider
    async fn listen(&self, audio: impl AsyncRead + Send + Unpin + 'static, options: &ListenOptions) -> Result<String>;
}

/// Trait for agents that support voice output (text-to-speech)
#[async_trait]
pub trait AgentVoiceSender: Send + Sync {
    /// Convert text to speech using the agent's voice provider
    async fn speak(&self, text: &str, options: &VoiceOptions) -> Result<BoxStream<'_, Result<Vec<u8>>>>;
}

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

    /// Get the working memory for the agent, if configured
    fn get_working_memory(&self) -> Option<Arc<dyn WorkingMemory>> {
        None
    }

    /// Get all tools available to the agent
    fn get_tools(&self) -> HashMap<String, Box<dyn Tool>>;

    /// Get tools with runtime context for dynamic resolution
    async fn get_tools_with_context(&self, context: &RuntimeContext) -> Result<HashMap<String, Box<dyn Tool>>> {
        // Default implementation returns static tools
        Ok(self.get_tools())
    }

    /// Add a tool to the agent
    fn add_tool(&mut self, tool: Box<dyn Tool>) -> Result<()>;

    /// Remove a tool from the agent
    fn remove_tool(&mut self, tool_name: &str) -> Result<()>;

    /// Get a specific tool by name
    fn get_tool(&self, tool_name: &str) -> Option<Box<dyn Tool>>;

    /// Get available workflows for the agent
    async fn get_workflows(&self, context: &RuntimeContext) -> Result<HashMap<String, Arc<dyn Workflow>>> {
        // Default implementation returns empty workflows
        Ok(HashMap::new())
    }

    /// Execute a workflow by name
    async fn execute_workflow(&self, workflow_name: &str, input: Value, context: &RuntimeContext) -> Result<Value> {
        let workflows = self.get_workflows(context).await?;
        if let Some(workflow) = workflows.get(workflow_name) {
            workflow.execute(input, context).await
        } else {
            Err(Error::NotFound(format!("Workflow '{}' not found", workflow_name)))
        }
    }
    
    /// Parse the LLM response to extract tool calls
    fn parse_tool_calls(&self, response: &str) -> Result<Vec<ToolCall>>;
    
    /// Execute a tool call and return the result
    async fn execute_tool_call(&self, tool_call: &ToolCall) -> Result<Value>;
    
    /// Format messages for the LLM provider
    fn format_messages(&self, messages: &[Message], options: &AgentGenerateOptions) -> Vec<Message>;
    
    /// Generate a title for a conversation
    async fn generate_title(&self, user_message: &Message) -> Result<String>;
    
    /// Get instructions with runtime context for dynamic resolution
    async fn get_instructions_with_context(&self, context: &RuntimeContext) -> Result<String> {
        // Default implementation returns static instructions
        Ok(self.get_instructions().to_string())
    }

    /// Generate a response given a set of messages
    async fn generate(&self,
        messages: &[Message],
        options: &AgentGenerateOptions
    ) -> Result<AgentGenerateResult>;

    /// Generate with runtime context for dynamic resolution
    async fn generate_with_context(&self,
        messages: &[Message],
        options: &AgentGenerateOptions,
        context: &RuntimeContext
    ) -> Result<AgentGenerateResult> {
        // Default implementation ignores context
        self.generate(messages, options).await
    }

    /// Generate a simple response from a text input (convenience method for plan4.md API)
    async fn generate_simple(&self, input: &str) -> Result<String> {
        use crate::llm::{Message, Role};

        let message = Message {
            role: Role::User,
            content: input.to_string(),
            metadata: None,
            name: None,
        };

        let options = AgentGenerateOptions::default();
        let result = self.generate(&[message], &options).await?;

        Ok(result.response)
    }

    /// Generate with multi-step reasoning
    async fn generate_with_steps(&self,
        messages: &[Message],
        options: &AgentGenerateOptions,
        max_steps: Option<u32>
    ) -> Result<AgentGenerateResult> {
        // Default implementation uses single step
        self.generate(messages, options).await
    }
    
    /// Generate a response with memory thread integration
    async fn generate_with_memory(&self,
        messages: &[Message],
        thread_id: Option<String>,
        options: &AgentGenerateOptions
    ) -> Result<AgentGenerateResult>;
    
    /// Stream a response given a set of messages
    async fn stream<'a>(&'a self, 
        messages: &'a [Message], 
        options: &'a AgentStreamOptions
    ) -> Result<BoxStream<'a, Result<String>>>;

    /// Stream with callbacks for advanced control
    async fn stream_with_callbacks<'a>(
        &'a self, 
        messages: &'a [Message], 
        options: &'a AgentStreamOptions,
        on_step_finish: Option<Box<dyn FnMut(AgentStep) + Send + 'a>>,
        on_finish: Option<Box<dyn FnOnce(AgentGenerateResult) + Send + 'a>>
    ) -> Result<BoxStream<'a, Result<String>>>;

    /// Get the agent's voice provider if configured
    fn get_voice(&self) -> Option<Arc<dyn VoiceProvider>>;

    /// Set a voice provider for the agent
    fn set_voice(&mut self, voice: Arc<dyn VoiceProvider>);
    
    /// Get a value from working memory
    async fn get_memory_value(&self, key: &str) -> Result<Option<Value>> {
        if let Some(memory) = self.get_working_memory() {
            memory.get_value(key).await
        } else {
            Err(Error::Unsupported("Working memory not enabled for this agent".to_string()))
        }
    }
    
    /// Set a value in working memory
    async fn set_memory_value(&self, key: &str, value: Value) -> Result<()> {
        if let Some(memory) = self.get_working_memory() {
            memory.set_value(key, value).await
        } else {
            Err(Error::Unsupported("Working memory not enabled for this agent".to_string()))
        }
    }
    
    /// Delete a value from working memory
    async fn delete_memory_value(&self, key: &str) -> Result<()> {
        if let Some(memory) = self.get_working_memory() {
            memory.delete_value(key).await
        } else {
            Err(Error::Unsupported("Working memory not enabled for this agent".to_string()))
        }
    }
    
    /// Clear the working memory
    async fn clear_memory(&self) -> Result<()> {
        if let Some(memory) = self.get_working_memory() {
            memory.clear().await
        } else {
            Err(Error::Unsupported("Working memory not enabled for this agent".to_string()))
        }
    }

    // === 统一Agent接口扩展 ===

    /// 获取Agent的配置信息
    fn get_config(&self) -> &AgentConfig {
        // 默认实现，子类应该重写
        panic!("get_config not implemented")
    }

    /// 获取Agent的状态信息
    fn get_status(&self) -> AgentStatus {
        AgentStatus::Ready
    }

    /// 设置Agent的状态
    fn set_status(&mut self, _status: AgentStatus) -> Result<()> {
        Ok(())
    }

    /// 获取Agent的元数据
    fn get_metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// 设置Agent的元数据
    fn set_metadata(&mut self, _metadata: HashMap<String, String>) -> Result<()> {
        Ok(())
    }

    /// 验证Agent配置
    fn validate_config(&self) -> Result<()> {
        Ok(())
    }

    /// 重新加载Agent配置
    async fn reload_config(&mut self, _config: AgentConfig) -> Result<()> {
        Ok(())
    }

    /// 获取Agent的健康状态
    async fn health_check(&self) -> Result<HashMap<String, Value>> {
        let mut health = HashMap::new();
        health.insert("status".to_string(), serde_json::to_value(self.get_status())?);
        health.insert("name".to_string(), Value::String(self.get_name().to_string()));
        health.insert("has_memory".to_string(), Value::Bool(self.has_own_memory()));
        health.insert("tools_count".to_string(), Value::Number(self.get_tools().len().into()));
        Ok(health)
    }

    /// 获取Agent的性能指标
    async fn get_metrics(&self) -> Result<HashMap<String, Value>> {
        // 默认返回空指标，子类可以重写
        Ok(HashMap::new())
    }

    /// 重置Agent状态
    async fn reset(&mut self) -> Result<()> {
        self.clear_memory().await?;
        self.set_status(AgentStatus::Ready)?;
        Ok(())
    }
}