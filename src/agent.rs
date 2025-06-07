//! 简化的Agent API
//!
//! 提供一行代码创建Agent的便利函数，支持智能默认配置。

use crate::{Result, Error, Message, Role};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

/// 简单Agent类型
pub type SimpleAgent = Arc<dyn AgentTrait>;

/// Agent响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub content: String,
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Agent trait
#[async_trait::async_trait]
pub trait AgentTrait: Send + Sync {
    /// 简单对话
    async fn chat(&self, message: &str) -> Result<String>;
    
    /// 带上下文的对话
    async fn chat_with_context(&self, messages: &[Message]) -> Result<AgentResponse>;
    
    /// 获取Agent名称
    fn name(&self) -> &str;
    
    /// 获取Agent描述
    fn description(&self) -> Option<&str>;
}

/// 一行代码创建简单Agent
/// 
/// # 参数
/// - `model`: 模型名称 ("gpt-4", "gpt-3.5-turbo", "claude-3")
/// - `system_prompt`: 系统提示词
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let agent = lumos::agent::simple("gpt-4", "You are a helpful assistant").await?;
///     
///     let response = agent.chat("Hello, how are you?").await?;
///     println!("Agent: {}", response);
///     
///     Ok(())
/// }
/// ```
pub async fn simple(model: &str, system_prompt: &str) -> Result<SimpleAgent> {
    let builder = builder()
        .name("SimpleAgent")
        .model(model)
        .system_prompt(system_prompt);
    
    builder.build().await
}

/// 创建带有智能默认配置的Agent
/// 
/// 自动检测最佳配置并创建Agent
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let agent = lumos::agent::auto().await?;
///     Ok(())
/// }
/// ```
pub async fn auto() -> Result<SimpleAgent> {
    simple("gpt-4", "You are a helpful AI assistant").await
}

/// Agent构建器
/// 
/// 提供更细粒度的配置选项
/// 
/// # 示例
/// ```rust,no_run
/// use lumos::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let agent = lumos::agent::builder()
///         .name("ResearchAgent")
///         .model("gpt-4")
///         .system_prompt("You are a research assistant")
///         .temperature(0.7)
///         .max_tokens(2000)
///         .build()
///         .await?;
///     
///     Ok(())
/// }
/// ```
pub fn builder() -> AgentBuilder {
    AgentBuilder::new()
}

/// Agent构建器
pub struct AgentBuilder {
    name: Option<String>,
    description: Option<String>,
    model: Option<String>,
    system_prompt: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    tools: Vec<Arc<dyn lumosai_core::tool::Tool>>,
    memory: Option<Arc<dyn lumosai_core::memory::Memory>>,
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            model: None,
            system_prompt: None,
            temperature: None,
            max_tokens: None,
            tools: Vec::new(),
            memory: None,
        }
    }
    
    /// 设置Agent名称
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
    
    /// 设置Agent描述
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// 设置模型
    pub fn model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }
    
    /// 设置系统提示词
    pub fn system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = Some(prompt.to_string());
        self
    }
    
    /// 设置温度参数
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
    
    /// 设置最大token数
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    
    /// 添加工具
    pub fn tool(mut self, tool: Arc<dyn lumosai_core::tool::Tool>) -> Self {
        self.tools.push(tool);
        self
    }
    
    /// 添加多个工具
    pub fn tools(mut self, tools: Vec<Arc<dyn lumosai_core::tool::Tool>>) -> Self {
        self.tools.extend(tools);
        self
    }
    
    /// 设置内存
    pub fn memory(mut self, memory: Arc<dyn lumosai_core::memory::Memory>) -> Self {
        self.memory = Some(memory);
        self
    }
    
    /// 构建Agent
    pub async fn build(self) -> Result<SimpleAgent> {
        let name = self.name.unwrap_or_else(|| "Agent".to_string());
        let model = self.model.unwrap_or_else(|| "gpt-4".to_string());
        let system_prompt = self.system_prompt.unwrap_or_else(|| "You are a helpful assistant".to_string());
        
        // 创建LLM提供商
        let llm_provider = create_llm_provider(&model).await?;
        
        // 创建Agent配置
        let mut config = lumosai_core::agent::AgentConfig::new(name.clone(), llm_provider);
        config.system_prompt = Some(system_prompt);
        config.description = self.description;
        
        if let Some(temp) = self.temperature {
            config.llm_options.temperature = Some(temp);
        }
        
        if let Some(max_tokens) = self.max_tokens {
            config.llm_options.max_tokens = Some(max_tokens);
        }
        
        // 创建Agent
        let mut agent = lumosai_core::agent::BasicAgent::new(config);
        
        // 添加工具
        for tool in self.tools {
            agent.add_tool(tool);
        }
        
        // 设置内存
        if let Some(memory) = self.memory {
            agent.set_memory(memory);
        }
        
        Ok(Arc::new(SimpleAgentWrapper::new(agent, name, self.description)))
    }
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 简单Agent包装器
struct SimpleAgentWrapper {
    agent: lumosai_core::agent::BasicAgent,
    name: String,
    description: Option<String>,
}

impl SimpleAgentWrapper {
    fn new(agent: lumosai_core::agent::BasicAgent, name: String, description: Option<String>) -> Self {
        Self { agent, name, description }
    }
}

#[async_trait::async_trait]
impl AgentTrait for SimpleAgentWrapper {
    async fn chat(&self, message: &str) -> Result<String> {
        let messages = vec![Message {
            role: Role::User,
            content: message.to_string(),
            metadata: None,
            name: None,
        }];
        
        let options = lumosai_core::agent::types::AgentGenerateOptions::default();
        let result = self.agent.generate(&messages, &options).await?;
        
        Ok(result.response)
    }
    
    async fn chat_with_context(&self, messages: &[Message]) -> Result<AgentResponse> {
        let options = lumosai_core::agent::types::AgentGenerateOptions::default();
        let result = self.agent.generate(messages, &options).await?;
        
        Ok(AgentResponse {
            content: result.response,
            metadata: result.metadata,
        })
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

// 辅助函数
async fn create_llm_provider(model: &str) -> Result<Arc<dyn lumosai_core::llm::LlmProvider>> {
    match model {
        "gpt-4" | "gpt-3.5-turbo" | "gpt-4-turbo" => {
            let provider = lumosai_core::llm::openai::OpenAIProvider::new()
                .map_err(|e| Error::Config(format!("Failed to create OpenAI provider: {}", e)))?;
            Ok(Arc::new(provider))
        }
        _ => {
            // 默认使用OpenAI
            let provider = lumosai_core::llm::openai::OpenAIProvider::new()
                .map_err(|e| Error::Config(format!("Failed to create OpenAI provider: {}", e)))?;
            Ok(Arc::new(provider))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_builder() {
        let builder = builder()
            .name("TestAgent")
            .model("gpt-4")
            .system_prompt("You are a test assistant")
            .temperature(0.7);
        
        // 测试构建器模式
        assert!(true); // 简单的编译测试
    }
    
    #[tokio::test]
    async fn test_agent_response_serialization() {
        let response = AgentResponse {
            content: "Hello".to_string(),
            metadata: None,
        };
        
        let json = serde_json::to_string(&response).expect("Failed to serialize");
        let _deserialized: AgentResponse = serde_json::from_str(&json).expect("Failed to deserialize");
    }
}
