//! Agent 链式操作 API
//! 
//! 实现 plan10.md 中提到的链式操作功能，支持流畅的对话流程管理。

use crate::{Result, Error};
use crate::agent::trait_def::Agent as AgentTrait;
use crate::agent::types::{AgentGenerateOptions, AgentGenerateResult};
use crate::llm::{Message, Role};
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// 对话步骤类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainStepType {
    /// 用户输入
    UserInput,
    /// Agent 响应
    AgentResponse,
    /// 系统消息
    SystemMessage,
    /// 工具调用
    ToolCall,
}

/// 链式操作中的单个步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStep {
    /// 步骤ID
    pub id: String,
    /// 步骤类型
    pub step_type: ChainStepType,
    /// 步骤内容
    pub content: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ChainStep {
    /// 创建新的链式步骤
    pub fn new(step_type: ChainStepType, content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            step_type,
            content,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Agent 链式操作上下文
#[derive(Debug, Clone)]
pub struct ChainContext {
    /// 对话历史
    pub messages: Vec<Message>,
    /// 操作步骤历史
    pub steps: Vec<ChainStep>,
    /// 上下文变量
    pub variables: HashMap<String, serde_json::Value>,
    /// 配置选项
    pub options: AgentGenerateOptions,
}

impl ChainContext {
    /// 创建新的链式上下文
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            steps: Vec::new(),
            variables: HashMap::new(),
            options: AgentGenerateOptions::default(),
        }
    }
    
    /// 添加消息到上下文
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }
    
    /// 添加步骤到历史
    pub fn add_step(&mut self, step: ChainStep) {
        self.steps.push(step);
    }
    
    /// 设置上下文变量
    pub fn set_variable(&mut self, key: String, value: serde_json::Value) {
        self.variables.insert(key, value);
    }
    
    /// 获取上下文变量
    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }
    
    /// 获取最后的用户消息
    pub fn last_user_message(&self) -> Option<&Message> {
        self.messages.iter().rev().find(|msg| msg.role == Role::User)
    }
    
    /// 获取最后的助手消息
    pub fn last_assistant_message(&self) -> Option<&Message> {
        self.messages.iter().rev().find(|msg| msg.role == Role::Assistant)
    }
    
    /// 清空上下文
    pub fn clear(&mut self) {
        self.messages.clear();
        self.steps.clear();
        self.variables.clear();
    }
}

impl Default for ChainContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent 链式操作构建器
pub struct AgentChain {
    /// 关联的 Agent
    agent: Arc<dyn AgentTrait>,
    /// 链式上下文
    context: ChainContext,
    /// 是否自动保存历史
    auto_save_history: bool,
}

impl AgentChain {
    /// 创建新的链式操作
    pub fn new(agent: Arc<dyn AgentTrait>) -> Self {
        Self {
            agent,
            context: ChainContext::new(),
            auto_save_history: true,
        }
    }
    
    /// 从现有上下文创建链式操作
    pub fn with_context(agent: Arc<dyn AgentTrait>, context: ChainContext) -> Self {
        Self {
            agent,
            context,
            auto_save_history: true,
        }
    }
    
    /// 设置是否自动保存历史
    pub fn auto_save_history(mut self, enabled: bool) -> Self {
        self.auto_save_history = enabled;
        self
    }
    
    /// 设置生成选项
    pub fn with_options(mut self, options: AgentGenerateOptions) -> Self {
        self.context.options = options;
        self
    }
    
    /// 添加系统消息
    pub fn system(mut self, message: impl Into<String>) -> Self {
        let content = message.into();
        let msg = Message {
            role: Role::System,
            content: content.clone(),
            metadata: None,
            name: None,
        };
        
        self.context.add_message(msg);
        
        if self.auto_save_history {
            let step = ChainStep::new(ChainStepType::SystemMessage, content);
            self.context.add_step(step);
        }
        
        self
    }
    
    /// 发送用户消息（不等待响应）
    pub fn say(mut self, message: impl Into<String>) -> Self {
        let content = message.into();
        let msg = Message {
            role: Role::User,
            content: content.clone(),
            metadata: None,
            name: None,
        };
        
        self.context.add_message(msg);
        
        if self.auto_save_history {
            let step = ChainStep::new(ChainStepType::UserInput, content);
            self.context.add_step(step);
        }
        
        self
    }
    
    /// 发送用户消息并等待 Agent 响应
    pub async fn ask(mut self, question: impl Into<String>) -> Result<ChainResponse> {
        let content = question.into();
        
        // 添加用户消息
        let user_msg = Message {
            role: Role::User,
            content: content.clone(),
            metadata: None,
            name: None,
        };
        self.context.add_message(user_msg);
        
        if self.auto_save_history {
            let step = ChainStep::new(ChainStepType::UserInput, content);
            self.context.add_step(step);
        }
        
        // 获取 Agent 响应
        let response = self.agent.generate(&self.context.messages, &self.context.options).await?;
        
        // 添加 Agent 响应到上下文
        let assistant_msg = Message {
            role: Role::Assistant,
            content: response.response.clone(),
            metadata: None,
            name: None,
        };
        self.context.add_message(assistant_msg);
        
        if self.auto_save_history {
            let step = ChainStep::new(ChainStepType::AgentResponse, response.response.clone());
            self.context.add_step(step);
        }
        
        Ok(ChainResponse {
            content: response.response.clone(),
            chain: self,
            full_response: response,
        })
    }
    
