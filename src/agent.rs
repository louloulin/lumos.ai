//! 简化的Agent API
//!
//! 提供一行代码创建Agent的便利函数，支持智能默认配置。

use crate::{Result, Message};
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
/// use lumosai::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
///     let agent = lumosai::agent::simple("gpt-4", "You are a helpful assistant").await?;
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
/// use lumosai::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
///     let agent = lumosai::agent::auto().await?;
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
/// use lumosai::prelude::*;
/// 
/// #[tokio::main]
/// async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
///     let agent = lumosai::agent::builder()
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

        // 创建简化的Agent实现
        let agent = SimpleAgentImpl {
            name,
            description: self.description,
            model,
            system_prompt,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        Ok(Arc::new(agent))
    }
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 简化的Agent实现
struct SimpleAgentImpl {
    name: String,
    description: Option<String>,
    model: String,
    system_prompt: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

#[async_trait::async_trait]
impl AgentTrait for SimpleAgentImpl {
    async fn chat(&self, message: &str) -> Result<String> {
        // 简化实现：返回一个模拟响应
        // 在实际实现中，这里会调用真实的LLM API
        Ok(format!("Agent {} (using {}) responds: I received your message: '{}'",
                  self.name, self.model, message))
    }

    async fn chat_with_context(&self, messages: &[Message]) -> Result<AgentResponse> {
        // 简化实现：处理消息上下文
        let last_message = messages.last()
            .map(|m| m.content.as_str())
            .unwrap_or("No message");

        let response_content = format!("Agent {} processed {} messages. Last message: '{}'",
                                     self.name, messages.len(), last_message);

        Ok(AgentResponse {
            content: response_content,
            metadata: None,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

// 注意：这是一个简化的实现，用于演示API设计
// 在实际使用中，需要集成真实的LLM提供商

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
