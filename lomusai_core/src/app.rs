use crate::{Result, Error};
use crate::agent::Agent;
use crate::tool::Tool;
use std::collections::HashMap;
use std::sync::Arc;

/// Lomusai应用主类，用于整合代理、工具、RAG和MCP等组件
pub struct LumosApp {
    name: String,
    description: Option<String>,
    agents: HashMap<String, Arc<dyn Agent>>,
    tools: HashMap<String, Box<dyn Tool>>,
    default_agent: Option<String>,
}

impl LumosApp {
    /// 创建一个新的Lomusai应用实例
    pub fn new() -> Self {
        Self {
            name: "lomusai_app".to_string(),
            description: None,
            agents: HashMap::new(),
            tools: HashMap::new(),
            default_agent: None,
        }
    }
    
    /// 设置应用名称
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    
    /// 设置应用描述
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// 添加代理到应用
    pub fn add_agent<S: AsRef<str>, A: Agent + 'static>(&mut self, name: S, agent: A) {
        let name = name.as_ref().to_string();
        self.agents.insert(name.clone(), Arc::new(agent));
        
        // 如果是第一个添加的代理，设为默认代理
        if self.default_agent.is_none() {
            self.default_agent = Some(name);
        }
    }
    
    /// 添加工具到应用
    pub fn add_tool<S: AsRef<str>, T: Tool + 'static>(&mut self, name: S, tool: T) {
        self.tools.insert(name.as_ref().to_string(), Box::new(tool));
    }
    
    /// 添加RAG到应用
    pub fn add_rag<S: AsRef<str>, R>(&mut self, name: S, _rag: R) {
        // RAG实现将在后续版本中支持
        println!("添加RAG: {} (尚未实现)", name.as_ref());
    }
    
    /// 添加工作流到应用
    pub fn add_workflow<S: AsRef<str>, W>(&mut self, name: S, _workflow: W) {
        // 工作流实现将在后续版本中支持
        println!("添加工作流: {} (尚未实现)", name.as_ref());
    }
    
    /// 配置MCP客户端
    pub fn configure_mcp<E>(&mut self, _endpoints: E) {
        // MCP客户端配置将在后续版本中支持
        println!("配置MCP客户端 (尚未实现)");
    }
    
    /// 使用应用处理用户输入
    pub async fn run(&self, input: &str) -> Result<String> {
        let agent_name = match &self.default_agent {
            Some(name) => name,
            None => return Err(Error::RuntimeError("没有可用的代理".to_string())),
        };
        
        let agent = match self.agents.get(agent_name) {
            Some(agent) => agent,
            None => return Err(Error::RuntimeError(format!("找不到代理: {}", agent_name))),
        };
        
        agent.run(input).await
    }
    
    /// 使用指定代理处理用户输入
    pub async fn run_with_agent(&self, agent_name: &str, input: &str) -> Result<String> {
        let agent = match self.agents.get(agent_name) {
            Some(agent) => agent,
            None => return Err(Error::RuntimeError(format!("找不到代理: {}", agent_name))),
        };
        
        agent.run(input).await
    }
    
    /// 获取应用名称
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// 获取应用描述
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    /// 获取所有代理列表
    pub fn agents(&self) -> &HashMap<String, Arc<dyn Agent>> {
        &self.agents
    }
    
    /// 获取所有工具列表
    pub fn tools(&self) -> &HashMap<String, Box<dyn Tool>> {
        &self.tools
    }
}

impl Default for LumosApp {
    fn default() -> Self {
        Self::new()
    }
} 