    /// 继续对话（基于当前上下文）
    pub async fn continue_conversation(mut self) -> Result<ChainResponse> {
        if self.context.messages.is_empty() {
            return Err(Error::Configuration("No messages in context to continue conversation".to_string()));
        }
        
        // 获取 Agent 响应
        let response = self.agent.generate(&self.context.messages, &self.context.options).await?;
        
        // 添加 Agent 响应到上下文
        let assistant_msg = Message {
            role: Role::Assistant,
            content: response.response.clone(),
            metadata: None,
            name: None,
        };
        self.context.add_message(assistant_msg);
        
        if self.auto_save_history {
            let step = ChainStep::new(ChainStepType::AgentResponse, response.response.clone());
            self.context.add_step(step);
        }
        
        Ok(ChainResponse {
            content: response.response.clone(),
            chain: self,
            full_response: response,
        })
    }
    
    /// 设置上下文变量
    pub fn set_variable(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.context.set_variable(key.into(), value);
        self
    }
    
    /// 获取上下文变量
    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.context.get_variable(key)
    }
    
    /// 获取对话历史
    pub fn get_messages(&self) -> &[Message] {
        &self.context.messages
    }
    
    /// 获取操作步骤历史
    pub fn get_steps(&self) -> &[ChainStep] {
        &self.context.steps
    }
    
    /// 获取上下文的克隆
    pub fn get_context(&self) -> ChainContext {
        self.context.clone()
    }
    
    /// 清空对话历史
    pub fn clear_history(mut self) -> Self {
        self.context.clear();
        self
    }
    
    /// 保存当前上下文到文件
    pub fn save_context(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.context.steps)
            .map_err(|e| Error::Configuration(format!("Failed to serialize context: {}", e)))?;
        
        std::fs::write(path, json)
            .map_err(|e| Error::Configuration(format!("Failed to write context file: {}", e)))?;
        
        Ok(())
    }
    
    /// 从文件加载上下文
    pub fn load_context(mut self, path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Configuration(format!("Failed to read context file: {}", e)))?;
        
        let steps: Vec<ChainStep> = serde_json::from_str(&content)
            .map_err(|e| Error::Configuration(format!("Failed to parse context file: {}", e)))?;
        
        // 重建消息历史
        self.context.messages.clear();
        for step in &steps {
            match step.step_type {
                ChainStepType::UserInput => {
                    let msg = Message {
                        role: Role::User,
                        content: step.content.clone(),
                        metadata: None,
                        name: None,
                    };
                    self.context.add_message(msg);
                }
                ChainStepType::AgentResponse => {
                    let msg = Message {
                        role: Role::Assistant,
                        content: step.content.clone(),
                        metadata: None,
                        name: None,
                    };
                    self.context.add_message(msg);
                }
                ChainStepType::SystemMessage => {
                    let msg = Message {
                        role: Role::System,
                        content: step.content.clone(),
                        metadata: None,
                        name: None,
                    };
                    self.context.add_message(msg);
                }
                _ => {} // 忽略其他类型
            }
        }
        
        self.context.steps = steps;
        Ok(self)
    }
}

/// 链式操作响应
pub struct ChainResponse {
    /// 响应内容
    pub content: String,
    /// 链式操作对象（用于继续对话）
    pub chain: AgentChain,
    /// 完整的 Agent 响应
    pub full_response: AgentGenerateResult,
}

impl ChainResponse {
    /// 继续对话
    pub async fn then_ask(self, question: impl Into<String>) -> Result<ChainResponse> {
        self.chain.ask(question).await
    }
    
    /// 添加用户消息
    pub fn then_say(self, message: impl Into<String>) -> AgentChain {
        self.chain.say(message)
    }
    
    /// 获取响应内容
    pub fn content(&self) -> &str {
        &self.content
    }
    
    /// 获取完整响应
    pub fn full_response(&self) -> &AgentGenerateResult {
        &self.full_response
    }
    
    /// 获取链式操作对象
    pub fn chain(self) -> AgentChain {
        self.chain
    }
}

/// 为 Agent trait 添加链式操作扩展
pub trait AgentChainExt {
    /// 开始链式操作
    fn chain(&self) -> AgentChain;

    /// 使用指定上下文开始链式操作
    fn chain_with_context(&self, context: ChainContext) -> AgentChain;
}

// For now, disable the generic implementation since BasicAgent cannot be cloned
// We'll need to implement this differently for specific agent types

// Specific implementation for BasicAgent will be added in executor.rs
